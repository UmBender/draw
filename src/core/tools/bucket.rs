//! Bucket: fills the clicked closed shape.
//!
//! A press sets the fill of the topmost [`Shape::Rect`] or [`Shape::Ellipse`]
//! containing the point to the current colour, as one undo step. If the
//! topmost candidate is a [`Shape::Grid`] whose box contains the point, only
//! the cell under the point is filled (ADR-T21-1). Refilling with the same
//! colour, or pressing on empty space or another open shape, changes nothing
//! and records no step. The bucket has no gesture to preview.
//!
//! [`Shape::Rect`]: crate::core::shape::Shape::Rect
//! [`Shape::Ellipse`]: crate::core::shape::Shape::Ellipse
//! [`Shape::Grid`]: crate::core::shape::Shape::Grid

use super::{Overlay, Phase, Pointer, ToolCtx, ToolView};
use crate::core::document::{Document, ShapeId, tx_replace};
use crate::core::geom::Vec2;
use crate::core::shape::Shape;
use crate::core::snapshot::{Reader, SnapshotError, Writer, check_ids};

/// A drag across the cells of one grid (ADR-T88-1).
#[derive(Debug, Clone, PartialEq)]
struct Drag {
    /// The grid pressed; the only one this drag paints.
    id: ShapeId,
    /// Last pointer position, in screen pixels.
    last: Vec2,
    /// The grid with the cells painted so far.
    painted: Shape,
}

/// Gesture state of this tool: the drag in progress, if any.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    /// `None` when idle, and for a press that fills a whole shape.
    drag: Option<Drag>,
}

/// The cells of `grid` that the world segment `from → to` passes through,
/// in order, each one step from the one before.
fn cells_along(grid: &Shape, from: Vec2, to: Vec2) -> Vec<(u32, u32)> {
    let _ = (grid, from, to);
    todo!()
}

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    let _ = state;
    if pointer.phase != Phase::Down {
        return false;
    }
    let world = ctx.camera.screen_to_world(pointer.pos);
    let fill = Some(ctx.style.color);
    let Some(id) = ctx.doc.topmost_where(|shape| {
        (shape.is_closed() && shape.contains(world)) || shape.grid_cell_at(world).is_some()
    }) else {
        return false;
    };
    let Some(shape) = ctx.doc.get(id) else {
        return false;
    };
    let filled = match shape.grid_cell_at(world) {
        Some((col, row)) => shape.clone().with_cell_fill(col, row, fill),
        None => shape.clone().with_fill(fill),
    };
    if &filled == shape {
        return false;
    }
    let tx = tx_replace(ctx.doc, id, filled);
    ctx.commit(tx)
}

/// What the gesture in progress draws on top of the document.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    let _ = (state, view);
    Overlay::default()
}

/// Discards the gesture in progress without changing the document. Returns
/// whether a redraw is needed.
pub fn cancel(state: &mut State) -> bool {
    let _ = state;
    false
}

impl State {
    /// `bucket <last> <id> <painted grid>`, only while a drag runs
    /// (snapshots, ADR-T88-1).
    pub(crate) fn write_snapshot(&self, w: &mut Writer) {
        if let Some(drag) = &self.drag {
            w.record("bucket", |l| {
                l.put(&drag.last).put(&drag.id).put(&drag.painted);
            });
        }
    }

    /// Reads what [`State::write_snapshot`] wrote: the id must be in `doc`
    /// and the shape a grid.
    pub(crate) fn read_snapshot(r: &mut Reader<'_>, doc: &Document) -> Result<Self, SnapshotError> {
        if r.peek_key() != Some("bucket") {
            return Ok(Self::default());
        }
        let mut t = r.record("bucket")?;
        let last: Vec2 = t.get()?;
        let id: ShapeId = t.get()?;
        let painted: Shape = t.get()?;
        t.end()?;
        check_ids(&t, doc, &[id])?;
        if !matches!(painted, Shape::Grid { .. }) {
            return Err(t.error("the bucket can only drag across a grid".to_owned()));
        }
        Ok(Self {
            drag: Some(Drag { id, last, painted }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::clipboard::testkit::{Fixture, ellipse, line, ptr, rect};
    use crate::core::command::Tool;
    use crate::core::document::ShapeId;
    use crate::core::geom::Vec2;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use crate::core::tools::Phase;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseError;

    /// Unfilled `cols × rows` grid dragged from `(x, y)` to `(x + w, y + h)`.
    fn grid(x: f32, y: f32, w: f32, h: f32, cols: u32, rows: u32) -> Shape {
        Shape::Grid {
            a: Vec2::new(x, y),
            b: Vec2::new(x + w, y + h),
            cols,
            rows,
            style: Style {
                color: ColorId::INK,
                width: 1.0,
            },
            axes: false,
            fills: Vec::new(),
            texts: Vec::new(),
        }
    }

    fn red() -> ColorId {
        ColorId::new(2).unwrap_or(ColorId::INK)
    }

    fn click(fx: &mut Fixture, x: f32, y: f32) -> bool {
        let mut state = State::default();
        let mut changed = on_pointer(
            &mut state,
            &mut fx.ctx(Tool::Bucket),
            ptr(Phase::Down, x, y),
        );
        changed |= on_pointer(&mut state, &mut fx.ctx(Tool::Bucket), ptr(Phase::Up, x, y));
        changed
    }

    #[test]
    fn click_inside_rect_fills_with_current_color() {
        // Arrange
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        fx.style.color = red();

        // Act
        let changed = click(&mut fx, 5.0, 5.0);

        // Assert
        assert!(changed);
        assert_eq!(fx.shape(ids[0]).fill(), Some(red()));
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn click_inside_ellipse_fills() {
        let (mut fx, ids) = Fixture::new(vec![ellipse(0.0, 0.0, 10.0, 10.0)]);
        fx.style.color = red();

        assert!(click(&mut fx, 5.0, 5.0));

        assert_eq!(fx.shape(ids[0]).fill(), Some(red()));
    }

    #[test]
    fn click_fills_topmost_closed_shape() {
        let (mut fx, ids) = Fixture::new(vec![
            rect(0.0, 0.0, 10.0, 10.0),
            rect(2.0, 2.0, 6.0, 6.0),
            line(0.0, 5.0, 10.0, 5.0),
        ]);
        fx.style.color = red();

        click(&mut fx, 5.0, 4.0);

        assert_eq!(fx.shape(ids[0]).fill(), None);
        assert_eq!(fx.shape(ids[1]).fill(), Some(red()));
    }

    #[test]
    fn same_color_is_noop() {
        let (mut fx, _) = Fixture::new(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        fx.style.color = red();
        click(&mut fx, 5.0, 5.0);

        let changed = click(&mut fx, 5.0, 5.0);

        assert!(!changed);
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn empty_space_is_noop() {
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 10.0, 10.0)]);

        assert!(!click(&mut fx, 50.0, 50.0));

        assert_eq!(fx.shape(ids[0]).fill(), None);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn open_shape_is_not_filled() {
        let (mut fx, ids) = Fixture::new(vec![line(0.0, 0.0, 10.0, 0.0)]);

        assert!(!click(&mut fx, 5.0, 0.0));

        assert_eq!(fx.shape(ids[0]), line(0.0, 0.0, 10.0, 0.0));
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn fill_is_undoable() {
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        fx.style.color = red();
        click(&mut fx, 5.0, 5.0);

        assert!(fx.history.undo(&mut fx.doc));

        assert_eq!(fx.shape(ids[0]).fill(), None);
    }

    #[test]
    fn preview_and_cancel_are_inert() {
        let (fx, _) = Fixture::new(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let mut state = State::default();

        assert!(preview(&state, &fx.view(Tool::Bucket)).is_empty());
        assert!(!cancel(&mut state));
    }

    // ---- T21 cell fills -----------------------------------------------------

    #[test]
    fn click_in_grid_cell_fills_it() {
        // Arrange: 4 × 4 cells of 25.
        let (mut fx, ids) = Fixture::new(vec![grid(0.0, 0.0, 100.0, 100.0, 4, 4)]);
        fx.style.color = red();

        // Act
        let changed = click(&mut fx, 37.0, 62.0);

        // Assert
        assert!(changed);
        let g = fx.shape(ids[0]);
        assert_eq!(g.cell_fill(1, 2), Some(red()));
        assert_eq!(g.cell_fills().len(), 1);
    }

    #[test]
    fn refill_cell_same_color_is_noop() {
        let (mut fx, _) = Fixture::new(vec![grid(0.0, 0.0, 100.0, 100.0, 4, 4)]);
        fx.style.color = red();
        click(&mut fx, 37.0, 62.0);

        assert!(!click(&mut fx, 40.0, 60.0));

        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn cell_fill_is_one_undo_step() {
        let (mut fx, ids) = Fixture::new(vec![grid(0.0, 0.0, 100.0, 100.0, 4, 4)]);
        fx.style.color = red();
        click(&mut fx, 10.0, 10.0);
        assert_eq!(fx.history.undo_len(), 1);

        assert!(fx.history.undo(&mut fx.doc));

        assert_eq!(fx.shape(ids[0]).cell_fills(), []);
    }

    #[test]
    fn topmost_of_rect_and_grid_wins() {
        // Arrange: a grid over a rect, and a rect over another grid.
        let (mut fx, ids) = Fixture::new(vec![
            rect(0.0, 0.0, 100.0, 100.0),
            grid(0.0, 0.0, 100.0, 100.0, 4, 4),
            grid(200.0, 0.0, 100.0, 100.0, 4, 4),
            rect(200.0, 0.0, 100.0, 100.0),
        ]);
        fx.style.color = red();

        // Act
        click(&mut fx, 10.0, 10.0);
        click(&mut fx, 210.0, 10.0);

        // Assert
        assert_eq!(fx.shape(ids[0]).fill(), None);
        assert_eq!(fx.shape(ids[1]).cell_fill(0, 0), Some(red()));
        assert_eq!(fx.shape(ids[2]).cell_fills(), []);
        assert_eq!(fx.shape(ids[3]).fill(), Some(red()));
    }

    #[test]
    fn click_outside_grid_box_does_nothing() {
        let (mut fx, ids) = Fixture::new(vec![grid(0.0, 0.0, 100.0, 100.0, 4, 4)]);

        assert!(!click(&mut fx, 150.0, 50.0));

        assert_eq!(fx.shape(ids[0]).cell_fills(), []);
        assert_eq!(fx.history.undo_len(), 0);
    }

    // ---- T88 drag across cells ----------------------------------------------

    /// A 4 × 4 grid of 25 px cells at the origin, the bucket red, and its id.
    fn red_board() -> (Fixture, ShapeId) {
        let (mut fx, ids) = Fixture::new(vec![grid(0.0, 0.0, 100.0, 100.0, 4, 4)]);
        fx.style.color = red();
        (fx, ids[0])
    }

    fn send(fx: &mut Fixture, state: &mut State, phase: Phase, x: f32, y: f32) -> bool {
        on_pointer(state, &mut fx.ctx(Tool::Bucket), ptr(phase, x, y))
    }

    /// Presses at the first point, moves through the others and releases at
    /// the last. Returns the state, ended.
    fn drag(fx: &mut Fixture, path: &[(f32, f32)]) -> State {
        let mut state = State::default();
        let ((x, y), rest) = match path.split_first() {
            Some((first, rest)) => (*first, rest),
            None => panic!("empty path"),
        };
        send(fx, &mut state, Phase::Down, x, y);
        for (x, y) in rest {
            send(fx, &mut state, Phase::Move, *x, *y);
        }
        let (x, y) = path[path.len() - 1];
        send(fx, &mut state, Phase::Up, x, y);
        state
    }

    /// `(col, row)` of the filled cells of `shape`, sorted by `(row, col)`.
    fn cells(shape: &Shape) -> Vec<(u32, u32)> {
        shape.cell_fills().iter().map(|f| (f.col, f.row)).collect()
    }

    #[test]
    fn drag_fills_each_cell_passed_over() {
        // Arrange
        let (mut fx, id) = red_board();

        // Act: right along the top row, then down the third column.
        drag(
            &mut fx,
            &[(10.0, 10.0), (35.0, 10.0), (60.0, 10.0), (60.0, 35.0)],
        );

        // Assert
        let board = fx.shape(id);
        assert_eq!(cells(&board), [(0, 0), (1, 0), (2, 0), (2, 1)]);
        assert!(board.cell_fills().iter().all(|f| f.color == red()));
    }

    #[test]
    fn drag_stays_in_the_pressed_grid() {
        // Arrange: a second grid right beside the first.
        let (mut fx, ids) = Fixture::new(vec![
            grid(0.0, 0.0, 100.0, 100.0, 4, 4),
            grid(100.0, 0.0, 100.0, 100.0, 4, 4),
        ]);
        fx.style.color = red();

        // Act
        drag(&mut fx, &[(60.0, 10.0), (140.0, 10.0)]);

        // Assert
        assert_eq!(cells(&fx.shape(ids[0])), [(2, 0), (3, 0)]);
        assert_eq!(cells(&fx.shape(ids[1])), []);
    }

    #[test]
    fn drag_leaving_and_reentering_fills_only_inside() {
        // Arrange
        let (mut fx, id) = red_board();

        // Act: out through the right side, back in one row lower.
        drag(
            &mut fx,
            &[
                (85.0, 10.0),
                (160.0, 10.0),
                (160.0, 35.0),
                (85.0, 35.0),
            ],
        );

        // Assert
        assert_eq!(cells(&fx.shape(id)), [(3, 0), (3, 1)]);
    }

    #[test]
    fn drag_on_reversed_grid_counts_from_the_start_corner() {
        // Arrange: drawn from the bottom-right corner up to the origin.
        let (mut fx, ids) = Fixture::new(vec![grid(100.0, 100.0, -100.0, -100.0, 4, 4)]);
        fx.style.color = red();

        // Act: the top row of the screen, left to right.
        drag(&mut fx, &[(10.0, 10.0), (35.0, 10.0)]);

        // Assert: column 3 is the leftmost, row 3 the topmost.
        assert_eq!(cells(&fx.shape(ids[0])), [(2, 3), (3, 3)]);
    }

    #[test]
    fn drag_uses_the_current_colour() {
        // Arrange: a cell already filled with another colour.
        let (mut fx, id) = red_board();
        click(&mut fx, 35.0, 10.0);
        let blue = ColorId::new(3).unwrap_or(ColorId::INK);
        fx.style.color = blue;

        // Act
        drag(&mut fx, &[(10.0, 10.0), (35.0, 10.0)]);

        // Assert
        let board = fx.shape(id);
        assert_eq!(board.cell_fill(0, 0), Some(blue));
        assert_eq!(board.cell_fill(1, 0), Some(blue));
    }

    #[test]
    fn drag_preview_shows_the_painted_grid_over_the_hidden_one() {
        // Arrange
        let (mut fx, id) = red_board();
        let mut state = State::default();

        // Act
        send(&mut fx, &mut state, Phase::Down, 10.0, 10.0);
        let redraw = send(&mut fx, &mut state, Phase::Move, 35.0, 10.0);
        let overlay = preview(&state, &fx.view(Tool::Bucket));

        // Assert
        assert!(redraw);
        assert_eq!(overlay.hidden, [id]);
        assert_eq!(overlay.shapes.len(), 1);
        assert_eq!(cells(&overlay.shapes[0]), [(0, 0), (1, 0)]);
    }

    #[test]
    fn fast_move_fills_every_cell_on_the_segment() {
        // Arrange
        let (mut fx, id) = red_board();

        // Act: one move across the whole row.
        drag(&mut fx, &[(10.0, 60.0), (90.0, 60.0)]);

        // Assert
        assert_eq!(cells(&fx.shape(id)), [(0, 2), (1, 2), (2, 2), (3, 2)]);
    }

    #[test]
    fn fast_diagonal_move_leaves_no_gaps() {
        // Arrange
        let (mut fx, id) = red_board();

        // Act: corner to corner, slightly off the diagonal.
        drag(&mut fx, &[(5.0, 2.0), (95.0, 98.0)]);

        // Assert: one step at a time reaches seven cells.
        let filled = cells(&fx.shape(id));
        assert_eq!(filled.len(), 7, "{filled:?}");
        assert!(filled.contains(&(0, 0)));
        assert!(filled.contains(&(3, 3)));
    }

    #[test]
    fn release_far_from_the_last_move_walks_the_rest() {
        // Arrange
        let (mut fx, id) = red_board();
        let mut state = State::default();
        send(&mut fx, &mut state, Phase::Down, 10.0, 10.0);

        // Act: no move at all before the release.
        send(&mut fx, &mut state, Phase::Up, 90.0, 10.0);

        // Assert
        assert_eq!(cells(&fx.shape(id)), [(0, 0), (1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn cells_along_on_a_zero_size_grid_stays_in_range() {
        // Arrange
        let flat = grid(0.0, 0.0, 0.0, 100.0, 1, 4);

        // Act
        let walked = cells_along(&flat, Vec2::new(0.0, 10.0), Vec2::new(0.0, 90.0));

        // Assert
        assert!(walked.len() <= 1);
        assert!(walked.iter().all(|&(col, row)| col == 0 && row < 4));
    }

    #[test]
    fn drag_is_one_undo_step() {
        // Arrange
        let (mut fx, id) = red_board();
        drag(&mut fx, &[(10.0, 10.0), (35.0, 10.0), (60.0, 10.0)]);
        assert_eq!(fx.history.undo_len(), 1);

        // Act
        assert!(fx.history.undo(&mut fx.doc));

        // Assert
        assert_eq!(cells(&fx.shape(id)), []);
    }

    #[test]
    fn drag_changes_the_document_only_on_release() {
        // Arrange
        let (mut fx, id) = red_board();
        let mut state = State::default();

        // Act
        send(&mut fx, &mut state, Phase::Down, 10.0, 10.0);
        send(&mut fx, &mut state, Phase::Move, 35.0, 10.0);

        // Assert
        assert_eq!(cells(&fx.shape(id)), []);
        assert_eq!(fx.history.undo_len(), 0);
        send(&mut fx, &mut state, Phase::Up, 35.0, 10.0);
        assert_eq!(cells(&fx.shape(id)), [(0, 0), (1, 0)]);
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn drag_over_cells_of_the_same_colour_records_no_step() {
        // Arrange
        let (mut fx, _) = red_board();
        drag(&mut fx, &[(10.0, 10.0), (35.0, 10.0)]);
        assert_eq!(fx.history.undo_len(), 1);
        let mut state = State::default();

        // Act
        let pressed = send(&mut fx, &mut state, Phase::Down, 10.0, 10.0);
        let moved = send(&mut fx, &mut state, Phase::Move, 35.0, 10.0);
        let overlay = preview(&state, &fx.view(Tool::Bucket));
        let released = send(&mut fx, &mut state, Phase::Up, 35.0, 10.0);

        // Assert
        assert!(!pressed && !moved && !released);
        assert!(overlay.is_empty());
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn cancel_discards_the_drag() {
        // Arrange
        let (mut fx, id) = red_board();
        let mut state = State::default();
        send(&mut fx, &mut state, Phase::Down, 10.0, 10.0);
        send(&mut fx, &mut state, Phase::Move, 35.0, 10.0);

        // Act
        let redraw = cancel(&mut state);
        let released = send(&mut fx, &mut state, Phase::Up, 35.0, 10.0);

        // Assert
        assert!(redraw);
        assert!(!released);
        assert!(preview(&state, &fx.view(Tool::Bucket)).is_empty());
        assert_eq!(cells(&fx.shape(id)), []);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn press_on_rect_fills_at_once_and_ignores_the_drag() {
        // Arrange: a rectangle beside a grid.
        let (mut fx, ids) = Fixture::new(vec![
            rect(0.0, 0.0, 50.0, 50.0),
            grid(50.0, 0.0, 100.0, 100.0, 4, 4),
        ]);
        fx.style.color = red();
        let mut state = State::default();

        // Act
        assert!(send(&mut fx, &mut state, Phase::Down, 10.0, 10.0));
        let filled_on_press = fx.shape(ids[0]).fill();
        let moved = send(&mut fx, &mut state, Phase::Move, 90.0, 10.0);
        let released = send(&mut fx, &mut state, Phase::Up, 90.0, 10.0);

        // Assert
        assert_eq!(filled_on_press, Some(red()));
        assert!(!moved && !released);
        assert_eq!(cells(&fx.shape(ids[1])), []);
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn press_on_ellipse_fills_at_once_and_ignores_the_drag() {
        // Arrange
        let (mut fx, ids) = Fixture::new(vec![ellipse(0.0, 0.0, 50.0, 50.0)]);
        fx.style.color = red();
        let mut state = State::default();

        // Act
        assert!(send(&mut fx, &mut state, Phase::Down, 25.0, 25.0));
        let moved = send(&mut fx, &mut state, Phase::Move, 200.0, 200.0);

        // Assert
        assert_eq!(fx.shape(ids[0]).fill(), Some(red()));
        assert!(!moved);
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn press_on_empty_space_then_drag_into_grid_fills_nothing() {
        // Arrange
        let (mut fx, id) = red_board();

        // Act
        drag(&mut fx, &[(150.0, 10.0), (50.0, 10.0)]);

        // Assert
        assert_eq!(cells(&fx.shape(id)), []);
        assert_eq!(fx.history.undo_len(), 0);
    }

    proptest! {
        #[test]
        fn cells_along_is_connected_and_inside(
            (x, y) in (-500.0f32..500.0, -500.0f32..500.0),
            (w, h) in (1.0f32..400.0, 1.0f32..400.0),
            (flip_x, flip_y) in (any::<bool>(), any::<bool>()),
            (cols, rows) in (1u32..=64, 1u32..=64),
            from in (-1000.0f32..1000.0, -1000.0f32..1000.0),
            to in (-1000.0f32..1000.0, -1000.0f32..1000.0),
        ) {
            // Arrange
            let w = if flip_x { -w } else { w };
            let h = if flip_y { -h } else { h };
            let board = grid(x, y, w, h, cols, rows);
            let (from, to) = (Vec2::new(from.0, from.1), Vec2::new(to.0, to.1));

            // Act
            let walked = cells_along(&board, from, to);

            // Assert: in range, one step at a time, no longer than needed.
            prop_assert!(walked.len() <= (cols + rows) as usize);
            prop_assert!(walked.iter().all(|&(c, r)| c < cols && r < rows));
            for pair in walked.windows(2) {
                let step = pair[0].0.abs_diff(pair[1].0) + pair[0].1.abs_diff(pair[1].1);
                prop_assert_eq!(step, 1, "{:?}", walked);
            }
            // Endpoints inside the box are walked, give or take rounding.
            for (end, cell) in [(from, walked.first()), (to, walked.last())] {
                if let Some((col, row)) = board.grid_cell_at(end) {
                    let &(c, r) = cell.ok_or_else(|| TestCaseError::fail("no cells"))?;
                    prop_assert!(c.abs_diff(col) <= 1 && r.abs_diff(row) <= 1);
                }
            }
        }
    }
}
