//! DPI, scale and window metrics for X11, as pure functions (T175 AC-2).

use crate::platform::WindowMetrics;

/// The DPI of `Xft.dpi` in an X resource string, if it holds a positive
/// number.
#[must_use]
pub fn xft_dpi(resources: &str) -> Option<f32> {
    todo!("xft_dpi {resources}")
}

/// DPI from a screen size in pixels and millimetres, `None` if either is
/// not positive.
#[must_use]
pub fn screen_dpi(pixels: i32, millimetres: i32) -> Option<f32> {
    todo!("screen_dpi {pixels} {millimetres}")
}

/// `Xft.dpi` if present, else the screen size, else 96.
#[must_use]
pub fn resolve_dpi(resources: Option<&str>, pixels: i32, millimetres: i32) -> f32 {
    todo!("resolve_dpi {resources:?} {pixels} {millimetres}")
}

/// Physical pixels per logical pixel for `dpi`: `dpi / 96` in `1..=4`,
/// `1` for a non-finite value.
#[must_use]
pub fn scale_from_dpi(dpi: f32) -> f32 {
    todo!("scale_from_dpi {dpi}")
}

/// The metrics of a window of `width × height` physical pixels at `scale`.
#[must_use]
pub fn window_metrics(width: u32, height: u32, scale: f32) -> WindowMetrics {
    todo!("window_metrics {width} {height} {scale}")
}
