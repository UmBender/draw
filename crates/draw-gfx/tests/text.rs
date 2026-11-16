//! Retained text and formula quads (T180, AC-1 and AC-2).

#![allow(clippy::unwrap_used)]

use draw_core::core::document::ShapeId;
use draw_core::core::geom::{Vec2, approx_eq};
use draw_core::core::tex::Face;
use draw_gfx::text::{
    GlyphEntry, GlyphSource, TextCache, TextSpec, label_raster, raster_px, text_raster,
    text_screen_px,
};

const EPS: f32 = 1e-3;
const INK: [u8; 4] = [10, 20, 30, 255];
const WARN: [u8; 4] = [200, 0, 0, 255];
const NONE: [[f32; 2]; 0] = [];

/// Every glyph is `rpx/2` wide and `rpx` tall; `Z` lives on page 1.
struct Fake {
    missing: Option<char>,
}

impl GlyphSource for Fake {
    fn page_side(&self) -> u16 {
        1024
    }

    fn glyph(&self, _face: Face, ch: char, rpx: u16) -> Option<GlyphEntry> {
        if self.missing == Some(ch) {
            return None;
        }
        Some(GlyphEntry {
            page: u32::from(ch == 'Z'),
            x: 16,
            y: 16,
            w: rpx / 2,
            h: rpx,
            xmin: 0.0,
            ymin: 0.0,
            advance: f32::from(rpx) * 0.6,
        })
    }
}

fn fake() -> Fake {
    Fake { missing: None }
}

fn block(text: &str, size: f32) -> TextSpec<'_> {
    TextSpec::Block {
        text,
        origin: Vec2::new(10.0, 20.0),
        size,
        color: INK,
        warn: WARN,
    }
}

fn label(text: &str, world_size: f32) -> TextSpec<'_> {
    TextSpec::Label {
        text,
        center: Vec2::new(100.0, 50.0),
        world_size,
        color: INK,
    }
}

fn positions(cache: &TextCache, id: u64) -> Vec<[f32; 2]> {
    cache
        .get(ShapeId(id))
        .unwrap()
        .runs()
        .iter()
        .flat_map(|r| r.vertices.iter().map(|v| v.pos))
        .collect()
}

// AC-1

#[test]
fn text_raster_below_min_is_none() {
    assert_eq!(text_screen_px(10.0, 0.3), None);
    assert_eq!(text_screen_px(f32::NAN, 1.0), None);
    assert!(text_screen_px(10.0, 0.5).is_some());
}

#[test]
fn text_raster_picks_smallest_at_or_above() {
    let (raster, scale) = text_raster(30.0);
    assert_eq!(raster, 32);
    assert!(approx_eq(scale, 30.0 / 32.0, EPS));
    assert_eq!(text_raster(9999.0).0, 256);
    assert_eq!(label_raster(20.0).0, 32);
    assert_eq!(raster_px(256, 4.0), 512);
    assert_eq!(raster_px(12, f32::NAN), 12);
}

// AC-2

#[test]
fn block_quads_are_world_space_and_zoom_independent() {
    // Arrange: size 20 at zoom 0.9 and 1.0 is the same raster step.
    let mut a = TextCache::new(1.0);
    let mut b = TextCache::new(1.0);
    // Act
    a.update(ShapeId(1), 0, &block("ab", 20.0), 0.9, &fake());
    b.update(ShapeId(1), 0, &block("ab", 20.0), 1.0, &fake());
    // Assert: two quads, equal geometry, world units (near the origin).
    let pa = positions(&a, 1);
    assert_eq!(pa.len(), 8);
    assert_eq!(pa, positions(&b, 1));
    assert!(pa.iter().all(|p| p[0] >= 10.0 && p[0] < 60.0));
}

#[test]
fn pan_re_emits_nothing() {
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.0, &fake());
    let before = cache.emissions();
    for _ in 0..5 {
        assert!(!cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.0, &fake()));
    }
    assert_eq!(cache.emissions(), before);
}

#[test]
fn zoom_inside_step_re_emits_nothing() {
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &block("abc", 20.0), 0.9, &fake());
    let before = cache.emissions();
    assert!(!cache.update(ShapeId(1), 0, &block("abc", 20.0), 0.95, &fake()));
    assert!(!cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.0, &fake()));
    assert_eq!(cache.emissions(), before);
}

#[test]
fn zoom_across_step_re_emits_that_block_only() {
    // Arrange: a size-20 block and a size-200 block at zoom 1.
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.0, &fake());
    cache.update(ShapeId(2), 0, &block("abc", 200.0), 1.0, &fake());
    let before = cache.emissions();
    // Act: 1.1 moves the first to the 24 step; the second stays on 256.
    let first = cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.1, &fake());
    let second = cache.update(ShapeId(2), 0, &block("abc", 200.0), 1.1, &fake());
    // Assert
    assert!(first);
    assert!(!second);
    assert_eq!(cache.emissions(), before + 1);
}

#[test]
fn edit_re_emits_that_block() {
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.0, &fake());
    assert!(cache.update(ShapeId(1), 1, &block("abcd", 20.0), 1.0, &fake()));
    assert_eq!(positions(&cache, 1).len(), 16);
}

#[test]
fn atlas_epoch_change_re_emits() {
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.0, &fake());
    cache.set_epoch(1);
    assert!(cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.0, &fake()));
}

#[test]
fn glyphs_on_two_pages_make_two_runs() {
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &block("aZb", 20.0), 1.0, &fake());
    let pages: Vec<u32> = cache
        .get(ShapeId(1))
        .unwrap()
        .runs()
        .iter()
        .map(|r| r.page)
        .collect();
    assert_eq!(pages, vec![0, 1, 0]);
}

#[test]
fn formula_has_quads_and_rules() {
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &block("$\\frac{a}{b}$", 40.0), 1.0, &fake());
    let chunk = cache.get(ShapeId(1)).unwrap();
    assert!(chunk.runs().iter().map(|r| r.indices.len()).sum::<usize>() >= 12);
    assert_ne!(chunk.rules().indices.len(), 0);
}

#[test]
fn label_is_centered_and_capped_label_re_emits_on_zoom() {
    // Arrange: a label of world size 10 at zoom 2 (20 px).
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &label("ab", 10.0), 2.0, &fake());
    let p = positions(&cache, 1);
    let (min_x, max_x) = p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), v| {
        (lo.min(v[0]), hi.max(v[0]))
    });
    // Assert: ink centred on x = 100 within the advance box.
    assert!((f32::midpoint(min_x, max_x) - 100.0).abs() < 5.0);
    // Same world size: nothing to re-emit; another world size: re-emit.
    assert!(!cache.update(ShapeId(1), 0, &label("ab", 10.0), 2.0, &fake()));
    assert!(cache.update(ShapeId(1), 0, &label("ab", 5.0), 4.0, &fake()));
}

#[test]
fn missing_glyph_is_skipped() {
    let mut cache = TextCache::new(1.0);
    let source = Fake { missing: Some('b') };
    cache.update(ShapeId(1), 0, &block("abc", 20.0), 1.0, &source);
    assert_eq!(positions(&cache, 1).len(), 8);
}

#[test]
fn text_below_minimum_size_emits_nothing() {
    let mut cache = TextCache::new(1.0);
    cache.update(ShapeId(1), 0, &block("abc", 20.0), 0.1, &fake());
    assert_eq!(positions(&cache, 1), NONE);
}

#[test]
fn non_finite_input_emits_nothing() {
    let mut cache = TextCache::new(1.0);
    for zoom in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
        cache.update(ShapeId(1), 0, &block("abc", 20.0), zoom, &fake());
        assert_eq!(positions(&cache, 1), NONE);
    }
    cache.update(ShapeId(2), 0, &block("abc", f32::NAN), 1.0, &fake());
    assert_eq!(positions(&cache, 2), NONE);
    cache.update(ShapeId(3), 0, &label("ab", f32::NAN), 1.0, &fake());
    assert_eq!(positions(&cache, 3), NONE);
}
