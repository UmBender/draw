//! [`Zoom`]: pixels per world unit, finite and within [`Zoom::MIN`]`..=`[`Zoom::MAX`]
//! (ADR-0013), so a camera cannot hold a NaN or absurd scale.

/// Why a number is not a [`Zoom`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomError {
    /// NaN or infinite.
    NotFinite,
    /// Finite but outside the allowed range.
    OutOfRange,
}

/// Pixels per world unit, always finite and in range.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Zoom(f32);

impl Zoom {
    /// Smallest allowed zoom.
    pub const MIN: f32 = 0.05;
    /// Largest allowed zoom.
    pub const MAX: f32 = 20.0;
    /// One pixel per world unit.
    pub const ONE: Self = Self(1.0);

    /// Checks `value`: parse, don't validate.
    ///
    /// # Errors
    /// [`ZoomError::NotFinite`] for NaN or infinity, [`ZoomError::OutOfRange`]
    /// outside [`Zoom::MIN`]`..=`[`Zoom::MAX`].
    pub fn new(value: f32) -> Result<Self, ZoomError> {
        if !value.is_finite() {
            Err(ZoomError::NotFinite)
        } else if !(Self::MIN..=Self::MAX).contains(&value) {
            Err(ZoomError::OutOfRange)
        } else {
            Ok(Self(value))
        }
    }

    /// Clamps into range; a non-finite value becomes `1.0`.
    #[must_use]
    pub fn clamped(value: f32) -> Self {
        if value.is_finite() {
            Self(value.clamp(Self::MIN, Self::MAX))
        } else {
            Self::ONE
        }
    }

    /// The factor, within [`Zoom::MIN`]`..=`[`Zoom::MAX`].
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::approx_eq;

    #[test]
    fn zoom_new_rejects_nan() {
        assert_eq!(Zoom::new(f32::NAN), Err(ZoomError::NotFinite));
        assert_eq!(Zoom::new(f32::INFINITY), Err(ZoomError::NotFinite));
    }

    #[test]
    fn zoom_new_rejects_out_of_range() {
        assert_eq!(Zoom::new(0.0), Err(ZoomError::OutOfRange));
        assert_eq!(Zoom::new(Zoom::MIN * 0.5), Err(ZoomError::OutOfRange));
        assert_eq!(Zoom::new(Zoom::MAX * 2.0), Err(ZoomError::OutOfRange));
        assert_eq!(Zoom::new(-1.0), Err(ZoomError::OutOfRange));
    }

    #[test]
    fn zoom_new_accepts_bounds() {
        assert!(Zoom::new(Zoom::MIN).is_ok());
        assert!(Zoom::new(Zoom::MAX).is_ok());
        assert!(approx_eq(Zoom::ONE.get(), 1.0, 1e-6));
    }

    #[test]
    fn zoom_clamped_maps_non_finite_to_one_and_clamps_the_rest() {
        assert!(approx_eq(Zoom::clamped(f32::NAN).get(), 1.0, 1e-6));
        assert!(approx_eq(Zoom::clamped(1e9).get(), Zoom::MAX, 1e-6));
        assert!(approx_eq(Zoom::clamped(-5.0).get(), Zoom::MIN, 1e-6));
        assert!(approx_eq(Zoom::clamped(2.0).get(), 2.0, 1e-6));
    }
}
