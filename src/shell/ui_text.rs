//! Text of the toolbar, the command palette, the shortcut sheet and the
//! page indicator (ADR-T123-1).
//!
//! macroquad's built-in font is a small bitmap-style face, and its glyphs
//! are drawn at fractional positions, so menus looked rough and shimmered
//! when anything moved. Menus use the bundled text font instead (the face
//! of the canvas text), and every string is placed on a whole physical
//! pixel.

use crate::shell::app::TEXT_FONT;
use macroquad::color::Color;
use macroquad::miniquad;
use macroquad::text::{
    Font, TextDimensions, TextParams, draw_text_ex, load_ttf_font_from_bytes, measure_text,
};
use std::cell::OnceCell;

thread_local! {
    /// The menu font, loaded on first use: it needs the graphics context.
    static FONT: OnceCell<Option<Font>> = const { OnceCell::new() };
}

/// Runs `body` with the menu font, or `None` when it did not load (the
/// built-in font is used then).
fn with_font<T>(body: impl FnOnce(Option<&Font>) -> T) -> T {
    FONT.with(|cell| {
        let font = cell.get_or_init(|| load_ttf_font_from_bytes(TEXT_FONT).ok());
        body(font.as_ref())
    })
}

/// The size of `label` at `font_size` pixels, as `measure_text` reports it.
#[must_use]
pub fn measure(label: &str, font_size: u16) -> TextDimensions {
    with_font(|font| measure_text(label, font, font_size, 1.0))
}

/// `value` rounded to a whole physical pixel of a window scaled by `dpi`.
#[must_use]
pub fn snap(value: f32, dpi: f32) -> f32 {
    if dpi.is_finite() && dpi > 0.0 && value.is_finite() {
        (value * dpi).round() / dpi
    } else {
        value
    }
}

/// Draws `label` with its baseline at `y` and its left edge at `x`, both
/// snapped to whole physical pixels.
pub fn draw(label: &str, x: f32, y: f32, font_size: u16, color: Color) {
    let dpi = miniquad::window::dpi_scale();
    with_font(|font| {
        draw_text_ex(
            label,
            snap(x, dpi),
            snap(y, dpi),
            TextParams {
                font,
                font_size,
                color,
                ..TextParams::default()
            },
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;

    #[test]
    fn snap_rounds_to_physical_pixels() {
        assert!(approx_eq(snap(10.4, 1.0), 10.0, 1e-6));
        assert!(approx_eq(snap(10.6, 1.0), 11.0, 1e-6));
        assert!(approx_eq(snap(10.3, 2.0), 10.5, 1e-6));
    }

    #[test]
    fn snap_leaves_odd_input_alone() {
        assert!(snap(f32::NAN, 1.0).is_nan());
        assert!(approx_eq(snap(3.7, 0.0), 3.7, 1e-6));
        assert!(approx_eq(snap(3.7, f32::NAN), 3.7, 1e-6));
    }

    /// Every character the menus show is in the bundled font (T123).
    #[test]
    fn menu_font_has_every_printable_ascii_character() {
        let Ok(face) = ttf_parser::Face::parse(TEXT_FONT, 0) else {
            panic!("the text font does not parse");
        };
        for ch in ' '..='~' {
            assert!(face.glyph_index(ch).is_some(), "{ch:?} is missing");
        }
    }
}
