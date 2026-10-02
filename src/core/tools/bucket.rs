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
use crate::core::document::tx_replace;

/// Gesture state of this tool: the bucket acts on press and keeps none.
#[derive(Debug, Clone, Default)]
pub struct State;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::clipboard::testkit::{Fixture, ellipse, line, ptr, rect};
    use crate::core::command::Tool;
    use crate::core::geom::Vec2;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use crate::core::tools::Phase;

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
        }
    }

    fn red() -> ColorId {
        ColorId::new(2).unwrap_or(ColorId::INK)
    }

    fn click(fx: &mut Fixture, x: f32, y: f32) -> bool {
        let mut state = State;
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
        let mut state = State;

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

        assert!(fx.shape(ids[0]).cell_fills().is_empty());
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
        assert!(fx.shape(ids[2]).cell_fills().is_empty());
        assert_eq!(fx.shape(ids[3]).fill(), Some(red()));
    }

    #[test]
    fn click_outside_grid_box_does_nothing() {
        let (mut fx, ids) = Fixture::new(vec![grid(0.0, 0.0, 100.0, 100.0, 4, 4)]);

        assert!(!click(&mut fx, 150.0, 50.0));

        assert!(fx.shape(ids[0]).cell_fills().is_empty());
        assert_eq!(fx.history.undo_len(), 0);
    }
}
