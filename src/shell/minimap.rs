//! The minimap: a cached schematic overview of the document and the panel
//! drawn over the window while panning (ADR-T110-1).
//!
//! [`core::minimap`](crate::core::minimap) owns the geometry and the
//! editor decides when the minimap shows. This module draws it: the
//! overview of the shapes goes into a render target owned by
//! `shell::app`, keyed by [`OverviewKey`], and [`draw_panel`] puts the
//! panel, that texture and the view frame on the window. The camera is
//! not part of the key, so panning never re-renders the overview.

use macroquad::texture::Texture2D;

use crate::core::editor::Editor;
use crate::core::geom::{Aabb, Vec2};
use crate::core::minimap::{Fit, Minimap};

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
        todo!("T110 behaviour: {} {map:?} {dpi}", editor.document().len())
    }

    /// The map the overview is rendered through: the content fitted into
    /// a target of the drawing area's size, in logical pixels.
    #[must_use]
    pub fn image(&self, area: Aabb) -> Fit {
        todo!("T110 behaviour: {:?} {area:?}", self.content)
    }
}

/// Whether the overview must be rendered: nothing is cached, or the cache
/// shows something else than `next`.
#[must_use]
pub fn needs_render(cached: Option<&OverviewKey>, next: &OverviewKey) -> bool {
    todo!("T110 behaviour: {cached:?} {next:?}")
}

/// The marker drawn over content that takes less than
/// [`MARKER_BELOW_PX`] of the minimap: a square of [`MARKER_PX`] around
/// the content's place, kept inside the drawing area.
#[must_use]
pub fn marker_rect(map: &Minimap) -> Option<Aabb> {
    todo!("T110 behaviour: {map:?}")
}

/// Draws every shape of `editor`'s document as a schematic through
/// `image`, into the current render target: lines [`LINE_PX`] wide, fills
/// as areas, text blocks as bars, shapes below [`DOT_BELOW_PX`] as dots.
pub fn draw_overview(editor: &Editor, image: &Fit) {
    todo!("T110 behaviour: {} {image:?}", editor.document().len())
}

/// Draws the minimap on the window: the panel, the cached `overview`
/// (its texture and the map it was rendered through), the content marker
/// and the frame of the visible world rectangle `view`.
pub fn draw_panel(map: &Minimap, view: Aabb, overview: Option<(&Texture2D, &Fit)>) {
    todo!("T110 behaviour: {map:?} {view:?} {} {LINE_PX}", overview.is_some())
}

/// Size in physical pixels of a target covering `size` logical pixels at
/// scale factor `dpi`, at least 1 × 1; a bad `dpi` counts as 1.
#[must_use]
pub fn target_size(size: Vec2, dpi: f32) -> (u32, u32) {
    todo!("T110 behaviour: {size:?} {dpi}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};

    const VIEWPORT: Vec2 = Vec2::new(800.0, 600.0);
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
        drag(&mut editor, Vec2::new(100.0, 100.0), Vec2::new(300.0, 250.0));
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
        assert_eq!(key_of(&editor, f32::NAN).map(|key| key.size), Some((188, 138)));
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
