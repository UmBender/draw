//! Chunks of tessellated shapes, rebuilt only when their shapes change (T146).
//!
//! Shapes in z-order are grouped `CHUNK_LEN` at a time, so the draw order of
//! the chunks is the z-order of the document. A chunk is keyed by the ids and
//! revisions of its shapes; the cache keeps its mesh while the key is the same.

use draw_core::core::document::ShapeId;
use draw_core::core::geom::{Aabb, Vec2};
use draw_core::core::shape::Shape;

use crate::tessellate::{Mesh, tessellate};

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
        let mut rebuilt = 0;
        let groups = items.chunks(CHUNK_LEN);
        self.chunks.truncate(groups.len());
        for (slot, group) in groups.enumerate() {
            let same = self.chunks.get(slot).is_some_and(|chunk| {
                chunk.key.len() == group.len()
                    && chunk
                        .key
                        .iter()
                        .zip(group)
                        .all(|(k, item)| *k == (item.id, item.revision))
            });
            if same {
                continue;
            }
            let chunk = build(group);
            self.tessellations += group.len();
            rebuilt += 1;
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
        let _ = zoom;
        todo!()
    }

    /// [`ChunkCache::recompute`] for a view at `zoom`.
    #[must_use]
    pub fn recompute_at(items: &[Item<'_>], zoom: f32) -> Vec<Mesh> {
        let _ = (items, zoom);
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
        self.chunks
            .iter()
            .filter(move |chunk| chunk.bounds.is_some_and(|b| b.intersects(&view)))
    }

    /// The meshes of `items`, tessellated from scratch, one per chunk.
    #[must_use]
    pub fn recompute(items: &[Item<'_>]) -> Vec<Mesh> {
        items.chunks(CHUNK_LEN).map(|g| build(g).mesh).collect()
    }
}

/// Tessellates one group of shapes into a chunk.
fn build(group: &[Item<'_>]) -> Chunk {
    let mut mesh = Mesh::default();
    for item in group {
        let part = tessellate(item.shape);
        let base = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
        mesh.vertices.extend(part.vertices);
        mesh.indices.extend(part.indices.iter().map(|i| i + base));
    }
    let points: Vec<Vec2> = mesh
        .vertices
        .iter()
        .map(|v| Vec2::new(v.pos[0], v.pos[1]))
        .collect();
    Chunk {
        key: group.iter().map(|i| (i.id, i.revision)).collect(),
        bounds: Aabb::from_points(&points),
        mesh,
    }
}
