//! The macroquad interpreter of the `Gpu` trait (ADR-T140-1, T157): buffers,
//! textures, targets and pipelines live in tables and `submit` draws through
//! `draw_mesh` and macroquad's render passes.
//!
//! The logic that needs no window is separate and tested: [`Table`] (handle
//! checks), [`chunks`] (geometry split to what macroquad's `u16` meshes
//! hold) and [`PassChecker`] (pass structure of a command list).

use std::collections::HashMap;

use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::Color;
use macroquad::math::Rect;
use macroquad::models::{Mesh, Vertex as MqVertex, draw_mesh};
use macroquad::texture::{
    FilterMode, Image, RenderTarget, RenderTargetParams, Texture2D, render_target_ex,
};
use macroquad::window::clear_background;

use draw_gfx::gpu::{
    Blend, Command, CommandList, Filter, Geometry, Gpu, GpuError, PipelineDesc, TargetDesc,
    TextureDesc, Vertex,
};

use super::render::{BATCH_MAX_INDICES, BATCH_MAX_VERTICES};

/// Most vertices of one macroquad mesh chunk.
pub const CHUNK_MAX_VERTICES: usize = BATCH_MAX_VERTICES;

/// Most indices of one macroquad mesh chunk (a multiple of 3).
pub const CHUNK_MAX_INDICES: usize = BATCH_MAX_INDICES;

/// One mesh chunk: local vertices and `u16` indices into them.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    /// The vertices the chunk uses, in first-use order.
    pub vertices: Vec<Vertex>,
    /// Triangle list indices into `vertices`.
    pub indices: Vec<u16>,
}

/// Splits triangles into chunks within [`CHUNK_MAX_VERTICES`] and
/// [`CHUNK_MAX_INDICES`], keeping triangles whole and in order.
///
/// # Errors
/// [`GpuError::Backend`] if `indices` is not a whole number of triangles or
/// names a vertex that does not exist.
pub fn chunks(vertices: &[Vertex], indices: &[u32]) -> Result<Vec<Chunk>, GpuError> {
    if !indices.len().is_multiple_of(3) {
        return Err(backend("indices are not whole triangles"));
    }
    let mut out = Vec::new();
    let mut chunk = Chunk {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    let mut local: HashMap<u32, u16> = HashMap::new();
    let (triangles, _) = indices.as_chunks::<3>();
    for triangle in triangles {
        if triangle.iter().any(|&i| i as usize >= vertices.len()) {
            return Err(backend("index out of range"));
        }
        let [a, b, c] = *triangle;
        let mut fresh = usize::from(!local.contains_key(&a));
        fresh += usize::from(b != a && !local.contains_key(&b));
        fresh += usize::from(c != a && c != b && !local.contains_key(&c));
        let full = chunk.vertices.len() + fresh > CHUNK_MAX_VERTICES
            || chunk.indices.len() + 3 > CHUNK_MAX_INDICES;
        if full && !chunk.indices.is_empty() {
            out.push(std::mem::replace(
                &mut chunk,
                Chunk {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                },
            ));
            local.clear();
        }
        for &i in triangle {
            let slot = if let Some(&slot) = local.get(&i) {
                slot
            } else {
                let slot =
                    u16::try_from(chunk.vertices.len()).map_err(|_| backend("chunk too large"))?;
                chunk.vertices.push(vertices[i as usize]);
                local.insert(i, slot);
                slot
            };
            chunk.indices.push(slot);
        }
    }
    if !chunk.indices.is_empty() {
        out.push(chunk);
    }
    Ok(out)
}

/// A [`GpuError::Backend`] with `text`.
fn backend(text: &str) -> GpuError {
    GpuError::Backend(text.to_owned())
}

/// Handles of one kind: ids to values.
#[derive(Debug)]
pub struct Table<T> {
    items: HashMap<u32, T>,
    next: u32,
}

impl<T> Table<T> {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            next: 0,
        }
    }

    /// Stores `value` and returns its id.
    pub fn insert(&mut self, value: T) -> u32 {
        let id = self.next;
        self.next = self.next.saturating_add(1);
        self.items.insert(id, value);
        id
    }

    /// The value of `id`.
    ///
    /// # Errors
    /// [`GpuError::UnknownHandle`] if `id` was never handed out.
    pub fn get(&self, id: u32) -> Result<&T, GpuError> {
        self.items.get(&id).ok_or(GpuError::UnknownHandle)
    }
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// One step of a command list, for [`PassChecker`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// `BeginPass`.
    Begin,
    /// `Draw`.
    Draw,
    /// `EndPass`.
    End,
}

/// Checks that draws sit inside one pass and passes do not nest.
#[derive(Debug, Default)]
pub struct PassChecker {
    open: bool,
}

impl PassChecker {
    /// Accepts the next `step` or says why not.
    ///
    /// # Errors
    /// [`GpuError::Backend`] for a draw or end outside a pass, or a begin
    /// inside one.
    pub fn step(&mut self, step: Step) -> Result<(), GpuError> {
        match (step, self.open) {
            (Step::Begin, false) => self.open = true,
            (Step::End, true) => self.open = false,
            (Step::Draw, true) => {}
            (Step::Begin, true) => return Err(backend("BeginPass inside a pass")),
            (Step::End, false) => return Err(backend("EndPass outside a pass")),
            (Step::Draw, false) => return Err(backend("Draw outside a pass")),
        }
        Ok(())
    }

    /// Checks that no pass is left open.
    ///
    /// # Errors
    /// [`GpuError::Backend`] if a pass is open.
    pub fn finish(&self) -> Result<(), GpuError> {
        if self.open {
            Err(backend("pass left open"))
        } else {
            Ok(())
        }
    }
}

/// A texture stored by the interpreter.
#[derive(Debug, Clone)]
pub struct MqTexture {
    id: u32,
}

/// A buffer stored by the interpreter.
#[derive(Debug, Clone)]
pub struct MqBuffer {
    id: u32,
}

/// A render target: its own id and the texture id of what it holds.
#[derive(Debug, Clone)]
pub struct MqTarget {
    id: u32,
    texture: u32,
}

/// A pipeline: an id into the interpreter's pipeline table.
#[derive(Debug, Clone, Copy)]
pub struct MqPipeline {
    id: u32,
}

/// The macroquad interpreter of [`Gpu`]. Needs a macroquad context.
#[derive(Debug, Default)]
pub struct MqGpu {
    buffers: Table<(Vec<Vertex>, Vec<u32>)>,
    textures: Table<Texture2D>,
    targets: Table<RenderTarget>,
    pipelines: Table<PipelineDesc>,
}

impl Gpu for MqGpu {
    type Buffer = MqBuffer;
    type Texture = MqTexture;
    type Target = MqTarget;
    type Pipeline = MqPipeline;

    fn create_buffer(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<MqBuffer, GpuError> {
        if !indices.len().is_multiple_of(3) || indices.iter().any(|&i| i as usize >= vertices.len())
        {
            return Err(backend("buffer indices do not fit its vertices"));
        }
        let id = self.buffers.insert((vertices.to_vec(), indices.to_vec()));
        Ok(MqBuffer { id })
    }

    fn create_texture(&mut self, desc: TextureDesc, rgba: &[u8]) -> Result<MqTexture, GpuError> {
        let expected = desc.byte_len().ok_or(GpuError::BadSize)?;
        if desc.width == 0 || desc.height == 0 {
            return Err(GpuError::BadSize);
        }
        if rgba.len() != expected {
            return Err(GpuError::BadPixelData {
                expected,
                got: rgba.len(),
            });
        }
        let width = u16::try_from(desc.width).map_err(|_| GpuError::BadSize)?;
        let height = u16::try_from(desc.height).map_err(|_| GpuError::BadSize)?;
        let texture = Texture2D::from_rgba8(width, height, rgba);
        texture.set_filter(filter_mode(desc.filter));
        Ok(MqTexture {
            id: self.textures.insert(texture),
        })
    }

    fn write_texture(
        &mut self,
        texture: &MqTexture,
        rect: [u32; 4],
        rgba: &[u8],
    ) -> Result<(), GpuError> {
        let target = self.textures.get(texture.id)?;
        let [x, y, width, height] = rect;
        let size = TextureDesc {
            width,
            height,
            filter: Filter::Nearest,
        };
        let expected = size.byte_len().ok_or(GpuError::BadSize)?;
        if rgba.len() != expected {
            return Err(GpuError::BadPixelData {
                expected,
                got: rgba.len(),
            });
        }
        let fits = |origin: u32, extent: u32, whole: f32| {
            origin
                .checked_add(extent)
                .is_some_and(|end| end as f32 <= whole)
        };
        if width == 0
            || height == 0
            || !fits(x, width, target.width())
            || !fits(y, height, target.height())
        {
            return Err(GpuError::BadSize);
        }
        let narrow = |value: u32| u16::try_from(value).map_err(|_| GpuError::BadSize);
        let image = Image {
            bytes: rgba.to_vec(),
            width: narrow(width)?,
            height: narrow(height)?,
        };
        let int = |value: u32| i32::try_from(value).map_err(|_| GpuError::BadSize);
        target.update_part(&image, int(x)?, int(y)?, int(width)?, int(height)?);
        Ok(())
    }

    fn create_target(&mut self, desc: TargetDesc) -> Result<MqTarget, GpuError> {
        if desc.width == 0 || desc.height == 0 || desc.samples == 0 {
            return Err(GpuError::BadSize);
        }
        let target = render_target_ex(
            desc.width,
            desc.height,
            RenderTargetParams {
                sample_count: i32::from(desc.samples),
                depth: false,
            },
        );
        target.texture.set_filter(filter_mode(desc.filter));
        let texture = self.textures.insert(target.texture.clone());
        let id = self.targets.insert(target);
        Ok(MqTarget { id, texture })
    }

    fn set_filter(&mut self, texture: &MqTexture, filter: Filter) -> Result<(), GpuError> {
        self.textures
            .get(texture.id)?
            .set_filter(filter_mode(filter));
        Ok(())
    }

    fn read_texture(&mut self, texture: &MqTexture) -> Result<Vec<u8>, GpuError> {
        Ok(self.textures.get(texture.id)?.get_texture_data().bytes)
    }

    fn target_texture(&self, target: &MqTarget) -> MqTexture {
        MqTexture { id: target.texture }
    }

    fn pipeline(&mut self, desc: PipelineDesc) -> MqPipeline {
        MqPipeline {
            id: self.pipelines.insert(desc),
        }
    }

    fn submit(&mut self, list: &CommandList<Self>) -> Result<(), GpuError> {
        self.validate(list)?;
        let mut in_target = false;
        let result = self.execute(list, &mut in_target);
        if in_target {
            // A failed pass must not leave the target as the camera.
            set_default_camera();
        }
        result
    }
}

impl MqGpu {
    /// An interpreter with no resource yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks pass structure, handles and ranges of the whole `list`, so a
    /// bad list draws nothing.
    fn validate(&self, list: &CommandList<Self>) -> Result<(), GpuError> {
        let mut passes = PassChecker::default();
        for command in list.commands() {
            match command {
                Command::BeginPass { target, .. } => {
                    passes.step(Step::Begin)?;
                    if let Some(target) = target {
                        self.targets.get(target.id)?;
                    }
                }
                Command::Draw {
                    pipeline,
                    texture,
                    geometry,
                } => {
                    passes.step(Step::Draw)?;
                    if self.pipelines.get(pipeline.id)?.blend != Blend::Alpha {
                        return Err(backend("opaque blend is not supported yet"));
                    }
                    if let Some(texture) = texture {
                        self.textures.get(texture.id)?;
                    }
                    if let Geometry::Buffer {
                        buffer,
                        first,
                        count,
                    } = geometry
                    {
                        let (_, indices) = self.buffers.get(buffer.id)?;
                        buffer_range(indices, *first, *count)?;
                    }
                }
                Command::EndPass => passes.step(Step::End)?,
            }
        }
        passes.finish()
    }

    /// Runs a validated `list`. `in_target` tells whether a target pass
    /// is open (it owns the camera, [`pass_owns_camera`]).
    fn execute(&self, list: &CommandList<Self>, in_target: &mut bool) -> Result<(), GpuError> {
        for command in list.commands() {
            match command {
                Command::BeginPass {
                    target,
                    view,
                    clear,
                } => {
                    *in_target = pass_owns_camera(target.is_some());
                    // A pass never inherits the caller's camera (ADR-T159-1):
                    // no target draws to the window in its pixels.
                    if target.is_none() {
                        set_default_camera();
                    }
                    if let Some(target) = target {
                        let render_target = self.targets.get(target.id)?;
                        let mut camera = Camera2D::from_display_rect(Rect::new(
                            0.0,
                            0.0,
                            view.width,
                            view.height,
                        ));
                        camera.render_target = Some(render_target.clone());
                        set_camera(&camera);
                    }
                    if let Some(color) = clear {
                        clear_background(Color::new(color.r, color.g, color.b, color.a));
                    }
                }
                Command::Draw {
                    texture, geometry, ..
                } => {
                    let texture = match texture {
                        Some(texture) => Some(self.textures.get(texture.id)?),
                        None => None,
                    };
                    match geometry {
                        Geometry::Inline { vertices, indices } => {
                            draw_chunks(vertices, indices, texture)?;
                        }
                        Geometry::Buffer {
                            buffer,
                            first,
                            count,
                        } => {
                            let (vertices, indices) = self.buffers.get(buffer.id)?;
                            let range = buffer_range(indices, *first, *count)?;
                            draw_chunks(vertices, range, texture)?;
                        }
                    }
                }
                Command::EndPass => {
                    if *in_target {
                        set_default_camera();
                        *in_target = false;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Whether a pass with (`true`) or without a render target sets the
/// macroquad camera itself: every pass does (ADR-T159-1).
#[must_use]
pub fn pass_owns_camera(_has_target: bool) -> bool {
    true
}

/// The `count` indices of `indices` from `first`.
fn buffer_range(indices: &[u32], first: u32, count: u32) -> Result<&[u32], GpuError> {
    let start = first as usize;
    let end = start
        .checked_add(count as usize)
        .ok_or_else(|| backend("index range overflows"))?;
    indices
        .get(start..end)
        .ok_or_else(|| backend("index range out of the buffer"))
}

/// Draws the triangles in chunks macroquad can hold.
fn draw_chunks(
    vertices: &[Vertex],
    indices: &[u32],
    texture: Option<&Texture2D>,
) -> Result<(), GpuError> {
    for chunk in chunks(vertices, indices)? {
        let mesh = Mesh {
            vertices: chunk.vertices.iter().map(to_mq_vertex).collect(),
            indices: chunk.indices,
            texture: texture.cloned(),
        };
        draw_mesh(&mesh);
    }
    Ok(())
}

/// The macroquad vertex with the same position, texture coordinate and
/// colour bytes.
fn to_mq_vertex(vertex: &Vertex) -> MqVertex {
    let mut out = MqVertex::new(
        vertex.pos[0],
        vertex.pos[1],
        0.0,
        vertex.uv[0],
        vertex.uv[1],
        // Replaced by the exact bytes below.
        Color::default(),
    );
    out.color = vertex.color;
    out
}

/// The macroquad sampling filter of `filter`.
fn filter_mode(filter: Filter) -> FilterMode {
    match filter {
        Filter::Nearest => FilterMode::Nearest,
        Filter::Linear => FilterMode::Linear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn ok<T>(result: Result<T, GpuError>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error}"),
        }
    }

    fn vertex(n: u32) -> Vertex {
        Vertex {
            pos: [n as f32, 0.0],
            uv: [0.0, 0.0],
            color: [1, 2, 3, 4],
        }
    }

    fn vertices(count: usize) -> Vec<Vertex> {
        (0..count as u32).map(vertex).collect()
    }

    // ---- AC-1 chunks ---------------------------------------------------

    #[test]
    fn chunks_of_small_geometry_is_one_chunk() {
        // Arrange
        let v = vertices(3);

        // Act
        let chunks = ok(chunks(&v, &[0, 1, 2]));

        // Assert
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].indices, vec![0, 1, 2]);
        assert_eq!(chunks[0].vertices.len(), 3);
    }

    #[test]
    fn chunks_of_empty_geometry_is_empty() {
        assert_eq!(ok(chunks(&[], &[])), Vec::new());
    }

    #[test]
    fn chunks_split_at_the_index_limit() {
        // Arrange: one shared triangle repeated past the index limit.
        let v = vertices(3);
        let triangles = CHUNK_MAX_INDICES / 3 + 1;
        let indices: Vec<u32> = (0..triangles).flat_map(|_| [0, 1, 2]).collect();

        // Act
        let chunks = ok(chunks(&v, &indices));

        // Assert
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].indices.len(), CHUNK_MAX_INDICES);
        assert_eq!(chunks[1].indices.len(), 3);
    }

    #[test]
    fn chunks_split_at_the_vertex_limit() {
        // Arrange: disjoint triangles, three new vertices each.
        let triangles = CHUNK_MAX_VERTICES / 3 + 1;
        let v = vertices(triangles * 3);
        let indices: Vec<u32> = (0..(triangles * 3) as u32).collect();

        // Act
        let chunks = ok(chunks(&v, &indices));

        // Assert
        assert_eq!(chunks.len(), 2);
        assert!(
            chunks
                .iter()
                .all(|c| c.vertices.len() <= CHUNK_MAX_VERTICES)
        );
    }

    #[test]
    fn chunks_remap_indices_to_local_vertices() {
        // Arrange: the second triangle uses vertices 3, 4, 5 only.
        let v = vertices(6);
        let indices = [0, 1, 2, 3, 4, 5];

        // Act
        let one = ok(chunks(&v, &indices));
        let second = ok(chunks(&v, &indices[3..]));

        // Assert
        assert_eq!(one[0].indices, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(second[0].indices, vec![0, 1, 2]);
        assert_eq!(second[0].vertices[0], v[3]);
    }

    #[test]
    fn chunks_reject_index_out_of_range() {
        let v = vertices(3);
        assert!(matches!(chunks(&v, &[0, 1, 3]), Err(GpuError::Backend(_))));
    }

    #[test]
    fn chunks_reject_partial_triangle() {
        let v = vertices(3);
        assert!(matches!(chunks(&v, &[0, 1]), Err(GpuError::Backend(_))));
    }

    proptest! {
        #[test]
        fn chunks_cover_every_triangle_within_limits(
            count in 3usize..40,
            picks in proptest::collection::vec(0u32..40, 0..9_000),
        ) {
            // Arrange
            let v = vertices(count);
            let triangles = picks.len() / 3;
            let indices: Vec<u32> = picks[..triangles * 3]
                .iter()
                .map(|i| i % count as u32)
                .collect();

            // Act
            let chunks = ok(chunks(&v, &indices));

            // Assert: limits hold and the triangles come back in order.
            let mut back = Vec::new();
            for chunk in &chunks {
                prop_assert!(chunk.vertices.len() <= CHUNK_MAX_VERTICES);
                prop_assert!(chunk.indices.len() <= CHUNK_MAX_INDICES);
                prop_assert_eq!(chunk.indices.len() % 3, 0);
                for &i in &chunk.indices {
                    back.push(chunk.vertices[usize::from(i)]);
                }
            }
            let want: Vec<Vertex> = indices.iter().map(|&i| v[i as usize]).collect();
            prop_assert_eq!(back, want);
        }
    }

    // ---- AC-2 handles --------------------------------------------------

    #[test]
    fn table_get_after_insert_returns_value() {
        let mut table = Table::new();
        let id = table.insert("a");
        assert_eq!(ok(table.get(id)), &"a");
    }

    #[test]
    fn table_get_unknown_id_is_unknown_handle() {
        let table: Table<u8> = Table::new();
        assert!(matches!(table.get(7), Err(GpuError::UnknownHandle)));
    }

    #[test]
    fn table_ids_are_unique() {
        let mut table = Table::new();
        let a = table.insert(1);
        let b = table.insert(2);
        assert_ne!(a, b);
    }

    // ---- AC-3 passes ---------------------------------------------------

    fn run(steps: &[Step]) -> Result<(), GpuError> {
        let mut checker = PassChecker::default();
        for &step in steps {
            checker.step(step)?;
        }
        checker.finish()
    }

    #[test]
    fn passes_balanced_list_is_ok() {
        assert!(run(&[Step::Begin, Step::Draw, Step::Draw, Step::End]).is_ok());
    }

    #[test]
    fn passes_draw_outside_pass_is_error() {
        assert!(matches!(run(&[Step::Draw]), Err(GpuError::Backend(_))));
    }

    #[test]
    fn passes_nested_begin_is_error() {
        assert!(matches!(
            run(&[Step::Begin, Step::Begin]),
            Err(GpuError::Backend(_))
        ));
    }

    #[test]
    fn passes_end_without_begin_is_error() {
        assert!(matches!(run(&[Step::End]), Err(GpuError::Backend(_))));
    }

    #[test]
    fn passes_unfinished_list_is_error() {
        assert!(matches!(run(&[Step::Begin]), Err(GpuError::Backend(_))));
    }

    // ---- T159 AC-5 camera ownership -------------------------------------------

    #[test]
    fn every_pass_owns_the_camera() {
        assert!(pass_owns_camera(false));
        assert!(pass_owns_camera(true));
    }
}
