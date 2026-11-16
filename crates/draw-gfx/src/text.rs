//! Retained text, labels and formulas: glyph quads in world units, one
//! chunk per block, re-emitted only when the raster step changes
//! (ADR-T180-1).

use std::collections::HashMap;

use draw_core::core::document::ShapeId;
use draw_core::core::geom::{Aabb, Vec2};
use draw_core::core::tex::{Face, Ink};
use draw_core::core::text_layout::{glyphs, has_markup, typeset};

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
        let points: Vec<Vec2> = self
            .runs
            .iter()
            .flat_map(|r| r.vertices.iter())
            .chain(self.rules.vertices.iter())
            .map(|v| Vec2::new(v.pos[0], v.pos[1]))
            .collect();
        Aabb::from_points(&points)
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
        source: &impl GlyphSource,
    ) -> bool {
        let key = Key {
            revision,
            epoch: self.epoch,
            ..plan(spec, zoom, self.dpi)
        };
        if self.chunks.get(&id).is_some_and(|c| c.key == key) {
            return false;
        }
        let (runs, rules) = emit(spec, zoom, self.dpi, source);
        self.chunks.insert(id, TextChunk { key, runs, rules });
        self.emissions += 1;
        true
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
    let px = size * zoom;
    (px.is_finite() && px >= TEXT_MIN_PX).then_some(px)
}

/// The smallest of `ladder` at least `px` (else the largest) and the scale
/// mapping it to `px`.
fn ladder_raster(ladder: &[u16], px: f32) -> (u16, f32) {
    let largest = ladder.last().copied().unwrap_or(1);
    let raster = ladder
        .iter()
        .copied()
        .find(|&r| f32::from(r) >= px)
        .unwrap_or(largest);
    (raster, px / f32::from(raster))
}

/// Raster size and scale for text of `px` pixels.
#[must_use]
pub fn text_raster(px: f32) -> (u16, f32) {
    ladder_raster(&TEXT_RASTER_SIZES, px)
}

/// Raster size and scale for a label of `size` pixels.
#[must_use]
pub fn label_raster(size: f32) -> (u16, f32) {
    ladder_raster(&LABEL_RASTER_SIZES, size)
}

/// Physical em size of a raster at `dpi`, in `1..=MAX_RASTER_PX`.
#[must_use]
pub fn raster_px(raster: u16, dpi: f32) -> u16 {
    let dpi = if dpi.is_finite() && dpi > 0.0 {
        dpi
    } else {
        1.0
    };
    let px = (f32::from(raster) * dpi).ceil();
    // Clamped into the u16 range first, so the cast is exact.
    px.clamp(1.0, f32::from(MAX_RASTER_PX)) as u16
}

/// Whether `ch` has ink to draw.
fn drawable(ch: char) -> bool {
    !(ch.is_whitespace() || ch.is_control())
}

/// Physical raster px of text of world `size` at `zoom`; `None` too small.
fn block_rpx(size: f32, zoom: f32, dpi: f32) -> Option<u16> {
    let px = text_screen_px(size, zoom)?;
    Some(raster_px(text_raster(px).0, dpi))
}

/// Physical raster px of a label of world `size` at `zoom`.
fn label_rpx(world_size: f32, zoom: f32, dpi: f32) -> Option<u16> {
    let px = world_size * zoom;
    (px.is_finite() && px > 0.0).then(|| raster_px(label_raster(px).0, dpi))
}

/// Drops the low mantissa bits so float noise does not change a key.
fn quantize(size: f32) -> f32 {
    f32::from_bits(size.to_bits() & !0x3FF)
}

/// The text of a block as `(ch, face, world pen, scale, warn)` glyphs and
/// `(a, b, width)` rules in world units.
type Laid = (Vec<(char, Face, Vec2, f32, bool)>, Vec<(Vec2, Vec2, f32)>);

fn lay_out(text: &str, origin: Vec2, size: f32) -> Laid {
    let mut glyph_list = Vec::new();
    let mut rules = Vec::new();
    if has_markup(text) {
        let at = |p: Vec2| origin + p * size;
        for ink in &typeset(text).ink {
            match *ink {
                Ink::Glyph {
                    ch,
                    face,
                    pos,
                    scale,
                    warn,
                } => glyph_list.push((ch, face, at(pos), scale, warn)),
                Ink::Rule { a, b, width } => rules.push((at(a), at(b), width * size)),
            }
        }
    } else {
        glyph_list
            .extend(glyphs(origin, text, size).map(|g| (g.ch, Face::Text, g.pos, 1.0, false)));
    }
    (glyph_list, rules)
}

/// The key parts that depend on the spec and the zoom: raster steps (and
/// whether rules are clamped to the minimum width, which depends on zoom).
fn plan(spec: &TextSpec<'_>, zoom: f32, dpi: f32) -> Key {
    let mut steps: Vec<(u32, u16)> = Vec::new();
    let mut extra = 0;
    match *spec {
        TextSpec::Block {
            text, origin, size, ..
        } => {
            let (list, rules) = lay_out(text, origin, size);
            for &(_, _, _, scale, _) in &list {
                let bits = scale.to_bits();
                if steps.iter().all(|s| s.0 != bits) {
                    steps.push((bits, block_rpx(size * scale, zoom, dpi).unwrap_or(0)));
                }
            }
            let visible = text_screen_px(size, zoom).is_some();
            if visible
                && rules
                    .iter()
                    .any(|r| r.2 * zoom < MIN_RULE_PX || !(r.2 * zoom).is_finite())
            {
                extra = zoom.to_bits();
            }
            // A change of the layout origin or size is a change of the key.
            steps.push((size.to_bits(), u16::from(visible)));
            steps.push((origin.x.to_bits(), 0));
            steps.push((origin.y.to_bits(), 0));
        }
        TextSpec::Label {
            text,
            center,
            world_size,
            ..
        } => {
            let q = quantize(world_size);
            steps.push((q.to_bits(), label_rpx(q, zoom, dpi).unwrap_or(0)));
            steps.push((center.x.to_bits(), 0));
            steps.push((center.y.to_bits(), 0));
            steps.push((text.len() as u32, 0));
        }
    }
    Key {
        revision: 0,
        epoch: 0,
        steps,
        extra,
    }
}

/// Appends the quad of `entry` with its pen at `pen`, `k` world units per
/// texel; nothing for an empty bitmap or a non-finite value.
fn push_quad(
    runs: &mut Vec<Run>,
    side: u16,
    entry: &GlyphEntry,
    pen: Vec2,
    k: f32,
    color: [u8; 4],
) {
    if entry.w == 0 || entry.h == 0 || !pen.is_finite() || !(k.is_finite() && k > 0.0) {
        return;
    }
    let (w, h) = (f32::from(entry.w), f32::from(entry.h));
    let min = Vec2::new(pen.x + entry.xmin * k, pen.y - (entry.ymin + h) * k);
    let max = Vec2::new(min.x + w * k, min.y + h * k);
    let side = f32::from(side.max(1));
    let (u0, v0) = (f32::from(entry.x) / side, f32::from(entry.y) / side);
    let (u1, v1) = (
        (f32::from(entry.x) + w) / side,
        (f32::from(entry.y) + h) / side,
    );
    if runs.last().is_none_or(|r| r.page != entry.page) {
        runs.push(Run {
            page: entry.page,
            ..Run::default()
        });
    }
    let Some(run) = runs.last_mut() else { return };
    let base = u32::try_from(run.vertices.len()).unwrap_or(u32::MAX);
    for (x, y, u, v) in [
        (min.x, min.y, u0, v0),
        (max.x, min.y, u1, v0),
        (max.x, max.y, u1, v1),
        (min.x, max.y, u0, v1),
    ] {
        run.vertices.push(Vertex {
            pos: [x, y],
            uv: [u, v],
            color,
        });
    }
    run.indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Appends the rule `a → b` of world `width` as one quad.
fn push_rule(mesh: &mut Mesh, a: Vec2, b: Vec2, width: f32, color: [u8; 4]) {
    let d = b - a;
    let len = d.length();
    if !len.is_finite() || len <= 0.0 || !width.is_finite() {
        return;
    }
    let n = Vec2::new(-d.y, d.x) * (width * 0.5 / len);
    let base = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
    for p in [a + n, b + n, b - n, a - n] {
        mesh.vertices.push(Vertex {
            pos: [p.x, p.y],
            uv: [0.0, 0.0],
            color,
        });
    }
    mesh.indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// The quads and rules of `spec` at `zoom`.
fn emit(spec: &TextSpec<'_>, zoom: f32, dpi: f32, source: &impl GlyphSource) -> (Vec<Run>, Mesh) {
    let side = source.page_side();
    let mut runs = Vec::new();
    let mut rules = Mesh::default();
    match *spec {
        TextSpec::Block {
            text,
            origin,
            size,
            color,
            warn,
        } => {
            let (list, rule_list) = lay_out(text, origin, size);
            for (ch, face, pen, scale, warned) in list {
                let Some(rpx) = block_rpx(size * scale, zoom, dpi) else {
                    continue;
                };
                let Some(entry) = source.glyph(face, ch, rpx).filter(|_| drawable(ch)) else {
                    continue;
                };
                let k = size * scale / f32::from(rpx);
                push_quad(
                    &mut runs,
                    side,
                    &entry,
                    pen,
                    k,
                    if warned { warn } else { color },
                );
            }
            if text_screen_px(size, zoom).is_some() {
                for (a, b, width) in rule_list {
                    let world = width.max(MIN_RULE_PX / zoom);
                    push_rule(&mut rules, a, b, world, color);
                }
            }
        }
        TextSpec::Label {
            text,
            center,
            world_size,
            color,
        } => {
            let em = quantize(world_size);
            let Some(rpx) = label_rpx(em, zoom, dpi) else {
                return (runs, rules);
            };
            let k = em / f32::from(rpx);
            let entries = || {
                text.chars()
                    .filter_map(|ch| source.glyph(Face::Text, ch, rpx))
            };
            let width: f32 = entries().map(|e| e.advance * k).sum();
            let ink = entries()
                .map(|e| (e.ymin + f32::from(e.h)) * k)
                .fold(0.0, f32::max);
            let mut pen = Vec2::new(center.x - width * 0.5, center.y + ink * 0.5);
            for entry in entries() {
                push_quad(&mut runs, side, &entry, pen, k, color);
                pen.x += entry.advance * k;
            }
        }
    }
    (runs, rules)
}
