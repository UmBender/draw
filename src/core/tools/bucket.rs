//! Bucket: fills the clicked closed shape.
//!
//! A press sets the fill of the topmost [`Shape::Rect`] or [`Shape::Ellipse`]
//! containing the point to the current colour, as one undo step. Refilling
//! with the same colour, or pressing on empty space or an open shape, changes
//! nothing and records no step. The bucket has no gesture to preview.
//!
//! [`Shape::Rect`]: crate::core::shape::Shape::Rect
//! [`Shape::Ellipse`]: crate::core::shape::Shape::Ellipse

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
    let Some(id) = ctx
        .doc
        .topmost_where(|shape| shape.is_closed() && shape.contains(world))
    else {
        return false;
    };
    let Some(shape) = ctx.doc.get(id).filter(|shape| shape.fill() != fill) else {
        return false;
    };
    let tx = tx_replace(ctx.doc, id, shape.clone().with_fill(fill));
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
    use crate::core::palette::ColorId;
    use crate::core::tools::Phase;

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
}
