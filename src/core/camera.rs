//! Camera: world/screen transform, pan and zoom.
//!
//! Owned by T03; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Aabb, Vec2, approx_eq};
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

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

    /// Exact equality of two cameras (tolerance 0), without `==` on floats.
    fn assert_camera_unchanged(actual: Camera, expected: Camera) {
        assert!(
            actual.offset().approx_eq(expected.offset(), 0.0)
                && approx_eq(actual.zoom(), expected.zoom(), 0.0),
            "expected {expected:?}, got {actual:?}"
        );
    }

    fn sample_camera() -> Camera {
        Camera::new(v(10.0, -20.0), 2.5)
    }

    // ---- AC-1: model and defaults ----

    #[test]
    fn default_is_identity() {
        let cam = Camera::default();

        assert_vec_eq(cam.offset(), Vec2::ZERO);
        assert_f32_eq(cam.zoom(), 1.0);
        assert_vec_eq(cam.world_to_screen(v(3.0, -4.0)), v(3.0, -4.0));
        assert_vec_eq(cam.screen_to_world(v(3.0, -4.0)), v(3.0, -4.0));
        assert_f32_eq(ZOOM_MIN, 0.05);
        assert_f32_eq(ZOOM_MAX, 20.0);
        assert_f32_eq(ZOOM_STEP, 1.15);
    }

    #[test]
    fn new_clamps_zoom_and_rejects_non_finite() {
        let too_far_in = Camera::new(v(1.0, 2.0), 1000.0);
        let too_far_out = Camera::new(v(1.0, 2.0), 0.0);
        let negative = Camera::new(v(1.0, 2.0), -3.0);
        let nan_zoom = Camera::new(v(1.0, 2.0), f32::NAN);
        let inf_zoom = Camera::new(v(1.0, 2.0), f32::INFINITY);
        let nan_offset = Camera::new(v(f32::NAN, 2.0), 3.0);

        assert_vec_eq(too_far_in.offset(), v(1.0, 2.0));
        assert_f32_eq(too_far_in.zoom(), ZOOM_MAX);
        assert_f32_eq(too_far_out.zoom(), ZOOM_MIN);
        assert_f32_eq(negative.zoom(), ZOOM_MIN);
        assert_f32_eq(nan_zoom.zoom(), 1.0);
        assert_f32_eq(inf_zoom.zoom(), 1.0);
        assert_vec_eq(nan_offset.offset(), Vec2::ZERO);
        assert_f32_eq(nan_offset.zoom(), 3.0);
    }

    // ---- AC-2: transforms ----

    #[test]
    fn transform_examples_match_formula() {
        let cam = sample_camera();

        // screen = (world - offset) * zoom
        assert_vec_eq(cam.world_to_screen(v(10.0, -20.0)), Vec2::ZERO);
        assert_vec_eq(cam.world_to_screen(v(14.0, -18.0)), v(10.0, 5.0));
        // world = screen / zoom + offset
        assert_vec_eq(cam.screen_to_world(v(10.0, 5.0)), v(14.0, -18.0));
        assert!(!cam.world_to_screen(v(f32::NAN, 0.0)).is_finite());
        assert!(!cam.screen_to_world(v(0.0, f32::INFINITY)).is_finite());
    }

    // ---- AC-3: pan ----

    #[test]
    fn pan_moves_content_by_screen_delta() {
        let mut cam = sample_camera();
        let world_point = v(7.0, 3.0);
        let delta = v(12.0, -30.0);
        let before = cam.world_to_screen(world_point);

        cam.pan_by_screen(delta);

        assert_vec_eq(cam.world_to_screen(world_point), before + delta);
        assert_f32_eq(cam.zoom(), 2.5);
    }

    // ---- AC-4: zoom ----

    #[test]
    fn zoom_one_notch_multiplies_by_step() {
        let mut cam = Camera::default();

        cam.zoom_at(Vec2::ZERO, 1.0);
        assert_f32_eq(cam.zoom(), ZOOM_STEP);

        cam.zoom_at(Vec2::ZERO, -2.0);
        assert_f32_eq(cam.zoom(), 1.0 / ZOOM_STEP);
    }

    #[test]
    fn zoom_keeps_anchor_fixed() {
        let mut cam = sample_camera();
        let anchor = v(320.0, 240.0);
        let world_before = cam.screen_to_world(anchor);

        cam.zoom_at(anchor, 3.0);

        assert_vec_eq(cam.screen_to_world(anchor), world_before);
        assert_vec_eq(cam.world_to_screen(world_before), anchor);
        assert_f32_eq(cam.zoom(), 2.5 * ZOOM_STEP.powi(3));
    }

    #[test]
    fn zoom_clamped_to_range() {
        let mut cam = sample_camera();
        let anchor = v(100.0, 50.0);
        let world_before = cam.screen_to_world(anchor);

        cam.zoom_at(anchor, 1000.0);
        assert_f32_eq(cam.zoom(), ZOOM_MAX);
        assert_vec_eq(cam.screen_to_world(anchor), world_before);

        cam.zoom_at(anchor, -1000.0);
        assert_f32_eq(cam.zoom(), ZOOM_MIN);
        assert_vec_eq(cam.screen_to_world(anchor), world_before);
    }

    // ---- AC-5: non-finite input ----

    #[test]
    fn non_finite_input_is_ignored() {
        let start = sample_camera();
        let bounds = Aabb::from_corners(v(0.0, 0.0), v(10.0, 10.0));
        let viewport = v(800.0, 600.0);
        let mut cam = start;

        cam.pan_by_screen(v(f32::NAN, 1.0));
        cam.pan_by_screen(v(1.0, f32::INFINITY));
        cam.zoom_at(v(f32::NAN, 0.0), 1.0);
        cam.zoom_at(v(0.0, 0.0), f32::NAN);
        cam.zoom_at(v(0.0, 0.0), f32::INFINITY);
        cam.zoom_at(v(0.0, 0.0), f32::NEG_INFINITY);
        cam.fit(
            Aabb::from_corners(v(f32::NAN, 0.0), v(1.0, 1.0)),
            viewport,
            0.0,
        );
        cam.fit(bounds, v(f32::INFINITY, 600.0), 0.0);
        cam.fit(bounds, viewport, f32::NAN);

        assert_camera_unchanged(cam, start);
    }

    #[test]
    fn overflowing_pan_is_ignored() {
        let start = Camera::new(v(f32::MAX, 0.0), ZOOM_MIN);
        let mut cam = start;

        cam.pan_by_screen(v(-f32::MAX, 0.0));
        cam.zoom_at(v(f32::MAX, 0.0), -1.0);

        assert_camera_unchanged(cam, start);
    }

    // ---- AC-6: visible rect and length conversion ----

    #[test]
    fn visible_rect_default_matches_viewport() {
        let rect = Camera::default().visible_world_rect(v(800.0, 600.0));

        assert_vec_eq(rect.min, Vec2::ZERO);
        assert_vec_eq(rect.max, v(800.0, 600.0));
    }

    #[test]
    fn visible_rect_follows_offset_and_zoom() {
        let cam = Camera::new(v(-50.0, 100.0), 2.0);

        let rect = cam.visible_world_rect(v(800.0, 600.0));

        assert_vec_eq(rect.min, v(-50.0, 100.0));
        assert_vec_eq(rect.max, v(350.0, 400.0));
    }

    #[test]
    fn visible_rect_non_finite_viewport_is_degenerate() {
        let cam = sample_camera();

        let rect = cam.visible_world_rect(v(f32::NAN, f32::INFINITY));

        assert_vec_eq(rect.min, cam.offset());
        assert_vec_eq(rect.max, cam.offset());
    }

    #[test]
    fn len_conversion_scales_with_zoom() {
        let cam = Camera::new(Vec2::ZERO, 4.0);

        assert_f32_eq(cam.world_len(8.0), 2.0);
        assert_f32_eq(cam.screen_len(2.0), 8.0);
        assert_f32_eq(Camera::default().world_len(5.0), 5.0);
    }

    #[test]
    fn len_conversion_round_trip() {
        let cam = sample_camera();

        assert_f32_eq(cam.screen_len(cam.world_len(6.0)), 6.0);
        assert_f32_eq(cam.world_len(cam.screen_len(6.0)), 6.0);
    }

    // ---- AC-7: fit and reset ----

    #[test]
    fn fit_centres_bounds() {
        let mut cam = sample_camera();
        let bounds = Aabb::from_corners(v(100.0, 100.0), v(300.0, 200.0));
        let viewport = v(800.0, 600.0);

        cam.fit(bounds, viewport, 20.0);

        assert_vec_eq(cam.world_to_screen(bounds.center()), viewport * 0.5);
    }

    #[test]
    fn fit_zooms_to_show_bounds_with_margin() {
        let mut cam = Camera::default();
        // 200 x 100 world units into 800 x 600 px with 50 px margins:
        // usable 700 x 500 → zoom = min(700 / 200, 500 / 100) = 3.5.
        let bounds = Aabb::from_corners(v(100.0, 100.0), v(300.0, 200.0));
        let viewport = v(800.0, 600.0);

        cam.fit(bounds, viewport, 50.0);

        assert_f32_eq(cam.zoom(), 3.5);
        let min = cam.world_to_screen(bounds.min);
        let max = cam.world_to_screen(bounds.max);
        assert_f32_eq(min.x, 50.0);
        assert_f32_eq(max.x, 750.0);
        assert!(min.y >= 50.0 - EPS && max.y <= 550.0 + EPS);
    }

    #[test]
    fn fit_clamps_zoom() {
        let viewport = v(800.0, 600.0);
        let tiny = Aabb::from_corners(v(0.0, 0.0), v(0.001, 0.001));
        let huge = Aabb::from_corners(v(-1.0e6, -1.0e6), v(1.0e6, 1.0e6));
        let mut zoomed_in = Camera::default();
        let mut zoomed_out = Camera::default();

        zoomed_in.fit(tiny, viewport, 0.0);
        zoomed_out.fit(huge, viewport, 0.0);

        assert_f32_eq(zoomed_in.zoom(), ZOOM_MAX);
        assert_f32_eq(zoomed_out.zoom(), ZOOM_MIN);
        assert_vec_eq(zoomed_in.world_to_screen(tiny.center()), viewport * 0.5);
        assert_vec_eq(zoomed_out.world_to_screen(huge.center()), viewport * 0.5);
    }

    #[test]
    fn fit_degenerate_bounds_keeps_zoom() {
        let viewport = v(800.0, 600.0);
        let point = Aabb::from_corners(v(5.0, 5.0), v(5.0, 5.0));
        let horizontal_line = Aabb::from_corners(v(0.0, 7.0), v(100.0, 7.0));
        let mut on_point = sample_camera();
        let mut on_line = sample_camera();

        on_point.fit(point, viewport, 10.0);
        on_line.fit(horizontal_line, viewport, 0.0);

        assert_f32_eq(on_point.zoom(), 2.5);
        assert_vec_eq(on_point.world_to_screen(v(5.0, 5.0)), viewport * 0.5);
        // Only the x axis constrains zoom: 800 / 100 = 8.
        assert_f32_eq(on_line.zoom(), 8.0);
        assert_vec_eq(
            on_line.world_to_screen(horizontal_line.center()),
            viewport * 0.5,
        );
    }

    #[test]
    fn fit_invalid_viewport_is_ignored() {
        let start = sample_camera();
        let bounds = Aabb::from_corners(v(0.0, 0.0), v(10.0, 10.0));
        let mut cam = start;

        cam.fit(bounds, v(0.0, 600.0), 0.0);
        cam.fit(bounds, v(800.0, -1.0), 0.0);

        assert_camera_unchanged(cam, start);
    }

    #[test]
    fn fit_margin_larger_than_viewport_still_fits_safely() {
        let mut cam = Camera::default();
        let bounds = Aabb::from_corners(v(0.0, 0.0), v(10.0, 10.0));

        cam.fit(bounds, v(100.0, 100.0), 1000.0);

        assert!(cam.offset().is_finite());
        assert!((ZOOM_MIN..=ZOOM_MAX).contains(&cam.zoom()));
    }

    #[test]
    fn reset_returns_default() {
        let mut cam = sample_camera();
        cam.zoom_at(v(10.0, 10.0), 4.0);

        cam.reset();

        assert_camera_unchanged(cam, Camera::default());
    }

    // ---- properties ----

    fn coord() -> impl Strategy<Value = f32> {
        -1.0e4_f32..1.0e4_f32
    }

    fn point() -> impl Strategy<Value = Vec2> {
        (coord(), coord()).prop_map(|(x, y)| Vec2::new(x, y))
    }

    fn camera() -> impl Strategy<Value = Camera> {
        (point(), ZOOM_MIN..=ZOOM_MAX).prop_map(|(offset, zoom)| Camera::new(offset, zoom))
    }

    /// Absolute tolerance for a value built from operands up to `magnitude`:
    /// a few f32 ulps of the largest intermediate.
    fn tolerance(magnitude: f32) -> f32 {
        16.0 * f32::EPSILON * magnitude.max(1.0)
    }

    proptest! {
        #[test]
        fn round_trip_within_tolerance(cam in camera(), p in point()) {
            let o = cam.offset();
            let z = cam.zoom();
            let p_mag = p.x.abs().max(p.y.abs());
            let o_mag = o.x.abs().max(o.y.abs());
            // Rounding error of each direction is a few ulps of its largest term.
            let world_scale = p_mag + o_mag;
            let screen_scale = p_mag + o_mag * z;

            let world_back = cam.screen_to_world(cam.world_to_screen(p));
            let screen_back = cam.world_to_screen(cam.screen_to_world(p));

            prop_assert!(
                world_back.approx_eq(p, tolerance(world_scale)),
                "world {p:?} -> {world_back:?}"
            );
            prop_assert!(
                screen_back.approx_eq(p, tolerance(screen_scale)),
                "screen {p:?} -> {screen_back:?}"
            );
        }

        #[test]
        fn zoom_keeps_anchor_fixed_for_any_input(
            cam in camera(), anchor in point(), notches in -40.0_f32..40.0
        ) {
            let mut cam = cam;
            let before = cam.screen_to_world(anchor);
            let scale = before.x.abs().max(before.y.abs()) + anchor.x.abs().max(anchor.y.abs()) / ZOOM_MIN;

            cam.zoom_at(anchor, notches);

            let after = cam.screen_to_world(anchor);
            prop_assert!(after.approx_eq(before, tolerance(scale)), "{before:?} -> {after:?}");
        }

        #[test]
        fn zoom_stays_in_range_for_any_sequence(
            cam in camera(),
            steps in prop::collection::vec((point(), -50.0_f32..50.0, point()), 0..32)
        ) {
            let mut cam = cam;

            for (anchor, notches, delta) in steps {
                cam.zoom_at(anchor, notches);
                cam.pan_by_screen(delta);

                prop_assert!((ZOOM_MIN..=ZOOM_MAX).contains(&cam.zoom()));
                prop_assert!(cam.offset().is_finite());
            }
        }
    }
}
