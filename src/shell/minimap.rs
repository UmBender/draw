//! The minimap: a cached schematic overview of the document and the panel
//! drawn over the window while panning (ADR-T110-1).
//!
//! [`core::minimap`](crate::core::minimap) owns the geometry and the
//! editor decides when the minimap shows. This module draws it: the
//! overview of the shapes goes into a render target owned by
//! `shell::app`, keyed by [`OverviewKey`], and [`draw_panel`] puts the
//! panel, that texture and the view frame on the window. The camera is
//! not part of the key, so panning never re-renders the overview.

use macroquad::color::Color;
use macroquad::math::vec2;
use macroquad::shapes::draw_rectangle;
use macroquad::texture::{DrawTextureParams, Texture2D, draw_texture_ex};

use crate::core::editor::Editor;
use crate::core::geom::{Aabb, Vec2};
use crate::core::minimap::{Fit, Minimap};
use crate::core::palette::{Rgba, palette};
use crate::core::shape::{Shape, grid_cell_rect};
use crate::shell::render::{Batch, GlSink, hex_color, hex_color_alpha, to_mq_color};
use crate::theme;

/// Content drawn smaller than this, in pixels on its longer side, gets a
/// marker: the scaled-down overview can lose its lines.
pub const MARKER_BELOW_PX: f32 = 12.0;

/// Side of the content marker, in pixels.
pub const MARKER_PX: f32 = 4.0;

/// Width of the overview's lines and of the view frame, in pixels.
pub const LINE_PX: f32 = 1.0;

/// A shape drawn smaller than this, in pixels on its longer side, is a
/// dot.
pub const DOT_BELOW_PX: f32 = 2.0;

/// Opacity of the panel background, of 255.
pub const PANEL_ALPHA: u8 = 217;

/// What the cached overview shows: it is rendered again only when this
/// changes. The camera is not part of it (ADR-T110-1).
#[derive(Debug, Clone, PartialEq)]
pub struct OverviewKey {
    /// [`Editor::document_revision`] of the shapes drawn.
    revision: u64,
    /// Number of shapes drawn.
    len: usize,
    /// Bounds of the shapes drawn.
    content: Aabb,
    /// Size of the render target in physical pixels.
    pub size: (u32, u32),
}

impl OverviewKey {
    /// The key of the overview `map` needs for `editor`'s document at
    /// scale factor `dpi`; `None` when there is nothing to draw.
    #[must_use]
    pub fn of(editor: &Editor, map: &Minimap, dpi: f32) -> Option<Self> {
        let area = map.area();
        Some(Self {
            revision: editor.document_revision(),
            len: editor.document().len(),
            content: map.content()?,
            size: target_size(Vec2::new(area.width(), area.height()), dpi),
        })
    }

    /// The map the overview is rendered through: the content fitted into
    /// a target of the drawing area's size, in logical pixels.
    #[must_use]
    pub fn image(&self, area: Aabb) -> Fit {
        let size = Vec2::new(area.width(), area.height());
        Fit::new(self.content, Aabb::from_corners(Vec2::ZERO, size))
    }
}

/// Whether the overview must be rendered: nothing is cached, or the cache
/// shows something else than `next`.
#[must_use]
pub fn needs_render(cached: Option<&OverviewKey>, next: &OverviewKey) -> bool {
    cached != Some(next)
}

/// The marker drawn over content that takes less than
/// [`MARKER_BELOW_PX`] of the minimap: a square of [`MARKER_PX`] around
/// the content's place, kept inside the drawing area.
#[must_use]
pub fn marker_rect(map: &Minimap) -> Option<Aabb> {
    let rect = map.content_rect()?;
    if rect.width().max(rect.height()) >= MARKER_BELOW_PX {
        return None;
    }
    Some(square_in(rect.center(), MARKER_PX, map.area()))
}

/// A square of `side` pixels centred on `center`, moved to lie in `area`
/// (which is at least `side` wide and high).
fn square_in(center: Vec2, side: f32, area: Aabb) -> Aabb {
    let start = |c: f32, min: f32, max: f32| (c - side * 0.5).clamp(min, (max - side).max(min));
    let min = Vec2::new(
        start(center.x, area.min.x, area.max.x),
        start(center.y, area.min.y, area.max.y),
    );
    Aabb::from_corners(min, min + Vec2::new(side, side))
}

/// Draws every shape of `editor`'s document as a schematic through
/// `image`, into the current render target: lines [`LINE_PX`] wide, fills
/// as areas, text blocks as bars, shapes below [`DOT_BELOW_PX`] as dots.
pub fn draw_overview(editor: &Editor, image: &Fit) {
    let mut batch = Batch::new(GlSink);
    for (_, shape) in editor.document().shapes() {
        if shape.is_finite() {
            draw_schematic(&mut batch, shape, image);
        }
    }
    batch.finish();
}

/// Segments of an ellipse's outline in the overview.
const ELLIPSE_SEGMENTS: u16 = 24;

/// Opacity of the bar a text block is drawn as, of 255.
const TEXT_BAR_ALPHA: u8 = 140;

/// Adds the schematic of one finite `shape` to `batch`, through `image`.
fn draw_schematic(batch: &mut Batch, shape: &Shape, image: &Fit) {
    let color = to_mq_color(palette(shape.style().color));
    let bounds = image.rect_of(shape.bounds());
    if bounds.width().max(bounds.height()) < DOT_BELOW_PX {
        let half = Vec2::new(DOT_BELOW_PX * 0.5, DOT_BELOW_PX * 0.5);
        let center = bounds.center();
        batch.rect(Aabb::from_corners(center - half, center + half), color);
        return;
    }
    match shape {
        Shape::Stroke { points, .. } => {
            batch.polyline(points.iter().map(|p| image.point(*p)), LINE_PX, color);
        }
        Shape::Line { a, b, .. } | Shape::Arrow { a, b, .. } => {
            batch.line(image.point(*a), image.point(*b), LINE_PX, color);
        }
        Shape::Rect { a, b, fill, .. } => {
            let rect = Aabb::from_corners(image.point(*a), image.point(*b));
            if let Some(fill) = fill {
                batch.rect(rect, to_mq_color(palette(*fill)));
            }
            outline(batch, rect, color);
        }
        Shape::Ellipse { a, b, fill, .. } => {
            let rect = Aabb::from_corners(image.point(*a), image.point(*b));
            let (center, radii) = (rect.center(), (rect.max - rect.min) * 0.5);
            let rim: Vec<Vec2> = (0..=ELLIPSE_SEGMENTS)
                .map(|i| {
                    let angle = std::f32::consts::TAU * f32::from(i) / f32::from(ELLIPSE_SEGMENTS);
                    center + Vec2::new(radii.x * angle.cos(), radii.y * angle.sin())
                })
                .collect();
            if let Some(fill) = fill {
                batch.fan(center, rim.iter().copied(), to_mq_color(palette(*fill)));
            }
            batch.polyline(rim, LINE_PX, color);
        }
        Shape::Grid {
            a, b, cols, rows, ..
        } => {
            let (a, b) = (image.point(*a), image.point(*b));
            for fill in shape.cell_fills() {
                let cell = grid_cell_rect(a, b, *cols, *rows, fill.col, fill.row);
                batch.rect(cell, to_mq_color(palette(fill.color)));
            }
            outline(batch, Aabb::from_corners(a, b), color);
        }
        Shape::Text { .. } => {
            let bar = palette(shape.style().color).faded(TEXT_BAR_ALPHA);
            batch.rect(bounds, to_mq_color(bar));
        }
    }
}

/// Adds the outline of `rect` to `batch`, [`LINE_PX`] wide.
fn outline(batch: &mut Batch, rect: Aabb, color: Color) {
    let (min, max) = (rect.min, rect.max);
    let corners = [
        min,
        Vec2::new(max.x, min.y),
        max,
        Vec2::new(min.x, max.y),
        min,
    ];
    batch.polyline(corners, LINE_PX, color);
}

/// Draws the border of `rect` on the window as four bars `width` pixels
/// wide, inside the rectangle.
fn draw_border(rect: Aabb, width: f32, color: Color) {
    let (min, size) = (rect.min, rect.max - rect.min);
    draw_rectangle(min.x, min.y, size.x, width, color);
    draw_rectangle(min.x, rect.max.y - width, size.x, width, color);
    draw_rectangle(min.x, min.y, width, size.y, color);
    draw_rectangle(rect.max.x - width, min.y, width, size.y, color);
}

/// Draws the minimap on the window: the panel, the cached `overview`
/// (its texture and the map it was rendered through), the content marker
/// and the frame of the visible world rectangle `view`.
pub fn draw_panel(map: &Minimap, view: Aabb, overview: Option<(&Texture2D, &Fit)>) {
    let panel = map.panel();
    let background = hex_color_alpha(theme::TOOLBAR_BACKGROUND, PANEL_ALPHA);
    draw_rectangle(
        panel.min.x,
        panel.min.y,
        panel.width(),
        panel.height(),
        background,
    );
    draw_border(panel, LINE_PX, hex_color(theme::TOOLBAR_BORDER));
    if let Some((texture, image)) = overview
        && let Some(dest) = map.place(image)
    {
        draw_texture_ex(
            texture,
            dest.min.x,
            dest.min.y,
            to_mq_color(Rgba::NO_TINT),
            DrawTextureParams {
                dest_size: Some(vec2(dest.width(), dest.height())),
                // GL textures start at the bottom row.
                flip_y: true,
                ..DrawTextureParams::default()
            },
        );
    }
    if let Some(marker) = marker_rect(map) {
        draw_rectangle(
            marker.min.x,
            marker.min.y,
            marker.width(),
            marker.height(),
            hex_color(theme::TOOLBAR_TEXT),
        );
    }
    draw_border(
        map.frame(view),
        LINE_PX,
        hex_color(theme::SELECTION_OUTLINE),
    );
}

/// The colour an overview target is cleared to: the panel background,
/// fully transparent, so the image's empty margins show the panel.
#[must_use]
pub fn clear_color() -> Color {
    hex_color_alpha(theme::TOOLBAR_BACKGROUND, 0)
}

/// Size in physical pixels of a target covering `size` logical pixels at
/// scale factor `dpi`, at least 1 × 1; a bad `dpi` counts as 1.
#[must_use]
pub fn target_size(size: Vec2, dpi: f32) -> (u32, u32) {
    let scale = if dpi.is_finite() && dpi > 0.0 {
        dpi
    } else {
        1.0
    };
    // Saturating float-to-int casts; NaN becomes 0 and is raised to 1.
    let pixels = |logical: f32| ((logical * scale).round() as u32).max(1);
    (pixels(size.x), pixels(size.y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};

    const VIEWPORT: Vec2 = Vec2::new(800.0, 600.0);

    #[test]
    fn border_bars_cover_the_rect_edges() {
        // Arrange
        let rect = Aabb::from_corners(Vec2::new(10.0, 20.0), Vec2::new(110.0, 70.0));

        // Act
        let [top, bottom, left, right] = border_rects(rect, 2.0);

        // Assert
        assert_eq!((top.min, top.max), (Vec2::new(10.0, 20.0), Vec2::new(110.0, 22.0)));
        assert_eq!((bottom.min, bottom.max), (Vec2::new(10.0, 68.0), Vec2::new(110.0, 70.0)));
        assert_eq!((left.min, left.max), (Vec2::new(10.0, 20.0), Vec2::new(12.0, 70.0)));
        assert_eq!((right.min, right.max), (Vec2::new(108.0, 20.0), Vec2::new(110.0, 70.0)));
    }

    #[test]
    fn minimap_and_frame_cache_use_no_macroquad_draw_calls() {
        // Arrange: the code of both files, without their test modules.
        let code = |source: &'static str| source.split("#[cfg(test)]").next().unwrap_or("");
        let minimap = code(include_str!("minimap.rs"));
        let app = code(include_str!("app.rs"));
        let forbidden = [
            "draw_rectangle(",
            "draw_texture_ex(",
            "render_target_ex(",
            "set_camera(",
            "get_texture_data(",
            ".set_filter(",
            "clear_background(",
        ];

        // Act / Assert
        for call in forbidden {
            assert!(!minimap.contains(call), "minimap.rs uses {call}");
            // The overlay (T160) still draws its marquee with rectangles.
            if call != "draw_rectangle(" {
                assert!(!app.contains(call), "app.rs uses {call}");
            }
        }
    }

    const CTRL: Modifiers = Modifiers {
        shift: false,
        ctrl: true,
        alt: false,
    };

    fn key(editor: &mut Editor, key: Key, mods: Modifiers) {
        editor.handle(InputEvent::KeyDown { key, mods });
    }

    fn drag(editor: &mut Editor, from: Vec2, to: Vec2) {
        let (button, mods) = (PointerButton::Left, Modifiers::NONE);
        editor.handle(InputEvent::PointerDown {
            pos: from,
            button,
            mods,
        });
        editor.handle(InputEvent::PointerMove { pos: to, mods });
        editor.handle(InputEvent::PointerUp {
            pos: to,
            button,
            mods,
        });
    }

    /// An editor with an 800 × 600 viewport, one pen stroke and `Space`
    /// held.
    fn panning_editor() -> Editor {
        let mut editor = Editor::new();
        editor.handle(InputEvent::Resize { size: VIEWPORT });
        drag(
            &mut editor,
            Vec2::new(100.0, 100.0),
            Vec2::new(300.0, 250.0),
        );
        key(&mut editor, Key::Space, Modifiers::NONE);
        editor
    }

    fn key_of(editor: &Editor, dpi: f32) -> Option<OverviewKey> {
        let Some(map) = editor.minimap() else {
            panic!("the minimap shows");
        };
        OverviewKey::of(editor, &map, dpi)
    }

    fn boxed(x0: f32, y0: f32, x1: f32, y1: f32) -> Aabb {
        Aabb::from_corners(Vec2::new(x0, y0), Vec2::new(x1, y1))
    }

    // ---- AC-3: the overview is re-rendered only for a new document ----

    #[test]
    fn overview_key_ignores_the_camera() {
        // Arrange
        let mut editor = panning_editor();
        let before = key_of(&editor, 1.0);
        let camera = *editor.camera();

        // Act: pan with `Space` held, then zoom.
        drag(&mut editor, Vec2::new(400.0, 300.0), Vec2::new(150.0, 80.0));
        editor.handle(InputEvent::Scroll {
            pos: Vec2::new(10.0, 10.0),
            delta: -2.0,
        });

        // Assert
        assert_ne!(*editor.camera(), camera);
        assert!(before.is_some());
        assert_eq!(key_of(&editor, 1.0), before);
    }

    #[test]
    fn overview_key_changes_with_the_document() {
        // Arrange
        let mut editor = panning_editor();
        let before = key_of(&editor, 1.0);

        // Act: undo the stroke with `Space` still held, then redo it.
        key(&mut editor, Key::Z, CTRL);
        let undone = key_of(&editor, 1.0);
        key(&mut editor, Key::Y, CTRL);
        let redone = key_of(&editor, 1.0);

        // Assert: an empty document has nothing to render.
        assert!(undone.is_none());
        assert!(redone.is_some());
        assert_ne!(redone, before, "a new revision is a new key");
    }

    #[test]
    fn overview_key_changes_with_the_target_size() {
        // Arrange
        let editor = panning_editor();

        // Act
        let (one, two) = (key_of(&editor, 1.0), key_of(&editor, 2.0));

        // Assert
        assert_ne!(one, two);
        assert_eq!(one.map(|key| key.size), Some((188, 138)));
        assert_eq!(two.map(|key| key.size), Some((376, 276)));
        assert_eq!(
            key_of(&editor, f32::NAN).map(|key| key.size),
            Some((188, 138))
        );
    }

    #[test]
    fn needs_render_only_for_a_new_key() {
        // Arrange
        let editor = panning_editor();
        let (Some(key), Some(other)) = (key_of(&editor, 1.0), key_of(&editor, 2.0)) else {
            panic!("the document has content");
        };

        // Act / Assert
        assert!(needs_render(None, &key));
        assert!(!needs_render(Some(&key), &key));
        assert!(needs_render(Some(&other), &key));
    }

    #[test]
    fn marker_only_for_small_content() {
        // Arrange: content 100 units wide in views of 800 and 8000 units.
        let content = boxed(0.0, 0.0, 100.0, 100.0);
        let near = Minimap::new(VIEWPORT, Some(content), boxed(0.0, 0.0, 800.0, 600.0));
        let far = Minimap::new(VIEWPORT, Some(content), boxed(0.0, 0.0, 8000.0, 6000.0));
        let empty = Minimap::new(VIEWPORT, None, boxed(0.0, 0.0, 800.0, 600.0));
        let (Some(near), Some(far), Some(empty)) = (near, far, empty) else {
            panic!("an 800 x 600 viewport holds the panel");
        };

        // Act
        let (near_marker, far_marker) = (marker_rect(&near), marker_rect(&far));

        // Assert: 23 px of content needs no marker, 2.3 px does.
        assert!(near_marker.is_none());
        assert!(marker_rect(&empty).is_none());
        let Some(marker) = far_marker else {
            panic!("small content gets a marker");
        };
        assert!(approx_eq(marker.width(), MARKER_PX, 1e-3), "{marker:?}");
        assert!(approx_eq(marker.height(), MARKER_PX, 1e-3), "{marker:?}");
        let area = far.area().expand(1e-3);
        assert!(area.contains(marker.min) && area.contains(marker.max));
        let Some(rect) = far.content_rect() else {
            panic!("there is content");
        };
        assert!(marker.expand(1e-3).contains(rect.center()));
    }
}
