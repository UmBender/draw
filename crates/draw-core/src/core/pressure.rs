//! Pen pressure to stroke width (T35, ADR-T35-2).
//!
//! A pen stroke stores one width *factor* per point, multiplying the
//! stroke's chosen width. A half press draws the chosen width; a light
//! touch goes down to [`MIN_WIDTH_FACTOR`] of it and a hard press up to
//! [`MAX_WIDTH_FACTOR`].

/// Width factor of the lightest press.
pub const MIN_WIDTH_FACTOR: f32 = 0.25;

/// Width factor of the hardest press.
pub const MAX_WIDTH_FACTOR: f32 = 2.0;

/// The width factor for pen pressure `p` (`0` none, `1` full):
/// `2p` clamped to [`MIN_WIDTH_FACTOR`]`..=`[`MAX_WIDTH_FACTOR`]. A
/// non-finite pressure gives `1`.
#[must_use]
pub fn width_factor(p: f32) -> f32 {
    if p.is_finite() {
        (2.0 * p).clamp(MIN_WIDTH_FACTOR, MAX_WIDTH_FACTOR)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;
    use proptest::prelude::*;

    const EPS: f32 = 1e-6;

    #[test]
    fn width_factor_half_press_is_one() {
        // Arrange / Act
        let factor = width_factor(0.5);

        // Assert
        assert!(approx_eq(factor, 1.0, EPS));
    }

    #[test]
    fn width_factor_clamps_light_and_hard_presses() {
        assert!(approx_eq(width_factor(0.0), MIN_WIDTH_FACTOR, EPS));
        assert!(approx_eq(width_factor(0.05), MIN_WIDTH_FACTOR, EPS));
        assert!(approx_eq(width_factor(0.25), 0.5, EPS));
        assert!(approx_eq(width_factor(0.75), 1.5, EPS));
        assert!(approx_eq(width_factor(1.0), MAX_WIDTH_FACTOR, EPS));
        assert!(approx_eq(width_factor(7.0), MAX_WIDTH_FACTOR, EPS));
        assert!(approx_eq(width_factor(-1.0), MIN_WIDTH_FACTOR, EPS));
    }

    #[test]
    fn width_factor_non_finite_is_one() {
        for p in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(approx_eq(width_factor(p), 1.0, EPS), "{p}");
        }
    }

    proptest! {
        #[test]
        fn width_factor_is_monotonic_and_bounded(a in any::<f32>(), b in any::<f32>()) {
            let (fa, fb) = (width_factor(a), width_factor(b));
            prop_assert!((MIN_WIDTH_FACTOR..=MAX_WIDTH_FACTOR).contains(&fa));
            prop_assert!((MIN_WIDTH_FACTOR..=MAX_WIDTH_FACTOR).contains(&fb));
            if a.is_finite() && b.is_finite() && a <= b {
                prop_assert!(fa <= fb);
            }
        }
    }
}
