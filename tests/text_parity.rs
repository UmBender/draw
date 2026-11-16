//! The retained text quads against the current text path (T180, AC-1 and
//! AC-3): the world quads mapped through the camera equal the vertices
//! `push_text_block` and `push_text_centered` send to a sink.

#![allow(clippy::unwrap_used)]

use draw::core::camera::Camera;
use draw::core::document::ShapeId;
use draw::core::geom::{Aabb, Vec2};
use draw::core::tex::Face;
use draw::shell::app::{MATH_FONT, TEXT_FONT};
use draw::shell::glyphs::{GlyphAtlas, PAGE_SIDE, WHITE_UV};
use draw::shell::render::{
    Batch, LABEL_RASTER_SIZES, MIN_SCREEN_WIDTH_PX, MeshSink, TEXT_MIN_PX, TEXT_RASTER_SIZES,
    label_raster, prepare_text_block, prepare_text_centered, push_text_block, push_text_centered,
    text_raster, warn_color,
};
use draw_gfx::gpu::Vertex;
use draw_gfx::text::{self, GlyphEntry, GlyphSource, TextCache, TextSpec};
use macroquad::color::Color;

const INK: Color = Color::new(0.1, 0.2, 0.3, 1.0);
const ZOOMS: [f32; 3] = [0.25, 1.0, 4.0];

struct Source<'a>(&'a GlyphAtlas);

impl GlyphSource for Source<'_> {
    fn page_side(&self) -> u16 {
        PAGE_SIDE
    }

    fn glyph(&self, face: Face, ch: char, rpx: u16) -> Option<GlyphEntry> {
        let e = self.0.get_in(face, ch, rpx)?;
        Some(GlyphEntry {
            page: e.page,
            x: e.rect.x,
            y: e.rect.y,
            w: e.rect.w,
            h: e.rect.h,
            xmin: e.xmin,
            ymin: e.ymin,
            advance: e.advance,
        })
    }
}

#[derive(Default)]
struct Sink(Vec<Vertex>);

impl MeshSink for Sink {
    fn draw(&mut self, vertices: &[Vertex], _indices: &[u32], _page: Option<u32>) {
        self.0.extend_from_slice(vertices);
    }
}

fn atlas() -> GlyphAtlas {
    GlyphAtlas::new(TEXT_FONT)
        .unwrap()
        .with_math(MATH_FONT)
        .unwrap()
}

fn is_white(v: &Vertex) -> bool {
    v.uv == WHITE_UV
}

fn close(a: &[Vertex], b: &[Vertex], camera: &Camera) {
    assert_eq!(a.len(), b.len(), "vertex count");
    for (x, y) in a.iter().zip(b) {
        let s = camera.world_to_screen(Vec2::new(y.pos[0], y.pos[1]));
        assert!((x.pos[0] - s.x).abs() < 1e-2, "x {} vs {}", x.pos[0], s.x);
        assert!((x.pos[1] - s.y).abs() < 1e-2, "y {} vs {}", x.pos[1], s.y);
        assert_eq!(x.color, y.color);
        if !is_white(x) {
            assert!((x.uv[0] - y.uv[0]).abs() < 1e-6 && (x.uv[1] - y.uv[1]).abs() < 1e-6);
        }
    }
}

fn block_parity(text: &str, dpi: f32) {
    for zoom in ZOOMS {
        // Arrange
        let camera = Camera::new(Vec2::new(-30.0, 12.0), zoom);
        let (origin, size) = (Vec2::new(5.0, 7.0), 24.0);
        let mut atlas = atlas();
        prepare_text_block(&mut atlas, text, size, &camera, dpi);
        let rgba: [u8; 4] = INK.into();
        let warn: [u8; 4] = warn_color(INK).into();
        // Act: the current path.
        let mut batch = Batch::new(Sink::default());
        push_text_block(&mut batch, &atlas, origin, text, size, &camera, dpi, INK);
        let current = batch.finish().0;
        let mut cache = TextCache::new(dpi);
        let spec = TextSpec::Block {
            text,
            origin,
            size,
            color: rgba,
            warn,
        };
        cache.update(ShapeId(1), 0, &spec, zoom, &Source(&atlas));
        let chunk = cache.get(ShapeId(1)).unwrap();
        // Assert: glyph quads and rules each in order.
        let glyphs: Vec<Vertex> = chunk
            .runs()
            .iter()
            .flat_map(|r| r.vertices.iter().copied())
            .collect();
        let (white, ink): (Vec<_>, Vec<_>) = current.into_iter().partition(is_white);
        assert!(!ink.is_empty(), "the scene draws glyphs at zoom {zoom}");
        close(&ink, &glyphs, &camera);
        close(&white, &chunk.rules().vertices, &camera);
    }
}

// AC-1

#[test]
fn raster_ladders_match_the_shell() {
    assert_eq!(text::TEXT_RASTER_SIZES, TEXT_RASTER_SIZES);
    assert_eq!(text::LABEL_RASTER_SIZES, LABEL_RASTER_SIZES);
    assert!((text::TEXT_MIN_PX - TEXT_MIN_PX).abs() < f32::EPSILON);
    assert!((text::MIN_RULE_PX - MIN_SCREEN_WIDTH_PX).abs() < f32::EPSILON);
    assert_eq!(text::MAX_RASTER_PX, draw::shell::glyphs::MAX_RASTER_PX);
    for px in [4.0, 12.0, 13.0, 77.0, 300.0] {
        assert_eq!(text::text_raster(px), text_raster(px));
        assert_eq!(text::label_raster(px), label_raster(px));
    }
    for (r, dpi) in [(12, 1.0), (256, 4.0), (24, 1.5), (16, f32::NAN)] {
        assert_eq!(
            text::raster_px(r, dpi),
            draw::shell::glyphs::raster_px(r, dpi)
        );
    }
}

// AC-3

#[test]
fn retained_block_matches_push_text_block() {
    block_parity("hello\nworld 42", 1.0);
    block_parity("hello", 2.0);
}

#[test]
fn retained_formula_matches_push_text_block() {
    block_parity("a $\\frac{x_1}{2}$ b $\\sqrt{y^2}$ $\\foo$", 1.0);
    block_parity("$\\sum_{i=0}^{n} a_i$", 2.0);
}

#[test]
fn retained_label_matches_push_text_centered() {
    for zoom in ZOOMS {
        // Arrange: a 30 px label centred in a screen rectangle.
        let camera = Camera::new(Vec2::new(-30.0, 12.0), zoom);
        let (size, dpi, label) = (30.0, 1.0, "ab 12");
        let world = Aabb::from_corners(Vec2::new(0.0, 0.0), Vec2::new(80.0, 40.0));
        let rect = Aabb::from_corners(
            camera.world_to_screen(world.min),
            camera.world_to_screen(world.max),
        );
        let mut atlas = atlas();
        prepare_text_centered(&mut atlas, label, size, dpi);
        // Act
        let mut batch = Batch::new(Sink::default());
        push_text_centered(&mut batch, &atlas, label, rect, size, dpi, INK);
        let current = batch.finish().0;
        let mut cache = TextCache::new(dpi);
        let spec = TextSpec::Label {
            text: label,
            center: world.center(),
            world_size: size / zoom,
            color: INK.into(),
        };
        cache.update(ShapeId(1), 0, &spec, zoom, &Source(&atlas));
        // Assert
        let quads: Vec<Vertex> = cache
            .get(ShapeId(1))
            .unwrap()
            .runs()
            .iter()
            .flat_map(|r| r.vertices.iter().copied())
            .collect();
        close(&current, &quads, &camera);
    }
}
