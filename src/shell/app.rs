//! Window configuration and the main loop.
//!
//! The loop follows ADR-T12-1: miniquad's blocking event loop sleeps until
//! input arrives, raw events are replayed into an [`Collector`],
//! routed past the toolbar into the [`Editor`], and the scene is re-rendered
//! into a cached render target only when something changed. Every woken frame
//! then blits that texture.

use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::WHITE;
use macroquad::conf::{Conf, UpdateTrigger};
use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::math::{Rect, vec2};
use macroquad::miniquad::conf::{Conf as WindowConf, Platform};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::texture::{
    DrawTextureParams, FilterMode, RenderTarget, draw_texture_ex, render_target,
};
use macroquad::window::{
    clear_background, next_frame, screen_dpi_scale, screen_height, screen_width,
};

use crate::core::editor::Editor;
use crate::core::geom::{Aabb, Vec2};
use crate::core::input::InputEvent;
use crate::core::palette::THEME;
use crate::shell::input_map::Collector;
use crate::shell::render::{self, to_mq_color};
use crate::shell::toolbar::{self, Route};

/// Opacity of the marquee fill.
const MARQUEE_FILL_ALPHA: f32 = 0.25;

/// Viewport sizes closer than this many pixels count as unchanged.
const RESIZE_EPS_PX: f32 = 0.5;

/// The window the app opens. `main` passes it to macroquad, which converts it
/// into a [`Conf`] with the blocking event loop (ADR-T12-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSettings {
    /// Window title.
    pub window_title: String,
    /// Initial width in logical pixels.
    pub window_width: i32,
    /// Initial height in logical pixels.
    pub window_height: i32,
    /// Whether the user can resize the window.
    pub window_resizable: bool,
    /// Whether to render at the display's full resolution.
    pub high_dpi: bool,
}

impl From<WindowSettings> for Conf {
    /// Blocking event loop woken by every kind of pointer and key input.
    fn from(settings: WindowSettings) -> Self {
        Self {
            miniquad_conf: WindowConf {
                window_title: settings.window_title,
                window_width: settings.window_width,
                window_height: settings.window_height,
                window_resizable: settings.window_resizable,
                high_dpi: settings.high_dpi,
                platform: Platform {
                    blocking_event_loop: true,
                    ..Platform::default()
                },
                ..WindowConf::default()
            },
            update_on: Some(UpdateTrigger {
                key_down: true,
                mouse_down: true,
                mouse_up: true,
                mouse_motion: true,
                mouse_wheel: true,
                specific_key: None,
                touch: false,
            }),
            ..Conf::default()
        }
    }
}

/// Window settings used by `main`.
#[must_use]
pub fn window_conf() -> WindowSettings {
    WindowSettings {
        window_title: "draw".to_owned(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        high_dpi: true,
    }
}

/// Runs the application until the window is closed.
pub async fn run() {
    let subscriber = register_input_subscriber();
    let mut collector = Collector::new();
    let mut editor = Editor::new();
    let mut frame = CachedFrame::default();
    loop {
        let viewport = Vec2::new(screen_width(), screen_height());
        let dpi = screen_dpi_scale();
        let mut dirty = false;
        if !viewport.approx_eq(editor.viewport(), RESIZE_EPS_PX) {
            dirty |= editor.handle(InputEvent::Resize { size: viewport });
        }
        collector.set_dpi(dpi);
        repeat_all_miniquad_input(&mut collector, subscriber);
        for event in collector.drain() {
            dirty |= dispatch(&mut editor, event);
        }
        frame.present(&editor, viewport, dpi, dirty);
        next_frame().await;
    }
}

/// Sends one event to the toolbar or the editor. Returns whether the view
/// needs a redraw.
fn dispatch(editor: &mut Editor, event: InputEvent) -> bool {
    match toolbar::route(editor.toolbar_visible(), editor.viewport(), event) {
        Route::Forward(event) => editor.handle(event),
        Route::Apply(command) => editor.apply(command),
        Route::Swallow => false,
    }
}

/// The last rendered scene, kept in a framebuffer-sized render target.
#[derive(Default)]
struct CachedFrame {
    target: Option<RenderTarget>,
    /// Size of `target` in physical pixels.
    size: (u32, u32),
}

impl CachedFrame {
    /// Re-renders the scene if `dirty` or the framebuffer size changed, then
    /// blits it to the window.
    fn present(&mut self, editor: &Editor, viewport: Vec2, dpi: f32, dirty: bool) {
        let size = physical_size(viewport, dpi);
        let target = match &self.target {
            Some(target) if self.size == size && !dirty => target,
            Some(target) if self.size == size => {
                render_into(target, editor, viewport);
                target
            }
            _ => {
                let target = render_target(size.0, size.1);
                target.texture.set_filter(FilterMode::Nearest);
                render_into(&target, editor, viewport);
                self.size = size;
                self.target.insert(target)
            }
        };
        draw_texture_ex(
            &target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(viewport.x, viewport.y)),
                // GL textures start at the bottom row.
                flip_y: true,
                ..DrawTextureParams::default()
            },
        );
    }
}

/// Framebuffer size in physical pixels for a logical `viewport`, at least 1×1.
fn physical_size(viewport: Vec2, dpi: f32) -> (u32, u32) {
    let scale = if dpi.is_finite() && dpi > 0.0 {
        dpi
    } else {
        1.0
    };
    // Saturating float-to-int casts; NaN becomes 0 and is raised to 1.
    let side = |logical: f32| ((logical * scale).round() as u32).max(1);
    (side(viewport.x), side(viewport.y))
}

/// Renders the whole scene into `target`, in logical pixels.
fn render_into(target: &RenderTarget, editor: &Editor, viewport: Vec2) {
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, viewport.x, viewport.y));
    camera.render_target = Some(target.clone());
    set_camera(&camera);
    draw_scene(editor, viewport);
    set_default_camera();
}

/// Draws background, shapes, gesture overlay, selection and toolbar.
fn draw_scene(editor: &Editor, viewport: Vec2) {
    clear_background(to_mq_color(THEME.bg));
    let camera = editor.camera();
    let overlay = editor.overlay();
    let shapes = editor
        .document()
        .shapes()
        .filter(|(id, _)| !overlay.hidden.contains(id))
        .map(|(_, shape)| shape);
    render::draw_shapes(shapes, camera, viewport);
    for shape in &overlay.shapes {
        render::draw_preview(shape, camera);
    }
    if editor.active_gesture().is_none() {
        if let Some(bounds) = editor.selection_bounds() {
            render::draw_selection(bounds, camera);
        }
    }
    if let Some(marquee) = overlay.marquee {
        let rect = Aabb::from_corners(
            camera.world_to_screen(marquee.min),
            camera.world_to_screen(marquee.max),
        );
        let mut fill = to_mq_color(THEME.selection);
        fill.a *= MARQUEE_FILL_ALPHA;
        draw_rectangle(rect.min.x, rect.min.y, rect.width(), rect.height(), fill);
        draw_rectangle_lines(
            rect.min.x,
            rect.min.y,
            rect.width(),
            rect.height(),
            1.0,
            to_mq_color(THEME.accent),
        );
    }
    if editor.toolbar_visible() {
        toolbar::draw(editor, viewport);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn frame_target_params_uses_sample_count() {
        // Arrange / Act
        let params = frame_target_params(4);

        // Assert
        assert_eq!(params.sample_count, 4);
    }

    #[test]
    fn frame_target_params_has_no_depth() {
        // Arrange / Act
        let params = frame_target_params(DEFAULT_MSAA_SAMPLES);

        // Assert
        assert!(!params.depth);
    }

    #[test]
    fn msaa_samples_unset_is_default() {
        assert_eq!(msaa_samples(None), DEFAULT_MSAA_SAMPLES);
        assert_eq!(DEFAULT_MSAA_SAMPLES, 4);
    }

    #[test]
    fn msaa_samples_valid_counts_are_used() {
        // Arrange
        let cases = [("2", 2), ("4", 4), ("8", 8), (" 8 ", 8)];

        for (setting, expected) in cases {
            // Act
            let samples = msaa_samples(Some(setting));

            // Assert
            assert_eq!(samples, expected, "setting {setting:?}");
        }
    }

    #[test]
    fn msaa_samples_off_values_disable() {
        for setting in ["1", "0", "off", "OFF", " Off\n"] {
            assert_eq!(msaa_samples(Some(setting)), 1, "setting {setting:?}");
        }
    }

    #[test]
    fn msaa_samples_invalid_is_default() {
        for setting in ["", "3", "16", "-4", "four", "4x", "NaN"] {
            assert_eq!(
                msaa_samples(Some(setting)),
                DEFAULT_MSAA_SAMPLES,
                "setting {setting:?}"
            );
        }
    }

    #[test]
    fn frame_filter_is_nearest() {
        assert_eq!(FRAME_FILTER, FilterMode::Nearest);
    }

    #[test]
    fn physical_size_scales_by_dpi() {
        // Arrange / Act
        let size = physical_size(Vec2::new(640.0, 400.0), 2.0);

        // Assert
        assert_eq!(size, (1280, 800));
    }

    #[test]
    fn physical_size_is_at_least_one_pixel() {
        assert_eq!(physical_size(Vec2::ZERO, 1.0), (1, 1));
        assert_eq!(physical_size(Vec2::new(f32::NAN, -5.0), 1.0), (1, 1));
    }

    #[test]
    fn physical_size_bad_dpi_uses_one() {
        for dpi in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                physical_size(Vec2::new(300.0, 200.0), dpi),
                (300, 200),
                "dpi {dpi}"
            );
        }
    }

    proptest! {
        #[test]
        fn msaa_samples_is_always_supported_count(setting in ".*") {
            let samples = msaa_samples(Some(&setting));
            prop_assert!([1, 2, 4, 8].contains(&samples), "{samples}");
        }
    }
}
