//! Pan and zoom gestures.
//!
//! Panning follows the pointer exactly: content moves by the pointer delta in
//! pixels. Zooming is anchored at the cursor (see
//! [`Camera::zoom_at`]). Both report whether the camera actually changed so
//! the editor can skip redraws.

use crate::core::camera::Camera;
use crate::core::geom::Vec2;

/// An in-progress pan drag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pan {
    /// Pointer position (screen) at the previous drag step.
    last: Vec2,
}

impl Pan {
    /// Starts a pan with the pointer at `pos` (screen, finite).
    #[must_use]
    pub fn new(pos: Vec2) -> Self {
        Self { last: pos }
    }

    /// Moves the pointer to `pos` (screen, finite), panning by the delta.
    /// Returns whether the camera changed.
    pub fn drag(&mut self, camera: &mut Camera, pos: Vec2) -> bool {
        let before = *camera;
        camera.pan_by_screen(pos - self.last);
        self.last = pos;
        *camera != before
    }
}

/// Zooms by `notches` wheel steps anchored at `pos` (screen). Returns whether
/// the camera changed; non-finite input changes nothing.
pub fn zoom(camera: &mut Camera, pos: Vec2, notches: f32) -> bool {
    let before = *camera;
    camera.zoom_at(pos, notches);
    *camera != before
}

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
