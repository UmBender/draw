//! DPI, scale and window metrics for X11, as pure functions (T175 AC-2).

use crate::platform::WindowMetrics;

/// The DPI a screen without information has.
const BASE_DPI: f32 = 96.0;
const MM_PER_INCH: f32 = 25.4;

/// The DPI of `Xft.dpi` in an X resource string, if it holds a positive
/// number.
#[must_use]
pub fn xft_dpi(resources: &str) -> Option<f32> {
    let value = resources
        .lines()
        .find_map(|line| line.strip_prefix("Xft.dpi:"))?;
    let dpi: f32 = value.trim().parse().ok()?;
    (dpi.is_finite() && dpi > 0.0).then_some(dpi)
}

/// DPI from a screen size in pixels and millimetres, `None` if either is
/// not positive.
#[must_use]
pub fn screen_dpi(pixels: i32, millimetres: i32) -> Option<f32> {
    (pixels > 0 && millimetres > 0).then(|| pixels as f32 * MM_PER_INCH / millimetres as f32)
}

/// `Xft.dpi` if present, else the screen size, else 96.
#[must_use]
pub fn resolve_dpi(resources: Option<&str>, pixels: i32, millimetres: i32) -> f32 {
    resources
        .and_then(xft_dpi)
        .or_else(|| screen_dpi(pixels, millimetres))
        .unwrap_or(BASE_DPI)
}

/// Physical pixels per logical pixel for `dpi`: `dpi / 96` in `1..=4`,
/// `1` for a non-finite value.
#[must_use]
pub fn scale_from_dpi(dpi: f32) -> f32 {
    if dpi.is_finite() {
        (dpi / BASE_DPI).clamp(1.0, 4.0)
    } else {
        1.0
    }
}

/// The metrics of a window of `width × height` physical pixels at `scale`.
#[must_use]
pub fn window_metrics(width: u32, height: u32, scale: f32) -> WindowMetrics {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    WindowMetrics {
        width: width as f32 / scale,
        height: height as f32 / scale,
        scale,
    }
}
