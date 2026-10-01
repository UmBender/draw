//! Grid tool (T18): drag a box, get a `cols × rows`
//! [`Shape::Grid`](crate::core::shape::Shape::Grid).
//!
//! The dimensions come from
//! [`Helpers`](crate::core::editor::Helpers) in [`ToolView::style`]; arrow
//! keys change them live during the drag (ADR-T16-2).
//!
//! Skeleton (T16): every entry point does nothing until T18 fills it in.

use crate::core::tools::{Overlay, Pointer, ToolCtx, ToolView};

/// Gesture state of the grid tool.
#[derive(Debug, Clone, Default)]
pub struct State {}

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
///
/// Skeleton: ignores the event.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    let _ = (state, ctx, pointer);
    false
}

/// The overlay of the gesture in progress.
///
/// Skeleton: empty.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    let _ = (state, view);
    Overlay::default()
}

/// Discards the gesture in progress. Returns whether a redraw is needed.
///
/// Skeleton: there is never a gesture.
pub fn cancel(state: &mut State) -> bool {
    let _ = state;
    false
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
    use crate::core::shape::{GRID_MAX_CELLS, Shape, Style};
    use crate::core::smoothing::SmoothingLevel;
    use crate::core::tools::Phase;
    use proptest::prelude::*;

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
            on_pointer(&mut self.state, &mut ctx, Pointer { phase, pos, mods })
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
        });

        // Act
        f.send(Phase::Down, 200.0, 3.0);
        f.send(Phase::Move, 253.0, 28.0);
        let overlay = f.overlay();
        f.send(Phase::Up, 253.0, 28.0);

        // Assert
        assert!(!overlay.guides.is_empty());
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
}
