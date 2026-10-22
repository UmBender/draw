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
//!
//! Text edit mode (ADR-T29-2): while a text block is open, typed characters
//! and editing keys go to it and no key reaches the keymap; `Esc`, a press
//! outside the block or any applied command commits it as one undo step.

use crate::core::camera::Camera;
use crate::core::clipboard::{self, Clipboard};
use crate::core::command::{Command, Tool};
use crate::core::document::{Document, ShapeId, tx_clear, tx_insert, tx_remove, tx_replace};
use crate::core::experimental::Experimental;
use crate::core::geom::{Aabb, Vec2};
use crate::core::history::{HISTORY_LIMIT, History};
use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
use crate::core::keymap;
use crate::core::numbering::{self, FIRST_NUMBER};
use crate::core::palette::ColorId;
use crate::core::shape::{CellStep, GRID_MAX_CELLS, Shape, TextSlot};
use crate::core::smoothing::SmoothingLevel;
use crate::core::tap::TapFilter;
use crate::core::tools::navigate::{self, Pan};
use crate::core::tools::text::{self, TextEdit};
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
    /// Experimental features, set once by [`Editor::with_experimental`]
    /// (ADR-T31-1).
    pub experimental: Experimental,
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
            experimental: Experimental::default(),
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
    /// A plain Select press on the text block that was the only selection:
    /// press position and block id. Released as a click, it re-opens the
    /// block (ADR-T29-2).
    reedit: Option<(Vec2, ShapeId)>,
    keymap: Keymap,
    /// Holds pen taps still before the events reach the tools (ADR-T34-1).
    tap: TapFilter,
    /// Pen pressure of the pointer events, in `[0, 1]` (ADR-T35-2).
    pressure: Option<f32>,
    /// Grows whenever the document may have changed (ADR-T24-1).
    doc_revision: u64,
    /// `doc_revision` of the latest change that was not a pure append of
    /// shapes on top (ADR-T24-3).
    doc_base_revision: u64,
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
            reedit: None,
            keymap: keymap::resolve,
            tap: TapFilter::new(0.0),
            pressure: None,
            doc_revision: 0,
            doc_base_revision: 0,
        }
    }

    /// Sets the tap slop in screen pixels: a press released within it is a
    /// click at the press point; `0` turns it off (ADR-T34-1).
    #[must_use]
    pub fn with_tap_slop(mut self, slop_px: f32) -> Self {
        self.tap = TapFilter::new(slop_px);
        self
    }

    /// The tap slop in screen pixels, `0` when off.
    #[must_use]
    pub fn tap_slop(&self) -> f32 {
        self.tap.slop()
    }

    /// The pen pressure of the pointer events, in `[0, 1]`; `None` for a
    /// mouse (ADR-T35-2).
    #[must_use]
    pub fn pressure(&self) -> Option<f32> {
        self.pressure
    }

    /// Replaces the keymap (used by tests and alternative bindings).
    #[must_use]
    pub fn with_keymap(self, keymap: Keymap) -> Self {
        Self { keymap, ..self }
    }

    /// Turns the `experimental` features on or off (ADR-T31-1).
    #[must_use]
    pub fn with_experimental(mut self, experimental: Experimental) -> Self {
        self.style.helpers.experimental = experimental;
        self
    }

    /// The experimental features that are on.
    #[must_use]
    pub fn experimental(&self) -> Experimental {
        self.style.helpers.experimental
    }

    /// Handles one input event. Returns whether the view needs a redraw.
    ///
    /// Events go through the tap filter first (ADR-T34-1).
    pub fn handle(&mut self, event: InputEvent) -> bool {
        let mut redraw = false;
        for event in self.tap.filter(event).into_iter().flatten() {
            redraw |= self.handle_filtered(event);
        }
        redraw
    }

    /// [`Editor::handle`] for one event that left the tap filter.
    fn handle_filtered(&mut self, event: InputEvent) -> bool {
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
            InputEvent::KeyDown { key, mods } if self.tools.text.edit.is_some() => {
                self.text_key(key, mods)
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
            InputEvent::Char { ch, mods } => {
                let typed = !mods.ctrl && !ch.is_control();
                match &mut self.tools.text.edit {
                    Some(edit) if typed => edit.insert(ch),
                    _ => false,
                }
            }
            InputEvent::Pressure { pressure } => {
                self.pressure = pressure
                    .filter(|p| p.is_finite())
                    .map(|p| p.clamp(0.0, 1.0));
                false
            }
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
    ///
    /// An open text block is committed first, except that
    /// [`Command::Undo`] cancels it instead (ADR-T29-2).
    pub fn apply(&mut self, command: Command) -> bool {
        if self.tools.text.edit.is_some() {
            if command == Command::Undo {
                return self.cancel_text();
            }
            let committed = self.finish_text();
            return self.apply(command) || committed;
        }
        if edits_document(command) {
            let appends = matches!(command, Command::Paste | Command::Duplicate);
            self.track_document(appends, |ed| ed.execute(command))
        } else {
            self.execute(command)
        }
    }

    /// Executes `command` without tracking document changes.
    fn execute(&mut self, command: Command) -> bool {
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

    /// A counter that grows whenever the document may have changed and stays
    /// put otherwise (ADR-T24-1). The shell re-renders its document layer
    /// only when this, the camera, the viewport or the hidden set changes.
    #[must_use]
    pub fn document_revision(&self) -> u64 {
        self.doc_revision
    }

    /// The [`Editor::document_revision`] of the latest document change that
    /// was not a pure append of shapes on top (ADR-T24-3). While it stays at
    /// or below a cached revision, the document is that cached one plus the
    /// shapes from its old length on.
    #[must_use]
    pub fn document_base_revision(&self) -> u64 {
        self.doc_base_revision
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

    /// What the gesture in progress and the open text block draw on top of
    /// the document.
    #[must_use]
    pub fn overlay(&self) -> Overlay {
        let view = |tool| ToolView {
            doc: &self.doc,
            camera: &self.camera,
            selection: &self.selection,
            tool,
            style: self.style,
            smoothing: self.smoothing,
            cursor: self.cursor,
        };
        let mut overlay = match self.gesture {
            Some(Gesture::Tool { tool, .. }) if tool != Tool::Text => {
                self.tools.preview(&view(tool))
            }
            _ => Overlay::default(),
        };
        let edit = text::preview(&self.tools.text, &view(Tool::Text));
        overlay.shapes.extend(edit.shapes);
        overlay.hidden.extend(edit.hidden);
        overlay
    }

    /// The gesture in progress, if any.
    #[must_use]
    pub fn active_gesture(&self) -> Option<ActiveGesture> {
        self.gesture.map(|gesture| match gesture {
            Gesture::Pan { .. } => ActiveGesture::Pan,
            Gesture::Tool { tool, .. } => ActiveGesture::Tool(tool),
        })
    }

    /// The text block being edited, if any (ADR-T29-2).
    #[must_use]
    pub fn text_edit(&self) -> Option<&TextEdit> {
        self.tools.text.edit.as_ref()
    }

    /// The caret of the text block being edited, as a world segment from
    /// the top to the bottom of its line; `None` when not editing.
    #[must_use]
    pub fn text_caret(&self) -> Option<[Vec2; 2]> {
        self.text_edit().map(TextEdit::caret_line)
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

    /// Starts a gesture, unless one is already running. With a text block
    /// open, a left press inside it moves the caret and any other press
    /// that does not pan commits it first (ADR-T29-2).
    fn pointer_down(&mut self, pos: Vec2, button: PointerButton, mods: Modifiers) -> bool {
        if self.gesture.is_some() {
            return false;
        }
        let pans = match button {
            PointerButton::Middle => true,
            PointerButton::Left => self.tool == Tool::Hand || self.space_held,
            PointerButton::Right => false,
        };
        let mut committed = false;
        if !pans {
            let world = self.camera.screen_to_world(pos);
            let tol = self.camera.world_len(select::HIT_TOLERANCE_PX);
            if let Some(edit) = &mut self.tools.text.edit {
                if button == PointerButton::Left && edit.bounds().expand(tol).contains(world) {
                    edit.set_caret_at(world);
                    return true;
                }
                committed = self.finish_text();
            }
            if button == PointerButton::Left && self.tool == Tool::Select && mods == Modifiers::NONE
            {
                self.reedit = self.selected_text_at(world, tol).map(|id| (pos, id));
            }
        }
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
        self.tool_pointer(tool, Phase::Down, pos, mods) || committed
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
                let reopened = self.reopen_clicked_text(pos);
                changed || reopened
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

    /// Sends one pointer event to `tool`. Tools commit only on `Down` or
    /// `Up`, so a `Move` never touches the document revision.
    fn tool_pointer(&mut self, tool: Tool, phase: Phase, pos: Vec2, mods: Modifiers) -> bool {
        if phase == Phase::Move {
            self.send_pointer(tool, phase, pos, mods)
        } else {
            let appends = phase == Phase::Up && creates_shapes(tool);
            self.track_document(appends, |ed| ed.send_pointer(tool, phase, pos, mods))
        }
    }

    /// Runs `edit`, which may change the document through the history, and
    /// bumps the document revision if it did (ADR-T24-1). A commit on a full
    /// undo stack keeps both stack lengths, so there `edit`'s result decides.
    /// Unless `appends` (the edit only ever inserts shapes on top) and the
    /// document grew, the base revision follows (ADR-T24-3).
    fn track_document(&mut self, appends: bool, edit: impl FnOnce(&mut Self) -> bool) -> bool {
        let lengths = |history: &History| (history.undo_len(), history.redo_len());
        let before = lengths(&self.history);
        let len = self.doc.len();
        let changed = edit(self);
        let after = lengths(&self.history);
        let full = after.0 >= HISTORY_LIMIT;
        if before != after || (changed && full) {
            self.doc_revision = self.doc_revision.wrapping_add(1);
            if !(appends && self.doc.len() > len) {
                self.doc_base_revision = self.doc_revision;
            }
        }
        changed
    }

    /// Delivers one pointer event to `tool`.
    fn send_pointer(&mut self, tool: Tool, phase: Phase, pos: Vec2, mods: Modifiers) -> bool {
        let pointer = Pointer {
            phase,
            pos,
            mods,
            pressure: self.pressure,
        };
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

    /// Handles a key while a text block is open: editing keys edit it,
    /// `Ctrl+[` / `Ctrl+]` resize it, `Esc` and `Ctrl+Enter` commit it,
    /// `Ctrl+Z` cancels it, in a grid cell `Tab` and `Enter` (with or
    /// without `Shift`) move to another cell, every other key does nothing
    /// (ADR-T29-2, ADR-T29-3, ADR-T31-1). Returns whether a redraw is
    /// needed.
    fn text_key(&mut self, key: Key, mods: Modifiers) -> bool {
        let ctrl_only = Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        };
        let shift_only = Modifiers {
            shift: true,
            ..Modifiers::NONE
        };
        let in_cell = self
            .text_edit()
            .and_then(TextEdit::host)
            .is_some_and(|host| matches!(host.slot, TextSlot::Cell { .. }));
        match key {
            Key::Escape => return self.finish_text(),
            Key::Enter if mods.ctrl => return self.finish_text(),
            Key::Z if mods == ctrl_only => return self.cancel_text(),
            // ADR-T31-1: Tab and Enter walk the cells of a grid.
            Key::Tab | Key::Enter if in_cell && (mods == Modifiers::NONE || mods == shift_only) => {
                let step = match (key, mods.shift) {
                    (Key::Tab, false) => CellStep::Next,
                    (Key::Tab, true) => CellStep::Prev,
                    (_, false) => CellStep::Down,
                    (_, true) => CellStep::Up,
                };
                return self.step_cell(step);
            }
            _ => {}
        }
        let Some(edit) = &mut self.tools.text.edit else {
            return false;
        };
        match key {
            // ADR-T29-3: grow or shrink the open block.
            Key::BracketLeft | Key::BracketRight if mods == ctrl_only => {
                edit.step_size(key == Key::BracketRight, self.camera.zoom())
            }
            Key::Enter => edit.insert('\n'),
            Key::Tab => edit.tab(),
            Key::Backspace => edit.backspace(),
            Key::Delete => edit.delete(),
            Key::ArrowLeft => edit.left(),
            Key::ArrowRight => edit.right(),
            Key::ArrowUp => edit.up(),
            Key::ArrowDown => edit.down(),
            Key::Home => edit.home(),
            Key::End => edit.end(),
            _ => false,
        }
    }

    /// Ends the open text block, committing it as one undo step: a new
    /// block is inserted, a changed one replaced, a blank one removed
    /// (ADR-T29-2). A resized block's on-screen size becomes the size of
    /// new blocks (ADR-T37-1). Returns whether there was a block.
    fn finish_text(&mut self) -> bool {
        let Some(edit) = self.tools.text.edit.take() else {
            return false;
        };
        let px = edit.size * self.camera.zoom();
        if edit.resized() && px.is_finite() && px > 0.0 {
            self.tools.text.size_px = Some(px);
        }
        if let Some(host) = edit.host().copied() {
            self.track_document(false, |ed| {
                let Some(shape) = ed.doc.get(host.id) else {
                    return false;
                };
                let changed = shape.clone().with_slot_text(host.slot, edit.shape_text());
                if &changed == shape {
                    return false;
                }
                let tx = tx_replace(&ed.doc, host.id, changed);
                !tx.is_empty() && ed.history.commit(&mut ed.doc, tx).is_ok()
            });
            return true;
        }
        let appends = edit.target().is_none();
        self.track_document(appends, |ed| {
            let target = edit.target().filter(|id| ed.doc.get(*id).is_some());
            let tx = match target {
                Some(id) if edit.is_blank() => tx_remove(&ed.doc, &[id]),
                Some(id) if ed.doc.get(id) == Some(&edit.shape()) => return false,
                Some(id) => tx_replace(&ed.doc, id, edit.shape()),
                None if edit.is_blank() => return false,
                None => tx_insert(&mut ed.doc, [edit.shape()]),
            };
            let changed = !tx.is_empty() && ed.history.commit(&mut ed.doc, tx).is_ok();
            ed.prune_selection();
            changed
        });
        true
    }

    /// Commits the open grid cell and opens the cell one `step` away, its
    /// caret at the end; a new cell takes the size of the one left. Does
    /// nothing, returning `false`, when there is no such cell (ADR-T31-1).
    fn step_cell(&mut self, step: CellStep) -> bool {
        let Some(edit) = &self.tools.text.edit else {
            return false;
        };
        let Some(host) = edit.host().copied() else {
            return false;
        };
        let TextSlot::Cell { col, row } = host.slot else {
            return false;
        };
        let size = edit.size;
        let next = self
            .doc
            .get(host.id)
            .and_then(|shape| shape.grid_cell_step(col, row, step));
        let Some((col, row)) = next else {
            return false;
        };
        self.finish_text();
        let slot = TextSlot::Cell { col, row };
        self.tools.text.edit = self
            .doc
            .get(host.id)
            .and_then(|shape| TextEdit::open_in(host.id, shape, slot, size));
        true
    }

    /// Throws the open text block away. Returns whether there was one.
    fn cancel_text(&mut self) -> bool {
        self.tools.text.edit.take().is_some()
    }

    /// The text block hit at world `p` (tolerance `tol`), if it is the
    /// topmost shape there and the only selected one. With shape text on,
    /// also the only selected shape if it holds text at `p` (ADR-T31-1).
    fn selected_text_at(&self, p: Vec2, tol: f32) -> Option<ShapeId> {
        let &[selected] = self.selection.as_slice() else {
            return None;
        };
        let hosts = self.experimental().shape_text;
        match self.doc.topmost_where(|shape| shape.hit(p, tol)) {
            Some(id) if id == selected => {
                let shape = self.doc.get(id)?;
                let is_text = matches!(shape, Shape::Text { .. });
                (is_text || (hosts && shape.text_slot_at(p).is_some())).then_some(id)
            }
            None if hosts => {
                let shape = self.doc.get(selected)?;
                shape.text_slot_at(p).is_some().then_some(selected)
            }
            Some(_) | None => None,
        }
    }

    /// Re-opens the text block of a Select press armed in
    /// [`Editor::pointer_down`] if the release at `pos` makes it a click
    /// and the block is still the only selection. Returns whether it did.
    fn reopen_clicked_text(&mut self, pos: Vec2) -> bool {
        let Some((start, id)) = self.reedit.take() else {
            return false;
        };
        if start.distance(pos) >= select::CLICK_SLOP_PX || self.selection != [id] {
            return false;
        }
        let at = self.camera.screen_to_world(pos);
        let size = self
            .camera
            .world_len(self.tools.text.new_size_px(self.style.width_px));
        let hosts = self.experimental().shape_text;
        self.tools.text.edit = self
            .doc
            .get(id)
            .and_then(|shape| match shape.text_slot_at(at) {
                Some(slot) if hosts => TextEdit::open_in(id, shape, slot, size),
                _ => TextEdit::open(id, shape, at),
            });
        self.tools.text.edit.is_some()
    }

    /// Discards the gesture in progress. Returns whether there was one.
    fn cancel_gesture(&mut self) -> bool {
        self.reedit = None;
        match self.gesture.take() {
            Some(Gesture::Tool { tool, .. }) => {
                self.tools.cancel(tool);
                true
            }
            Some(Gesture::Pan { .. }) => true,
            None => false,
        }
    }

    /// Sets the stroke width if `width` is `Some`, and forgets the
    /// remembered text size either way (ADR-T37-1). Returns whether the
    /// width changed.
    fn set_width(&mut self, width: Option<f32>) -> bool {
        self.tools.text.size_px = None;
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

/// Whether `tool` commits only new shapes on top, on pointer up.
fn creates_shapes(tool: Tool) -> bool {
    matches!(
        tool,
        Tool::Pen | Tool::Line | Tool::Arrow | Tool::Rect | Tool::Ellipse | Tool::Grid
    )
}

/// Whether `command` can change the document (through the history).
fn edits_document(command: Command) -> bool {
    matches!(
        command,
        Command::Undo
            | Command::Redo
            | Command::Cut
            | Command::Paste
            | Command::Duplicate
            | Command::DeleteSelection
            | Command::ClearAll
    )
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
    use crate::core::experimental::Experimental;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use crate::core::numbering::FIRST_NUMBER;
    use crate::core::palette::ColorId;
    use crate::core::shape::GRID_MAX_CELLS;
    use crate::core::shape::{Shape, ShapeText, Style, TextSlot};
    use crate::core::smoothing::SmoothingLevel;
    use crate::core::tap::PEN_TAP_SLOP_PX;

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
            text: None,
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
        assert_eq!(ed.smoothing(), SmoothingLevel::Low);
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

    #[test]
    fn barrel_pan_press_right_after_the_tip_takes_over() {
        // Arrange: a pen reports the tip before a held barrel button.
        let mut ed = Editor::new();
        down(&mut ed, PointerButton::Left, 10.0, 10.0);

        // Act
        down(&mut ed, PointerButton::Middle, 10.0, 10.0);
        move_to(&mut ed, 30.0, 40.0);
        up(&mut ed, PointerButton::Left, 30.0, 40.0);
        let panning = ed.active_gesture();
        up(&mut ed, PointerButton::Middle, 30.0, 40.0);

        // Assert
        assert_eq!(panning, Some(ActiveGesture::Pan));
        assert!(origin_on_screen(&ed).approx_eq(Vec2::new(20.0, 30.0), 1e-4));
        assert!(ed.document().is_empty());
        assert_eq!(ed.active_gesture(), None);
    }

    #[test]
    fn barrel_erase_press_right_after_the_tip_takes_over() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        down(&mut ed, PointerButton::Left, 0.0, 5.0);

        // Act
        down(&mut ed, PointerButton::Right, 0.0, 5.0);
        move_to(&mut ed, 0.0, 6.0);
        up(&mut ed, PointerButton::Left, 0.0, 6.0);
        up(&mut ed, PointerButton::Right, 0.0, 6.0);

        // Assert: the shape is erased and no pen stroke was added.
        assert!(ed.document().is_empty());
        assert_eq!(ed.active_gesture(), None);
    }

    #[test]
    fn barrel_press_after_the_tip_moved_does_not_take_over() {
        // Arrange
        let mut ed = Editor::new();
        down(&mut ed, PointerButton::Left, 0.0, 0.0);
        move_to(&mut ed, 20.0, 0.0);

        // Act
        down(&mut ed, PointerButton::Middle, 20.0, 0.0);

        // Assert
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Tool(Tool::Pen)));
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
        assert_eq!(ed.smoothing(), SmoothingLevel::Medium);
        ed.apply(Command::CycleSmoothing);
        assert_eq!(ed.smoothing(), SmoothingLevel::High);
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

    /// Whether the shapes are grids with indices, `None` for other shapes.
    fn grid_axes(ed: &Editor) -> Vec<Option<bool>> {
        ed.document()
            .shapes()
            .map(|(_, s)| match s {
                Shape::Grid { axes, .. } => Some(*axes),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn numbered_grid_keeps_counter() {
        // Arrange: numbering on, counter at 5.
        let mut ed = numbering_editor();
        for i in 0..4 {
            draw_node(&mut ed, 100.0 * i as f32);
        }

        // Act: circle 5, a grid, square 6.
        draw_node(&mut ed, 400.0);
        ed.apply(Command::SetTool(Tool::Grid));
        draw_node(&mut ed, 500.0);
        ed.apply(Command::SetTool(Tool::Rect));
        draw_node(&mut ed, 600.0);

        // Assert
        let labels = labels(&ed);
        assert_eq!(labels[4..], [Some(5), None, Some(6)]);
        assert_eq!(grid_axes(&ed)[5], Some(true), "numbered grid has indices");
        assert_eq!(ed.helpers().next_number, 7);
    }

    #[test]
    fn undo_numbered_grid_keeps_counter() {
        // Arrange
        let mut ed = numbering_editor();
        draw_node(&mut ed, 0.0);
        ed.apply(Command::SetTool(Tool::Grid));
        draw_node(&mut ed, 100.0);
        assert_eq!(ed.helpers().next_number, 2);

        // Act / Assert
        assert!(ed.apply(Command::Undo));
        assert_eq!(ed.helpers().next_number, 2);
        assert!(ed.apply(Command::Redo));
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

    // ---- T24 AC-2 document revision ---------------------------------------

    /// Left click (down and up without moving) at screen `(x, y)`.
    fn click(ed: &mut Editor, x: f32, y: f32) -> bool {
        let pressed = down(ed, PointerButton::Left, x, y);
        up(ed, PointerButton::Left, x, y) || pressed
    }

    #[test]
    fn document_revision_new_editor_is_zero() {
        assert_eq!(Editor::new().document_revision(), 0);
    }

    #[test]
    fn document_revision_pen_drag_moves_keep_revision() {
        // Arrange
        let mut ed = Editor::new();
        down(&mut ed, PointerButton::Left, 0.0, 0.0);
        let before = ed.document_revision();

        // Act
        for i in 1..20u8 {
            assert!(move_to(&mut ed, f32::from(i) * 3.0, f32::from(i % 4)));
        }

        // Assert
        assert_eq!(ed.document_revision(), before);
    }

    #[test]
    fn document_revision_pen_commit_bumps_revision() {
        // Arrange
        let mut ed = Editor::new();
        down(&mut ed, PointerButton::Left, 0.0, 0.0);
        move_to(&mut ed, 30.0, 10.0);
        let before = ed.document_revision();

        // Act
        up(&mut ed, PointerButton::Left, 60.0, 0.0);

        // Assert
        assert_eq!(ed.document().len(), 1);
        assert!(ed.document_revision() > before);
    }

    #[test]
    fn document_revision_select_click_and_marquee_keep_revision() {
        // Arrange
        let mut ed = editor_with(vec![
            rect(0.0, 0.0, 10.0, 10.0),
            rect(50.0, 50.0, 10.0, 10.0),
        ]);
        ed.apply(Command::SetTool(Tool::Select));
        let before = ed.document_revision();

        // Act: click a shape, then marquee both from empty space.
        assert!(click(&mut ed, 0.0, 5.0));
        down(&mut ed, PointerButton::Left, -20.0, -20.0);
        assert!(move_to(&mut ed, 100.0, 100.0));
        assert!(up(&mut ed, PointerButton::Left, 100.0, 100.0));
        assert!(ed.apply(Command::Cancel));

        // Assert
        assert_eq!(ed.document_revision(), before);
    }

    #[test]
    fn document_revision_select_all_and_copy_keep_revision() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let before = ed.document_revision();

        // Act
        assert!(ed.apply(Command::SelectAll));
        ed.apply(Command::Copy);

        // Assert
        assert_eq!(ed.selection().len(), 1);
        assert_eq!(ed.document_revision(), before);
    }

    #[test]
    fn document_revision_move_drag_bumps_only_on_release() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        ed.apply(Command::SetTool(Tool::Select));
        let before = ed.document_revision();

        // Act / Assert: pressing and dragging only change the overlay.
        down(&mut ed, PointerButton::Left, 0.0, 5.0);
        move_to(&mut ed, 20.0, 5.0);
        move_to(&mut ed, 40.0, 25.0);
        assert_eq!(ed.document_revision(), before);
        assert!(!ed.overlay().hidden.is_empty());

        // Act / Assert: releasing commits the move.
        up(&mut ed, PointerButton::Left, 40.0, 25.0);
        assert!(ed.document_revision() > before);
    }

    #[test]
    fn document_revision_delete_paste_duplicate_bump_revision() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        ed.apply(Command::SelectAll);

        for command in [
            Command::Duplicate,
            Command::Copy,
            Command::Paste,
            Command::DeleteSelection,
        ] {
            // Arrange
            let before = ed.document_revision();
            let len = ed.document().len();

            // Act
            ed.apply(command);

            // Assert
            if command == Command::Copy {
                assert_eq!(ed.document_revision(), before, "{command:?}");
            } else {
                assert_ne!(ed.document().len(), len, "{command:?}");
                assert!(ed.document_revision() > before, "{command:?}");
            }
        }
    }

    #[test]
    fn document_revision_undo_redo_clear_bump_revision() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);

        for command in [Command::Undo, Command::Redo, Command::ClearAll] {
            // Arrange
            let before = ed.document_revision();

            // Act
            assert!(ed.apply(command), "{command:?}");

            // Assert
            assert!(ed.document_revision() > before, "{command:?}");
        }
    }

    #[test]
    fn document_revision_noop_undo_keeps_revision() {
        // Arrange
        let mut ed = Editor::new();

        // Act
        ed.apply(Command::Undo);
        ed.apply(Command::Redo);
        ed.apply(Command::ClearAll);
        ed.apply(Command::DeleteSelection);
        ed.apply(Command::Paste);

        // Assert
        assert_eq!(ed.document_revision(), 0);
    }

    #[test]
    fn document_revision_eraser_bumps_only_on_release() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let before = ed.document_revision();

        // Act / Assert: marking hides the shape but keeps the document.
        down(&mut ed, PointerButton::Right, 0.0, 5.0);
        move_to(&mut ed, 0.0, 6.0);
        assert_eq!(ed.overlay().hidden.len(), 1);
        assert_eq!(ed.document_revision(), before);

        // Act / Assert: releasing removes it.
        up(&mut ed, PointerButton::Right, 0.0, 6.0);
        assert!(ed.document().is_empty());
        assert!(ed.document_revision() > before);
    }

    #[test]
    fn document_revision_bucket_fill_bumps_revision() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        ed.apply(Command::SetTool(Tool::Bucket));
        let before = ed.document_revision();

        // Act
        assert!(click(&mut ed, 5.0, 5.0));

        // Assert
        assert!(ed.document_revision() > before);
    }

    #[test]
    fn document_revision_view_and_settings_keep_revision() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let before = ed.document_revision();

        // Act: resize, pan, zoom, and every setting command.
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        down(&mut ed, PointerButton::Middle, 0.0, 0.0);
        move_to(&mut ed, 30.0, 20.0);
        up(&mut ed, PointerButton::Middle, 30.0, 20.0);
        ed.handle(InputEvent::Scroll {
            pos: Vec2::new(5.0, 5.0),
            delta: 1.0,
        });
        for command in [
            Command::SetTool(Tool::Rect),
            Command::SetColor(ColorId::INK),
            Command::WidthUp,
            Command::WidthDown,
            Command::CycleSmoothing,
            Command::ToggleToolbar,
            Command::ToggleSmartSnap,
            Command::ToggleGridSnap,
            Command::ToggleNumbering,
            Command::ResetNumbering,
            Command::GridCols(1),
            Command::GridRows(-1),
            Command::FitView,
            Command::ResetView,
            Command::Cancel,
        ] {
            ed.apply(command);
        }
        move_to(&mut ed, 1.0, 1.0);

        // Assert
        assert_eq!(ed.document_revision(), before);
    }

    #[test]
    fn document_revision_commit_with_full_history_bumps_revision() {
        // Arrange: fill the undo stack with pen dots.
        let mut ed = Editor::new();
        for i in 0..HISTORY_LIMIT {
            click(&mut ed, i as f32, 0.0);
        }
        assert_eq!(ed.history.undo_len(), HISTORY_LIMIT);
        let before = ed.document_revision();

        // Act: one more commit keeps both stack lengths.
        click(&mut ed, -10.0, -10.0);

        // Assert
        assert_eq!(ed.history.undo_len(), HISTORY_LIMIT);
        assert!(ed.document_revision() > before);
    }

    // ---- T24 AC-3c base revision -------------------------------------------

    /// Left drag from screen `(x, y)` to `(x + 40, y + 30)`.
    fn drag(ed: &mut Editor, x: f32, y: f32) {
        down(ed, PointerButton::Left, x, y);
        move_to(ed, x + 20.0, y + 15.0);
        move_to(ed, x + 40.0, y + 30.0);
        up(ed, PointerButton::Left, x + 40.0, y + 30.0);
    }

    #[test]
    fn document_base_revision_new_editor_is_zero() {
        assert_eq!(Editor::new().document_base_revision(), 0);
    }

    #[test]
    fn document_base_revision_creation_commits_keep_base() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let tools = [
            Tool::Pen,
            Tool::Line,
            Tool::Arrow,
            Tool::Rect,
            Tool::Ellipse,
            Tool::Grid,
        ];

        for (i, tool) in tools.into_iter().enumerate() {
            // Arrange
            ed.apply(Command::SetTool(tool));
            let (revision, base) = (ed.document_revision(), ed.document_base_revision());
            let len = ed.document().len();

            // Act
            drag(&mut ed, 100.0 + 100.0 * i as f32, 50.0);

            // Assert
            assert_eq!(ed.document().len(), len + 1, "{tool:?}");
            assert!(ed.document_revision() > revision, "{tool:?}");
            assert_eq!(ed.document_base_revision(), base, "{tool:?}");
        }
    }

    #[test]
    fn document_base_revision_paste_and_duplicate_keep_base() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        ed.apply(Command::SelectAll);
        ed.apply(Command::Copy);
        let (revision, base) = (ed.document_revision(), ed.document_base_revision());

        // Act
        ed.apply(Command::Paste);
        ed.apply(Command::Duplicate);

        // Assert
        assert_eq!(ed.document().len(), 3);
        assert!(ed.document_revision() > revision);
        assert_eq!(ed.document_base_revision(), base);
    }

    /// A named edit that rewrites the document.
    type Rewrite = (&'static str, fn(&mut Editor));

    #[test]
    fn document_base_revision_rewrites_raise_base() {
        let rewrites: [Rewrite; 8] = [
            ("undo", |ed| {
                ed.apply(Command::Undo);
            }),
            ("redo", |ed| {
                ed.apply(Command::Undo);
                ed.apply(Command::Redo);
            }),
            ("delete", |ed| {
                ed.apply(Command::SelectAll);
                ed.apply(Command::DeleteSelection);
            }),
            ("cut", |ed| {
                ed.apply(Command::SelectAll);
                ed.apply(Command::Cut);
            }),
            ("clear", |ed| {
                ed.apply(Command::ClearAll);
            }),
            ("move", |ed| {
                ed.apply(Command::SetTool(Tool::Select));
                drag(ed, 0.0, 5.0);
            }),
            ("bucket", |ed| {
                ed.apply(Command::SetTool(Tool::Bucket));
                down(ed, PointerButton::Left, 5.0, 5.0);
                up(ed, PointerButton::Left, 5.0, 5.0);
            }),
            ("eraser", |ed| {
                down(ed, PointerButton::Right, 0.0, 5.0);
                up(ed, PointerButton::Right, 0.0, 5.0);
            }),
        ];

        for (name, rewrite) in rewrites {
            // Arrange: one shape committed through the editor (an append).
            let mut ed = Editor::new();
            ed.apply(Command::SetTool(Tool::Rect));
            down(&mut ed, PointerButton::Left, 0.0, 0.0);
            move_to(&mut ed, 10.0, 10.0);
            up(&mut ed, PointerButton::Left, 10.0, 10.0);
            let revision = ed.document_revision();
            assert_eq!(ed.document_base_revision(), 0, "{name}");

            // Act
            rewrite(&mut ed);

            // Assert
            assert!(ed.document_revision() > revision, "{name}");
            assert_eq!(
                ed.document_base_revision(),
                ed.document_revision(),
                "{name}"
            );
        }
    }

    // ---- T29 text blocks ---------------------------------------------------

    const CTRL: Modifiers = Modifiers {
        shift: false,
        ctrl: true,
        alt: false,
    };

    /// Editor with an 800 × 600 viewport and the Text tool.
    fn text_editor() -> Editor {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        ed.apply(Command::SetTool(Tool::Text));
        ed
    }

    /// Types `s` as the shell does: one `Char` per character.
    fn typed(ed: &mut Editor, s: &str) {
        for ch in s.chars() {
            ed.handle(InputEvent::Char { ch, mods: NONE });
        }
    }

    fn key_with(ed: &mut Editor, key: Key, mods: Modifiers) -> bool {
        ed.handle(InputEvent::KeyDown { key, mods })
    }

    /// Clicks at screen `(x, y)` and types `s` (the block stays open).
    fn place(ed: &mut Editor, x: f32, y: f32, s: &str) {
        click(ed, x, y);
        typed(ed, s);
    }

    /// The text of every text block in the document, bottom first.
    fn texts(ed: &Editor) -> Vec<String> {
        ed.document()
            .shapes()
            .filter_map(|(_, shape)| match shape {
                Shape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    /// The text being edited, if any.
    fn editing(ed: &Editor) -> Option<String> {
        ed.text_edit().map(|edit| edit.text().to_owned())
    }

    /// Screen x of caret column `col` of a block at screen x `x0`, at the
    /// default width (18 px text) and zoom 1.
    fn column_x(x0: f32, col: f32) -> f32 {
        x0 + col * 18.0 * crate::core::text_layout::ADVANCE
    }

    /// The character a bare or `Shift` chord types, if any.
    fn chord_char(key: Key, mods: Modifiers) -> Option<char> {
        if mods.ctrl || mods.alt {
            return None;
        }
        let name = format!("{key:?}");
        let ch = match key {
            Key::Space => ' ',
            Key::BracketLeft => '[',
            Key::BracketRight => ']',
            _ if name.len() == 1 => name.chars().next()?.to_ascii_lowercase(),
            _ => name.strip_prefix("Digit")?.chars().next()?,
        };
        Some(if mods.shift {
            ch.to_ascii_uppercase()
        } else {
            ch
        })
    }

    #[test]
    fn text_click_starts_block_at_pointer() {
        // Arrange
        let mut ed = text_editor();

        // Act
        let changed = click(&mut ed, 100.0, 50.0);

        // Assert
        assert!(changed);
        let Some(edit) = ed.text_edit() else {
            panic!("the click opens a block");
        };
        let world = ed.camera().screen_to_world(Vec2::new(100.0, 50.0));
        assert!(edit.origin.approx_eq(world, 1e-4), "{edit:?}");
        assert_eq!(edit.text(), "");
        assert_eq!(edit.caret(), 0);
        assert_eq!(edit.target(), None);
        assert!(ed.document().is_empty());
    }

    #[test]
    fn text_typed_chars_appear_in_overlay() {
        // Arrange
        let mut ed = text_editor();

        // Act
        place(&mut ed, 100.0, 50.0, "hi ∑");

        // Assert
        let overlay = ed.overlay();
        assert!(
            overlay
                .shapes
                .iter()
                .any(|s| matches!(s, Shape::Text { text, .. } if text == "hi ∑")),
            "{overlay:?}"
        );
        assert!(ed.document().is_empty());
    }

    #[test]
    fn text_escape_commits_one_shape() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "abc");

        // Act
        let changed = key(&mut ed, Key::Escape);

        // Assert
        assert!(changed);
        assert!(ed.text_edit().is_none());
        assert_eq!(texts(&ed), ["abc"]);
        assert_eq!(ed.document().len(), 1);
        assert!(ed.can_undo());
    }

    #[test]
    fn text_ctrl_enter_commits() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "abc");

        assert!(key_with(&mut ed, Key::Enter, CTRL));

        assert!(ed.text_edit().is_none());
        assert_eq!(texts(&ed), ["abc"]);
    }

    #[test]
    fn text_click_outside_commits_and_starts_new_block() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");

        // Act
        click(&mut ed, 400.0, 300.0);

        // Assert
        assert_eq!(texts(&ed), ["a"]);
        let Some(edit) = ed.text_edit() else {
            panic!("the click opens the next block");
        };
        assert!(edit.origin.approx_eq(Vec2::new(400.0, 300.0), 1e-4));
        assert_eq!(edit.text(), "");
    }

    #[test]
    fn text_empty_block_leaves_no_shape_and_no_history() {
        let mut ed = text_editor();

        click(&mut ed, 100.0, 50.0);
        click(&mut ed, 300.0, 50.0);
        key(&mut ed, Key::Escape);

        assert!(ed.document().is_empty());
        assert!(!ed.can_undo());
        assert!(ed.text_edit().is_none());
    }

    #[test]
    fn text_whitespace_only_block_is_discarded() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "  ");
        key(&mut ed, Key::Enter);
        key(&mut ed, Key::Tab);

        key(&mut ed, Key::Escape);

        assert!(ed.document().is_empty());
        assert!(!ed.can_undo());
    }

    #[test]
    fn text_new_block_size_follows_width_and_zoom() {
        // Arrange: one notch of zoom, one step up the width ladder (4 px).
        let mut ed = text_editor();
        ed.handle(InputEvent::Scroll {
            pos: Vec2::new(10.0, 10.0),
            delta: 1.0,
        });
        ed.apply(Command::WidthUp);

        // Act
        place(&mut ed, 100.0, 50.0, "x");
        key(&mut ed, Key::Escape);

        // Assert: 12 + 2 × 4 = 20 px on screen.
        let Some((_, Shape::Text { size, .. })) = ed.document().shapes().next() else {
            panic!("a text block was committed");
        };
        assert!(approx_eq(*size, 20.0 / ZOOM_STEP, 1e-4), "{size}");
    }

    #[test]
    fn text_new_block_uses_current_colour() {
        let Some(red) = ColorId::new(2) else {
            panic!("palette has colour 2");
        };
        let mut ed = text_editor();
        ed.apply(Command::SetColor(red));

        place(&mut ed, 100.0, 50.0, "x");
        key(&mut ed, Key::Escape);

        let Some((_, shape)) = ed.document().shapes().next() else {
            panic!("a text block was committed");
        };
        assert_eq!(shape.style().color, red);
    }

    #[test]
    fn text_edit_mode_every_binding_types_instead_of_acting() {
        for &(chord, command, _) in keymap::BINDINGS {
            // Arrange: an open block holding "x".
            let mut ed = text_editor();
            place(&mut ed, 100.0, 50.0, "x");
            let (style, smoothing, camera) = (ed.style(), ed.smoothing(), *ed.camera());

            // Act: the shell sends the character, then the key.
            if let Some(ch) = chord_char(chord.key, chord.mods) {
                ed.handle(InputEvent::Char {
                    ch,
                    mods: chord.mods,
                });
            }
            key_with(&mut ed, chord.key, chord.mods);

            // Assert
            let label = format!("{chord:?} -> {command:?}");
            match command {
                Command::Cancel => {
                    assert_eq!(texts(&ed), ["x"], "{label}: Esc commits");
                    continue;
                }
                Command::Undo => {
                    assert!(ed.document().is_empty(), "{label}: Ctrl+Z cancels");
                    assert!(ed.text_edit().is_none(), "{label}");
                    continue;
                }
                _ => {}
            }
            assert!(ed.text_edit().is_some(), "{label}: still editing");
            assert!(ed.document().is_empty(), "{label}: nothing committed");
            assert_eq!(ed.tool(), Tool::Text, "{label}");
            assert_eq!(ed.style(), style, "{label}");
            assert_eq!(ed.smoothing(), smoothing, "{label}");
            assert_eq!(*ed.camera(), camera, "{label}");
            assert!(ed.toolbar_visible(), "{label}");
            assert!(ed.selection().is_empty(), "{label}");
            if let Some(ch) = chord_char(chord.key, chord.mods) {
                let text = editing(&ed).unwrap_or_default();
                assert!(text.ends_with(ch), "{label}: typed {text:?}");
            }
        }
    }

    #[test]
    fn text_after_escape_keys_act_as_shortcuts_again() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "p");
        key(&mut ed, Key::Escape);

        ed.handle(InputEvent::Char {
            ch: 'p',
            mods: NONE,
        });
        key(&mut ed, Key::D);

        assert_eq!(ed.tool(), Tool::Pen);
        assert_eq!(texts(&ed), ["p"]);
    }

    #[test]
    fn text_left_hand_keys_type_letters() {
        // Arrange
        let mut ed = text_editor();
        click(&mut ed, 100.0, 50.0);

        // Act: the shell sends a key press and a character for each.
        for (k, ch) in [(Key::D, 'd'), (Key::W, 'w'), (Key::X, 'x')] {
            key(&mut ed, k);
            ed.handle(InputEvent::Char { ch, mods: NONE });
        }
        key(&mut ed, Key::Escape);

        // Assert
        assert_eq!(ed.tool(), Tool::Text);
        assert_eq!(ed.helpers().smart_snap, Editor::new().helpers().smart_snap);
        assert_eq!(texts(&ed), ["dwx"]);
    }

    #[test]
    fn text_ctrl_chars_are_not_inserted() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");

        ed.handle(InputEvent::Char {
            ch: 'c',
            mods: CTRL,
        });
        for ch in ['\u{8}', '\r', '\t', '\u{1b}', '\u{7f}'] {
            ed.handle(InputEvent::Char { ch, mods: NONE });
        }

        assert_eq!(editing(&ed).as_deref(), Some("a"));
    }

    #[test]
    fn text_tab_types_four_spaces() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");

        key(&mut ed, Key::Tab);

        assert_eq!(editing(&ed).as_deref(), Some("a    "));
        assert!(ed.toolbar_visible());
    }

    #[test]
    fn text_space_in_edit_mode_does_not_pan() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");
        ed.handle(InputEvent::Char {
            ch: ' ',
            mods: NONE,
        });
        key(&mut ed, Key::Space);

        // Act: a left drag elsewhere.
        down(&mut ed, PointerButton::Left, 300.0, 300.0);
        move_to(&mut ed, 350.0, 320.0);
        up(&mut ed, PointerButton::Left, 350.0, 320.0);

        // Assert: no pan; the block was committed and a new one opened.
        assert_eq!(*ed.camera(), Camera::default());
        assert_eq!(texts(&ed), ["a "]);
        assert!(ed.text_edit().is_some());
    }

    #[test]
    fn text_char_outside_edit_mode_is_ignored() {
        let mut ed = text_editor();

        assert!(!ed.handle(InputEvent::Char {
            ch: 'p',
            mods: NONE
        }));

        assert!(ed.document().is_empty());
        assert!(ed.text_edit().is_none());
    }

    #[test]
    fn text_editing_keys_edit_the_block() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "aβc");

        // Act
        key(&mut ed, Key::ArrowLeft);
        key(&mut ed, Key::Backspace);
        key(&mut ed, Key::Home);
        key(&mut ed, Key::Delete);
        key(&mut ed, Key::End);
        key(&mut ed, Key::Enter);
        typed(&mut ed, "de");
        key(&mut ed, Key::ArrowUp);
        typed(&mut ed, "!");
        key(&mut ed, Key::ArrowDown);
        key(&mut ed, Key::ArrowRight);
        typed(&mut ed, "?");

        // Assert
        assert_eq!(editing(&ed).as_deref(), Some("c!\nde?"));
    }

    #[test]
    fn text_ctrl_brackets_resize_open_block() {
        // Arrange: default width gives 18 px at zoom 1.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "ab");
        let width = ed.style().width_px;

        // Act: the shell sends a control character, then the key.
        ed.handle(InputEvent::Char {
            ch: '\u{1d}',
            mods: CTRL,
        });
        assert!(key_with(&mut ed, Key::BracketRight, CTRL));
        key_with(&mut ed, Key::BracketLeft, CTRL);
        key_with(&mut ed, Key::BracketLeft, CTRL);

        // Assert
        let Some(edit) = ed.text_edit() else {
            panic!("still editing");
        };
        assert_eq!(edit.text(), "ab");
        assert!(approx_eq(edit.size, 16.0, 1e-4), "{}", edit.size);
        assert!(approx_eq(ed.style().width_px, width, 1e-6));
        key(&mut ed, Key::Escape);
        let Some((_, Shape::Text { size, .. })) = ed.document().shapes().next() else {
            panic!("a text block was committed");
        };
        assert!(approx_eq(*size, 16.0, 1e-4));
    }

    #[test]
    fn text_tool_click_on_block_reopens_it_with_caret_at_click() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abcd");
        key(&mut ed, Key::Escape);
        let ids = ids(&ed);

        // Act: just right of column 2.
        click(&mut ed, column_x(100.0, 2.0) + 1.0, 105.0);

        // Assert
        let Some(edit) = ed.text_edit() else {
            panic!("the click re-opens the block");
        };
        assert_eq!(edit.target(), Some(ids[0]));
        assert_eq!(edit.text(), "abcd");
        assert_eq!(edit.caret(), 2);
        assert_eq!(ed.document().len(), 1);
    }

    #[test]
    fn select_click_on_selected_block_reopens_it() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abcd");
        key(&mut ed, Key::Escape);
        ed.apply(Command::SetTool(Tool::Select));
        ed.apply(Command::SelectAll);
        let ids = ids(&ed);

        // Act
        click(&mut ed, column_x(100.0, 1.0), 105.0);

        // Assert
        let Some(edit) = ed.text_edit() else {
            panic!("a click on the selected block edits it");
        };
        assert_eq!(edit.target(), Some(ids[0]));
        assert_eq!(edit.caret(), 1);
        assert_eq!(ed.tool(), Tool::Select);
    }

    #[test]
    fn select_double_click_on_block_reopens_it() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abcd");
        key(&mut ed, Key::Escape);
        ed.apply(Command::SetTool(Tool::Select));

        click(&mut ed, 110.0, 105.0);
        click(&mut ed, 110.0, 105.0);

        assert_eq!(editing(&ed).as_deref(), Some("abcd"));
    }

    #[test]
    fn select_click_on_unselected_block_only_selects() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abcd");
        key(&mut ed, Key::Escape);
        ed.apply(Command::SetTool(Tool::Select));

        click(&mut ed, 110.0, 105.0);

        assert!(ed.text_edit().is_none());
        assert_eq!(ed.selection(), ids(&ed));
    }

    #[test]
    fn text_reedit_hides_original_while_editing() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abcd");
        key(&mut ed, Key::Escape);
        let ids = ids(&ed);

        // Act
        click(&mut ed, 110.0, 105.0);
        typed(&mut ed, "X");

        // Assert
        let overlay = ed.overlay();
        assert_eq!(overlay.hidden, ids);
        assert!(
            overlay
                .shapes
                .iter()
                .any(|s| matches!(s, Shape::Text { text, .. } if text == "aXbcd")),
            "{overlay:?}"
        );
        assert_eq!(texts(&ed), ["abcd"], "the document keeps the old text");
    }

    #[test]
    fn text_click_inside_edited_block_moves_caret() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abcd");

        click(&mut ed, column_x(100.0, 1.0), 105.0);

        let Some(edit) = ed.text_edit() else {
            panic!("still editing");
        };
        assert_eq!(edit.target(), None);
        assert_eq!(edit.caret(), 1);
        assert!(ed.document().is_empty());
    }

    #[test]
    fn text_edit_is_one_history_entry() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "hello");
        key(&mut ed, Key::Enter);
        typed(&mut ed, "world");
        key(&mut ed, Key::Escape);

        assert!(ed.apply(Command::Undo));

        assert!(ed.document().is_empty());
        assert!(!ed.can_undo());
    }

    #[test]
    fn text_undo_redo_restore_text_exactly() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");
        key(&mut ed, Key::Escape);
        click(&mut ed, 105.0, 105.0);
        key(&mut ed, Key::End);
        typed(&mut ed, "δ");
        key(&mut ed, Key::Escape);
        assert_eq!(texts(&ed), ["abcδ"]);

        // Act / Assert
        ed.apply(Command::Undo);
        assert_eq!(texts(&ed), ["abc"]);
        ed.apply(Command::Undo);
        assert!(texts(&ed).is_empty());
        ed.apply(Command::Redo);
        assert_eq!(texts(&ed), ["abc"]);
        ed.apply(Command::Redo);
        assert_eq!(texts(&ed), ["abcδ"]);
    }

    #[test]
    fn text_ctrl_z_while_editing_cancels_edit() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");
        key(&mut ed, Key::Escape);
        click(&mut ed, 105.0, 105.0);
        typed(&mut ed, "XYZ");

        // Act
        assert!(key_with(&mut ed, Key::Z, CTRL));

        // Assert: the old text is back and the next undo removes the block.
        assert!(ed.text_edit().is_none());
        assert_eq!(texts(&ed), ["abc"]);
        key_with(&mut ed, Key::Z, CTRL);
        assert!(ed.document().is_empty());

        // A new block vanishes.
        place(&mut ed, 300.0, 100.0, "q");
        key_with(&mut ed, Key::Z, CTRL);
        assert!(ed.document().is_empty());
        assert!(ed.text_edit().is_none());
    }

    #[test]
    fn text_reedit_unchanged_adds_no_history() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");
        key(&mut ed, Key::Escape);
        let revision = ed.document_revision();

        click(&mut ed, 105.0, 105.0);
        key(&mut ed, Key::Escape);

        assert_eq!(ed.document_revision(), revision);
        ed.apply(Command::Undo);
        assert!(ed.document().is_empty());
    }

    #[test]
    fn text_reedit_to_empty_removes_block_undoably() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");
        key(&mut ed, Key::Escape);

        click(&mut ed, 105.0, 105.0);
        key(&mut ed, Key::End);
        for _ in 0..3 {
            key(&mut ed, Key::Backspace);
        }
        key(&mut ed, Key::Escape);

        assert!(ed.document().is_empty());
        ed.apply(Command::Undo);
        assert_eq!(texts(&ed), ["abc"]);
    }

    #[test]
    fn text_toolbar_command_commits_edit_first() {
        let Some(red) = ColorId::new(2) else {
            panic!("palette has colour 2");
        };
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");

        ed.apply(Command::SetColor(red));
        assert!(ed.text_edit().is_none());
        assert_eq!(texts(&ed), ["abc"]);

        // Undo from the toolbar while editing cancels the edit instead.
        place(&mut ed, 300.0, 100.0, "zz");
        assert!(ed.apply(Command::Undo));
        assert!(ed.text_edit().is_none());
        assert_eq!(texts(&ed), ["abc"]);
    }

    #[test]
    fn text_commit_bumps_document_revision() {
        // Arrange
        let mut ed = text_editor();
        let (revision, base) = (ed.document_revision(), ed.document_base_revision());

        // Act / Assert: typing touches only the overlay.
        place(&mut ed, 100.0, 100.0, "abc");
        assert_eq!(ed.document_revision(), revision);
        key(&mut ed, Key::Escape);
        assert!(ed.document_revision() > revision);
        assert_eq!(
            ed.document_base_revision(),
            base,
            "a new block is appended on top"
        );

        // Re-editing replaces a shape: the base follows.
        click(&mut ed, 105.0, 105.0);
        typed(&mut ed, "d");
        key(&mut ed, Key::Escape);
        assert_eq!(ed.document_base_revision(), ed.document_revision());
    }

    #[test]
    fn text_click_snaps_origin_to_grid() {
        let mut ed = text_editor();
        ed.apply(Command::ToggleGridSnap);

        click(&mut ed, 107.0, 52.0);

        let Some(edit) = ed.text_edit() else {
            panic!("the click opens a block");
        };
        assert!(
            edit.origin.approx_eq(Vec2::new(100.0, 60.0), 1e-4),
            "{edit:?}"
        );
    }

    #[test]
    fn text_block_moves_duplicates_and_erases_like_other_shapes() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");
        key(&mut ed, Key::Escape);
        ed.apply(Command::SetTool(Tool::Select));

        // Act / Assert: move by (40, 30).
        drag(&mut ed, 105.0, 105.0);
        let origins = |ed: &Editor| -> Vec<Vec2> {
            ed.document()
                .shapes()
                .filter_map(|(_, s)| match s {
                    Shape::Text { origin, .. } => Some(*origin),
                    _ => None,
                })
                .collect()
        };
        assert!(origins(&ed)[0].approx_eq(Vec2::new(140.0, 130.0), 1e-3));

        // Alt-drag duplicates.
        ed.handle(InputEvent::PointerDown {
            pos: Vec2::new(145.0, 135.0),
            button: PointerButton::Left,
            mods: Modifiers { alt: true, ..NONE },
        });
        ed.handle(InputEvent::PointerUp {
            pos: Vec2::new(245.0, 135.0),
            button: PointerButton::Left,
            mods: NONE,
        });
        assert_eq!(texts(&ed), ["abc", "abc"]);

        // A right click erases one.
        down(&mut ed, PointerButton::Right, 145.0, 135.0);
        up(&mut ed, PointerButton::Right, 145.0, 135.0);
        assert_eq!(texts(&ed), ["abc"]);
        assert!(origins(&ed)[0].approx_eq(Vec2::new(240.0, 130.0), 1e-3));
    }

    #[test]
    fn text_block_copy_paste_keeps_text() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "a∑b");
        key(&mut ed, Key::Escape);

        ed.apply(Command::SelectAll);
        ed.apply(Command::Copy);
        ed.apply(Command::Paste);

        assert_eq!(texts(&ed), ["a∑b", "a∑b"]);
    }

    #[test]
    fn fit_view_includes_text_blocks() {
        // Arrange: a block far off screen.
        let mut ed = editor_with(vec![Shape::Text {
            origin: Vec2::new(5000.0, 4000.0),
            text: "far".to_owned(),
            size: 20.0,
            style: Style {
                color: ColorId::INK,
                width: 0.0,
            },
        }]);
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });

        // Act
        assert!(ed.apply(Command::FitView));

        // Assert
        let view = ed.camera().visible_world_rect(ed.viewport());
        let Some((_, shape)) = ed.document().shapes().next() else {
            panic!("one shape");
        };
        let bounds = shape.bounds();
        assert!(view.contains(bounds.min) && view.contains(bounds.max));
    }

    #[test]
    fn text_bucket_ignores_text_block() {
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");
        key(&mut ed, Key::Escape);
        ed.apply(Command::SetTool(Tool::Bucket));
        let revision = ed.document_revision();

        click(&mut ed, 105.0, 105.0);

        assert_eq!(ed.document_revision(), revision);
        assert_eq!(
            ed.document().shapes().next().and_then(|(_, s)| s.fill()),
            None
        );
    }

    #[test]
    fn text_caret_follows_typing() {
        // Arrange
        let mut ed = text_editor();
        click(&mut ed, 100.0, 50.0);
        let Some([top, bottom]) = ed.text_caret() else {
            panic!("editing shows a caret");
        };
        assert!(top.approx_eq(Vec2::new(100.0, 50.0), 1e-4));
        assert!(approx_eq(bottom.y - top.y, 18.0 * 1.25, 1e-3));

        // Act
        typed(&mut ed, "ab");

        // Assert
        let Some([top, _]) = ed.text_caret() else {
            panic!("still editing");
        };
        assert!(top.approx_eq(Vec2::new(column_x(100.0, 2.0), 50.0), 1e-3));
    }

    #[test]
    fn text_caret_none_when_not_editing() {
        let mut ed = text_editor();
        assert!(ed.text_caret().is_none());

        place(&mut ed, 100.0, 50.0, "a");
        key(&mut ed, Key::Escape);

        assert!(ed.text_caret().is_none());
    }
    // ---- T31 shape text -----------------------------------------------------

    const SHIFT: Modifiers = Modifiers {
        shift: true,
        ctrl: false,
        alt: false,
    };

    const SHAPE_TEXT_ON: Experimental = Experimental { shape_text: true };

    fn ellipse(x: f32, y: f32, w: f32, h: f32) -> Shape {
        Shape::Ellipse {
            a: Vec2::new(x, y),
            b: Vec2::new(x + w, y + h),
            style: Style {
                color: ColorId::INK,
                width: 1.0,
            },
            fill: None,
            label: None,
            text: None,
        }
    }

    /// Unfilled `cols × rows` grid over `(x, y)`–`(x + w, y + h)`.
    fn grid(x: f32, y: f32, w: f32, h: f32, cols: u32, rows: u32) -> Shape {
        Shape::Grid {
            a: Vec2::new(x, y),
            b: Vec2::new(x + w, y + h),
            cols,
            rows,
            style: Style {
                color: ColorId::INK,
                width: 1.0,
            },
            axes: false,
            fills: Vec::new(),
            texts: Vec::new(),
        }
    }

    fn cell(col: u32, row: u32) -> TextSlot {
        TextSlot::Cell { col, row }
    }

    /// Editor with an 800 × 600 viewport holding `shapes`, the shape text
    /// switch set to `on`, and `tool` active.
    fn shape_text_editor(shapes: Vec<Shape>, on: bool, tool: Tool) -> Editor {
        let experimental = if on {
            SHAPE_TEXT_ON
        } else {
            Experimental::default()
        };
        let mut ed = editor_with(shapes).with_experimental(experimental);
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        ed.apply(Command::SetTool(tool));
        ed
    }

    /// The text in `slot` of the `index`-th shape (bottom first).
    fn slot_text(ed: &Editor, index: usize, slot: TextSlot) -> Option<String> {
        let (_, shape) = ed.document().shapes().nth(index)?;
        shape.slot_text(slot).map(|t| t.text.clone())
    }

    /// The host (shape index, slot) of the open edit, if it is hosted.
    fn editing_host(ed: &Editor) -> Option<(ShapeId, TextSlot)> {
        ed.text_edit()?.host().map(|h| (h.id, h.slot))
    }

    #[test]
    fn with_experimental_sets_the_switch_in_helpers() {
        let ed = Editor::new();
        assert_eq!(ed.experimental(), Experimental::default());

        let ed = ed.with_experimental(SHAPE_TEXT_ON);

        assert_eq!(ed.experimental(), SHAPE_TEXT_ON);
        assert!(ed.helpers().experimental.shape_text);
    }

    #[test]
    fn shape_text_off_text_tool_in_rect_starts_free_block() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], false, Tool::Text);

        // Act
        place(&mut ed, 50.0, 50.0, "ab");
        key(&mut ed, Key::Escape);

        // Assert: T29 behaviour, the rectangle is untouched.
        assert_eq!(texts(&ed), ["ab"]);
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape), None);
    }

    #[test]
    fn shape_text_off_select_click_inside_selected_rect_clears_selection() {
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], false, Tool::Select);
        click(&mut ed, 0.0, 50.0);
        assert_eq!(ed.selection().len(), 1);

        click(&mut ed, 100.0, 50.0);

        assert!(ed.selection().is_empty());
        assert!(ed.text_edit().is_none());
    }

    #[test]
    fn shape_text_text_tool_click_in_rect_edits_its_text() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], true, Tool::Text);
        let id = ids(&ed)[0];

        // Act
        place(&mut ed, 100.0, 50.0, "ab");

        // Assert: editing the rectangle, not a free block.
        assert_eq!(editing_host(&ed), Some((id, TextSlot::Shape)));
        assert_eq!(editing(&ed).as_deref(), Some("ab"));
        key(&mut ed, Key::Escape);
        assert!(texts(&ed).is_empty());
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("ab"));
        let Some((_, shape)) = ed.document().shapes().next() else {
            panic!("the rectangle stays");
        };
        let size = shape.slot_text(TextSlot::Shape).map(|t| t.size);
        assert!(size.is_some_and(|s| approx_eq(s, 18.0, 1e-4)), "{size:?}");
    }

    #[test]
    fn shape_text_text_tool_click_in_ellipse_edits_its_text() {
        let mut ed = shape_text_editor(vec![ellipse(0.0, 0.0, 200.0, 100.0)], true, Tool::Text);

        place(&mut ed, 100.0, 50.0, "3");
        key(&mut ed, Key::Escape);

        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("3"));
        assert!(texts(&ed).is_empty());
    }

    #[test]
    fn shape_text_text_tool_click_in_grid_edits_that_cell() {
        // Arrange: 4 × 4 cells of 50.
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 200.0, 200.0, 4, 4)], true, Tool::Text);

        // Act
        place(&mut ed, 60.0, 110.0, "7");
        key(&mut ed, Key::Escape);

        // Assert
        assert_eq!(slot_text(&ed, 0, cell(1, 2)).as_deref(), Some("7"));
        assert_eq!(slot_text(&ed, 0, cell(0, 0)), None);
        assert!(texts(&ed).is_empty());
    }

    #[test]
    fn shape_text_text_block_on_top_of_rect_is_reopened() {
        // Arrange: a text block inside a rectangle.
        let block = Shape::Text {
            origin: Vec2::new(50.0, 40.0),
            text: "abc".to_owned(),
            size: 18.0,
            style: Style {
                color: ColorId::INK,
                width: 0.0,
            },
        };
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0), block], true, Tool::Text);
        let block_id = ids(&ed)[1];

        // Act
        click(&mut ed, 55.0, 45.0);

        // Assert
        assert_eq!(ed.text_edit().and_then(TextEdit::target), Some(block_id));
        assert_eq!(editing_host(&ed), None);
    }

    #[test]
    fn shape_text_select_second_click_edits_selected_shape() {
        // Arrange
        let shapes = vec![
            rect(0.0, 0.0, 200.0, 100.0),
            grid(300.0, 0.0, 200.0, 200.0, 4, 4),
        ];
        let mut ed = shape_text_editor(shapes, true, Tool::Select);
        let ids = ids(&ed);

        // Act / Assert: a first click on the outline only selects.
        click(&mut ed, 0.0, 50.0);
        assert_eq!(ed.selection(), [ids[0]]);
        assert!(ed.text_edit().is_none());

        // The second click, inside, opens its text.
        click(&mut ed, 100.0, 50.0);
        assert_eq!(editing_host(&ed), Some((ids[0], TextSlot::Shape)));
        typed(&mut ed, "x");
        key(&mut ed, Key::Escape);
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("x"));

        // A grid: select by a line, then click a cell.
        click(&mut ed, 300.0, 120.0);
        assert_eq!(ed.selection(), [ids[1]]);
        click(&mut ed, 420.0, 170.0);
        assert_eq!(editing_host(&ed), Some((ids[1], cell(2, 3))));
    }

    #[test]
    fn shape_text_edit_is_one_history_entry() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], true, Tool::Text);
        let before = ed.history.undo_len();

        // Act
        place(&mut ed, 100.0, 50.0, "hello");
        key(&mut ed, Key::Escape);

        // Assert
        assert_eq!(ed.history.undo_len(), before + 1);
        assert!(ed.apply(Command::Undo));
        assert_eq!(ed.document().len(), 1);
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape), None);
        ed.apply(Command::Redo);
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("hello"));
    }

    #[test]
    fn shape_text_ctrl_z_cancels_edit() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], true, Tool::Text);
        place(&mut ed, 100.0, 50.0, "x");
        key(&mut ed, Key::Escape);
        place(&mut ed, 100.0, 50.0, "yz");

        // Act
        assert!(key_with(&mut ed, Key::Z, CTRL));

        // Assert
        assert!(ed.text_edit().is_none());
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("x"));
    }

    #[test]
    fn shape_text_blank_edit_clears_text() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], true, Tool::Text);
        place(&mut ed, 100.0, 50.0, "x");
        key(&mut ed, Key::Escape);

        // Act: re-open (caret at the end), delete, commit.
        click(&mut ed, 100.0, 50.0);
        key(&mut ed, Key::Backspace);
        typed(&mut ed, "  ");
        key(&mut ed, Key::Escape);

        // Assert: the rectangle stays, without text; undo brings it back.
        assert_eq!(ed.document().len(), 1);
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape), None);
        ed.apply(Command::Undo);
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("x"));
    }

    #[test]
    fn shape_text_overlay_hides_host_and_shows_edit_as_block() {
        // Arrange: a grid with text in cell (0, 0).
        let g = grid(0.0, 0.0, 200.0, 200.0, 4, 4)
            .with_slot_text(cell(0, 0), ShapeText::new("a".to_owned(), 18.0));
        let mut ed = shape_text_editor(vec![g.clone()], true, Tool::Text);
        let id = ids(&ed)[0];

        // Act: edit cell (0, 0).
        place(&mut ed, 20.0, 20.0, "b");

        // Assert: the grid is hidden and drawn without the edited text;
        // the edit is a text block.
        let overlay = ed.overlay();
        assert_eq!(overlay.hidden, [id]);
        let Some(edit) = ed.text_edit() else {
            panic!("editing");
        };
        assert_eq!(edit.text(), "ab");
        assert!(overlay.shapes.contains(&g.with_slot_text(cell(0, 0), None)));
        assert!(overlay.shapes.contains(&edit.shape()));
        assert!(matches!(edit.shape(), Shape::Text { .. }));
    }

    #[test]
    fn shape_text_tab_fills_5x5_grid_without_mouse() {
        // Arrange: 5 × 5 cells of 50; one click into cell (0, 0).
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 250.0, 250.0, 5, 5)], true, Tool::Text);
        let before = ed.history.undo_len();
        click(&mut ed, 25.0, 25.0);

        // Act: type every cell, Tab to the next.
        for i in 0..25 {
            typed(&mut ed, &i.to_string());
            assert!(key(&mut ed, Key::Tab));
        }
        key(&mut ed, Key::Escape);

        // Assert: reading order, one history entry per cell.
        for row in 0..5 {
            for col in 0..5 {
                let want = (row * 5 + col).to_string();
                assert_eq!(
                    slot_text(&ed, 0, cell(col, row)),
                    Some(want),
                    "({col}, {row})"
                );
            }
        }
        assert_eq!(ed.history.undo_len(), before + 25);
    }

    #[test]
    fn shape_text_shift_tab_goes_back_and_tab_wraps() {
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 150.0, 100.0, 3, 2)], true, Tool::Text);
        let id = ids(&ed)[0];
        click(&mut ed, 25.0, 25.0);

        assert!(key_with(&mut ed, Key::Tab, SHIFT));
        assert_eq!(editing_host(&ed), Some((id, cell(2, 1))));
        assert!(key(&mut ed, Key::Tab));
        assert_eq!(editing_host(&ed), Some((id, cell(0, 0))));
        assert!(key(&mut ed, Key::Tab));
        assert_eq!(editing_host(&ed), Some((id, cell(1, 0))));
        assert_eq!(
            editing(&ed).as_deref(),
            Some(""),
            "Tab types no spaces in a cell"
        );
    }

    #[test]
    fn shape_text_enter_moves_down_and_stays_at_bottom_row() {
        // Arrange
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 100.0, 100.0, 2, 2)], true, Tool::Text);
        let id = ids(&ed)[0];
        place(&mut ed, 25.0, 25.0, "a");

        // Act / Assert
        key(&mut ed, Key::Enter);
        assert_eq!(editing_host(&ed), Some((id, cell(0, 1))));
        typed(&mut ed, "b");
        key(&mut ed, Key::Enter);
        assert_eq!(editing_host(&ed), Some((id, cell(0, 1))), "bottom row");
        assert_eq!(editing(&ed).as_deref(), Some("b"), "no new line in a cell");
        key_with(&mut ed, Key::Enter, SHIFT);
        assert_eq!(editing_host(&ed), Some((id, cell(0, 0))));
        assert_eq!(editing(&ed).as_deref(), Some("a"));
        assert_eq!(
            ed.text_edit().map(TextEdit::caret),
            Some(1),
            "caret at the end"
        );
        key(&mut ed, Key::Escape);
        assert_eq!(slot_text(&ed, 0, cell(0, 1)).as_deref(), Some("b"));
    }

    #[test]
    fn shape_text_enter_in_rect_is_a_new_line() {
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], true, Tool::Text);

        place(&mut ed, 100.0, 50.0, "a");
        key(&mut ed, Key::Enter);
        typed(&mut ed, "b");

        assert_eq!(editing(&ed).as_deref(), Some("a\nb"));
    }

    #[test]
    fn shape_text_undo_restores_cells_one_edit_at_a_time() {
        // Arrange: fill three cells with Tab.
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 150.0, 50.0, 3, 1)], true, Tool::Text);
        click(&mut ed, 25.0, 25.0);
        for s in ["x", "y", "z"] {
            typed(&mut ed, s);
            key(&mut ed, Key::Tab);
        }
        key(&mut ed, Key::Escape);
        let all = |ed: &Editor| {
            (0..3)
                .map(|c| slot_text(ed, 0, cell(c, 0)))
                .collect::<Vec<_>>()
        };
        let some = |s: &str| Some(s.to_owned());
        assert_eq!(all(&ed), [some("x"), some("y"), some("z")]);

        // Act / Assert
        ed.apply(Command::Undo);
        assert_eq!(all(&ed), [some("x"), some("y"), None]);
        ed.apply(Command::Undo);
        assert_eq!(all(&ed), [some("x"), None, None]);
        ed.apply(Command::Undo);
        assert_eq!(all(&ed), [None, None, None]);
        assert_eq!(ed.document().len(), 1);
    }

    #[test]
    fn shape_text_moves_copies_and_duplicates_with_shape() {
        // Arrange
        let r = rect(0.0, 0.0, 200.0, 100.0)
            .with_slot_text(TextSlot::Shape, ShapeText::new("n".to_owned(), 18.0));
        let mut ed = shape_text_editor(vec![r], true, Tool::Select);

        // Act: move by dragging the outline, duplicate, copy and paste.
        down(&mut ed, PointerButton::Left, 0.0, 50.0);
        move_to(&mut ed, 10.0, 60.0);
        up(&mut ed, PointerButton::Left, 10.0, 60.0);
        ed.apply(Command::Duplicate);
        ed.apply(Command::Copy);
        ed.apply(Command::Paste);

        // Assert
        assert_eq!(ed.document().len(), 3);
        for i in 0..3 {
            assert_eq!(
                slot_text(&ed, i, TextSlot::Shape).as_deref(),
                Some("n"),
                "{i}"
            );
        }
        let Some((_, Shape::Rect { a, .. })) = ed.document().shapes().next() else {
            panic!("the rectangle stays first");
        };
        assert!(a.approx_eq(Vec2::new(10.0, 10.0), 1e-4));
    }

    #[test]
    fn shape_text_erased_with_its_shape() {
        let r = rect(0.0, 0.0, 200.0, 100.0)
            .with_slot_text(TextSlot::Shape, ShapeText::new("n".to_owned(), 18.0));
        let mut ed = shape_text_editor(vec![r], true, Tool::Pen);

        // A right drag across the outline removes the shape and its text.
        down(&mut ed, PointerButton::Right, 0.0, 40.0);
        move_to(&mut ed, 0.0, 60.0);
        up(&mut ed, PointerButton::Right, 0.0, 60.0);
        assert!(ed.document().is_empty());

        ed.apply(Command::Undo);
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("n"));
    }

    #[test]
    fn shape_text_hides_label_and_keeps_counter() {
        // Arrange: a numbered rectangle (label 1, counter 2).
        let mut ed = shape_text_editor(Vec::new(), true, Tool::Rect);
        ed.apply(Command::ToggleNumbering);
        down(&mut ed, PointerButton::Left, 0.0, 0.0);
        move_to(&mut ed, 100.0, 60.0);
        up(&mut ed, PointerButton::Left, 200.0, 100.0);
        assert_eq!(ed.helpers().next_number, 2);

        // Act: type text into it, then undo the text.
        ed.apply(Command::SetTool(Tool::Text));
        place(&mut ed, 100.0, 50.0, "A");
        key(&mut ed, Key::Escape);

        // Assert: label and counter untouched.
        let label = |ed: &Editor| ed.document().shapes().next().and_then(|(_, s)| s.label());
        assert_eq!(label(&ed), Some(1));
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("A"));
        assert_eq!(ed.helpers().next_number, 2);
        ed.apply(Command::Undo);
        assert_eq!(label(&ed), Some(1));
        assert_eq!(ed.helpers().next_number, 2);
        ed.apply(Command::Redo);
        assert_eq!(ed.helpers().next_number, 2);
    }

    // ---- T34 pen taps -----------------------------------------------------

    /// A tap that slips 12 px right and 5 px down between press and
    /// release, as a pen tip does: further than the tools' own click
    /// thresholds, inside the pen slop.
    fn slipping_tap(ed: &mut Editor, at: Vec2) {
        down(ed, PointerButton::Left, at.x, at.y);
        move_to(ed, at.x + 6.0, at.y + 2.0);
        move_to(ed, at.x + 12.0, at.y + 5.0);
        up(ed, PointerButton::Left, at.x + 12.0, at.y + 5.0);
    }

    fn pen_editor(shapes: Vec<Shape>) -> Editor {
        editor_with(shapes).with_tap_slop(PEN_TAP_SLOP_PX)
    }

    #[test]
    fn tap_slop_is_off_by_default() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 100.0, 100.0)]);
        ed.apply(Command::SetTool(Tool::Select));
        let at = origin_on_screen(&ed) + Vec2::new(0.0, 50.0);

        // Act: the slip drags the rectangle along.
        slipping_tap(&mut ed, at);

        // Assert
        assert!(ed.tap_slop().abs() < f32::EPSILON);
        assert!(ed.can_undo());
        assert!(
            ed.document()
                .shapes()
                .all(|(_, s)| *s != rect(0.0, 0.0, 100.0, 100.0))
        );
    }

    #[test]
    fn pen_tap_with_select_selects_instead_of_marquee() {
        // Arrange
        let mut ed = pen_editor(vec![rect(0.0, 0.0, 100.0, 100.0)]);
        ed.apply(Command::SetTool(Tool::Select));
        let at = origin_on_screen(&ed) + Vec2::new(0.0, 50.0);

        // Act
        slipping_tap(&mut ed, at);

        // Assert
        assert!(approx_eq(ed.tap_slop(), PEN_TAP_SLOP_PX, 1e-6));
        assert_eq!(ed.selection(), ids(&ed).as_slice());
        assert_eq!(ed.selection().len(), 1);
        assert!(
            ed.document()
                .shapes()
                .all(|(_, s)| *s == rect(0.0, 0.0, 100.0, 100.0))
        );
    }

    #[test]
    fn pen_tap_with_rect_makes_no_shape() {
        // Arrange
        let mut ed = pen_editor(Vec::new());
        ed.apply(Command::SetTool(Tool::Rect));

        // Act
        slipping_tap(&mut ed, Vec2::new(200.0, 200.0));

        // Assert
        assert!(ed.document().is_empty());
        assert!(ed.preview().is_none());
    }

    #[test]
    fn pen_tap_with_pen_commits_a_dot() {
        // Arrange
        let mut ed = pen_editor(Vec::new());
        ed.apply(Command::SetTool(Tool::Pen));
        let at = Vec2::new(200.0, 200.0);

        // Act
        slipping_tap(&mut ed, at);

        // Assert
        let shapes: Vec<_> = ed.document().shapes().map(|(_, s)| s.clone()).collect();
        let [Shape::Stroke { points, .. }] = shapes.as_slice() else {
            panic!("expected one stroke, got {shapes:?}");
        };
        let press = ed.camera().screen_to_world(at);
        assert!(
            points.iter().all(|p| p.approx_eq(press, 1e-3)),
            "{points:?}"
        );
    }

    #[test]
    fn pen_stroke_beyond_slop_runs_from_press_to_release() {
        // Arrange
        let mut ed = pen_editor(Vec::new());
        ed.apply(Command::SetTool(Tool::Pen));

        // Act
        down(&mut ed, PointerButton::Left, 100.0, 100.0);
        move_to(&mut ed, 105.0, 100.0);
        move_to(&mut ed, 140.0, 100.0);
        move_to(&mut ed, 200.0, 100.0);
        up(&mut ed, PointerButton::Left, 200.0, 100.0);

        // Assert
        let shapes: Vec<_> = ed.document().shapes().map(|(_, s)| s.clone()).collect();
        let [Shape::Stroke { points, .. }] = shapes.as_slice() else {
            panic!("expected one stroke, got {shapes:?}");
        };
        let first = ed.camera().screen_to_world(Vec2::new(100.0, 100.0));
        let last = ed.camera().screen_to_world(Vec2::new(200.0, 100.0));
        assert!(
            points.first().is_some_and(|p| p.approx_eq(first, 1e-3)),
            "{points:?}"
        );
        assert!(
            points.last().is_some_and(|p| p.approx_eq(last, 1e-3)),
            "{points:?}"
        );
    }

    // ---- T35 pressure -------------------------------------------------------

    fn press(ed: &mut Editor, pressure: Option<f32>) -> bool {
        ed.handle(InputEvent::Pressure { pressure })
    }

    fn only_shape(ed: &Editor) -> Shape {
        let shapes: Vec<_> = ed.document().shapes().map(|(_, s)| s.clone()).collect();
        let [shape] = shapes.as_slice() else {
            panic!("expected one shape, got {shapes:?}");
        };
        shape.clone()
    }

    #[test]
    fn pressure_event_sets_the_pen_pressure() {
        // Arrange
        let mut ed = Editor::new();
        assert_eq!(ed.pressure(), None);

        // Act
        press(&mut ed, Some(0.4));

        // Assert
        assert!(ed.pressure().is_some_and(|p| approx_eq(p, 0.4, 1e-6)));
        press(&mut ed, None);
        assert_eq!(ed.pressure(), None);
    }

    #[test]
    fn pressure_event_clamps_and_drops_non_finite_values() {
        let mut ed = Editor::new();

        press(&mut ed, Some(1.5));
        assert!(ed.pressure().is_some_and(|p| approx_eq(p, 1.0, 1e-6)));
        press(&mut ed, Some(-0.5));
        assert!(ed.pressure().is_some_and(|p| approx_eq(p, 0.0, 1e-6)));
        press(&mut ed, Some(f32::NAN));
        assert_eq!(ed.pressure(), None);
        press(&mut ed, Some(0.3));
        press(&mut ed, Some(f32::INFINITY));
        assert_eq!(ed.pressure(), None);
    }

    #[test]
    fn pressure_event_needs_no_redraw() {
        let mut ed = Editor::new();

        assert!(!press(&mut ed, Some(0.9)));
        assert!(!press(&mut ed, None));
    }

    #[test]
    fn pressure_reaches_the_pen_tool() {
        // Arrange
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Pen));

        // Act: light at the start, hard at the end.
        press(&mut ed, Some(0.1));
        down(&mut ed, PointerButton::Left, 100.0, 100.0);
        move_to(&mut ed, 150.0, 100.0);
        press(&mut ed, Some(1.0));
        move_to(&mut ed, 150.0, 150.0);
        up(&mut ed, PointerButton::Left, 200.0, 150.0);

        // Assert
        let Shape::Stroke { points, widths, .. } = only_shape(&ed) else {
            panic!("expected a stroke");
        };
        let Some(widths) = widths else {
            panic!("expected widths");
        };
        assert_eq!(widths.len(), points.len());
        assert!(widths.first().is_some_and(|w| *w < 1.0), "{widths:?}");
        assert!(widths.last().is_some_and(|w| *w > 1.0), "{widths:?}");
    }

    #[test]
    fn pressure_leaves_rect_width_alone() {
        // Arrange
        let mut plain = Editor::new();
        let mut pressed = Editor::new();
        for ed in [&mut plain, &mut pressed] {
            ed.apply(Command::SetTool(Tool::Rect));
        }

        // Act
        press(&mut pressed, Some(1.0));
        for ed in [&mut plain, &mut pressed] {
            down(ed, PointerButton::Left, 100.0, 100.0);
            move_to(ed, 200.0, 180.0);
            up(ed, PointerButton::Left, 200.0, 180.0);
        }

        // Assert
        assert_eq!(only_shape(&plain), only_shape(&pressed));
    }

    // ---- T37 remembered text size -------------------------------------------

    /// The world size of every text block in the document, bottom first.
    fn block_sizes(ed: &Editor) -> Vec<f32> {
        ed.document()
            .shapes()
            .filter_map(|(_, shape)| match shape {
                Shape::Text { size, .. } => Some(*size),
                _ => None,
            })
            .collect()
    }

    /// Types `s` in a new block at screen `(x, y)`, grows it `steps` rungs
    /// with `Ctrl+]` and finishes it.
    fn place_grown(ed: &mut Editor, x: f32, y: f32, s: &str, steps: usize) {
        place(ed, x, y, s);
        for _ in 0..steps {
            assert!(key_with(ed, Key::BracketRight, CTRL));
        }
        key(ed, Key::Escape);
    }

    /// The world size of the open edit.
    fn open_size(ed: &Editor) -> f32 {
        let Some(edit) = ed.text_edit() else {
            panic!("a text edit is open");
        };
        edit.size
    }

    #[test]
    fn text_size_memory_next_block_starts_at_last_resized_size() {
        // Arrange: 18 → 20 → 24 px.
        let mut ed = text_editor();
        place_grown(&mut ed, 100.0, 50.0, "a", 2);

        // Act
        place(&mut ed, 100.0, 300.0, "b");
        key(&mut ed, Key::Escape);

        // Assert
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 2);
        assert!(approx_eq(sizes[1], 24.0, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_unresized_block_keeps_it() {
        // Arrange: a grown block, then one typed without resizing.
        let mut ed = text_editor();
        place_grown(&mut ed, 100.0, 50.0, "a", 2);
        place(&mut ed, 100.0, 200.0, "b");
        key(&mut ed, Key::Escape);

        // Act
        place(&mut ed, 100.0, 350.0, "c");
        key(&mut ed, Key::Escape);

        // Assert
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 3);
        assert!(approx_eq(sizes[2], 24.0, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_cancelled_edit_is_not_remembered() {
        // Arrange: grow a block, then throw the edit away.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");
        key_with(&mut ed, Key::BracketRight, CTRL);
        key_with(&mut ed, Key::Z, CTRL);
        assert!(ed.text_edit().is_none());

        // Act
        place(&mut ed, 100.0, 300.0, "b");
        key(&mut ed, Key::Escape);

        // Assert: the width rule, 18 px.
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 1);
        assert!(approx_eq(sizes[0], 18.0, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_survives_undo() {
        // Arrange: grow a block, then undo it.
        let mut ed = text_editor();
        place_grown(&mut ed, 100.0, 50.0, "a", 2);
        key_with(&mut ed, Key::Z, CTRL);
        assert!(block_sizes(&ed).is_empty());

        // Act
        place(&mut ed, 100.0, 300.0, "b");
        key(&mut ed, Key::Escape);

        // Assert
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 1);
        assert!(approx_eq(sizes[0], 24.0, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_keeps_on_screen_size_across_zoom() {
        // Arrange: 20 px at zoom 1, then one notch of zoom.
        let mut ed = text_editor();
        place_grown(&mut ed, 100.0, 50.0, "a", 1);
        ed.handle(InputEvent::Scroll {
            pos: Vec2::new(10.0, 10.0),
            delta: 1.0,
        });

        // Act
        place(&mut ed, 100.0, 300.0, "b");
        key(&mut ed, Key::Escape);

        // Assert: still 20 px on screen.
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 2);
        assert!(approx_eq(sizes[1], 20.0 / ZOOM_STEP, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_width_up_resets_to_width_rule() {
        // Arrange
        let mut ed = text_editor();
        place_grown(&mut ed, 100.0, 50.0, "a", 3);

        // Act: width 3 → 4 px.
        ed.apply(Command::WidthUp);
        place(&mut ed, 100.0, 300.0, "b");
        key(&mut ed, Key::Escape);

        // Assert: 12 + 2 × 4 = 20 px.
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 2);
        assert!(approx_eq(sizes[1], 20.0, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_width_key_at_ladder_end_still_resets() {
        // Arrange: the thinnest width (14 px text), a block grown to 18 px.
        let mut ed = text_editor();
        ed.apply(Command::WidthDown);
        ed.apply(Command::WidthDown);
        assert!(approx_eq(ed.style().width_px, MIN_WIDTH_PX, 1e-6));
        place_grown(&mut ed, 100.0, 50.0, "a", 2);

        // Act: the width cannot go lower.
        ed.apply(Command::WidthDown);
        place(&mut ed, 100.0, 300.0, "b");
        key(&mut ed, Key::Escape);

        // Assert
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 2);
        assert!(approx_eq(sizes[0], 18.0, 1e-4), "{sizes:?}");
        assert!(approx_eq(sizes[1], 14.0, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_reopened_block_keeps_its_size() {
        // Arrange: an 18 px block, then a block grown to 24 px.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "abcd");
        key(&mut ed, Key::Escape);
        place_grown(&mut ed, 100.0, 300.0, "b", 2);

        // Act: re-open the first block.
        click(&mut ed, column_x(100.0, 1.0), 55.0);

        // Assert
        assert_eq!(editing(&ed).as_deref(), Some("abcd"));
        assert!(approx_eq(open_size(&ed), 18.0, 1e-4));
    }

    #[test]
    fn text_size_memory_empty_shape_slot_starts_at_it() {
        // Arrange: a block grown to 24 px away from the rectangle.
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], true, Tool::Text);
        place_grown(&mut ed, 400.0, 300.0, "a", 2);

        // Act
        click(&mut ed, 100.0, 50.0);

        // Assert
        assert!(editing_host(&ed).is_some());
        assert!(approx_eq(open_size(&ed), 24.0, 1e-4));
    }

    #[test]
    fn text_size_memory_select_click_in_empty_slot_starts_at_it() {
        // Arrange: a block grown to 24 px, then the rectangle selected.
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], true, Tool::Text);
        place_grown(&mut ed, 400.0, 300.0, "a", 2);
        ed.apply(Command::SetTool(Tool::Select));
        click(&mut ed, 0.0, 50.0);
        assert_eq!(ed.selection().len(), 1);

        // Act
        click(&mut ed, 100.0, 50.0);

        // Assert
        assert!(editing_host(&ed).is_some());
        assert!(approx_eq(open_size(&ed), 24.0, 1e-4));
    }

    #[test]
    fn text_size_memory_existing_shape_text_keeps_its_size() {
        // Arrange: a rectangle holding 18 px text, a block grown to 24 px.
        let held = ShapeText::new("ab".to_owned(), 18.0);
        let boxed = rect(0.0, 0.0, 200.0, 100.0).with_slot_text(TextSlot::Shape, held);
        let mut ed = shape_text_editor(vec![boxed], true, Tool::Text);
        place_grown(&mut ed, 400.0, 300.0, "a", 2);

        // Act
        click(&mut ed, 100.0, 50.0);

        // Assert
        assert_eq!(editing(&ed).as_deref(), Some("ab"));
        assert!(approx_eq(open_size(&ed), 18.0, 1e-4));
    }
}
