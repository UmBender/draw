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
use crate::shell::render::{Batch, MeshSink};
use crate::shell::render_ctx::RenderCtx;
use draw_gfx::gpu::{Gpu, Vertex};
use macroquad::color::Color;
use macroquad::text::{Font, TextDimensions, load_ttf_font_from_bytes, measure_text};
use std::cell::OnceCell;

/// A glyph quad with the atlas page it samples.
pub type Placed = (GlyphQuad, u32);

/// The atlas of the menu font and its pages on the GPU, kept by the
/// [`RenderCtx`].
pub struct MenuGlyphs<G: Gpu> {
    /// Glyphs of the bundled text font, rasterized at the sizes asked.
    atlas: GlyphAtlas,
    /// The pages of `atlas` on the context's interpreter.
    textures: GlyphTextures<G>,
}

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

impl<G: Gpu> RenderCtx<G> {
    /// Rasterizes the glyphs of `label` into the menu atlas, puts them on
    /// the GPU and returns their quads with the pages' textures. `None`
    /// when the font did not load.
    fn prepare_menu_label(
        &mut self,
        label: &str,
        x: f32,
        y: f32,
        font_size: u16,
    ) -> Option<(Vec<Placed>, Vec<G::Texture>)> {
        let dpi = self.dpi();
        if self.menu.is_none() {
            self.menu = GlyphAtlas::new(TEXT_FONT).ok().map(|atlas| MenuGlyphs {
                atlas,
                textures: GlyphTextures::default(),
            });
        }
        let menu = self.menu.as_mut()?;
        if menu.atlas.overflowed() {
            menu.atlas.clear();
        }
        let px = raster_px(font_size, dpi);
        for ch in label.chars() {
            menu.atlas.prepare(ch, px);
        }
        if let Err(err) = menu.textures.sync(&mut self.gpu, &mut menu.atlas) {
            eprintln!("menu glyph textures: {err}");
        }
        menu.atlas.take_rasterized();
        let quads = label_quads(&menu.atlas, label, x, y, font_size, dpi);
        let pages = (0..menu.atlas.page_count())
            .filter_map(|page| u32::try_from(page).ok())
            .filter_map(|page| menu.textures.get(page).cloned())
            .collect();
        Some((quads, pages))
    }

    /// Draws `label` with its baseline at `y` and its left edge at `x`,
    /// both snapped to whole physical pixels of the window's DPI.
    pub fn draw_menu_label(&mut self, label: &str, x: f32, y: f32, font_size: u16, color: Color) {
        let Some((quads, pages)) = self.prepare_menu_label(label, x, y, font_size) else {
            return;
        };
        let mut batch = Batch::new(PageSink { ctx: self, pages });
        for (quad, page) in &quads {
            batch.glyph(quad, *page, color);
        }
        batch.finish();
    }
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

/// Sends the chunks of a label to a [`RenderCtx`], binding the atlas page
/// each one samples.
struct PageSink<'c, G: Gpu> {
    /// Where the chunks go.
    ctx: &'c mut RenderCtx<G>,
    /// The textures of the atlas pages, by page index.
    pages: Vec<G::Texture>,
}

impl<G: Gpu> MeshSink for PageSink<'_, G> {
    fn draw(&mut self, vertices: &[Vertex], indices: &[u32], page: Option<u32>) {
        let texture = page
            .and_then(|page| usize::try_from(page).ok())
            .and_then(|page| self.pages.get(page).cloned());
        self.ctx.draw_inline(vertices, indices, texture);
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

    /// Goal of T161 (and T155 AC-1): macroquad draw calls appear only in
    /// the `Gpu` interpreter, `gpu_mq.rs`.
    #[test]
    fn no_macroquad_draw_calls_outside_the_interpreter() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shell");
        let Ok(entries) = std::fs::read_dir(&dir) else {
            panic!("cannot read {dir:?}");
        };
        for path in entries.filter_map(|e| e.ok().map(|e| e.path())) {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_rust = path.extension().is_some_and(|ext| ext == "rs");
            if !is_rust || name == "gpu_mq.rs" {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(&path) else {
                panic!("cannot read {path:?}");
            };
            let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
            for call in [
                "draw_mesh(",
                "draw_line(",
                "draw_rectangle(",
                "draw_texture_ex(",
                "draw_text_ex(",
                "render_target_ex(",
                "set_camera(",
            ] {
                assert!(!production.contains(call), "{name} calls {call}");
            }
        }
    }

    /// What stays until T149 removes macroquad: `(file, text on the line)`.
    /// The list may only shrink; every entry names T149.
    const MACROQUAD_ALLOWLIST: &[(&str, &str)] = &[
        // T149: `Color` of the menus and the drawing.
        ("app.rs", "macroquad::color::Color"),
        ("command_palette.rs", "macroquad::color::Color"),
        ("minimap.rs", "macroquad::color::Color"),
        ("render.rs", "macroquad::color::Color"),
        ("sheet.rs", "macroquad::color::Color"),
        ("toolbar.rs", "macroquad::color::Color"),
        ("ui_shapes.rs", "macroquad::color::Color"),
        ("ui_text.rs", "macroquad::color::Color"),
        // T149: the menu font measure.
        ("ui_text.rs", "macroquad::text::"),
        // T149: `Vec2` of the macroquad vertices in the batch.
        ("render.rs", "macroquad::math::Vec2"),
        // T149: window config and signals.
        ("app.rs", "macroquad::conf::"),
        ("app.rs", "macroquad::miniquad::conf::"),
        ("app.rs", "macroquad::miniquad::signal::"),
        ("main.rs", "macroquad::miniquad::signal"),
        ("main.rs", "macroquad::Window::from_config"),
        // T149: `LinuxBackend`.
        ("backend.rs", "macroquad::miniquad::conf::LinuxBackend"),
    ];

    /// The lines of the Rust files of the shell and `main.rs` that name a
    /// macroquad or miniquad path outside tests and comments, as
    /// `(file, line)`; the two interpreters are left out.
    fn macroquad_lines() -> Vec<(String, String)> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut paths = vec![root.join("main.rs")];
        let Ok(entries) = std::fs::read_dir(root.join("shell")) else {
            panic!("cannot read the shell directory");
        };
        paths.extend(entries.filter_map(|e| e.ok().map(|e| e.path())));
        let mut found = Vec::new();
        for path in paths {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_rust = path.extension().is_some_and(|ext| ext == "rs");
            if !is_rust || name == "gpu_mq.rs" || name == "platform_mq.rs" {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(&path) else {
                panic!("cannot read {path:?}");
            };
            let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
            for line in production.lines() {
                let code = line.trim();
                if code.starts_with("//") {
                    continue;
                }
                if code.contains("macroquad::") || code.contains("miniquad::") {
                    found.push((name.to_owned(), code.to_owned()));
                }
            }
        }
        found
    }

    /// AC-2 of T167: no macroquad or miniquad path outside its two
    /// interpreters, except the allowlist.
    #[test]
    fn shell_sources_outside_the_mq_interpreters_name_no_macroquad() {
        for (file, line) in macroquad_lines() {
            let allowed = MACROQUAD_ALLOWLIST
                .iter()
                .any(|(name, text)| *name == file && line.contains(text));
            assert!(allowed, "{file} names macroquad outside the list: {line}");
        }
    }

    /// AC-2 of T167: an allowlist entry that matches no line is removed.
    #[test]
    fn macroquad_allowlist_has_no_stale_entry() {
        let lines = macroquad_lines();
        for (name, text) in MACROQUAD_ALLOWLIST {
            let used = lines
                .iter()
                .any(|(file, line)| file == name && line.contains(text));
            assert!(used, "{name} no longer needs the entry {text}");
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
