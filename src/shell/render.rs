//! Draws shapes, previews and selection with macroquad.
//!
//! Everything is drawn in **screen space** with macroquad's default camera:
//! world points go through [`Camera::world_to_screen`] and widths through
//! [`screen_width`] (ADR-T07-1). Off-screen shapes are culled by their
//! bounding box (ADR-0006). Discs and ellipses are tessellated here with
//! `draw_triangle`, which does not allocate, instead of macroquad's
//! `draw_circle`/`draw_ellipse`, which do.

use macroquad::color::Color;
use macroquad::math::Vec2 as MqVec2;

use crate::core::camera::Camera;
use crate::core::geom::{Aabb, Vec2};
use crate::core::shape::Shape;

/// Smallest on-screen outline width in pixels (ADR-0013).
pub const MIN_SCREEN_WIDTH_PX: f32 = 1.0;

/// Strokes wider than this many pixels get round joints and caps.
pub const JOINT_THRESHOLD_PX: f32 = 2.0;

/// Extra pixels around the viewport kept when culling, covering the
/// overhang of outlines clamped to [`MIN_SCREEN_WIDTH_PX`].
pub const CULL_MARGIN_PX: f32 = 2.0;

/// Largest allowed distance in pixels between a circle and its polygon.
pub const CHORD_TOLERANCE_PX: f32 = 0.25;

/// Fewest segments used for a disc or ellipse.
pub const MIN_SEGMENTS: u16 = 8;

/// Most segments used for a disc or ellipse.
pub const MAX_SEGMENTS: u16 = 256;

/// Width of the selection outline in pixels, independent of zoom.
pub const SELECTION_WIDTH_PX: f32 = 1.0;

/// Gap in pixels between the selected shapes' bounds and the selection outline.
pub const SELECTION_PAD_PX: f32 = 4.0;

/// On-screen outline width in pixels for a world-space `world_width`:
/// `world_width * zoom`, at least [`MIN_SCREEN_WIDTH_PX`]. A NaN, infinite or
/// negative product gives [`MIN_SCREEN_WIDTH_PX`].
#[must_use]
pub fn screen_width(world_width: f32, zoom: f32) -> f32 {
    let _ = (world_width, zoom);
    todo!()
}

/// `true` if `bounds` overlaps or touches `view`; `false` if either box has a
/// NaN coordinate.
#[must_use]
pub fn is_visible(bounds: Aabb, view: Aabb) -> bool {
    let _ = (bounds, view);
    todo!()
}

/// World rectangle used for culling: the area visible through a viewport of
/// `viewport` pixels, grown by [`CULL_MARGIN_PX`] (in world units).
#[must_use]
pub fn cull_rect(camera: &Camera, viewport: Vec2) -> Aabb {
    let _ = (camera, viewport);
    todo!()
}

/// Converts a core vector to a macroquad vector.
#[must_use]
pub fn to_mq(v: Vec2) -> MqVec2 {
    let _ = v;
    todo!()
}

/// Converts a palette colour to a macroquad colour (channels in `0..=1`).
#[must_use]
pub fn to_mq_color(c: crate::core::palette::Rgba) -> Color {
    let _ = c;
    todo!()
}

/// `true` if a stroke `width_px` wide needs round joints, i.e. it is wider
/// than [`JOINT_THRESHOLD_PX`]. Thinner strokes skip them to stay cheap.
#[must_use]
pub fn stroke_needs_joints(width_px: f32) -> bool {
    let _ = width_px;
    todo!()
}

/// Number of segments for a circle of `radius_px` pixels: the fewest whose
/// chord error `r (1 − cos(π/n))` is at most [`CHORD_TOLERANCE_PX`], clamped
/// to [`MIN_SEGMENTS`]`..=`[`MAX_SEGMENTS`]. A non-finite or non-positive
/// radius gives [`MIN_SEGMENTS`].
#[must_use]
pub fn circle_segments(radius_px: f32) -> u16 {
    let _ = radius_px;
    todo!()
}

/// Screen rectangle of the selection outline: `bounds` mapped to pixels and
/// grown by [`SELECTION_PAD_PX`].
#[must_use]
pub fn selection_rect(bounds: Aabb, camera: &Camera) -> Aabb {
    let _ = (bounds, camera);
    todo!()
}

/// Draws every shape whose bounds are visible through `camera` in a viewport
/// of `viewport` pixels, in iteration order (later shapes on top).
pub fn draw_shapes<'a>(shapes: impl IntoIterator<Item = &'a Shape>, camera: &Camera, viewport: Vec2) {
    let _ = (shapes.into_iter(), camera, viewport);
    todo!()
}

/// Draws the shape being created, without culling.
pub fn draw_preview(shape: &Shape, camera: &Camera) {
    let _ = (shape, camera);
    todo!()
}

/// Draws the selection outline around world `bounds`, [`SELECTION_WIDTH_PX`]
/// wide in the theme accent colour.
pub fn draw_selection(bounds: Aabb, camera: &Camera) {
    let _ = (bounds, camera);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::geom::{Aabb, Vec2, approx_eq};
    use crate::core::palette::{ColorId, Rgba, palette};
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

    fn aabb(x0: f32, y0: f32, x1: f32, y1: f32) -> Aabb {
        Aabb::from_corners(Vec2::new(x0, y0), Vec2::new(x1, y1))
    }

    fn aabb_approx_eq(a: Aabb, b: Aabb) -> bool {
        a.min.approx_eq(b.min, EPS) && a.max.approx_eq(b.max, EPS)
    }

    // AC-1

    #[test]
    fn screen_width_scales_with_zoom() {
        // Arrange
        let (world, zoom) = (3.0, 2.5);
        // Act
        let px = screen_width(world, zoom);
        // Assert
        assert!(approx_eq(px, 7.5, EPS));
    }

    #[test]
    fn screen_width_clamps_to_one_pixel() {
        assert!(approx_eq(screen_width(1.0, 0.05), MIN_SCREEN_WIDTH_PX, EPS));
        assert!(approx_eq(screen_width(0.0, 4.0), MIN_SCREEN_WIDTH_PX, EPS));
    }

    #[test]
    fn screen_width_non_finite_or_negative_is_one_pixel() {
        for (w, z) in [
            (f32::NAN, 1.0),
            (1.0, f32::NAN),
            (f32::INFINITY, 1.0),
            (f32::MAX, 20.0),
            (-5.0, 2.0),
            (f32::NEG_INFINITY, 1.0),
        ] {
            assert!(
                approx_eq(screen_width(w, z), MIN_SCREEN_WIDTH_PX, EPS),
                "width {w}, zoom {z}"
            );
        }
    }

    proptest! {
        #[test]
        fn screen_width_is_at_least_one_pixel_and_finite(w in any::<f32>(), z in any::<f32>()) {
            let px = screen_width(w, z);
            prop_assert!(px.is_finite());
            prop_assert!(px >= MIN_SCREEN_WIDTH_PX);
        }
    }

    // AC-2

    #[test]
    fn is_visible_overlapping_is_true() {
        assert!(is_visible(aabb(5.0, 5.0, 15.0, 15.0), aabb(0.0, 0.0, 10.0, 10.0)));
    }

    #[test]
    fn is_visible_touching_edge_is_true() {
        assert!(is_visible(aabb(10.0, 0.0, 20.0, 5.0), aabb(0.0, 0.0, 10.0, 10.0)));
    }

    #[test]
    fn is_visible_disjoint_is_false() {
        let view = aabb(0.0, 0.0, 10.0, 10.0);
        assert!(!is_visible(aabb(11.0, 0.0, 20.0, 5.0), view));
        assert!(!is_visible(aabb(0.0, -9.0, 5.0, -1.0), view));
    }

    #[test]
    fn is_visible_bounds_containing_view_is_true() {
        assert!(is_visible(aabb(-100.0, -100.0, 100.0, 100.0), aabb(0.0, 0.0, 10.0, 10.0)));
    }

    #[test]
    fn is_visible_nan_bounds_is_false() {
        let view = aabb(0.0, 0.0, 10.0, 10.0);
        let nan = Aabb {
            min: Vec2::new(f32::NAN, 0.0),
            max: Vec2::new(5.0, 5.0),
        };
        assert!(!is_visible(nan, view));
        assert!(!is_visible(view, nan));
    }

    #[test]
    fn cull_rect_is_view_grown_by_margin() {
        // Arrange
        let camera = Camera::new(Vec2::new(10.0, 20.0), 2.0);
        let viewport = Vec2::new(200.0, 100.0);
        // Act
        let rect = cull_rect(&camera, viewport);
        // Assert
        let m = CULL_MARGIN_PX / 2.0;
        assert!(aabb_approx_eq(rect, aabb(10.0 - m, 20.0 - m, 110.0 + m, 70.0 + m)));
    }

    // AC-3

    #[test]
    fn conversion_vec2_keeps_coordinates() {
        let v = to_mq(Vec2::new(1.5, -2.25));
        assert!(approx_eq(v.x, 1.5, EPS));
        assert!(approx_eq(v.y, -2.25, EPS));
    }

    #[test]
    fn conversion_color_scales_channels() {
        let c = to_mq_color(Rgba {
            r: 255,
            g: 0,
            b: 51,
            a: 102,
        });
        assert!(approx_eq(c.r, 1.0, EPS));
        assert!(approx_eq(c.g, 0.0, EPS));
        assert!(approx_eq(c.b, 0.2, EPS));
        assert!(approx_eq(c.a, 0.4, EPS));
    }

    #[test]
    fn conversion_palette_ink_is_opaque() {
        let c = to_mq_color(palette(ColorId::INK));
        assert!(approx_eq(c.a, 1.0, EPS));
    }

    // AC-4

    #[test]
    fn joints_threshold() {
        assert!(!stroke_needs_joints(1.0));
        assert!(!stroke_needs_joints(JOINT_THRESHOLD_PX));
        assert!(stroke_needs_joints(JOINT_THRESHOLD_PX + 0.01));
        assert!(stroke_needs_joints(12.0));
        assert!(!stroke_needs_joints(f32::NAN));
    }

    // AC-8

    #[test]
    fn circle_segments_small_radius_is_minimum() {
        assert_eq!(circle_segments(0.5), MIN_SEGMENTS);
        assert_eq!(circle_segments(1.0), MIN_SEGMENTS);
    }

    #[test]
    fn circle_segments_grows_with_radius() {
        let small = circle_segments(20.0);
        let large = circle_segments(400.0);
        assert!(small > MIN_SEGMENTS);
        assert!(large > small);
        assert!(large < MAX_SEGMENTS);
    }

    #[test]
    fn circle_segments_huge_radius_is_capped() {
        assert_eq!(circle_segments(1.0e6), MAX_SEGMENTS);
        assert_eq!(circle_segments(f32::MAX), MAX_SEGMENTS);
    }

    #[test]
    fn circle_segments_non_finite_is_minimum() {
        for r in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -3.0] {
            assert_eq!(circle_segments(r), MIN_SEGMENTS, "radius {r}");
        }
    }

    proptest! {
        #[test]
        fn circle_segments_chord_error_within_tolerance(r in 0.01_f32..10_000.0) {
            let n = circle_segments(r);
            prop_assert!((MIN_SEGMENTS..=MAX_SEGMENTS).contains(&n));
            if n < MAX_SEGMENTS {
                let err = r * (1.0 - (std::f32::consts::PI / f32::from(n)).cos());
                prop_assert!(err <= CHORD_TOLERANCE_PX + EPS, "r {r}, n {n}, err {err}");
            }
        }
    }

    // AC-9

    #[test]
    fn selection_rect_pads_screen_bounds() {
        // Arrange
        let camera = Camera::new(Vec2::new(-10.0, 0.0), 1.0);
        let bounds = aabb(0.0, 0.0, 30.0, 20.0);
        // Act
        let rect = selection_rect(bounds, &camera);
        // Assert
        let p = SELECTION_PAD_PX;
        assert!(aabb_approx_eq(rect, aabb(10.0 - p, -p, 40.0 + p, 20.0 + p)));
    }

    #[test]
    fn selection_rect_padding_is_independent_of_zoom() {
        // Arrange
        let camera = Camera::new(Vec2::ZERO, 4.0);
        let bounds = aabb(1.0, 2.0, 3.0, 5.0);
        // Act
        let rect = selection_rect(bounds, &camera);
        // Assert
        let p = SELECTION_PAD_PX;
        assert!(aabb_approx_eq(rect, aabb(4.0 - p, 8.0 - p, 12.0 + p, 20.0 + p)));
    }
}
