//! The editor: owns all state and routes input to tools.
//!
//! [`Editor`] is the single object the shell talks to: it feeds
//! [`InputEvent`]s to [`Editor::handle`] and [`Command`]s (from the toolbar) to
//! [`Editor::apply`], then reads state back for drawing. Input is filtered at
//! the boundary: events with non-finite numbers are dropped, so tools only see
//! finite screen positions.
//!
//! Routing (ADR-T08-1): a middle drag, a left drag with the hand tool or with
//! `Space` held pans; a right drag erases from any tool; any other left drag
//! goes to the active tool. One gesture runs at a time; other buttons are
//! ignored until it ends. Keys go through the keymap, then [`Editor::apply`].

use crate::core::camera::Camera;
use crate::core::clipboard::{self, Clipboard};
use crate::core::command::{Command, Tool};
use crate::core::document::{Document, ShapeId};
use crate::core::geom::{Aabb, Vec2};
use crate::core::history::History;
use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
use crate::core::keymap;
use crate::core::palette::ColorId;
use crate::core::shape::Shape;
use crate::core::smoothing::SmoothingLevel;
use crate::core::tools::navigate::{self, Pan};
use crate::core::tools::{Overlay, Phase, Pointer, ToolCtx, ToolStates, ToolView, select};

/// Stroke widths offered by `[` / `]`, in screen pixels, ascending.
pub const WIDTH_LADDER_PX: [f32; 10] = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 32.0];
/// Thinnest stroke width in screen pixels.
pub const MIN_WIDTH_PX: f32 = WIDTH_LADDER_PX[0];
/// Thickest stroke width in screen pixels.
pub const MAX_WIDTH_PX: f32 = WIDTH_LADDER_PX[WIDTH_LADDER_PX.len() - 1];
/// Stroke width of a new editor, in screen pixels.
pub const DEFAULT_WIDTH_PX: f32 = 3.0;
/// Free space kept around content by [`Command::FitView`], in pixels.
pub const FIT_MARGIN_PX: f32 = 32.0;

/// Maps a key chord to a command; [`keymap::resolve`] by default.
pub type Keymap = fn(Key, Modifiers) -> Option<Command>;

/// Style for new shapes. The width is in screen pixels; tools convert it to
/// world units with the zoom at creation time (ADR-0013).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawStyle {
    /// Outline colour.
    pub color: ColorId,
    /// Outline width in screen pixels, one of [`WIDTH_LADDER_PX`].
    pub width_px: f32,
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self {
            color: ColorId::INK,
            width_px: DEFAULT_WIDTH_PX,
        }
    }
}

/// The gesture in progress, as seen from outside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActiveGesture {
    /// Panning the view.
    Pan,
    /// A tool gesture (`Eraser` for a right drag).
    Tool(Tool),
}

/// The gesture in progress and the button that drives it.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Gesture {
    /// Panning the view.
    Pan {
        /// Button whose release ends the gesture.
        button: PointerButton,
        /// Pan state.
        pan: Pan,
    },
    /// A tool gesture.
    Tool {
        /// Button whose release ends the gesture.
        button: PointerButton,
        /// Tool receiving the events.
        tool: Tool,
    },
}

/// All editor state: document, history, view, tools and settings.
#[derive(Debug, Clone)]
pub struct Editor {
    doc: Document,
    history: History,
    camera: Camera,
    selection: Vec<ShapeId>,
    clipboard: Clipboard,
    tools: ToolStates,
    tool: Tool,
    style: DrawStyle,
    smoothing: SmoothingLevel,
    toolbar_visible: bool,
    /// Viewport size in pixels (finite, non-negative).
    viewport: Vec2,
    /// Last finite pointer position in screen pixels.
    cursor: Vec2,
    space_held: bool,
    gesture: Option<Gesture>,
    keymap: Keymap,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    /// An empty document at the default view, pen tool, default style.
    #[must_use]
    pub fn new() -> Self {
        todo!()
    }

    /// Replaces the keymap (used by tests and alternative bindings).
    #[must_use]
    pub fn with_keymap(self, keymap: Keymap) -> Self {
        let _ = keymap;
        todo!()
    }

    /// Handles one input event. Returns whether the view needs a redraw.
    pub fn handle(&mut self, event: InputEvent) -> bool {
        let _ = event;
        todo!()
    }

    /// Executes `command`. Returns whether the view needs a redraw.
    pub fn apply(&mut self, command: Command) -> bool {
        let _ = command;
        todo!()
    }

    /// The document.
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// The camera.
    #[must_use]
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// The active tool (what a left drag does).
    #[must_use]
    pub fn tool(&self) -> Tool {
        self.tool
    }

    /// Style for new shapes.
    #[must_use]
    pub fn style(&self) -> DrawStyle {
        self.style
    }

    /// Anti-tremor level for new strokes.
    #[must_use]
    pub fn smoothing(&self) -> SmoothingLevel {
        self.smoothing
    }

    /// Selected shape ids; every id exists in the document.
    #[must_use]
    pub fn selection(&self) -> &[ShapeId] {
        &self.selection
    }

    /// World bounds of the selected shapes, `None` if nothing is selected.
    #[must_use]
    pub fn selection_bounds(&self) -> Option<Aabb> {
        todo!()
    }

    /// The first shape of [`Editor::overlay`]: the shape being drawn, if any.
    #[must_use]
    pub fn preview(&self) -> Option<Shape> {
        todo!()
    }

    /// What the gesture in progress draws on top of the document.
    #[must_use]
    pub fn overlay(&self) -> Overlay {
        todo!()
    }

    /// The gesture in progress, if any.
    #[must_use]
    pub fn active_gesture(&self) -> Option<ActiveGesture> {
        todo!()
    }

    /// Whether the toolbar is shown.
    #[must_use]
    pub fn toolbar_visible(&self) -> bool {
        self.toolbar_visible
    }

    /// Whether there is an action to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether there is an action to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Viewport size in pixels, as last reported by [`InputEvent::Resize`].
    #[must_use]
    pub fn viewport(&self) -> Vec2 {
        self.viewport
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::{Camera, ZOOM_STEP};
    use crate::core::document::tx_insert;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use crate::core::smoothing::SmoothingLevel;

    const NONE: Modifiers = Modifiers::NONE;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Shape {
        Shape::Rect {
            a: Vec2::new(x, y),
            b: Vec2::new(x + w, y + h),
            style: Style {
                color: ColorId::INK,
                width: 1.0,
            },
            fill: None,
        }
    }

    /// Editor whose document holds `shapes`, committed as one undo step.
    fn editor_with(shapes: Vec<Shape>) -> Editor {
        let mut ed = Editor::new();
        let tx = tx_insert(&mut ed.doc, shapes);
        assert!(ed.history.commit(&mut ed.doc, tx).is_ok());
        ed
    }

    fn ids(ed: &Editor) -> Vec<ShapeId> {
        ed.document().shapes().map(|(id, _)| id).collect()
    }

    fn down(ed: &mut Editor, button: PointerButton, x: f32, y: f32) -> bool {
        ed.handle(InputEvent::PointerDown {
            pos: Vec2::new(x, y),
            button,
            mods: NONE,
        })
    }

    fn move_to(ed: &mut Editor, x: f32, y: f32) -> bool {
        ed.handle(InputEvent::PointerMove {
            pos: Vec2::new(x, y),
            mods: NONE,
        })
    }

    fn up(ed: &mut Editor, button: PointerButton, x: f32, y: f32) -> bool {
        ed.handle(InputEvent::PointerUp {
            pos: Vec2::new(x, y),
            button,
            mods: NONE,
        })
    }

    fn key(ed: &mut Editor, key: Key) -> bool {
        ed.handle(InputEvent::KeyDown { key, mods: NONE })
    }

    fn origin_on_screen(ed: &Editor) -> Vec2 {
        ed.camera().world_to_screen(Vec2::ZERO)
    }

    // ---- AC-8 defaults ----------------------------------------------------

    #[test]
    fn new_editor_defaults() {
        let ed = Editor::new();

        assert_eq!(ed.tool(), Tool::Pen);
        assert_eq!(ed.style().color, ColorId::INK);
        assert!(approx_eq(ed.style().width_px, DEFAULT_WIDTH_PX, 1e-6));
        assert_eq!(ed.smoothing(), SmoothingLevel::Medium);
        assert_eq!(*ed.camera(), Camera::default());
        assert!(ed.toolbar_visible());
        assert!(ed.document().is_empty());
        assert!(ed.selection().is_empty());
        assert!(ed.selection_bounds().is_none());
        assert!(ed.preview().is_none());
        assert_eq!(ed.overlay(), Overlay::default());
        assert!(!ed.can_undo() && !ed.can_redo());
        assert_eq!(ed.active_gesture(), None);
    }

    #[test]
    fn selection_bounds_unions_selected_shapes() {
        // Arrange
        let mut ed = editor_with(vec![
            rect(0.0, 0.0, 10.0, 10.0),
            rect(50.0, 50.0, 10.0, 10.0),
            rect(100.0, 100.0, 10.0, 10.0),
        ]);
        let all = ids(&ed);
        ed.selection = vec![all[0], all[1]];

        // Act
        let Some(bounds) = ed.selection_bounds() else {
            panic!("selection is not empty");
        };

        // Assert
        assert!(bounds.min.approx_eq(Vec2::new(0.0, 0.0), 1e-3));
        assert!(bounds.max.approx_eq(Vec2::new(60.0, 60.0), 1e-3));
    }

    #[test]
    fn selection_bounds_none_when_empty() {
        let ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);

        assert!(ed.selection_bounds().is_none());
    }

    // ---- AC-4 boundary ----------------------------------------------------

    #[test]
    fn non_finite_pointer_ignored() {
        let mut ed = Editor::new();

        assert!(!down(&mut ed, PointerButton::Middle, f32::NAN, 0.0));
        assert_eq!(ed.active_gesture(), None);

        assert!(!down(&mut ed, PointerButton::Middle, 0.0, 0.0));
        assert!(!move_to(&mut ed, f32::INFINITY, 5.0));
        assert!(!up(&mut ed, PointerButton::Middle, 1.0, f32::NEG_INFINITY));
        assert_eq!(*ed.camera(), Camera::default());
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Pan));
    }

    #[test]
    fn non_finite_scroll_ignored() {
        let mut ed = Editor::new();

        let changed = ed.handle(InputEvent::Scroll {
            pos: Vec2::new(10.0, 10.0),
            delta: f32::NAN,
        }) || ed.handle(InputEvent::Scroll {
            pos: Vec2::new(f32::NAN, 10.0),
            delta: 1.0,
        });

        assert!(!changed);
        assert_eq!(*ed.camera(), Camera::default());
    }

    #[test]
    fn resize_sets_viewport() {
        let mut ed = Editor::new();

        assert!(ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        }));
        assert!(ed.viewport().approx_eq(Vec2::new(800.0, 600.0), 1e-6));
    }

    #[test]
    fn non_finite_resize_ignored() {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });

        assert!(!ed.handle(InputEvent::Resize {
            size: Vec2::new(f32::NAN, 600.0),
        }));
        assert!(!ed.handle(InputEvent::Resize {
            size: Vec2::new(-1.0, 600.0),
        }));
        assert!(ed.viewport().approx_eq(Vec2::new(800.0, 600.0), 1e-6));
    }

    // ---- AC-5 navigation --------------------------------------------------

    #[test]
    fn scroll_zooms_at_cursor() {
        // Arrange
        let mut ed = Editor::new();
        let cursor = Vec2::new(100.0, 50.0);
        let world_before = ed.camera().screen_to_world(cursor);

        // Act
        let changed = ed.handle(InputEvent::Scroll {
            pos: cursor,
            delta: 1.0,
        });

        // Assert
        assert!(changed);
        assert!(approx_eq(ed.camera().zoom(), ZOOM_STEP, 1e-5));
        assert!(
            ed.camera()
                .screen_to_world(cursor)
                .approx_eq(world_before, 1e-3)
        );
    }

    #[test]
    fn middle_drag_pans_from_any_tool() {
        for tool in Tool::ALL {
            // Arrange
            let mut ed = Editor::new();
            ed.apply(Command::SetTool(tool));

            // Act
            down(&mut ed, PointerButton::Middle, 10.0, 10.0);
            let moved = move_to(&mut ed, 30.0, 40.0);
            up(&mut ed, PointerButton::Middle, 30.0, 40.0);

            // Assert
            assert!(moved, "{tool:?}");
            assert!(
                origin_on_screen(&ed).approx_eq(Vec2::new(20.0, 30.0), 1e-4),
                "{tool:?}"
            );
            assert!(ed.document().is_empty(), "{tool:?}");
            assert_eq!(ed.active_gesture(), None, "{tool:?}");
        }
    }

    #[test]
    fn space_drag_pans() {
        // Arrange
        let mut ed = Editor::new();

        // Act
        key(&mut ed, Key::Space);
        down(&mut ed, PointerButton::Left, 0.0, 0.0);
        move_to(&mut ed, 5.0, 5.0);
        ed.handle(InputEvent::KeyUp {
            key: Key::Space,
            mods: NONE,
        });
        move_to(&mut ed, 10.0, -20.0);
        up(&mut ed, PointerButton::Left, 10.0, -20.0);

        // Assert
        assert!(origin_on_screen(&ed).approx_eq(Vec2::new(10.0, -20.0), 1e-4));
        assert_eq!(ed.active_gesture(), None);
    }

    #[test]
    fn hand_tool_drag_pans() {
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Hand));

        down(&mut ed, PointerButton::Left, 0.0, 0.0);
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Pan));
        move_to(&mut ed, -7.0, 3.0);
        up(&mut ed, PointerButton::Left, -7.0, 3.0);

        assert!(origin_on_screen(&ed).approx_eq(Vec2::new(-7.0, 3.0), 1e-4));
    }

    // ---- AC-6 routing -----------------------------------------------------

    #[test]
    fn right_drag_routes_to_eraser() {
        for tool in [Tool::Pen, Tool::Rect, Tool::Select, Tool::Hand] {
            let mut ed = Editor::new();
            ed.apply(Command::SetTool(tool));

            down(&mut ed, PointerButton::Right, 0.0, 0.0);
            assert_eq!(
                ed.active_gesture(),
                Some(ActiveGesture::Tool(Tool::Eraser)),
                "{tool:?}"
            );
            assert_eq!(ed.tool(), tool, "right drag must not switch the tool");
            up(&mut ed, PointerButton::Right, 0.0, 0.0);
            assert_eq!(ed.active_gesture(), None);
        }
    }

    #[test]
    fn left_drag_routes_to_active_tool() {
        for tool in Tool::ALL.into_iter().filter(|t| *t != Tool::Hand) {
            let mut ed = Editor::new();
            ed.apply(Command::SetTool(tool));

            down(&mut ed, PointerButton::Left, 0.0, 0.0);

            assert_eq!(ed.active_gesture(), Some(ActiveGesture::Tool(tool)));
        }
    }

    #[test]
    fn other_button_does_not_end_gesture() {
        let mut ed = Editor::new();

        down(&mut ed, PointerButton::Middle, 0.0, 0.0);
        down(&mut ed, PointerButton::Right, 0.0, 0.0);
        up(&mut ed, PointerButton::Left, 0.0, 0.0);
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Pan));

        up(&mut ed, PointerButton::Middle, 0.0, 0.0);
        assert_eq!(ed.active_gesture(), None);
    }

    fn test_keymap(key: Key, mods: Modifiers) -> Option<Command> {
        match (key, mods.ctrl) {
            (Key::R, false) => Some(Command::SetTool(Tool::Rect)),
            (Key::Space | Key::Tab, _) => Some(Command::ToggleToolbar),
            _ => None,
        }
    }

    #[test]
    fn keydown_uses_keymap() {
        let mut ed = Editor::new().with_keymap(test_keymap);

        assert!(key(&mut ed, Key::R));
        assert_eq!(ed.tool(), Tool::Rect);

        assert!(!key(&mut ed, Key::Q));
        assert_eq!(ed.tool(), Tool::Rect);

        assert!(key(&mut ed, Key::Tab));
        assert!(!ed.toolbar_visible());
    }

    #[test]
    fn space_key_not_resolved() {
        let mut ed = Editor::new().with_keymap(test_keymap);

        key(&mut ed, Key::Space);

        assert!(ed.toolbar_visible());
    }

    // ---- AC-7 commands ----------------------------------------------------

    #[test]
    fn set_tool_changes_tool() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::SetTool(Tool::Ellipse)));
        assert_eq!(ed.tool(), Tool::Ellipse);
        assert!(!ed.apply(Command::SetTool(Tool::Ellipse)));
    }

    #[test]
    fn set_tool_cancels_gesture() {
        let mut ed = Editor::new();
        down(&mut ed, PointerButton::Left, 0.0, 0.0);

        ed.apply(Command::SetTool(Tool::Line));

        assert_eq!(ed.active_gesture(), None);
        assert_eq!(ed.tool(), Tool::Line);
    }

    #[test]
    fn set_color_changes_style() {
        let mut ed = Editor::new();
        let Some(red) = ColorId::new(1) else {
            panic!("palette has colour 1");
        };

        assert!(ed.apply(Command::SetColor(red)));
        assert_eq!(ed.style().color, red);
        assert!(!ed.apply(Command::SetColor(red)));
    }

    #[test]
    fn width_up_steps_ladder() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::WidthUp));

        assert!(approx_eq(ed.style().width_px, 4.0, 1e-6));
    }

    #[test]
    fn width_up_clamps_at_max() {
        let mut ed = Editor::new();
        for _ in 0..20 {
            ed.apply(Command::WidthUp);
        }

        assert!(approx_eq(ed.style().width_px, MAX_WIDTH_PX, 1e-6));
        assert!(!ed.apply(Command::WidthUp));
    }

    #[test]
    fn width_down_steps_ladder() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::WidthDown));

        assert!(approx_eq(ed.style().width_px, 2.0, 1e-6));
    }

    #[test]
    fn width_down_clamps_at_min() {
        let mut ed = Editor::new();
        for _ in 0..20 {
            ed.apply(Command::WidthDown);
        }

        assert!(approx_eq(ed.style().width_px, MIN_WIDTH_PX, 1e-6));
        assert!(!ed.apply(Command::WidthDown));
    }

    #[test]
    fn width_ladder_round_trips() {
        let mut ed = Editor::new();
        for _ in 0..20 {
            ed.apply(Command::WidthDown);
        }
        let mut seen = vec![ed.style().width_px];
        while ed.apply(Command::WidthUp) {
            seen.push(ed.style().width_px);
        }

        assert_eq!(seen.len(), WIDTH_LADDER_PX.len());
        assert!(
            seen.iter()
                .zip(WIDTH_LADDER_PX)
                .all(|(a, b)| approx_eq(*a, b, 1e-6))
        );
    }

    #[test]
    fn cycle_smoothing_advances() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::CycleSmoothing));
        assert_eq!(ed.smoothing(), SmoothingLevel::High);
        ed.apply(Command::CycleSmoothing);
        assert_eq!(ed.smoothing(), SmoothingLevel::Off);
    }

    #[test]
    fn undo_reverts_last_step() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);

        assert!(ed.apply(Command::Undo));

        assert!(ed.document().is_empty());
        assert!(!ed.can_undo());
        assert!(ed.can_redo());
    }

    #[test]
    fn undo_on_empty_history_returns_false() {
        let mut ed = Editor::new();

        assert!(!ed.apply(Command::Undo));
    }

    #[test]
    fn undo_prunes_selection() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        ed.selection = ids(&ed);

        ed.apply(Command::Undo);

        assert!(ed.selection().is_empty());
    }

    #[test]
    fn undo_cancels_gesture() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        down(&mut ed, PointerButton::Right, 0.0, 0.0);

        ed.apply(Command::Undo);

        assert_eq!(ed.active_gesture(), None);
    }

    #[test]
    fn redo_reapplies_undone_step() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        let before = ids(&ed);
        ed.apply(Command::Undo);

        assert!(ed.apply(Command::Redo));

        assert_eq!(ids(&ed), before);
        assert!(!ed.can_redo());
    }

    #[test]
    fn redo_on_empty_stack_returns_false() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);

        assert!(!ed.apply(Command::Redo));
    }

    #[test]
    fn clear_all_removes_everything_in_one_step() {
        let mut ed = editor_with(vec![
            rect(0.0, 0.0, 1.0, 1.0),
            rect(5.0, 5.0, 1.0, 1.0),
            rect(9.0, 9.0, 1.0, 1.0),
        ]);
        let before = ids(&ed);

        assert!(ed.apply(Command::ClearAll));
        assert!(ed.document().is_empty());

        ed.apply(Command::Undo);
        assert_eq!(ids(&ed), before);
    }

    #[test]
    fn clear_all_on_empty_document_is_noop() {
        let mut ed = Editor::new();

        assert!(!ed.apply(Command::ClearAll));
        assert!(!ed.can_undo());
    }

    #[test]
    fn clear_all_clears_selection() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        ed.selection = ids(&ed);

        ed.apply(Command::ClearAll);

        assert!(ed.selection().is_empty());
    }

    #[test]
    fn cancel_ends_gesture_first() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        ed.selection = ids(&ed);
        down(&mut ed, PointerButton::Middle, 0.0, 0.0);

        assert!(ed.apply(Command::Cancel));

        assert_eq!(ed.active_gesture(), None);
        assert_eq!(ed.selection().len(), 1);
    }

    #[test]
    fn cancel_clears_selection_when_idle() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        ed.selection = ids(&ed);

        assert!(ed.apply(Command::Cancel));

        assert!(ed.selection().is_empty());
    }

    #[test]
    fn cancel_with_nothing_to_cancel_returns_false() {
        let mut ed = Editor::new();

        assert!(!ed.apply(Command::Cancel));
    }

    #[test]
    fn reset_view_restores_default_camera() {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Scroll {
            pos: Vec2::new(30.0, 30.0),
            delta: 3.0,
        });

        assert!(ed.apply(Command::ResetView));

        assert_eq!(*ed.camera(), Camera::default());
        assert!(!ed.apply(Command::ResetView));
    }

    #[test]
    fn fit_view_frames_content() {
        // Arrange
        let mut ed = editor_with(vec![
            rect(0.0, 0.0, 100.0, 100.0),
            rect(1000.0, 1000.0, 100.0, 100.0),
        ]);
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });

        // Act
        let changed = ed.apply(Command::FitView);

        // Assert
        let cam = ed.camera();
        let min = cam.world_to_screen(Vec2::new(0.0, 0.0));
        let max = cam.world_to_screen(Vec2::new(1100.0, 1100.0));
        assert!(changed);
        assert!(min.x >= 0.0 && min.y >= 0.0, "{min:?}");
        assert!(max.x <= 800.0 && max.y <= 600.0, "{max:?}");
        assert!(((min + max) * 0.5).approx_eq(Vec2::new(400.0, 300.0), 1e-2));
    }

    #[test]
    fn fit_view_on_empty_document_resets() {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        ed.handle(InputEvent::Scroll {
            pos: Vec2::new(30.0, 30.0),
            delta: 2.0,
        });

        ed.apply(Command::FitView);

        assert_eq!(*ed.camera(), Camera::default());
    }

    #[test]
    fn toggle_toolbar_flips_visibility() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::ToggleToolbar));
        assert!(!ed.toolbar_visible());
        ed.apply(Command::ToggleToolbar);
        assert!(ed.toolbar_visible());
    }

    // ---- AC-9 stubs -------------------------------------------------------

    #[test]
    fn stub_commands_do_not_panic() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let commands = [
            Command::SelectAll,
            Command::Copy,
            Command::Paste,
            Command::Duplicate,
            Command::Cut,
            Command::DeleteSelection,
        ];
        for command in commands {
            ed.apply(command);
        }
        for tool in Tool::ALL {
            ed.apply(Command::SetTool(tool));
            down(&mut ed, PointerButton::Left, 1.0, 1.0);
            move_to(&mut ed, 5.0, 5.0);
            let _ = ed.overlay();
            up(&mut ed, PointerButton::Left, 5.0, 5.0);
        }
        assert!(ed.selection().iter().all(|id| ed.document().get(*id).is_some()));
    }
}
