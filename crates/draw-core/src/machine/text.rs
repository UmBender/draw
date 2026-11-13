//! The text edit's anchor (ADR-T153-1): what an open text block is attached
//! to. Tests first; the types follow.

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
