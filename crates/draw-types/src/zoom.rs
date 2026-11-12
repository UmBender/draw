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
