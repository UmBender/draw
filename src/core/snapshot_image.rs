//! Crop box and size limit of the selection image `Ctrl+Shift+C` puts on the
//! clipboard (ADR-T115-1). Pure geometry: the shell renders and encodes.

use crate::core::geom::Aabb;

/// Margin around the selection's box, in image pixels.
pub const MARGIN_PX: f32 = 16.0;

/// Largest width or height of the image, in pixels.
pub const MAX_SIDE_PX: u32 = 4096;

/// Where and how large the selection image is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImagePlan {
    /// World rectangle the image shows: the selection's box plus the margin.
    pub world: Aabb,
    /// Image pixels per world unit: the zoom (at least 1), scaled down if
    /// the image would exceed [`MAX_SIDE_PX`].
    pub scale: f32,
    /// Width and height in pixels, each in `1..=MAX_SIDE_PX`.
    pub size: (u32, u32),
    /// Whether the limit forced `scale` below the zoom.
    pub scaled_down: bool,
}

/// The image of the world box `bounds` at camera `zoom`. `None` if `bounds`
/// is not finite; a non-finite or non-positive `zoom` counts as 1.
#[must_use]
pub fn plan(bounds: Aabb, zoom: f32) -> Option<ImagePlan> {
    let finite = [bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y]
        .iter()
        .all(|v| v.is_finite());
    if !finite {
        return None;
    }
    let zoom = if zoom.is_finite() { zoom.max(1.0) } else { 1.0 };
    let margin = f64::from(MARGIN_PX);
    let limit = f64::from(MAX_SIDE_PX);
    // f64 keeps boxes as wide as f32 allows from overflowing.
    let width = f64::from(bounds.max.x) - f64::from(bounds.min.x);
    let height = f64::from(bounds.max.y) - f64::from(bounds.min.y);
    let wanted = f64::from(zoom);
    let longest = (width.max(height) * wanted + 2.0 * margin).max(1.0);
    let shrink = (limit / longest).min(1.0);
    let scale = wanted * shrink;
    let side = |extent: f64| {
        ((extent * scale + 2.0 * margin * shrink)
            .ceil()
            .clamp(1.0, limit)) as u32
    };
    // The margin is MARGIN_PX image pixels, so it is shorter in world units
    // at a larger scale.
    let world_margin = margin * shrink / scale;
    Some(ImagePlan {
        world: bounds.expand(world_margin as f32),
        scale: scale as f32,
        size: (side(width), side(height)),
        scaled_down: shrink < 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Vec2, approx_eq};

    fn bounds(w: f32, h: f32) -> Aabb {
        Aabb::from_corners(Vec2::new(10.0, 20.0), Vec2::new(10.0 + w, 20.0 + h))
    }

    /// The plan of a finite box.
    fn planned(b: Aabb, zoom: f32) -> ImagePlan {
        plan(b, zoom).unwrap_or_else(|| panic!("no plan for a finite box"))
    }

    #[test]
    fn selection_image_box_adds_margin() {
        // Arrange: a 100 × 50 box at zoom 1.
        let plan = planned(bounds(100.0, 50.0), 1.0);

        // Assert: 16 px on every side.
        assert_eq!(plan.size, (132, 82));
        assert!(!plan.scaled_down);
        assert!(approx_eq(plan.world.min.x, -6.0, 1e-3));
        assert!(approx_eq(plan.world.max.y, 86.0, 1e-3));
        assert!(approx_eq(plan.scale, 1.0, 1e-6));
    }

    #[test]
    fn selection_image_zoom_below_one_is_raised_to_one() {
        let plan = planned(bounds(100.0, 50.0), 0.25);

        assert_eq!(plan.size, (132, 82));
        assert!(approx_eq(plan.scale, 1.0, 1e-6));
    }

    #[test]
    fn selection_image_zoom_above_one_is_kept() {
        let plan = planned(bounds(100.0, 50.0), 2.0);

        // 100 · 2 + 32 and 50 · 2 + 32.
        assert_eq!(plan.size, (232, 132));
        assert!(approx_eq(plan.scale, 2.0, 1e-6));
    }

    #[test]
    fn selection_image_size_scales_down_above_limit() {
        let plan = planned(bounds(20_000.0, 1_000.0), 1.0);

        assert_eq!(plan.size.0, MAX_SIDE_PX);
        assert!(plan.size.1 < MAX_SIDE_PX);
        assert!(plan.scaled_down);
        assert!(plan.scale < 1.0);
    }

    #[test]
    fn selection_image_of_a_point_is_just_the_margin() {
        let plan = planned(bounds(0.0, 0.0), 1.0);

        assert_eq!(plan.size, (32, 32));
    }

    #[test]
    fn selection_image_with_non_finite_bounds_is_none() {
        let bad = Aabb {
            min: Vec2::new(f32::NAN, 0.0),
            max: Vec2::new(1.0, f32::INFINITY),
        };

        assert!(plan(bad, 1.0).is_none());
    }

    #[test]
    fn selection_image_with_non_finite_zoom_uses_one() {
        let plan = planned(bounds(100.0, 50.0), f32::NAN);

        assert_eq!(plan.size, (132, 82));
    }

    mod fuzz {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn selection_image_size_fuzz(
                x in -1e9f32..1e9, y in -1e9f32..1e9,
                w in 0.0f32..1e9, h in 0.0f32..1e9,
                zoom in prop_oneof![0.0f32..1000.0, Just(f32::NAN), Just(f32::INFINITY)],
            ) {
                let b = Aabb::from_corners(Vec2::new(x, y), Vec2::new(x + w, y + h));
                if let Some(plan) = plan(b, zoom) {
                    prop_assert!(plan.size.0 >= 1 && plan.size.0 <= MAX_SIDE_PX);
                    prop_assert!(plan.size.1 >= 1 && plan.size.1 <= MAX_SIDE_PX);
                    prop_assert!(plan.scale.is_finite() && plan.scale > 0.0);
                }
            }
        }
    }
}
