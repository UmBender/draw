//! Geometry primitives: [`Vec2`], [`Aabb`], [`distance_to_segment`] and the
//! float helper [`approx_eq`].
//!
//! Everything here is plain `f32` math with no dependencies. Floats are never
//! compared with `==` (clippy `float_cmp` is denied); use [`approx_eq`] or
//! [`Vec2::approx_eq`]. Non-finite input never panics: it is either rejected
//! explicitly ([`Vec2::sanitize`], [`Aabb::from_points`]) or propagates as a
//! non-finite result the caller can check with [`Vec2::is_finite`].

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// Returns `true` when `|a − b| <= eps`.
///
/// Any NaN or infinite operand (including `eps`) yields `false`, so two
/// infinities are never "approximately equal".
#[must_use]
pub fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
    eps.is_finite() && (a - b).abs() <= eps
}

/// A 2D vector or point in world or screen space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl Vec2 {
    /// The origin `(0, 0)`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Creates a vector from its components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Euclidean length `√(x² + y²)`.
    #[must_use]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    /// Squared length `x² + y²`; cheaper than [`Vec2::length`] for comparisons.
    #[must_use]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    /// Dot product `self · other`.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Euclidean distance between two points.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        (other - self).length()
    }

    /// Linear interpolation: `t = 0` gives `self`, `t = 1` gives `other`.
    /// `t` is not clamped, so values outside `[0, 1]` extrapolate.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// Component-wise [`approx_eq`] with tolerance `eps`.
    #[must_use]
    pub fn approx_eq(self, other: Self, eps: f32) -> bool {
        approx_eq(self.x, other.x, eps) && approx_eq(self.y, other.y, eps)
    }

    /// `true` when both components are finite (not NaN, not ±∞).
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Returns `Some(self)` if both components are finite, otherwise `None`.
    #[must_use]
    pub fn sanitize(self) -> Option<Self> {
        self.is_finite().then_some(self)
    }

    /// Component-wise minimum.
    fn component_min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    /// Component-wise maximum.
    fn component_max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

/// Axis-aligned bounding box. Invariant: `min.x <= max.x` and `min.y <= max.y`
/// when built through the constructors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Corner with the smallest coordinates.
    pub min: Vec2,
    /// Corner with the largest coordinates.
    pub max: Vec2,
}

impl Aabb {
    /// Tightest box around the finite points in `points`.
    ///
    /// Non-finite points are skipped; returns `None` when no finite point
    /// remains (including an empty slice).
    #[must_use]
    pub fn from_points(points: &[Vec2]) -> Option<Self> {
        let mut finite = points.iter().copied().filter(|p| p.is_finite());
        let first = finite.next()?;
        Some(finite.fold(
            Self {
                min: first,
                max: first,
            },
            |acc, p| Self {
                min: acc.min.component_min(p),
                max: acc.max.component_max(p),
            },
        ))
    }

    /// Box spanned by two opposite corners given in any order.
    #[must_use]
    pub fn from_corners(a: Vec2, b: Vec2) -> Self {
        Self {
            min: a.component_min(b),
            max: a.component_max(b),
        }
    }

    /// Grows every side by `margin`. A negative margin shrinks the box; an axis
    /// that would invert collapses to its centre instead.
    #[must_use]
    pub fn expand(self, margin: f32) -> Self {
        let (min_x, max_x) = expand_axis(self.min.x, self.max.x, margin);
        let (min_y, max_y) = expand_axis(self.min.y, self.max.y, margin);
        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    /// `true` if `p` lies inside or on the boundary.
    #[must_use]
    pub fn contains(&self, p: Vec2) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    /// `true` if the boxes overlap or touch.
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && other.min.x <= self.max.x
            && self.min.y <= other.max.y
            && other.min.y <= self.max.y
    }

    /// Smallest box containing both boxes.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.component_min(other.min),
            max: self.max.component_max(other.max),
        }
    }

    /// Centre point.
    #[must_use]
    pub fn center(&self) -> Vec2 {
        self.min.lerp(self.max, 0.5)
    }

    /// Extent along x.
    #[must_use]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Extent along y.
    #[must_use]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// The same box moved by `delta`.
    #[must_use]
    pub fn translate(self, delta: Vec2) -> Self {
        Self {
            min: self.min + delta,
            max: self.max + delta,
        }
    }
}

/// Expands the interval `[min, max]` by `margin` on both ends, collapsing to
/// its midpoint if a negative margin would invert it.
fn expand_axis(min: f32, max: f32, margin: f32) -> (f32, f32) {
    let (lo, hi) = (min - margin, max + margin);
    if lo > hi {
        let mid = (min + max) * 0.5;
        (mid, mid)
    } else {
        (lo, hi)
    }
}

/// Euclidean distance from `p` to the closed segment `ab`.
///
/// Projects `p` onto the line through `a` and `b`, clamps the projection to the
/// segment, and measures to that point. A degenerate segment (`a` and `b`
/// coincide) is treated as the point `a`, so there is no division by zero.
#[must_use]
pub fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_sq();
    if len_sq <= 0.0 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

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
