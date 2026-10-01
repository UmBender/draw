//! Copy, cut, paste and duplicate of selected shapes.
//!
//! Owned by T10; the signatures are fixed by ADR-T08-1, the bodies are stubs
//! until T10 fills them in. The clipboard is internal to the editor (no system
//! clipboard).

use crate::core::shape::Shape;
use crate::core::tools::ToolCtx;

/// Shapes copied from the document.
#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    /// Copied shapes, in z-order.
    shapes: Vec<Shape>,
}

impl Clipboard {
    /// The copied shapes, in z-order.
    #[must_use]
    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }
}

/// Copies the selected shapes. Returns whether a redraw is needed.
pub fn copy(ctx: &mut ToolCtx<'_>) -> bool {
    let _ = ctx;
    false
}

/// Copies, then deletes the selected shapes (one undo step). Returns whether
/// the document changed.
pub fn cut(ctx: &mut ToolCtx<'_>) -> bool {
    let _ = ctx;
    false
}

/// Pastes the clipboard centred at `ctx.cursor` and selects the copies (one
/// undo step). Returns whether the document changed.
pub fn paste(ctx: &mut ToolCtx<'_>) -> bool {
    let _ = ctx;
    false
}

/// Duplicates the selection with a small screen offset (one undo step).
/// Returns whether the document changed.
pub fn duplicate(ctx: &mut ToolCtx<'_>) -> bool {
    let _ = ctx;
    false
}
