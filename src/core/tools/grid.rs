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
