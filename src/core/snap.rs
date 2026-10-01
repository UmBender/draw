//! Snapping of dragged points (T17).
//!
//! Smart snap (round shapes, matching sizes, alignment) and grid snap are
//! switched by [`Helpers`](crate::core::editor::Helpers) flags, which tools
//! read from [`ToolView::style`] (ADR-T16-3). Alignment guides go to
//! [`Overlay::guides`](crate::core::tools::Overlay::guides).
//!
//! Skeleton (T16): [`snap_end`] is the identity until T17 fills it in.

use crate::core::geom::Vec2;
use crate::core::tools::ToolView;

/// A snapped point and the guides that explain it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapped {
    /// The point after snapping, in world coordinates.
    pub point: Vec2,
    /// Alignment guides as world-space segments.
    pub guides: Vec<[Vec2; 2]>,
}

/// Snaps the moving end `end` of a drag that started at `start` (both in
/// world coordinates) according to the helper flags in `view`.
///
/// Skeleton: returns `end` unchanged with no guides.
#[must_use]
pub fn snap_end(start: Vec2, end: Vec2, view: &ToolView<'_>) -> Snapped {
    let _ = (start, view);
    Snapped {
        point: end,
        guides: Vec::new(),
    }
}
