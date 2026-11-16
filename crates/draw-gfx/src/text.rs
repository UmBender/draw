//! Retained text, labels and formulas: glyph quads in world units, one
//! chunk per block, re-emitted only when the raster step changes
//! (ADR-T180-1).

use std::collections::HashMap;

use draw_core::core::document::ShapeId;
use draw_core::core::geom::{Aabb, Vec2};
use draw_core::core::tex::Face;

use crate::gpu::Vertex;
use crate::tessellate::Mesh;

/// Font sizes text is rasterised at, ascending (ADR-T29-1).
pub const TEXT_RASTER_SIZES: [u16; 12] = [12, 16, 20, 24, 32, 40, 48, 64, 96, 128, 192, 256];

/// Font sizes labels are rasterised at, ascending.
pub const LABEL_RASTER_SIZES: [u16; 4] = [16, 32, 64, 128];

/// Smallest on-screen text size in pixels; smaller text is not drawn.
pub const TEXT_MIN_PX: f32 = 4.0;

/// Largest physical raster size.
pub const MAX_RASTER_PX: u16 = 512;

/// Narrowest on-screen rule, in pixels.
pub const MIN_RULE_PX: f32 = 1.0;

/// A glyph bitmap in an atlas page and its metrics, in texels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphEntry {
    /// Atlas page.
    pub page: u32,
    /// Left of the bitmap in the page.
    pub x: u16,
    /// Top of the bitmap in the page.
    pub y: u16,
    /// Bitmap width.
    pub w: u16,
    /// Bitmap height.
    pub h: u16,
    /// Left of the bitmap, right of the pen.
    pub xmin: f32,
    /// Bottom of the bitmap, above the baseline.
    pub ymin: f32,
    /// Horizontal advance.
    pub advance: f32,
}

/// Where glyph bitmaps come from (the shell's atlas).
pub trait GlyphSource {
    /// Side of an atlas page in texels.
    fn page_side(&self) -> u16;
    /// The prepared glyph of `ch` in `face` at `rpx` physical pixels.
    fn glyph(&self, face: Face, ch: char, rpx: u16) -> Option<GlyphEntry>;
}

/// What a chunk draws.
#[derive(Debug, Clone, Copy)]
pub enum TextSpec<'a> {
    /// A text block or formula at a world origin.
    Block {
        /// The text, formulas between `$`.
        text: &'a str,
        /// Top-left of the block, world units.
        origin: Vec2,
        /// World font size.
        size: f32,
        /// Ink colour bytes.
        color: [u8; 4],
        /// Colour bytes of formula source outside the subset.
        warn: [u8; 4],
    },
    /// A shape label centred on a world point.
    Label {
        /// The text.
        text: &'a str,
        /// Centre, world units.
        center: Vec2,
        /// Font size in world units (screen size over zoom).
        world_size: f32,
        /// Ink colour bytes.
        color: [u8; 4],
    },
}

/// Glyph quads of one atlas page, in draw order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Run {
    /// Atlas page the quads sample.
    pub page: u32,
    /// Vertices, world units.
    pub vertices: Vec<Vertex>,
    /// Triangle indices into `vertices`.
    pub indices: Vec<u32>,
}

/// What decides whether a chunk is still valid.
#[derive(Debug, Clone, PartialEq)]
struct Key {
    revision: u64,
    epoch: u64,
    steps: Vec<(u32, u16)>,
    extra: u32,
}

/// The quads of one text block or label.
#[derive(Debug, Clone)]
pub struct TextChunk {
    key: Key,
    runs: Vec<Run>,
    rules: Mesh,
}

impl TextChunk {
    /// Glyph quads by page, in draw order.
    #[must_use]
    pub fn runs(&self) -> &[Run] {
        &self.runs
    }

    /// Untextured rules (fraction bars, root signs), world units.
    #[must_use]
    pub fn rules(&self) -> &Mesh {
        &self.rules
    }

    /// World bounds of everything, `None` when empty.
    #[must_use]
    pub fn bounds(&self) -> Option<Aabb> {
        let _ = &self.key;
        todo!()
    }
}

/// The text chunks of a document.
#[derive(Debug, Clone)]
pub struct TextCache {
    chunks: HashMap<ShapeId, TextChunk>,
    emissions: usize,
    epoch: u64,
    dpi: f32,
}

impl TextCache {
    /// An empty cache for a display of `dpi` physical pixels per logical one.
    #[must_use]
    pub fn new(dpi: f32) -> Self {
        Self {
            chunks: HashMap::new(),
            emissions: 0,
            epoch: 0,
            dpi,
        }
    }

    /// Sets the atlas epoch; a new one drops every chunk's validity.
    pub fn set_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
    }

    /// Brings the chunk of `id` in line with `spec` seen at `zoom`; `true`
    /// when its quads were emitted again.
    pub fn update(
        &mut self,
        id: ShapeId,
        revision: u64,
        spec: &TextSpec<'_>,
        zoom: f32,
        source: &dyn GlyphSource,
    ) -> bool {
        let _ = (id, revision, spec, zoom, source);
        todo!()
    }

    /// The chunk of `id`.
    #[must_use]
    pub fn get(&self, id: ShapeId) -> Option<&TextChunk> {
        self.chunks.get(&id)
    }

    /// Drops the chunk of `id`.
    pub fn remove(&mut self, id: ShapeId) {
        self.chunks.remove(&id);
    }

    /// Chunks emitted since the cache was created.
    #[must_use]
    pub fn emissions(&self) -> usize {
        self.emissions
    }
}

/// On-screen size in pixels of text of world `size` at `zoom`; `None` below
/// [`TEXT_MIN_PX`] or when not finite.
#[must_use]
pub fn text_screen_px(size: f32, zoom: f32) -> Option<f32> {
    let _ = (size, zoom);
    todo!()
}

/// Raster size and scale for text of `px` pixels.
#[must_use]
pub fn text_raster(px: f32) -> (u16, f32) {
    let _ = px;
    todo!()
}

/// Raster size and scale for a label of `size` pixels.
#[must_use]
pub fn label_raster(size: f32) -> (u16, f32) {
    let _ = size;
    todo!()
}

/// Physical em size of a raster at `dpi`, in `1..=MAX_RASTER_PX`.
#[must_use]
pub fn raster_px(raster: u16, dpi: f32) -> u16 {
    let _ = (raster, dpi);
    todo!()
}
