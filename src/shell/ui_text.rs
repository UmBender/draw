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
    /// Leaked on purpose: dropping its texture after the window is gone
    /// (a thread-local destructor at exit) crashes.
    static FONT: OnceCell<Option<&'static Font>> = const { OnceCell::new() };
}

/// Runs `body` with the menu font, or `None` when it did not load (the
/// built-in font is used then).
fn with_font<T>(body: impl FnOnce(Option<&Font>) -> T) -> T {
    let font = FONT.with(|cell| {
        *cell.get_or_init(|| {
            load_ttf_font_from_bytes(TEXT_FONT)
                .ok()
                .map(|font| &*Box::leak(Box::new(font)))
        })
    });
    body(font)
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
    use crate::shell::glyphs::raster_px;

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

    /// An atlas over the menu font with `label` rasterized at `dpi`.
    fn atlas_with(label: &str, font_size: u16, dpi: f32) -> GlyphAtlas {
        let mut atlas = GlyphAtlas::new(TEXT_FONT).expect("the text font parses");
        let px = raster_px(font_size, dpi);
        for ch in label.chars() {
            atlas.prepare(ch, px);
        }
        atlas
    }

    #[test]
    fn label_quads_place_glyphs_along_the_baseline() {
        let atlas = atlas_with("ab", 16, 1.0);
        let quads = label_quads(&atlas, "ab", 10.0, 40.0, 16, 1.0);
        assert_eq!(quads.len(), 2);
        let advance = atlas.get('a', 16).expect("a is prepared").advance;
        let (a, b) = (&quads[0].0, &quads[1].0);
        // Both glyphs sit above the baseline; the second starts one
        // advance after the first, give or take their own bearings.
        assert!(a.max.y <= 40.0 + 4.0 && b.max.y <= 40.0 + 4.0);
        assert!(b.min.x > a.min.x);
        let entry_b = atlas.get('b', 16).expect("b is prepared");
        assert!(approx_eq(b.min.x, 10.0 + advance + entry_b.xmin, 1e-3));
    }

    #[test]
    fn label_quads_skip_spaces() {
        let atlas = atlas_with("a b", 16, 1.0);
        let quads = label_quads(&atlas, "a b", 0.0, 20.0, 16, 1.0);
        assert_eq!(quads.len(), 2);
        let space = atlas.get(' ', 16).expect("space is prepared").advance;
        let a = atlas.get('a', 16).expect("a is prepared");
        let b = atlas.get('b', 16).expect("b is prepared");
        assert!(approx_eq(
            quads[1].0.min.x,
            a.advance + space + b.xmin,
            1e-3
        ));
    }

    #[test]
    fn label_quads_snap_the_origin_to_physical_pixels() {
        let atlas = atlas_with("a", 16, 2.0);
        let exact = label_quads(&atlas, "a", 10.5, 20.5, 16, 2.0);
        let shifted = label_quads(&atlas, "a", 10.51, 20.49, 16, 2.0);
        assert_eq!(exact, shifted);
    }

    #[test]
    fn label_quads_of_an_unprepared_label_are_empty() {
        let atlas = GlyphAtlas::new(TEXT_FONT).expect("the text font parses");
        assert!(label_quads(&atlas, "abc", 0.0, 0.0, 16, 1.0).is_empty());
    }

    /// AC-1 of T161: no macroquad draw call in this file.
    #[test]
    fn ui_text_uses_no_macroquad_draw_calls() {
        let source = include_str!("ui_text.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for call in [
            "draw_text_ex(",
            "draw_text(",
            "draw_mesh(",
            "draw_texture_ex(",
            "draw_rectangle(",
        ] {
            assert!(!production.contains(call), "{call} is still used");
        }
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
