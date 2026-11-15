#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::geom::Vec2;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use crate::shell::app::{MATH_FONT, TEXT_FONT};
    use crate::shell::glyphs::GlyphAtlas;
    use crate::shell::recording::{GpuCall, RecordingGpu};
    use draw_gfx::gpu::{
        Color as GfxColor, Command, CommandOf, Filter, Geometry, Gpu, TargetDesc, View,
    };
    use macroquad::color::WHITE;

    const VIEW: View = View {
        width: 800.0,
        height: 600.0,
    };

    fn ctx() -> RenderCtx<RecordingGpu> {
        let mut ctx = RenderCtx::new(RecordingGpu::new());
        ctx.set_window(VIEW, 1.0);
        ctx
    }

    fn style() -> Style {
        Style {
            color: ColorId::INK,
            width: 2.0,
        }
    }

    fn stroke() -> Shape {
        Shape::Stroke {
            points: vec![Vec2::new(10.0, 10.0), Vec2::new(60.0, 40.0)],
            widths: None,
            style: style(),
        }
    }

    fn text_block() -> Shape {
        Shape::Text {
            origin: Vec2::new(20.0, 20.0),
            text: "AB".to_owned(),
            size: 20.0,
            style: style(),
        }
    }

    fn atlas() -> GlyphAtlas {
        match GlyphAtlas::new(TEXT_FONT).and_then(|atlas| atlas.with_math(MATH_FONT)) {
            Ok(atlas) => atlas,
            Err(err) => panic!("embedded fonts load: {err}"),
        }
    }

    /// The command lists submitted so far.
    fn submitted(ctx: &mut RenderCtx<RecordingGpu>) -> Vec<Vec<CommandOf<RecordingGpu>>> {
        ctx.gpu_mut()
            .take_calls()
            .into_iter()
            .filter_map(|call| match call {
                GpuCall::Submit { commands } => Some(commands),
                _ => None,
            })
            .collect()
    }

    fn is_draw(command: &CommandOf<RecordingGpu>) -> bool {
        matches!(command, Command::Draw { .. })
    }

    #[test]
    fn ctx_stroke_draws_one_untextured_chunk_in_a_pass() {
        // Arrange
        let mut ctx = ctx();
        let shapes = [stroke()];

        // Act
        ctx.draw_shapes(&shapes, &Camera::default(), Vec2::new(800.0, 600.0));

        // Assert
        let lists = submitted(&mut ctx);
        assert_eq!(lists.len(), 1);
        let commands = &lists[0];
        assert_eq!(commands.len(), 3);
        assert_eq!(
            commands[0],
            Command::BeginPass {
                target: None,
                view: VIEW,
                clear: None
            }
        );
        assert!(matches!(
            &commands[1],
            Command::Draw {
                texture: None,
                geometry: Geometry::Inline { .. },
                ..
            }
        ));
        assert_eq!(commands[2], Command::EndPass);
    }

    #[test]
    fn ctx_text_block_binds_the_glyph_page() {
        // Arrange
        let mut ctx = ctx();
        ctx.set_glyph_atlas(atlas());
        let shapes = [text_block()];

        // Act
        ctx.draw_shapes(&shapes, &Camera::default(), Vec2::new(800.0, 600.0));

        // Assert
        let calls = ctx.gpu_mut().take_calls();
        assert!(
            calls
                .iter()
                .any(|call| matches!(call, GpuCall::CreateTexture { .. })),
            "the glyph page is created on the Gpu"
        );
        let textured = calls.iter().any(|call| match call {
            GpuCall::Submit { commands } => commands
                .iter()
                .any(|c| matches!(c, Command::Draw { texture: Some(_), .. })),
            _ => false,
        });
        assert!(textured, "a glyph chunk samples the page");
    }

    #[test]
    fn ctx_menu_label_binds_the_menu_page() {
        // Arrange
        let mut ctx = ctx();

        // Act
        ctx.draw_menu_label("Hi", 10.0, 20.0, 16, WHITE);

        // Assert
        let calls = ctx.gpu_mut().take_calls();
        assert!(
            calls
                .iter()
                .any(|call| matches!(call, GpuCall::CreateTexture { .. }))
        );
        let lists: Vec<_> = calls
            .iter()
            .filter_map(|call| match call {
                GpuCall::Submit { commands } => Some(commands),
                _ => None,
            })
            .collect();
        assert_eq!(lists.len(), 1);
        assert!(
            lists[0]
                .iter()
                .any(|c| matches!(c, Command::Draw { texture: Some(_), .. }))
        );
    }

    #[test]
    fn ctx_render_to_collects_draws_in_one_pass() {
        // Arrange
        let mut ctx = ctx();
        let target = ctx
            .gpu_mut()
            .create_target(TargetDesc {
                width: 64,
                height: 64,
                samples: 1,
                filter: Filter::Linear,
            })
            .ok();
        let Some(target) = target else {
            panic!("the recording gpu makes a target");
        };
        let view = View {
            width: 64.0,
            height: 64.0,
        };
        let clear = Some(GfxColor::rgba(0.0, 0.0, 0.0, 1.0));
        ctx.gpu_mut().take_calls();

        // Act
        ctx.render_to(&target, view, clear, |ctx| {
            ctx.draw_preview(&stroke(), &Camera::default());
            ctx.draw_guides(&[[Vec2::new(0.0, 0.0), Vec2::new(30.0, 0.0)]], &Camera::default());
        });

        // Assert
        let lists = submitted(&mut ctx);
        assert_eq!(lists.len(), 1, "one pass, one submit");
        let commands = &lists[0];
        assert!(matches!(
            commands[0],
            Command::BeginPass {
                target: Some(_),
                clear: Some(_),
                ..
            }
        ));
        assert_eq!(commands.iter().filter(|c| is_draw(c)).count(), 2);
        assert_eq!(commands.last(), Some(&Command::EndPass));
    }

    #[test]
    fn ctx_pipeline_is_created_once() {
        // Arrange
        let mut ctx = ctx();

        // Act
        ctx.draw_preview(&stroke(), &Camera::default());
        ctx.draw_preview(&stroke(), &Camera::default());

        // Assert
        let pipelines = ctx
            .gpu_mut()
            .take_calls()
            .iter()
            .filter(|call| matches!(call, GpuCall::Pipeline { .. }))
            .count();
        assert_eq!(pipelines, 1);
    }

    /// AC-2 of T165: no thread-local holds the `Gpu` or its resources in
    /// `render.rs` or `ui_text.rs`; the bridge in this file is the only
    /// one in the shell.
    #[test]
    fn render_and_ui_text_hold_no_gpu_thread_local() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shell");
        let Ok(entries) = std::fs::read_dir(&dir) else {
            panic!("cannot read {dir:?}");
        };
        let mut bridges = Vec::new();
        for path in entries.filter_map(|e| e.ok().map(|e| e.path())) {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_owned();
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(&path) else {
                panic!("cannot read {path:?}");
            };
            let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
            let holds_gpu = production.contains("thread_local!")
                && ["MqGpu", "GlyphTextures", "MqPipeline", "CommandOf", "Glyphs>"]
                    .iter()
                    .any(|word| {
                        production
                            .lines()
                            .any(|l| l.contains("static ") && l.contains(word))
                    });
            if holds_gpu {
                bridges.push(name);
            }
        }
        assert_eq!(bridges, vec!["render_ctx.rs".to_owned()]);
    }
}
