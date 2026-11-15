//! Tessellation of shapes into triangles in world coordinates (T146).
//!
//! The result does not depend on the camera: a pan or a zoom changes the
//! uniform the vertex shader applies (T173), never these vertices.

use draw_core::core::shape::Shape;

use crate::gpu::{Geometry, Vertex};

/// Indexed triangles in world units.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    /// The vertices, `uv` zero, straight colour.
    pub vertices: Vec<Vertex>,
    /// Triangle list indices into `vertices`.
    pub indices: Vec<u32>,
}

impl Mesh {
    /// The mesh as inline geometry of a draw command.
    #[must_use]
    pub fn geometry<B>(&self) -> Geometry<B> {
        Geometry::Inline {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
        }
    }
}

/// Tessellates `shape`; a shape with a non-finite coordinate, and text
/// (drawn by glyph quads), give an empty mesh.
#[must_use]
pub fn tessellate(shape: &Shape) -> Mesh {
    let _ = shape;
    todo!()
}
