//! The chunk cache and the world-space tessellation (T146, AC-1 and AC-2).

use draw_core::core::document::ShapeId;
use draw_core::core::geom::{Aabb, Vec2};
use draw_core::core::palette::ColorId;
use draw_core::core::shape::{Shape, Style};
use draw_gfx::chunks::{CHUNK_LEN, ChunkCache, Item};
use draw_gfx::gpu::Geometry;
use draw_gfx::tessellate::{Mesh, tessellate};
use proptest::prelude::*;

const STYLE: Style = Style {
    color: ColorId::INK,
    width: 4.0,
};

fn line(i: usize) -> Shape {
    let x = i as f32 * 10.0;
    Shape::Line {
        a: Vec2::new(x, 0.0),
        b: Vec2::new(x, 50.0),
        style: STYLE,
    }
}

fn shapes(n: usize) -> Vec<Shape> {
    (0..n).map(line).collect()
}

fn items<'a>(shapes: &'a [Shape], revisions: &[u64]) -> Vec<Item<'a>> {
    shapes
        .iter()
        .zip(revisions)
        .enumerate()
        .map(|(i, (shape, &revision))| Item {
            id: ShapeId(i as u64),
            revision,
            shape,
        })
        .collect()
}

#[test]
fn edit_rebuilds_only_its_chunk() {
    // Arrange: three chunks.
    let mut all = shapes(CHUNK_LEN * 3);
    let mut revs = vec![0; all.len()];
    let mut cache = ChunkCache::new();
    cache.update(&items(&all, &revs));
    let before = cache.tessellations();
    // Act: edit one shape of the middle chunk.
    all[CHUNK_LEN + 5] = line(999);
    revs[CHUNK_LEN + 5] = 1;
    let stats = cache.update(&items(&all, &revs));
    // Assert
    assert_eq!(stats.rebuilt, 1);
    assert_eq!(cache.tessellations() - before, CHUNK_LEN);
}

#[test]
fn unchanged_update_tessellates_nothing() {
    let all = shapes(CHUNK_LEN + 3);
    let revs = vec![0; all.len()];
    let mut cache = ChunkCache::new();
    let first = cache.update(&items(&all, &revs));
    assert_eq!(first.rebuilt, 2);
    let before = cache.tessellations();
    let again = cache.update(&items(&all, &revs));
    assert_eq!(again.rebuilt, 0);
    assert_eq!(cache.tessellations(), before);
}

#[test]
fn insert_at_end_rebuilds_only_the_last_chunk() {
    let all = shapes(CHUNK_LEN * 2);
    let revs = vec![0; all.len()];
    let mut cache = ChunkCache::new();
    cache.update(&items(&all, &revs));
    let more = shapes(CHUNK_LEN * 2 + 1);
    let revs = vec![0; more.len()];
    let stats = cache.update(&items(&more, &revs));
    assert_eq!(stats.rebuilt, 1);
    assert_eq!(cache.chunks().len(), 3);
}

#[test]
fn pan_and_zoom_tessellate_nothing() {
    let all = shapes(CHUNK_LEN * 2);
    let revs = vec![0; all.len()];
    let mut cache = ChunkCache::new();
    cache.update(&items(&all, &revs));
    let before = cache.tessellations();
    // Act: many "camera changes": updates with the same items and views.
    for k in 0..20 {
        let shift = k as f32 * 37.0;
        let view = Aabb::from_corners(Vec2::new(shift, 0.0), Vec2::new(shift + 300.0, 200.0));
        let _ = cache.visible(view).count();
        cache.update(&items(&all, &revs));
    }
    assert_eq!(cache.tessellations(), before);
}

#[test]
fn visible_chunks_follow_the_view() {
    let all = shapes(CHUNK_LEN * 3);
    let revs = vec![0; all.len()];
    let mut cache = ChunkCache::new();
    cache.update(&items(&all, &revs));
    let width = CHUNK_LEN as f32 * 10.0;
    let first = Aabb::from_corners(Vec2::new(0.0, 0.0), Vec2::new(width - 20.0, 50.0));
    let far = Aabb::from_corners(Vec2::new(-5000.0, -5000.0), Vec2::new(-4000.0, -4000.0));
    let everything = Aabb::from_corners(Vec2::new(-10.0, -10.0), Vec2::new(width * 4.0, 100.0));
    assert_eq!(cache.visible(first).count(), 1);
    assert_eq!(cache.visible(far).count(), 0);
    assert_eq!(cache.visible(everything).count(), 3);
}

#[test]
fn tessellate_line_covers_its_width() {
    let mesh = tessellate(&line(0));
    assert!(!mesh.vertices.is_empty());
    assert_eq!(mesh.indices.len() % 3, 0);
    let xs: Vec<f32> = mesh.vertices.iter().map(|v| v.pos[0]).collect();
    let min = xs.iter().copied().fold(f32::INFINITY, f32::min);
    let max = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!((max - min - 4.0).abs() < 1e-3, "width {}", max - min);
}

#[test]
fn tessellate_filled_rect_has_fill_and_outline() {
    let rect = |fill| Shape::Rect {
        a: Vec2::new(0.0, 0.0),
        b: Vec2::new(40.0, 30.0),
        style: STYLE,
        fill,
        label: None,
        text: None,
    };
    let open = tessellate(&rect(None));
    let filled = tessellate(&rect(ColorId::new(2)));
    assert!(!open.indices.is_empty());
    assert!(filled.indices.len() > open.indices.len());
    for mesh in [&open, &filled] {
        assert!(mesh.indices.iter().all(|&i| (i as usize) < mesh.vertices.len()));
    }
}

#[test]
fn tessellate_non_finite_is_empty() {
    let bad = Shape::Line {
        a: Vec2::new(f32::NAN, 0.0),
        b: Vec2::new(1.0, f32::INFINITY),
        style: STYLE,
    };
    assert_eq!(tessellate(&bad), Mesh::default());
}

#[test]
fn mesh_geometry_is_inline_with_the_same_triangles() {
    let mesh = tessellate(&line(1));
    let Geometry::<()>::Inline { vertices, indices } = mesh.geometry() else {
        panic!("a mesh is inline geometry");
    };
    assert_eq!(vertices, mesh.vertices);
    assert_eq!(indices, mesh.indices);
}

fn view(rows: &[(u64, u64, Shape)]) -> Vec<Item<'_>> {
    rows.iter()
        .map(|(id, revision, shape)| Item {
            id: ShapeId(*id),
            revision: *revision,
            shape,
        })
        .collect()
}

#[derive(Debug, Clone)]
enum EditOp {
    Replace(usize, usize),
    Insert(usize, usize),
    Remove(usize),
}

fn edit_op() -> impl Strategy<Value = EditOp> {
    prop_oneof![
        (0..400usize, 0..50usize).prop_map(|(i, s)| EditOp::Replace(i, s)),
        (0..400usize, 0..50usize).prop_map(|(i, s)| EditOp::Insert(i, s)),
        (0..400usize).prop_map(EditOp::Remove),
    ]
}

proptest! {
    #[test]
    fn cached_geometry_equals_recomputed(
        start in 0..200usize,
        ops in proptest::collection::vec(edit_op(), 0..30),
    ) {
        // Arrange: a document of (id, revision, shape) rows.
        let mut next_id = 0u64;
        let mut rows: Vec<(u64, u64, Shape)> = (0..start)
            .map(|i| { next_id += 1; (next_id, 0, line(i)) })
            .collect();
        let mut cache = ChunkCache::new();
        cache.update(&view(&rows));
        for op in ops {
            match op {
                EditOp::Replace(i, s) if !rows.is_empty() => {
                    let i = i % rows.len();
                    rows[i].1 += 1;
                    rows[i].2 = line(s);
                }
                EditOp::Insert(i, s) => {
                    let i = i % (rows.len() + 1);
                    next_id += 1;
                    rows.insert(i, (next_id, 0, line(s)));
                }
                EditOp::Remove(i) if !rows.is_empty() => {
                    let i = i % rows.len();
                    rows.remove(i);
                }
                EditOp::Replace(..) | EditOp::Remove(_) => {}
            }
            // Act
            cache.update(&view(&rows));
            // Assert: the cache holds what a fresh tessellation gives.
            let fresh = ChunkCache::recompute(&view(&rows));
            let cached: Vec<&Mesh> = cache.chunks().iter().map(|c| c.mesh()).collect();
            prop_assert_eq!(cached.len(), fresh.len());
            for (c, f) in cached.iter().zip(&fresh) {
                prop_assert_eq!(*c, f);
            }
        }
    }
}
