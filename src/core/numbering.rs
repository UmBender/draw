//! Auto-numbering of new nodes (T19).
//!
//! With numbering on, every committed [`Shape::Rect`] or [`Shape::Ellipse`]
//! gets the next number as its label (ADR-T16-1). The editor stores the
//! switch and the counter in [`Helpers`] (ADR-T16-3).
//!
//! Skeleton (T16): [`label_new`] returns the shape unchanged until T19
//! fills it in.

use crate::core::editor::Helpers;
use crate::core::shape::Shape;

/// The first number given out, and the counter value after a reset.
pub const FIRST_NUMBER: u32 = 1;

/// `shape` with the label numbering gives a newly committed shape under
/// `helpers`.
///
/// Skeleton: returns `shape` unchanged.
#[must_use]
pub fn label_new(shape: Shape, helpers: &Helpers) -> Shape {
    let _ = helpers;
    shape
}
