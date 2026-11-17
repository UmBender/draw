//! The drawing context: everything the drawing of the canvas and of the
//! menus needs from a [`Gpu`], in one value (T165).
//!
//! [`RenderCtx`] owns the `Gpu`, the glyph page textures, the alpha
//! pipeline, the draws of an open [`RenderCtx::render_to`] scope and the
//! menu glyphs. `render.rs` and `ui_text.rs` draw through its methods;
//! `app::run` owns one `RenderCtx<G>` and passes it down (T166, T167), after
//! [`RenderCtx::sync_window`] with the platform's metrics.

use draw_gfx::gpu::{
    Blend, Camera as GfxCamera, Color as GfxColor, Command, CommandList, CommandOf, Geometry, Gpu,
    PipelineDesc, Vertex, View,
};
use draw_gfx::tessellate::Mesh;
use draw_sys::platform::WindowMetrics;

use crate::core::geom::Aabb;
use crate::shell::glyphs::{GlyphAtlas, GlyphTextures};
use crate::shell::render::{texture_quad, wrap_pass};
use crate::shell::ui_text::MenuGlyphs;

/// A [`Gpu`] with the resources the drawing keeps on it.
pub struct RenderCtx<G: Gpu> {
    /// The interpreter every draw goes through.
    pub(crate) gpu: G,
    /// The glyph atlas pages of the canvas text on `gpu`.
    textures: GlyphTextures<G>,
    /// The alpha-blend pipeline, made on first use.
    pipeline: Option<G::Pipeline>,
    /// The feathered pipeline of the retained path, made on first use.
    feathered: Option<G::Pipeline>,
    /// The draws collected by the open [`RenderCtx::render_to`], if any.
    scope: Option<Vec<CommandOf<G>>>,
    /// The menu font's atlas and pages; made on first use.
    pub(crate) menu: Option<MenuGlyphs<G>>,
    /// The window in logical pixels: the view of a draw outside a scope.
    window: View,
    /// The window's DPI scale.
    dpi: f32,
}

/// One draw inside a retained world-space pass (ADR-T173-1, ADR-T180-1).
pub enum WorldDraw<'a> {
    /// Feathered shape geometry (`Blend::AlphaFeathered`, untextured).
    Feathered(&'a Mesh),
    /// Untextured straight-alpha geometry (math rules, `Blend::Alpha`).
    Alpha(&'a Mesh),
    /// Glyph quads sampling atlas `page` (`Blend::Alpha`).
    Glyphs {
        /// Page index in `RenderCtx::textures`.
        page: u32,
        /// World-space vertices.
        vertices: &'a [Vertex],
        /// Triangle list indices into `vertices`.
        indices: &'a [u32],
    },
}

impl<G: Gpu> RenderCtx<G> {
    /// A context drawing on `gpu`, with a 1×1 window until
    /// [`RenderCtx::set_window`].
    #[must_use]
    pub fn new(gpu: G) -> Self {
        Self {
            gpu,
            textures: GlyphTextures::default(),
            pipeline: None,
            feathered: None,
            scope: None,
            menu: None,
            window: View {
                width: 1.0,
                height: 1.0,
            },
            dpi: 1.0,
        }
    }

    /// The interpreter.
    #[must_use]
    pub fn gpu(&self) -> &G {
        &self.gpu
    }

    /// The interpreter, for resources and [`RenderCtx::submit`]; not for
    /// drawing.
    pub fn gpu_mut(&mut self) -> &mut G {
        &mut self.gpu
    }

    /// The glyph page textures of the canvas text.
    #[must_use]
    pub fn textures(&self) -> &GlyphTextures<G> {
        &self.textures
    }

    /// Sets the window size (logical pixels) and DPI scale the drawing
    /// reads.
    pub fn set_window(&mut self, view: View, dpi: f32) {
        self.window = view;
        self.dpi = dpi;
    }

    /// [`RenderCtx::set_window`] from what the platform reports.
    pub fn sync_window(&mut self, metrics: WindowMetrics) {
        let view = View {
            width: metrics.width,
            height: metrics.height,
        };
        self.set_window(view, metrics.scale);
    }

    /// The window in logical pixels.
    #[must_use]
    pub fn window(&self) -> View {
        self.window
    }

    /// The window's DPI scale.
    #[must_use]
    pub fn dpi(&self) -> f32 {
        self.dpi
    }

    /// Whether a [`RenderCtx::render_to`] scope is open.
    #[must_use]
    pub fn in_scope(&self) -> bool {
        self.scope.is_some()
    }

    /// The alpha-blend pipeline; made on the first call.
    pub fn alpha_pipeline(&mut self) -> G::Pipeline {
        let gpu = &mut self.gpu;
        *self.pipeline.get_or_insert_with(|| {
            gpu.pipeline(PipelineDesc {
                blend: Blend::Alpha,
            })
        })
    }

    /// The feathered alpha pipeline of the retained path (ADR-T173-1); made
    /// on the first call.
    pub fn feathered_pipeline(&mut self) -> G::Pipeline {
        let gpu = &mut self.gpu;
        *self.feathered.get_or_insert_with(|| {
            gpu.pipeline(PipelineDesc {
                blend: Blend::AlphaFeathered,
            })
        })
    }

    /// Draws world-space `meshes` in order under `camera`, in the open
    /// scope or a pass of their own on the window; nothing for no meshes.
    pub fn draw_world(&mut self, camera: GfxCamera, meshes: &[&Mesh]) {
        if meshes.is_empty() {
            return;
        }
        let items: Vec<WorldDraw<'_>> = meshes.iter().map(|m| WorldDraw::Feathered(m)).collect();
        self.draw_retained(camera, &items);
    }

    /// Draws world-space shape chunks, rules and glyph runs in order under
    /// `camera`, in the open scope or a pass of their own on the window;
    /// nothing for an empty slice.
    pub fn draw_retained(&mut self, camera: GfxCamera, items: &[WorldDraw<'_>]) {
        if items.is_empty() {
            return;
        }
        let need_feathered = items
            .iter()
            .any(|item| matches!(item, WorldDraw::Feathered(_)));
        let need_alpha = items
            .iter()
            .any(|item| matches!(item, WorldDraw::Alpha(_) | WorldDraw::Glyphs { .. }));
        let feathered = need_feathered.then(|| self.feathered_pipeline());
        let alpha = need_alpha.then(|| self.alpha_pipeline());
        let mut draws: Vec<CommandOf<G>> = Vec::with_capacity(items.len() + 2);
        draws.push(Command::SetCamera(camera));
        for item in items {
            match *item {
                WorldDraw::Feathered(mesh) => {
                    if let Some(pipeline) = feathered {
                        draws.push(Command::Draw {
                            pipeline,
                            texture: None,
                            geometry: mesh.geometry(),
                        });
                    }
                }
                WorldDraw::Alpha(mesh) => {
                    if let Some(pipeline) = alpha {
                        draws.push(Command::Draw {
                            pipeline,
                            texture: None,
                            geometry: mesh.geometry(),
                        });
                    }
                }
                WorldDraw::Glyphs {
                    page,
                    vertices,
                    indices,
                } => {
                    if let Some(pipeline) = alpha {
                        draws.push(Command::Draw {
                            pipeline,
                            texture: self.textures.get(page).cloned(),
                            geometry: Geometry::Inline {
                                vertices: vertices.to_vec(),
                                indices: indices.to_vec(),
                            },
                        });
                    }
                }
            }
        }
        draws.push(Command::SetCamera(GfxCamera::IDENTITY));
        if let Some(open) = self.scope.as_mut() {
            open.extend(draws);
        } else {
            let list = wrap_pass::<G>(None, self.window, None, draws);
            self.submit(&list);
        }
    }

    /// Executes `list`. The lists are built from valid handles; a refusal
    /// drops the list.
    pub fn submit(&mut self, list: &CommandList<G>) {
        let _ = self.gpu.submit(list);
    }

    /// Puts the new pages and uploads of `atlas` on the GPU. A failed
    /// upload leaves the page as it was: its glyphs draw blank.
    pub fn sync_textures(&mut self, atlas: &mut GlyphAtlas) {
        if let Err(err) = self.textures.sync(&mut self.gpu, atlas) {
            eprintln!("glyph textures: {err}");
        }
    }

    /// Sends one inline draw with `texture`: collected into the open scope,
    /// else a pass of its own on the window.
    pub fn draw_inline(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
        texture: Option<G::Texture>,
    ) {
        let draw = Command::Draw {
            pipeline: self.alpha_pipeline(),
            texture,
            geometry: Geometry::Inline {
                vertices: vertices.to_vec(),
                indices: indices.to_vec(),
            },
        };
        if let Some(open) = self.scope.as_mut() {
            open.push(draw);
        } else {
            let list = wrap_pass::<G>(None, self.window, None, vec![draw]);
            self.submit(&list);
        }
    }

    /// Sends one chunk bound to glyph atlas `page` (`None`: untextured).
    pub fn draw_chunk(&mut self, vertices: &[Vertex], indices: &[u32], page: Option<u32>) {
        let texture = page.and_then(|page| self.textures.get(page).cloned());
        self.draw_inline(vertices, indices, texture);
    }

    /// Runs `draw` with every draw collected into one pass over `target`,
    /// seen through `view` (logical pixels), after clearing it to `clear`
    /// if given. One pass means one resolve of a multisampled target
    /// (ADR-T159-1). Not reentrant.
    pub fn render_to(
        &mut self,
        target: &G::Target,
        view: View,
        clear: Option<GfxColor>,
        draw: impl FnOnce(&mut Self),
    ) {
        self.begin_scope();
        draw(self);
        self.end_scope(target, view, clear);
    }

    /// Opens a scope: draws are collected until [`RenderCtx::end_scope`].
    pub fn begin_scope(&mut self) {
        self.scope = Some(Vec::new());
    }

    /// Closes the scope, submitting its draws in one pass over `target`.
    pub fn end_scope(&mut self, target: &G::Target, view: View, clear: Option<GfxColor>) {
        let draws = self.scope.take().unwrap_or_default();
        let list = wrap_pass::<G>(Some(target.clone()), view, clear, draws);
        self.submit(&list);
    }

    /// Clears the window to `color`.
    pub fn clear_window(&mut self, color: GfxColor) {
        let list = wrap_pass::<G>(None, self.window, Some(color), Vec::new());
        self.submit(&list);
    }

    /// Draws the whole `texture` stretched over `dest` on the window, in
    /// window pixels, rows from the bottom when `flip_y`.
    pub fn draw_texture_quad(&mut self, texture: &G::Texture, dest: Aabb, flip_y: bool) {
        let (vertices, indices) = texture_quad(dest, flip_y, [255; 4]);
        self.draw_inline(&vertices, &indices, Some(texture.clone()));
    }
}

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
            GpuCall::Submit { commands } => commands.iter().any(|c| {
                matches!(
                    c,
                    Command::Draw {
                        texture: Some(_),
                        ..
                    }
                )
            }),
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
        assert!(lists[0].iter().any(|c| matches!(
            c,
            Command::Draw {
                texture: Some(_),
                ..
            }
        )));
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
            ctx.draw_guides(
                &[[Vec2::new(0.0, 0.0), Vec2::new(30.0, 0.0)]],
                &Camera::default(),
            );
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

    /// The production source (above `#[cfg(test)]`) of `src/shell/<name>`.
    fn shell_production(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/shell")
            .join(name);
        let Ok(source) = std::fs::read_to_string(&path) else {
            panic!("cannot read {path:?}");
        };
        source
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(&source)
            .to_owned()
    }

    /// AC-1 of T166: no `thread_local!` in the shell holds the `Gpu`, its
    /// resources or a `RenderCtx`, and nothing reaches one by a bridge.
    #[test]
    fn shell_sources_have_no_gpu_thread_local() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shell");
        let Ok(entries) = std::fs::read_dir(&dir) else {
            panic!("cannot read {dir:?}");
        };
        let words = [
            "Gpu",
            "GPU",
            "TEXTURES",
            "SCOPE",
            "PIPELINE",
            "RenderCtx",
            "with_gpu",
            "render::submit",
        ];
        let mut offenders = Vec::new();
        for path in entries.filter_map(|e| e.ok().map(|e| e.path())) {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_owned();
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let production = shell_production(&name);
            for block in production.split("thread_local!").skip(1) {
                let body = block.split("\n}").next().unwrap_or(block);
                if words.iter().any(|word| body.contains(word)) {
                    offenders.push(name.clone());
                }
            }
            for bridge in ["with_gpu", "with_ctx", "with_drawing"] {
                let defines = production.contains(&format!("fn {bridge}"));
                if defines {
                    offenders.push(format!("{name}: fn {bridge}"));
                }
            }
        }
        assert_eq!(offenders, Vec::<String>::new());
    }

    /// AC-2 of T166: the drawing asks no macroquad function for the window
    /// size or the DPI scale.
    #[test]
    fn drawing_reads_the_window_size_from_the_platform() {
        let names = [
            "screen_height",
            "screen_dpi_scale",
            "dpi_scale",
            "macroquad::window",
        ];
        let mut direct = Vec::new();
        for file in ["render.rs", "ui_text.rs", "app.rs", "render_ctx.rs"] {
            let code = shell_production(file);
            for name in names {
                if code.contains(name) {
                    direct.push(format!("{file}: {name}"));
                }
            }
        }
        assert_eq!(direct, Vec::<String>::new());
    }

    #[test]
    fn ctx_sync_window_draws_at_the_platform_size_and_dpi() {
        use draw_sys::platform::WindowMetrics;

        // Arrange
        let mut ctx = ctx();
        let metrics = WindowMetrics {
            width: 1024.0,
            height: 768.0,
            scale: 2.0,
        };

        // Act
        ctx.sync_window(metrics);
        ctx.draw_preview(&stroke(), &Camera::default());

        // Assert
        let lists = submitted(&mut ctx);
        assert_eq!(
            lists[0][0],
            Command::BeginPass {
                target: None,
                view: View {
                    width: 1024.0,
                    height: 768.0
                },
                clear: None
            }
        );
        assert!(crate::core::geom::approx_eq(ctx.dpi(), 2.0, 1e-6));
    }
}
