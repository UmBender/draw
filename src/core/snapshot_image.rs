#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(w: f32, h: f32) -> Aabb {
        Aabb::from_corners(Vec2::new(10.0, 20.0), Vec2::new(10.0 + w, 20.0 + h))
    }

    #[test]
    fn selection_image_box_adds_margin() {
        // Arrange: a 100 × 50 box at zoom 1.
        let plan = plan(bounds(100.0, 50.0), 1.0).expect("finite box");

        // Assert: 16 px on every side.
        assert_eq!(plan.size, (132, 82));
        assert!(!plan.scaled_down);
        assert!(approx_eq(plan.world.min.x, -6.0, 1e-3));
        assert!(approx_eq(plan.world.max.y, 86.0, 1e-3));
        assert!(approx_eq(plan.scale, 1.0, 1e-6));
    }

    #[test]
    fn selection_image_zoom_below_one_is_raised_to_one() {
        let plan = plan(bounds(100.0, 50.0), 0.25).expect("finite box");

        assert_eq!(plan.size, (132, 82));
        assert!(approx_eq(plan.scale, 1.0, 1e-6));
    }

    #[test]
    fn selection_image_zoom_above_one_is_kept() {
        let plan = plan(bounds(100.0, 50.0), 2.0).expect("finite box");

        // 100 · 2 + 32 and 50 · 2 + 32.
        assert_eq!(plan.size, (232, 132));
        assert!(approx_eq(plan.scale, 2.0, 1e-6));
    }

    #[test]
    fn selection_image_size_scales_down_above_limit() {
        let plan = plan(bounds(20_000.0, 1_000.0), 1.0).expect("finite box");

        assert_eq!(plan.size.0, MAX_SIDE_PX);
        assert!(plan.size.1 < MAX_SIDE_PX);
        assert!(plan.scaled_down);
        assert!(plan.scale < 1.0);
    }

    #[test]
    fn selection_image_of_a_point_is_just_the_margin() {
        let plan = plan(bounds(0.0, 0.0), 1.0).expect("finite box");

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
        let plan = plan(bounds(100.0, 50.0), f32::NAN).expect("finite box");

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
