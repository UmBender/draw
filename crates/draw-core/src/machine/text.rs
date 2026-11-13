//! The text edit's anchor (ADR-T153-1): what an open text block is attached
//! to. A block is new, a re-opened text block of the document, or the text
//! slot of a shape; a block that is both a document block and a slot is not
//! a variant, so it cannot be built.

use crate::core::document::ShapeId;
use crate::core::geom::Aabb;
use crate::core::shape::TextSlot;

/// The shape whose text an edit changes (ADR-T31-1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Host {
    /// The rectangle, ellipse or grid.
    pub id: ShapeId,
    /// Its interior or the grid cell.
    pub slot: TextSlot,
    /// The world box the text is fitted into (`Shape::text_frame`).
    pub frame: Aabb,
    /// The slot is filled, so its text is drawn in the background colour.
    pub filled: bool,
}

/// What an open text edit is attached to: one variant per state the code
/// reaches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    /// A new block, not in the document until the edit commits.
    New,
    /// The document text block being re-edited.
    Block(ShapeId),
    /// The text slot of a shape: re-fitted into the host's frame as typed.
    Slot(Host),
}

/// A snapshot record named both a document block and a host shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetWithHost;

impl Anchor {
    /// The anchor of a record's `target` and `host` columns. Both set is
    /// no state.
    ///
    /// # Errors
    ///
    /// [`TargetWithHost`] if both are set.
    pub fn from_parts(
        target: Option<ShapeId>,
        host: Option<Host>,
    ) -> Result<Self, TargetWithHost> {
        match (target, host) {
            (None, None) => Ok(Self::New),
            (Some(id), None) => Ok(Self::Block(id)),
            (None, Some(host)) => Ok(Self::Slot(host)),
            (Some(_), Some(_)) => Err(TargetWithHost),
        }
    }

    /// The document block being re-edited, if any.
    #[must_use]
    pub fn target(&self) -> Option<ShapeId> {
        match self {
            Self::New | Self::Slot(_) => None,
            Self::Block(id) => Some(*id),
        }
    }

    /// The shape whose text this is, if any.
    #[must_use]
    pub fn host(&self) -> Option<&Host> {
        match self {
            Self::New | Self::Block(_) => None,
            Self::Slot(host) => Some(host),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::ShapeId;
    use crate::core::geom::{Aabb, Vec2};
    use crate::core::shape::TextSlot;

    fn host() -> Host {
        Host {
            id: ShapeId(7),
            slot: TextSlot::Shape,
            frame: Aabb::from_corners(Vec2::ZERO, Vec2::new(10.0, 5.0)),
            filled: false,
        }
    }

    #[test]
    fn anchor_new_has_no_target_or_host() {
        let anchor = Anchor::New;
        assert_eq!(anchor.target(), None);
        assert_eq!(anchor.host(), None);
    }

    #[test]
    fn anchor_block_has_target_only() {
        let anchor = Anchor::Block(ShapeId(3));
        assert_eq!(anchor.target(), Some(ShapeId(3)));
        assert_eq!(anchor.host(), None);
    }

    #[test]
    fn anchor_slot_has_host_only() {
        let anchor = Anchor::Slot(host());
        assert_eq!(anchor.target(), None);
        assert_eq!(anchor.host(), Some(&host()));
    }

    #[test]
    fn anchor_from_parts_maps_each_reached_combination() {
        assert_eq!(Anchor::from_parts(None, None), Ok(Anchor::New));
        assert_eq!(
            Anchor::from_parts(Some(ShapeId(3)), None),
            Ok(Anchor::Block(ShapeId(3)))
        );
        assert_eq!(
            Anchor::from_parts(None, Some(host())),
            Ok(Anchor::Slot(host()))
        );
    }

    #[test]
    fn anchor_from_parts_rejects_target_with_host() {
        assert_eq!(
            Anchor::from_parts(Some(ShapeId(3)), Some(host())),
            Err(TargetWithHost)
        );
    }
}
