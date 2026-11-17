//! The retained document path of the app (T172): the shapes of the document
//! as cached chunks of world-space triangles, plus world-space glyph quads and
//! math rules cached by raster step (T180), drawn under the camera uniform so
//! a pan or a zoom inside a bucket tessellates nothing.

use std::collections::{HashMap, HashSet};

use draw_gfx::chunks::{CHUNK_LEN, ChunkCache, Item};
use draw_gfx::gpu::{Camera as GfxCamera, Gpu};
use draw_gfx::text::{TextCache, TextSpec, for_each_glyph};

use crate::core::camera::Camera;
use crate::core::document::ShapeId;
use crate::core::geom::{Aabb, Vec2};
use crate::core::palette::{ColorId, Rgba, palette};
use crate::core::shape::{Shape, grid_axis_labels};
use crate::core::text_layout::bounds;
use crate::shell::render::{
    atlas_epoch, axis_label_size, cull_rect, label_area, label_size, shown_label, warn_color_rgba,
    with_atlas,
};
use crate::shell::render_ctx::{RenderCtx, WorldDraw};
use crate::theme;

#[derive(Debug)]
struct BufferedChunk<B> {
    chunk_rev: u64,
    buffer: B,
    count: u32,
}

/// The chunks of a document, their GPU buffers, its text caches and the
/// revisions that key them.
#[derive(Debug)]
pub struct RetainedScene<B> {
    cache: ChunkCache,
    /// One GPU buffer per chunk slot, rebuilt only when `chunk.revision()` changes.
    buffers: Vec<Option<BufferedChunk<B>>>,
    /// Each shape as last seen and its revision.
    seen: HashMap<ShapeId, (u64, Shape)>,
    next_revision: u64,
    /// The document revision and hidden shapes the chunks were built for.
    built_for: Option<(u64, Vec<ShapeId>)>,
    /// Cached text and label quads keyed by synthetic ids, with the dpi built for.
    text: Option<(f32, TextCache)>,
    /// Text-bearing visible shape ids, grouped by chunk slot.
    chunk_text_owners: Vec<Vec<ShapeId>>,
    /// Synthetic `TextCache` ids owned by each visible text-bearing shape.
    text_slots: HashMap<ShapeId, Vec<ShapeId>>,
    next_text_id: u64,
    /// Last `(scene_seq, zoom_bits, dpi_bits, atlas_epoch)` synced into `text`.
    text_state: Option<(u64, u32, u32, u64)>,
    /// Incremented whenever the visible shapes or hidden list changed.
    scene_seq: u64,
}

impl<B> Default for RetainedScene<B> {
    fn default() -> Self {
        Self {
            cache: ChunkCache::default(),
            buffers: Vec::new(),
            seen: HashMap::new(),
            next_revision: 0,
            built_for: None,
            text: None,
            chunk_text_owners: Vec::new(),
            text_slots: HashMap::new(),
            next_text_id: 0,
            text_state: None,
            scene_seq: 0,
        }
    }
}

impl<B> RetainedScene<B> {
    /// An empty scene.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Shapes tessellated since the scene was made.
    #[must_use]
    pub fn tessellations(&self) -> usize {
        self.cache.tessellations()
    }

    /// Whether the chunks were built for this document `revision` and
    /// `hidden` list, so [`RetainedScene::draw_cached`] needs no scan.
    #[must_use]
    pub fn is_current(&self, revision: u64, hidden: &[ShapeId]) -> bool {
        self.built_for
            .as_ref()
            .is_some_and(|(r, h)| *r == revision && h == hidden)
    }

    /// Draws the chunks as last built, through `camera`.
    pub fn draw_cached<G: Gpu<Buffer = B>>(
        &mut self,
        ctx: &mut RenderCtx<G>,
        camera: &Camera,
        viewport: Vec2,
    ) {
        if self.cache.set_zoom(camera.zoom()) {
            // The bucket changed: the caller must rebuild.
            self.built_for = None;
        }
        self.emit(ctx, camera, viewport);
    }

    /// [`RetainedScene::draw`], remembering the `revision` and `hidden`
    /// the chunks are built for.
    pub fn draw_revision<G: Gpu<Buffer = B>>(
        &mut self,
        ctx: &mut RenderCtx<G>,
        revision: u64,
        shapes: &[(ShapeId, &Shape)],
        hidden: &[ShapeId],
        camera: &Camera,
        viewport: Vec2,
    ) {
        self.draw(ctx, shapes, hidden, camera, viewport);
        self.built_for = Some((revision, hidden.to_vec()));
    }

    /// Draws `shapes` (bottom to top) except the `hidden` ones, through
    /// `camera` in a viewport of `viewport` logical pixels: tessellates the
    /// chunks whose shapes changed, then draws the chunks in view.
    pub fn draw<G: Gpu<Buffer = B>>(
        &mut self,
        ctx: &mut RenderCtx<G>,
        shapes: &[(ShapeId, &Shape)],
        hidden: &[ShapeId],
        camera: &Camera,
        viewport: Vec2,
    ) {
        let visible: Vec<(ShapeId, &Shape)> = shapes
            .iter()
            .filter(|(id, _)| !hidden.contains(id))
            .copied()
            .collect();
        let items: Vec<Item<'_>> = visible
            .iter()
            .map(|&(id, shape)| Item {
                id,
                revision: self.revision_of(id, shape),
                shape,
            })
            .collect();
        if self.seen.len() > 2 * items.len() + 64 {
            let live: std::collections::HashSet<ShapeId> = items.iter().map(|i| i.id).collect();
            self.seen.retain(|id, _| live.contains(id));
        }
        self.cache.set_zoom(camera.zoom());
        self.cache.update(&items);
        self.rebuild_text_owners(&items);
        self.emit(ctx, camera, viewport);
    }

    /// Cached text-emission count since the scene was made (for tests).
    #[must_use]
    pub fn text_emissions(&self) -> usize {
        self.text.as_ref().map_or(0, |(_, cache)| cache.emissions())
    }

    fn rebuild_text_owners(&mut self, items: &[Item<'_>]) {
        self.chunk_text_owners.clear();
        let mut live_owners = HashSet::new();
        for group in items.chunks(CHUNK_LEN) {
            let mut owners = Vec::new();
            for item in group {
                if shape_has_text(item.shape) {
                    owners.push(item.id);
                    live_owners.insert(item.id);
                }
            }
            self.chunk_text_owners.push(owners);
        }
        let text_cache = self.text.as_mut().map(|(_, cache)| cache);
        let mut removed = Vec::new();
        self.text_slots.retain(|doc_id, slots| {
            let keep = live_owners.contains(doc_id);
            if !keep {
                removed.append(slots);
            }
            keep
        });
        if let Some(cache) = text_cache {
            for syn_id in removed {
                cache.remove(syn_id);
            }
        }
        self.scene_seq = self.scene_seq.wrapping_add(1);
    }

    fn sync_text<G: Gpu>(&mut self, ctx: &mut RenderCtx<G>, camera: &Camera) {
        let Some(epoch) = atlas_epoch() else {
            return;
        };
        let (zoom, dpi) = (camera.zoom(), ctx.dpi());
        let state = (self.scene_seq, zoom.to_bits(), dpi.to_bits(), epoch);
        if self.text_state == Some(state) {
            return;
        }
        if self.chunk_text_owners.iter().all(Vec::is_empty) {
            self.text_state = Some(state);
            return;
        }
        let recreate = self
            .text
            .as_ref()
            .is_none_or(|(old_dpi, _)| old_dpi.to_bits() != dpi.to_bits());
        if recreate {
            self.text = Some((dpi, TextCache::new(dpi)));
            self.text_slots.clear();
        }
        let Some((_, text_cache)) = self.text.as_mut() else {
            return;
        };
        text_cache.set_epoch(epoch);

        let mut updates: Vec<(ShapeId, u64, OwnedTextSpec<'_>)> = Vec::new();
        for owners in &self.chunk_text_owners {
            for doc_id in owners {
                let Some((shape_rev, shape)) = self.seen.get(doc_id) else {
                    continue;
                };
                let specs = shape_text_specs(shape, zoom);
                let slots = self.text_slots.entry(*doc_id).or_default();
                while slots.len() > specs.len() {
                    if let Some(dropped) = slots.pop() {
                        text_cache.remove(dropped);
                    }
                }
                while slots.len() < specs.len() {
                    self.next_text_id += 1;
                    slots.push(ShapeId(self.next_text_id));
                }
                for (syn_id, spec) in slots.iter().copied().zip(specs) {
                    updates.push((syn_id, *shape_rev, spec));
                }
            }
        }

        with_atlas(
            ctx,
            |atlas| {
                for (_, _, spec) in &updates {
                    for_each_glyph(&spec.as_spec(), zoom, dpi, |face, ch, rpx| {
                        atlas.prepare_in(face, ch, rpx);
                    });
                }
            },
            |source, _| {
                for (syn_id, shape_rev, spec) in &updates {
                    text_cache.update(*syn_id, *shape_rev, &spec.as_spec(), zoom, source);
                }
            },
        );
        self.text_state = Some(state);
    }

    fn sync_buffers<G: Gpu<Buffer = B>>(&mut self, ctx: &mut RenderCtx<G>) {
        let chunks = self.cache.chunks();
        self.buffers.truncate(chunks.len());
        self.buffers.resize_with(chunks.len(), || None);
        for (slot, chunk) in chunks.iter().enumerate() {
            let mesh = chunk.mesh();
            if mesh.indices.is_empty() {
                self.buffers[slot] = None;
                continue;
            }
            let rev = chunk.revision();
            if self.buffers[slot]
                .as_ref()
                .is_some_and(|cached| cached.chunk_rev == rev)
            {
                continue;
            }
            if let Ok(buffer) = ctx.gpu_mut().create_buffer(&mesh.vertices, &mesh.indices) {
                let count = u32::try_from(mesh.indices.len()).unwrap_or(u32::MAX);
                self.buffers[slot] = Some(BufferedChunk {
                    chunk_rev: rev,
                    buffer,
                    count,
                });
            } else {
                self.buffers[slot] = None;
            }
        }
    }

    fn emit<G: Gpu<Buffer = B>>(
        &mut self,
        ctx: &mut RenderCtx<G>,
        camera: &Camera,
        viewport: Vec2,
    ) {
        self.sync_text(ctx, camera);
        self.sync_buffers(ctx);
        let view = cull_rect(camera, viewport);
        let mut draws: Vec<WorldDraw<'_, B>> = Vec::new();
        let text_cache = self.text.as_ref().map(|(_, cache)| cache);
        for (slot, chunk) in self.cache.chunks().iter().enumerate() {
            if chunk.bounds().is_some_and(|b| b.intersects(&view))
                && !chunk.mesh().indices.is_empty()
            {
                match self.buffers.get(slot).and_then(Option::as_ref) {
                    Some(cached) => draws.push(WorldDraw::FeatheredBuffer {
                        buffer: &cached.buffer,
                        count: cached.count,
                    }),
                    None => draws.push(WorldDraw::Feathered(chunk.mesh())),
                }
            }
            if let (Some(text_cache), Some(owners)) = (text_cache, self.chunk_text_owners.get(slot))
            {
                for doc_id in owners {
                    let Some(slots) = self.text_slots.get(doc_id) else {
                        continue;
                    };
                    for syn_id in slots {
                        let Some(tc) = text_cache.get(*syn_id) else {
                            continue;
                        };
                        if !tc.bounds().is_some_and(|b| b.intersects(&view)) {
                            continue;
                        }
                        if !tc.rules().indices.is_empty() {
                            draws.push(WorldDraw::Alpha(tc.rules()));
                        }
                        for run in tc.runs() {
                            if !run.indices.is_empty() {
                                draws.push(WorldDraw::Glyphs {
                                    page: run.page,
                                    vertices: &run.vertices,
                                    indices: &run.indices,
                                });
                            }
                        }
                    }
                }
            }
        }
        let gfx = GfxCamera {
            origin: [camera.offset().x, camera.offset().y],
            scale: camera.zoom(),
        };
        ctx.draw_retained(gfx, &draws);
    }

    /// The revision of `shape`: a new one when it differs from the last
    /// time `id` was seen.
    fn revision_of(&mut self, id: ShapeId, shape: &Shape) -> u64 {
        match self.seen.get(&id) {
            Some((revision, old)) if old == shape => *revision,
            _ => {
                self.next_revision += 1;
                self.seen.insert(id, (self.next_revision, shape.clone()));
                self.next_revision
            }
        }
    }
}

enum OwnedTextSpec<'a> {
    Block {
        text: &'a str,
        origin: Vec2,
        size: f32,
        color: [u8; 4],
        warn: [u8; 4],
    },
    Label {
        text: String,
        center: Vec2,
        world_size: f32,
        color: [u8; 4],
    },
}

impl OwnedTextSpec<'_> {
    fn as_spec(&self) -> TextSpec<'_> {
        match self {
            Self::Block {
                text,
                origin,
                size,
                color,
                warn,
            } => TextSpec::Block {
                text,
                origin: *origin,
                size: *size,
                color: *color,
                warn: *warn,
            },
            Self::Label {
                text,
                center,
                world_size,
                color,
            } => TextSpec::Label {
                text,
                center: *center,
                world_size: *world_size,
                color: *color,
            },
        }
    }
}

fn shape_has_text(shape: &Shape) -> bool {
    match shape {
        Shape::Text { .. } => true,
        Shape::Rect { label, text, .. } | Shape::Ellipse { label, text, .. } => {
            label.is_some() || text.is_some()
        }
        Shape::Grid { axes, texts, .. } => *axes || !texts.is_empty(),
        Shape::Stroke { .. } | Shape::Line { .. } | Shape::Arrow { .. } => false,
    }
}

fn palette_bytes(id: ColorId) -> [u8; 4] {
    let c = palette(id);
    [c.r, c.g, c.b, c.a]
}

fn label_color_bytes(fill: Option<ColorId>, outline: ColorId) -> [u8; 4] {
    if fill.is_some() {
        let c = Rgba::from_hex(theme::TEXT_ON_FILL);
        [c.r, c.g, c.b, c.a]
    } else {
        palette_bytes(outline)
    }
}

fn warn_bytes(color: [u8; 4]) -> [u8; 4] {
    warn_color_rgba(color)
}

fn push_label_spec(
    out: &mut Vec<OwnedTextSpec<'_>>,
    shape: &Shape,
    fill: Option<ColorId>,
    outline: ColorId,
    ellipse: bool,
    zoom: f32,
) {
    let Some(label) = shown_label(shape) else {
        return;
    };
    let text = label.to_string();
    let world = shape.bounds();
    let screen_rect = Aabb::from_corners(world.min * zoom, world.max * zoom);
    let Some(px) = label_size(text.len(), label_area(screen_rect, ellipse)) else {
        return;
    };
    out.push(OwnedTextSpec::Label {
        text,
        center: world.center(),
        world_size: px / zoom,
        color: label_color_bytes(fill, outline),
    });
}

fn push_placed_specs<'a>(
    out: &mut Vec<OwnedTextSpec<'a>>,
    shape: &'a Shape,
    mut color_at: impl FnMut(Vec2) -> [u8; 4],
) {
    for placed in shape.placed_texts() {
        let center = bounds(placed.origin, placed.text, placed.size).center();
        let color = color_at(center);
        out.push(OwnedTextSpec::Block {
            text: placed.text,
            origin: placed.origin,
            size: placed.size,
            color,
            warn: warn_bytes(color),
        });
    }
}

fn shape_text_specs(shape: &Shape, zoom: f32) -> Vec<OwnedTextSpec<'_>> {
    let mut out = Vec::new();
    if !shape.is_finite() || !zoom.is_finite() || zoom <= 0.0 {
        return out;
    }
    match shape {
        Shape::Rect { style, fill, .. } => {
            push_label_spec(&mut out, shape, *fill, style.color, false, zoom);
            let color = label_color_bytes(*fill, style.color);
            push_placed_specs(&mut out, shape, |_| color);
        }
        Shape::Ellipse { style, fill, .. } => {
            push_label_spec(&mut out, shape, *fill, style.color, true, zoom);
            let color = label_color_bytes(*fill, style.color);
            push_placed_specs(&mut out, shape, |_| color);
        }
        Shape::Grid {
            a,
            b,
            cols,
            rows,
            style,
            axes,
            ..
        } => {
            let color = palette_bytes(style.color);
            if *axes && let Some(first) = grid_axis_labels(*a, *b, *cols, *rows).next() {
                let cell = Vec2::new(first.rect.width() * zoom, first.rect.height() * zoom);
                if let Some(px) = axis_label_size(*cols, *rows, cell) {
                    let world_size = px / zoom;
                    for label in grid_axis_labels(*a, *b, *cols, *rows) {
                        out.push(OwnedTextSpec::Label {
                            text: label.index.to_string(),
                            center: label.rect.center(),
                            world_size,
                            color,
                        });
                    }
                }
            }
            push_placed_specs(&mut out, shape, |center| {
                let filled = shape
                    .grid_cell_at(center)
                    .and_then(|(col, row)| shape.cell_fill(col, row));
                label_color_bytes(filled, style.color)
            });
        }
        Shape::Text {
            origin,
            text,
            size,
            style,
        } => {
            let color = palette_bytes(style.color);
            out.push(OwnedTextSpec::Block {
                text,
                origin: *origin,
                size: *size,
                color,
                warn: warn_bytes(color),
            });
        }
        Shape::Stroke { .. } | Shape::Line { .. } | Shape::Arrow { .. } => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::document::ShapeId;
    use crate::core::geom::Vec2;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use crate::shell::recording::{GpuCall, RecordingGpu};
    use crate::shell::render_ctx::RenderCtx;
    use draw_gfx::gpu::{Blend, Camera as GfxCamera, Command, CommandOf, View};

    const VIEW: View = View {
        width: 800.0,
        height: 600.0,
    };
    const PORT: Vec2 = Vec2 { x: 800.0, y: 600.0 };

    fn ctx() -> RenderCtx<RecordingGpu> {
        let mut ctx = RenderCtx::new(RecordingGpu::new());
        ctx.set_window(VIEW, 1.0);
        ctx
    }

    fn line(x: f32) -> Shape {
        Shape::Stroke {
            points: vec![Vec2::new(x, 10.0), Vec2::new(x + 50.0, 40.0)],
            widths: None,
            style: Style {
                color: ColorId::INK,
                width: 2.0,
            },
        }
    }

    fn items(shapes: &[Shape]) -> Vec<(ShapeId, &Shape)> {
        shapes
            .iter()
            .enumerate()
            .map(|(i, s)| (ShapeId(i as u64 + 1), s))
            .collect()
    }

    fn commands(ctx: &mut RenderCtx<RecordingGpu>) -> Vec<CommandOf<RecordingGpu>> {
        ctx.gpu_mut()
            .take_calls()
            .into_iter()
            .filter_map(|call| match call {
                GpuCall::Submit { commands } => Some(commands),
                _ => None,
            })
            .flatten()
            .collect()
    }

    #[test]
    fn retained_draw_sets_camera_and_feathered_pipeline() {
        // Arrange
        let mut ctx = ctx();
        let mut scene = RetainedScene::new();
        let shapes = [line(0.0)];
        let camera = Camera::new(Vec2::new(5.0, 7.0), 2.0);

        // Act
        scene.draw(&mut ctx, &items(&shapes), &[], &camera, PORT);

        // Assert
        let calls = ctx.gpu_mut().take_calls();
        assert!(calls.iter().any(|c| matches!(
            c,
            GpuCall::Pipeline { desc, .. } if desc.blend == Blend::AlphaFeathered
        )));
        let list: Vec<_> = calls
            .into_iter()
            .filter_map(|c| match c {
                GpuCall::Submit { commands } => Some(commands),
                _ => None,
            })
            .flatten()
            .collect();
        assert!(list.contains(&Command::SetCamera(GfxCamera {
            origin: [5.0, 7.0],
            scale: 2.0
        })));
        assert_eq!(
            list.iter()
                .filter(|c| matches!(c, Command::Draw { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn retained_pan_tessellates_nothing() {
        // Arrange
        let mut ctx = ctx();
        let mut scene = RetainedScene::new();
        let shapes = [line(0.0), line(100.0)];
        scene.draw(&mut ctx, &items(&shapes), &[], &Camera::default(), PORT);
        let before = scene.tessellations();

        // Act
        scene.draw(
            &mut ctx,
            &items(&shapes),
            &[],
            &Camera::new(Vec2::new(30.0, 20.0), 1.0),
            PORT,
        );

        // Assert
        assert_eq!(scene.tessellations(), before);
    }

    #[test]
    fn retained_zoom_in_bucket_tessellates_nothing() {
        // Arrange
        let mut ctx = ctx();
        let mut scene = RetainedScene::new();
        let shapes = [line(0.0)];
        scene.draw(
            &mut ctx,
            &items(&shapes),
            &[],
            &Camera::new(Vec2::ZERO, 1.0),
            PORT,
        );
        let before = scene.tessellations();

        // Act
        scene.draw(
            &mut ctx,
            &items(&shapes),
            &[],
            &Camera::new(Vec2::ZERO, 1.01),
            PORT,
        );

        // Assert
        assert_eq!(scene.tessellations(), before);
    }

    #[test]
    fn retained_edit_rebuilds_only_the_changed_chunk() {
        // Arrange
        let mut ctx = ctx();
        let mut scene = RetainedScene::new();
        let mut shapes: Vec<Shape> = (0..130).map(|i| line(i as f32)).collect();
        scene.draw(&mut ctx, &items(&shapes), &[], &Camera::default(), PORT);
        let before = scene.tessellations();
        shapes[129] = line(500.0);

        // Act
        scene.draw(&mut ctx, &items(&shapes), &[], &Camera::default(), PORT);

        // Assert: only the last chunk (2 shapes) is tessellated again.
        assert_eq!(scene.tessellations(), before + 2);
    }

    #[test]
    fn retained_hidden_shape_is_not_drawn() {
        // Arrange
        let mut ctx = ctx();
        let mut scene = RetainedScene::new();
        let shapes = [line(0.0)];
        let hidden = [ShapeId(1)];

        // Act
        scene.draw(&mut ctx, &items(&shapes), &hidden, &Camera::default(), PORT);

        // Assert
        let drawn = commands(&mut ctx)
            .iter()
            .filter(|c| matches!(c, Command::Draw { .. }))
            .count();
        assert_eq!(drawn, 0);
    }

    fn text_ctx() -> RenderCtx<RecordingGpu> {
        let mut ctx = ctx();
        let atlas = crate::shell::glyphs::GlyphAtlas::new(crate::shell::app::TEXT_FONT)
            .and_then(|a| a.with_math(crate::shell::app::MATH_FONT))
            .unwrap_or_else(|err| panic!("embedded fonts load: {err}"));
        ctx.set_glyph_atlas(atlas);
        ctx
    }

    #[test]
    fn retained_text_formula_label_and_grid_draw_and_reuse_cache() {
        use crate::core::shape::{CellText, ShapeText};

        // Arrange
        let mut ctx = text_ctx();
        let mut scene = RetainedScene::new();
        let ink = Style {
            color: ColorId::INK,
            width: 2.0,
        };
        let shapes = [
            Shape::Text {
                origin: Vec2::new(10.0, 20.0),
                text: "$\\frac{x}{2}$".to_owned(),
                size: 24.0,
                style: ink,
            },
            Shape::Rect {
                a: Vec2::new(100.0, 20.0),
                b: Vec2::new(200.0, 100.0),
                style: ink,
                fill: None,
                label: Some(7),
                text: None,
            },
            Shape::Grid {
                a: Vec2::new(220.0, 20.0),
                b: Vec2::new(340.0, 140.0),
                cols: 2,
                rows: 2,
                style: ink,
                axes: true,
                fills: Vec::new(),
                texts: vec![CellText {
                    col: 0,
                    row: 0,
                    content: ShapeText {
                        text: "ok".to_owned(),
                        size: 16.0,
                    },
                }],
            },
        ];

        // Act 1: initial draw emits shapes, rules, labels and glyphs.
        ctx.gpu_mut().take_calls();
        scene.draw_revision(&mut ctx, 1, &items(&shapes), &[], &Camera::default(), PORT);
        let first_emissions = scene.text_emissions();
        let cmds = commands(&mut ctx);
        let has_textured = cmds.iter().any(|c| {
            matches!(
                c,
                Command::Draw {
                    texture: Some(_),
                    ..
                }
            )
        });
        let untextured_draws = cmds
            .iter()
            .filter(|c| matches!(c, Command::Draw { texture: None, .. }))
            .count();

        // Assert 1: 1 chunk of shapes + 1 untextured fraction rule, plus glyph draws.
        assert!(first_emissions >= 7, "emissions {first_emissions}");
        assert!(has_textured);
        assert_eq!(untextured_draws, 2);

        // Act 2: pan on cached revision emits zero new text chunks and creates
        // zero new GPU buffers.
        let _ = ctx.gpu_mut().take_calls();
        scene.draw_cached(&mut ctx, &Camera::new(Vec2::new(15.0, 5.0), 1.0), PORT);
        assert_eq!(scene.text_emissions(), first_emissions);
        assert!(
            !ctx.gpu_mut()
                .take_calls()
                .iter()
                .any(|c| matches!(c, GpuCall::CreateBuffer { .. }))
        );
    }
}
