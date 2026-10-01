//! Anti-tremor stroke smoothing: resampling, EMA, Douglas-Peucker.
//!
//! Owned by T04; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{approx_eq, distance_to_segment, Vec2};
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

    fn assert_points_eq(actual: &[Vec2], expected: &[Vec2]) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "length differs: {actual:?} vs {expected:?}"
        );
        for (a, e) in actual.iter().zip(expected) {
            assert_vec_eq(*a, *e);
        }
    }

    fn assert_params(p: SmoothingParams, min_dist_px: f32, alpha: f32, epsilon_px: f32) {
        assert!(approx_eq(p.min_dist_px, min_dist_px, EPS), "{p:?}");
        assert!(approx_eq(p.alpha, alpha, EPS), "{p:?}");
        assert!(approx_eq(p.epsilon_px, epsilon_px, EPS), "{p:?}");
    }

    fn medium() -> SmoothingParams {
        SmoothingLevel::Medium.params()
    }

    /// Distance from `p` to the nearest segment of `polyline`.
    fn distance_to_polyline(p: Vec2, polyline: &[Vec2]) -> f32 {
        match polyline {
            [] => f32::INFINITY,
            [only] => p.distance(*only),
            _ => polyline
                .windows(2)
                .map(|w| distance_to_segment(p, w[0], w[1]))
                .fold(f32::INFINITY, f32::min),
        }
    }

    /// `true` if `sub` appears in `full` in order (exact copies of input points).
    fn is_subsequence(sub: &[Vec2], full: &[Vec2]) -> bool {
        let mut rest = full.iter();
        sub.iter()
            .all(|s| rest.any(|f| f.approx_eq(*s, 0.0)))
    }

    /// Population variance of the y components.
    fn variance_y(points: &[Vec2]) -> f32 {
        let n = points.len() as f32;
        let mean = points.iter().map(|p| p.y).sum::<f32>() / n;
        points.iter().map(|p| (p.y - mean).powi(2)).sum::<f32>() / n
    }

    // ---- AC-1: levels ----

    #[test]
    fn level_default_is_medium() {
        assert_eq!(SmoothingLevel::default(), SmoothingLevel::Medium);
    }

    #[test]
    fn level_next_cycles_through_all() {
        let start = SmoothingLevel::Off;

        let one = start.next();
        let two = one.next();
        let three = two.next();
        let four = three.next();

        assert_eq!(one, SmoothingLevel::Low);
        assert_eq!(two, SmoothingLevel::Medium);
        assert_eq!(three, SmoothingLevel::High);
        assert_eq!(four, SmoothingLevel::Off);
    }

    #[test]
    fn level_params_match_adr_table() {
        assert_params(SmoothingLevel::Off.params(), 0.0, 1.0, 0.0);
        assert_params(SmoothingLevel::Low.params(), 1.5, 0.6, 0.8);
        assert_params(SmoothingLevel::Medium.params(), 2.5, 0.4, 1.5);
        assert_params(SmoothingLevel::High.params(), 4.0, 0.25, 2.5);
    }

    #[test]
    fn level_labels_are_distinct() {
        let labels = [
            SmoothingLevel::Off.label(),
            SmoothingLevel::Low.label(),
            SmoothingLevel::Medium.label(),
            SmoothingLevel::High.label(),
        ];

        assert_eq!(labels, ["off", "low", "medium", "high"]);
    }

    // ---- AC-2: push (resample + EMA) ----

    #[test]
    fn push_first_point_is_kept_unfiltered() {
        let mut s = Smoother::new(medium(), 1.0);

        let kept = s.push(v(3.0, -4.0));

        assert!(kept);
        assert_points_eq(s.points(), &[v(3.0, -4.0)]);
    }

    #[test]
    fn push_drops_close_points() {
        // Medium: min_dist 2.5 px, px_to_world 1 → 2.5 world units.
        let mut s = Smoother::new(medium(), 1.0);

        let a = s.push(v(0.0, 0.0));
        let b = s.push(v(1.0, 0.0));
        let c = s.push(v(2.0, 0.0));
        let d = s.push(v(3.0, 0.0));
        let e = s.push(v(4.0, 0.0));

        assert!(a);
        assert!(!b, "1.0 < 2.5 from last kept");
        assert!(!c, "2.0 < 2.5 from last kept");
        assert!(d, "3.0 >= 2.5 from last kept raw (0,0)");
        assert!(!e, "1.0 from last kept raw (3,0)");
        assert_eq!(s.points().len(), 2);
    }

    #[test]
    fn push_min_dist_scales_with_px_to_world() {
        // Zoomed out 2x: 1 px = 2 world units, min_dist = 5 world units.
        let mut s = Smoother::new(medium(), 2.0);

        s.push(v(0.0, 0.0));
        let near = s.push(v(4.0, 0.0));
        let far = s.push(v(5.0, 0.0));

        assert!(!near);
        assert!(far);
    }

    #[test]
    fn push_ignores_non_finite() {
        let mut s = Smoother::new(medium(), 1.0);

        let nan_first = s.push(v(f32::NAN, 0.0));
        s.push(v(0.0, 0.0));
        let inf = s.push(v(f32::INFINITY, 10.0));
        let neg_inf = s.push(v(10.0, f32::NEG_INFINITY));
        let nan = s.push(v(f32::NAN, f32::NAN));

        assert!(!nan_first && !inf && !neg_inf && !nan);
        assert_points_eq(s.points(), &[v(0.0, 0.0)]);
        assert_points_eq(&s.finish(), &[v(0.0, 0.0)]);
    }

    #[test]
    fn push_drops_exact_duplicates() {
        let mut s = Smoother::new(SmoothingLevel::Off.params(), 1.0);

        let first = s.push(v(1.0, 1.0));
        let dup = s.push(v(1.0, 1.0));

        assert!(first);
        assert!(!dup);
        assert_eq!(s.points().len(), 1);
    }

    #[test]
    fn ema_reduces_jitter() {
        // Zig-zag tremor around the x axis: y alternates ±2.
        let raw: Vec<Vec2> = (0..40)
            .map(|i| v(i as f32 * 5.0, if i % 2 == 0 { 2.0 } else { -2.0 }))
            .collect();
        let mut s = Smoother::new(medium(), 1.0);

        for p in &raw {
            s.push(*p);
        }

        let smoothed = s.points();
        assert_eq!(smoothed.len(), raw.len(), "points are 5 apart, none dropped");
        // Skip the warm-up: the first point is unfiltered by design.
        let raw_var = variance_y(&raw[10..]);
        let smooth_var = variance_y(&smoothed[10..]);
        assert!(
            smooth_var < raw_var * 0.5,
            "raw variance {raw_var}, smoothed {smooth_var}"
        );
    }

    #[test]
    fn ema_moves_alpha_of_the_way() {
        let params = SmoothingParams {
            min_dist_px: 0.0,
            alpha: 0.25,
            epsilon_px: 0.0,
        };
        let mut s = Smoother::new(params, 1.0);

        s.push(v(0.0, 0.0));
        s.push(v(8.0, 4.0));

        assert_points_eq(s.points(), &[v(0.0, 0.0), v(2.0, 1.0)]);
    }

    #[test]
    fn new_sanitizes_bad_parameters() {
        let bad = SmoothingParams {
            min_dist_px: f32::NAN,
            alpha: f32::INFINITY,
            epsilon_px: -3.0,
        };
        let zero_alpha = SmoothingParams {
            min_dist_px: -1.0,
            alpha: 0.0,
            epsilon_px: f32::NAN,
        };
        let mut a = Smoother::new(bad, f32::NAN);
        let mut b = Smoother::new(zero_alpha, -2.0);
        let mut c = Smoother::new(medium(), 0.0);

        for i in 0..10 {
            let p = v(i as f32, (i % 2) as f32);
            a.push(p);
            b.push(p);
            c.push(p);
        }

        // alpha=∞ → 1.0 (passthrough), min_dist NaN → 0, eps < 0 → 0.
        assert_eq!(a.points().len(), 10);
        assert_vec_eq(a.points()[9], v(9.0, 1.0));
        // alpha=0 would freeze the stroke; it must still move forward.
        assert!(b.points().iter().all(|p| p.is_finite()));
        assert!(b.points()[9].x > 0.0);
        // px_to_world 0 → 1.0, so medium min_dist is 2.5 world units.
        assert!(c.points().len() < 10);
        for out in [a.finish(), b.finish(), c.finish()] {
            assert!(!out.is_empty());
            assert!(out.iter().all(|p| p.is_finite()));
        }
    }

    // ---- AC-3: live polyline ----

    #[test]
    fn points_exposes_live_smoothed_polyline() {
        let mut s = Smoother::new(medium(), 1.0);

        s.push(v(0.0, 0.0));
        s.push(v(10.0, 0.0));
        s.push(v(11.0, 0.0)); // dropped by resampling

        // 0 + 0.4 * (10 - 0) = 4; the dropped raw point is not in the preview.
        assert_points_eq(s.points(), &[v(0.0, 0.0), v(4.0, 0.0)]);
    }

    // ---- AC-4: finish ----

    #[test]
    fn finish_ends_at_last_raw_point() {
        let mut s = Smoother::new(medium(), 1.0);
        for i in 0..10 {
            s.push(v(i as f32 * 3.0, if i % 2 == 0 { 1.0 } else { -1.0 }));
        }
        s.push(v(27.5, 0.3)); // too close to (27, -1): dropped, but is the true end

        let out = s.finish();

        let Some(last) = out.last() else {
            panic!("expected a non-empty stroke");
        };
        assert_vec_eq(*last, v(27.5, 0.3));
    }

    #[test]
    fn finish_starts_at_first_raw_point() {
        let mut s = Smoother::new(SmoothingLevel::High.params(), 1.0);
        for i in 0..20 {
            s.push(v(i as f32 * 5.0, (i * i) as f32));
        }

        let out = s.finish();

        assert_vec_eq(out[0], v(0.0, 0.0));
    }

    #[test]
    fn finish_empty_is_empty() {
        let s = Smoother::new(medium(), 1.0);

        assert!(s.finish().is_empty());
    }

    #[test]
    fn finish_single_point_is_dot() {
        let mut s = Smoother::new(medium(), 1.0);
        s.push(v(5.0, 5.0));
        s.push(v(5.5, 5.0)); // dropped by resampling, still the last raw point

        let out = s.finish();

        assert_points_eq(&out, &[v(5.0, 5.0), v(5.5, 5.0)]);

        let mut dot = Smoother::new(medium(), 1.0);
        dot.push(v(5.0, 5.0));
        assert_points_eq(&dot.finish(), &[v(5.0, 5.0)]);
    }

    #[test]
    fn finish_simplifies_straight_stroke_to_two_points() {
        let mut s = Smoother::new(medium(), 1.0);
        for i in 0..=50 {
            s.push(v(i as f32 * 4.0, i as f32 * 2.0));
        }

        let out = s.finish();

        assert_points_eq(&out, &[v(0.0, 0.0), v(200.0, 100.0)]);
    }

    // ---- AC-5: Ramer–Douglas–Peucker ----

    #[test]
    fn rdp_straight_line_returns_two_points() {
        let line: Vec<Vec2> = (0..=10).map(|i| v(i as f32, 2.0 * i as f32)).collect();

        let out = simplify_rdp(&line, 0.01);

        assert_points_eq(&out, &[v(0.0, 0.0), v(10.0, 20.0)]);
    }

    #[test]
    fn rdp_keeps_corner_beyond_eps() {
        let l_shape = [v(0.0, 0.0), v(5.0, 0.0), v(10.0, 0.0), v(10.0, 5.0), v(10.0, 10.0)];

        let out = simplify_rdp(&l_shape, 0.5);

        assert_points_eq(&out, &[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)]);
    }

    #[test]
    fn rdp_drops_bump_within_eps() {
        let bumpy = [v(0.0, 0.0), v(2.0, 0.4), v(4.0, -0.3), v(6.0, 0.0)];

        let out = simplify_rdp(&bumpy, 0.5);

        assert_points_eq(&out, &[v(0.0, 0.0), v(6.0, 0.0)]);
    }

    #[test]
    fn rdp_short_input_is_unchanged() {
        let two = [v(1.0, 1.0), v(1.0, 1.0)];

        assert!(simplify_rdp(&[], 1.0).is_empty());
        assert_points_eq(&simplify_rdp(&[v(1.0, 2.0)], 1.0), &[v(1.0, 2.0)]);
        assert_points_eq(&simplify_rdp(&two, 1.0), &two);
    }

    #[test]
    fn rdp_non_positive_eps_is_identity() {
        let collinear = [v(0.0, 0.0), v(1.0, 0.0), v(2.0, 0.0), v(3.0, 0.0)];

        assert_points_eq(&simplify_rdp(&collinear, 0.0), &collinear);
        assert_points_eq(&simplify_rdp(&collinear, -1.0), &collinear);
        assert_points_eq(&simplify_rdp(&collinear, f32::NAN), &collinear);
    }

    #[test]
    fn rdp_closed_loop_keeps_shape() {
        // First and last coincide: distances are measured to that point.
        let square = [
            v(0.0, 0.0),
            v(10.0, 0.0),
            v(10.0, 10.0),
            v(0.0, 10.0),
            v(0.0, 0.0),
        ];

        let out = simplify_rdp(&square, 1.0);

        assert_points_eq(&out, &square);
    }

    #[test]
    fn rdp_huge_input_does_not_overflow_stack() {
        // A sawtooth makes every split land next to an endpoint, so a recursive
        // version would nest ~N calls deep. Run it on a deliberately small stack.
        let saw: Vec<Vec2> = (0..10_000)
            .map(|i| v(i as f32, if i % 2 == 0 { 0.0 } else { 3.0 }))
            .collect();

        let worker = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || simplify_rdp(&saw, 1.0).len());
        let result = match worker {
            Ok(handle) => handle.join(),
            Err(e) => panic!("could not spawn worker thread: {e}"),
        };

        match result {
            Ok(len) => assert_eq!(len, 10_000, "every tooth exceeds eps"),
            Err(_) => panic!("simplify_rdp panicked or overflowed its stack"),
        }
    }

    #[test]
    fn rdp_handles_non_finite_without_panic() {
        let messy = [v(0.0, 0.0), v(f32::NAN, 1.0), v(f32::INFINITY, 0.0), v(4.0, 0.0)];

        let out = simplify_rdp(&messy, 1.0);

        assert!(out.len() <= messy.len());
        assert!(out.len() >= 2);
    }

    // ---- AC-6: Off level ----

    #[test]
    fn off_level_is_passthrough() {
        let input = [
            v(0.0, 0.0),
            v(0.1, 0.0),
            v(0.1, 0.0), // exact duplicate: dropped
            v(0.2, 0.05),
            v(0.3, 0.1),
            v(0.4, 0.1),
            v(5.0, -3.0),
            v(5.0, -3.0), // exact duplicate at the end: dropped
        ];
        let expected = [
            v(0.0, 0.0),
            v(0.1, 0.0),
            v(0.2, 0.05),
            v(0.3, 0.1),
            v(0.4, 0.1),
            v(5.0, -3.0),
        ];
        let mut s = Smoother::new(SmoothingLevel::Off.params(), 1.0);

        for p in input {
            s.push(p);
        }

        assert_points_eq(s.points(), &expected);
        let out = s.finish();
        assert_eq!(out.len(), expected.len());
        for (a, e) in out.iter().zip(&expected) {
            assert!(a.approx_eq(*e, 0.0), "Off must be exact: {a:?} vs {e:?}");
        }
    }

    // ---- properties ----

    fn coord() -> impl Strategy<Value = f32> {
        -1.0e3_f32..1.0e3_f32
    }

    fn point() -> impl Strategy<Value = Vec2> {
        (coord(), coord()).prop_map(|(x, y)| Vec2::new(x, y))
    }

    fn polyline() -> impl Strategy<Value = Vec<Vec2>> {
        prop::collection::vec(point(), 0..64)
    }

    fn any_f32() -> impl Strategy<Value = f32> {
        prop_oneof![
            8 => coord(),
            1 => Just(f32::NAN),
            1 => Just(f32::INFINITY),
            1 => Just(f32::NEG_INFINITY),
        ]
    }

    fn level() -> impl Strategy<Value = SmoothingLevel> {
        prop_oneof![
            Just(SmoothingLevel::Off),
            Just(SmoothingLevel::Low),
            Just(SmoothingLevel::Medium),
            Just(SmoothingLevel::High),
        ]
    }

    proptest! {
        #[test]
        fn rdp_keeps_endpoints(points in polyline(), eps in 0.0_f32..50.0) {
            let out = simplify_rdp(&points, eps);

            match (points.first(), points.last(), out.first(), out.last()) {
                (Some(pf), Some(pl), Some(of), Some(ol)) => {
                    prop_assert!(of.approx_eq(*pf, 0.0));
                    prop_assert!(ol.approx_eq(*pl, 0.0));
                }
                (None, None, None, None) => {}
                _ => prop_assert!(false, "empty in/out mismatch: {points:?} → {out:?}"),
            }
        }

        #[test]
        fn rdp_error_bounded(points in polyline(), eps in 0.01_f32..50.0) {
            let out = simplify_rdp(&points, eps);

            for p in &points {
                let d = distance_to_polyline(*p, &out);
                // f32 rounding on coordinates up to 1e3 stays well below 1e-2.
                prop_assert!(d <= eps + 1e-2, "point {p:?} is {d} from result, eps {eps}");
            }
        }

        #[test]
        fn rdp_idempotent(points in polyline(), eps in 0.0_f32..50.0) {
            let once = simplify_rdp(&points, eps);
            let twice = simplify_rdp(&once, eps);

            prop_assert_eq!(once.len(), twice.len());
            for (a, b) in once.iter().zip(&twice) {
                prop_assert!(a.approx_eq(*b, 0.0));
            }
        }

        #[test]
        fn rdp_is_subsequence_and_never_grows(points in polyline(), eps in any_f32()) {
            let out = simplify_rdp(&points, eps);

            prop_assert!(out.len() <= points.len());
            prop_assert!(is_subsequence(&out, &points));
        }

        #[test]
        fn smoother_output_is_finite(
            lvl in level(),
            px_to_world in any_f32(),
            raw in prop::collection::vec((any_f32(), any_f32()), 0..64),
        ) {
            let mut s = Smoother::new(lvl.params(), px_to_world);

            for (x, y) in raw {
                s.push(Vec2::new(x, y));
                prop_assert!(s.points().iter().all(|p| p.is_finite()));
            }
            let out = s.finish();

            prop_assert!(out.iter().all(|p| p.is_finite()));
        }
    }
}
