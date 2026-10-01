//! Eraser: removes whole shapes it touches (also right drag from any tool).
//!
//! Owned by T10; the signatures are fixed by ADR-T08-1, the bodies are stubs
//! until T10 fills them in.

use super::{Overlay, Pointer, ToolCtx, ToolView};

/// Gesture state of this tool.
#[derive(Debug, Clone, Default)]
pub struct State;

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    let _ = (state, ctx, pointer);
    false
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
