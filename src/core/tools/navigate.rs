//! Pan and zoom gestures.
//!
//! Owned by T08; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::Vec2;

    #[test]
    fn pan_moves_by_pointer_delta() {
        // Arrange
        let mut camera = Camera::default();
        let mut pan = Pan::new(Vec2::new(10.0, 10.0));

        // Act
        let changed = pan.drag(&mut camera, Vec2::new(30.0, 40.0));

        // Assert
        assert!(changed);
        assert!(
            camera
                .world_to_screen(Vec2::ZERO)
                .approx_eq(Vec2::new(20.0, 30.0), 1e-4)
        );
    }

    #[test]
    fn pan_without_movement_reports_no_change() {
        let mut camera = Camera::default();
        let mut pan = Pan::new(Vec2::new(5.0, 5.0));

        assert!(!pan.drag(&mut camera, Vec2::new(5.0, 5.0)));
        assert_eq!(camera, Camera::default());
    }

    #[test]
    fn zoom_with_non_finite_input_reports_no_change() {
        let mut camera = Camera::default();

        assert!(!zoom(&mut camera, Vec2::new(f32::NAN, 0.0), 1.0));
        assert!(!zoom(&mut camera, Vec2::ZERO, f32::INFINITY));
        assert_eq!(camera, Camera::default());
    }

    #[test]
    fn zoom_at_limit_reports_no_change() {
        let mut camera = Camera::new(Vec2::ZERO, crate::core::camera::ZOOM_MAX);

        assert!(!zoom(&mut camera, Vec2::new(10.0, 10.0), 1.0));
    }
}
