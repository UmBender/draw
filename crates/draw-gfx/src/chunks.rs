//! Chunks of tessellated shapes, rebuilt only when their shapes change (T146).
//!
//! Shapes in z-order are grouped `CHUNK_LEN` at a time, so the draw order of
//! the chunks is the z-order of the document. A chunk is keyed by the ids and
//! revisions of its shapes; the cache keeps its mesh while the key is the same.

use draw_core::core::document::ShapeId;
use draw_core::core::geom::{Aabb, Vec2};
use draw_core::core::shape::Shape;

use crate::tessellate::{Mesh, tessellate_many_at, zoom_bucket};

/// Shapes per chunk.
pub const CHUNK_LEN: usize = 64;

/// A shape as the cache sees it: its id, its revision (it grows with each
/// edit of the shape) and the shape.
#[derive(Debug, Clone, Copy)]
pub struct Item<'a> {
    /// The shape's id.
    pub id: ShapeId,
    /// Changes whenever the shape changes.
    pub revision: u64,
    /// The shape.
    pub shape: &'a Shape,
}

/// One cached chunk.
#[derive(Debug, Clone)]
pub struct Chunk {
    key: Vec<(ShapeId, u64)>,
    revision: u64,
    mesh: Mesh,
    bounds: Option<Aabb>,
}

impl Chunk {
    /// Monotonic revision bumped whenever this chunk is built or rebuilt.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The tessellation of the chunk's shapes, in z-order.
    #[must_use]
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    /// World bounds of the mesh, `None` when it is empty.
    #[must_use]
    pub fn bounds(&self) -> Option<Aabb> {
        self.bounds
    }
}

/// What one [`ChunkCache::update`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateStats {
    /// Chunks that were tessellated again.
    pub rebuilt: usize,
}

/// The chunks of a document.
#[derive(Debug, Clone)]
pub struct ChunkCache {
    chunks: Vec<Chunk>,
    tessellations: usize,
    next_rev: u64,
    /// The zoom bucket the chunks are tessellated for (ADR-T179-1).
    bucket: f32,
}

impl Default for ChunkCache {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            tessellations: 0,
            next_rev: 0,
            bucket: 1.0,
        }
    }
}

impl ChunkCache {
    /// An empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Brings the cache in line with `items` (bottom to top), tessellating
    /// only the chunks whose key changed.
    pub fn update(&mut self, items: &[Item<'_>]) -> UpdateStats {
        let groups: Vec<&[Item<'_>]> = items.chunks(CHUNK_LEN).collect();
        self.chunks.truncate(groups.len());
        let mut dirty_slots = Vec::new();
        let mut dirty_shapes = Vec::new();
        for (slot, group) in groups.iter().enumerate() {
            let same = self.chunks.get(slot).is_some_and(|chunk| {
                chunk.key.len() == group.len()
                    && chunk
                        .key
                        .iter()
                        .zip(*group)
                        .all(|(k, item)| *k == (item.id, item.revision))
            });
            if !same {
                dirty_slots.push(slot);
                dirty_shapes.extend(group.iter().map(|item| item.shape));
            }
        }
        if dirty_slots.is_empty() {
            return UpdateStats { rebuilt: 0 };
        }
        let threads = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        let meshes = tessellate_many_at(&dirty_shapes, self.bucket, threads);
        let mut offset = 0;
        let rebuilt = dirty_slots.len();
        for slot in dirty_slots {
            let group = groups[slot];
            let end = offset + group.len();
            self.next_rev += 1;
            let chunk = assemble(group, &meshes[offset..end], self.next_rev);
            offset = end;
            self.tessellations += group.len();
            match self.chunks.get_mut(slot) {
                Some(old) => *old = chunk,
                None => self.chunks.push(chunk),
            }
        }
        UpdateStats { rebuilt }
    }

    /// Sets the zoom the chunks are tessellated for (ADR-T179-1); `true`
    /// when its bucket changed, which drops every chunk.
    pub fn set_zoom(&mut self, zoom: f32) -> bool {
        let bucket = zoom_bucket(zoom);
        if bucket.to_bits() == self.bucket.to_bits() {
            return false;
        }
        self.bucket = bucket;
        self.chunks.clear();
        true
    }

    /// [`ChunkCache::recompute`] for a view at `zoom`.
    #[must_use]
    pub fn recompute_at(items: &[Item<'_>], zoom: f32) -> Vec<Mesh> {
        let bucket = zoom_bucket(zoom);
        let shapes: Vec<&Shape> = items.iter().map(|i| i.shape).collect();
        let threads = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        let meshes = tessellate_many_at(&shapes, bucket, threads);
        let mut offset = 0;
        items
            .chunks(CHUNK_LEN)
            .map(|g| {
                let end = offset + g.len();
                let chunk = assemble(g, &meshes[offset..end], 0);
                offset = end;
                chunk.mesh
            })
            .collect()
    }

    /// The chunks, bottom to top.
    #[must_use]
    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    /// Shapes tessellated since the cache was created.
    #[must_use]
    pub fn tessellations(&self) -> usize {
        self.tessellations
    }

    /// The chunks whose bounds meet `view` (world units), bottom to top.
    pub fn visible(&self, view: Aabb) -> impl Iterator<Item = &Chunk> + '_ {
        self.chunks
            .iter()
            .filter(move |chunk| chunk.bounds.is_some_and(|b| b.intersects(&view)))
    }

    /// The meshes of `items`, tessellated from scratch, one per chunk.
    #[must_use]
    pub fn recompute(items: &[Item<'_>]) -> Vec<Mesh> {
        Self::recompute_at(items, 1.0)
    }
}

/// Assembles one group of shapes and their tessellated parts into a chunk.
fn assemble(group: &[Item<'_>], parts: &[Mesh], revision: u64) -> Chunk {
    let mut mesh = Mesh::default();
    for part in parts {
        let base = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
        mesh.vertices.extend_from_slice(&part.vertices);
        mesh.indices.extend(part.indices.iter().map(|i| i + base));
    }
    let points: Vec<Vec2> = mesh
        .vertices
        .iter()
        .map(|v| Vec2::new(v.pos[0], v.pos[1]))
        .collect();
    Chunk {
        key: group.iter().map(|i| (i.id, i.revision)).collect(),
        revision,
        bounds: Aabb::from_points(&points),
        mesh,
    }
}
