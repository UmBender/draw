//! Chunks of tessellated shapes, rebuilt only when their shapes change (T146).
//!
//! Shapes in z-order are grouped `CHUNK_LEN` at a time, so the draw order of
//! the chunks is the z-order of the document. A chunk is keyed by the ids and
//! revisions of its shapes; the cache keeps its mesh while the key is the same.

use draw_core::core::document::ShapeId;
use draw_core::core::geom::Aabb;
use draw_core::core::shape::Shape;

use crate::tessellate::Mesh;

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
    mesh: Mesh,
    bounds: Option<Aabb>,
}

impl Chunk {
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
#[derive(Debug, Clone, Default)]
pub struct ChunkCache {
    chunks: Vec<Chunk>,
    tessellations: usize,
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
        let _ = items;
        todo!()
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
        let _ = view;
        todo!();
        #[allow(unreachable_code)]
        self.chunks.iter()
    }

    /// The meshes of `items`, tessellated from scratch, one per chunk.
    #[must_use]
    pub fn recompute(items: &[Item<'_>]) -> Vec<Mesh> {
        let _ = items;
        todo!()
    }
}
