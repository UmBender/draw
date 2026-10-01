//! Geometry primitives: `Vec2`, `Aabb`, distance helpers.
//!
//! Owned by T01; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const EPS: f32 = 1e-5;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn assert_vec_eq(actual: Vec2, expected: Vec2) {
        assert!(
            actual.approx_eq(expected, EPS),
            "expected {expected:?}, got {actual:?}"
        );
    }

    fn assert_f32_eq(actual: f32, expected: f32) {
        assert!(
            approx_eq(actual, expected, EPS),
            "expected {expected}, got {actual}"
        );
    }

    fn assert_aabb_eq(actual: Aabb, min: Vec2, max: Vec2) {
        assert_vec_eq(actual.min, min);
        assert_vec_eq(actual.max, max);
    }

    // ---- AC-5: approx_eq ----

    #[test]
    fn approx_eq_within_eps_is_true() {
        assert!(approx_eq(0.1 + 0.2, 0.3, 1e-6));
        assert!(approx_eq(1.0, 1.0, 0.0));
        assert!(approx_eq(-2.0, -2.05, 0.1));
    }

    #[test]
    fn approx_eq_outside_eps_is_false() {
        assert!(!approx_eq(1.0, 1.1, 0.05));
        assert!(!approx_eq(0.0, -1.0, 0.5));
    }

    #[test]
    fn approx_eq_nan_or_inf_is_false() {
        assert!(!approx_eq(f32::NAN, f32::NAN, 1.0));
        assert!(!approx_eq(f32::NAN, 0.0, f32::MAX));
        assert!(!approx_eq(f32::INFINITY, f32::INFINITY, 1.0));
        assert!(!approx_eq(f32::NEG_INFINITY, 0.0, f32::MAX));
        assert!(!approx_eq(0.0, 0.0, f32::NAN));
    }

    // ---- AC-1: Vec2 ----

    #[test]
    fn vec2_new_and_zero_set_components() {
        let a = Vec2::new(1.5, -2.0);

        assert_f32_eq(a.x, 1.5);
        assert_f32_eq(a.y, -2.0);
        assert_vec_eq(Vec2::ZERO, v(0.0, 0.0));
        assert_vec_eq(Vec2::default(), Vec2::ZERO);
    }

    #[test]
    fn vec2_add_sub_are_component_wise() {
        let a = v(1.0, 2.0);
        let b = v(3.0, -5.0);

        assert_vec_eq(a + b, v(4.0, -3.0));
        assert_vec_eq(a - b, v(-2.0, 7.0));
    }

    #[test]
    fn vec2_mul_div_by_scalar_scale_components() {
        let a = v(2.0, -4.0);

        assert_vec_eq(a * 1.5, v(3.0, -6.0));
        assert_vec_eq(a / 2.0, v(1.0, -2.0));
    }

    #[test]
    fn vec2_neg_and_assign_ops_match_binary_ops() {
        let a = v(1.0, -2.0);
        let b = v(0.5, 4.0);
        let mut sum = a;
        let mut diff = a;

        sum += b;
        diff -= b;

        assert_vec_eq(-a, v(-1.0, 2.0));
        assert_vec_eq(sum, a + b);
        assert_vec_eq(diff, a - b);
    }

    #[test]
    fn vec2_length_of_3_4_is_5() {
        assert_f32_eq(v(3.0, 4.0).length(), 5.0);
        assert_f32_eq(v(-3.0, -4.0).length(), 5.0);
        assert_f32_eq(Vec2::ZERO.length(), 0.0);
    }

    #[test]
    fn vec2_length_sq_avoids_sqrt() {
        assert_f32_eq(v(3.0, 4.0).length_sq(), 25.0);
        assert_f32_eq(v(-1.0, 2.0).length_sq(), 5.0);
    }

    #[test]
    fn vec2_dot_of_perpendicular_is_zero() {
        assert_f32_eq(v(1.0, 2.0).dot(v(-2.0, 1.0)), 0.0);
        assert_f32_eq(v(1.0, 2.0).dot(v(3.0, 4.0)), 11.0);
    }

    #[test]
    fn vec2_distance_is_symmetric() {
        let a = v(1.0, 1.0);
        let b = v(4.0, 5.0);

        assert_f32_eq(a.distance(b), 5.0);
        assert_f32_eq(b.distance(a), 5.0);
        assert_f32_eq(a.distance(a), 0.0);
    }

    #[test]
    fn vec2_lerp_hits_endpoints_and_midpoint() {
        let a = v(0.0, 10.0);
        let b = v(4.0, -2.0);

        assert_vec_eq(a.lerp(b, 0.0), a);
        assert_vec_eq(a.lerp(b, 1.0), b);
        assert_vec_eq(a.lerp(b, 0.5), v(2.0, 4.0));
        assert_vec_eq(a.lerp(b, 2.0), v(8.0, -14.0));
    }

    #[test]
    fn vec2_approx_eq_uses_per_component_tolerance() {
        let a = v(1.0, 2.0);

        assert!(a.approx_eq(v(1.05, 1.95), 0.1));
        assert!(!a.approx_eq(v(1.0, 2.2), 0.1));
        assert!(!a.approx_eq(v(1.2, 2.0), 0.1));
        assert!(!a.approx_eq(v(f32::NAN, 2.0), 0.1));
    }

    // ---- AC-2: finiteness ----

    #[test]
    fn vec2_is_finite_detects_nan_and_inf() {
        assert!(v(1.0, -1.0).is_finite());
        assert!(!v(f32::NAN, 0.0).is_finite());
        assert!(!v(0.0, f32::INFINITY).is_finite());
        assert!(!v(f32::NEG_INFINITY, 0.0).is_finite());
    }

    #[test]
    fn vec2_sanitize_rejects_nan_and_inf() {
        assert!(v(f32::NAN, 0.0).sanitize().is_none());
        assert!(v(0.0, f32::NAN).sanitize().is_none());
        assert!(v(f32::INFINITY, 0.0).sanitize().is_none());
        assert!(v(0.0, f32::NEG_INFINITY).sanitize().is_none());
    }

    #[test]
    fn vec2_sanitize_keeps_finite_values() {
        let a = v(3.0, -7.5);

        let sanitized = a.sanitize();

        assert!(sanitized.is_some_and(|s| s.approx_eq(a, 0.0)));
    }

    // ---- AC-3: Aabb ----

    #[test]
    fn aabb_from_points_empty_is_none() {
        assert!(Aabb::from_points(&[]).is_none());
    }

    #[test]
    fn aabb_from_points_spans_all_points() {
        let points = [v(1.0, 5.0), v(-2.0, 3.0), v(4.0, -1.0)];

        let bounds = Aabb::from_points(&points);

        let Some(bounds) = bounds else {
            panic!("expected Some for non-empty input");
        };
        assert_aabb_eq(bounds, v(-2.0, -1.0), v(4.0, 5.0));
    }

    #[test]
    fn aabb_from_points_skips_non_finite() {
        let points = [
            v(f32::NAN, 0.0),
            v(1.0, 2.0),
            v(0.0, f32::INFINITY),
            v(3.0, 1.0),
        ];

        let bounds = Aabb::from_points(&points);

        let Some(bounds) = bounds else {
            panic!("expected Some when finite points exist");
        };
        assert_aabb_eq(bounds, v(1.0, 1.0), v(3.0, 2.0));
        assert!(Aabb::from_points(&[v(f32::NAN, f32::NAN)]).is_none());
    }

    #[test]
    fn aabb_from_corners_is_order_independent() {
        let min = v(-1.0, 2.0);
        let max = v(3.0, 6.0);

        assert_aabb_eq(Aabb::from_corners(v(-1.0, 2.0), v(3.0, 6.0)), min, max);
        assert_aabb_eq(Aabb::from_corners(v(3.0, 6.0), v(-1.0, 2.0)), min, max);
        assert_aabb_eq(Aabb::from_corners(v(-1.0, 6.0), v(3.0, 2.0)), min, max);
    }

    #[test]
    fn aabb_expand_grows_each_side() {
        let b = Aabb::from_corners(v(0.0, 0.0), v(2.0, 4.0));

        assert_aabb_eq(b.expand(1.5), v(-1.5, -1.5), v(3.5, 5.5));
    }

    #[test]
    fn aabb_expand_negative_collapses_to_center() {
        let b = Aabb::from_corners(v(0.0, 0.0), v(2.0, 4.0));

        assert_aabb_eq(b.expand(-0.5), v(0.5, 0.5), v(1.5, 3.5));
        // x would invert at -1.5: it collapses to the centre while y still shrinks.
        assert_aabb_eq(b.expand(-1.5), v(1.0, 1.5), v(1.0, 2.5));
        assert_aabb_eq(b.expand(-10.0), v(1.0, 2.0), v(1.0, 2.0));
    }

    #[test]
    fn aabb_contains_is_inclusive() {
        let b = Aabb::from_corners(v(0.0, 0.0), v(2.0, 2.0));

        assert!(b.contains(v(1.0, 1.0)));
        assert!(b.contains(v(0.0, 0.0)));
        assert!(b.contains(v(2.0, 1.0)));
        assert!(!b.contains(v(2.1, 1.0)));
        assert!(!b.contains(v(1.0, -0.1)));
        assert!(!b.contains(v(f32::NAN, 1.0)));
    }

    #[test]
    fn aabb_intersects_overlapping_and_touching() {
        let a = Aabb::from_corners(v(0.0, 0.0), v(2.0, 2.0));
        let overlapping = Aabb::from_corners(v(1.0, 1.0), v(3.0, 3.0));
        let touching = Aabb::from_corners(v(2.0, 0.0), v(4.0, 2.0));
        let inside = Aabb::from_corners(v(0.5, 0.5), v(1.0, 1.0));

        assert!(a.intersects(&overlapping));
        assert!(overlapping.intersects(&a));
        assert!(a.intersects(&touching));
        assert!(a.intersects(&inside));
        assert!(inside.intersects(&a));
    }

    #[test]
    fn aabb_intersects_disjoint_is_false() {
        let a = Aabb::from_corners(v(0.0, 0.0), v(2.0, 2.0));
        let right = Aabb::from_corners(v(2.5, 0.0), v(4.0, 2.0));
        let above = Aabb::from_corners(v(0.0, 3.0), v(2.0, 4.0));

        assert!(!a.intersects(&right));
        assert!(!right.intersects(&a));
        assert!(!a.intersects(&above));
    }

    #[test]
    fn aabb_union_covers_both() {
        let a = Aabb::from_corners(v(0.0, 0.0), v(1.0, 1.0));
        let b = Aabb::from_corners(v(3.0, -2.0), v(4.0, 0.5));

        let u = a.union(&b);

        assert_aabb_eq(u, v(0.0, -2.0), v(4.0, 1.0));
        assert_aabb_eq(b.union(&a), v(0.0, -2.0), v(4.0, 1.0));
    }

    #[test]
    fn aabb_center_width_height() {
        let b = Aabb::from_corners(v(-1.0, 2.0), v(3.0, 8.0));

        assert_vec_eq(b.center(), v(1.0, 5.0));
        assert_f32_eq(b.width(), 4.0);
        assert_f32_eq(b.height(), 6.0);
    }

    #[test]
    fn aabb_translate_moves_both_corners() {
        let b = Aabb::from_corners(v(0.0, 0.0), v(2.0, 1.0));

        assert_aabb_eq(b.translate(v(5.0, -3.0)), v(5.0, -3.0), v(7.0, -2.0));
    }

    // ---- AC-4: distance_to_segment ----

    #[test]
    fn segment_distance_interior_projection_is_perpendicular() {
        let horizontal = distance_to_segment(v(2.0, 3.0), v(0.0, 0.0), v(4.0, 0.0));
        let diagonal = distance_to_segment(v(0.0, 2.0), v(-1.0, 1.0), v(1.0, -1.0));

        assert_f32_eq(horizontal, 3.0);
        assert_f32_eq(diagonal, 2.0_f32.sqrt());
    }

    #[test]
    fn segment_distance_before_start_is_distance_to_a() {
        let d = distance_to_segment(v(-3.0, 4.0), v(0.0, 0.0), v(4.0, 0.0));

        assert_f32_eq(d, 5.0);
    }

    #[test]
    fn segment_distance_after_end_is_distance_to_b() {
        let d = distance_to_segment(v(7.0, -4.0), v(0.0, 0.0), v(4.0, 0.0));

        assert_f32_eq(d, 5.0);
    }

    #[test]
    fn segment_distance_degenerate_segment_is_point_distance() {
        let a = v(1.0, 1.0);

        let d = distance_to_segment(v(4.0, 5.0), a, a);

        assert_f32_eq(d, 5.0);
        assert_f32_eq(distance_to_segment(a, a, a), 0.0);
    }

    #[test]
    fn segment_distance_point_on_segment_is_zero() {
        let a = v(0.0, 0.0);
        let b = v(2.0, 2.0);

        assert_f32_eq(distance_to_segment(v(1.0, 1.0), a, b), 0.0);
        assert_f32_eq(distance_to_segment(b, a, b), 0.0);
    }

    fn coord() -> impl Strategy<Value = f32> {
        -1.0e3_f32..1.0e3_f32
    }

    fn point() -> impl Strategy<Value = Vec2> {
        (coord(), coord()).prop_map(|(x, y)| Vec2::new(x, y))
    }

    proptest! {
        #[test]
        fn segment_distance_never_exceeds_endpoint_distance(
            p in point(), a in point(), b in point()
        ) {
            let d = distance_to_segment(p, a, b);
            let nearest_endpoint = p.distance(a).min(p.distance(b));

            // f32 rounding on coordinates up to 1e3 stays well below 1e-2.
            prop_assert!(d <= nearest_endpoint + 1e-2, "d={d}, endpoint={nearest_endpoint}");
        }

        #[test]
        fn segment_distance_is_non_negative_and_finite(
            p in point(), a in point(), b in point()
        ) {
            let d = distance_to_segment(p, a, b);

            prop_assert!(d.is_finite());
            prop_assert!(d >= 0.0);
        }
    }
}
