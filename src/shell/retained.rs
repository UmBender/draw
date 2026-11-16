//! The retained document path of the app (T172): the shapes of the document
//! as cached chunks of world-space triangles, drawn under the camera
//! uniform, so a pan or a zoom inside a bucket tessellates nothing.
//!
//! Text, formulas, labels and the minimap are not on this path yet
//! (T172 AC-1d); the default renderer stays the layer cache.

use std::collections::HashMap;

use draw_gfx::chunks::{ChunkCache, Item};
use draw_gfx::gpu::{Camera as GfxCamera, Gpu};

use crate::core::camera::Camera;
use crate::core::document::ShapeId;
use crate::core::geom::Vec2;
use crate::core::shape::Shape;
use crate::shell::render::cull_rect;
use crate::shell::render_ctx::RenderCtx;

/// The chunks of a document and the revisions that key them.
#[derive(Debug, Default)]
pub struct RetainedScene {
    cache: ChunkCache,
    /// Each shape as last seen and its revision.
    seen: HashMap<ShapeId, (u64, Shape)>,
    next_revision: u64,
}

impl RetainedScene {
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

    /// Draws `shapes` (bottom to top) except the `hidden` ones, through
    /// `camera` in a viewport of `viewport` logical pixels: tessellates the
    /// chunks whose shapes changed, then draws the chunks in view.
    pub fn draw<G: Gpu>(
        &mut self,
        ctx: &mut RenderCtx<G>,
        shapes: &[(ShapeId, &Shape)],
        hidden: &[ShapeId],
        camera: &Camera,
        viewport: Vec2,
    ) {
        todo!("T172")
    }
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
        scene.draw(&mut ctx, &items(&shapes), &[], &Camera::new(Vec2::ZERO, 1.0), PORT);
        let before = scene.tessellations();

        // Act
        scene.draw(&mut ctx, &items(&shapes), &[], &Camera::new(Vec2::ZERO, 1.01), PORT);

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
}
