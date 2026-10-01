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
use crate::core::document::{Document, ShapeId, tx_clear};
use crate::core::geom::{Aabb, Vec2};
use crate::core::history::History;
use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
use crate::core::keymap;
use crate::core::numbering::{self, FIRST_NUMBER};
use crate::core::palette::ColorId;
use crate::core::shape::{GRID_MAX_CELLS, Shape};
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

/// Columns and rows of a new editor's grids.
pub const DEFAULT_GRID_CELLS: u32 = 4;

/// Maps a key chord to a command; [`keymap::resolve`] by default.
pub type Keymap = fn(Key, Modifiers) -> Option<Command>;

/// Helper settings for new shapes: snapping, grid size and numbering
/// (ADR-T16-3). Only [`Editor::apply`] changes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Helpers {
    /// Smart snapping (round, sizes, alignment) is on.
    pub smart_snap: bool,
    /// Snapping to the world grid is on.
    pub grid_snap: bool,
    /// New rectangles and ellipses are numbered.
    pub numbering: bool,
    /// The number the next numbered shape gets, at least [`FIRST_NUMBER`].
    pub next_number: u32,
    /// Columns of new grids, in `1..=GRID_MAX_CELLS`.
    pub grid_cols: u32,
    /// Rows of new grids, in `1..=GRID_MAX_CELLS`.
    pub grid_rows: u32,
}

impl Default for Helpers {
    fn default() -> Self {
        Self {
            smart_snap: false,
            grid_snap: false,
            numbering: false,
            next_number: FIRST_NUMBER,
            grid_cols: DEFAULT_GRID_CELLS,
            grid_rows: DEFAULT_GRID_CELLS,
        }
    }
}

/// Settings for new shapes. The width is in screen pixels; tools convert it
/// to world units with the zoom at creation time (ADR-0013). The helper
/// settings travel here so every tool sees them (ADR-T16-3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawStyle {
    /// Outline colour.
    pub color: ColorId,
    /// Outline width in screen pixels, one of [`WIDTH_LADDER_PX`].
    pub width_px: f32,
    /// Snapping, grid size and numbering.
    pub helpers: Helpers,
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self {
            color: ColorId::INK,
            width_px: DEFAULT_WIDTH_PX,
            helpers: Helpers::default(),
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
        Self {
            doc: Document::new(),
            history: History::new(),
            camera: Camera::default(),
            selection: Vec::new(),
            clipboard: Clipboard::default(),
            tools: ToolStates::default(),
            tool: Tool::default(),
            style: DrawStyle::default(),
            smoothing: SmoothingLevel::default(),
            toolbar_visible: true,
            viewport: Vec2::ZERO,
            cursor: Vec2::ZERO,
            space_held: false,
            gesture: None,
            keymap: keymap::resolve,
        }
    }

    /// Replaces the keymap (used by tests and alternative bindings).
    #[must_use]
    pub fn with_keymap(self, keymap: Keymap) -> Self {
        Self { keymap, ..self }
    }

    /// Handles one input event. Returns whether the view needs a redraw.
    pub fn handle(&mut self, event: InputEvent) -> bool {
        match event {
            InputEvent::PointerDown { pos, button, mods } => self
                .track_cursor(pos)
                .is_some_and(|pos| self.pointer_down(pos, button, mods)),
            InputEvent::PointerMove { pos, mods } => self
                .track_cursor(pos)
                .is_some_and(|pos| self.pointer_move(pos, mods)),
            InputEvent::PointerUp { pos, button, mods } => self
                .track_cursor(pos)
                .is_some_and(|pos| self.pointer_up(pos, button, mods)),
            InputEvent::Scroll { pos, delta } => {
                if !delta.is_finite() {
                    return false;
                }
                self.track_cursor(pos)
                    .is_some_and(|pos| navigate::zoom(&mut self.camera, pos, delta))
            }
            InputEvent::KeyDown {
                key: Key::Space, ..
            } => {
                self.space_held = true;
                false
            }
            InputEvent::KeyDown { key, mods } => {
                (self.keymap)(key, mods).is_some_and(|command| self.apply(command))
            }
            InputEvent::KeyUp {
                key: Key::Space, ..
            } => {
                self.space_held = false;
                false
            }
            InputEvent::KeyUp { .. } => false,
            InputEvent::Resize { size } => match size.sanitize() {
                Some(size) if size.x >= 0.0 && size.y >= 0.0 => {
                    self.viewport = size;
                    true
                }
                _ => false,
            },
        }
    }

    /// Executes `command`. Returns whether the view needs a redraw.
    pub fn apply(&mut self, command: Command) -> bool {
        match command {
            Command::SetTool(tool) => {
                let cancelled = self.cancel_gesture();
                let changed = self.tool != tool;
                self.tool = tool;
                changed || cancelled
            }
            Command::SetColor(color) => {
                let changed = self.style.color != color;
                self.style.color = color;
                changed
            }
            Command::WidthUp => {
                let current = self.style.width_px;
                self.set_width(WIDTH_LADDER_PX.into_iter().find(|w| *w > current))
            }
            Command::WidthDown => {
                let current = self.style.width_px;
                self.set_width(WIDTH_LADDER_PX.into_iter().rev().find(|w| *w < current))
            }
            Command::CycleSmoothing => {
                self.smoothing = self.smoothing.next();
                true
            }
            Command::Undo => {
                let cancelled = self.cancel_gesture();
                let given = self.helpers().next_number.saturating_sub(1);
                let before = numbering::label_count(&self.doc, given);
                let changed = self.history.undo(&mut self.doc);
                let after = numbering::label_count(&self.doc, given);
                let helpers = &mut self.style.helpers;
                helpers.next_number = numbering::roll_back(helpers.next_number, before, after);
                self.prune_selection();
                changed || cancelled
            }
            Command::Redo => {
                let cancelled = self.cancel_gesture();
                let next = self.helpers().next_number;
                let before = numbering::label_count(&self.doc, next);
                let changed = self.history.redo(&mut self.doc);
                self.advance_numbering(before);
                self.prune_selection();
                changed || cancelled
            }
            Command::Copy => self.run_edit(clipboard::copy),
            Command::Cut => self.run_edit(clipboard::cut),
            Command::Paste => self.run_edit(clipboard::paste),
            Command::Duplicate => self.run_edit(clipboard::duplicate),
            Command::SelectAll => self.run_edit(select::select_all),
            Command::DeleteSelection => self.run_edit(select::delete_selection),
            Command::ClearAll => {
                let cancelled = self.cancel_gesture();
                if self.doc.is_empty() {
                    return cancelled;
                }
                let tx = tx_clear(&self.doc);
                let changed = self.history.commit(&mut self.doc, tx).is_ok();
                self.selection.clear();
                changed || cancelled
            }
            Command::Cancel => {
                if self.cancel_gesture() {
                    true
                } else if self.selection.is_empty() {
                    false
                } else {
                    self.selection.clear();
                    true
                }
            }
            Command::ResetView => {
                let before = self.camera;
                self.camera.reset();
                self.camera != before
            }
            Command::FitView => {
                let before = self.camera;
                let content = self
                    .doc
                    .shapes()
                    .map(|(_, shape)| shape.bounds())
                    .reduce(|a, b| a.union(&b));
                match content {
                    Some(bounds) => self.camera.fit(bounds, self.viewport, FIT_MARGIN_PX),
                    None => self.camera.reset(),
                }
                self.camera != before
            }
            Command::ToggleToolbar => {
                self.toolbar_visible = !self.toolbar_visible;
                true
            }
            // Helper commands never cancel the gesture, so they act live
            // during a drag (ADR-T16-2).
            Command::ToggleSmartSnap
            | Command::ToggleGridSnap
            | Command::ToggleNumbering
            | Command::ResetNumbering
            | Command::GridCols(_)
            | Command::GridRows(_) => apply_helper(&mut self.style.helpers, command),
        }
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

    /// Snapping, grid size and numbering settings.
    #[must_use]
    pub fn helpers(&self) -> Helpers {
        self.style.helpers
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
        self.selection
            .iter()
            .filter_map(|id| self.doc.get(*id))
            .map(Shape::bounds)
            .reduce(|a, b| a.union(&b))
    }

    /// The first shape of [`Editor::overlay`]: the shape being drawn, if any.
    #[must_use]
    pub fn preview(&self) -> Option<Shape> {
        self.overlay().shapes.into_iter().next()
    }

    /// What the gesture in progress draws on top of the document.
    #[must_use]
    pub fn overlay(&self) -> Overlay {
        let Some(Gesture::Tool { tool, .. }) = self.gesture else {
            return Overlay::default();
        };
        self.tools.preview(&ToolView {
            doc: &self.doc,
            camera: &self.camera,
            selection: &self.selection,
            tool,
            style: self.style,
            smoothing: self.smoothing,
            cursor: self.cursor,
        })
    }

    /// The gesture in progress, if any.
    #[must_use]
    pub fn active_gesture(&self) -> Option<ActiveGesture> {
        self.gesture.map(|gesture| match gesture {
            Gesture::Pan { .. } => ActiveGesture::Pan,
            Gesture::Tool { tool, .. } => ActiveGesture::Tool(tool),
        })
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

    /// Records `pos` as the cursor if it is finite and returns it.
    fn track_cursor(&mut self, pos: Vec2) -> Option<Vec2> {
        let pos = pos.sanitize()?;
        self.cursor = pos;
        Some(pos)
    }

    /// Starts a gesture, unless one is already running.
    fn pointer_down(&mut self, pos: Vec2, button: PointerButton, mods: Modifiers) -> bool {
        if self.gesture.is_some() {
            return false;
        }
        let pans = match button {
            PointerButton::Middle => true,
            PointerButton::Left => self.tool == Tool::Hand || self.space_held,
            PointerButton::Right => false,
        };
        if pans {
            self.gesture = Some(Gesture::Pan {
                button,
                pan: Pan::new(pos),
            });
            return false;
        }
        let tool = if button == PointerButton::Right {
            Tool::Eraser
        } else {
            self.tool
        };
        self.gesture = Some(Gesture::Tool { button, tool });
        self.tool_pointer(tool, Phase::Down, pos, mods)
    }

    /// Continues the gesture in progress, if any.
    fn pointer_move(&mut self, pos: Vec2, mods: Modifiers) -> bool {
        match &mut self.gesture {
            Some(Gesture::Pan { pan, .. }) => pan.drag(&mut self.camera, pos),
            Some(Gesture::Tool { tool, .. }) => {
                let tool = *tool;
                self.tool_pointer(tool, Phase::Move, pos, mods)
            }
            None => false,
        }
    }

    /// Ends the gesture in progress if `button` is the one driving it.
    fn pointer_up(&mut self, pos: Vec2, button: PointerButton, mods: Modifiers) -> bool {
        match self.gesture {
            Some(Gesture::Pan { button: b, mut pan }) if b == button => {
                self.gesture = None;
                pan.drag(&mut self.camera, pos)
            }
            Some(Gesture::Tool { button: b, tool }) if b == button => {
                self.gesture = None;
                let numbering = self.helpers().numbering;
                let next = self.helpers().next_number;
                let before = numbering.then(|| numbering::label_count(&self.doc, next));
                let changed = self.tool_pointer(tool, Phase::Up, pos, mods);
                if let Some(before) = before {
                    self.advance_numbering(before);
                }
                self.prune_selection();
                changed
            }
            _ => false,
        }
    }

    /// Moves the numbering counter on if the shapes labelled with it grew
    /// from `before` (ADR-T19-1).
    fn advance_numbering(&mut self, before: usize) {
        let helpers = &mut self.style.helpers;
        let after = numbering::label_count(&self.doc, helpers.next_number);
        helpers.next_number = numbering::advance(helpers.next_number, before, after);
    }

    /// Sends one pointer event to `tool`.
    fn tool_pointer(&mut self, tool: Tool, phase: Phase, pos: Vec2, mods: Modifiers) -> bool {
        let pointer = Pointer { phase, pos, mods };
        let mut ctx = ToolCtx {
            doc: &mut self.doc,
            history: &mut self.history,
            selection: &mut self.selection,
            clipboard: &mut self.clipboard,
            camera: &self.camera,
            tool,
            style: self.style,
            smoothing: self.smoothing,
            cursor: self.cursor,
        };
        self.tools.on_pointer(&mut ctx, pointer)
    }

    /// Runs an editing command (clipboard, selection) after cancelling the
    /// gesture, then drops selected ids that no longer exist.
    fn run_edit(&mut self, edit: fn(&mut ToolCtx<'_>) -> bool) -> bool {
        let cancelled = self.cancel_gesture();
        let mut ctx = ToolCtx {
            doc: &mut self.doc,
            history: &mut self.history,
            selection: &mut self.selection,
            clipboard: &mut self.clipboard,
            camera: &self.camera,
            tool: self.tool,
            style: self.style,
            smoothing: self.smoothing,
            cursor: self.cursor,
        };
        let changed = edit(&mut ctx);
        self.prune_selection();
        changed || cancelled
    }

    /// Discards the gesture in progress. Returns whether there was one.
    fn cancel_gesture(&mut self) -> bool {
        match self.gesture.take() {
            Some(Gesture::Tool { tool, .. }) => {
                self.tools.cancel(tool);
                true
            }
            Some(Gesture::Pan { .. }) => true,
            None => false,
        }
    }

    /// Sets the stroke width if `width` is `Some`. Returns whether it changed.
    fn set_width(&mut self, width: Option<f32>) -> bool {
        width.is_some_and(|width| {
            self.style.width_px = width;
            true
        })
    }

    /// Removes selected ids that are no longer in the document.
    fn prune_selection(&mut self) {
        let doc = &self.doc;
        self.selection.retain(|id| doc.get(*id).is_some());
    }
}

/// Executes a helper command on `helpers`. Returns whether anything changed;
/// other commands change nothing.
fn apply_helper(helpers: &mut Helpers, command: Command) -> bool {
    match command {
        Command::ToggleSmartSnap => {
            helpers.smart_snap = !helpers.smart_snap;
            true
        }
        Command::ToggleGridSnap => {
            helpers.grid_snap = !helpers.grid_snap;
            true
        }
        Command::ToggleNumbering => {
            helpers.numbering = !helpers.numbering;
            true
        }
        Command::ResetNumbering => {
            let changed = helpers.next_number != FIRST_NUMBER;
            helpers.next_number = FIRST_NUMBER;
            changed
        }
        Command::GridCols(delta) => step_cells(&mut helpers.grid_cols, delta),
        Command::GridRows(delta) => step_cells(&mut helpers.grid_rows, delta),
        _ => false,
    }
}

/// Adds `delta` to the grid dimension `cells`, clamped to
/// `1..=GRID_MAX_CELLS`. Returns whether it changed.
fn step_cells(cells: &mut u32, delta: i32) -> bool {
    let stepped = (i64::from(*cells) + i64::from(delta)).clamp(1, i64::from(GRID_MAX_CELLS));
    let stepped = u32::try_from(stepped).unwrap_or(GRID_MAX_CELLS);
    let changed = *cells != stepped;
    *cells = stepped;
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::{Camera, ZOOM_STEP};
    use crate::core::document::tx_insert;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use crate::core::numbering::FIRST_NUMBER;
    use crate::core::palette::ColorId;
    use crate::core::shape::GRID_MAX_CELLS;
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
            label: None,
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

        // Assert: bounds include half the outline width on every side.
        let expected = rect(0.0, 0.0, 10.0, 10.0)
            .bounds()
            .union(&rect(50.0, 50.0, 10.0, 10.0).bounds());
        assert!(bounds.min.approx_eq(expected.min, 1e-3));
        assert!(bounds.max.approx_eq(expected.max, 1e-3));
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

    // ---- T16 AC-4 helpers -------------------------------------------------

    #[test]
    fn helpers_default_off_and_4x4() {
        // Arrange / Act
        let ed = Editor::new();
        let helpers = ed.helpers();

        // Assert
        assert!(!helpers.smart_snap && !helpers.grid_snap && !helpers.numbering);
        assert_eq!(helpers.next_number, FIRST_NUMBER);
        assert_eq!((helpers.grid_cols, helpers.grid_rows), (4, 4));
        assert_eq!(ed.style().helpers, helpers);
    }

    #[test]
    fn toggle_smart_snap_flips() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::ToggleSmartSnap));
        assert!(ed.helpers().smart_snap);
        assert!(ed.apply(Command::ToggleSmartSnap));
        assert!(!ed.helpers().smart_snap);
    }

    #[test]
    fn toggle_grid_snap_flips() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::ToggleGridSnap));
        assert!(ed.helpers().grid_snap);
        assert!(!ed.helpers().smart_snap, "independent flags");
        assert!(ed.apply(Command::ToggleGridSnap));
        assert!(!ed.helpers().grid_snap);
    }

    #[test]
    fn toggle_numbering_flips() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::ToggleNumbering));
        assert!(ed.helpers().numbering);
        assert!(ed.apply(Command::ToggleNumbering));
        assert!(!ed.helpers().numbering);
    }

    #[test]
    fn reset_numbering_restarts_at_one() {
        // Arrange
        let mut ed = Editor::new();
        ed.style.helpers.next_number = 5;

        // Act / Assert
        assert!(ed.apply(Command::ResetNumbering));
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER);
        assert!(!ed.apply(Command::ResetNumbering), "already at the start");
    }

    #[test]
    fn grid_dims_step_by_one() {
        let mut ed = Editor::new();

        assert!(ed.apply(Command::GridCols(1)));
        assert!(ed.apply(Command::GridRows(-1)));

        assert_eq!((ed.helpers().grid_cols, ed.helpers().grid_rows), (5, 3));
    }

    #[test]
    fn grid_dims_clamped() {
        // Arrange
        let mut ed = Editor::new();

        // Act / Assert: the floor.
        for _ in 0..3 {
            ed.apply(Command::GridCols(-1));
        }
        assert_eq!(ed.helpers().grid_cols, 1);
        assert!(!ed.apply(Command::GridCols(-1)), "no change at the floor");
        assert!(!ed.apply(Command::GridCols(i32::MIN)));
        assert_eq!(ed.helpers().grid_cols, 1);

        // The ceiling.
        assert!(ed.apply(Command::GridRows(1000)));
        assert_eq!(ed.helpers().grid_rows, GRID_MAX_CELLS);
        assert!(!ed.apply(Command::GridRows(1)), "no change at the ceiling");
        assert!(ed.apply(Command::GridCols(i32::MAX)));
        assert_eq!(ed.helpers().grid_cols, GRID_MAX_CELLS);
    }

    #[test]
    fn grid_dims_change_keeps_gesture() {
        // Arrange: a grid drag in progress.
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Grid));
        down(&mut ed, PointerButton::Left, 100.0, 100.0);

        // Act
        let changed = ed.apply(Command::GridCols(1))
            && ed.apply(Command::ToggleSmartSnap)
            && ed.apply(Command::ToggleNumbering);

        // Assert
        assert!(changed);
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Tool(Tool::Grid)));
        assert_eq!(ed.helpers().grid_cols, 5);
    }

    // ---- T19 numbering ----------------------------------------------------

    /// Editor with numbering on and the ellipse tool active.
    fn numbering_editor() -> Editor {
        let mut ed = Editor::new();
        ed.apply(Command::ToggleNumbering);
        ed.apply(Command::SetTool(Tool::Ellipse));
        ed
    }

    /// Drags a 30 px node at screen `x`.
    fn draw_node(ed: &mut Editor, x: f32) {
        down(ed, PointerButton::Left, x, 0.0);
        move_to(ed, x + 30.0, 30.0);
        up(ed, PointerButton::Left, x + 30.0, 30.0);
    }

    fn labels(ed: &Editor) -> Vec<Option<u32>> {
        ed.document().shapes().map(|(_, s)| s.label()).collect()
    }

    #[test]
    fn numbered_shapes_count_up() {
        // Arrange
        let mut ed = numbering_editor();

        // Act
        draw_node(&mut ed, 0.0);
        ed.apply(Command::SetTool(Tool::Rect));
        draw_node(&mut ed, 100.0);
        ed.apply(Command::SetTool(Tool::Line));
        draw_node(&mut ed, 200.0);
        ed.apply(Command::SetTool(Tool::Ellipse));
        draw_node(&mut ed, 300.0);

        // Assert
        assert_eq!(labels(&ed), vec![Some(1), Some(2), None, Some(3)]);
        assert_eq!(ed.helpers().next_number, 4);
    }

    #[test]
    fn numbering_off_leaves_counter() {
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Ellipse));

        draw_node(&mut ed, 0.0);

        assert_eq!(labels(&ed), vec![None]);
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER);
    }

    #[test]
    fn stray_click_keeps_counter() {
        let mut ed = numbering_editor();

        down(&mut ed, PointerButton::Left, 10.0, 10.0);
        up(&mut ed, PointerButton::Left, 10.0, 10.0);

        assert!(ed.document().is_empty());
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER);
    }

    #[test]
    fn reset_restarts_at_one() {
        // Arrange
        let mut ed = numbering_editor();
        draw_node(&mut ed, 0.0);
        draw_node(&mut ed, 100.0);

        // Act
        ed.handle(InputEvent::KeyDown {
            key: Key::N,
            mods: Modifiers {
                shift: true,
                ..NONE
            },
        });
        draw_node(&mut ed, 200.0);

        // Assert
        assert_eq!(labels(&ed), vec![Some(1), Some(2), Some(1)]);
        assert_eq!(ed.helpers().next_number, 2);
    }

    #[test]
    fn undo_rolls_counter_back() {
        // Arrange
        let mut ed = numbering_editor();
        draw_node(&mut ed, 0.0);
        draw_node(&mut ed, 100.0);

        // Act
        assert!(ed.apply(Command::Undo));

        // Assert: the next node reuses 2.
        assert_eq!(ed.helpers().next_number, 2);
        draw_node(&mut ed, 200.0);
        assert_eq!(labels(&ed), vec![Some(1), Some(2)]);
        assert_eq!(ed.helpers().next_number, 3);
    }

    #[test]
    fn redo_restores_counter() {
        // Arrange
        let mut ed = numbering_editor();
        draw_node(&mut ed, 0.0);
        draw_node(&mut ed, 100.0);
        ed.apply(Command::Undo);
        ed.apply(Command::Undo);
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER);

        // Act / Assert
        assert!(ed.apply(Command::Redo));
        assert_eq!(ed.helpers().next_number, 2);
        assert!(ed.apply(Command::Redo));
        assert_eq!(ed.helpers().next_number, 3);
        assert_eq!(labels(&ed), vec![Some(1), Some(2)]);
    }

    #[test]
    fn undo_after_reset_rolls_back_duplicate() {
        // Arrange: 1, 2, reset, 1 again.
        let mut ed = numbering_editor();
        draw_node(&mut ed, 0.0);
        draw_node(&mut ed, 100.0);
        ed.apply(Command::ResetNumbering);
        draw_node(&mut ed, 200.0);
        assert_eq!(ed.helpers().next_number, 2);

        // Act
        ed.apply(Command::Undo);

        // Assert
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER);
        assert_eq!(labels(&ed), vec![Some(1), Some(2)]);
    }

    #[test]
    fn undo_of_unnumbered_keeps_counter() {
        // Arrange: node 1, then an unnumbered line.
        let mut ed = numbering_editor();
        draw_node(&mut ed, 0.0);
        ed.apply(Command::SetTool(Tool::Line));
        draw_node(&mut ed, 100.0);

        // Act / Assert
        ed.apply(Command::Undo);
        assert_eq!(ed.helpers().next_number, 2);
        ed.apply(Command::Redo);
        assert_eq!(ed.helpers().next_number, 2);
    }

    #[test]
    fn duplicate_keeps_labels_and_counter() {
        // Arrange
        let mut ed = numbering_editor();
        draw_node(&mut ed, 0.0);
        draw_node(&mut ed, 100.0);

        // Act
        ed.apply(Command::SelectAll);
        ed.apply(Command::Copy);
        ed.apply(Command::Paste);
        ed.apply(Command::Duplicate);

        // Assert
        assert_eq!(
            labels(&ed),
            vec![Some(1), Some(2), Some(1), Some(2), Some(1), Some(2)]
        );
        assert_eq!(ed.helpers().next_number, 3);
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
        assert!(
            ed.selection()
                .iter()
                .all(|id| ed.document().get(*id).is_some())
        );
    }
}
