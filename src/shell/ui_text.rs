//! Text of the toolbar, the command palette, the shortcut sheet and the
//! page indicator (ADR-T123-1).
//!
//! macroquad's built-in font is a small bitmap-style face, and its glyphs
//! are drawn at fractional positions, so menus looked rough and shimmered
//! when anything moved. Menus use the bundled text font instead (the face
//! of the canvas text), and every string is placed on a whole physical
//! pixel.

use crate::core::geom::Vec2;
use crate::shell::app::TEXT_FONT;
use crate::shell::glyphs::{
    GlyphAtlas, GlyphQuad, GlyphTextures, glyph_quad, glyph_scale, raster_px,
};
use crate::shell::gpu_mq::{MqGpu, MqTexture};
use crate::shell::render::{Batch, MeshSink, draw_inline, with_gpu};
use draw_gfx::gpu::Vertex;
use macroquad::color::Color;
use macroquad::miniquad;
use macroquad::text::{Font, TextDimensions, load_ttf_font_from_bytes, measure_text};
use std::cell::{OnceCell, RefCell};

/// A glyph quad with the atlas page it samples.
pub type Placed = (GlyphQuad, u32);

/// The atlas of the menu font and its pages on the GPU.
struct Glyphs {
    /// Glyphs of the bundled text font, rasterized at the sizes asked.
    atlas: GlyphAtlas,
    /// The pages of `atlas` on the shared interpreter.
    textures: GlyphTextures<MqGpu>,
}

thread_local! {
    /// The menu glyphs; `None` until first used, and for good when the font
    /// does not parse.
    static GLYPHS: RefCell<Option<Glyphs>> = const { RefCell::new(None) };
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
    let Some((quads, pages)) = prepare(label, x, y, font_size, dpi) else {
        return;
    };
    let mut batch = Batch::new(PageSink { pages });
    for (quad, page) in &quads {
        batch.glyph(quad, *page, color);
    }
    batch.finish();
}

/// Rasterizes the glyphs of `label` into the menu atlas, puts them on the
/// GPU and returns their quads with the pages' textures. `None` when the
/// font did not load.
fn prepare(
    label: &str,
    x: f32,
    y: f32,
    font_size: u16,
    dpi: f32,
) -> Option<(Vec<Placed>, Vec<MqTexture>)> {
    GLYPHS.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_none() {
            *guard = GlyphAtlas::new(TEXT_FONT).ok().map(|atlas| Glyphs {
                atlas,
                textures: GlyphTextures::default(),
            });
        }
        let glyphs = guard.as_mut()?;
        if glyphs.atlas.overflowed() {
            glyphs.atlas.clear();
        }
        let px = raster_px(font_size, dpi);
        for ch in label.chars() {
            glyphs.atlas.prepare(ch, px);
        }
        let synced = with_gpu(|gpu| glyphs.textures.sync(gpu, &mut glyphs.atlas));
        if let Err(err) = synced {
            eprintln!("menu glyph textures: {err}");
        }
        glyphs.atlas.take_rasterized();
        let quads = label_quads(&glyphs.atlas, label, x, y, font_size, dpi);
        let pages = (0..glyphs.atlas.page_count())
            .filter_map(|page| u32::try_from(page).ok())
            .filter_map(|page| glyphs.textures.get(page).cloned())
            .collect();
        Some((quads, pages))
    })
}

/// The glyph quads of `label` with its baseline at `y` and its left edge at
/// `x` (snapped to whole physical pixels at `dpi`), each with the atlas page
/// it samples. Glyphs missing from `atlas` and blank ones (spaces) add no
/// quad; every glyph advances the pen.
#[must_use]
pub fn label_quads(
    atlas: &GlyphAtlas,
    label: &str,
    x: f32,
    y: f32,
    font_size: u16,
    dpi: f32,
) -> Vec<Placed> {
    let px = raster_px(font_size, dpi);
    let scale = glyph_scale(f32::from(font_size), px);
    let mut pen = Vec2::new(snap(x, dpi), snap(y, dpi));
    let mut out = Vec::new();
    for ch in label.chars() {
        let Some(entry) = atlas.get(ch, px) else {
            continue;
        };
        if let Some(quad) = glyph_quad(pen, &entry, scale) {
            out.push((quad, entry.page));
        }
        pen.x += entry.advance * scale;
    }
    out
}

/// Sends the chunks of a label to the shared interpreter, binding the
/// atlas page each one samples.
struct PageSink {
    /// The textures of the atlas pages, by page index.
    pages: Vec<MqTexture>,
}

impl MeshSink for PageSink {
    fn draw(&mut self, vertices: &[Vertex], indices: &[u32], page: Option<u32>) {
        let texture = page
            .and_then(|page| usize::try_from(page).ok())
            .and_then(|page| self.pages.get(page).cloned());
        draw_inline(vertices, indices, texture);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;
    use crate::shell::glyphs::AtlasEntry;

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
        let Ok(mut atlas) = GlyphAtlas::new(TEXT_FONT) else {
            panic!("the text font does not parse");
        };
        let px = raster_px(font_size, dpi);
        for ch in label.chars() {
            atlas.prepare(ch, px);
        }
        atlas
    }

    /// The entry of `ch` at 16 px, which the test prepared.
    fn entry(atlas: &GlyphAtlas, ch: char) -> AtlasEntry {
        let Some(entry) = atlas.get(ch, 16) else {
            panic!("{ch:?} is not prepared");
        };
        entry
    }

    #[test]
    fn label_quads_place_glyphs_along_the_baseline() {
        let atlas = atlas_with("ab", 16, 1.0);
        let quads = label_quads(&atlas, "ab", 10.0, 40.0, 16, 1.0);
        assert_eq!(quads.len(), 2);
        let advance = entry(&atlas, 'a').advance;
        let (a, b) = (&quads[0].0, &quads[1].0);
        // Both glyphs sit above the baseline; the second starts one
        // advance after the first, give or take its own bearing.
        assert!(a.max.y <= 40.0 + 4.0 && b.max.y <= 40.0 + 4.0);
        assert!(b.min.x > a.min.x);
        assert!(approx_eq(
            b.min.x,
            10.0 + advance + entry(&atlas, 'b').xmin,
            1e-3
        ));
    }

    #[test]
    fn label_quads_skip_spaces() {
        let atlas = atlas_with("a b", 16, 1.0);
        let quads = label_quads(&atlas, "a b", 0.0, 20.0, 16, 1.0);
        assert_eq!(quads.len(), 2);
        let pen = entry(&atlas, 'a').advance + entry(&atlas, ' ').advance;
        assert!(approx_eq(
            quads[1].0.min.x,
            pen + entry(&atlas, 'b').xmin,
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
        let atlas = atlas_with("", 16, 1.0);
        assert_eq!(label_quads(&atlas, "abc", 0.0, 0.0, 16, 1.0).len(), 0);
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
