//! Grid tool (T18): drag a box, get a `cols × rows`
//! [`Shape::Grid`].
//!
//! The drag works like the rectangle tool: `Down` starts it, `Move` updates
//! the end, `Up` commits one grid as one undo step unless the drag is
//! shorter than [`MIN_DRAG_PX`] on screen. Corners snap like a box drag
//! (ADR-T17-1).
//!
//! The dimensions are never stored in the drag: [`preview`] and the commit
//! read them from [`Helpers`] in
//! [`ToolView::style`], so the arrow keys change them live and the grid
//! gets the values at release (ADR-T18-1). `Shift` makes the cells square.
//! With numbering on, new grids show their axis indices (ADR-T22-1).

use crate::core::command::Tool;
use crate::core::document::tx_insert;
use crate::core::editor::Helpers;
use crate::core::geom::Vec2;
use crate::core::shape::{CellText, GRID_MAX_CELLS, SHAPE_TEXT_FILL, Shape, ShapeText, Style};
use crate::core::snap::{self, DragKind, Snapped};
use crate::core::snapshot::{Reader, SnapshotError, Writer};
use crate::core::text_layout;
use crate::core::tools::shape_tool::MIN_DRAG_PX;
use crate::core::tools::text::PASTE_MAX_BYTES;
use crate::core::tools::{Overlay, Phase, Pointer, ToolCtx, ToolView};
use std::fmt;

/// Gesture state of the grid tool.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    /// The drag in progress, if the button is down.
    drag: Option<Drag>,
}

/// A grid drag in progress.
#[derive(Debug, Clone, PartialEq)]
struct Drag {
    /// World position of the `Down`, snapped.
    start: Vec2,
    /// World position of the latest event, snapped.
    end: Vec2,
    /// Alignment guides of the latest event.
    guides: Vec<[Vec2; 2]>,
    /// Whether `Shift` was held on the latest event.
    shift: bool,
    /// Style sampled on `Down`, width already in world units.
    style: Style,
}

impl Drag {
    /// The grid this drag spans with the dimensions of `helpers` and axis
    /// indices if numbering is on (dimensions clamped to `1..=GRID_MAX_CELLS`), with square
    /// cells if `shift` is held.
    fn shape(&self, helpers: &Helpers) -> Shape {
        let cols = helpers.grid_cols.clamp(1, GRID_MAX_CELLS);
        let rows = helpers.grid_rows.clamp(1, GRID_MAX_CELLS);
        let a = self.start;
        let b = if self.shift {
            square_cells(a, self.end, cols, rows)
        } else {
            self.end
        };
        Shape::Grid {
            a,
            b,
            cols,
            rows,
            style: self.style,
            // Numbered grids show their indices (ADR-T22-1).
            axes: helpers.numbering,
            fills: Vec::new(),
            texts: Vec::new(),
        }
    }
}

/// Why a text cannot become a table (ADR-T84-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableError {
    /// More than [`GRID_MAX_CELLS`] columns or rows.
    TooManyCells {
        /// Columns of the widest line.
        cols: usize,
        /// Lines that are not blank.
        rows: usize,
    },
    /// More than [`PASTE_MAX_BYTES`] of text.
    TooLong {
        /// Bytes of the text.
        bytes: usize,
    },
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyCells { cols, rows } => write!(
                f,
                "{cols} columns and {rows} rows, a grid holds at most {GRID_MAX_CELLS} of each"
            ),
            Self::TooLong { bytes } => {
                write!(f, "{bytes} bytes of text, the limit is {PASTE_MAX_BYTES}")
            }
        }
    }
}

impl std::error::Error for TableError {}

/// A text read as a table: lines are rows, whitespace separates cells
/// (ADR-T84-1). Has at least one cell, and fits a [`Shape::Grid`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextTable {
    /// Cell tokens by row; no row is empty.
    cells: Vec<Vec<String>>,
    /// Length of the longest row, in `1..=GRID_MAX_CELLS`.
    cols: u32,
    /// Number of rows, in `1..=GRID_MAX_CELLS`.
    rows: u32,
}

impl TextTable {
    /// The table `text` holds; `Ok(None)` when it holds no cell.
    ///
    /// # Errors
    ///
    /// [`TableError`] if the table would not fit a grid or the text is
    /// over [`PASTE_MAX_BYTES`]; the text is never cut to fit.
    pub fn parse(text: &str) -> Result<Option<Self>, TableError> {
        if text.len() > PASTE_MAX_BYTES {
            return Err(TableError::TooLong { bytes: text.len() });
        }
        let cells: Vec<Vec<String>> = text
            .split(['\n', '\r'])
            .map(line_tokens)
            .filter(|row| !row.is_empty())
            .collect();
        let widest = cells.iter().map(Vec::len).max().unwrap_or(0);
        let too_many = TableError::TooManyCells {
            cols: widest,
            rows: cells.len(),
        };
        let (Ok(cols), Ok(rows)) = (u32::try_from(widest), u32::try_from(cells.len())) else {
            return Err(too_many);
        };
        if cols > GRID_MAX_CELLS || rows > GRID_MAX_CELLS {
            return Err(too_many);
        }
        Ok((cols > 0).then_some(Self { cells, cols, rows }))
    }

    /// Number of columns.
    #[must_use]
    pub fn cols(&self) -> u32 {
        self.cols
    }

    /// Number of rows.
    #[must_use]
    pub fn rows(&self) -> u32 {
        self.rows
    }

    /// The grid with corner `corner` showing this table at `text_size`:
    /// its cells take the longest token at full size. `None` if the size
    /// is not finite and positive or the grid would not be finite.
    #[must_use]
    pub fn shape(&self, corner: Vec2, text_size: f32, style: Style, axes: bool) -> Option<Shape> {
        if !text_size.is_finite() || text_size <= 0.0 {
            return None;
        }
        // Starts at one line: an empty formula has no height of its own.
        let (mut width, mut height) = (0.0_f32, text_layout::LINE_HEIGHT * text_size);
        let mut texts = Vec::new();
        for (row, line) in (0_u32..).zip(&self.cells) {
            for (col, token) in (0_u32..).zip(line) {
                let token_box = text_layout::bounds(Vec2::ZERO, token, text_size);
                width = width.max(token_box.width());
                height = height.max(token_box.height());
                let content = ShapeText::new(token.clone(), text_size)?;
                texts.push(CellText { col, row, content });
            }
        }
        // Text takes SHAPE_TEXT_FILL of a cell; digits get square cells.
        let cell = Vec2::new(width.max(height), height) * SHAPE_TEXT_FILL.recip();
        let shape = Shape::Grid {
            a: corner,
            b: corner + Vec2::new(cell.x * self.cols as f32, cell.y * self.rows as f32),
            cols: self.cols,
            rows: self.rows,
            style,
            // Numbered grids show their indices (ADR-T22-1).
            axes,
            fills: Vec::new(),
            texts,
        };
        shape.is_finite().then_some(shape)
    }
}

/// The cell tokens of one line: runs of whitespace separate them, control
/// characters inside one are dropped.
fn line_tokens(line: &str) -> Vec<String> {
    line.split_whitespace()
        .map(|token| token.chars().filter(|ch| !ch.is_control()).collect())
        .filter(|token: &String| !token.is_empty())
        .collect()
}

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
///
/// `Down` starts a drag, `Move` updates its end and `Shift` state, `Up`
/// commits the grid with the current dimensions as one undo step unless the
/// drag is shorter than [`MIN_DRAG_PX`] on screen. Events without a drag
/// are ignored.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    let world = ctx.camera.screen_to_world(pointer.pos);
    let shift = pointer.mods.shift;
    match pointer.phase {
        Phase::Down => {
            if ctx.tool != Tool::Grid {
                return cancel(state);
            }
            let start = snap::snap_start(world, pointer.mods, &ctx.view());
            state.drag = Some(Drag {
                start,
                end: start,
                guides: Vec::new(),
                shift,
                style: Style {
                    color: ctx.style.color,
                    width: ctx.camera.world_len(ctx.style.width_px),
                },
            });
            true
        }
        Phase::Move => match &mut state.drag {
            Some(drag) => {
                let Snapped { point, guides } =
                    snap::snap_end(drag.start, world, DragKind::Box, pointer.mods, &ctx.view());
                let changed = drag.end != point || drag.shift != shift || drag.guides != guides;
                drag.end = point;
                drag.guides = guides;
                drag.shift = shift;
                changed
            }
            None => false,
        },
        Phase::Up => {
            let Some(mut drag) = state.drag.take() else {
                return false;
            };
            drag.end =
                snap::snap_end(drag.start, world, DragKind::Box, pointer.mods, &ctx.view()).point;
            drag.shift = shift;
            let drag_px = ctx.camera.screen_len(drag.start.distance(drag.end));
            if drag_px.is_nan() || drag_px < MIN_DRAG_PX {
                return true;
            }
            let shape = drag.shape(&ctx.style.helpers);
            if shape.is_finite() {
                let tx = tx_insert(ctx.doc, [shape]);
                ctx.commit(tx);
            }
            true
        }
    }
}

/// What the gesture in progress draws on top of the document: the grid
/// being dragged, with the current dimensions, and its alignment guides;
/// nothing when idle.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    let Some(drag) = &state.drag else {
        return Overlay::default();
    };
    Overlay {
        shapes: vec![drag.shape(&view.style.helpers)],
        guides: drag.guides.clone(),
        ..Overlay::default()
    }
}

/// Discards the gesture in progress without changing the document. Returns
/// whether a redraw is needed (there was a drag to discard).
pub fn cancel(state: &mut State) -> bool {
    state.drag.take().is_some()
}

/// `end` moved so that the box from `start` holds `cols × rows` square
/// cells: the side is the larger of the dragged cell width and height.
/// The drag direction is kept; a non-finite result leaves `end` unchanged.
fn square_cells(start: Vec2, end: Vec2, cols: u32, rows: u32) -> Vec2 {
    let (cols, rows) = (cols as f32, rows as f32);
    let d = end - start;
    let side = (d.x.abs() / cols).max(d.y.abs() / rows);
    let squared = start + Vec2::new((side * cols).copysign(d.x), (side * rows).copysign(d.y));
    if squared.is_finite() { squared } else { end }
}

impl State {
    /// `grid -` or `grid <start> <end> <shift> <style> <guides>`
    /// (snapshots, ADR-T74-1).
    pub(crate) fn write_snapshot(&self, w: &mut Writer) {
        w.record("grid", |l| match &self.drag {
            None => {
                l.none();
            }
            Some(drag) => {
                l.put(&drag.start)
                    .put(&drag.end)
                    .bool(drag.shift)
                    .put(&drag.style)
                    .put(&drag.guides);
            }
        });
    }

    /// Reads what [`State::write_snapshot`] wrote.
    pub(crate) fn read_snapshot(r: &mut Reader<'_>) -> Result<Self, SnapshotError> {
        let mut t = r.record("grid")?;
        let drag = if t.none() {
            None
        } else {
            Some(Drag {
                start: t.loose_vec2()?,
                end: t.loose_vec2()?,
                shift: t.bool("shift")?,
                style: t.get()?,
                guides: t.loose_segments()?,
            })
        };
        t.end()?;
        Ok(Self { drag })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::{Camera, ZOOM_MAX, ZOOM_MIN};
    use crate::core::clipboard::Clipboard;
    use crate::core::command::Tool;
    use crate::core::document::{Document, ShapeId, tx_insert};
    use crate::core::editor::DrawStyle;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::history::History;
    use crate::core::input::Modifiers;
    use crate::core::palette::ColorId;
    use crate::core::shape::{GRID_MAX_CELLS, SHAPE_TEXT_FILL, Shape, Style, TextSlot};
    use crate::core::smoothing::SmoothingLevel;
    use crate::core::text_layout;
    use crate::core::tools::Phase;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseError;

    const EPS: f32 = 1e-4;

    /// Editor state a grid gesture runs against.
    struct Fixture {
        doc: Document,
        history: History,
        selection: Vec<ShapeId>,
        clipboard: Clipboard,
        camera: Camera,
        style: DrawStyle,
        state: State,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                doc: Document::new(),
                history: History::new(),
                selection: Vec::new(),
                clipboard: Clipboard::default(),
                camera: Camera::default(),
                style: DrawStyle::default(),
                state: State::default(),
            }
        }

        fn send_with(&mut self, phase: Phase, x: f32, y: f32, mods: Modifiers) -> bool {
            let pos = Vec2::new(x, y);
            let mut ctx = ToolCtx {
                doc: &mut self.doc,
                history: &mut self.history,
                selection: &mut self.selection,
                clipboard: &mut self.clipboard,
                camera: &self.camera,
                tool: Tool::Grid,
                style: self.style,
                smoothing: SmoothingLevel::Medium,
                cursor: pos,
            };
            on_pointer(
                &mut self.state,
                &mut ctx,
                Pointer {
                    phase,
                    pos,
                    mods,
                    pressure: None,
                },
            )
        }

        fn send_mods(&mut self, phase: Phase, x: f32, y: f32, shift: bool) -> bool {
            let mods = Modifiers {
                shift,
                ..Modifiers::NONE
            };
            self.send_with(phase, x, y, mods)
        }

        fn send(&mut self, phase: Phase, x: f32, y: f32) -> bool {
            self.send_mods(phase, x, y, false)
        }

        /// A full drag from `from` to `to` (screen), one move in between.
        fn drag(&mut self, from: (f32, f32), to: (f32, f32), shift: bool) -> bool {
            self.send_mods(Phase::Down, from.0, from.1, shift);
            self.send_mods(Phase::Move, to.0, to.1, shift);
            self.send_mods(Phase::Up, to.0, to.1, shift)
        }

        /// Sets the grid dimensions as the arrow keys would.
        fn dims(&mut self, cols: u32, rows: u32) {
            self.style.helpers.grid_cols = cols;
            self.style.helpers.grid_rows = rows;
        }

        /// Puts `shape` in the document as one undo step.
        fn insert(&mut self, shape: Shape) {
            let tx = tx_insert(&mut self.doc, [shape]);
            assert!(self.history.commit(&mut self.doc, tx).is_ok());
        }

        fn overlay(&self) -> Overlay {
            preview(
                &self.state,
                &ToolView {
                    doc: &self.doc,
                    camera: &self.camera,
                    selection: &self.selection,
                    tool: Tool::Grid,
                    style: self.style,
                    smoothing: SmoothingLevel::Medium,
                    cursor: Vec2::ZERO,
                },
            )
        }

        /// The shape added last.
        fn last_shape(&self) -> &Shape {
            match self.doc.shapes().last() {
                Some((_, shape)) => shape,
                None => panic!("document is empty"),
            }
        }
    }

    /// Corners, columns, rows and style of a grid.
    fn grid(shape: &Shape) -> (Vec2, Vec2, u32, u32, Style) {
        match shape {
            Shape::Grid {
                a,
                b,
                cols,
                rows,
                style,
                ..
            } => (*a, *b, *cols, *rows, *style),
            other => panic!("expected a grid, got {other:?}"),
        }
    }

    /// The only shape of a preview.
    fn previewed(overlay: &Overlay) -> &Shape {
        assert_eq!(overlay.shapes.len(), 1, "expected one preview shape");
        &overlay.shapes[0]
    }

    // ---- AC-1 drag, preview, commit ---------------------------------------

    #[test]
    fn drag_commits_one_grid() {
        // Arrange
        let mut f = Fixture::new();

        // Act
        let changed = f.drag((10.0, 20.0), (90.0, 60.0), false);

        // Assert
        assert!(changed);
        assert_eq!(f.doc.len(), 1);
        let (a, b, cols, rows, style) = grid(f.last_shape());
        assert!(a.approx_eq(Vec2::new(10.0, 20.0), EPS), "{a:?}");
        assert!(b.approx_eq(Vec2::new(90.0, 60.0), EPS), "{b:?}");
        assert_eq!((cols, rows), (4, 4));
        assert_eq!(style.color, f.style.color);
        assert!(f.overlay().is_empty(), "preview ends with the drag");
    }

    #[test]
    fn commit_is_one_undo_step() {
        // Arrange
        let mut f = Fixture::new();

        // Act
        f.send(Phase::Down, 0.0, 0.0);
        for i in 1..10 {
            f.send(Phase::Move, 10.0 * i as f32, 5.0 * i as f32);
        }
        f.send(Phase::Up, 100.0, 50.0);

        // Assert
        assert_eq!(f.history.undo_len(), 1);
        assert!(f.history.undo(&mut f.doc));
        assert!(f.doc.is_empty());
    }

    #[test]
    fn short_drag_commits_nothing() {
        // Arrange: at zoom 0.5 a 1.5 px drag is 3 world units long.
        let mut f = Fixture::new();
        f.camera = Camera::new(Vec2::ZERO, 0.5);

        // Act
        f.drag((10.0, 10.0), (11.5, 10.0), false);

        // Assert
        assert!(f.doc.is_empty());
        assert_eq!(f.history.undo_len(), 0);
        assert!(f.overlay().is_empty());
    }

    #[test]
    fn preview_uses_current_dims() {
        // Arrange
        let mut f = Fixture::new();
        f.dims(7, 3);

        // Act
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 70.0, 30.0);
        let overlay = f.overlay();

        // Assert
        let (a, b, cols, rows, _) = grid(previewed(&overlay));
        assert!(a.approx_eq(Vec2::ZERO, EPS));
        assert!(b.approx_eq(Vec2::new(70.0, 30.0), EPS));
        assert_eq!((cols, rows), (7, 3));
        assert!(f.doc.is_empty(), "nothing committed while dragging");
    }

    #[test]
    fn preview_empty_when_idle() {
        let f = Fixture::new();

        assert!(f.overlay().is_empty());
    }

    #[test]
    fn move_without_drag_is_ignored() {
        let mut f = Fixture::new();

        assert!(!f.send(Phase::Move, 30.0, 30.0));
        assert!(!f.send(Phase::Up, 30.0, 30.0));

        assert!(f.doc.is_empty());
        assert!(f.overlay().is_empty());
    }

    #[test]
    fn width_is_world_units_at_zoom() {
        // Arrange
        let mut f = Fixture::new();
        f.camera = Camera::new(Vec2::new(-40.0, 10.0), 4.0);

        // Act
        f.drag((0.0, 0.0), (80.0, 40.0), false);

        // Assert
        let (a, b, _, _, style) = grid(f.last_shape());
        assert!(a.approx_eq(Vec2::new(-40.0, 10.0), EPS), "{a:?}");
        assert!(b.approx_eq(Vec2::new(-20.0, 20.0), EPS), "{b:?}");
        assert!(approx_eq(style.width, f.style.width_px / 4.0, EPS));
    }

    // ---- AC-2 live dimensions ---------------------------------------------

    #[test]
    fn arrows_change_dims_live() {
        // Arrange
        let mut f = Fixture::new();
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 60.0, 40.0);

        // Act: arrows between pointer events, no pointer motion.
        f.dims(5, 4);
        let first = f.overlay();
        f.dims(5, 2);
        let second = f.overlay();
        f.dims(6, 2);
        f.send(Phase::Up, 60.0, 40.0);

        // Assert
        assert_eq!(grid(previewed(&first)).2, 5);
        let (_, _, cols, rows, _) = grid(previewed(&second));
        assert_eq!((cols, rows), (5, 2));
        let (_, _, cols, rows, _) = grid(f.last_shape());
        assert_eq!((cols, rows), (6, 2), "commit uses the values at release");
    }

    #[test]
    fn dims_persist_for_next_grid() {
        // Arrange
        let mut f = Fixture::new();
        f.dims(3, 9);
        f.drag((0.0, 0.0), (30.0, 90.0), false);

        // Act
        f.drag((100.0, 0.0), (130.0, 90.0), false);

        // Assert
        assert_eq!(f.doc.len(), 2);
        let (_, _, cols, rows, _) = grid(f.last_shape());
        assert_eq!((cols, rows), (3, 9));
    }

    // ---- AC-3 Shift: square cells -----------------------------------------

    #[test]
    fn shift_makes_square_cells() {
        // Arrange
        let mut f = Fixture::new();
        f.dims(4, 2);

        // Act: 100 × 30 box; 25-wide columns win over 15-tall rows.
        f.drag((0.0, 0.0), (100.0, 30.0), true);

        // Assert
        let (a, b, _, _, _) = grid(f.last_shape());
        assert!(a.approx_eq(Vec2::ZERO, EPS));
        assert!(b.approx_eq(Vec2::new(100.0, 50.0), EPS), "{b:?}");
    }

    #[test]
    fn shift_keeps_drag_direction() {
        // Arrange
        let mut f = Fixture::new();
        f.dims(3, 1);

        // Act: drag up-left by 60 × 10; cell side 20.
        f.drag((100.0, 100.0), (40.0, 90.0), true);

        // Assert
        let (a, b, _, _, _) = grid(f.last_shape());
        assert!(a.approx_eq(Vec2::new(100.0, 100.0), EPS));
        assert!(b.approx_eq(Vec2::new(40.0, 80.0), EPS), "{b:?}");
    }

    // ---- AC-4 snapping ------------------------------------------------------

    #[test]
    fn grid_snap_snaps_corners() {
        // Arrange
        let mut f = Fixture::new();
        f.style.helpers.grid_snap = true;

        // Act
        f.drag((11.0, 9.0), (52.0, 68.0), false);

        // Assert
        let (a, b, _, _, _) = grid(f.last_shape());
        assert!(a.approx_eq(Vec2::new(20.0, 0.0), EPS), "{a:?}");
        assert!(b.approx_eq(Vec2::new(60.0, 60.0), EPS), "{b:?}");
    }

    #[test]
    fn preview_shows_guides() {
        // Arrange: a 50 × 30 box at the origin to align with.
        let mut f = Fixture::new();
        f.style.helpers.smart_snap = true;
        f.insert(Shape::Rect {
            a: Vec2::ZERO,
            b: Vec2::new(50.0, 30.0),
            style: Style {
                color: ColorId::INK,
                width: 1.0,
            },
            fill: None,
            label: None,
            text: None,
        });

        // Act
        f.send(Phase::Down, 200.0, 3.0);
        f.send(Phase::Move, 253.0, 28.0);
        let overlay = f.overlay();
        f.send(Phase::Up, 253.0, 28.0);

        // Assert
        assert_ne!(overlay.guides.len(), 0);
        let (a, _, _, _, _) = grid(f.last_shape());
        assert!(a.approx_eq(Vec2::new(200.0, 0.0), EPS), "{a:?}");
        assert!(f.overlay().is_empty(), "guides go away with the drag");
    }

    // ---- AC-5 cancel and robustness ---------------------------------------

    #[test]
    fn cancel_keeps_document() {
        // Arrange
        let mut f = Fixture::new();
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 50.0, 50.0);

        // Act
        let redraw = cancel(&mut f.state);
        f.send(Phase::Up, 50.0, 50.0);

        // Assert
        assert!(redraw);
        assert!(f.doc.is_empty());
        assert_eq!(f.history.undo_len(), 0);
        assert!(f.overlay().is_empty());
    }

    #[test]
    fn cancel_when_idle_needs_no_redraw() {
        let mut f = Fixture::new();

        assert!(!cancel(&mut f.state));
    }

    fn coord() -> impl Strategy<Value = f32> {
        -5000.0_f32..5000.0
    }

    proptest! {
        #[test]
        fn committed_grid_is_finite_and_in_range(
            zoom in ZOOM_MIN..ZOOM_MAX,
            from in (coord(), coord()),
            moves in proptest::collection::vec(
                (coord(), coord(), any::<bool>(), any::<u32>(), any::<u32>()),
                0..8,
            ),
            to in (coord(), coord()),
            shift in any::<bool>(),
            grid_snap in any::<bool>(),
            smart_snap in any::<bool>(),
        ) {
            let mut f = Fixture::new();
            f.camera = Camera::new(Vec2::new(13.0, -7.0), zoom);
            f.style.helpers.grid_snap = grid_snap;
            f.style.helpers.smart_snap = smart_snap;

            f.send_mods(Phase::Down, from.0, from.1, shift);
            for (x, y, s, cols, rows) in moves {
                // Raw values, even out of range: the tool must clamp.
                f.dims(cols, rows);
                f.send_mods(Phase::Move, x, y, s);
                for shape in &f.overlay().shapes {
                    let (_, _, c, r, _) = grid(shape);
                    prop_assert!((1..=GRID_MAX_CELLS).contains(&c));
                    prop_assert!((1..=GRID_MAX_CELLS).contains(&r));
                }
            }
            f.send_mods(Phase::Up, to.0, to.1, shift);

            prop_assert!(f.doc.len() <= 1);
            prop_assert_eq!(f.doc.len(), f.history.undo_len());
            for (_, shape) in f.doc.shapes() {
                let (_, _, c, r, _) = grid(shape);
                prop_assert!(shape.is_finite());
                prop_assert!((1..=GRID_MAX_CELLS).contains(&c));
                prop_assert!((1..=GRID_MAX_CELLS).contains(&r));
            }
            prop_assert!(f.overlay().is_empty());
        }

        #[test]
        fn shift_cells_always_square(
            from in (coord(), coord()),
            to in (coord(), coord()),
            cols in 1..=GRID_MAX_CELLS,
            rows in 1..=GRID_MAX_CELLS,
        ) {
            let mut f = Fixture::new();
            f.dims(cols, rows);

            f.drag(from, to, true);

            if let Some((_, shape)) = f.doc.shapes().next() {
                let (a, b, c, r, _) = grid(shape);
                let d = b - a;
                let (cw, ch) = (d.x.abs() / c as f32, d.y.abs() / r as f32);
                let tol = 1e-3 * cw.max(ch).max(1.0);
                prop_assert!((cw - ch).abs() <= tol, "cell {cw} × {ch}");
            }
        }
    }

    // ---- AC-7 axis indices (T22: driven by numbering) ----------------------

    /// Whether `shape` is a grid with axis indices.
    fn has_axes(shape: &Shape) -> bool {
        matches!(shape, Shape::Grid { axes: true, .. })
    }

    #[test]
    fn axes_off_by_default() {
        let mut f = Fixture::new();

        f.drag((0.0, 0.0), (40.0, 40.0), false);

        assert!(!has_axes(f.last_shape()));
    }

    #[test]
    fn numbering_turns_on_axes() {
        // Arrange
        let mut f = Fixture::new();
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 40.0, 40.0);

        // Act: numbering toggled mid-drag, like the dimensions (ADR-T22-1).
        f.style.helpers.numbering = true;
        let overlay = f.overlay();
        f.send(Phase::Up, 40.0, 40.0);

        // Assert
        assert!(has_axes(previewed(&overlay)));
        assert!(has_axes(f.last_shape()));
    }

    // ---- T84 tables from text ----------------------------------------------

    const TABLE_STYLE: Style = Style {
        color: ColorId::INK,
        width: 1.0,
    };

    /// The table `text` parses to.
    fn table(text: &str) -> TextTable {
        match TextTable::parse(text) {
            Ok(Some(table)) => table,
            other => panic!("expected a table from {text:?}, got {other:?}"),
        }
    }

    /// The grid of `text` at the origin, text size 18.
    fn table_shape(text: &str) -> Shape {
        match table(text).shape(Vec2::ZERO, 18.0, TABLE_STYLE, false) {
            Some(shape) => shape,
            None => panic!("no grid from {text:?}"),
        }
    }

    /// `(col, row, text)` of every cell of `shape`.
    fn table_cells(shape: &Shape) -> Vec<(u32, u32, String)> {
        shape
            .cell_texts()
            .iter()
            .map(|t| (t.col, t.row, t.content.text.clone()))
            .collect()
    }

    /// `n` tokens on a line.
    fn tokens(n: usize) -> String {
        vec!["x"; n].join(" ")
    }

    /// Width and height of one cell of `shape`.
    fn cell_size(shape: &Shape) -> (f32, f32) {
        let (a, b, cols, rows, _) = grid(shape);
        ((b.x - a.x) / cols as f32, (b.y - a.y) / rows as f32)
    }

    #[test]
    fn table_parse_splits_lines_and_whitespace() {
        // Act
        let shape = table_shape("1  2\t3\n4 5 6");

        // Assert
        assert_eq!((grid(&shape).2, grid(&shape).3), (3, 2));
        let cells = table_cells(&shape);
        let text: Vec<&str> = cells.iter().map(|c| c.2.as_str()).collect();
        assert_eq!(text, ["1", "2", "3", "4", "5", "6"]);
    }

    #[test]
    fn table_parse_ignores_blank_lines() {
        let parsed = table("\n1 2\n   \n\n3 4\n\n");

        assert_eq!((parsed.cols(), parsed.rows()), (2, 2));
    }

    #[test]
    fn table_parse_short_rows_leave_empty_cells() {
        // Act
        let shape = table_shape("1 2 3\n4");

        // Assert: row 1 holds one cell, columns 1 and 2 stay empty.
        assert_eq!((grid(&shape).2, grid(&shape).3), (3, 2));
        assert_eq!(
            table_cells(&shape).last(),
            Some(&(0, 1, "4".to_owned())),
            "{shape:?}"
        );
        assert_eq!(shape.cell_texts().len(), 4);
    }

    #[test]
    fn table_parse_handles_line_endings_and_controls() {
        // Act: `\r\n`, a bare `\r`, a bell inside a token and a lone one.
        let shape = table_shape("1 2\r\n3\u{7}4 \u{7}\r5");

        // Assert
        assert_eq!(
            table_cells(&shape),
            [
                (0, 0, "1".to_owned()),
                (1, 0, "2".to_owned()),
                (0, 1, "34".to_owned()),
                (0, 2, "5".to_owned()),
            ]
        );
    }

    #[test]
    fn table_parse_one_line_is_a_strip() {
        let parsed = table("3 1 4 1 5");

        assert_eq!((parsed.cols(), parsed.rows()), (5, 1));
    }

    #[test]
    fn table_parse_blank_text_is_no_table() {
        for text in ["", "   ", "\n \t\r\n", "\u{7}"] {
            assert_eq!(TextTable::parse(text), Ok(None), "{text:?}");
        }
    }

    #[test]
    fn table_parse_at_the_limits_is_accepted() {
        // Arrange
        let wide = tokens(GRID_MAX_CELLS as usize);
        let tall = vec!["x"; GRID_MAX_CELLS as usize].join("\n");

        // Act / Assert
        assert_eq!(table(&wide).cols(), GRID_MAX_CELLS);
        assert_eq!(table(&tall).rows(), GRID_MAX_CELLS);
    }

    #[test]
    fn table_parse_over_the_limits_is_refused() {
        // Arrange
        let cells = GRID_MAX_CELLS as usize + 1;
        let wide = tokens(cells);
        let tall = vec!["x"; cells].join("\n");

        // Act / Assert
        assert_eq!(
            TextTable::parse(&wide),
            Err(TableError::TooManyCells {
                cols: cells,
                rows: 1
            })
        );
        assert_eq!(
            TextTable::parse(&tall),
            Err(TableError::TooManyCells {
                cols: 1,
                rows: cells
            })
        );
    }

    #[test]
    fn table_parse_over_the_byte_limit_is_refused() {
        // Arrange: one token, so only the byte count is over a limit.
        let at_limit = "x".repeat(PASTE_MAX_BYTES);
        let over = "x".repeat(PASTE_MAX_BYTES + 1);

        // Act / Assert
        assert_eq!(table(&at_limit).cols(), 1);
        assert_eq!(
            TextTable::parse(&over),
            Err(TableError::TooLong {
                bytes: PASTE_MAX_BYTES + 1
            })
        );
    }

    #[test]
    fn table_error_names_the_reason() {
        let cells = TableError::TooManyCells { cols: 70, rows: 2 }.to_string();
        let long = TableError::TooLong { bytes: 70_000 }.to_string();

        assert!(cells.contains("70") && cells.contains("64"), "{cells}");
        assert!(long.contains("70000"), "{long}");
    }

    #[test]
    fn table_shape_places_tokens_in_sorted_cells() {
        // Act
        let shape = table_shape("a b\nc");

        // Assert
        assert_eq!(
            table_cells(&shape),
            [
                (0, 0, "a".to_owned()),
                (1, 0, "b".to_owned()),
                (0, 1, "c".to_owned()),
            ]
        );
        assert!(
            shape
                .cell_texts()
                .iter()
                .all(|t| approx_eq(t.content.size, 18.0, EPS))
        );
    }

    #[test]
    fn table_shape_cells_follow_the_longest_token() {
        // Arrange: 20 characters at size 18 are 216 wide.
        let longest = "x".repeat(20);
        let unit = crate::core::text_layout::bounds(Vec2::ZERO, &longest, 18.0);

        // Act
        let shape = table_shape(&format!("1 {longest}\n2 3"));

        // Assert
        let (w, h) = cell_size(&shape);
        assert!(approx_eq(w, unit.width() / SHAPE_TEXT_FILL, EPS), "{w}");
        assert!(approx_eq(h, unit.height() / SHAPE_TEXT_FILL, EPS), "{h}");
    }

    #[test]
    fn table_shape_digits_get_square_cells() {
        let (w, h) = cell_size(&table_shape("1 2\n3 4"));

        assert!(approx_eq(w, h, EPS), "{w} × {h}");
    }

    #[test]
    fn table_shape_text_fits_at_full_size() {
        // Arrange: a long token and a formula.
        let shape = table_shape("abcdefgh $\\frac{a}{b}$\n1 2");

        // Act / Assert: no cell has to shrink its text.
        for t in shape.cell_texts() {
            let slot = TextSlot::Cell {
                col: t.col,
                row: t.row,
            };
            let frame = shape.text_frame(slot);
            let fitted = frame.and_then(|f| text_layout::fit(&t.content.text, f, 18.0));
            let size = fitted.map_or(0.0, |(_, size)| size);
            assert!(approx_eq(size, 18.0, 1e-2), "{:?}: {size}", t.content.text);
        }
    }

    #[test]
    fn table_shape_uses_corner_style_and_axes() {
        // Act
        let shape = table("1 2").shape(Vec2::new(5.0, 7.0), 18.0, TABLE_STYLE, true);

        // Assert
        let Some(shape) = shape else {
            panic!("no grid");
        };
        let (a, _, _, _, style) = grid(&shape);
        assert!(a.approx_eq(Vec2::new(5.0, 7.0), EPS));
        assert_eq!(style, TABLE_STYLE);
        assert!(has_axes(&shape));
    }

    #[test]
    fn table_shape_with_bad_size_is_none() {
        let parsed = table("1 2");

        for size in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::MAX] {
            assert_eq!(
                parsed.shape(Vec2::ZERO, size, TABLE_STYLE, false),
                None,
                "{size}"
            );
        }
    }

    proptest! {
        #[test]
        fn table_parse_and_shape_hold_the_invariants(
            text in "[a-z0-9$\\\\{} \t\r\n\u{7}β]{0,300}",
            size in 1.0_f32..200.0,
        ) {
            let Ok(Some(parsed)) = TextTable::parse(&text) else {
                return Ok(());
            };
            let tokens = text
                .split(['\n', '\r'])
                .flat_map(str::split_whitespace)
                .filter(|token| token.chars().any(|ch| !ch.is_control()))
                .count();

            let Some(shape) = parsed.shape(Vec2::ZERO, size, TABLE_STYLE, false) else {
                return Err(TestCaseError::fail("no grid"));
            };

            let (_, _, cols, rows, _) = grid(&shape);
            prop_assert!(shape.is_finite());
            prop_assert!((1..=GRID_MAX_CELLS).contains(&cols));
            prop_assert!((1..=GRID_MAX_CELLS).contains(&rows));
            let cells = shape.cell_texts();
            prop_assert_eq!(cells.len(), tokens);
            for pair in cells.windows(2) {
                prop_assert!((pair[0].row, pair[0].col) < (pair[1].row, pair[1].col));
            }
            for cell in cells {
                prop_assert!(cell.col < cols && cell.row < rows);
                prop_assert!(!cell.content.text.trim().is_empty());
            }
        }

        #[test]
        fn table_parse_never_panics(text in "\\PC{0,200}") {
            let _ = TextTable::parse(&text);
        }
    }
}
