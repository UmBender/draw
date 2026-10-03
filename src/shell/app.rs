//! Window configuration and the main loop.
//!
//! The loop follows ADR-T12-1: miniquad's blocking event loop sleeps until
//! input arrives, raw events are replayed into an [`Collector`],
//! routed past the toolbar into the [`Editor`]. The document is cached in a
//! render target that is re-rendered (or appended to) only when its
//! [`LayerKey`] changes (ADR-T24-1, ADR-T24-3); every woken frame blits it
//! and draws the overlay on top (ADR-T24-2).

use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::WHITE;
use macroquad::conf::{Conf, UpdateTrigger};
use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::math::{Rect, vec2};
use macroquad::miniquad::conf::{Conf as WindowConf, Platform};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::texture::{
    DrawTextureParams, FilterMode, RenderTarget, RenderTargetParams, Texture2D, draw_texture_ex,
    render_target_ex,
};
use macroquad::window::{
    clear_background, next_frame, screen_dpi_scale, screen_height, screen_width,
};

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::core::camera::Camera;
use crate::core::command::Tool;
use crate::core::document::ShapeId;
use crate::core::editor::Editor;
use crate::core::geom::{Aabb, Vec2};
use crate::core::input::InputEvent;
use crate::core::palette::THEME;
use crate::core::tools::Overlay;
use crate::shell::input_map::Collector;
use crate::shell::render::{self, to_mq_color};
use crate::shell::toolbar::{self, Route};

/// Opacity of the marquee fill.
const MARQUEE_FILL_ALPHA: f32 = 0.25;

/// Viewport sizes closer than this many pixels count as unchanged.
const RESIZE_EPS_PX: f32 = 0.5;

/// MSAA samples per pixel for the cached frame unless `DRAW_MSAA` says
/// otherwise (ADR-T15-1).
pub const DEFAULT_MSAA_SAMPLES: i32 = 4;

/// Environment variable overriding [`DEFAULT_MSAA_SAMPLES`].
pub const MSAA_ENV: &str = "DRAW_MSAA";

/// Environment variable that turns on the frame timer (T24 AC-1).
pub const FRAME_TIMES_ENV: &str = "DRAW_FRAME_TIMES";

/// Filter of the resolved frame texture: the blit is 1:1 in physical pixels,
/// so nearest keeps it sharp.
const FRAME_FILTER: FilterMode = FilterMode::Nearest;

/// Sample count for a `DRAW_MSAA` value: `1`, `0` or `off` disable
/// multisampling, `2`, `4` or `8` select that count, anything else (or unset)
/// gives [`DEFAULT_MSAA_SAMPLES`].
#[must_use]
pub fn msaa_samples(setting: Option<&str>) -> i32 {
    let Some(setting) = setting else {
        return DEFAULT_MSAA_SAMPLES;
    };
    match setting.trim().to_ascii_lowercase().as_str() {
        "0" | "1" | "off" => 1,
        "2" => 2,
        "4" => 4,
        "8" => 8,
        _ => DEFAULT_MSAA_SAMPLES,
    }
}

/// Whether a `DRAW_FRAME_TIMES` value turns the frame timer on: `1`, `on`,
/// `true` or `yes`, in any case and trimmed. Unset or anything else is off.
#[must_use]
pub fn frame_timing_enabled(setting: Option<&str>) -> bool {
    setting.is_some_and(|setting| {
        matches!(
            setting.trim().to_ascii_lowercase().as_str(),
            "1" | "on" | "true" | "yes"
        )
    })
}

/// The frame timer's line for one re-render: which layers, CPU time in
/// milliseconds and the number of shapes in the document.
#[must_use]
pub fn frame_log_line(redraw: Redraw, elapsed: Duration, shapes: usize) -> String {
    let layers = match redraw {
        Redraw::None => "no",
        Redraw::Overlay => "overlay",
        Redraw::Append { .. } => "append",
        Redraw::Full => "full",
    };
    let ms = elapsed.as_secs_f64() * 1000.0;
    format!("draw: {layers} re-render {ms:.3} ms, {shapes} shapes")
}

/// What the document layer shows (ADR-T24-1); it is re-rendered when this
/// changes.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerKey {
    /// [`Editor::document_revision`].
    pub revision: u64,
    /// Number of shapes in the document.
    pub len: usize,
    /// The view.
    pub camera: Camera,
    /// Framebuffer size in physical pixels.
    pub size: (u32, u32),
    /// Shapes the gesture in progress hides (`Overlay::hidden`).
    pub hidden: Vec<ShapeId>,
    /// Whether the underlay dot grid is drawn (grid snap on).
    pub underlay: bool,
}

/// What a frame re-renders before the blit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Redraw {
    /// Nothing: blit the cached frame.
    None,
    /// The frame only: cached document layer plus the overlay.
    Overlay,
    /// Only the shapes from index `from` on, drawn onto the cached document
    /// layer (ADR-T24-3).
    Append {
        /// First new shape, in z-order.
        from: usize,
    },
    /// Both layers.
    Full,
}

/// Picks the work for a frame from the `cached` document layer's key, the
/// `next` key, [`Editor::document_base_revision`] (`base`) and whether
/// input changed editor state (`dirty`).
#[must_use]
pub fn plan_redraw(cached: Option<&LayerKey>, next: &LayerKey, base: u64, dirty: bool) -> Redraw {
    let Some(cached) = cached else {
        return Redraw::Full;
    };
    if cached == next {
        return if dirty { Redraw::Overlay } else { Redraw::None };
    }
    let same_view = cached.camera == next.camera
        && cached.size == next.size
        && cached.hidden == next.hidden
        && cached.underlay == next.underlay;
    let appended = next.revision > cached.revision && next.len > cached.len;
    if same_view && appended && base <= cached.revision {
        Redraw::Append { from: cached.len }
    } else {
        Redraw::Full
    }
}

/// Render-target parameters for the cached frame: `samples` per pixel and no
/// depth buffer.
fn frame_target_params(samples: i32) -> RenderTargetParams {
    RenderTargetParams {
        sample_count: samples,
        depth: false,
    }
}

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
    let setting = std::env::var(MSAA_ENV).ok();
    let timed = frame_timing_enabled(std::env::var(FRAME_TIMES_ENV).ok().as_deref());
    let mut frame = CachedFrame::new(msaa_samples(setting.as_deref()), timed);
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
    let flyout = editor.tool() == Tool::Grid;
    match toolbar::route(editor.toolbar_visible(), flyout, editor.viewport(), event) {
        Route::Forward(event) => editor.handle(event),
        Route::Apply(command) => editor.apply(command),
        Route::Swallow => false,
    }
}

/// The document layer: background, underlay and document shapes in a
/// framebuffer-sized, multisampled render target that macroquad resolves
/// into a plain texture (ADR-T15-1, ADR-T24-1). The overlay is drawn on the
/// window every frame (ADR-T24-2).
struct CachedFrame {
    /// The document layer.
    document: Option<RenderTarget>,
    /// What `document` shows.
    key: Option<LayerKey>,
    /// MSAA samples per pixel for new targets.
    samples: i32,
    /// Whether every re-render logs its time (`DRAW_FRAME_TIMES`).
    timed: bool,
}

impl CachedFrame {
    /// An empty cache whose targets use `samples` per pixel; `timed` turns
    /// on the frame timer.
    fn new(samples: i32, timed: bool) -> Self {
        Self {
            document: None,
            key: None,
            samples,
            timed,
        }
    }

    /// Updates the document layer as [`plan_redraw`] says, then blits it to
    /// the window and draws the overlay on top. `dirty` says whether input
    /// changed editor state.
    fn present(&mut self, editor: &Editor, viewport: Vec2, dpi: f32, dirty: bool) {
        let size = physical_size(viewport, dpi);
        let overlay = editor.overlay();
        let key = LayerKey {
            revision: editor.document_revision(),
            len: editor.document().len(),
            camera: *editor.camera(),
            size,
            hidden: overlay.hidden.clone(),
            underlay: editor.helpers().grid_snap,
        };
        let base = editor.document_base_revision();
        let redraw = plan_redraw(self.key.as_ref(), &key, base, dirty);
        let started = self.timed.then(Instant::now);
        let from = match redraw {
            Redraw::Full => Some(0),
            Redraw::Append { from } => Some(from),
            Redraw::Overlay | Redraw::None => None,
        };
        if let Some(from) = from {
            if self.key.as_ref().is_none_or(|cached| cached.size != size) {
                self.document = Some(self.new_target(size));
            }
            if let Some(document) = &self.document {
                render_into(document, viewport, || {
                    draw_document(editor, &overlay, viewport, from);
                });
            }
            self.key = Some(key);
        }
        let Some(document) = &self.document else {
            return;
        };
        blit(&document.texture, viewport);
        draw_overlay(editor, &overlay, viewport);
        if let Some(started) = started.filter(|_| redraw != Redraw::None) {
            let shapes = editor.document().len();
            eprintln!("{}", frame_log_line(redraw, started.elapsed(), shapes));
        }
    }

    /// A render target of `size` physical pixels with this cache's sample
    /// count.
    fn new_target(&self, size: (u32, u32)) -> RenderTarget {
        let target = render_target_ex(size.0, size.1, frame_target_params(self.samples));
        target.texture.set_filter(FRAME_FILTER);
        target
    }
}

/// Draws a resolved layer `texture` over the whole `viewport`, 1:1 in
/// physical pixels.
fn blit(texture: &Texture2D, viewport: Vec2) {
    draw_texture_ex(
        texture,
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

/// Runs `draw` with `target` as the destination, in logical pixels.
fn render_into(target: &RenderTarget, viewport: Vec2, draw: impl FnOnce()) {
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, viewport.x, viewport.y));
    camera.render_target = Some(target.clone());
    set_camera(&camera);
    draw();
    set_default_camera();
}

/// Draws the document layer: background, underlay and every shape the
/// gesture in progress does not hide. With `from > 0` only the shapes from
/// that index on are drawn over what the layer already holds (ADR-T24-3).
fn draw_document(editor: &Editor, overlay: &Overlay, viewport: Vec2, from: usize) {
    if from == 0 {
        clear_background(to_mq_color(THEME.bg));
        render::draw_underlay(editor, viewport);
    }
    let hidden: HashSet<ShapeId> = overlay.hidden.iter().copied().collect();
    let shapes = editor
        .document()
        .shapes()
        .skip(from)
        .filter(|(id, _)| !hidden.contains(id))
        .map(|(_, shape)| shape);
    render::draw_shapes(shapes, editor.camera(), viewport);
}

/// Draws what lies above the document: gesture overlay, guides, selection,
/// marquee and toolbar.
fn draw_overlay(editor: &Editor, overlay: &Overlay, viewport: Vec2) {
    let camera = editor.camera();
    for shape in &overlay.shapes {
        render::draw_preview(shape, camera);
    }
    render::draw_guides(&overlay.guides, camera);
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
    use crate::core::camera::{ZOOM_MAX, ZOOM_MIN};
    use crate::core::geom::approx_eq;
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

    // ---- T24 AC-1 frame timer ---------------------------------------------

    #[test]
    fn frame_timing_enabled_unset_is_off() {
        assert!(!frame_timing_enabled(None));
    }

    #[test]
    fn frame_timing_enabled_on_values_enable() {
        for setting in ["1", "on", "ON", "true", "Yes", " yes\n"] {
            assert!(frame_timing_enabled(Some(setting)), "setting {setting:?}");
        }
    }

    #[test]
    fn frame_timing_enabled_other_values_disable() {
        for setting in ["", "0", "off", "false", "no", "2", "enable"] {
            assert!(!frame_timing_enabled(Some(setting)), "setting {setting:?}");
        }
    }

    #[test]
    fn frame_log_line_names_layers_time_and_shapes() {
        // Arrange
        let elapsed = Duration::from_micros(12_345);

        // Act
        let full = frame_log_line(Redraw::Full, elapsed, 5120);
        let overlay = frame_log_line(Redraw::Overlay, Duration::from_micros(800), 7);

        // Assert
        assert_eq!(full, "draw: full re-render 12.345 ms, 5120 shapes");
        assert_eq!(overlay, "draw: overlay re-render 0.800 ms, 7 shapes");
    }

    // ---- T24 AC-2 layer planning -------------------------------------------

    fn key() -> LayerKey {
        LayerKey {
            revision: 3,
            len: 10,
            camera: Camera::default(),
            size: (800, 600),
            hidden: vec![ShapeId(1)],
            underlay: false,
        }
    }

    #[test]
    fn plan_redraw_without_cache_is_full() {
        assert_eq!(plan_redraw(None, &key(), 0, false), Redraw::Full);
        assert_eq!(plan_redraw(None, &key(), 0, true), Redraw::Full);
    }

    #[test]
    fn plan_redraw_same_key_clean_is_none() {
        assert_eq!(plan_redraw(Some(&key()), &key(), 0, false), Redraw::None);
    }

    #[test]
    fn plan_redraw_same_key_dirty_is_overlay() {
        assert_eq!(plan_redraw(Some(&key()), &key(), 0, true), Redraw::Overlay);
    }

    #[test]
    fn plan_redraw_changed_key_is_full() {
        // Arrange
        let changed = [
            LayerKey {
                revision: 4,
                ..key()
            },
            LayerKey {
                size: (801, 600),
                ..key()
            },
            LayerKey {
                hidden: vec![ShapeId(1), ShapeId(2)],
                ..key()
            },
            LayerKey {
                hidden: Vec::new(),
                ..key()
            },
            LayerKey {
                underlay: true,
                ..key()
            },
        ];

        for next in changed {
            for dirty in [false, true] {
                // Act
                let redraw = plan_redraw(Some(&key()), &next, 0, dirty);

                // Assert
                assert_eq!(redraw, Redraw::Full, "{next:?} dirty {dirty}");
            }
        }
    }

    // ---- T24 AC-3c append-only updates -------------------------------------

    fn appended() -> LayerKey {
        LayerKey {
            revision: 5,
            len: 12,
            ..key()
        }
    }

    #[test]
    fn plan_redraw_appended_shapes_only_is_append() {
        for dirty in [false, true] {
            assert_eq!(
                plan_redraw(Some(&key()), &appended(), 2, dirty),
                Redraw::Append { from: 10 },
                "dirty {dirty}"
            );
        }
        assert_eq!(
            plan_redraw(Some(&key()), &appended(), 3, true),
            Redraw::Append { from: 10 }
        );
    }

    #[test]
    fn plan_redraw_append_after_rewrite_is_full() {
        assert_eq!(
            plan_redraw(Some(&key()), &appended(), 4, true),
            Redraw::Full
        );
    }

    #[test]
    fn plan_redraw_append_with_other_change_is_full() {
        // Arrange
        let mut panned = Camera::default();
        panned.pan_by_screen(Vec2::new(10.0, 0.0));
        let changed = [
            LayerKey {
                camera: panned,
                ..appended()
            },
            LayerKey {
                size: (801, 600),
                ..appended()
            },
            LayerKey {
                hidden: Vec::new(),
                ..appended()
            },
            LayerKey {
                underlay: true,
                ..appended()
            },
        ];

        for next in changed {
            // Act / Assert
            assert_eq!(
                plan_redraw(Some(&key()), &next, 0, true),
                Redraw::Full,
                "{next:?}"
            );
        }
    }

    #[test]
    fn plan_redraw_shorter_document_is_full() {
        let next = LayerKey {
            revision: 4,
            len: 8,
            ..key()
        };

        assert_eq!(plan_redraw(Some(&key()), &next, 0, true), Redraw::Full);
    }

    #[test]
    fn frame_log_line_names_appends() {
        let line = frame_log_line(Redraw::Append { from: 3 }, Duration::from_micros(250), 4);

        assert_eq!(line, "draw: append re-render 0.250 ms, 4 shapes");
    }

    // ---- T26 AC-1 reuse during a gesture -----------------------------------

    fn panned_camera() -> Camera {
        let mut camera = Camera::default();
        camera.pan_by_screen(Vec2::new(10.0, 0.0));
        camera
    }

    fn assert_rect_eq(actual: Aabb, expected: Aabb) {
        assert!(
            actual.min.approx_eq(expected.min, 1e-3) && actual.max.approx_eq(expected.max, 1e-3),
            "{actual:?} != {expected:?}"
        );
    }

    #[test]
    fn plan_redraw_camera_only_is_moved() {
        // Arrange
        let next = LayerKey {
            camera: panned_camera(),
            ..key()
        };

        for dirty in [false, true] {
            // Act
            let redraw = plan_redraw(Some(&key()), &next, 0, dirty);

            // Assert
            assert_eq!(redraw, Redraw::Moved, "dirty {dirty}");
        }
    }

    #[test]
    fn blit_rect_same_camera_is_layer() {
        // Arrange
        let camera = Camera::new(Vec2::new(-30.0, 12.5), 2.5);
        let layer = Aabb::from_corners(Vec2::new(-100.0, -100.0), Vec2::new(900.0, 700.0));

        // Act
        let rect = blit_rect(&camera, &camera, layer);

        // Assert
        assert_rect_eq(rect.expect("finite"), layer);
    }

    #[test]
    fn blit_rect_pan_moves_by_screen_delta() {
        // Arrange
        let cached = Camera::new(Vec2::ZERO, 2.0);
        let mut current = cached;
        current.pan_by_screen(Vec2::new(40.0, -25.0));
        let layer = Aabb::from_corners(Vec2::new(-50.0, -50.0), Vec2::new(850.0, 650.0));

        // Act
        let rect = blit_rect(&cached, &current, layer);

        // Assert
        assert_rect_eq(
            rect.expect("finite"),
            layer.translate(Vec2::new(40.0, -25.0)),
        );
    }

    #[test]
    fn blit_rect_zoom_scales_about_anchor() {
        // Arrange
        let cached = Camera::default();
        let mut current = cached;
        let anchor = Vec2::new(200.0, 100.0);
        current.zoom_at(anchor, 1.0);
        let k = current.zoom() / cached.zoom();
        let layer = Aabb::from_corners(Vec2::ZERO, Vec2::new(800.0, 600.0));

        // Act
        let rect = blit_rect(&cached, &current, layer).expect("finite");

        // Assert
        let expected = Aabb::from_corners(anchor - anchor * k, anchor + (layer.max - anchor) * k);
        assert_rect_eq(rect, expected);
        assert!(approx_eq(rect.width(), 800.0 * k, 1e-3));
    }

    #[test]
    fn blit_filter_moved_is_linear() {
        assert_eq!(blit_filter(Redraw::Moved), FilterMode::Linear);
    }

    #[test]
    fn blit_filter_at_rest_is_nearest() {
        for redraw in [
            Redraw::None,
            Redraw::Overlay,
            Redraw::Append { from: 2 },
            Redraw::Full,
        ] {
            assert_eq!(blit_filter(redraw), FRAME_FILTER, "{redraw:?}");
        }
    }

    #[test]
    fn frame_log_line_names_moved() {
        let line = frame_log_line(Redraw::Moved, Duration::from_micros(90), 5120);

        assert_eq!(line, "draw: moved re-render 0.090 ms, 5120 shapes");
    }

    // ---- T26 AC-2 settle ----------------------------------------------------

    #[test]
    fn settle_period_is_100_ms() {
        assert_eq!(SETTLE, Duration::from_millis(100));
    }

    #[test]
    fn settle_moved_before_quiet_period_stays_moved() {
        for quiet in [Duration::ZERO, Duration::from_millis(99)] {
            assert_eq!(settle(Redraw::Moved, quiet), Redraw::Moved, "{quiet:?}");
        }
    }

    #[test]
    fn settle_moved_after_quiet_period_is_full() {
        for quiet in [SETTLE, Duration::from_secs(5)] {
            assert_eq!(settle(Redraw::Moved, quiet), Redraw::Full, "{quiet:?}");
        }
    }

    #[test]
    fn settle_other_redraws_are_unchanged() {
        for redraw in [
            Redraw::None,
            Redraw::Overlay,
            Redraw::Append { from: 4 },
            Redraw::Full,
        ] {
            for quiet in [Duration::ZERO, SETTLE] {
                assert_eq!(settle(redraw, quiet), redraw, "{redraw:?} {quiet:?}");
            }
        }
    }

    // ---- T26 AC-3 margin ----------------------------------------------------

    #[test]
    fn layer_margin_is_eighth_of_longer_side() {
        assert_eq!(layer_margin((2560, 1600)), 320);
        assert_eq!(layer_margin((1280, 800)), 160);
        assert_eq!(layer_margin((600, 1000)), 125);
        assert_eq!(layer_margin((1, 1)), 0);
    }

    #[test]
    fn layer_margin_keeps_layer_within_max_side() {
        // Arrange
        let cases = [(7680, 4320), (8000, 100), (8192, 8192), (20_000, 10)];

        for size in cases {
            // Act
            let margin = layer_margin(size);
            let layer = layer_size(size, margin);

            // Assert
            let longest = size.0.max(size.1);
            assert!(
                layer.0.max(layer.1) <= MAX_LAYER_SIDE.max(longest),
                "{size:?} margin {margin}"
            );
        }
        assert_eq!(layer_margin((7680, 4320)), 256);
        assert_eq!(layer_margin((8192, 8192)), 0);
    }

    #[test]
    fn layer_size_adds_margin_on_both_sides() {
        assert_eq!(layer_size((2560, 1600), 320), (3200, 2240));
        assert_eq!(layer_size((1, 1), 0), (1, 1));
        assert_eq!(layer_size((u32::MAX, 5), 10), (u32::MAX, 25));
    }

    #[test]
    fn layer_camera_shows_window_origin_at_margin() {
        // Arrange
        let camera = Camera::new(Vec2::new(17.0, -4.0), 2.5);
        let margin = 64.0;

        // Act
        let shifted = layer_camera(&camera, margin);

        // Assert
        let world = camera.screen_to_world(Vec2::ZERO);
        assert!(
            shifted
                .world_to_screen(world)
                .approx_eq(Vec2::new(margin, margin), 1e-3)
        );
        assert!(approx_eq(shifted.zoom(), camera.zoom(), 1e-6));
    }

    // ---- T26 AC-4 any other change wins -------------------------------------

    #[test]
    fn plan_redraw_camera_and_other_change_is_full() {
        // Arrange
        let camera = panned_camera();
        let changed = [
            LayerKey {
                camera,
                revision: 4,
                ..key()
            },
            LayerKey {
                camera,
                size: (801, 600),
                ..key()
            },
            LayerKey {
                camera,
                hidden: Vec::new(),
                ..key()
            },
            LayerKey {
                camera,
                underlay: true,
                ..key()
            },
        ];

        for next in changed {
            for dirty in [false, true] {
                // Act
                let redraw = plan_redraw(Some(&key()), &next, 0, dirty);

                // Assert
                assert_eq!(redraw, Redraw::Full, "{next:?} dirty {dirty}");
            }
        }
    }

    proptest! {
        #[test]
        fn msaa_samples_is_always_supported_count(setting in ".*") {
            let samples = msaa_samples(Some(&setting));
            prop_assert!([1, 2, 4, 8].contains(&samples), "{samples}");
        }

        #[test]
        fn blit_rect_maps_corners_through_cameras(
            ox in -1e4f32..1e4, oy in -1e4f32..1e4, z0 in ZOOM_MIN..ZOOM_MAX,
            px in -1e4f32..1e4, py in -1e4f32..1e4, z1 in ZOOM_MIN..ZOOM_MAX,
            w in 1.0f32..4000.0, h in 1.0f32..4000.0, m in 0.0f32..500.0,
        ) {
            // Arrange
            let cached = Camera::new(Vec2::new(ox, oy), z0);
            let current = Camera::new(Vec2::new(px, py), z1);
            let layer = Aabb::from_corners(Vec2::new(-m, -m), Vec2::new(w + m, h + m));

            // Act
            let rect = blit_rect(&cached, &current, layer);

            // Assert
            let rect = rect.expect("finite cameras give a finite rect");
            let corner = current.world_to_screen(cached.screen_to_world(layer.min));
            let tol = 1e-3 * (1.0 + corner.x.abs().max(corner.y.abs()));
            prop_assert!(rect.min.approx_eq(corner, tol), "{rect:?} {corner:?}");
            let scale = z1 / z0;
            let width_tol = 1e-3 * (1.0 + layer.width() * scale);
            prop_assert!(approx_eq(rect.width(), layer.width() * scale, width_tol));
        }
    }
}
