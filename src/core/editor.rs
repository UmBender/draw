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
use crate::core::camera::{ZOOM_MAX, ZOOM_MIN};
use crate::core::clipboard::{self, Clipboard};
use crate::core::command::{Command, Tool};
use crate::core::command_palette::CommandPalette;
use crate::core::document::{
    Document, Edit, IdSet, ShapeId, tx_clear, tx_insert, tx_lock, tx_remove, tx_replace,
    tx_restack, tx_unlock_all,
};
use crate::core::file::DocFile;
use crate::core::geom::{Aabb, Vec2};
use crate::core::history::History;
use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
use crate::core::keymap;
use crate::core::minimap::{self, Minimap};
use crate::core::numbering::{self, FIRST_NUMBER};
use crate::core::palette::ColorId;
use crate::core::recognize;
use crate::core::shape::{CellStep, GRID_MAX_CELLS, Shape, Style, TextSlot, stepped_cells};
use crate::core::sheet::Sheet;
use crate::core::smoothing::SmoothingLevel;
use crate::core::snapshot::{self, Reader, Snapshot, SnapshotError, Writer};
use crate::core::tap::TapFilter;
use crate::core::tools::grid::{TableError, TextTable};
use crate::core::tools::layout::{self, EdgeList, GraphError, PlotError, PointList};
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
// Each bool is an independent switch with its own key, not a hidden state.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Helpers {
    /// Smart snapping (round, sizes, alignment) is on.
    pub smart_snap: bool,
    /// Snapping to the world grid is on.
    pub grid_snap: bool,
    /// New rectangles and ellipses are numbered.
    pub numbering: bool,
    /// The first number is 0 instead of [`FIRST_NUMBER`] (ADR-T86-1).
    pub zero_based: bool,
    /// The number the next numbered shape gets, at least
    /// [`Self::first_number`].
    pub next_number: u32,
    /// Columns of new grids, in `1..=GRID_MAX_CELLS`.
    pub grid_cols: u32,
    /// Rows of new grids, in `1..=GRID_MAX_CELLS`.
    pub grid_rows: u32,
}

impl Helpers {
    /// The first number given out: 0 when zero-based, else [`FIRST_NUMBER`].
    #[must_use]
    pub const fn first_number(&self) -> u32 {
        if self.zero_based { 0 } else { FIRST_NUMBER }
    }
}

impl Default for Helpers {
    fn default() -> Self {
        Self {
            smart_snap: false,
            grid_snap: false,
            numbering: false,
            zero_based: false,
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
    /// Panning the view, by a drag on the canvas or in the minimap
    /// (ADR-T110-1).
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
    /// Moving the view by a press or drag in the minimap (ADR-T110-1).
    Minimap {
        /// Button whose release ends the gesture.
        button: PointerButton,
        /// The minimap's world box at the press, kept until the release.
        world: Aabb,
    },
    /// A tool gesture.
    Tool {
        /// Button whose release ends the gesture.
        button: PointerButton,
        /// Tool receiving the events.
        tool: Tool,
        /// No pointer motion since the press.
        fresh: bool,
    },
}

impl Gesture {
    /// Whether a press of `button` replaces this gesture: a pen reports
    /// its tip just before a held barrel button, in the same spot, so a
    /// right or middle press takes over a left tool press that has not
    /// moved yet.
    fn yields_to(self, button: PointerButton) -> bool {
        matches!(
            self,
            Self::Tool {
                button: PointerButton::Left,
                fresh: true,
                ..
            }
        ) && button != PointerButton::Left
    }
}

/// How many pages there are (ADR-T116-1).
pub const PAGE_COUNT: usize = 6;

/// What one page owns besides the shapes: the state of a page that is not
/// the current one (ADR-T116-1).
#[derive(Debug, Clone, PartialEq)]
struct Page {
    doc: Document,
    history: History,
    camera: Camera,
    selection: Vec<ShapeId>,
    /// The auto-number counter of this page.
    next_number: u32,
}

impl Page {
    /// An empty page whose numbering starts at `first_number`.
    fn fresh(first_number: u32) -> Self {
        Self {
            doc: Document::new(),
            history: History::new(),
            camera: Camera::default(),
            selection: Vec::new(),
            next_number: first_number,
        }
    }
}
/// A request the shell collects once (ADR-T114-2): kept apart from the
/// editor's own flags.
#[derive(Debug, Clone, Copy, Default)]
struct SvgAsk(bool);

/// All editor state: document, history, view, tools and settings.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Editor {
    /// The current page's document, history, camera and selection live in
    /// the fields below; the other pages wait in `pages` (ADR-T116-1).
    page: usize,
    /// One slot per page; the current page's slot and the slots of pages
    /// never touched are `None`.
    pages: PageSlots,
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
    /// Why the latest key refused to act, for the shell's log (ADR-T84-1).
    /// Transient: not part of the saved state.
    notice: Option<String>,
    /// `Ctrl+E` was pressed and the shell has not collected it yet
    /// (ADR-T92-2). Not saved state: the shell collects it every frame.
    export_asked: bool,
    /// `Ctrl+Shift+E` was pressed and the shell has not collected it yet
    /// (ADR-T114-2). Not saved state.
    svg_export_asked: SvgAsk,
    /// `Ctrl+Shift+C` was pressed with a selection: its world box, until
    /// the shell collects it (ADR-T115-1). Not saved state.
    copy_image_asked: Option<Aabb>,
    /// The content bounds the minimap shows and the document revision they
    /// were computed for; refreshed only while it shows (ADR-T110-1). Not
    /// saved state.
    minimap_content: Option<(u64, Option<Aabb>)>,
    /// The open command palette, which captures the keys while it is
    /// `Some` (ADR-T111-1). Not saved state.
    palette: Option<CommandPalette>,
    /// `Ctrl+S` was pressed and the shell has not collected it yet
    /// (ADR-T108-1). Not saved state.
    save_asked: bool,
    /// The open shortcut sheet, which captures the keys while it is `Some`
    /// (ADR-T113-1). Never open together with the palette. Not saved state.
    sheet: Option<Sheet>,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Editor {
    /// Compares the saved state (ADR-T74-1): the keymap, the document
    /// revisions and the history's change count and byte estimate are not
    /// part of it.
    fn eq(&self, other: &Self) -> bool {
        self.page == other.page
            && self.pages == other.pages
            && self.doc == other.doc
            && self.history == other.history
            && self.camera == other.camera
            && self.selection == other.selection
            && self.clipboard == other.clipboard
            && self.tools == other.tools
            && self.tool == other.tool
            && self.style == other.style
            && self.smoothing == other.smoothing
            && self.toolbar_visible == other.toolbar_visible
            && self.viewport == other.viewport
            && self.cursor == other.cursor
            && self.space_held == other.space_held
            && self.gesture == other.gesture
            && self.reedit == other.reedit
            && self.tap == other.tap
            && self.pressure == other.pressure
    }
}

impl Editor {
    /// This state as a [`Snapshot`] (ADR-T74-1).
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        Snapshot(self.clone())
    }

    /// The editor a snapshot holds, with the default keymap and fresh
    /// document revisions, so the shell renders everything.
    #[must_use]
    pub fn from_snapshot(snapshot: Snapshot) -> Self {
        Self {
            keymap: keymap::resolve,
            doc_revision: 0,
            doc_base_revision: 0,
            minimap_content: None,
            palette: None,
            sheet: None,
            ..snapshot.0
        }
    }

    /// Writes every section after the header (ADR-T74-1).
    pub(crate) fn write_snapshot(&self, w: &mut Writer) {
        let helpers = self.style.helpers;
        w.section("editor");
        w.record("tool", |l| {
            l.put(&self.tool);
        });
        w.record("style", |l| {
            l.put(&self.style.color).f32(self.style.width_px);
        });
        w.record("helpers", |l| {
            l.bool(helpers.smart_snap)
                .bool(helpers.grid_snap)
                .bool(helpers.numbering)
                .word(helpers.next_number)
                .word(helpers.grid_cols)
                .word(helpers.grid_rows)
                .bool(helpers.zero_based);
        });
        w.record("smoothing", |l| {
            l.put(&self.smoothing);
        });
        w.record("toolbar", |l| {
            l.bool(self.toolbar_visible);
        });
        w.record("viewport", |l| {
            l.put(&self.viewport);
        });
        w.record("cursor", |l| {
            l.put(&self.cursor);
        });
        w.record("space", |l| {
            l.bool(self.space_held);
        });
        w.record("pressure", |l| {
            l.opt(self.pressure.as_ref());
        });
        w.record("tap", |l| {
            l.put(&self.tap);
        });
        w.record("gesture", |l| match self.gesture {
            None => {
                l.none();
            }
            Some(Gesture::Pan { button, pan }) => {
                l.word("pan").put(&button).put(&pan.last());
            }
            Some(Gesture::Minimap { button, world }) => {
                l.word("minimap").put(&button).put(&world);
            }
            Some(Gesture::Tool {
                button,
                tool,
                fresh,
            }) => {
                l.word("tool").put(&button).put(&tool).bool(fresh);
            }
        });
        w.record("reedit", |l| match &self.reedit {
            None => {
                l.none();
            }
            Some((pos, id)) => {
                l.put(pos).put(id);
            }
        });
        w.record("camera", |l| {
            l.put(&self.camera.offset()).f32(self.camera.zoom());
        });
        w.record("selection", |l| {
            for id in &self.selection {
                l.put(id);
            }
        });
        snapshot::write_document(w, &self.doc);
        let (undo, redo, limit, budget) = self.history.parts();
        w.section("history");
        w.record("limits", |l| {
            l.word(limit).word(budget);
        });
        snapshot::write_transactions(w, "undo", undo);
        snapshot::write_transactions(w, "redo", redo);
        snapshot::write_clipboard(w, self.clipboard.shapes());
        self.tools.write_snapshot(w);
        self.write_pages(w);
    }

    /// Reads every section after the header and checks the editor's
    /// invariants (ADR-T74-1).
    #[allow(clippy::too_many_lines)]
    pub(crate) fn read_snapshot(r: &mut Reader<'_>, version: u32) -> Result<Self, SnapshotError> {
        r.section("editor")?;
        let mut t = r.record("tool")?;
        let tool = t.get()?;
        t.end()?;

        let style = read_style(r)?;
        // Text in shapes is always on (ADR-T52-1).
        r.set_shape_text(true);

        let mut t = r.record("smoothing")?;
        let smoothing = t.get()?;
        t.end()?;
        let toolbar_visible = read_flag(r, "toolbar")?;
        let mut t = r.record("viewport")?;
        let viewport: Vec2 = t.get()?;
        t.end()?;
        if viewport.x < 0.0 || viewport.y < 0.0 {
            return Err(t.error("negative viewport"));
        }
        let mut t = r.record("cursor")?;
        let cursor = t.get()?;
        t.end()?;
        let space_held = read_flag(r, "space")?;
        let mut t = r.record("pressure")?;
        let pressure: Option<f32> = t.opt()?;
        t.end()?;
        if pressure.is_some_and(|p| !(0.0..=1.0).contains(&p)) {
            return Err(t.error("pressure outside [0, 1]"));
        }
        let mut t = r.record("tap")?;
        let tap = t.get()?;
        t.end()?;

        let gesture = read_gesture(r)?;

        let mut reedit_tokens = r.record("reedit")?;
        let reedit = if reedit_tokens.none() {
            None
        } else {
            Some((reedit_tokens.get()?, reedit_tokens.get()?))
        };
        reedit_tokens.end()?;

        let camera = read_camera(r)?;

        let mut selection_tokens = r.record("selection")?;
        let mut selection = Vec::new();
        while !selection_tokens.at_end() {
            selection.push(selection_tokens.get()?);
        }

        let doc = read_numbered_document(r)?;
        snapshot::check_unlocked_ids(&selection_tokens, &doc, &selection)?;
        if let Some((_, id)) = reedit {
            snapshot::check_ids(&reedit_tokens, &doc, &[id])?;
        }

        let history = read_history(r)?;
        let clipboard = Clipboard::from_shapes(snapshot::read_clipboard(r)?);
        let tools = ToolStates::read_snapshot(r, &doc)?;
        // Version 2 has one page.
        let (page, pages) = if version >= 3 {
            Self::read_pages(r)?
        } else {
            (0, empty_pages())
        };

        Ok(Self {
            page,
            pages,
            doc,
            history,
            camera,
            selection,
            clipboard,
            tools,
            tool,
            style,
            smoothing,
            toolbar_visible,
            viewport,
            cursor,
            space_held,
            gesture,
            reedit,
            keymap: keymap::resolve,
            tap,
            pressure,
            doc_revision: 0,
            doc_base_revision: 0,
            notice: None,
            export_asked: false,
            svg_export_asked: SvgAsk::default(),
            copy_image_asked: None,
            minimap_content: None,
            palette: None,
            save_asked: false,
            sheet: None,
        })
    }

    /// An empty document at the default view, pen tool, default style.
    #[must_use]
    pub fn new() -> Self {
        Self {
            page: 0,
            pages: empty_pages(),
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
            notice: None,
            export_asked: false,
            svg_export_asked: SvgAsk::default(),
            copy_image_asked: None,
            minimap_content: None,
            palette: None,
            save_asked: false,
            sheet: None,
        }
    }

    /// This document as a [`DocFile`] (ADR-T108-1).
    #[must_use]
    pub fn file(&self) -> DocFile {
        // Only the document parts are copied: no history, clipboard or
        // tool state, so a save costs one document clone.
        let mut kept = Self::new();
        kept.page = self.page;
        kept.doc = self.doc.clone();
        kept.camera = Camera::new(self.camera.offset(), self.camera.zoom());
        kept.style.helpers.next_number = self.style.helpers.next_number;
        kept.pages = self
            .pages
            .iter()
            .map(|slot| {
                slot.as_ref().map(|p| {
                    Box::new(Page {
                        doc: p.doc.clone(),
                        history: History::new(),
                        camera: Camera::new(p.camera.offset(), p.camera.zoom()),
                        selection: Vec::new(),
                        next_number: p.next_number,
                    })
                })
            })
            .collect();
        DocFile(kept)
    }

    /// Replaces the document (every page and the current one) with the one
    /// in `file`; settings, tools and clipboard stay. The history starts
    /// empty and the shell redraws everything (ADR-T108-1).
    pub fn apply_file(&mut self, file: DocFile) {
        if self.tools.text.edit.is_some() {
            self.cancel_text();
        }
        self.cancel_gesture();
        let first = self.style.helpers.first_number();
        let loaded = file.0;
        self.page = loaded.page;
        self.pages = loaded.pages;
        for page in self.pages.iter_mut().flatten() {
            page.next_number = page.next_number.max(first);
        }
        self.doc = loaded.doc;
        self.camera = loaded.camera;
        self.history = History::new();
        self.selection.clear();
        self.reedit = None;
        self.style.helpers.next_number = loaded.style.helpers.next_number.max(first);
        self.doc_revision = self.doc_revision.wrapping_add(1);
        self.doc_base_revision = self.doc_revision;
    }

    /// Whether `Ctrl+S` was pressed since the last call (ADR-T108-1).
    pub fn take_save_request(&mut self) -> bool {
        std::mem::take(&mut self.save_asked)
    }

    /// Writes the pages for a document file (ADR-T108-1).
    ///
    /// A page that is still empty at the default view is left out; the
    /// numbers are at least 1 so the text does not depend on the
    /// zero-based setting of the writer.
    pub(crate) fn write_file(&self, w: &mut Writer) {
        let current = Page {
            doc: self.doc.clone(),
            history: History::new(),
            camera: Camera::new(self.camera.offset(), self.camera.zoom()),
            selection: Vec::new(),
            next_number: self.style.helpers.next_number,
        };
        let blank = Page::fresh(FIRST_NUMBER);
        let written: Vec<(usize, &Page)> = (0..PAGE_COUNT)
            .filter_map(|index| {
                let page = if index == self.page {
                    &current
                } else {
                    self.pages.get(index)?.as_deref()?
                };
                let empty = page.doc == blank.doc
                    && page.camera == blank.camera
                    && page.next_number <= FIRST_NUMBER;
                (!empty).then_some((index, page))
            })
            .collect();
        w.section("pages");
        w.record("current", |l| {
            l.word(self.page);
        });
        w.record("count", |l| {
            l.word(written.len());
        });
        for (index, page) in written {
            w.section("page");
            w.record("index", |l| {
                l.word(index);
            });
            w.record("camera", |l| {
                l.put(&page.camera.offset()).f32(page.camera.zoom());
            });
            w.record("number", |l| {
                l.word(page.next_number.max(FIRST_NUMBER));
            });
            snapshot::write_document(w, &page.doc);
        }
    }

    /// Reads the pages of a document file of `version` (ADR-T108-1).
    ///
    /// Version 1 is the only one so far; a later version keeps a branch
    /// here for every older one.
    pub(crate) fn read_file(r: &mut Reader<'_>, version: u32) -> Result<Self, SnapshotError> {
        debug_assert!(version >= 1);
        r.set_shape_text(true);
        r.section("pages")?;
        let mut t = r.record("current")?;
        let current: usize = t.number("current page")?;
        t.end()?;
        if current >= PAGE_COUNT {
            return Err(t.error(format!("page {current} out of range")));
        }
        let mut t = r.record("count")?;
        let count: usize = t.number("page count")?;
        t.end()?;
        if count > PAGE_COUNT {
            return Err(t.error(format!("{count} pages")));
        }
        let mut ed = Self::new();
        ed.page = current;
        let mut last: Option<usize> = None;
        for _ in 0..count {
            r.section("page")?;
            let mut t = r.record("index")?;
            let index: usize = t.number("page index")?;
            t.end()?;
            if index >= PAGE_COUNT || last.is_some_and(|l| index <= l) {
                return Err(t.error(format!("page {index} out of place")));
            }
            last = Some(index);
            let camera = read_camera(r)?;
            let mut t = r.record("number")?;
            let next_number: u32 = t.number("next number")?;
            t.end()?;
            let doc = read_numbered_document(r)?;
            if index == current {
                ed.doc = doc;
                ed.camera = camera;
                ed.style.helpers.next_number = next_number;
            } else {
                ed.pages[index] = Some(Box::new(Page {
                    doc,
                    history: History::new(),
                    camera,
                    selection: Vec::new(),
                    next_number,
                }));
            }
        }
        Ok(ed)
    }

    /// Whether page `page` (`0` is page 1) holds a shape; `false` for a
    /// page out of range (ADR-T124-1).
    #[must_use]
    pub fn page_has_shapes(&self, page: usize) -> bool {
        if page == self.page {
            return !self.doc.is_empty();
        }
        self.pages
            .get(page)
            .and_then(Option::as_ref)
            .is_some_and(|stored| !stored.doc.is_empty())
    }

    /// The current page, `0` for page 1 (ADR-T116-1).
    #[must_use]
    pub fn page(&self) -> usize {
        self.page
    }

    /// Shows page `page` (`0..PAGE_COUNT`): cancels the gesture, closes an
    /// open text edit and swaps the page states (ADR-T116-1). Returns
    /// whether the view needs a redraw.
    fn go_to_page(&mut self, page: usize) -> bool {
        if page >= PAGE_COUNT || page == self.page {
            return false;
        }
        self.cancel_gesture();
        let first = self.style.helpers.first_number();
        let incoming = match self.pages[page].take() {
            Some(stored) => *stored,
            None => Page::fresh(first),
        };
        let outgoing = Page {
            doc: std::mem::replace(&mut self.doc, incoming.doc),
            history: std::mem::replace(&mut self.history, incoming.history),
            camera: std::mem::replace(&mut self.camera, incoming.camera),
            selection: std::mem::replace(&mut self.selection, incoming.selection),
            // The first number may have changed (zero-based) while the
            // page was away.
            next_number: std::mem::replace(
                &mut self.style.helpers.next_number,
                incoming.next_number.max(first),
            ),
        };
        // A page left as it was found takes no slot.
        self.pages[self.page] = (outgoing != Page::fresh(first)).then(|| Box::new(outgoing));
        self.page = page;
        // A different document: the shell redraws its cached layer.
        self.doc_revision = self.doc_revision.wrapping_add(1);
        self.doc_base_revision = self.doc_revision;
        true
    }

    /// Writes the `[pages]` section and one `[page]` per stored page
    /// (ADR-T116-1).
    fn write_pages(&self, w: &mut Writer) {
        w.section("pages");
        w.record("current", |l| {
            l.word(self.page);
        });
        let stored = self.pages.iter().flatten().count();
        w.record("stored", |l| {
            l.word(stored);
        });
        for (index, page) in self.pages.iter().enumerate() {
            let Some(page) = page else { continue };
            w.section("page");
            w.record("index", |l| {
                l.word(index);
            });
            w.record("camera", |l| {
                l.put(&page.camera.offset()).f32(page.camera.zoom());
            });
            w.record("number", |l| {
                l.word(page.next_number);
            });
            w.record("selection", |l| {
                for id in &page.selection {
                    l.put(id);
                }
            });
            snapshot::write_document(w, &page.doc);
            let (undo, redo, limit, budget) = page.history.parts();
            w.section("history");
            w.record("limits", |l| {
                l.word(limit).word(budget);
            });
            snapshot::write_transactions(w, "undo", undo);
            snapshot::write_transactions(w, "redo", redo);
        }
    }

    /// Reads what [`Editor::write_pages`] wrote: the current page and the
    /// stored ones, each checked like the current page.
    fn read_pages(r: &mut Reader<'_>) -> Result<(usize, PageSlots), SnapshotError> {
        r.section("pages")?;
        let mut t = r.record("current")?;
        let current: usize = t.number("current page")?;
        t.end()?;
        if current >= PAGE_COUNT {
            return Err(t.error(format!("page {current} out of range")));
        }
        let mut t = r.record("stored")?;
        let stored: usize = t.number("stored pages")?;
        t.end()?;
        if stored >= PAGE_COUNT {
            return Err(t.error(format!("{stored} stored pages")));
        }
        let mut pages = empty_pages();
        let mut last: Option<usize> = None;
        for _ in 0..stored {
            r.section("page")?;
            let mut t = r.record("index")?;
            let index: usize = t.number("page index")?;
            t.end()?;
            if index >= PAGE_COUNT || index == current || last.is_some_and(|l| index <= l) {
                return Err(t.error(format!("page {index} out of place")));
            }
            last = Some(index);
            let camera = read_camera(r)?;
            let mut t = r.record("number")?;
            // May sit below the first number: the switch clamps it.
            let next_number: u32 = t.number("next number")?;
            t.end()?;
            let mut selection_tokens = r.record("selection")?;
            let mut selection = Vec::new();
            while !selection_tokens.at_end() {
                selection.push(selection_tokens.get()?);
            }
            let doc = read_numbered_document(r)?;
            snapshot::check_unlocked_ids(&selection_tokens, &doc, &selection)?;
            let history = read_history(r)?;
            pages[index] = Some(Box::new(Page {
                doc,
                history,
                camera,
                selection,
                next_number,
            }));
        }
        Ok((current, pages))
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

    /// Why the latest key refused to act, once: the shell writes it to the
    /// diagnostics log (ADR-T84-1).
    pub fn take_notice(&mut self) -> Option<String> {
        self.notice.take()
    }

    /// Replaces the keymap (used by tests and alternative bindings).
    #[must_use]
    pub fn with_keymap(self, keymap: Keymap) -> Self {
        Self { keymap, ..self }
    }

    /// Handles one input event. Returns whether the view needs a redraw.
    ///
    /// Events go through the tap filter first (ADR-T34-1).
    pub fn handle(&mut self, event: InputEvent) -> bool {
        let shown = self.minimap_shown();
        let mut redraw = false;
        for event in self.tap.filter(event).into_iter().flatten() {
            redraw |= self.handle_filtered(event);
        }
        self.refresh_minimap_content();
        // The minimap appearing or going is a change on screen (ADR-T110-1).
        redraw || self.minimap_shown() != shown
    }

    /// [`Editor::handle`] for one event that left the tap filter.
    fn handle_filtered(&mut self, event: InputEvent) -> bool {
        if self.sheet.is_some() {
            if let Some(redraw) = self.sheet_event(&event) {
                return redraw;
            }
        } else if self.palette.is_some() {
            if let Some(redraw) = self.palette_event(&event) {
                return redraw;
            }
        } else if matches!(
            event,
            InputEvent::Char { ch: '?', mods } if !mods.ctrl && !mods.alt
        ) && self.tools.text.edit.is_none()
        {
            return self.open_sheet();
        } else if matches!(
            event,
            InputEvent::KeyDown {
                key: Key::P,
                mods: Modifiers {
                    shift: true,
                    ctrl: true,
                    alt: false
                }
            }
        ) {
            return self.open_palette();
        }
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
            InputEvent::Paste(text) => self
                .tools
                .text
                .edit
                .as_mut()
                .is_some_and(|edit| edit.paste(&text)),
            InputEvent::PasteGrid(text) => self.grid_from_clipboard(&text),
            InputEvent::Resize { size } => match size.sanitize() {
                Some(size) if size.x >= 0.0 && size.y >= 0.0 => {
                    self.viewport = size;
                    true
                }
                _ => false,
            },
        }
    }

    /// Opens the command palette over the canvas (ADR-T111-1). A gesture
    /// in progress is cancelled and an open text block committed first, so
    /// nothing is left half-done under the overlay.
    fn open_palette(&mut self) -> bool {
        self.cancel_gesture();
        self.finish_text();
        self.palette = Some(CommandPalette::new());
        true
    }

    /// Opens the shortcut sheet over the canvas (ADR-T113-1). A gesture in
    /// progress is cancelled first, like for the palette.
    fn open_sheet(&mut self) -> bool {
        self.cancel_gesture();
        self.sheet = Some(Sheet::new());
        true
    }

    /// Handles `event` for the open shortcut sheet (ADR-T113-1). `None` for
    /// the events that are not about the keys and pointer (resize, pen
    /// pressure, key releases), which the editor handles as usual.
    #[allow(clippy::wildcard_enum_match_arm, reason = "key table, ADR-T131-1")]
    fn sheet_event(&mut self, event: &InputEvent) -> Option<bool> {
        let sheet = self.sheet.as_mut()?;
        let redraw = match *event {
            InputEvent::KeyDown { key, .. } => match key {
                Key::Escape => {
                    self.sheet = None;
                    true
                }
                Key::ArrowRight => {
                    sheet.next();
                    true
                }
                Key::ArrowLeft => {
                    sheet.prev();
                    true
                }
                Key::Digit1 => sheet.jump(1),
                Key::Digit2 => sheet.jump(2),
                Key::Digit3 => sheet.jump(3),
                Key::Digit4 => sheet.jump(4),
                Key::Digit5 => sheet.jump(5),
                Key::Digit6 => sheet.jump(6),
                Key::Digit7 => sheet.jump(7),
                Key::Digit8 => sheet.jump(8),
                Key::Digit9 => sheet.jump(9),
                _ => false,
            },
            InputEvent::Char { ch: '?', mods } if !mods.ctrl && !mods.alt => {
                self.sheet = None;
                true
            }
            InputEvent::Char { .. }
            | InputEvent::PointerDown { .. }
            | InputEvent::PointerMove { .. }
            | InputEvent::PointerUp { .. }
            | InputEvent::Scroll { .. }
            | InputEvent::Paste(_)
            | InputEvent::PasteGrid(_) => false,
            InputEvent::KeyUp { .. } | InputEvent::Pressure { .. } | InputEvent::Resize { .. } => {
                return None;
            }
        };
        Some(redraw)
    }

    /// Handles `event` for the open palette (ADR-T111-1). `None` for the
    /// events that are not about the keys and pointer (resize, pen
    /// pressure, key releases), which the editor handles as usual.
    #[allow(clippy::wildcard_enum_match_arm, reason = "key table, ADR-T131-1")]
    fn palette_event(&mut self, event: &InputEvent) -> Option<bool> {
        let palette = self.palette.as_mut()?;
        let redraw = match *event {
            InputEvent::KeyDown { key, .. } => match key {
                Key::Escape => {
                    self.palette = None;
                    true
                }
                Key::Enter => {
                    let row = palette.highlighted();
                    self.palette = None;
                    if let Some((_, command, _)) = row {
                        self.apply(command);
                    }
                    true
                }
                Key::Backspace => palette.backspace(),
                Key::ArrowUp => palette.up(),
                Key::ArrowDown => palette.down(),
                _ => false,
            },
            InputEvent::Char { ch, mods } => !mods.ctrl && !mods.alt && palette.insert(ch),
            InputEvent::PointerDown { .. }
            | InputEvent::PointerMove { .. }
            | InputEvent::PointerUp { .. }
            | InputEvent::Scroll { .. }
            | InputEvent::Paste(_)
            | InputEvent::PasteGrid(_) => false,
            InputEvent::KeyUp { .. } | InputEvent::Pressure { .. } | InputEvent::Resize { .. } => {
                return None;
            }
        };
        Some(redraw)
    }

    /// Executes `command`. Returns whether the view needs a redraw.
    ///
    /// An open text block is committed first, except that
    /// [`Command::Undo`] cancels it instead (ADR-T29-2) and
    /// [`Command::ExportView`] and [`Command::ExportSvg`] leave it alone
    /// (ADR-T92-2, ADR-T114-2). Nothing runs
    /// under the open command palette (ADR-T111-1).
    pub fn apply(&mut self, command: Command) -> bool {
        if self.palette.is_some() || self.sheet.is_some() {
            return false;
        }
        if self.tools.text.edit.is_some() {
            if command == Command::Undo {
                return self.cancel_text();
            }
            if matches!(
                command,
                Command::ExportView | Command::ExportSvg | Command::CopyImage
            ) {
                return false;
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
            Command::Undo => self.undo(),
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
            Command::GridFromText => self.grid_from_selection(),
            Command::GraphFromText => self.graph_from_selection(),
            Command::ScaleUp => self.run_edit(select::scale_selection_up),
            Command::ScaleDown => self.run_edit(select::scale_selection_down),
            Command::PlotFromText => self.plot_from_selection(),
            Command::RaiseSelection => self.restack(Restack::Up),
            Command::LowerSelection => self.restack(Restack::Down),
            Command::SelectionToTop => self.restack(Restack::Top),
            Command::SelectionToBottom => self.restack(Restack::Bottom),
            Command::RecognizeShape => self.recognize_last_stroke(),
            Command::Align(side) => self.arrange(|ctx| select::align_selection(ctx, side)),
            Command::Distribute(axis) => {
                self.arrange(|ctx| select::distribute_selection(ctx, axis))
            }
            Command::DeleteSelection => self.run_edit(select::delete_selection),
            Command::ClearAll => self.clear_all(),
            Command::LockSelection => self.lock_selection(),
            Command::UnlockAll => self.unlock_all(),
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
            Command::FitView => self.fit_view(),
            Command::ToggleToolbar => {
                self.toolbar_visible = !self.toolbar_visible;
                true
            }
            // The arrows act on the one selected grid (ADR-T85-1).
            Command::GridCols(delta) if self.resizable_grid().is_some() => {
                self.resize_selected_grid(delta, 0)
            }
            Command::GridRows(delta) if self.resizable_grid().is_some() => {
                self.resize_selected_grid(0, delta)
            }
            Command::GoToPage(page) => self.go_to_page(page),
            Command::ExportView => self.ask_export(),
            Command::SaveFile => {
                self.save_asked = true;
                false
            }
            Command::ExportSvg => self.ask_svg_export(),
            Command::CopyImage => self.ask_copy_image(),
            // Helper commands never cancel the gesture, so they act live
            // during a drag (ADR-T16-2).
            Command::ToggleSmartSnap
            | Command::ToggleGridSnap
            | Command::ToggleNumbering
            | Command::ResetNumbering
            | Command::ToggleZeroBased
            | Command::GridCols(_)
            | Command::GridRows(_) => apply_helper(&mut self.style.helpers, command),
        }
    }

    /// Undoes one step and gives its number back (ADR-T19-1). Returns
    /// whether anything changed.
    fn undo(&mut self) -> bool {
        let cancelled = self.cancel_gesture();
        let given = self.helpers().next_number.saturating_sub(1);
        let before = numbering::label_count(&self.doc, given);
        let changed = self.history.undo(&mut self.doc);
        let after = numbering::label_count(&self.doc, given);
        let helpers = &mut self.style.helpers;
        helpers.next_number =
            numbering::roll_back(helpers.next_number, before, after, helpers.first_number());
        self.prune_selection();
        changed || cancelled
    }

    /// Frames all content, or resets the view of an empty document. Returns
    /// whether the camera moved.
    fn fit_view(&mut self) -> bool {
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
    /// A text block being edited is left out: it shows its source, and
    /// [`TextEdit::bounds`] is its box until the edit ends (T81).
    #[must_use]
    pub fn selection_bounds(&self) -> Option<Aabb> {
        let editing = self.text_edit().and_then(TextEdit::target);
        self.selection
            .iter()
            .filter(|id| Some(**id) != editing)
            .filter_map(|id| self.doc.get(*id))
            .map(Shape::bounds)
            .reduce(|a, b| a.union(&b))
    }

    /// Screen centres of the resize handles, clockwise from the top-left
    /// corner: `Some` only in the Select tool, with no gesture and no open
    /// text block, for a selection that can scale (ADR-T91-1).
    #[must_use]
    pub fn selection_handles(&self) -> Option<[Vec2; 4]> {
        if self.tool != Tool::Select || self.gesture.is_some() || self.text_edit().is_some() {
            return None;
        }
        select::handle_centers(&self.doc, &self.selection, &self.camera)
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
            Gesture::Pan { .. } | Gesture::Minimap { .. } => ActiveGesture::Pan,
            Gesture::Tool { tool, .. } => ActiveGesture::Tool(tool),
        })
    }

    /// The minimap, while it shows: while `Space` is held with no text
    /// block open, during a pan with the middle button and during a drag
    /// in the minimap, in a viewport that can hold its panel. A drag in it
    /// keeps the world box of its press (ADR-T110-1).
    #[must_use]
    pub fn minimap(&self) -> Option<Minimap> {
        if !self.minimap_shown() {
            return None;
        }
        let content = self.content_bounds();
        if let Some(Gesture::Minimap { world, .. }) = self.gesture {
            return Minimap::with_world(self.viewport, content, world);
        }
        let view = self.camera.visible_world_rect(self.viewport);
        Minimap::new(self.viewport, content, view)
    }

    /// Whether the minimap is asked for and the viewport can hold it.
    fn minimap_shown(&self) -> bool {
        let asked = match self.gesture {
            Some(
                Gesture::Minimap { .. }
                | Gesture::Pan {
                    button: PointerButton::Middle,
                    ..
                },
            ) => true,
            _ => self.space_held && self.tools.text.edit.is_none(),
        };
        asked && minimap::panel_rect(self.viewport).is_some()
    }

    /// The bounds of every shape: the cached ones if they are of this
    /// document revision, else computed now.
    fn content_bounds(&self) -> Option<Aabb> {
        match self.minimap_content {
            Some((revision, bounds)) if revision == self.doc_revision => bounds,
            _ => minimap::content_bounds(&self.doc),
        }
    }

    /// Computes the content bounds for the minimap if it shows and the
    /// document changed since they were computed (ADR-T110-1).
    fn refresh_minimap_content(&mut self) {
        let fresh = matches!(self.minimap_content, Some((r, _)) if r == self.doc_revision);
        if !fresh && self.minimap_shown() {
            self.minimap_content = Some((self.doc_revision, minimap::content_bounds(&self.doc)));
        }
    }

    /// Pans so that `world` is at the centre of the viewport. Returns
    /// whether the camera changed.
    fn center_view(&mut self, world: Vec2) -> bool {
        let before = self.camera;
        let delta = self.viewport * 0.5 - self.camera.world_to_screen(world);
        self.camera.pan_by_screen(delta);
        self.camera != before
    }

    /// Centres the view on the point of the minimap under `pos`, with the
    /// world box `world` the drag started with. Returns whether the camera
    /// changed.
    fn minimap_drag(&mut self, world: Aabb, pos: Vec2) -> bool {
        Minimap::with_world(self.viewport, None, world)
            .is_some_and(|map| self.center_view(map.world_at(pos)))
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

    /// Whether the view may be exported now: no text block is open and no
    /// gesture is in progress (ADR-T92-2).
    #[must_use]
    pub fn can_export(&self) -> bool {
        self.tools.text.edit.is_none() && self.gesture.is_none()
    }

    /// Records an export request if the view may be exported now. An
    /// export never needs a redraw of its own, so this returns `false`.
    fn ask_export(&mut self) -> bool {
        self.export_asked |= self.can_export();
        false
    }

    /// The open shortcut sheet, if any (ADR-T113-1).
    #[must_use]
    pub fn sheet(&self) -> Option<&Sheet> {
        self.sheet.as_ref()
    }

    /// Records an SVG export request if the drawing may be exported now
    /// (no open text block, no gesture); never needs a redraw.
    fn ask_svg_export(&mut self) -> bool {
        self.svg_export_asked.0 |= self.can_export();
        false
    }

    /// Whether an SVG export was asked for since the last call
    /// (ADR-T114-2).
    pub fn take_svg_export_request(&mut self) -> bool {
        std::mem::take(&mut self.svg_export_asked).0
    }

    /// Records the selection's box for the shell to copy as an image, if
    /// the view may be exported now and something is selected. Needs no
    /// redraw, so this returns `false` (ADR-T115-1).
    fn ask_copy_image(&mut self) -> bool {
        if self.can_export()
            && let Some(bounds) = self.selection_bounds()
        {
            self.copy_image_asked = Some(bounds);
        }
        false
    }

    /// The world box of the selection to copy as an image, if it was asked
    /// for since the last call (ADR-T115-1).
    pub fn take_copy_image_request(&mut self) -> Option<Aabb> {
        self.copy_image_asked.take()
    }

    /// The open command palette, if any (ADR-T111-1).
    #[must_use]
    pub fn palette(&self) -> Option<&CommandPalette> {
        self.palette.as_ref()
    }

    /// Whether an export was asked for since the last call (ADR-T92-2).
    pub fn take_export_request(&mut self) -> bool {
        std::mem::take(&mut self.export_asked)
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

    /// Estimated heap bytes held by the undo history (ADR-T49-1).
    #[must_use]
    pub fn history_bytes(&self) -> usize {
        self.history.bytes()
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

    /// Starts a gesture, unless one is already running and does not yield
    /// to `button`. With a text block open, a left press inside it moves
    /// the caret and any other press that does not pan commits it first
    /// (ADR-T29-2).
    fn pointer_down(&mut self, pos: Vec2, button: PointerButton, mods: Modifiers) -> bool {
        let mut cancelled = false;
        if let Some(gesture) = self.gesture {
            if !gesture.yields_to(button) {
                return false;
            }
            cancelled = self.cancel_gesture();
        }
        // A press in the minimap moves the view there and never reaches a
        // tool (ADR-T110-1).
        if button == PointerButton::Left
            && self.space_held
            && let Some(map) = self.minimap().filter(|map| map.contains(pos))
        {
            self.gesture = Some(Gesture::Minimap {
                button,
                world: map.world(),
            });
            self.center_view(map.world_at(pos));
            return true;
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
            return cancelled;
        }
        let tool = if button == PointerButton::Right {
            Tool::Eraser
        } else {
            self.tool
        };
        self.gesture = Some(Gesture::Tool {
            button,
            tool,
            fresh: true,
        });
        self.tool_pointer(tool, Phase::Down, pos, mods) || committed || cancelled
    }

    /// Continues the gesture in progress, if any.
    fn pointer_move(&mut self, pos: Vec2, mods: Modifiers) -> bool {
        match &mut self.gesture {
            Some(Gesture::Pan { pan, .. }) => pan.drag(&mut self.camera, pos),
            Some(Gesture::Minimap { world, .. }) => {
                let world = *world;
                self.minimap_drag(world, pos)
            }
            Some(Gesture::Tool { tool, fresh, .. }) => {
                *fresh = false;
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
            Some(Gesture::Minimap { button: b, world }) if b == button => {
                self.gesture = None;
                self.minimap_drag(world, pos)
            }
            Some(Gesture::Tool {
                button: b, tool, ..
            }) if b == button => {
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
    /// bumps the document revision if it did (ADR-T24-1): the history's
    /// change count moved (ADR-T49-1). Unless `appends` (the edit only ever
    /// inserts shapes on top) and the document grew, the base revision
    /// follows (ADR-T24-3).
    fn track_document(&mut self, appends: bool, edit: impl FnOnce(&mut Self) -> bool) -> bool {
        let before = self.history.changes();
        let len = self.doc.len();
        let changed = edit(self);
        if self.history.changes() != before {
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
    fn run_edit(&mut self, edit: impl FnOnce(&mut ToolCtx<'_>) -> bool) -> bool {
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

    /// Replaces the only selected text block by the grid of its text, as
    /// one undo step (ADR-T84-1). Returns whether a redraw is needed.
    fn grid_from_selection(&mut self) -> bool {
        let &[id] = self.selection.as_slice() else {
            return false;
        };
        let Some(Shape::Text {
            origin,
            text,
            size,
            style,
        }) = self.doc.get(id)
        else {
            return false;
        };
        let (origin, size, color) = (*origin, *size, style.color);
        let parsed = TextTable::parse(text);
        let Some(grid) = self.table_grid(parsed, origin, size, color) else {
            return false;
        };
        self.run_edit(|ctx| {
            let tx = tx_replace(ctx.doc, id, grid);
            ctx.commit(tx)
        })
    }

    /// Replaces the only selected text block by the graph of its edges, as
    /// one undo step (ADR-T87-1). Returns whether a redraw is needed.
    fn graph_from_selection(&mut self) -> bool {
        // A `segtree:` block counts from the first number (ADR-T104-1).
        let first = self.style.helpers.first_number();
        self.block_to_shapes("graph", |text, origin, size, style| {
            let list = EdgeList::parse_numbered(text, first)?;
            Ok::<_, GraphError>(list.and_then(|list| list.shapes(origin, size, style)))
        })
    }

    /// Replaces the only selected text block by the coordinate plane of
    /// its points, as one undo step (ADR-T93-1). Returns whether a redraw
    /// is needed.
    fn plot_from_selection(&mut self) -> bool {
        self.block_to_shapes("plot", |text, origin, size, style| {
            let list = PointList::parse(text)?;
            Ok::<_, PlotError>(list.and_then(|list| list.shapes(origin, size, style)))
        })
    }

    /// Replaces the only selected text block by what `build` makes of its
    /// text, origin and size, in its colour and the current line width.
    /// A refusal becomes a notice naming `what`. Returns whether a redraw
    /// is needed.
    fn block_to_shapes<E: std::fmt::Display>(
        &mut self,
        what: &str,
        build: impl FnOnce(&str, Vec2, f32, Style) -> Result<Option<Vec<Shape>>, E>,
    ) -> bool {
        let &[id] = self.selection.as_slice() else {
            return false;
        };
        let Some(Shape::Text {
            origin,
            text,
            size,
            style,
        }) = self.doc.get(id)
        else {
            return false;
        };
        let style = Style {
            color: style.color,
            width: self.camera.world_len(self.style.width_px),
        };
        match build(text, *origin, *size, style) {
            Ok(Some(shapes)) => self.run_edit(|ctx| layout::replace_block(ctx, id, shapes)),
            Ok(None) => false,
            Err(err) => {
                self.notice = Some(format!("{what} from text refused: {err}"));
                false
            }
        }
    }

    /// Builds the grid of clipboard `text` at the cursor, as one undo step,
    /// and selects it (ADR-T84-1). Returns whether a redraw is needed.
    fn grid_from_clipboard(&mut self, text: &str) -> bool {
        if self.tools.text.edit.is_some() {
            return false;
        }
        let corner = self.camera.screen_to_world(self.cursor);
        let new_size = self.tools.text.new_size_px(self.style.width_px);
        let size = self.camera.world_len(new_size);
        let parsed = TextTable::parse(text);
        let Some(grid) = self.table_grid(parsed, corner, size, self.style.color) else {
            return false;
        };
        self.track_document(true, |ed| {
            ed.run_edit(|ctx| clipboard::insert_copies(ctx, vec![grid], Vec2::ZERO))
        })
    }

    /// The grid of a parsed table in the current line width, or `None`:
    /// there is no table, or it was refused and the reason is now the
    /// notice (ADR-T84-1).
    fn table_grid(
        &mut self,
        parsed: Result<Option<TextTable>, TableError>,
        corner: Vec2,
        text_size: f32,
        color: ColorId,
    ) -> Option<Shape> {
        let table = parsed
            .inspect_err(|err| self.notice = Some(format!("grid from text refused: {err}")))
            .ok()??;
        let style = Style {
            color,
            width: self.camera.world_len(self.style.width_px),
        };
        table.shape(corner, text_size, style, self.style.helpers.numbering)
    }

    /// Handles a key while a text block is open: editing keys edit it,
    /// `Ctrl+[` / `Ctrl+]` resize it, `Esc` and `Ctrl+Enter` commit it,
    /// `Ctrl+Z` cancels it, in a grid cell `Tab` and `Enter` (with or
    /// without `Shift`) move to another cell, every other key does nothing
    /// (ADR-T29-2, ADR-T29-3, ADR-T31-1). Returns whether a redraw is
    /// needed.
    #[allow(clippy::wildcard_enum_match_arm, reason = "key table, ADR-T131-1")]
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
    /// (ADR-T29-2). A resized block's on-screen size, kept to the text
    /// ladder's range, becomes the size of new blocks (ADR-T37-1,
    /// ADR-T76-1). Returns whether there was a block.
    fn finish_text(&mut self) -> bool {
        let Some(edit) = self.tools.text.edit.take() else {
            return false;
        };
        let px = edit.size * self.camera.zoom();
        if edit.resized() && px.is_finite() && px > 0.0 {
            let lowest = text::text_size_px(MIN_WIDTH_PX);
            let highest = text::text_size_px(MAX_WIDTH_PX);
            self.tools.text.size_px = Some(px.clamp(lowest, highest));
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
    /// topmost shape there and the only selected one; also the only
    /// selected shape if it can host text at `p` (ADR-T31-1, ADR-T52-1).
    fn selected_text_at(&self, p: Vec2, tol: f32) -> Option<ShapeId> {
        let &[selected] = self.selection.as_slice() else {
            return None;
        };
        match self.doc.topmost_where(|shape| shape.hit(p, tol)) {
            Some(id) if id == selected => {
                let shape = self.doc.get(id)?;
                let is_text = matches!(shape, Shape::Text { .. });
                (is_text || shape.text_host_at(p).is_some()).then_some(id)
            }
            None => {
                let shape = self.doc.get(selected)?;
                shape.text_host_at(p).is_some().then_some(selected)
            }
            Some(_) => None,
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
        self.tools.text.edit = self
            .doc
            .get(id)
            .and_then(|shape| match shape.text_host_at(at) {
                Some(slot) => TextEdit::open_in(id, shape, slot, size),
                None => TextEdit::open(id, shape, at),
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
            Some(Gesture::Pan { .. } | Gesture::Minimap { .. }) => true,
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

    /// Removes every unlocked shape as one undo step, after cancelling the
    /// gesture (ADR-T107-1). Returns whether anything changed.
    fn clear_all(&mut self) -> bool {
        let cancelled = self.cancel_gesture();
        let tx = tx_clear(&self.doc);
        if tx.is_empty() {
            return cancelled;
        }
        let changed = self.history.commit(&mut self.doc, tx).is_ok();
        self.selection.clear();
        changed || cancelled
    }

    /// Locks the selected shapes as one undo step and clears the selection,
    /// after cancelling the gesture (ADR-T107-2). Returns whether anything
    /// changed.
    fn lock_selection(&mut self) -> bool {
        let cancelled = self.cancel_gesture();
        self.prune_selection();
        let tx = tx_lock(&self.doc, &self.selection);
        let locked = !tx.is_empty() && self.history.commit(&mut self.doc, tx).is_ok();
        let deselected = !self.selection.is_empty();
        self.selection.clear();
        locked || deselected || cancelled
    }

    /// Unlocks every locked shape as one undo step, after cancelling the
    /// gesture (ADR-T107-2). Returns whether anything changed.
    fn unlock_all(&mut self) -> bool {
        let cancelled = self.cancel_gesture();
        let tx = tx_unlock_all(&self.doc);
        let unlocked = !tx.is_empty() && self.history.commit(&mut self.doc, tx).is_ok();
        unlocked || cancelled
    }

    /// The id and shape of the grid the arrow keys resize: the Select tool
    /// is active and that grid is the whole selection (ADR-T85-1).
    fn resizable_grid(&self) -> Option<(ShapeId, &Shape)> {
        let [id] = *self.selection.as_slice() else {
            return None;
        };
        let shape = self.doc.get(id)?;
        (self.tool == Tool::Select && matches!(shape, Shape::Grid { .. })).then_some((id, shape))
    }

    /// Adds `cols` columns and `rows` rows to the [`Self::resizable_grid`]
    /// as one undo step, cancelling the gesture in progress first. A step
    /// that changes nothing leaves the gesture alone. Returns whether
    /// anything changed.
    fn resize_selected_grid(&mut self, cols: i32, rows: i32) -> bool {
        let stepped = self
            .resizable_grid()
            .and_then(|(id, shape)| Some((id, shape.grid_stepped(cols, rows)?)));
        let Some((id, stepped)) = stepped else {
            return false;
        };
        self.cancel_gesture();
        let tx = tx_replace(&self.doc, id, stepped);
        self.history.commit(&mut self.doc, tx).is_ok()
    }

    /// Moves the selection up or down the stacking order as one undo step
    /// (ADR-T100-1). Does nothing during a gesture, with an empty
    /// selection, or when the order would not change. Returns whether the
    /// document changed.
    fn restack(&mut self, to: Restack) -> bool {
        if self.gesture.is_some() {
            return false;
        }
        self.prune_selection();
        let Some(bounds) = self.selection_bounds() else {
            return false;
        };
        let selected: IdSet = self.selection.iter().copied().collect();
        let is_selected = |(_, (id, _)): &(usize, (ShapeId, &Shape))| selected.contains(id);
        let (lowest, highest) = {
            let mut indices = self
                .doc
                .shapes()
                .enumerate()
                .filter(is_selected)
                .map(|(index, _)| index);
            let Some(lowest) = indices.next() else {
                return false;
            };
            (lowest, indices.next_back().unwrap_or(lowest))
        };
        let count = selected.len();
        let overlaps = |entry: &(usize, (ShapeId, &Shape))| {
            !is_selected(entry) && entry.1.1.bounds().intersects(&bounds)
        };
        // `below` counts the unselected shapes left under the block.
        let below = match to {
            Restack::Top => usize::MAX,
            Restack::Bottom => 0,
            Restack::Up => {
                let target = self
                    .doc
                    .shapes()
                    .enumerate()
                    .skip(highest + 1)
                    .find(overlaps);
                // Every selected shape lies under the target.
                let Some((index, _)) = target else {
                    return false;
                };
                index + 1 - count
            }
            Restack::Down => {
                let target = self
                    .doc
                    .shapes()
                    .enumerate()
                    .take(lowest)
                    .rev()
                    .find(overlaps);
                // No selected shape lies under the target.
                let Some((index, _)) = target else {
                    return false;
                };
                index
            }
        };
        let tx = tx_restack(&self.doc, &self.selection, below);
        !tx.is_empty() && self.history.commit(&mut self.doc, tx).is_ok()
    }

    /// Runs an align or distribute `edit` on the selection (ADR-T101-1).
    /// Unlike [`Self::run_edit`] it does nothing during a gesture rather
    /// than cancelling it. Returns whether the document changed.
    fn arrange(&mut self, edit: impl FnOnce(&mut ToolCtx<'_>) -> bool) -> bool {
        if self.gesture.is_some() {
            return false;
        }
        self.run_edit(edit)
    }

    /// Replaces the last pen stroke by the shape it looks like, as one undo
    /// step (ADR-T109-1). Does nothing during a gesture, when the newest
    /// undo step is not the insertion of one stroke, after an undo, or when
    /// the stroke fits nothing. Returns whether the document changed.
    fn recognize_last_stroke(&mut self) -> bool {
        if self.gesture.is_some() {
            return false;
        }
        let (undo, redo, _, _) = self.history.parts();
        let Some(last) = undo.back() else {
            return false;
        };
        let [
            Edit::Insert {
                id,
                shape: Shape::Stroke { .. },
                ..
            },
        ] = last.edits()
        else {
            return false;
        };
        let id = *id;
        if !redo.is_empty() {
            return false;
        }
        let Some(shape) = self.doc.get(id).and_then(recognize::recognize) else {
            return false;
        };
        let tx = tx_replace(&self.doc, id, shape);
        self.history.commit(&mut self.doc, tx).is_ok()
    }

    /// Removes selected ids that are no longer in the document or are
    /// locked (ADR-T107-1).
    fn prune_selection(&mut self) {
        select::prune(&mut self.selection, &self.doc);
    }
}

/// One slot per page (see [`Editor`]).
type PageSlots = Vec<Option<Box<Page>>>;

/// Reads a `camera` record: offset and a zoom inside the zoom range.
fn read_camera(r: &mut Reader<'_>) -> Result<Camera, SnapshotError> {
    let mut t = r.record("camera")?;
    let offset = t.get()?;
    let zoom = t.f32("zoom")?;
    t.end()?;
    if !(ZOOM_MIN..=ZOOM_MAX).contains(&zoom) {
        return Err(t.error(format!("zoom {zoom} outside [{ZOOM_MIN}, {ZOOM_MAX}]")));
    }
    Ok(Camera::new(offset, zoom))
}

/// Slots for the pages that are not current: all untouched.
fn empty_pages() -> PageSlots {
    vec![None; PAGE_COUNT]
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
            | Command::GridFromText
            | Command::GraphFromText
            | Command::PlotFromText
            | Command::DeleteSelection
            | Command::ClearAll
            | Command::LockSelection
            | Command::UnlockAll
            | Command::GridCols(_)
            | Command::GridRows(_)
            | Command::ScaleUp
            | Command::ScaleDown
            | Command::RaiseSelection
            | Command::LowerSelection
            | Command::SelectionToTop
            | Command::SelectionToBottom
            | Command::Align(_)
            | Command::Distribute(_)
            | Command::RecognizeShape
    )
}

/// Where a restack command moves the selection (ADR-T100-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Restack {
    /// Past the nearest overlapping unselected shape above it.
    Up,
    /// Past the nearest overlapping unselected shape below it.
    Down,
    /// Above every unselected shape.
    Top,
    /// Below every unselected shape.
    Bottom,
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
            let first = helpers.first_number();
            let changed = helpers.next_number != first;
            helpers.next_number = first;
            changed
        }
        Command::ToggleZeroBased => {
            helpers.zero_based = !helpers.zero_based;
            helpers.next_number = helpers.first_number();
            true
        }
        Command::GridCols(delta) => step_cells(&mut helpers.grid_cols, delta),
        Command::GridRows(delta) => step_cells(&mut helpers.grid_rows, delta),
        Command::SetTool(_)
        | Command::SetColor(_)
        | Command::WidthUp
        | Command::WidthDown
        | Command::CycleSmoothing
        | Command::Undo
        | Command::Redo
        | Command::Copy
        | Command::Cut
        | Command::Paste
        | Command::Duplicate
        | Command::SelectAll
        | Command::GridFromText
        | Command::GraphFromText
        | Command::ScaleUp
        | Command::ScaleDown
        | Command::PlotFromText
        | Command::RaiseSelection
        | Command::LowerSelection
        | Command::SelectionToTop
        | Command::SelectionToBottom
        | Command::Align(_)
        | Command::Distribute(_)
        | Command::RecognizeShape
        | Command::DeleteSelection
        | Command::ClearAll
        | Command::LockSelection
        | Command::UnlockAll
        | Command::Cancel
        | Command::ResetView
        | Command::FitView
        | Command::GoToPage(_)
        | Command::ToggleToolbar
        | Command::ExportView
        | Command::SaveFile
        | Command::ExportSvg
        | Command::CopyImage => false,
    }
}

/// Adds `delta` to the grid dimension `cells`, clamped to
/// `1..=GRID_MAX_CELLS`. Returns whether it changed.
fn step_cells(cells: &mut u32, delta: i32) -> bool {
    let stepped = stepped_cells(*cells, delta);
    let changed = *cells != stepped;
    *cells = stepped;
    changed
}

/// Reads the `document` section and checks that no numbered rectangle or
/// ellipse holds text (snapshots, ADR-T74-1, ADR-T52-1).
fn read_numbered_document(r: &mut Reader<'_>) -> Result<Document, SnapshotError> {
    let line = r.line();
    let doc = snapshot::read_document(r)?;
    let both = doc
        .shapes()
        .find(|(_, s)| s.label().is_some() && s.slot_text(TextSlot::Shape).is_some());
    match both {
        Some((id, _)) => Err(SnapshotError::new(
            line,
            format!("numbered shape {id:?} holds text"),
        )),
        None => Ok(doc),
    }
}

/// Reads the `style` record (width on the ladder) and the helpers.
fn read_style(r: &mut Reader<'_>) -> Result<DrawStyle, SnapshotError> {
    let mut t = r.record("style")?;
    let color = t.get()?;
    let width_px = t.f32("width")?;
    t.end()?;
    if !WIDTH_LADDER_PX
        .iter()
        .any(|w| w.to_bits() == width_px.to_bits())
    {
        return Err(t.error(format!("width {width_px} is not on the width ladder")));
    }
    let helpers = read_helpers(r)?;
    Ok(DrawStyle {
        color,
        width_px,
        helpers,
    })
}

/// Reads the `helpers` record (snapshots, ADR-T74-1).
fn read_helpers(r: &mut Reader<'_>) -> Result<Helpers, SnapshotError> {
    let mut t = r.record("helpers")?;
    let helpers = Helpers {
        smart_snap: t.bool("smart snap")?,
        grid_snap: t.bool("grid snap")?,
        numbering: t.bool("numbering")?,
        next_number: t.number("next number")?,
        grid_cols: t.number("grid columns")?,
        grid_rows: t.number("grid rows")?,
        // Last token, the old reserved one (ADR-T86-1).
        zero_based: t.bool("zero based")?,
    };
    t.end()?;
    let cells = 1..=GRID_MAX_CELLS;
    if helpers.next_number < helpers.first_number()
        || !cells.contains(&helpers.grid_cols)
        || !cells.contains(&helpers.grid_rows)
    {
        return Err(t.error("helper settings out of range"));
    }
    Ok(helpers)
}

/// Reads a record holding one flag, such as `toolbar` or `space`
/// (snapshots, ADR-T74-1).
fn read_flag(r: &mut Reader<'_>, key: &str) -> Result<bool, SnapshotError> {
    let mut t = r.record(key)?;
    let flag = t.bool(key)?;
    t.end()?;
    Ok(flag)
}

/// Reads the `gesture` record (snapshots, ADR-T74-1).
fn read_gesture(r: &mut Reader<'_>) -> Result<Option<Gesture>, SnapshotError> {
    let mut t = r.record("gesture")?;
    let gesture = if t.none() {
        None
    } else {
        match t.word("gesture")? {
            "pan" => Some(Gesture::Pan {
                button: t.get()?,
                pan: Pan::new(t.get()?),
            }),
            "minimap" => Some(Gesture::Minimap {
                button: t.get()?,
                world: t.get()?,
            }),
            "tool" => Some(Gesture::Tool {
                button: t.get()?,
                tool: t.get()?,
                fresh: t.bool("fresh")?,
            }),
            other => return Err(t.error(format!("unknown gesture `{other}`"))),
        }
    };
    t.end()?;
    Ok(gesture)
}

/// Reads the `[history]` section (snapshots, ADR-T74-1).
fn read_history(r: &mut Reader<'_>) -> Result<History, SnapshotError> {
    r.section("history")?;
    let mut t = r.record("limits")?;
    let limit: usize = t.number("undo limit")?;
    let budget: usize = t.number("byte budget")?;
    t.end()?;
    if limit == 0 {
        return Err(t.error("undo limit is 0"));
    }
    let undo = snapshot::read_transactions(r, "undo")?;
    if undo.len() > limit {
        return Err(SnapshotError::new(
            r.line(),
            format!("{} undo entries above the limit {limit}", undo.len()),
        ));
    }
    let redo = snapshot::read_transactions(r, "redo")?;
    let history = History::from_parts(undo.into(), redo, limit, budget);
    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::{Camera, ZOOM_STEP};
    use crate::core::document::tx_insert;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::history::HISTORY_LIMIT;
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use crate::core::numbering::FIRST_NUMBER;
    use crate::core::palette::ColorId;
    use crate::core::shape::GRID_MAX_CELLS;
    use crate::core::shape::{Shape, ShapeText, Style, TextSlot};
    use crate::core::sheet;
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
        assert_eq!(ed.selection(), []);
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

        assert_eq!(ed.selection(), []);
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

        assert_eq!(ed.selection(), []);
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

        assert_eq!(ed.selection(), []);
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
    fn reset_numbering_restarts_at_the_first_number() {
        // Arrange
        let mut ed = Editor::new();
        ed.apply(Command::ToggleZeroBased);
        ed.style.helpers.next_number = 5;

        // Act / Assert
        assert!(ed.apply(Command::ResetNumbering));
        assert_eq!(ed.helpers().next_number, 0);
        assert!(!ed.apply(Command::ResetNumbering), "already at the start");
        assert!(ed.helpers().zero_based, "the switch stays");
    }

    #[test]
    fn zero_based_switch_flips_and_restarts_the_counter() {
        // Arrange
        let mut ed = Editor::new();
        ed.style.helpers.next_number = 5;

        // Act / Assert: to 0 …
        assert!(ed.apply(Command::ToggleZeroBased));
        assert!(ed.helpers().zero_based);
        assert_eq!(ed.helpers().first_number(), 0);
        assert_eq!(ed.helpers().next_number, 0);

        // … and back to 1.
        ed.style.helpers.next_number = 3;
        assert!(ed.apply(Command::ToggleZeroBased));
        assert!(!ed.helpers().zero_based);
        assert_eq!(ed.helpers().first_number(), FIRST_NUMBER);
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER);
    }

    #[test]
    fn zero_based_switch_keeps_the_gesture() {
        // Arrange: a drag in progress.
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Ellipse));
        down(&mut ed, PointerButton::Left, 100.0, 100.0);

        // Act
        let changed = ed.apply(Command::ToggleZeroBased);

        // Assert
        assert!(changed);
        assert_eq!(
            ed.active_gesture(),
            Some(ActiveGesture::Tool(Tool::Ellipse))
        );
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

    // ---- T85 resizing a committed grid -------------------------------------

    /// Unfilled `cols × rows` grid at (100, 100) with cells of 100 × 100.
    fn plain_grid(cols: u32, rows: u32) -> Shape {
        Shape::Grid {
            a: Vec2::new(100.0, 100.0),
            b: Vec2::new(100.0 + 100.0 * cols as f32, 100.0 + 100.0 * rows as f32),
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

    /// Editor holding `shapes`, all selected, with the Select tool active.
    fn selected_editor(shapes: Vec<Shape>) -> Editor {
        let mut ed = editor_with(shapes);
        ed.apply(Command::SetTool(Tool::Select));
        ed.apply(Command::SelectAll);
        ed
    }

    fn arrow(ed: &mut Editor, key: Key) -> bool {
        ed.handle(InputEvent::KeyDown { key, mods: NONE })
    }

    /// `(cols, rows)` of the bottom shape; panics if it is not a grid.
    fn first_grid_size(ed: &Editor) -> (u32, u32) {
        match ed.document().shapes().next() {
            Some((_, Shape::Grid { cols, rows, .. })) => (*cols, *rows),
            other => panic!("not a grid: {other:?}"),
        }
    }

    fn helper_size(ed: &Editor) -> (u32, u32) {
        (ed.helpers().grid_cols, ed.helpers().grid_rows)
    }

    #[test]
    fn grid_resize_arrows_change_selected_grid_not_helpers() {
        // Arrange
        let mut ed = selected_editor(vec![plain_grid(4, 4)]);
        let helpers = helper_size(&ed);
        let selection = ed.selection().to_vec();

        // Act / Assert
        assert!(arrow(&mut ed, Key::ArrowRight));
        assert_eq!(first_grid_size(&ed), (5, 4));
        assert!(arrow(&mut ed, Key::ArrowDown));
        assert_eq!(first_grid_size(&ed), (5, 5));
        assert!(arrow(&mut ed, Key::ArrowLeft));
        assert_eq!(first_grid_size(&ed), (4, 5));
        assert!(arrow(&mut ed, Key::ArrowUp));
        assert_eq!(first_grid_size(&ed), (4, 4));

        assert_eq!(helper_size(&ed), helpers);
        assert_eq!(ed.selection(), &selection[..]);
    }

    #[test]
    fn grid_resize_clamps_to_cell_limits() {
        // Arrange
        let mut smallest = selected_editor(vec![plain_grid(1, 1)]);
        let mut largest = selected_editor(vec![plain_grid(GRID_MAX_CELLS, GRID_MAX_CELLS)]);
        let helpers = helper_size(&smallest);

        // Act / Assert
        assert!(!arrow(&mut smallest, Key::ArrowLeft));
        assert!(!arrow(&mut smallest, Key::ArrowUp));
        assert_eq!(first_grid_size(&smallest), (1, 1));
        assert!(!arrow(&mut largest, Key::ArrowRight));
        assert!(!arrow(&mut largest, Key::ArrowDown));
        assert_eq!(first_grid_size(&largest), (GRID_MAX_CELLS, GRID_MAX_CELLS));
        assert_eq!(helper_size(&smallest), helpers);
        assert_eq!(helper_size(&largest), helpers);
    }

    #[test]
    fn grid_resize_undo_restores_removed_cells() {
        // Arrange: text and a fill in the last column.
        let mut shape = plain_grid(2, 1);
        if let Shape::Grid { fills, texts, .. } = &mut shape {
            fills.push(crate::core::shape::CellFill {
                col: 1,
                row: 0,
                color: ColorId::INK,
            });
            texts.push(crate::core::shape::CellText {
                col: 1,
                row: 0,
                content: ShapeText {
                    text: "dp".to_owned(),
                    size: 16.0,
                },
            });
        }
        let mut ed = selected_editor(vec![shape.clone()]);

        // Act
        arrow(&mut ed, Key::ArrowLeft);
        let shrunk = ed.document().shapes().next().map(|(_, s)| s.clone());
        ed.apply(Command::Undo);
        let restored = ed.document().shapes().next().map(|(_, s)| s.clone());

        // Assert
        assert!(shrunk.as_ref().is_some_and(|s| s.cell_fills().is_empty()));
        assert!(shrunk.as_ref().is_some_and(|s| s.cell_texts().is_empty()));
        assert_eq!(restored, Some(shape));
        assert_eq!(first_grid_size(&ed), (2, 1));
    }

    #[test]
    fn grid_resize_each_press_is_one_undo_step() {
        // Arrange
        let mut ed = selected_editor(vec![plain_grid(4, 4)]);
        let before = ed.history.undo_len();

        // Act
        for _ in 0..3 {
            arrow(&mut ed, Key::ArrowRight);
        }

        // Assert
        assert_eq!(ed.history.undo_len(), before + 3);
        ed.apply(Command::Undo);
        assert_eq!(first_grid_size(&ed), (6, 4));
    }

    #[test]
    fn grid_resize_at_limit_adds_no_undo_step() {
        // Arrange
        let mut ed = selected_editor(vec![plain_grid(GRID_MAX_CELLS, 1)]);
        let before = ed.history.undo_len();
        let revision = ed.document_revision();

        // Act
        let changed = arrow(&mut ed, Key::ArrowRight);

        // Assert
        assert!(!changed);
        assert_eq!(ed.history.undo_len(), before);
        assert_eq!(ed.document_revision(), revision);
    }

    #[test]
    fn grid_resize_marks_the_document_changed() {
        // Arrange
        let mut ed = selected_editor(vec![plain_grid(4, 4)]);
        let revision = ed.document_revision();

        // Act
        arrow(&mut ed, Key::ArrowRight);

        // Assert: not an append, so the cached layer must be re-rendered.
        assert_ne!(ed.document_revision(), revision);
        assert_eq!(ed.document_base_revision(), ed.document_revision());
    }

    #[test]
    fn grid_arrows_change_helpers_unless_one_grid_is_selected() {
        // Arrange: nothing selected, another shape, two grids.
        let mut nothing = editor_with(vec![plain_grid(4, 4)]);
        nothing.apply(Command::SetTool(Tool::Select));
        let mut other = selected_editor(vec![plain_grid(4, 4), rect(0.0, 0.0, 10.0, 10.0)]);
        other.selection.remove(0);
        let mut two = selected_editor(vec![plain_grid(4, 4), plain_grid(4, 4)]);

        for ed in [&mut nothing, &mut other, &mut two] {
            let before = ed.history.undo_len();

            // Act
            let changed = arrow(ed, Key::ArrowRight);

            // Assert
            assert!(changed);
            assert_eq!(helper_size(ed), (5, 4));
            assert_eq!(first_grid_size(ed), (4, 4));
            assert_eq!(ed.history.undo_len(), before);
        }
    }

    #[test]
    fn grid_arrows_change_helpers_with_another_tool() {
        // Arrange: the grid stays selected while the Grid tool is active.
        let mut ed = selected_editor(vec![plain_grid(4, 4)]);
        ed.apply(Command::SetTool(Tool::Grid));
        assert_eq!(ed.selection().len(), 1);

        // Act
        let changed = arrow(&mut ed, Key::ArrowDown);

        // Assert
        assert!(changed);
        assert_eq!(helper_size(&ed), (4, 5));
        assert_eq!(first_grid_size(&ed), (4, 4));
    }

    #[test]
    fn grid_resize_during_a_move_cancels_it() {
        // Arrange: dragging the selected grid by its left edge.
        let mut ed = selected_editor(vec![plain_grid(4, 4)]);
        down(&mut ed, PointerButton::Left, 100.0, 150.0);
        move_to(&mut ed, 180.0, 190.0);
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Tool(Tool::Select)));

        // Act
        let changed = arrow(&mut ed, Key::ArrowRight);

        // Assert: resized where it was, and the drag is over.
        assert!(changed);
        assert_eq!(ed.active_gesture(), None);
        assert_eq!(first_grid_size(&ed), (5, 4));
        let bounds = ed.document().shapes().next().map(|(_, s)| s.bounds().min);
        assert!(bounds.is_some_and(|min| approx_eq(min.x, 100.0, 2.0)));
    }

    #[test]
    fn grid_resize_at_limit_keeps_the_gesture() {
        // Arrange
        let mut ed = selected_editor(vec![plain_grid(GRID_MAX_CELLS, 2)]);
        down(&mut ed, PointerButton::Left, 100.0, 150.0);
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Tool(Tool::Select)));

        // Act
        let changed = arrow(&mut ed, Key::ArrowRight);

        // Assert
        assert!(!changed);
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Tool(Tool::Select)));
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
    fn zero_based_nodes_count_from_zero() {
        // Arrange
        let mut ed = numbering_editor();
        ed.apply(Command::ToggleZeroBased);

        // Act
        for x in [0.0, 100.0, 200.0] {
            draw_node(&mut ed, x);
        }

        // Assert
        assert_eq!(labels(&ed), vec![Some(0), Some(1), Some(2)]);
        assert_eq!(ed.helpers().next_number, 3);
    }

    #[test]
    fn zero_based_undo_gives_zero_back_and_stops_there() {
        // Arrange: nodes 0 and 1.
        let mut ed = numbering_editor();
        ed.apply(Command::ToggleZeroBased);
        draw_node(&mut ed, 0.0);
        draw_node(&mut ed, 100.0);
        assert_eq!(ed.helpers().next_number, 2);

        // Act / Assert: undo all the way, past the bottom.
        ed.apply(Command::Undo);
        assert_eq!(ed.helpers().next_number, 1);
        ed.apply(Command::Undo);
        assert_eq!(ed.helpers().next_number, 0);
        assert_eq!(labels(&ed), Vec::<Option<u32>>::new());
        ed.apply(Command::Undo);
        assert_eq!(ed.helpers().next_number, 0);

        // Redo takes the numbers again.
        ed.apply(Command::Redo);
        ed.apply(Command::Redo);
        assert_eq!(labels(&ed), vec![Some(0), Some(1)]);
        assert_eq!(ed.helpers().next_number, 2);
    }

    #[test]
    fn one_based_undo_ignores_labels_left_at_zero() {
        // Arrange: node 0, then back to one-based.
        let mut ed = numbering_editor();
        ed.apply(Command::ToggleZeroBased);
        draw_node(&mut ed, 0.0);
        ed.apply(Command::ToggleZeroBased);
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER);
        assert_eq!(labels(&ed), vec![Some(0)], "existing labels stay");

        // Act
        ed.apply(Command::Undo);

        // Assert
        assert_eq!(labels(&ed), Vec::<Option<u32>>::new());
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER);
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
        assert_ne!(ed.overlay().hidden.len(), 0);

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
            Command::ToggleZeroBased,
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
        assert_eq!(texts(&ed), Vec::<String>::new());
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

    /// Editor with an 800 × 600 viewport holding `shapes` and `tool`
    /// active.
    fn shape_text_editor(shapes: Vec<Shape>, tool: Tool) -> Editor {
        let mut ed = editor_with(shapes);
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
    fn shape_text_text_tool_click_in_rect_edits_its_text() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Text);
        let id = ids(&ed)[0];

        // Act
        place(&mut ed, 100.0, 50.0, "ab");

        // Assert: editing the rectangle, not a free block.
        assert_eq!(editing_host(&ed), Some((id, TextSlot::Shape)));
        assert_eq!(editing(&ed).as_deref(), Some("ab"));
        key(&mut ed, Key::Escape);
        assert_eq!(texts(&ed), Vec::<String>::new());
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("ab"));
        let Some((_, shape)) = ed.document().shapes().next() else {
            panic!("the rectangle stays");
        };
        let size = shape.slot_text(TextSlot::Shape).map(|t| t.size);
        assert!(size.is_some_and(|s| approx_eq(s, 18.0, 1e-4)), "{size:?}");
    }

    #[test]
    fn shape_text_text_tool_click_in_ellipse_edits_its_text() {
        let mut ed = shape_text_editor(vec![ellipse(0.0, 0.0, 200.0, 100.0)], Tool::Text);

        place(&mut ed, 100.0, 50.0, "3");
        key(&mut ed, Key::Escape);

        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("3"));
        assert_eq!(texts(&ed), Vec::<String>::new());
    }

    #[test]
    fn shape_text_text_tool_click_in_grid_edits_that_cell() {
        // Arrange: 4 × 4 cells of 50.
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 200.0, 200.0, 4, 4)], Tool::Text);

        // Act
        place(&mut ed, 60.0, 110.0, "7");
        key(&mut ed, Key::Escape);

        // Assert
        assert_eq!(slot_text(&ed, 0, cell(1, 2)).as_deref(), Some("7"));
        assert_eq!(slot_text(&ed, 0, cell(0, 0)), None);
        assert_eq!(texts(&ed), Vec::<String>::new());
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
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0), block], Tool::Text);
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
        let mut ed = shape_text_editor(shapes, Tool::Select);
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
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![g.clone()], Tool::Text);
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
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 250.0, 250.0, 5, 5)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 150.0, 100.0, 3, 2)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 100.0, 100.0, 2, 2)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Text);

        place(&mut ed, 100.0, 50.0, "a");
        key(&mut ed, Key::Enter);
        typed(&mut ed, "b");

        assert_eq!(editing(&ed).as_deref(), Some("a\nb"));
    }

    #[test]
    fn shape_text_undo_restores_cells_one_edit_at_a_time() {
        // Arrange: fill three cells with Tab.
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 150.0, 50.0, 3, 1)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![r], Tool::Select);

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
        let mut ed = shape_text_editor(vec![r], Tool::Pen);

        // A right drag across the outline removes the shape and its text.
        down(&mut ed, PointerButton::Right, 0.0, 40.0);
        move_to(&mut ed, 0.0, 60.0);
        up(&mut ed, PointerButton::Right, 0.0, 60.0);
        assert!(ed.document().is_empty());

        ed.apply(Command::Undo);
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("n"));
    }

    // ---- T52 numbered shapes hold no text --------------------------------

    /// The label of the `index`-th shape (bottom first).
    fn label_of(ed: &Editor, index: usize) -> Option<u32> {
        ed.document()
            .shapes()
            .nth(index)
            .and_then(|(_, s)| s.label())
    }

    #[test]
    fn shape_text_text_tool_in_numbered_rect_starts_free_block() {
        // Arrange: a numbered rectangle (label 1, counter 2).
        let mut ed = shape_text_editor(Vec::new(), Tool::Rect);
        ed.apply(Command::ToggleNumbering);
        down(&mut ed, PointerButton::Left, 0.0, 0.0);
        move_to(&mut ed, 100.0, 60.0);
        up(&mut ed, PointerButton::Left, 200.0, 100.0);
        assert_eq!(ed.helpers().next_number, 2);

        // Act: the Text tool clicks inside it and types.
        ed.apply(Command::SetTool(Tool::Text));
        place(&mut ed, 100.0, 50.0, "A");
        assert_eq!(editing_host(&ed), None, "a free block, not the shape");
        key(&mut ed, Key::Escape);

        // Assert: a free text block; the rectangle keeps its number only.
        assert_eq!(texts(&ed), ["A"]);
        assert_eq!(label_of(&ed, 0), Some(1));
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape), None);
        assert_eq!(ed.helpers().next_number, 2);
    }

    #[test]
    fn shape_text_select_second_click_on_numbered_ellipse_opens_nothing() {
        // Arrange
        let numbered = ellipse(0.0, 0.0, 200.0, 100.0).with_label(Some(3));
        let mut ed = shape_text_editor(vec![numbered], Tool::Select);
        click(&mut ed, 0.0, 50.0);
        assert_eq!(ed.selection().len(), 1);

        // Act: a second click inside, then typing.
        click(&mut ed, 100.0, 50.0);
        typed(&mut ed, "ab");

        // Assert
        assert!(ed.text_edit().is_none());
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape), None);
        assert_eq!(label_of(&ed, 0), Some(3));
        assert_eq!(texts(&ed), Vec::<String>::new());
    }

    #[test]
    fn shape_text_typing_with_numbered_rect_selected_adds_no_text() {
        // Arrange
        let numbered = rect(0.0, 0.0, 200.0, 100.0).with_label(Some(1));
        let mut ed = shape_text_editor(vec![numbered], Tool::Select);
        click(&mut ed, 0.0, 50.0);
        assert_eq!(ed.selection().len(), 1);

        // Act
        typed(&mut ed, "xy");
        key(&mut ed, Key::Enter);

        // Assert
        assert!(ed.text_edit().is_none());
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape), None);
        assert_eq!(label_of(&ed, 0), Some(1));
    }

    #[test]
    fn shape_text_paste_and_duplicate_keep_numbered_shapes_text_free() {
        // Arrange: a numbered rectangle and a named one.
        let numbered = rect(0.0, 0.0, 100.0, 100.0).with_label(Some(1));
        let named = ellipse(200.0, 0.0, 100.0, 100.0)
            .with_slot_text(TextSlot::Shape, ShapeText::new("u".to_owned(), 18.0));
        let mut ed = shape_text_editor(vec![numbered, named], Tool::Select);

        // Act
        ed.apply(Command::SelectAll);
        ed.apply(Command::Duplicate);
        ed.apply(Command::Copy);
        ed.apply(Command::Paste);

        // Assert: three of each, none both numbered and named.
        let shapes: Vec<&Shape> = ed.document().shapes().map(|(_, s)| s).collect();
        assert_eq!(shapes.len(), 6);
        let labelled = shapes.iter().filter(|s| s.label().is_some()).count();
        let named = shapes
            .iter()
            .filter(|s| s.slot_text(TextSlot::Shape).is_some())
            .count();
        assert_eq!((labelled, named), (3, 3));
        assert!(
            shapes
                .iter()
                .all(|s| s.label().is_none() || s.slot_text(TextSlot::Shape).is_none())
        );
    }

    #[test]
    fn shape_text_numbered_grid_cells_hold_text() {
        // Arrange: a grid with indices on (numbering drives them, T22).
        let mut indexed = grid(0.0, 0.0, 200.0, 200.0, 4, 4);
        if let Shape::Grid { axes, .. } = &mut indexed {
            *axes = true;
        }
        let mut ed = shape_text_editor(vec![indexed], Tool::Text);
        ed.apply(Command::ToggleNumbering);

        // Act
        place(&mut ed, 60.0, 110.0, "7");
        key(&mut ed, Key::Escape);

        // Assert
        assert_eq!(slot_text(&ed, 0, cell(1, 2)).as_deref(), Some("7"));
    }

    /// `text` with the last token of its `helpers` record set to `token`.
    fn with_helpers_last_token(text: &str, token: &str) -> String {
        text.lines()
            .map(|line| match line.strip_prefix("helpers ") {
                Some(rest) => {
                    let (head, _) = rest.rsplit_once(' ').unwrap_or((rest, ""));
                    format!("helpers {head} {token}")
                }
                None => line.to_owned(),
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    #[test]
    fn snapshot_helpers_last_token_is_zero_based() {
        // Arrange: a rectangle holding text and a filled grid cell.
        let named = rect(0.0, 0.0, 200.0, 100.0)
            .with_slot_text(TextSlot::Shape, ShapeText::new("n".to_owned(), 18.0));
        let g = grid(300.0, 0.0, 100.0, 100.0, 2, 2)
            .with_slot_text(cell(1, 1), ShapeText::new("c".to_owned(), 18.0));
        let ed = shape_text_editor(vec![named, g], Tool::Select);
        let text = ed.snapshot().to_text();

        // Act / Assert: one-based is written as 0, what the token always
        // held while it was reserved (ADR-T86-1) …
        assert!(
            text.lines()
                .any(|l| l.starts_with("helpers ") && l.ends_with(" 0")),
            "{text}"
        );
        let Ok(same) = Snapshot::parse(&with_helpers_last_token(&text, "0")) else {
            panic!("token 0 is valid");
        };
        assert_eq!(Editor::from_snapshot(same), ed);
        // … and 1 loads a zero-based editor.
        let Ok(zero) = Snapshot::parse(&with_helpers_last_token(&text, "1")) else {
            panic!("token 1 is valid");
        };
        let zero = Editor::from_snapshot(zero);
        assert!(zero.helpers().zero_based);
        assert_eq!(zero.helpers().first_number(), 0);
    }

    #[test]
    fn read_snapshot_rejects_numbered_shape_with_text() {
        // Arrange: a rectangle with both a label and text, built directly.
        let both = Shape::Rect {
            a: Vec2::ZERO,
            b: Vec2::new(100.0, 100.0),
            style: Style {
                color: ColorId::INK,
                width: 1.0,
            },
            fill: None,
            label: Some(1),
            text: ShapeText::new("n".to_owned(), 18.0),
        };
        let text = editor_with(vec![both]).snapshot().to_text();

        // Act
        let result = Snapshot::parse(&text);

        // Assert
        let Err(err) = result else {
            panic!("a numbered shape with text loaded");
        };
        assert!(err.line >= 1, "{err}");
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

    // ---- T92 export the view ------------------------------------------------

    fn ctrl_e(ed: &mut Editor) -> bool {
        key_with(ed, Key::E, CTRL)
    }

    #[test]
    fn export_view_when_idle_asks_once() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let before = ed.clone();

        // Act
        let redraw = ctrl_e(&mut ed);

        // Assert: nothing but the request changes.
        assert!(!redraw);
        assert_eq!(ed, before);
        assert!(ed.take_export_request());
        assert!(!ed.take_export_request(), "a request is answered once");
    }

    #[test]
    fn save_file_asks_once_and_changes_nothing_else() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let before = ed.clone();

        let redraw = key_with(&mut ed, Key::S, CTRL);

        assert!(!redraw);
        assert_eq!(ed, before);
        assert!(ed.take_save_request());
        assert!(!ed.take_save_request(), "a request is answered once");
        assert!(!Editor::new().take_save_request());
    }

    #[test]
    fn save_file_during_text_edit_is_ignored_like_every_binding() {
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Text));
        place(&mut ed, 5.0, 5.0, "a");
        assert!(ed.text_edit().is_some());

        key_with(&mut ed, Key::S, CTRL);

        assert!(ed.text_edit().is_some(), "the block stays open");
        assert!(!ed.take_save_request());
    }

    #[test]
    fn save_file_command_during_text_edit_commits_the_text_and_asks() {
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Text));
        place(&mut ed, 5.0, 5.0, "a");

        ed.apply(Command::SaveFile);

        assert!(ed.text_edit().is_none(), "the block was committed");
        assert_eq!(ed.document().shapes().count(), 1);
        assert!(ed.take_save_request());
    }

    #[test]
    fn save_file_during_a_gesture_still_asks() {
        let mut ed = Editor::new();
        down(&mut ed, PointerButton::Left, 5.0, 5.0);
        assert!(ed.active_gesture().is_some());

        key_with(&mut ed, Key::S, CTRL);

        assert!(ed.take_save_request());
    }

    #[test]
    fn export_view_is_not_requested_by_default() {
        assert!(!Editor::new().take_export_request());
    }

    #[test]
    fn export_view_during_text_edit_is_ignored() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "hi");
        assert!(ed.text_edit().is_some());

        // Act
        ctrl_e(&mut ed);

        // Assert
        assert!(!ed.take_export_request());
        assert!(ed.text_edit().is_some(), "the block stays open");
    }

    #[test]
    fn export_view_command_during_text_edit_keeps_the_block_open() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "hi");

        // Act
        let redraw = ed.apply(Command::ExportView);

        // Assert
        assert!(!redraw);
        assert!(!ed.take_export_request());
        assert!(ed.text_edit().is_some(), "the block is not committed");
        assert!(ed.document().is_empty());
    }

    #[test]
    fn export_view_during_gesture_is_ignored() {
        // Arrange: a pen stroke in progress.
        let mut ed = Editor::new();
        down(&mut ed, PointerButton::Left, 10.0, 10.0);
        move_to(&mut ed, 20.0, 20.0);
        assert!(ed.active_gesture().is_some());

        // Act
        ctrl_e(&mut ed);

        // Assert
        assert!(!ed.take_export_request());
        assert!(ed.active_gesture().is_some(), "the stroke goes on");
    }

    #[test]
    fn can_export_is_false_during_text_edit_and_gesture() {
        // Arrange
        let idle = Editor::new();
        let mut drawing = Editor::new();
        down(&mut drawing, PointerButton::Left, 10.0, 10.0);
        let mut typing = text_editor();
        place(&mut typing, 100.0, 100.0, "hi");

        // Act / Assert
        assert!(idle.can_export());
        assert!(!drawing.can_export());
        assert!(!typing.can_export());
    }

    // ---- T114 export to SVG -------------------------------------------------

    #[test]
    fn export_svg_when_idle_asks_once_and_leaves_the_png_request_alone() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let before = ed.clone();

        // Act
        let redraw = key_with(&mut ed, Key::E, CTRL_SHIFT);

        // Assert
        assert!(!redraw);
        assert_eq!(ed, before, "nothing but the request changes");
        assert!(!ed.take_export_request(), "not a PNG export");
        assert!(ed.take_svg_export_request());
        assert!(!ed.take_svg_export_request(), "a request is answered once");
    }

    #[test]
    fn export_svg_is_not_requested_by_default() {
        assert!(!Editor::new().take_svg_export_request());
    }

    #[test]
    fn export_svg_during_text_edit_is_ignored() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "hi");

        // Act
        let redraw = ed.apply(Command::ExportSvg);

        // Assert
        assert!(!redraw);
        assert!(!ed.take_svg_export_request());
        assert!(ed.text_edit().is_some(), "the block stays open");
    }

    #[test]
    fn export_svg_during_gesture_is_ignored() {
        // Arrange
        let mut ed = Editor::new();
        down(&mut ed, PointerButton::Left, 10.0, 10.0);
        move_to(&mut ed, 20.0, 20.0);

        // Act
        ed.apply(Command::ExportSvg);

        // Assert
        assert!(!ed.take_svg_export_request());
        assert!(ed.active_gesture().is_some(), "the stroke goes on");
    }

    // ---- T115 copy the selection as an image --------------------------------

    fn ctrl_shift_c(ed: &mut Editor) -> bool {
        let mods = Modifiers {
            ctrl: true,
            shift: true,
            ..NONE
        };
        key_with(ed, Key::C, mods)
    }

    #[test]
    fn copy_image_with_nothing_selected_does_nothing() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        let before = ed.clone();

        // Act
        let redraw = ctrl_shift_c(&mut ed);

        // Assert
        assert!(!redraw);
        assert_eq!(ed, before);
        assert!(ed.take_copy_image_request().is_none());
    }

    #[test]
    fn copy_image_with_a_selection_asks_once_for_its_box() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        key_with(&mut ed, Key::A, CTRL);
        let expected = ed.selection_bounds();
        assert!(expected.is_some());

        // Act
        let redraw = ctrl_shift_c(&mut ed);

        // Assert
        assert!(!redraw, "a copy needs no redraw");
        assert_eq!(ed.take_copy_image_request(), expected);
        assert!(ed.take_copy_image_request().is_none());
    }

    #[test]
    fn copy_image_during_text_edit_is_ignored() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");
        assert!(ed.text_edit().is_some());

        // Act
        ctrl_shift_c(&mut ed);

        // Assert
        assert!(ed.text_edit().is_some(), "the block stays open");
        assert!(ed.take_copy_image_request().is_none());
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
        assert_eq!(block_sizes(&ed), []);

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

    // ---- T81 formula box too narrow while editing ---------------------------

    /// The `index`-th shape's id and box (bottom first).
    fn shape_box(ed: &Editor, index: usize) -> (ShapeId, Aabb) {
        let (id, shape) = ed
            .document()
            .shapes()
            .nth(index)
            .unwrap_or_else(|| panic!("no shape {index}"));
        (id, shape.bounds())
    }

    fn assert_same_box(actual: Option<Aabb>, expected: Aabb) {
        let Some(actual) = actual else {
            panic!("no box, expected {expected:?}");
        };
        let close = |a: Vec2, b: Vec2| approx_eq(a.x, b.x, 1e-3) && approx_eq(a.y, b.y, 1e-3);
        assert!(
            close(actual.min, expected.min) && close(actual.max, expected.max),
            "{actual:?} {expected:?}"
        );
    }

    #[test]
    fn regression_t81_formula_box_while_editing() {
        // Arrange: a block with a formula, selected, not being edited.
        let snapshot = Snapshot::parse(include_str!(
            "../../tests/snapshots/regressions/T81-formula-box-while-editing.snap"
        ))
        .unwrap_or_else(|err| panic!("{err}"));
        let mut ed = Editor::from_snapshot(snapshot);
        let (id, typeset) = shape_box(&ed, 0);
        assert_eq!(ed.selection(), [id]);
        assert!(ed.text_edit().is_none());

        // Act: a second click re-opens it.
        click(&mut ed, 110.0, 60.0);

        // Assert: only the source box is left to outline.
        let edit = ed
            .text_edit()
            .unwrap_or_else(|| panic!("the click re-opens the block"));
        assert_eq!(edit.target(), Some(id));
        assert!(edit.bounds().width() > typeset.width());
        assert_eq!(ed.selection(), [id]);
        assert_eq!(ed.selection_bounds(), None);
    }

    #[test]
    fn selection_bounds_after_text_edit_is_the_typeset_box() {
        // Arrange: a formula block, re-opened with the Select tool.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "asdf$\\frac{a}{b}x$");
        key(&mut ed, Key::Escape);
        ed.apply(Command::SetTool(Tool::Select));
        click(&mut ed, 110.0, 60.0);
        click(&mut ed, 110.0, 60.0);
        assert!(ed.text_edit().is_some());

        // Act
        key(&mut ed, Key::Escape);

        // Assert
        let (id, typeset) = shape_box(&ed, 0);
        assert_eq!(ed.selection(), [id]);
        assert_same_box(ed.selection_bounds(), typeset);
    }

    #[test]
    fn selection_bounds_skips_only_the_block_being_edited() {
        // Arrange: two selected blocks.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");
        key(&mut ed, Key::Escape);
        place(&mut ed, 100.0, 200.0, "b");
        key(&mut ed, Key::Escape);
        ed.selection = ids(&ed);

        // Act: the Text tool re-opens the first.
        click(&mut ed, 105.0, 60.0);

        // Assert
        let (first, _) = shape_box(&ed, 0);
        let (_, second) = shape_box(&ed, 1);
        assert_eq!(ed.text_edit().and_then(TextEdit::target), Some(first));
        assert_same_box(ed.selection_bounds(), second);
    }

    #[test]
    fn selection_bounds_keeps_the_host_of_the_text_being_edited() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Select);
        click(&mut ed, 0.0, 50.0);

        // Act
        click(&mut ed, 100.0, 50.0);

        // Assert
        let (id, frame) = shape_box(&ed, 0);
        assert_eq!(editing_host(&ed), Some((id, TextSlot::Shape)));
        assert_same_box(ed.selection_bounds(), frame);
    }

    // ---- T78 formula ink outside its text box -------------------------------

    #[test]
    fn regression_t78_formula_ink_outside_box() {
        // Arrange: the seed's block, 419 characters of formula source with
        // no `$`, still open.
        let snapshot = Snapshot::parse(include_str!(
            "../../tests/snapshots/regressions/T78-formula-ink-outside-box.snap"
        ))
        .unwrap_or_else(|err| panic!("{err}"));
        let mut ed = Editor::from_snapshot(snapshot);

        // Act: commit it.
        key_with(&mut ed, Key::Enter, CTRL);

        // Assert: its ink is inside its box, with the fuzz invariant's slack.
        let (_, shape) = ed
            .document()
            .shapes()
            .next()
            .unwrap_or_else(|| panic!("the block was committed"));
        let Shape::Text {
            origin, text, size, ..
        } = shape
        else {
            panic!("a text block expected: {shape:?}");
        };
        assert_eq!(text.chars().count(), 419);
        let b = shape.bounds();
        let room = b.expand(1e-3 * size);
        for ink in &crate::core::text_layout::typeset(text).ink {
            let ib = ink.bounds();
            let (lo, hi) = (*origin + ib.min * *size, *origin + ib.max * *size);
            assert!(room.contains(lo) && room.contains(hi), "{ink:?} {b:?}");
        }
    }

    // ---- T76 remembered text size past the ladder ---------------------------

    /// The on-screen size remembered for new blocks.
    fn remembered_px(ed: &Editor) -> f32 {
        ed.tools
            .text
            .size_px
            .unwrap_or_else(|| panic!("a size is remembered"))
    }

    /// Zooms by `notches` of the wheel over the top-left of the window.
    fn zoom_by(ed: &mut Editor, notches: f32) {
        ed.handle(InputEvent::Scroll {
            pos: Vec2::new(10.0, 10.0),
            delta: notches,
        });
    }

    #[test]
    fn regression_t76_remembered_size_past_the_ladder() {
        // Arrange: `Ctrl+[` to 16 px at zoom 1, then a zoom to `ZOOM_MAX`
        // with the block still open.
        let snapshot = Snapshot::parse(include_str!(
            "../../tests/snapshots/regressions/T76-remembered-size-past-the-ladder.snap"
        ))
        .unwrap_or_else(|err| panic!("{err}"));
        let mut ed = Editor::from_snapshot(snapshot);
        assert!(approx_eq(ed.camera().zoom(), ZOOM_MAX, 1e-4));

        // Act
        key(&mut ed, Key::Escape);

        // Assert: at most the top rung, and readable again.
        let px = remembered_px(&ed);
        assert!(
            px <= text::text_size_px(MAX_WIDTH_PX) && px <= snapshot::MAX_TEXT_SIZE_PX,
            "{px}"
        );
        let text = ed.snapshot().to_text();
        let parsed = Snapshot::parse(&text).unwrap_or_else(|err| panic!("{err}\n{text}"));
        assert_eq!(parsed.to_text(), text);
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 1);
        assert!(approx_eq(sizes[0], 16.0, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_stays_on_the_ladder_at_min_zoom() {
        // Arrange: 20 px at zoom 1, then a zoom to `ZOOM_MIN`.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");
        assert!(key_with(&mut ed, Key::BracketRight, CTRL));
        zoom_by(&mut ed, -200.0);
        assert!(approx_eq(ed.camera().zoom(), ZOOM_MIN, 1e-4));

        // Act
        key(&mut ed, Key::Escape);

        // Assert: the lowest rung, and the block keeps its own size.
        let px = remembered_px(&ed);
        assert!(
            approx_eq(px, text::text_size_px(MIN_WIDTH_PX), 1e-4),
            "{px}"
        );
        let sizes = block_sizes(&ed);
        assert_eq!(sizes.len(), 1);
        assert!(approx_eq(sizes[0], 20.0, 1e-4), "{sizes:?}");
    }

    #[test]
    fn text_size_memory_inside_the_ladder_is_not_clamped() {
        // Arrange: 20 px at zoom 1, then one notch of zoom with the block open.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");
        assert!(key_with(&mut ed, Key::BracketRight, CTRL));
        zoom_by(&mut ed, 1.0);

        // Act
        key(&mut ed, Key::Escape);

        // Assert: the on-screen size when the edit ended, between rungs.
        let px = remembered_px(&ed);
        assert!(approx_eq(px, 20.0 * ZOOM_STEP, 1e-4), "{px}");
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
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Text);
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
        let mut ed = shape_text_editor(vec![boxed], Tool::Text);
        place_grown(&mut ed, 400.0, 300.0, "a", 2);

        // Act
        click(&mut ed, 100.0, 50.0);

        // Assert
        assert_eq!(editing(&ed).as_deref(), Some("ab"));
        assert!(approx_eq(open_size(&ed), 18.0, 1e-4));
    }

    // ---- T49 AC-4 revision under the byte budget ---------------------------

    #[test]
    fn editor_commit_dropping_old_entries_bumps_revision() {
        // Arrange: a budget of two and a half pen dots.
        let mut probe = Editor::new();
        click(&mut probe, 0.0, 0.0);
        let dot = probe.history_bytes();
        assert!(dot > 0);
        let mut ed = Editor::new();
        ed.history = History::with_limits(HISTORY_LIMIT, dot * 5 / 2);
        click(&mut ed, 0.0, 0.0);
        click(&mut ed, 10.0, 0.0);
        let lengths = (ed.history.undo_len(), ed.history.redo_len());
        assert_eq!(lengths, (2, 0));
        let before = ed.document_revision();

        // Act: the third dot drops the first one.
        click(&mut ed, 20.0, 0.0);

        // Assert
        assert_eq!((ed.history.undo_len(), ed.history.redo_len()), lengths);
        assert!(ed.document_revision() > before);
        assert_eq!(ed.document().len(), 3);
    }

    // ---- T83 paste text ----------------------------------------------------

    /// Sends clipboard text as the shell does.
    fn paste_text(ed: &mut Editor, text: &str) -> bool {
        ed.handle(InputEvent::Paste(text.to_owned()))
    }

    #[test]
    fn paste_event_in_open_edit_inserts_at_caret() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "ab");
        key(&mut ed, Key::ArrowLeft);

        // Act
        let changed = paste_text(&mut ed, "xy");
        typed(&mut ed, "!");

        // Assert: the caret ended after the pasted text.
        assert!(changed);
        assert_eq!(editing(&ed).as_deref(), Some("axy!b"));
    }

    #[test]
    fn paste_event_in_shape_text_goes_to_the_shape() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Text);
        click(&mut ed, 100.0, 50.0);

        // Act
        assert!(paste_text(&mut ed, "hi"));
        key(&mut ed, Key::Escape);

        // Assert
        assert_eq!(slot_text(&ed, 0, TextSlot::Shape).as_deref(), Some("hi"));
    }

    #[test]
    fn paste_event_in_grid_cell_keeps_line_breaks() {
        // Arrange
        let mut ed = shape_text_editor(vec![grid(0.0, 0.0, 200.0, 200.0, 4, 4)], Tool::Text);
        click(&mut ed, 25.0, 25.0);
        let cell = TextSlot::Cell { col: 0, row: 0 };
        assert_eq!(editing_host(&ed).map(|(_, slot)| slot), Some(cell));

        // Act
        assert!(paste_text(&mut ed, "1\r\n2"));

        // Assert: still in the same cell, unlike the Enter key.
        assert_eq!(editing_host(&ed).map(|(_, slot)| slot), Some(cell));
        assert_eq!(editing(&ed).as_deref(), Some("1\n2"));
        key(&mut ed, Key::Escape);
        assert_eq!(slot_text(&ed, 0, cell).as_deref(), Some("1\n2"));
    }

    #[test]
    fn paste_event_with_nothing_to_insert_changes_nothing() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "ab");

        // Act
        let empty = paste_text(&mut ed, "");
        let controls = paste_text(&mut ed, "\u{7}");

        // Assert
        assert!(!empty && !controls);
        assert_eq!(editing(&ed).as_deref(), Some("ab"));
    }

    #[test]
    fn paste_event_without_open_edit_is_ignored() {
        // Arrange
        let mut ed = text_editor();

        // Act
        let changed = paste_text(&mut ed, "text");

        // Assert
        assert!(!changed);
        assert!(ed.text_edit().is_none());
        assert!(ed.document().is_empty());
    }

    #[test]
    fn paste_event_then_ctrl_z_discards_the_new_block() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "ab");
        paste_text(&mut ed, "cd");

        // Act
        let cancelled = key_with(&mut ed, Key::Z, CTRL);

        // Assert
        assert!(cancelled);
        assert!(ed.text_edit().is_none());
        assert!(ed.document().is_empty());
        assert_eq!(ed.history.undo_len(), 0);
    }

    #[test]
    fn paste_event_then_ctrl_z_keeps_the_old_text() {
        // Arrange: a finished block, opened again.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "ab");
        key(&mut ed, Key::Escape);
        click(&mut ed, 104.0, 106.0);
        assert_eq!(editing(&ed).as_deref(), Some("ab"));
        paste_text(&mut ed, "cd");

        // Act
        key_with(&mut ed, Key::Z, CTRL);

        // Assert
        assert!(ed.text_edit().is_none());
        assert_eq!(texts(&ed), ["ab"]);
    }

    #[test]
    fn ctrl_v_outside_edit_still_pastes_shapes() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], Tool::Select);
        ed.apply(Command::SelectAll);
        ed.apply(Command::Copy);

        // Act
        key_with(&mut ed, Key::V, CTRL);

        // Assert
        assert_eq!(ed.document().len(), 2);
    }

    // ---- T84 grid from text ------------------------------------------------

    /// A size-18 text block at `(x, y)`.
    fn block(x: f32, y: f32, text: &str) -> Shape {
        Shape::Text {
            origin: Vec2::new(x, y),
            text: text.to_owned(),
            size: 18.0,
            style: Style {
                color: ColorId::INK,
                width: 0.0,
            },
        }
    }

    /// Editor holding one selected block at `(40, 60)`.
    fn selected_block(text: &str) -> Editor {
        let mut ed = shape_text_editor(vec![block(40.0, 60.0, text)], Tool::Select);
        ed.apply(Command::SelectAll);
        ed
    }

    fn ctrl_t(ed: &mut Editor) -> bool {
        key_with(ed, Key::T, CTRL)
    }

    fn paste_grid(ed: &mut Editor, text: &str) -> bool {
        ed.handle(InputEvent::PasteGrid(text.to_owned()))
    }

    /// The `index`-th shape (bottom first).
    fn shape_at(ed: &Editor, index: usize) -> &Shape {
        match ed.document().shapes().nth(index) {
            Some((_, shape)) => shape,
            None => panic!("no shape {index}"),
        }
    }

    /// `(corner, cols, rows)` of a grid.
    fn grid_layout(shape: &Shape) -> (Vec2, u32, u32) {
        match shape {
            Shape::Grid { a, cols, rows, .. } => (*a, *cols, *rows),
            other => panic!("expected a grid, got {other:?}"),
        }
    }

    /// `(col, row, text)` of every cell of `shape`.
    fn cell_strings(shape: &Shape) -> Vec<(u32, u32, String)> {
        shape
            .cell_texts()
            .iter()
            .map(|t| (t.col, t.row, t.content.text.clone()))
            .collect()
    }

    /// A line of `n` tokens.
    fn wide_line(n: u32) -> String {
        vec!["x"; n as usize].join(" ")
    }

    #[test]
    fn grid_from_text_replaces_the_block_at_its_top_left() {
        // Arrange
        let mut ed = selected_block("1 2 3\n4 5 6");

        // Act
        let changed = ctrl_t(&mut ed);

        // Assert
        assert!(changed);
        assert_eq!(ed.document().len(), 1);
        let (corner, cols, rows) = grid_layout(shape_at(&ed, 0));
        assert!(corner.approx_eq(Vec2::new(40.0, 60.0), 1e-4), "{corner:?}");
        assert_eq!((cols, rows), (3, 2));
        assert_eq!(shape_at(&ed, 0).cell_texts().len(), 6);
    }

    #[test]
    fn grid_from_text_is_one_undo_step() {
        // Arrange
        let mut ed = selected_block("1 2\n3 4");
        let original = shape_at(&ed, 0).clone();
        let steps = ed.history.undo_len();
        let revision = ed.document_revision();

        // Act
        ctrl_t(&mut ed);

        // Assert
        assert_eq!(ed.history.undo_len(), steps + 1);
        assert!(ed.document_revision() > revision);
        assert!(ed.apply(Command::Undo));
        assert_eq!(shape_at(&ed, 0), &original);
        assert!(ed.apply(Command::Redo));
        assert_eq!(grid_layout(shape_at(&ed, 0)).1, 2);
    }

    #[test]
    fn grid_from_text_keeps_colour_size_and_selection() {
        // Arrange
        let red = ColorId::from_key_digit(2).unwrap_or(ColorId::INK);
        let original = Shape::Text {
            origin: Vec2::new(40.0, 60.0),
            text: "1 2".to_owned(),
            size: 30.0,
            style: Style {
                color: red,
                width: 0.0,
            },
        };
        let mut ed = shape_text_editor(
            vec![
                rect(0.0, 0.0, 10.0, 10.0),
                original,
                rect(0.0, 0.0, 20.0, 20.0),
            ],
            Tool::Select,
        );
        let id = ids(&ed)[1];
        ed.selection = vec![id];

        // Act
        ctrl_t(&mut ed);

        // Assert: same id, same place in the z-order.
        assert_eq!(ed.selection(), [id]);
        assert_eq!(ids(&ed)[1], id);
        let shape = shape_at(&ed, 1);
        assert!(matches!(shape, Shape::Grid { style, .. } if style.color == red));
        assert!(
            shape
                .cell_texts()
                .iter()
                .all(|t| approx_eq(t.content.size, 30.0, 1e-4))
        );
    }

    #[test]
    fn grid_from_text_cancels_a_gesture() {
        // Arrange: a rectangle drag in progress with the block selected.
        let mut ed = selected_block("1 2");
        ed.apply(Command::SetTool(Tool::Rect));
        down(&mut ed, PointerButton::Left, 400.0, 400.0);
        move_to(&mut ed, 450.0, 450.0);
        assert!(ed.active_gesture().is_some());

        // Act
        ctrl_t(&mut ed);
        up(&mut ed, PointerButton::Left, 450.0, 450.0);

        // Assert
        assert!(ed.active_gesture().is_none());
        assert_eq!(ed.document().len(), 1);
        assert!(matches!(shape_at(&ed, 0), Shape::Grid { .. }));
    }

    #[test]
    fn grid_from_text_axes_follow_numbering() {
        // Arrange
        let mut off = selected_block("1 2");
        let mut on = selected_block("1 2");
        on.apply(Command::ToggleNumbering);

        // Act
        ctrl_t(&mut off);
        ctrl_t(&mut on);

        // Assert
        assert!(matches!(shape_at(&off, 0), Shape::Grid { axes: false, .. }));
        assert!(matches!(shape_at(&on, 0), Shape::Grid { axes: true, .. }));
    }

    #[test]
    fn grid_from_text_in_open_edit_only_types() {
        // Arrange: Ctrl+T while a block is open is not a binding.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "1 2");

        // Act
        ctrl_t(&mut ed);

        // Assert
        assert_eq!(editing(&ed).as_deref(), Some("1 2"));
        assert!(ed.document().is_empty());
    }

    #[test]
    fn grid_from_text_too_wide_changes_nothing_and_leaves_a_notice() {
        // Arrange
        let mut ed = selected_block(&wide_line(GRID_MAX_CELLS + 1));
        let before = ed.clone();
        let revision = ed.document_revision();

        // Act
        let changed = ctrl_t(&mut ed);

        // Assert
        assert!(!changed);
        assert_eq!(ed, before);
        assert_eq!(ed.document_revision(), revision);
        let notice = ed.take_notice().unwrap_or_default();
        assert!(notice.contains("65"), "{notice}");
        assert_eq!(ed.take_notice(), None, "the notice is handed over once");
    }

    #[test]
    fn grid_from_text_refused_keeps_a_gesture() {
        // Arrange
        let mut ed = selected_block(&wide_line(GRID_MAX_CELLS + 1));
        ed.apply(Command::SetTool(Tool::Rect));
        down(&mut ed, PointerButton::Left, 400.0, 400.0);

        // Act
        ctrl_t(&mut ed);

        // Assert
        assert!(ed.active_gesture().is_some());
    }

    #[test]
    fn grid_from_clipboard_too_tall_changes_nothing_and_leaves_a_notice() {
        // Arrange
        let mut ed = shape_text_editor(Vec::new(), Tool::Select);
        let tall = vec!["x"; GRID_MAX_CELLS as usize + 1].join("\n");
        let before = ed.clone();

        // Act
        let changed = paste_grid(&mut ed, &tall);

        // Assert
        assert!(!changed);
        assert_eq!(ed, before);
        assert!(ed.document().is_empty(), "nothing was cut to fit");
        assert!(ed.take_notice().is_some_and(|n| n.contains("65")));
    }

    #[test]
    fn take_notice_is_empty_after_a_success() {
        // Arrange
        let mut ed = selected_block("1 2");

        // Act
        ctrl_t(&mut ed);

        // Assert
        assert_eq!(ed.take_notice(), None);
    }

    #[test]
    fn grid_from_text_without_one_text_block_does_nothing() {
        // Arrange: nothing selected, a rectangle, and two blocks.
        let mut none = shape_text_editor(vec![block(0.0, 0.0, "1 2")], Tool::Select);
        let mut other = shape_text_editor(vec![rect(0.0, 0.0, 9.0, 9.0)], Tool::Select);
        other.apply(Command::SelectAll);
        let two = vec![block(0.0, 0.0, "1 2"), block(0.0, 90.0, "3 4")];
        let mut both = shape_text_editor(two, Tool::Select);
        both.apply(Command::SelectAll);

        for ed in [&mut none, &mut other, &mut both] {
            let before = ed.clone();

            // Act
            let changed = ctrl_t(ed);

            // Assert
            assert!(!changed);
            assert_eq!(*ed, before);
            assert_eq!(ed.take_notice(), None);
        }
    }

    #[test]
    fn grid_from_text_with_blank_text_does_nothing() {
        // Arrange
        let mut ed = selected_block("  \n ");
        let before = ed.clone();

        // Act
        let changed = ctrl_t(&mut ed);

        // Assert
        assert!(!changed);
        assert_eq!(ed, before);
        assert_eq!(ed.take_notice(), None);
    }

    #[test]
    fn grid_from_clipboard_puts_the_corner_at_the_cursor() {
        // Arrange: a camera that is not the identity.
        let mut ed = shape_text_editor(Vec::new(), Tool::Select);
        ed.camera = Camera::new(Vec2::new(100.0, 50.0), 2.0);
        move_to(&mut ed, 300.0, 200.0);
        let world = ed.camera().screen_to_world(Vec2::new(300.0, 200.0));

        // Act
        let changed = paste_grid(&mut ed, "1 2\n3 4");

        // Assert
        assert!(changed);
        let (corner, cols, rows) = grid_layout(shape_at(&ed, 0));
        assert!(corner.approx_eq(world, 1e-3), "{corner:?} vs {world:?}");
        assert_eq!((cols, rows), (2, 2));
    }

    #[test]
    fn grid_from_clipboard_has_the_cells_of_ctrl_t() {
        // Arrange
        let text = "1 22\n\n333";
        let mut from_block = selected_block(text);
        let mut from_paste = shape_text_editor(Vec::new(), Tool::Select);

        // Act
        ctrl_t(&mut from_block);
        paste_grid(&mut from_paste, text);

        // Assert
        let (block_shape, paste_shape) = (shape_at(&from_block, 0), shape_at(&from_paste, 0));
        assert_eq!(cell_strings(block_shape), cell_strings(paste_shape));
        let dims = |shape| {
            let (_, cols, rows) = grid_layout(shape);
            (cols, rows)
        };
        assert_eq!(dims(block_shape), dims(paste_shape));
    }

    #[test]
    fn grid_from_clipboard_uses_the_new_text_size_on_screen() {
        // Arrange: at zoom 2 a new block is half as big in the world.
        let mut ed = shape_text_editor(Vec::new(), Tool::Select);
        ed.camera = Camera::new(Vec2::ZERO, 2.0);
        let expected = ed
            .camera()
            .world_len(text::text_size_px(ed.style().width_px));

        // Act
        paste_grid(&mut ed, "1");

        // Assert
        let size = shape_at(&ed, 0).cell_texts()[0].content.size;
        assert!(approx_eq(size, expected, 1e-4), "{size} vs {expected}");
    }

    #[test]
    fn grid_from_clipboard_is_one_undo_step_and_selected() {
        // Arrange: another shape stays below and leaves the selection.
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 9.0, 9.0)], Tool::Select);
        ed.apply(Command::SelectAll);
        let steps = ed.history.undo_len();

        // Act
        paste_grid(&mut ed, "1 2");

        // Assert
        assert_eq!(ed.history.undo_len(), steps + 1);
        assert_eq!(ed.document().len(), 2);
        assert_eq!(ed.selection(), [ids(&ed)[1]]);
        assert!(ed.apply(Command::Undo));
        assert_eq!(ed.document().len(), 1);
        assert_eq!(ed.selection(), []);
    }

    #[test]
    fn grid_from_clipboard_only_appends_to_the_document() {
        // Arrange
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 9.0, 9.0)], Tool::Select);
        let base = ed.document_base_revision();

        // Act
        paste_grid(&mut ed, "1 2");

        // Assert
        assert_eq!(ed.document_base_revision(), base);
    }

    #[test]
    fn grid_from_clipboard_in_open_edit_is_ignored() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "ab");

        // Act
        let changed = paste_grid(&mut ed, "1 2");

        // Assert
        assert!(!changed);
        assert_eq!(editing(&ed).as_deref(), Some("ab"));
        assert!(ed.document().is_empty());
    }

    #[test]
    fn grid_from_clipboard_blank_does_nothing() {
        // Arrange
        let mut ed = shape_text_editor(Vec::new(), Tool::Select);
        let before = ed.clone();

        // Act
        let changed = paste_grid(&mut ed, " \n\t ");

        // Assert
        assert!(!changed);
        assert_eq!(ed, before);
        assert_eq!(ed.history.undo_len(), 0);
        assert_eq!(ed.take_notice(), None);
    }

    // ---- T87 graph from an edge list --------------------------------------

    fn ctrl_l(ed: &mut Editor) -> bool {
        key_with(ed, Key::L, CTRL)
    }

    fn count(ed: &Editor, wanted: fn(&Shape) -> bool) -> usize {
        ed.document().shapes().filter(|(_, s)| wanted(s)).count()
    }

    fn is_ellipse(shape: &Shape) -> bool {
        matches!(shape, Shape::Ellipse { .. })
    }

    fn is_line(shape: &Shape) -> bool {
        matches!(shape, Shape::Line { .. })
    }

    fn is_text(shape: &Shape) -> bool {
        matches!(shape, Shape::Text { .. })
    }

    #[test]
    fn graph_from_text_replaces_the_block() {
        // Arrange
        let mut ed = selected_block("1-2");

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert
        assert!(changed);
        assert_eq!(ed.document().len(), 3);
        assert_eq!(count(&ed, is_line), 1);
        assert_eq!(count(&ed, is_ellipse), 2);
        assert_eq!(count(&ed, is_text), 0);
    }

    #[test]
    fn graph_from_text_keeps_the_block_place_and_colour() {
        // Arrange
        let red = ColorId::from_key_digit(2).unwrap_or(ColorId::INK);
        let original = Shape::Text {
            origin: Vec2::new(40.0, 60.0),
            text: "1-2".to_owned(),
            size: 18.0,
            style: Style {
                color: red,
                width: 0.0,
            },
        };
        let mut ed = shape_text_editor(
            vec![
                rect(0.0, 0.0, 10.0, 10.0),
                original,
                rect(0.0, 0.0, 20.0, 20.0),
            ],
            Tool::Select,
        );
        let (bottom, top) = (ids(&ed)[0], ids(&ed)[2]);
        ed.selection = vec![ids(&ed)[1]];

        // Act
        ctrl_l(&mut ed);

        // Assert: between the two rectangles, in the block's colour.
        let now = ids(&ed);
        assert_eq!(now.len(), 5);
        assert_eq!((now[0], now[4]), (bottom, top));
        for index in 1..4 {
            let style = shape_at(&ed, index).style();
            assert_eq!(style.color, red);
            assert!(approx_eq(style.width, ed.style().width_px, 1e-4));
        }
    }

    #[test]
    fn graph_from_text_is_one_undo_step() {
        // Arrange
        let mut ed = selected_block("1-2 2-3");
        let original = shape_at(&ed, 0).clone();
        let steps = ed.history.undo_len();
        let revision = ed.document_revision();

        // Act
        ctrl_l(&mut ed);

        // Assert
        assert_eq!(ed.history.undo_len(), steps + 1);
        assert!(ed.document_revision() > revision);
        assert_eq!(ed.document().len(), 5);
        assert!(ed.apply(Command::Undo));
        assert_eq!(ed.document().len(), 1);
        assert_eq!(shape_at(&ed, 0), &original);
        assert!(ed.apply(Command::Redo));
        assert_eq!(ed.document().len(), 5);
        assert_eq!(count(&ed, is_text), 0);
    }

    #[test]
    fn graph_from_text_reads_math_names_and_weights() {
        // Arrange
        let mut ed = selected_block("$s$ $a$ $w$\n$a$ $t$");
        let steps = ed.history.undo_len();

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert
        assert!(changed);
        assert_eq!(ed.history.undo_len(), steps + 1);
        assert_eq!(count(&ed, is_ellipse), 3);
        assert_eq!(count(&ed, is_line), 2);
        let weights: Vec<&str> = ed
            .document()
            .shapes()
            .filter_map(|(_, s)| match s {
                Shape::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(weights, ["$w$"]);
        assert!(ed.apply(Command::Undo));
        assert_eq!(ed.document().len(), 1);
    }

    #[test]
    fn graph_from_text_reads_weights_after_pairs() {
        // Arrange
        let mut ed = selected_block("1>3 7\n3 - 2 $w$");

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert
        assert!(changed);
        assert_eq!(count(&ed, is_ellipse), 3);
        assert_eq!(count(&ed, |s| matches!(s, Shape::Arrow { .. })), 1);
        assert_eq!(count(&ed, is_line), 1);
        let weights: Vec<&str> = ed
            .document()
            .shapes()
            .filter_map(|(_, s)| match s {
                Shape::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(weights, ["7", "$w$"]);
    }

    #[test]
    fn graph_from_text_reads_a_parent_list() {
        // Arrange
        let mut ed = selected_block("parents:\n4\n1 1 2");
        let original = shape_at(&ed, 0).clone();
        let steps = ed.history.undo_len();

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert: four nodes, three edges, one undo step.
        assert!(changed);
        assert_eq!(count(&ed, is_ellipse), 4);
        assert_eq!(count(&ed, is_line), 3);
        assert_eq!(ed.history.undo_len(), steps + 1);
        assert!(ed.apply(Command::Undo));
        assert_eq!(shape_at(&ed, 0), &original);
    }

    #[test]
    fn graph_from_text_reads_a_directed_list() {
        // Arrange
        let mut ed = selected_block("dgraph:\n3 2\n1 2\n2 3");

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert
        assert!(changed);
        assert_eq!(count(&ed, is_ellipse), 3);
        assert_eq!(count(&ed, |s| matches!(s, Shape::Arrow { .. })), 2);
        assert_eq!(count(&ed, is_line), 0);
    }
    #[test]
    fn graph_from_text_selects_the_result() {
        // Arrange
        let mut ed = selected_block("1-2");

        // Act
        ctrl_l(&mut ed);

        // Assert
        assert_eq!(ed.selection(), ids(&ed));
    }

    #[test]
    fn graph_from_text_cancels_a_gesture() {
        // Arrange: a rectangle drag in progress with the block selected.
        let mut ed = selected_block("1-2");
        ed.apply(Command::SetTool(Tool::Rect));
        down(&mut ed, PointerButton::Left, 400.0, 400.0);
        move_to(&mut ed, 450.0, 450.0);
        assert!(ed.active_gesture().is_some());

        // Act
        ctrl_l(&mut ed);
        up(&mut ed, PointerButton::Left, 450.0, 450.0);

        // Assert
        assert!(ed.active_gesture().is_none());
        assert_eq!(ed.document().len(), 3);
    }

    #[test]
    fn graph_from_text_in_open_edit_only_types() {
        // Arrange: Ctrl+L while a block is open is not a binding.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "1-2");

        // Act
        ctrl_l(&mut ed);

        // Assert
        assert_eq!(editing(&ed).as_deref(), Some("1-2"));
        assert!(ed.document().is_empty());
    }

    #[test]
    fn graph_from_text_shapes_are_ordinary() {
        // Arrange
        let mut ed = selected_block("1-2 2>3");
        ctrl_l(&mut ed);
        let len = ed.document().len();

        // Act / Assert: selection commands treat them like any shape.
        ed.apply(Command::Duplicate);
        assert_eq!(ed.document().len(), 2 * len);
        ed.apply(Command::DeleteSelection);
        assert_eq!(ed.document().len(), len);

        // Act: the Select tool drags one circle by its outline.
        ed.apply(Command::Cancel);
        let Some((id, circle)) = ed
            .document()
            .shapes()
            .find(|(_, s)| is_ellipse(s))
            .map(|(id, s)| (id, s.bounds()))
        else {
            panic!("no circle");
        };
        let grip = ed
            .camera()
            .world_to_screen(Vec2::new(circle.center().x, circle.min.y));
        down(&mut ed, PointerButton::Left, grip.x, grip.y);
        move_to(&mut ed, grip.x + 30.0, grip.y);
        up(&mut ed, PointerButton::Left, grip.x + 30.0, grip.y);

        // Assert
        assert_eq!(ed.selection(), [id]);
        let moved = ed.document().get(id).map(Shape::bounds);
        assert!(moved.is_some_and(|b| b.center().x > circle.center().x + 15.0));
    }

    #[test]
    fn graph_from_text_survives_a_snapshot() {
        // Arrange: node 0 included, a weight, an arrow.
        let mut ed = selected_block("0 1 5\n1>2");
        ctrl_l(&mut ed);

        // Act
        let loaded = Snapshot::parse(&ed.snapshot().to_text());

        // Assert
        match loaded {
            Ok(snapshot) => assert_eq!(Editor::from_snapshot(snapshot), ed),
            Err(err) => panic!("{err}"),
        }
    }

    #[test]
    fn graph_from_text_leaves_the_counter_alone() {
        // Arrange
        let mut ed = selected_block("5-6");
        ed.style.helpers.next_number = 3;

        // Act
        ctrl_l(&mut ed);

        // Assert
        assert_eq!(ed.helpers().next_number, 3);
    }

    #[test]
    fn graph_from_text_bad_text_changes_nothing_and_leaves_a_notice() {
        // Arrange: a drag in progress must survive a refused key.
        let mut ed = selected_block("1-2\nhello world");
        ed.apply(Command::SetTool(Tool::Rect));
        down(&mut ed, PointerButton::Left, 400.0, 400.0);
        move_to(&mut ed, 450.0, 450.0);
        let before = ed.clone();

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert
        assert!(!changed);
        assert_eq!(ed, before);
        let notice = ed.take_notice().unwrap_or_default();
        assert!(notice.contains("line 2"), "{notice}");
        assert!(!notice.contains("hello"), "{notice}");
        assert_eq!(ed.take_notice(), None, "taken once");
    }

    #[test]
    fn graph_from_text_too_many_nodes_changes_nothing_and_leaves_a_notice() {
        // Arrange
        let edges: Vec<String> = (1..=200).map(|i| format!("{i}-{}", i + 1)).collect();
        let mut ed = selected_block(&edges.join("\n"));
        let before = ed.clone();

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert
        assert!(!changed);
        assert_eq!(ed, before);
        let notice = ed.take_notice().unwrap_or_default();
        assert!(notice.contains("201"), "{notice}");
    }

    #[test]
    fn graph_from_text_without_one_text_block_does_nothing() {
        // Arrange: nothing selected, two blocks, a rectangle, blank text.
        let mut none = selected_block("1-2");
        none.selection.clear();
        let mut two = shape_text_editor(
            vec![block(0.0, 0.0, "1-2"), block(90.0, 0.0, "3-4")],
            Tool::Select,
        );
        two.apply(Command::SelectAll);
        let mut other = shape_text_editor(vec![rect(0.0, 0.0, 9.0, 9.0)], Tool::Select);
        other.apply(Command::SelectAll);
        let blank = selected_block(" \n ");

        for mut ed in [none, two, other, blank] {
            let before = ed.clone();

            // Act
            let changed = ctrl_l(&mut ed);

            // Assert
            assert!(!changed);
            assert_eq!(ed, before);
            assert_eq!(ed.take_notice(), None);
        }
    }

    #[test]
    fn graph_from_text_only_self_loops_draws_the_nodes_and_their_loops() {
        // Arrange
        let mut ed = selected_block("1-1 2-2");

        // Act
        ctrl_l(&mut ed);

        // Assert
        // The loops are kept since ADR-T121-1: a stroke each.
        assert_eq!(count(&ed, is_ellipse), 2);
        assert_eq!(ed.document().len(), 4);
    }

    // ---- T104 segment tree -------------------------------------------------

    /// The texts inside every circle, in document order.
    fn circle_texts(ed: &Editor) -> Vec<String> {
        ed.document()
            .shapes()
            .filter(|(_, s)| is_ellipse(s) || matches!(s, Shape::Rect { .. }))
            .filter_map(|(_, s)| s.slot_text(TextSlot::Shape).map(|t| t.text.clone()))
            .collect()
    }

    #[test]
    fn segtree_from_text_starts_at_one() {
        // Arrange
        let mut ed = selected_block("segtree: 8");

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert
        assert!(changed);
        assert_eq!(count(&ed, is_ellipse), 0);
        assert_eq!(count(&ed, |s| matches!(s, Shape::Rect { .. })), 30);
        assert_eq!(count(&ed, is_line), 14);
        let texts = circle_texts(&ed);
        assert_eq!(texts.len(), 15);
        assert_eq!(texts[0], "$[1, 8]$");
        assert!(texts.iter().any(|t| t == "$[8, 8]$"));
    }

    #[test]
    fn segtree_from_text_starts_at_zero_after_ctrl_n() {
        // Arrange
        let mut ed = selected_block("segtree:\n8");
        ed.apply(Command::ToggleZeroBased);

        // Act
        let changed = ctrl_l(&mut ed);

        // Assert
        assert!(changed);
        let texts = circle_texts(&ed);
        assert_eq!(texts.len(), 15);
        assert_eq!(texts[0], "$[0, 7]$");
        assert!(texts.iter().any(|t| t == "$[0, 0]$"));
        assert!(!texts.iter().any(|t| t.contains('8')));
    }

    #[test]
    fn segtree_from_text_is_one_undo_step_and_selected() {
        // Arrange
        let mut ed = selected_block("segtree: 4");
        let original = shape_at(&ed, 0).clone();
        let steps = ed.history.undo_len();

        // Act
        ctrl_l(&mut ed);

        // Assert
        assert_eq!(ed.history.undo_len(), steps + 1);
        assert_eq!(ed.document().len(), 2 * 7 + 6);
        assert_eq!(ed.selection(), ids(&ed));
        assert!(ed.apply(Command::Undo));
        assert_eq!(ed.document().len(), 1);
        assert_eq!(shape_at(&ed, 0), &original);
    }

    #[test]
    fn segtree_from_text_bad_size_changes_nothing_and_leaves_a_notice() {
        for (text, line) in [
            ("segtree: 101", "line 1"),
            ("segtree: 0", "line 1"),
            ("segtree:\n4\nx", "line 3"),
        ] {
            // Arrange
            let mut ed = selected_block(text);
            let before = ed.clone();

            // Act
            let changed = ctrl_l(&mut ed);

            // Assert
            assert!(!changed);
            assert_eq!(ed, before);
            let notice = ed.take_notice().unwrap_or_default();
            assert!(notice.contains(line), "{notice}");
            assert!(!notice.contains("segtree"), "{notice}");
        }
    }

    // ---- T88 bucket drag across cells --------------------------------------

    /// Editor on the Bucket holding one 4 × 4 grid of 25 px cells, and its id.
    fn bucket_on_grid() -> (Editor, ShapeId) {
        let mut ed = editor_with(vec![grid(0.0, 0.0, 100.0, 100.0, 4, 4)]);
        ed.apply(Command::SetTool(Tool::Bucket));
        let id = ids(&ed)[0];
        (ed, id)
    }

    fn filled_cells(ed: &Editor, id: ShapeId) -> Vec<(u32, u32)> {
        ed.document().get(id).map_or_else(Vec::new, |shape| {
            shape.cell_fills().iter().map(|f| (f.col, f.row)).collect()
        })
    }

    #[test]
    fn bucket_escape_during_drag_restores_the_cells() {
        // Arrange
        let (mut ed, id) = bucket_on_grid();
        down(&mut ed, PointerButton::Left, 10.0, 10.0);
        move_to(&mut ed, 90.0, 10.0);
        assert!(!ed.overlay().is_empty());

        // Act
        let redraw = key(&mut ed, Key::Escape);
        up(&mut ed, PointerButton::Left, 90.0, 10.0);

        // Assert
        assert!(redraw);
        assert!(ed.overlay().is_empty());
        assert_eq!(filled_cells(&ed, id), []);
        assert_eq!(ed.history.undo_len(), 1);
    }

    #[test]
    fn bucket_drag_moves_keep_the_document_revision() {
        // Arrange
        let (mut ed, id) = bucket_on_grid();
        let before = ed.document_revision();
        down(&mut ed, PointerButton::Left, 10.0, 10.0);

        // Act
        let redraw = move_to(&mut ed, 40.0, 10.0);

        // Assert: shown at once, committed on release only.
        assert!(redraw);
        assert_eq!(ed.document_revision(), before);
        up(&mut ed, PointerButton::Left, 40.0, 10.0);
        assert!(ed.document_revision() > before);
        assert_eq!(filled_cells(&ed, id), [(0, 0), (1, 0)]);
    }

    #[test]
    fn bucket_pen_tap_fills_the_pressed_cell_only() {
        // Arrange: the release is in the next cell, within the tap slop.
        let (ed, id) = bucket_on_grid();
        let mut ed = ed.with_tap_slop(16.0);

        // Act
        down(&mut ed, PointerButton::Left, 22.0, 10.0);
        move_to(&mut ed, 28.0, 10.0);
        up(&mut ed, PointerButton::Left, 28.0, 10.0);

        // Assert
        assert_eq!(filled_cells(&ed, id), [(0, 0)]);
        assert_eq!(ed.history.undo_len(), 2);
    }

    #[test]
    fn bucket_drag_survives_a_snapshot_round_trip() {
        // Arrange: a drag that has painted two cells.
        let (mut ed, id) = bucket_on_grid();
        down(&mut ed, PointerButton::Left, 10.0, 10.0);
        move_to(&mut ed, 40.0, 10.0);

        // Act
        let text = ed.snapshot().to_text();
        let mut loaded = match Snapshot::parse(&text) {
            Ok(snapshot) => Editor::from_snapshot(snapshot),
            Err(err) => panic!("{err}\n{text}"),
        };

        // Assert: equal now, and after the drag ends the same way.
        assert_eq!(loaded, ed);
        for editor in [&mut ed, &mut loaded] {
            move_to(editor, 65.0, 10.0);
            up(editor, PointerButton::Left, 65.0, 10.0);
        }
        assert_eq!(loaded, ed);
        assert_eq!(filled_cells(&loaded, id), [(0, 0), (1, 0), (2, 0)]);
    }

    // ---- T91 resize handles -----------------------------------------------

    const ALT: Modifiers = Modifiers {
        shift: false,
        ctrl: false,
        alt: true,
    };

    /// The `(a, b)` corners of the only rectangle.
    fn rect_box(ed: &Editor) -> (Vec2, Vec2) {
        match ed.document().shapes().next() {
            Some((_, Shape::Rect { a, b, .. })) => (*a, *b),
            other => panic!("expected a rectangle, got {other:?}"),
        }
    }

    /// An editor in `tool` with one selected 100 × 100 square at the origin.
    fn selected_square(tool: Tool) -> Editor {
        let mut ed = shape_text_editor(vec![rect(0.0, 0.0, 100.0, 100.0)], tool);
        ed.selection = ids(&ed);
        ed
    }

    #[test]
    fn alt_brackets_scale_the_selection() {
        // Arrange
        let mut ed = selected_square(Tool::Pen);
        let revision = ed.document_revision();
        let undo_len = ed.history.undo_len();

        // Act: any tool can use the keys.
        let changed = ed.handle(InputEvent::KeyDown {
            key: Key::BracketRight,
            mods: ALT,
        });

        // Assert
        assert!(changed);
        let (a, b) = rect_box(&ed);
        assert!(a.approx_eq(Vec2::new(-5.0, -5.0), 1e-3));
        assert!(b.approx_eq(Vec2::new(105.0, 105.0), 1e-3));
        assert_eq!(ed.history.undo_len(), undo_len + 1);
        assert!(ed.document_revision() > revision);
        assert_eq!(ed.document_base_revision(), ed.document_revision());

        // Act: the other key, then undoing it.
        ed.handle(InputEvent::KeyDown {
            key: Key::BracketLeft,
            mods: ALT,
        });
        let (a, _) = rect_box(&ed);
        assert!(a.approx_eq(Vec2::new(0.5, 0.5), 1e-3));
        assert!(ed.apply(Command::Undo));
        let (a, _) = rect_box(&ed);
        assert!(a.approx_eq(Vec2::new(-5.0, -5.0), 1e-3));
    }

    #[test]
    fn plain_brackets_still_change_the_stroke_width() {
        // Arrange
        let mut ed = selected_square(Tool::Select);
        let before = rect_box(&ed);
        let width = ed.style().width_px;

        // Act
        key(&mut ed, Key::BracketRight);

        // Assert
        assert!(ed.style().width_px > width);
        assert_eq!(rect_box(&ed), before);
    }

    #[test]
    fn selection_handles_show_only_when_idle_in_select() {
        // Arrange: another tool, then the Select tool.
        let mut ed = selected_square(Tool::Pen);
        assert!(ed.selection_handles().is_none());
        ed.apply(Command::SetTool(Tool::Select));
        assert!(ed.selection_handles().is_some());

        let Some(handles) = ed.selection_handles() else {
            panic!("no handles");
        };
        let at = handles[2];

        // Act / Assert: hidden during a gesture.
        down(&mut ed, PointerButton::Left, at.x, at.y);
        assert!(ed.selection_handles().is_none());
        up(&mut ed, PointerButton::Left, at.x, at.y);
        assert!(ed.selection_handles().is_some());

        // Act / Assert: hidden while the shape's text is open.
        down(&mut ed, PointerButton::Left, 50.0, 50.0);
        up(&mut ed, PointerButton::Left, 50.0, 50.0);
        assert!(ed.text_edit().is_some());
        assert!(ed.selection_handles().is_none());
    }

    #[test]
    fn resize_commits_through_the_editor() {
        // Arrange
        let mut ed = selected_square(Tool::Select);
        let Some(handles) = ed.selection_handles() else {
            panic!("no handles");
        };
        let at = handles[2];
        let revision = ed.document_revision();

        // Act
        down(&mut ed, PointerButton::Left, at.x, at.y);
        move_to(&mut ed, at.x + 100.0, at.y);
        assert_eq!(ed.document_revision(), revision);
        up(&mut ed, PointerButton::Left, at.x + 100.0, at.y);

        // Assert
        let (a, b) = rect_box(&ed);
        assert!(a.approx_eq(Vec2::ZERO, 1e-3));
        assert!(b.approx_eq(Vec2::new(200.0, 100.0), 1e-3));
        assert!(ed.document_revision() > revision);
        assert_eq!(ed.document_base_revision(), ed.document_revision());
    }

    #[test]
    fn regression_t91_scaled_box_side_overflow() {
        // Arrange: a selected grid so tall that 10 % more leaves both of
        // its corners finite but the length of its side past `f32`.
        let snapshot = Snapshot::parse(include_str!(
            "../../tests/snapshots/regressions/T91-scaled-box-side-overflow.snap"
        ))
        .unwrap_or_else(|err| panic!("{err}"));
        let mut ed = Editor::from_snapshot(snapshot);
        let before = ed.document().shapes().next().map(|(_, s)| s.clone());
        let undo_len = ed.history.undo_len();
        assert_eq!(ed.selection().len(), 1);

        // Act
        let changed = ed.handle(InputEvent::KeyDown {
            key: Key::BracketRight,
            mods: ALT,
        });

        // Assert: the scale is ignored.
        assert!(!changed);
        assert_eq!(
            ed.document().shapes().next().map(|(_, s)| s.clone()),
            before
        );
        assert_eq!(ed.history.undo_len(), undo_len);
    }

    #[test]
    fn resize_gesture_survives_a_snapshot() {
        // Arrange: stopped in the middle of a resize.
        let mut ed = selected_square(Tool::Select);
        let Some(handles) = ed.selection_handles() else {
            panic!("no handles");
        };
        let at = handles[2];
        down(&mut ed, PointerButton::Left, at.x, at.y);
        move_to(&mut ed, at.x + 30.0, at.y + 10.0);

        // Act
        let text = ed.snapshot().to_text();
        let restored = match crate::core::snapshot::Snapshot::parse(&text) {
            Ok(snapshot) => Editor::from_snapshot(snapshot),
            Err(error) => panic!("{error}"),
        };

        // Assert
        assert_eq!(restored.snapshot().to_text(), text);
        assert_eq!(restored.overlay(), ed.overlay());
        assert!(text.contains("select resize"));
    }

    // ---- T93 coordinate plane from points ---------------------------------

    fn ctrl_p(ed: &mut Editor) -> bool {
        key_with(ed, Key::P, CTRL)
    }

    /// Two points; the layout's own tests count what they draw.
    const PLOT_TEXT: &str = "0 0\n2 1";

    #[test]
    fn plot_from_text_replaces_the_block() {
        // Arrange
        let mut ed = selected_block(PLOT_TEXT);

        // Act
        let changed = ctrl_p(&mut ed);

        // Assert
        assert!(changed);
        assert_eq!(count(&ed, is_ellipse), 2);
        assert!(count(&ed, is_line) >= 2 + 4);
        assert!(count(&ed, is_text) >= 3);
        assert!(
            ed.document()
                .shapes()
                .all(|(_, s)| !matches!(s, Shape::Text { text, .. } if text == PLOT_TEXT)),
            "the block is gone"
        );
    }

    #[test]
    fn plot_from_text_keeps_the_block_place_and_colour() {
        // Arrange
        let red = ColorId::from_key_digit(2).unwrap_or(ColorId::INK);
        let original = Shape::Text {
            origin: Vec2::new(40.0, 60.0),
            text: PLOT_TEXT.to_owned(),
            size: 18.0,
            style: Style {
                color: red,
                width: 0.0,
            },
        };
        let mut ed = shape_text_editor(
            vec![
                rect(0.0, 0.0, 10.0, 10.0),
                original,
                rect(0.0, 0.0, 20.0, 20.0),
            ],
            Tool::Select,
        );
        let (bottom, top) = (ids(&ed)[0], ids(&ed)[2]);
        ed.selection = vec![ids(&ed)[1]];

        // Act
        ctrl_p(&mut ed);

        // Assert: between the two rectangles, in the block's colour.
        let now = ids(&ed);
        let last = now.len() - 1;
        assert_eq!((now[0], now[last]), (bottom, top));
        for index in 1..last {
            assert_eq!(shape_at(&ed, index).style().color, red);
        }
        let dot = shape_at(&ed, last - 1);
        assert_eq!(dot.fill(), Some(red));
        assert!(approx_eq(dot.style().width, ed.style().width_px, 1e-4));
    }

    #[test]
    fn plot_from_text_is_one_undo_step() {
        // Arrange
        let mut ed = selected_block(PLOT_TEXT);
        let original = shape_at(&ed, 0).clone();
        let steps = ed.history.undo_len();
        let revision = ed.document_revision();

        // Act
        ctrl_p(&mut ed);
        let len = ed.document().len();

        // Assert
        assert_eq!(ed.history.undo_len(), steps + 1);
        assert!(ed.document_revision() > revision);
        assert!(ed.apply(Command::Undo));
        assert_eq!(ed.document().len(), 1);
        assert_eq!(shape_at(&ed, 0), &original);
        assert!(ed.apply(Command::Redo));
        assert_eq!(ed.document().len(), len);
    }

    #[test]
    fn plot_from_text_keeps_the_labels_of_the_points() {
        // Arrange
        let mut ed = selected_block("0 0 A\n2 1 top right");

        // Act
        let changed = ctrl_p(&mut ed);

        // Assert
        assert!(changed);
        assert_eq!(count(&ed, is_ellipse), 2);
        let texts = texts(&ed);
        assert!(texts.iter().any(|t| t == "A"), "{texts:?}");
        assert!(texts.iter().any(|t| t == "top right"), "{texts:?}");
    }

    #[test]
    fn plot_from_text_undoes_a_labelled_plot_in_one_step() {
        // Arrange
        let mut ed = selected_block("0 0 A\n2 1");
        let original = shape_at(&ed, 0).clone();
        let steps = ed.history.undo_len();

        // Act
        ctrl_p(&mut ed);

        // Assert
        assert_eq!(ed.history.undo_len(), steps + 1);
        assert!(ed.apply(Command::Undo));
        assert_eq!(ed.document().len(), 1);
        assert_eq!(shape_at(&ed, 0), &original);
    }

    #[test]
    fn plot_from_text_selects_the_result() {
        // Arrange
        let mut ed = selected_block(PLOT_TEXT);

        // Act
        ctrl_p(&mut ed);

        // Assert
        assert_eq!(ed.selection(), ids(&ed));
    }

    #[test]
    fn plot_from_text_cancels_a_gesture() {
        // Arrange: a rectangle drag in progress with the block selected.
        let mut ed = selected_block(PLOT_TEXT);
        ed.apply(Command::SetTool(Tool::Rect));
        down(&mut ed, PointerButton::Left, 400.0, 400.0);
        move_to(&mut ed, 450.0, 450.0);
        assert!(ed.active_gesture().is_some());

        // Act
        ctrl_p(&mut ed);
        up(&mut ed, PointerButton::Left, 450.0, 450.0);

        // Assert
        assert!(ed.active_gesture().is_none());
        assert_eq!(count(&ed, is_ellipse), 2);
    }

    #[test]
    fn plot_from_text_in_open_edit_only_types() {
        // Arrange: Ctrl+P while a block is open is not a binding.
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "1 2");

        // Act
        ctrl_p(&mut ed);

        // Assert
        assert_eq!(editing(&ed).as_deref(), Some("1 2"));
        assert!(ed.document().is_empty());
    }

    #[test]
    fn plot_from_text_shapes_are_ordinary() {
        // Arrange
        let mut ed = selected_block(PLOT_TEXT);
        ctrl_p(&mut ed);
        let len = ed.document().len();

        // Act / Assert: selection commands treat them like any shape.
        ed.apply(Command::Duplicate);
        assert_eq!(ed.document().len(), 2 * len);
        ed.apply(Command::DeleteSelection);
        assert_eq!(ed.document().len(), len);
    }

    #[test]
    fn plot_from_text_survives_a_snapshot() {
        // Arrange
        let mut ed = selected_block("-1.5 2\n3 -4.25");
        ctrl_p(&mut ed);

        // Act
        let loaded = Snapshot::parse(&ed.snapshot().to_text());

        // Assert
        match loaded {
            Ok(snapshot) => assert_eq!(Editor::from_snapshot(snapshot), ed),
            Err(err) => panic!("{err}"),
        }
    }

    #[test]
    fn plot_from_text_bad_text_changes_nothing_and_leaves_a_notice() {
        for (text, wanted) in [("1 2\nhello world", "line 2"), ("1 2\n3 inf", "line 2")] {
            // Arrange: a drag in progress must survive a refused key.
            let mut ed = selected_block(text);
            ed.apply(Command::SetTool(Tool::Rect));
            down(&mut ed, PointerButton::Left, 400.0, 400.0);
            move_to(&mut ed, 450.0, 450.0);
            let before = ed.clone();

            // Act
            let changed = ctrl_p(&mut ed);

            // Assert
            assert!(!changed);
            assert_eq!(ed, before);
            let notice = ed.take_notice().unwrap_or_default();
            assert!(notice.contains(wanted), "{notice}");
            assert!(!notice.contains("hello"), "{notice}");
            assert_eq!(ed.take_notice(), None, "taken once");
        }
    }

    #[test]
    fn plot_from_text_too_many_points_changes_nothing_and_leaves_a_notice() {
        // Arrange
        let lines: Vec<String> = (0..=1000).map(|i| format!("{i} {i}")).collect();
        let mut ed = selected_block(&lines.join("\n"));
        let before = ed.clone();

        // Act
        let changed = ctrl_p(&mut ed);

        // Assert
        assert!(!changed);
        assert_eq!(ed, before);
        let notice = ed.take_notice().unwrap_or_default();
        assert!(notice.contains("1001"), "{notice}");
    }

    #[test]
    fn plot_from_text_without_one_text_block_does_nothing() {
        // Arrange: nothing selected, two blocks, a rectangle, blank text.
        let mut none = selected_block("1 2");
        none.selection.clear();
        let mut two = shape_text_editor(
            vec![block(0.0, 0.0, "1 2"), block(90.0, 0.0, "3 4")],
            Tool::Select,
        );
        two.apply(Command::SelectAll);
        let mut other = shape_text_editor(vec![rect(0.0, 0.0, 9.0, 9.0)], Tool::Select);
        other.apply(Command::SelectAll);
        let blank = selected_block(" \n ");

        for mut ed in [none, two, other, blank] {
            let before = ed.clone();

            // Act
            let changed = ctrl_p(&mut ed);

            // Assert
            assert!(!changed);
            assert_eq!(ed, before);
            assert_eq!(ed.take_notice(), None);
        }
    }

    // ---- T100 z-order keys -------------------------------------------------

    const CTRL_SHIFT: Modifiers = Modifiers {
        shift: true,
        ctrl: true,
        alt: false,
    };

    /// Editor holding `shapes` with the Select tool, and their ids bottom
    /// first.
    fn stacked(shapes: Vec<Shape>) -> (Editor, Vec<ShapeId>) {
        let mut ed = editor_with(shapes);
        ed.apply(Command::SetTool(Tool::Select));
        let ids = ids(&ed);
        (ed, ids)
    }

    /// Square of side 100 at `(x, x)`.
    fn square(x: f32) -> Shape {
        rect(x, x, 100.0, 100.0)
    }

    /// A small square far from every [`square`] near the origin.
    fn far() -> Shape {
        rect(5_000.0, 5_000.0, 10.0, 10.0)
    }

    #[test]
    fn ctrl_bracket_right_raises_selection_one_step() {
        // Arrange
        let (mut ed, id) = stacked(vec![square(0.0), square(50.0), square(80.0)]);
        ed.selection = vec![id[0]];

        // Act
        let changed = key_with(&mut ed, Key::BracketRight, CTRL);

        // Assert
        assert!(changed);
        assert_eq!(ids(&ed), vec![id[1], id[0], id[2]]);
    }

    #[test]
    fn ctrl_bracket_left_lowers_selection_one_step() {
        // Arrange
        let (mut ed, id) = stacked(vec![square(0.0), square(50.0), square(80.0)]);
        ed.selection = vec![id[2]];

        // Act
        let changed = key_with(&mut ed, Key::BracketLeft, CTRL);

        // Assert
        assert!(changed);
        assert_eq!(ids(&ed), vec![id[0], id[2], id[1]]);
    }

    #[test]
    fn ctrl_bracket_in_text_edit_resizes_text_not_order() {
        // Arrange: two overlapping squares, the bottom one selected, and a
        // text block open.
        let mut ed = text_editor();
        let tx = tx_insert(&mut ed.doc, [square(0.0), square(50.0)]);
        assert!(ed.history.commit(&mut ed.doc, tx).is_ok());
        let id = ids(&ed);
        place(&mut ed, 600.0, 500.0, "a");
        ed.selection = vec![id[0]];
        let size = open_size(&ed);

        // Act
        assert!(key_with(&mut ed, Key::BracketRight, CTRL));

        // Assert
        assert!(open_size(&ed) > size);
        assert!(ed.text_edit().is_some());
        assert_eq!(ids(&ed), id);
    }

    #[test]
    fn raise_passes_only_the_nearest_overlapping_shape() {
        // Arrange: a far shape, then two overlapping ones above the
        // selection.
        let (mut ed, id) = stacked(vec![square(0.0), far(), square(50.0), square(60.0)]);
        ed.selection = vec![id[0]];

        // Act
        assert!(ed.apply(Command::RaiseSelection));

        // Assert
        assert_eq!(ids(&ed), vec![id[1], id[2], id[0], id[3]]);
    }

    #[test]
    fn lower_passes_only_the_nearest_overlapping_shape() {
        // Arrange: mirror of the raise case.
        let (mut ed, id) = stacked(vec![square(60.0), square(50.0), far(), square(0.0)]);
        ed.selection = vec![id[3]];

        // Act
        assert!(ed.apply(Command::LowerSelection));

        // Assert
        assert_eq!(ids(&ed), vec![id[0], id[3], id[1], id[2]]);
    }

    #[test]
    fn raise_with_no_overlapping_shape_above_adds_no_undo_step() {
        // Arrange
        let (mut ed, id) = stacked(vec![square(60.0), square(0.0), far()]);
        ed.selection = vec![id[1]];
        let undo = ed.history.undo_len();

        // Act
        let changed = key_with(&mut ed, Key::BracketRight, CTRL);

        // Assert
        assert!(!changed);
        assert_eq!(ids(&ed), id);
        assert_eq!(ed.history.undo_len(), undo);
    }

    #[test]
    fn lower_with_no_overlapping_shape_below_adds_no_undo_step() {
        // Arrange
        let (mut ed, id) = stacked(vec![far(), square(0.0), square(60.0)]);
        ed.selection = vec![id[1]];
        let undo = ed.history.undo_len();

        // Act
        let changed = key_with(&mut ed, Key::BracketLeft, CTRL);

        // Assert
        assert!(!changed);
        assert_eq!(ids(&ed), id);
        assert_eq!(ed.history.undo_len(), undo);
    }

    #[test]
    fn ctrl_shift_bracket_right_moves_selection_to_top_keeping_order() {
        // Arrange
        let (mut ed, id) = stacked(vec![square(0.0), far(), square(20.0), square(30.0)]);
        ed.selection = vec![id[2], id[0]];

        // Act
        let changed = key_with(&mut ed, Key::BracketRight, CTRL_SHIFT);

        // Assert
        assert!(changed);
        assert_eq!(ids(&ed), vec![id[1], id[3], id[0], id[2]]);
    }

    #[test]
    fn ctrl_shift_bracket_left_moves_selection_to_bottom_keeping_order() {
        // Arrange
        let (mut ed, id) = stacked(vec![square(0.0), far(), square(20.0), square(30.0)]);
        ed.selection = vec![id[3], id[1]];

        // Act
        let changed = key_with(&mut ed, Key::BracketLeft, CTRL_SHIFT);

        // Assert
        assert!(changed);
        assert_eq!(ids(&ed), vec![id[1], id[3], id[0], id[2]]);
    }

    #[test]
    fn to_top_when_already_on_top_adds_no_undo_step() {
        // Arrange
        let (mut ed, id) = stacked(vec![square(0.0), square(20.0), square(30.0)]);
        ed.selection = vec![id[2], id[1]];
        let undo = ed.history.undo_len();

        // Act
        let top = ed.apply(Command::SelectionToTop);
        ed.selection = vec![id[0]];
        let bottom = ed.apply(Command::SelectionToBottom);

        // Assert
        assert!(!top);
        assert!(!bottom);
        assert_eq!(ids(&ed), id);
        assert_eq!(ed.history.undo_len(), undo);
    }

    #[test]
    fn restack_is_one_undo_step_and_keeps_selection() {
        // Arrange
        let (mut ed, id) = stacked(vec![square(0.0), square(50.0), square(70.0)]);
        ed.selection = vec![id[1], id[0]];
        let undo = ed.history.undo_len();
        let revision = ed.document_revision();

        // Act
        assert!(ed.apply(Command::SelectionToTop));

        // Assert
        assert_eq!(ids(&ed), vec![id[2], id[0], id[1]]);
        assert_eq!(ed.history.undo_len(), undo + 1);
        assert_eq!(ed.selection(), &[id[1], id[0]]);
        assert!(ed.document_revision() > revision);
        assert_eq!(ed.document_base_revision(), ed.document_revision());
        assert!(key_with(&mut ed, Key::Z, CTRL));
        assert_eq!(ids(&ed), id);
        assert_eq!(ed.selection(), &[id[1], id[0]]);
        assert!(ed.apply(Command::Redo));
        assert_eq!(ids(&ed), vec![id[2], id[0], id[1]]);
    }

    #[test]
    fn restack_during_gesture_does_nothing() {
        // Arrange: a pen stroke in progress, a square selected.
        let (mut ed, id) = stacked(vec![square(0.0), square(50.0)]);
        ed.apply(Command::SetTool(Tool::Pen));
        ed.selection = vec![id[0]];
        down(&mut ed, PointerButton::Left, 700.0, 700.0);
        move_to(&mut ed, 720.0, 730.0);
        assert!(ed.gesture.is_some());
        let undo = ed.history.undo_len();

        for command in [
            Command::RaiseSelection,
            Command::LowerSelection,
            Command::SelectionToTop,
            Command::SelectionToBottom,
        ] {
            // Act
            let changed = ed.apply(command);

            // Assert
            assert!(!changed, "{command:?}");
            assert!(ed.gesture.is_some(), "{command:?}");
            assert_eq!(ids(&ed), id, "{command:?}");
            assert_eq!(ed.history.undo_len(), undo, "{command:?}");
        }
    }

    #[test]
    fn restack_with_empty_selection_does_nothing() {
        // Arrange
        let (mut ed, id) = stacked(vec![square(0.0), square(50.0)]);
        let before = ed.clone();

        for command in [
            Command::RaiseSelection,
            Command::LowerSelection,
            Command::SelectionToTop,
            Command::SelectionToBottom,
        ] {
            // Act
            let changed = ed.apply(command);

            // Assert
            assert!(!changed, "{command:?}");
            assert_eq!(ids(&ed), id, "{command:?}");
        }
        assert_eq!(ed, before);
    }

    // ---- T101 align and distribute -----------------------------------------

    use crate::core::command::{Axis, Side};

    /// Every align and distribute command.
    const ARRANGE_COMMANDS: [Command; 6] = [
        Command::Align(Side::Left),
        Command::Align(Side::Right),
        Command::Align(Side::Top),
        Command::Align(Side::Bottom),
        Command::Distribute(Axis::Horizontal),
        Command::Distribute(Axis::Vertical),
    ];

    /// Tolerance for world coordinates in the T101 tests.
    const ARRANGE_EPS: f32 = 1e-3;

    /// World bounds of the shape `id`; panics if it is missing.
    fn bounds_of(ed: &Editor, id: ShapeId) -> Aabb {
        match ed.document().get(id) {
            Some(shape) => shape.bounds(),
            None => panic!("shape {id:?} missing"),
        }
    }

    /// Every shape of the document, bottom first.
    fn shapes_of(ed: &Editor) -> Vec<Shape> {
        ed.document().shapes().map(|(_, s)| s.clone()).collect()
    }

    /// Editor with the Select tool holding `shapes`, all selected, and
    /// their ids bottom first.
    fn arranged(shapes: Vec<Shape>) -> (Editor, Vec<ShapeId>) {
        let (mut ed, id) = stacked(shapes);
        ed.selection = id.clone();
        (ed, id)
    }

    /// Three rectangles of widths 10, 20 and 40 spread unevenly. With the
    /// outline width 1 their bounds are (-0.5, -0.5)–(10.5, 10.5),
    /// (29.5, 49.5)–(50.5, 80.5) and (199.5, 19.5)–(240.5, 25.5).
    fn uneven() -> Vec<Shape> {
        vec![
            rect(0.0, 0.0, 10.0, 10.0),
            rect(30.0, 50.0, 20.0, 30.0),
            rect(200.0, 20.0, 40.0, 5.0),
        ]
    }

    #[test]
    fn alt_arrow_left_aligns_left_edges() {
        // Arrange
        let (mut ed, id) = arranged(uneven());
        let before: Vec<Aabb> = id.iter().map(|&i| bounds_of(&ed, i)).collect();

        // Act
        let changed = key_with(&mut ed, Key::ArrowLeft, ALT);

        // Assert: left edges meet the box's, sizes and `y` kept.
        assert!(changed);
        for (&i, old) in id.iter().zip(before) {
            let b = bounds_of(&ed, i);
            assert!(approx_eq(b.min.x, -0.5, ARRANGE_EPS), "{i:?}: {b:?}");
            assert!(approx_eq(b.min.y, old.min.y, ARRANGE_EPS), "{i:?}");
            assert!(approx_eq(b.width(), old.width(), ARRANGE_EPS), "{i:?}");
            assert!(approx_eq(b.height(), old.height(), ARRANGE_EPS), "{i:?}");
        }
    }

    #[test]
    fn alt_arrows_align_each_side() {
        type Edge = fn(&Aabb) -> f32;
        let cases: [(Key, Edge, f32); 3] = [
            (Key::ArrowRight, |b| b.max.x, 240.5),
            (Key::ArrowUp, |b| b.min.y, -0.5),
            (Key::ArrowDown, |b| b.max.y, 80.5),
        ];
        for (key, edge, expected) in cases {
            // Arrange
            let (mut ed, id) = arranged(uneven());

            // Act
            let changed = key_with(&mut ed, key, ALT);

            // Assert
            assert!(changed, "{key:?}");
            for &i in &id {
                let got = edge(&bounds_of(&ed, i));
                assert!(
                    approx_eq(got, expected, ARRANGE_EPS),
                    "{key:?} {i:?}: {got}"
                );
            }
        }
    }

    #[test]
    fn align_moves_grid_and_text_as_whole_shapes() {
        // Arrange: a grid with index bands and a text block.
        let mut banded = grid(100.0, 100.0, 40.0, 40.0, 2, 2);
        if let Shape::Grid { axes, .. } = &mut banded {
            *axes = true;
        }
        let text = Shape::Text {
            origin: Vec2::new(0.0, 300.0),
            text: "abc".to_owned(),
            size: 20.0,
            style: Style {
                color: ColorId::INK,
                width: 1.0,
            },
        };
        let (mut ed, id) = arranged(vec![banded, text]);
        let before: Vec<Aabb> = id.iter().map(|&i| bounds_of(&ed, i)).collect();
        let left = before[0].min.x.min(before[1].min.x);

        // Act
        assert!(ed.apply(Command::Align(Side::Left)));

        // Assert: bands and text box line up, nothing else changes.
        for (&i, old) in id.iter().zip(&before) {
            let b = bounds_of(&ed, i);
            assert!(approx_eq(b.min.x, left, ARRANGE_EPS), "{i:?}: {b:?}");
            assert!(approx_eq(b.width(), old.width(), ARRANGE_EPS), "{i:?}");
            assert!(approx_eq(b.min.y, old.min.y, ARRANGE_EPS), "{i:?}");
        }
        assert!(matches!(
            ed.document().get(id[0]),
            Some(Shape::Grid {
                cols: 2,
                rows: 2,
                axes: true,
                ..
            })
        ));
        assert!(matches!(
            ed.document().get(id[1]),
            Some(Shape::Text { text, .. }) if text == "abc"
        ));
    }

    #[test]
    fn alt_h_spaces_shapes_with_equal_gaps() {
        // Arrange: gap = 41 / 4 = 10.25; the first shape's centre y is 5.
        let (mut ed, id) = arranged(uneven());
        let first = bounds_of(&ed, id[0]);
        let last = bounds_of(&ed, id[2]);

        // Act
        let changed = key_with(&mut ed, Key::H, ALT);

        // Assert: the first stays, the row is compact, all share y 5.
        assert!(changed);
        assert_eq!(bounds_of(&ed, id[0]), first);
        let middle = bounds_of(&ed, id[1]);
        let moved_last = bounds_of(&ed, id[2]);
        assert!(approx_eq(middle.min.x, 20.75, ARRANGE_EPS), "{middle:?}");
        assert!(approx_eq(middle.min.y, -10.5, ARRANGE_EPS), "{middle:?}");
        assert!(
            approx_eq(moved_last.min.x, 52.0, ARRANGE_EPS),
            "{moved_last:?}"
        );
        assert!(approx_eq(moved_last.width(), last.width(), ARRANGE_EPS));
        assert!(
            approx_eq(moved_last.min.y, 2.0, ARRANGE_EPS),
            "{moved_last:?}"
        );
        assert!(approx_eq(middle.min.x - first.max.x, 10.25, ARRANGE_EPS));
        assert!(approx_eq(
            moved_last.min.x - middle.max.x,
            10.25,
            ARRANGE_EPS
        ));
    }

    #[test]
    fn alt_v_spaces_shapes_with_equal_vertical_gaps() {
        // Arrange: tops -0.5, 299.5 and 29.5; the middle one (by top) is
        // 21 high, the largest is 41 high, so gap = 10.25; the first
        // shape's centre x is 5.
        let (mut ed, id) = arranged(vec![
            rect(0.0, 0.0, 10.0, 10.0),
            rect(50.0, 300.0, 10.0, 40.0),
            rect(20.0, 30.0, 30.0, 20.0),
        ]);
        let first = bounds_of(&ed, id[0]);
        let last = bounds_of(&ed, id[1]);

        // Act
        let changed = key_with(&mut ed, Key::V, ALT);

        // Assert
        assert!(changed);
        assert_eq!(bounds_of(&ed, id[0]), first);
        let moved_last = bounds_of(&ed, id[1]);
        assert!(
            approx_eq(moved_last.min.y, 52.0, ARRANGE_EPS),
            "{moved_last:?}"
        );
        assert!(approx_eq(moved_last.height(), last.height(), ARRANGE_EPS));
        assert!(
            approx_eq(moved_last.min.x, -0.5, ARRANGE_EPS),
            "{moved_last:?}"
        );
        let middle = bounds_of(&ed, id[2]);
        assert!(approx_eq(middle.min.y, 20.75, ARRANGE_EPS), "{middle:?}");
        assert!(approx_eq(middle.min.x, -10.5, ARRANGE_EPS), "{middle:?}");
    }

    #[test]
    fn distribute_overlapping_shapes_grows_the_row() {
        // Arrange: three 101-wide bounds starting at -0.5, 9.5 and 49.5;
        // the gap is 101 / 4 = 25.25, so the row grows.
        let (mut ed, id) = arranged(vec![
            rect(0.0, 0.0, 100.0, 10.0),
            rect(10.0, 20.0, 100.0, 10.0),
            rect(50.0, 40.0, 100.0, 10.0),
        ]);
        let before = bounds_of(&ed, id[0]);

        // Act
        assert!(ed.apply(Command::Distribute(Axis::Horizontal)));

        // Assert: no overlap, one line, the first shape still.
        let [first, middle, last] = [0, 1, 2].map(|k| bounds_of(&ed, id[k]));
        assert_eq!(first, before);
        assert!(approx_eq(middle.min.x, 125.75, ARRANGE_EPS), "{middle:?}");
        assert!(approx_eq(last.min.x, 252.0, ARRANGE_EPS), "{last:?}");
        for b in [middle, last] {
            assert!(approx_eq(b.min.y, -0.5, ARRANGE_EPS), "{b:?}");
        }
    }

    #[test]
    fn distribute_two_overlapping_shapes_separates_them() {
        // Arrange: 41-wide bounds at -0.5 and 9.5 overlap by 31; the gap
        // becomes 41 / 4 = 10.25 and the centres meet at y 20.
        let (mut ed, id) = arranged(vec![
            rect(0.0, 0.0, 40.0, 40.0),
            rect(10.0, 30.0, 40.0, 40.0),
        ]);
        let undo = ed.history.undo_len();

        // Act
        let changed = key_with(&mut ed, Key::H, ALT);

        // Assert
        assert!(changed);
        let second = bounds_of(&ed, id[1]);
        assert!(approx_eq(second.min.x, 50.75, ARRANGE_EPS), "{second:?}");
        assert!(approx_eq(second.min.y, -0.5, ARRANGE_EPS), "{second:?}");
        assert_eq!(ed.history.undo_len(), undo + 1);
    }

    #[test]
    fn align_with_one_shape_does_nothing() {
        // Arrange
        let (mut ed, _) = arranged(vec![rect(10.0, 20.0, 30.0, 40.0)]);
        let before = ed.clone();

        for command in ARRANGE_COMMANDS {
            // Act
            let changed = ed.apply(command);

            // Assert
            assert!(!changed, "{command:?}");
        }
        assert_eq!(ed, before);
    }

    #[test]
    fn distribute_with_one_shape_does_nothing() {
        // Arrange
        let (mut ed, _) = arranged(vec![rect(300.0, 200.0, 50.0, 10.0)]);
        let before = ed.clone();

        // Act
        let horizontal = key_with(&mut ed, Key::H, ALT);
        let vertical = key_with(&mut ed, Key::V, ALT);

        // Assert
        assert!(!horizontal);
        assert!(!vertical);
        assert_eq!(ed, before);
    }

    #[test]
    fn align_when_already_aligned_adds_no_undo_step() {
        // Arrange
        let (mut ed, _) = arranged(vec![
            rect(0.0, 0.0, 10.0, 10.0),
            rect(0.0, 50.0, 30.0, 10.0),
        ]);
        let shapes = shapes_of(&ed);
        let undo = ed.history.undo_len();

        // Act
        let changed = key_with(&mut ed, Key::ArrowLeft, ALT);

        // Assert
        assert!(!changed);
        assert_eq!(shapes_of(&ed), shapes);
        assert_eq!(ed.history.undo_len(), undo);
    }

    #[test]
    fn distribute_twice_adds_one_undo_step() {
        // Arrange
        let (mut ed, _) = arranged(uneven());
        let undo = ed.history.undo_len();

        // Act
        let first = key_with(&mut ed, Key::H, ALT);
        let spaced = shapes_of(&ed);
        let second = key_with(&mut ed, Key::H, ALT);

        // Assert
        assert!(first);
        assert!(!second);
        assert_eq!(shapes_of(&ed), spaced);
        assert_eq!(ed.history.undo_len(), undo + 1);
    }

    #[test]
    fn align_is_one_undo_step_and_keeps_selection() {
        // Arrange
        let (mut ed, id) = arranged(uneven());
        ed.selection = vec![id[2], id[0], id[1]];
        let before = shapes_of(&ed);
        let undo = ed.history.undo_len();
        let revision = ed.document_revision();

        // Act
        assert!(key_with(&mut ed, Key::ArrowDown, ALT));
        let after = shapes_of(&ed);

        // Assert
        assert_ne!(after, before);
        assert_eq!(ids(&ed), id);
        assert_eq!(ed.history.undo_len(), undo + 1);
        assert_eq!(ed.selection(), &[id[2], id[0], id[1]]);
        assert!(ed.document_revision() > revision);
        assert!(key_with(&mut ed, Key::Z, CTRL));
        assert_eq!(shapes_of(&ed), before);
        assert_eq!(ed.selection(), &[id[2], id[0], id[1]]);
        assert!(ed.apply(Command::Redo));
        assert_eq!(shapes_of(&ed), after);
    }

    #[test]
    fn align_during_gesture_does_nothing() {
        // Arrange: a pen stroke in progress, three shapes selected.
        let (mut ed, id) = arranged(uneven());
        ed.apply(Command::SetTool(Tool::Pen));
        ed.selection = id.clone();
        down(&mut ed, PointerButton::Left, 700.0, 700.0);
        move_to(&mut ed, 720.0, 730.0);
        assert!(ed.gesture.is_some());
        let shapes = shapes_of(&ed);
        let undo = ed.history.undo_len();

        for command in ARRANGE_COMMANDS {
            // Act
            let changed = ed.apply(command);

            // Assert
            assert!(!changed, "{command:?}");
            assert!(ed.gesture.is_some(), "{command:?}");
            assert_eq!(shapes_of(&ed), shapes, "{command:?}");
            assert_eq!(ed.history.undo_len(), undo, "{command:?}");
        }
    }

    #[test]
    fn alt_arrow_in_text_edit_moves_caret_not_shapes() {
        // Arrange: two rectangles selected and a text block open.
        let mut ed = text_editor();
        let tx = tx_insert(
            &mut ed.doc,
            [rect(0.0, 0.0, 10.0, 10.0), rect(40.0, 40.0, 30.0, 10.0)],
        );
        assert!(ed.history.commit(&mut ed.doc, tx).is_ok());
        let id = ids(&ed);
        place(&mut ed, 600.0, 500.0, "ab");
        ed.selection = id.clone();
        let rects: Vec<Shape> = id
            .iter()
            .filter_map(|&i| ed.document().get(i).cloned())
            .collect();
        let caret = ed.text_edit().map(TextEdit::caret);

        // Act
        key_with(&mut ed, Key::ArrowLeft, ALT);

        // Assert
        assert!(ed.text_edit().is_some());
        assert_eq!(ed.text_edit().map(TextEdit::caret), caret.map(|c| c - 1));
        let now: Vec<Shape> = id
            .iter()
            .filter_map(|&i| ed.document().get(i).cloned())
            .collect();
        assert_eq!(now, rects);
    }

    // ---- T107 locks -----------------------------------------------------------

    /// The locked ids, bottom first.
    fn locked(ed: &Editor) -> Vec<ShapeId> {
        ids(ed)
            .into_iter()
            .filter(|id| ed.document().is_locked(*id))
            .collect()
    }

    /// Editor with an 800 × 600 viewport holding `shapes`, those at the
    /// positions `lock` locked with `Command::LockSelection`, and `tool`
    /// active.
    fn locked_editor(shapes: Vec<Shape>, lock: &[usize], tool: Tool) -> Editor {
        let mut ed = shape_text_editor(shapes, Tool::Select);
        let all = ids(&ed);
        ed.selection = lock.iter().map(|&i| all[i]).collect();
        assert!(ed.apply(Command::LockSelection));
        ed.apply(Command::SetTool(tool));
        ed
    }

    fn three_rects() -> Vec<Shape> {
        vec![
            rect(0.0, 0.0, 50.0, 50.0),
            rect(100.0, 0.0, 50.0, 50.0),
            rect(200.0, 0.0, 50.0, 50.0),
        ]
    }

    #[test]
    fn lock_locks_selection_and_clears_it() {
        // Arrange
        let mut ed = shape_text_editor(three_rects(), Tool::Select);
        let all = ids(&ed);
        ed.selection = vec![all[2], all[0]];

        // Act
        let redraw = key_with(&mut ed, Key::K, CTRL);

        // Assert
        assert!(redraw);
        assert_eq!(locked(&ed), vec![all[0], all[2]]);
        assert_eq!(ed.selection(), []);
        assert_eq!(ids(&ed), all);
    }

    #[test]
    fn lock_is_one_undo_step() {
        // Arrange
        let mut ed = locked_editor(three_rects(), &[0, 2], Tool::Select);
        let all = ids(&ed);

        // Act
        let undone = ed.apply(Command::Undo);
        let after_undo = locked(&ed);
        let redone = ed.apply(Command::Redo);

        // Assert
        assert!(undone && redone);
        assert_eq!(after_undo, vec![]);
        assert_eq!(ids(&ed), all);
        assert_eq!(locked(&ed), vec![all[0], all[2]]);
    }

    #[test]
    fn lock_with_empty_selection_records_nothing() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);

        // Act
        let redraw = ed.apply(Command::LockSelection);
        ed.apply(Command::Undo);

        // Assert: the undo took back the insert, not a lock.
        assert!(!redraw);
        assert!(ed.document().is_empty());
    }

    #[test]
    fn unlock_all_is_one_undo_step() {
        // Arrange
        let mut ed = locked_editor(three_rects(), &[0, 1], Tool::Select);
        let all = ids(&ed);

        // Act
        let redraw = key_with(&mut ed, Key::K, CTRL_SHIFT);
        let after = locked(&ed);
        ed.apply(Command::Undo);

        // Assert
        assert!(redraw);
        assert_eq!(after, vec![]);
        assert_eq!(locked(&ed), vec![all[0], all[1]]);
    }

    #[test]
    fn unlock_all_without_locked_shapes_records_nothing() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);

        // Act
        let redraw = ed.apply(Command::UnlockAll);
        ed.apply(Command::Undo);

        // Assert
        assert!(!redraw);
        assert!(ed.document().is_empty());
    }

    #[test]
    fn selection_commands_never_touch_locked_shapes() {
        let commands = [
            Command::DeleteSelection,
            Command::Cut,
            Command::Duplicate,
            Command::ScaleUp,
            Command::ScaleDown,
            Command::Align(Side::Left),
            Command::Align(Side::Bottom),
            Command::Distribute(Axis::Horizontal),
            Command::RaiseSelection,
            Command::LowerSelection,
            Command::SelectionToTop,
            Command::SelectionToBottom,
            Command::GridCols(1),
            Command::GridRows(-1),
        ];
        for command in commands {
            // Arrange: a locked grid under two free rectangles.
            let shapes = vec![
                grid(0.0, 0.0, 400.0, 400.0, 4, 4),
                rect(30.0, 30.0, 40.0, 40.0),
                rect(250.0, 120.0, 60.0, 20.0),
            ];
            let mut ed = locked_editor(shapes, &[0], Tool::Select);
            let all = ids(&ed);
            let before = ed.document().get(all[0]).cloned();

            // Act
            ed.apply(Command::SelectAll);
            let selected = ed.selection().to_vec();
            ed.apply(command);

            // Assert
            assert_eq!(selected, vec![all[1], all[2]], "{command:?}");
            assert_eq!(ed.document().get(all[0]).cloned(), before, "{command:?}");
            assert!(ed.document().is_locked(all[0]), "{command:?}");
        }
    }

    #[test]
    fn clear_all_keeps_locked_shapes() {
        // Arrange
        let mut ed = locked_editor(three_rects(), &[1], Tool::Select);
        let all = ids(&ed);

        // Act
        let redraw = ed.apply(Command::ClearAll);
        let left = ids(&ed);
        ed.apply(Command::Undo);

        // Assert
        assert!(redraw);
        assert_eq!(left, vec![all[1]]);
        assert_eq!(ids(&ed), all);
        assert_eq!(locked(&ed), vec![all[1]]);
    }

    #[test]
    fn clear_all_with_only_locked_shapes_records_nothing() {
        // Arrange
        let mut ed = locked_editor(vec![rect(0.0, 0.0, 10.0, 10.0)], &[0], Tool::Select);
        let all = ids(&ed);

        // Act
        let redraw = ed.apply(Command::ClearAll);
        ed.apply(Command::Undo);

        // Assert: the undo took back the lock.
        assert!(!redraw);
        assert_eq!(ids(&ed), all);
        assert_eq!(locked(&ed), vec![]);
    }

    #[test]
    fn redo_of_lock_drops_it_from_selection() {
        // Arrange
        let mut ed = locked_editor(three_rects(), &[0], Tool::Select);
        let all = ids(&ed);
        ed.apply(Command::Undo);
        ed.selection = all.clone();

        // Act
        ed.apply(Command::Redo);

        // Assert
        assert_eq!(ed.selection(), [all[1], all[2]]);
    }

    #[test]
    fn bucket_fills_locked_grid_cell() {
        // Arrange: 4 × 4 cells of 50.
        let mut ed = locked_editor(vec![grid(0.0, 0.0, 200.0, 200.0, 4, 4)], &[0], Tool::Bucket);
        let id = ids(&ed)[0];
        let color = ed.style().color;

        // Act
        click(&mut ed, 60.0, 110.0);

        // Assert
        let fill = ed.document().get(id).and_then(|s| s.cell_fill(1, 2));
        assert_eq!(fill, Some(color));
        assert!(ed.document().is_locked(id));
    }

    #[test]
    fn bucket_fills_locked_rect() {
        // Arrange
        let mut ed = locked_editor(vec![rect(0.0, 0.0, 200.0, 100.0)], &[0], Tool::Bucket);
        let id = ids(&ed)[0];
        let color = ed.style().color;

        // Act
        click(&mut ed, 100.0, 50.0);

        // Assert
        assert_eq!(ed.document().get(id).and_then(Shape::fill), Some(color));
        assert!(ed.document().is_locked(id));
    }

    #[test]
    fn text_tool_types_into_locked_grid_cell() {
        // Arrange: 4 × 4 cells of 50.
        let mut ed = locked_editor(vec![grid(0.0, 0.0, 200.0, 200.0, 4, 4)], &[0], Tool::Text);
        let id = ids(&ed)[0];

        // Act
        place(&mut ed, 60.0, 110.0, "7");
        key(&mut ed, Key::Escape);

        // Assert
        assert_eq!(slot_text(&ed, 0, cell(1, 2)).as_deref(), Some("7"));
        assert!(ed.document().is_locked(id));
    }

    #[test]
    fn emptying_locked_text_block_keeps_it() {
        // Arrange
        let block = Shape::Text {
            origin: Vec2::new(50.0, 40.0),
            text: "abc".to_owned(),
            size: 18.0,
            style: Style {
                color: ColorId::INK,
                width: 0.0,
            },
        };
        let mut ed = locked_editor(vec![block], &[0], Tool::Text);
        let id = ids(&ed)[0];

        // Act
        click(&mut ed, 55.0, 45.0);
        let opened = ed.text_edit().and_then(TextEdit::target);
        for _ in 0..4 {
            key(&mut ed, Key::Delete);
            key(&mut ed, Key::Backspace);
        }
        key(&mut ed, Key::Escape);

        // Assert
        assert_eq!(opened, Some(id));
        assert_eq!(texts(&ed), vec!["abc".to_owned()]);
        assert!(ed.document().is_locked(id));
    }

    // ---- T109 shape recognition --------------------------------------------

    mod recognize_shape {
        use super::*;

        /// An editor with a viewport, on the pen tool.
        fn pen_editor() -> Editor {
            let mut ed = Editor::new();
            ed.handle(InputEvent::Resize {
                size: Vec2::new(1_200.0, 900.0),
            });
            ed.apply(Command::SetTool(Tool::Pen));
            ed
        }

        /// Drags the left button through `points` (screen pixels).
        fn drag_through(ed: &mut Editor, points: &[Vec2]) {
            let (Some(first), Some(last)) = (points.first(), points.last()) else {
                return;
            };
            down(ed, PointerButton::Left, first.x, first.y);
            for p in &points[1..] {
                move_to(ed, p.x, p.y);
            }
            up(ed, PointerButton::Left, last.x, last.y);
        }

        /// A slightly wobbly circle of radius 150 around `(cx, cy)`.
        fn circle_path(cx: f32, cy: f32) -> Vec<Vec2> {
            (0..=72)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 72.0;
                    let r = 150.0 + 3.0 * (5.0 * a).sin();
                    Vec2::new(cx + r * a.cos(), cy + r * a.sin())
                })
                .collect()
        }

        /// A zigzag scribble starting at `(x, y)`.
        fn scribble_path(x: f32, y: f32) -> Vec<Vec2> {
            (0..=12)
                .map(|i| {
                    let dx = if i % 2 == 0 { 0.0 } else { 180.0 };
                    Vec2::new(x + dx, y + i as f32 * 15.0)
                })
                .collect()
        }

        fn shapes_of(ed: &Editor) -> Vec<(ShapeId, Shape)> {
            ed.document()
                .shapes()
                .map(|(id, shape)| (id, shape.clone()))
                .collect()
        }

        #[test]
        fn z_after_a_pen_stroke_replaces_it_with_its_shape() {
            // Arrange
            let mut ed = pen_editor();
            drag_through(&mut ed, &circle_path(400.0, 400.0));
            let drawn = shapes_of(&ed);
            assert!(matches!(drawn.as_slice(), [(_, Shape::Stroke { .. })]));
            let revision = ed.document_revision();

            // Act
            let changed = key(&mut ed, Key::Z);

            // Assert
            assert!(changed);
            let now = shapes_of(&ed);
            assert_eq!(now.len(), 1);
            assert_eq!(now[0].0, drawn[0].0, "same id, same place");
            assert!(matches!(now[0].1, Shape::Ellipse { .. }), "{:?}", now[0].1);
            assert_eq!(now[0].1.style(), drawn[0].1.style());
            assert!(ed.document_revision() > revision);
        }

        #[test]
        fn z_after_another_edit_does_nothing() {
            // A circle, then a rectangle drawn with the tool; a circle, then
            // the selection grown.
            let edits: [fn(&mut Editor); 2] = [
                |ed| {
                    ed.apply(Command::SetTool(Tool::Rect));
                    drag_through(ed, &[Vec2::new(700.0, 100.0), Vec2::new(800.0, 200.0)]);
                },
                |ed| {
                    ed.apply(Command::SelectAll);
                    assert!(ed.apply(Command::ScaleUp));
                },
            ];
            for (case, edit) in edits.into_iter().enumerate() {
                // Arrange
                let mut ed = pen_editor();
                drag_through(&mut ed, &circle_path(400.0, 400.0));
                edit(&mut ed);
                let before = shapes_of(&ed);
                let undo = ed.history.undo_len();

                // Act
                let changed = key(&mut ed, Key::Z);

                // Assert
                assert!(!changed, "case {case}");
                assert_eq!(shapes_of(&ed), before, "case {case}");
                assert_eq!(ed.history.undo_len(), undo, "case {case}");
            }
        }

        #[test]
        fn z_after_undo_does_nothing() {
            // Arrange: two circles, the second undone.
            let mut ed = pen_editor();
            drag_through(&mut ed, &circle_path(300.0, 300.0));
            drag_through(&mut ed, &circle_path(700.0, 500.0));
            assert!(ed.apply(Command::Undo));
            let before = shapes_of(&ed);

            // Act
            let changed = key(&mut ed, Key::Z);

            // Assert
            assert!(!changed);
            assert_eq!(shapes_of(&ed), before);
            assert!(ed.can_redo());
        }

        #[test]
        fn z_during_a_gesture_does_nothing() {
            // Arrange: a circle, then a second stroke still being drawn.
            let mut ed = pen_editor();
            drag_through(&mut ed, &circle_path(300.0, 300.0));
            let before = shapes_of(&ed);
            down(&mut ed, PointerButton::Left, 800.0, 100.0);
            move_to(&mut ed, 850.0, 160.0);
            assert!(ed.gesture.is_some());

            // Act
            let changed = key(&mut ed, Key::Z);

            // Assert
            assert!(!changed);
            assert!(ed.gesture.is_some());
            assert_eq!(shapes_of(&ed), before);
        }

        #[test]
        fn z_in_text_edit_types_z() {
            // Arrange: a circle, then a text block open.
            let mut ed = pen_editor();
            drag_through(&mut ed, &circle_path(300.0, 300.0));
            let circle = shapes_of(&ed);
            ed.apply(Command::SetTool(Tool::Text));
            place(&mut ed, 700.0, 600.0, "a");

            // Act
            ed.handle(InputEvent::Char {
                ch: 'z',
                mods: NONE,
            });
            key(&mut ed, Key::Z);

            // Assert
            assert_eq!(ed.text_edit().map(TextEdit::text), Some("az"));
            assert_eq!(shapes_of(&ed), circle);
        }

        #[test]
        fn z_is_one_undo_step_and_undo_restores_the_stroke() {
            // Arrange
            let mut ed = pen_editor();
            drag_through(&mut ed, &circle_path(400.0, 400.0));
            let drawn = shapes_of(&ed);
            let undo = ed.history.undo_len();

            // Act
            assert!(key(&mut ed, Key::Z));

            // Assert
            assert_eq!(ed.history.undo_len(), undo + 1);
            let shape = shapes_of(&ed);
            assert!(key_with(&mut ed, Key::Z, CTRL));
            assert_eq!(shapes_of(&ed), drawn);
            assert!(ed.apply(Command::Redo));
            assert_eq!(shapes_of(&ed), shape);
        }

        #[test]
        fn z_on_a_scribble_changes_nothing_and_records_no_step() {
            // Arrange
            let mut ed = pen_editor();
            drag_through(&mut ed, &scribble_path(300.0, 300.0));
            let before = ed.clone();
            let revision = ed.document_revision();

            // Act
            let changed = key(&mut ed, Key::Z);

            // Assert
            assert!(!changed);
            assert_eq!(ed, before);
            assert_eq!(ed.document_revision(), revision);
        }

        #[test]
        fn z_twice_acts_once() {
            // Arrange
            let mut ed = pen_editor();
            drag_through(&mut ed, &circle_path(400.0, 400.0));
            assert!(key(&mut ed, Key::Z));
            let once = shapes_of(&ed);
            let undo = ed.history.undo_len();

            // Act
            let again = key(&mut ed, Key::Z);

            // Assert
            assert!(!again);
            assert_eq!(shapes_of(&ed), once);
            assert_eq!(ed.history.undo_len(), undo);
        }
    }

    // ---- T110: minimap ----------------------------------------------------

    /// Editor with an 800 × 600 viewport and two rectangles far apart, so
    /// the view at the origin lies inside the content.
    fn minimap_editor() -> Editor {
        let mut ed = editor_with(vec![
            rect(0.0, 0.0, 100.0, 100.0),
            rect(3660.0, 2660.0, 100.0, 100.0),
        ]);
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        ed
    }

    fn release_space(ed: &mut Editor) -> bool {
        ed.handle(InputEvent::KeyUp {
            key: Key::Space,
            mods: NONE,
        })
    }

    /// The minimap, which must be showing.
    fn shown(ed: &Editor) -> Minimap {
        let Some(map) = ed.minimap() else {
            panic!("the minimap shows");
        };
        map
    }

    fn view_rect(ed: &Editor) -> Aabb {
        ed.camera().visible_world_rect(ed.viewport())
    }

    fn assert_view_centred(ed: &Editor, world: Vec2) {
        let center = view_rect(ed).center();
        assert!(center.approx_eq(world, 0.05), "{center:?} is not {world:?}");
    }

    #[test]
    fn minimap_shows_while_space_is_held() {
        // Arrange
        let mut ed = minimap_editor();
        let hidden = ed.minimap();

        // Act
        key(&mut ed, Key::Space);
        let held = ed.minimap();
        release_space(&mut ed);

        // Assert
        assert!(hidden.is_none());
        let Some(map) = held else {
            panic!("the minimap shows while Space is held");
        };
        assert_eq!(Some(map.panel()), minimap::panel_rect(ed.viewport()));
        assert!(map.world().min.approx_eq(Vec2::new(-0.5, -0.5), 1e-3));
        assert!(map.world().max.approx_eq(Vec2::new(3760.5, 2760.5), 1e-3));
        assert!(ed.minimap().is_none());
    }

    #[test]
    fn minimap_shows_during_a_middle_pan() {
        // Arrange
        let mut ed = minimap_editor();

        // Act
        down(&mut ed, PointerButton::Middle, 100.0, 100.0);
        let pressed = ed.minimap().is_some();
        move_to(&mut ed, 90.0, 80.0);
        let dragging = ed.minimap().is_some();
        up(&mut ed, PointerButton::Middle, 90.0, 80.0);

        // Assert
        assert!(pressed && dragging);
        assert!(ed.minimap().is_none());
    }

    #[test]
    fn minimap_hidden_for_a_hand_tool_pan() {
        // Arrange
        let mut ed = minimap_editor();
        ed.apply(Command::SetTool(Tool::Hand));

        // Act
        down(&mut ed, PointerButton::Left, 100.0, 100.0);

        // Assert
        assert_eq!(ed.active_gesture(), Some(ActiveGesture::Pan));
        assert!(ed.minimap().is_none());
    }

    #[test]
    fn minimap_space_in_text_edit_shows_nothing() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");

        // Act
        ed.handle(InputEvent::Char {
            ch: ' ',
            mods: NONE,
        });
        key(&mut ed, Key::Space);

        // Assert
        assert!(ed.minimap().is_none());
        assert_eq!(editing(&ed).as_deref(), Some("a "));
    }

    #[test]
    fn minimap_show_and_hide_ask_for_a_redraw() {
        let mut ed = minimap_editor();

        assert!(key(&mut ed, Key::Space), "Space shows it");
        assert!(release_space(&mut ed), "releasing Space hides it");
        assert!(
            down(&mut ed, PointerButton::Middle, 100.0, 100.0),
            "a middle press shows it"
        );
        assert!(
            up(&mut ed, PointerButton::Middle, 100.0, 100.0),
            "its release hides it"
        );
    }

    #[test]
    fn minimap_space_repeat_asks_for_no_redraw() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);

        // Act / Assert: key repeat while held.
        assert!(!key(&mut ed, Key::Space));
        assert!(!key(&mut ed, Key::Space));
        // Without room for the panel nothing shows, so nothing redraws.
        let mut small = Editor::new();
        assert!(!key(&mut small, Key::Space));
        assert!(!release_space(&mut small));
    }

    #[test]
    fn minimap_frame_moves_with_the_pan() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);
        let before = shown(&ed).frame(view_rect(&ed));

        // Act: drag the content 400 px left and 200 px up.
        down(&mut ed, PointerButton::Left, 500.0, 300.0);
        move_to(&mut ed, 100.0, 100.0);
        let map = shown(&ed);
        let after = map.frame(view_rect(&ed));

        // Assert: the frame moved by the pan, at the minimap's scale; the
        // view stayed inside the content, so the world box did not change.
        let step = Vec2::new(400.0, 200.0) * (138.0 / 2761.0);
        assert!(after.min.approx_eq(before.min + step, 0.01), "{after:?}");
        assert!(after.max.approx_eq(before.max + step, 0.01), "{after:?}");
        assert!(map.world().max.approx_eq(Vec2::new(3760.5, 2760.5), 1e-3));
    }

    #[test]
    fn minimap_click_centres_the_view() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);
        let target = shown(&ed).world_at(Vec2::new(700.0, 520.0));

        // Act
        let pressed = down(&mut ed, PointerButton::Left, 700.0, 520.0);
        let gesture = ed.active_gesture();
        up(&mut ed, PointerButton::Left, 700.0, 520.0);

        // Assert
        assert!(pressed);
        assert_eq!(gesture, Some(ActiveGesture::Pan));
        assert_eq!(ed.active_gesture(), None);
        assert_view_centred(&ed, target);
        assert!(approx_eq(ed.camera().zoom(), 1.0, 1e-6));
    }

    #[test]
    fn minimap_drag_moves_the_view_continuously() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);
        let map = shown(&ed);

        // Act / Assert: every step centres the view under the pointer.
        down(&mut ed, PointerButton::Left, 650.0, 500.0);
        assert_view_centred(&ed, map.world_at(Vec2::new(650.0, 500.0)));
        for (x, y) in [(660.0, 505.0), (700.0, 530.0), (720.0, 470.0)] {
            assert!(move_to(&mut ed, x, y));
            assert_view_centred(&ed, map.world_at(Vec2::new(x, y)));
        }
        // Outside the panel the pointer is clamped to the drawing area.
        move_to(&mut ed, 10.0, 10.0);
        assert_view_centred(&ed, map.world_at(map.area().min));
        up(&mut ed, PointerButton::Left, 700.0, 500.0);
        assert_view_centred(&ed, map.world_at(Vec2::new(700.0, 500.0)));
    }

    #[test]
    fn minimap_drag_keeps_the_world_box_of_its_press() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);
        let map = shown(&ed);
        let corner = map.area().max;

        // Act: centring on the content's corner puts the view outside it.
        down(&mut ed, PointerButton::Left, corner.x, corner.y);
        move_to(&mut ed, corner.x, corner.y);
        let during = shown(&ed).world();
        up(&mut ed, PointerButton::Left, corner.x, corner.y);
        let after = shown(&ed).world();

        // Assert: frozen during the drag, live again after it.
        assert_eq!(during, map.world());
        assert!(after.max.x > during.max.x + 100.0, "{after:?}");
        assert_view_centred(&ed, map.world().max);
    }

    #[test]
    fn minimap_click_never_reaches_the_tool() {
        for tool in [
            Tool::Pen,
            Tool::Rect,
            Tool::Select,
            Tool::Eraser,
            Tool::Text,
        ] {
            // Arrange
            let mut ed = minimap_editor();
            ed.apply(Command::SetTool(tool));
            let before = ed.document().clone();
            key(&mut ed, Key::Space);

            // Act
            down(&mut ed, PointerButton::Left, 620.0, 460.0);
            move_to(&mut ed, 700.0, 520.0);
            up(&mut ed, PointerButton::Left, 700.0, 520.0);
            release_space(&mut ed);

            // Assert
            assert_eq!(*ed.document(), before, "{tool:?}");
            assert!(ed.selection().is_empty(), "{tool:?}");
            assert!(ed.text_edit().is_none(), "{tool:?}");
            assert_eq!(ed.document_revision(), 0, "{tool:?}");
            assert_eq!(ed.tool(), tool);
        }
    }

    #[test]
    fn minimap_press_without_space_goes_to_the_tool() {
        // Arrange
        let mut ed = minimap_editor();
        let camera = *ed.camera();

        // Act: a pen drag where the panel would be.
        down(&mut ed, PointerButton::Left, 620.0, 460.0);
        let gesture = ed.active_gesture();
        move_to(&mut ed, 700.0, 520.0);
        up(&mut ed, PointerButton::Left, 700.0, 520.0);

        // Assert
        assert_eq!(gesture, Some(ActiveGesture::Tool(Tool::Pen)));
        assert_eq!(ed.document().len(), 3);
        assert_eq!(*ed.camera(), camera);
    }

    #[test]
    fn space_drag_outside_the_minimap_pans() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);

        // Act: the drag starts outside and ends over the panel.
        down(&mut ed, PointerButton::Left, 100.0, 100.0);
        move_to(&mut ed, 150.0, 130.0);
        move_to(&mut ed, 700.0, 500.0);
        up(&mut ed, PointerButton::Left, 700.0, 500.0);

        // Assert: content moved by the pointer delta.
        assert!(origin_on_screen(&ed).approx_eq(Vec2::new(600.0, 400.0), 1e-3));
        assert_eq!(ed.document().len(), 2);
    }

    #[test]
    fn minimap_drag_ends_after_space_is_released() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);
        let map = shown(&ed);
        down(&mut ed, PointerButton::Left, 650.0, 500.0);

        // Act: Space goes up before the button.
        let released = release_space(&mut ed);
        let still_shown = ed.minimap().is_some();
        move_to(&mut ed, 700.0, 520.0);
        let ended = up(&mut ed, PointerButton::Left, 700.0, 520.0);

        // Assert
        assert!(!released, "nothing changed on screen");
        assert!(still_shown);
        assert!(ended, "the minimap hides with the release");
        assert_view_centred(&ed, map.world_at(Vec2::new(700.0, 520.0)));
        assert!(ed.minimap().is_none());
        assert_eq!(ed.active_gesture(), None);
    }

    #[test]
    fn minimap_drag_survives_a_snapshot_round_trip() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);
        down(&mut ed, PointerButton::Left, 650.0, 500.0);

        // Act
        let text = ed.snapshot().to_text();
        let mut loaded = match Snapshot::parse(&text) {
            Ok(snapshot) => Editor::from_snapshot(snapshot),
            Err(err) => panic!("own snapshot rejected: {err}\n{text}"),
        };

        // Assert: the same state, and the drag goes on the same way.
        assert!(text.contains("gesture minimap left "), "{text}");
        assert_eq!(loaded, ed);
        assert_eq!(loaded.minimap(), ed.minimap());
        move_to(&mut ed, 720.0, 470.0);
        move_to(&mut loaded, 720.0, 470.0);
        assert_eq!(loaded, ed);
        assert_eq!(loaded.camera(), ed.camera());
    }

    #[test]
    fn minimap_drag_is_cancelled_by_undo() {
        // Arrange
        let mut ed = minimap_editor();
        key(&mut ed, Key::Space);
        down(&mut ed, PointerButton::Left, 650.0, 500.0);

        // Act
        key_with(&mut ed, Key::Z, CTRL);
        let camera = *ed.camera();
        move_to(&mut ed, 720.0, 470.0);

        // Assert: the gesture is gone, Space still shows the minimap.
        assert_eq!(ed.active_gesture(), None);
        assert_eq!(*ed.camera(), camera);
        assert!(ed.document().is_empty());
        assert!(shown(&ed).content().is_none());
    }

    #[test]
    fn minimap_hidden_computes_no_content_bounds() {
        // Arrange
        let mut ed = minimap_editor();

        // Act: draw, pan with the hand tool, zoom, undo.
        drag(&mut ed, 100.0, 100.0);
        ed.apply(Command::SetTool(Tool::Hand));
        drag(&mut ed, 200.0, 200.0);
        ed.handle(InputEvent::Scroll {
            pos: Vec2::new(10.0, 10.0),
            delta: 1.0,
        });
        key_with(&mut ed, Key::Z, CTRL);

        // Assert
        assert!(ed.minimap().is_none());
        assert!(ed.minimap_content.is_none());
    }

    #[test]
    fn minimap_content_bounds_follow_the_document_revision() {
        // Arrange: one pen stroke.
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        drag(&mut ed, 100.0, 100.0);
        let drawn = ed.document_revision();

        // Act: show, pan, then undo with Space held.
        key(&mut ed, Key::Space);
        let first = ed.minimap_content;
        drag(&mut ed, 300.0, 300.0);
        let panned = ed.minimap_content;
        key_with(&mut ed, Key::Z, CTRL);
        let undone = ed.minimap_content;

        // Assert
        let Some((revision, Some(bounds))) = first else {
            panic!("bounds are computed when the minimap shows: {first:?}");
        };
        assert_eq!(revision, drawn);
        assert_eq!(
            Some(bounds),
            minimap::content_bounds(ed_redone(&ed).document())
        );
        assert_eq!(panned, first, "panning computes nothing");
        assert_eq!(undone, Some((ed.document_revision(), None)));
        assert!(ed.document_revision() > drawn);
        assert!(shown(&ed).content().is_none());
    }

    /// A copy of `ed` with its last undone step redone.
    fn ed_redone(ed: &Editor) -> Editor {
        let mut copy = ed.clone();
        copy.apply(Command::Redo);
        copy
    }

    #[test]
    fn minimap_empty_document_has_no_content() {
        // Arrange
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });

        // Act
        key(&mut ed, Key::Space);
        let map = shown(&ed);

        // Assert: only the view, which fills the drawing area's height.
        assert!(map.content().is_none() && map.content_rect().is_none());
        assert_eq!(map.world(), view_rect(&ed));
        let frame = map.frame(view_rect(&ed));
        assert!(approx_eq(frame.height(), map.area().height(), 1e-3));
    }

    // ---- T111 command palette ----------------------------------------------

    /// Editor with a viewport and the palette open.
    fn palette_editor() -> Editor {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        assert!(key_with(&mut ed, Key::P, CTRL_SHIFT));
        ed
    }

    /// The descriptions of the palette's rows.
    fn palette_rows(ed: &Editor) -> Vec<&'static str> {
        ed.palette()
            .map(|p| p.rows().map(|(_, _, description)| description).collect())
            .unwrap_or_default()
    }

    #[test]
    fn palette_ctrl_shift_p_opens() {
        // Arrange
        let mut ed = Editor::new();
        assert!(ed.palette().is_none());

        // Act
        let redraw = key_with(&mut ed, Key::P, CTRL_SHIFT);

        // Assert
        assert!(redraw);
        assert!(ed.palette().is_some());
        assert_eq!(palette_rows(&ed).len(), keymap::BINDINGS.len());
    }

    #[test]
    fn palette_typing_filters_rows() {
        // Arrange
        let mut ed = palette_editor();

        // Act
        typed(&mut ed, "RECT");

        // Assert
        assert_eq!(ed.palette().map(CommandPalette::query), Some("RECT"));
        let rows = palette_rows(&ed);
        assert_eq!(rows.first(), Some(&"Rectangle"));
        assert!(rows.len() < keymap::BINDINGS.len());
        assert!(rows.iter().all(|d| d.to_lowercase().contains('r')));
    }

    #[test]
    fn palette_backspace_widens_the_filter() {
        // Arrange
        let mut ed = palette_editor();
        typed(&mut ed, "rectx");
        assert_eq!(palette_rows(&ed).len(), 0);

        // Act
        key(&mut ed, Key::Backspace);

        // Assert
        assert_eq!(palette_rows(&ed).first(), Some(&"Rectangle"));
    }

    #[test]
    fn palette_arrows_move_the_highlight() {
        // Arrange
        let mut ed = palette_editor();
        let at = |ed: &Editor| ed.palette().map(CommandPalette::selected);
        assert_eq!(at(&ed), Some(0));

        // Act / Assert
        key(&mut ed, Key::ArrowDown);
        key(&mut ed, Key::ArrowDown);
        assert_eq!(at(&ed), Some(2));
        key(&mut ed, Key::ArrowUp);
        assert_eq!(at(&ed), Some(1));
        for _ in 0..3 {
            key(&mut ed, Key::ArrowUp);
        }
        assert_eq!(at(&ed), Some(0));
        for _ in 0..keymap::BINDINGS.len() + 5 {
            key(&mut ed, Key::ArrowDown);
        }
        assert_eq!(at(&ed), Some(keymap::BINDINGS.len() - 1));
    }

    #[test]
    fn palette_enter_runs_the_highlighted_command() {
        // Arrange
        let mut ed = palette_editor();
        typed(&mut ed, "rect");

        // Act
        key(&mut ed, Key::Enter);

        // Assert
        assert!(ed.palette().is_none());
        assert_eq!(ed.tool(), Tool::Rect);
    }

    #[test]
    fn palette_enter_runs_the_row_the_arrows_picked() {
        // Arrange: the second row of the empty query.
        let mut ed = palette_editor();
        key(&mut ed, Key::ArrowDown);

        // Act
        key(&mut ed, Key::Enter);

        // Assert
        assert_eq!(
            Some(Command::SetTool(ed.tool())),
            keymap::BINDINGS.get(1).map(|&(_, command, _)| command)
        );
        assert!(ed.palette().is_none());
    }

    #[test]
    fn palette_enter_without_a_match_just_closes() {
        // Arrange
        let mut ed = palette_editor();
        typed(&mut ed, "zzzz");

        // Act
        key(&mut ed, Key::Enter);

        // Assert
        assert!(ed.palette().is_none());
        assert_eq!(ed.tool(), Tool::Pen);
    }

    #[test]
    fn palette_escape_closes_doing_nothing() {
        // Arrange
        let mut ed = palette_editor();
        typed(&mut ed, "rect");

        // Act
        key(&mut ed, Key::Escape);

        // Assert
        assert!(ed.palette().is_none());
        assert_eq!(ed.tool(), Tool::Pen);
    }

    #[test]
    fn palette_keys_and_pointer_do_not_reach_the_tools() {
        // Arrange
        let mut ed = palette_editor();
        let before = ed.snapshot();

        // Act: tool keys, a space, a drag and a wheel turn.
        for k in [Key::R, Key::Digit3, Key::Tab, Key::Space, Key::Delete] {
            key(&mut ed, k);
        }
        let pos = Vec2::new(100.0, 100.0);
        let end = Vec2::new(200.0, 160.0);
        ed.handle(InputEvent::PointerDown {
            pos,
            button: PointerButton::Left,
            mods: NONE,
        });
        ed.handle(InputEvent::PointerMove {
            pos: end,
            mods: NONE,
        });
        ed.handle(InputEvent::PointerUp {
            pos: end,
            button: PointerButton::Left,
            mods: NONE,
        });
        ed.handle(InputEvent::Scroll { pos, delta: 3.0 });

        // Assert
        assert!(ed.palette().is_some());
        assert!(ed.document().is_empty());
        assert_eq!(ed.tool(), Tool::Pen);
        assert!(ed.active_gesture().is_none());
        assert!(ed.toolbar_visible());
        assert_eq!(ed.snapshot(), before);
    }

    #[test]
    fn palette_ignores_commands_while_open() {
        // Arrange
        let mut ed = palette_editor();

        // Act: what a toolbar click would send.
        let changed = ed.apply(Command::SetTool(Tool::Rect));

        // Assert
        assert!(!changed);
        assert_eq!(ed.tool(), Tool::Pen);
        assert!(ed.palette().is_some());
    }

    #[test]
    fn palette_open_and_close_record_nothing() {
        // Arrange
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        let before = ed.snapshot();

        // Act
        key_with(&mut ed, Key::P, CTRL_SHIFT);
        typed(&mut ed, "undo");
        key(&mut ed, Key::Escape);
        key_with(&mut ed, Key::P, CTRL_SHIFT);
        key(&mut ed, Key::Escape);

        // Assert
        assert!(!ed.can_undo());
        assert_eq!(ed.snapshot(), before);
    }

    #[test]
    fn palette_opens_while_a_text_block_is_open() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "x");

        // Act
        key_with(&mut ed, Key::P, CTRL_SHIFT);

        // Assert: the block was committed and the palette is open.
        assert!(ed.palette().is_some());
        assert!(ed.text_edit().is_none());
        assert_eq!(texts(&ed), ["x"]);
    }

    #[test]
    fn palette_chord_is_not_a_binding() {
        assert_eq!(keymap::resolve(Key::P, CTRL_SHIFT), None);
    }

    // ---- T116 pages ------------------------------------------------------

    fn go(ed: &mut Editor, page: usize) -> bool {
        ed.apply(Command::GoToPage(page))
    }

    #[test]
    fn new_editor_is_on_page_one() {
        assert_eq!(Editor::new().page(), 0);
    }

    #[test]
    fn going_to_the_current_page_changes_nothing() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        let before = ed.clone();

        // Act
        let redraw = go(&mut ed, 0);

        // Assert
        assert!(!redraw);
        assert_eq!(ed, before);
        assert_eq!(ed.document_revision(), before.document_revision());
    }

    #[test]
    fn page_count_is_six() {
        assert_eq!(PAGE_COUNT, 6);
    }

    #[test]
    fn going_to_the_last_page_works() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);

        // Act
        let redraw = go(&mut ed, PAGE_COUNT - 1);

        // Assert
        assert!(redraw);
        assert_eq!(ed.page(), PAGE_COUNT - 1);
        assert!(!ed.page_has_shapes(PAGE_COUNT - 1));
        assert!(ed.page_has_shapes(0));
    }

    #[test]
    fn page_has_shapes_follows_the_documents() {
        // Arrange: a shape on page 1, page 3 untouched.
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);

        // Assert: current page, another page, out of range.
        assert!(ed.page_has_shapes(0));
        assert!(!ed.page_has_shapes(2));
        assert!(!ed.page_has_shapes(PAGE_COUNT));
        // A page that was left keeps its answer, and an emptied one flips.
        go(&mut ed, 2);
        assert!(ed.page_has_shapes(0));
        assert!(!ed.page_has_shapes(2));
        ed.apply(Command::SetTool(Tool::Rect));
        ed.apply(Command::SelectAll);
        go(&mut ed, 0);
        assert!(ed.page_has_shapes(0));
    }

    #[test]
    fn going_to_a_page_out_of_range_changes_nothing() {
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        let before = ed.clone();

        assert!(!go(&mut ed, PAGE_COUNT));
        assert_eq!(ed, before);
    }

    #[test]
    fn pages_keep_their_own_shapes_selection_and_view() {
        // Arrange: page 1 has a selected rectangle and a panned, zoomed view.
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        ed.selection = ids(&ed);
        ed.camera = Camera::new(Vec2::new(40.0, 30.0), 2.0);
        let first = ids(&ed);

        // Act: a new page is empty with the default view and no selection.
        assert!(go(&mut ed, 4));
        let fresh = (ed.document().len(), ed.selection().len(), ed.camera);
        let tx = tx_insert(
            &mut ed.doc,
            vec![rect(9.0, 9.0, 1.0, 1.0), rect(5.0, 5.0, 1.0, 1.0)],
        );
        assert!(ed.history.commit(&mut ed.doc, tx).is_ok());
        ed.camera = Camera::new(Vec2::new(-7.0, 8.0), 0.5);
        go(&mut ed, 0);

        // Assert
        assert_eq!(ed.page(), 0);
        assert_eq!(fresh, (0, 0, Camera::default()));
        assert_eq!(ids(&ed), first);
        assert_eq!(ed.selection(), first.as_slice());
        assert_eq!(ed.camera, Camera::new(Vec2::new(40.0, 30.0), 2.0));
        go(&mut ed, 4);
        assert_eq!(ed.document().len(), 2);
        assert_eq!(ed.camera, Camera::new(Vec2::new(-7.0, 8.0), 0.5));
    }

    #[test]
    fn undo_acts_on_the_current_page_only() {
        // Arrange: one rectangle on each of pages 1 and 2.
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        go(&mut ed, 1);
        let tx = tx_insert(&mut ed.doc, vec![rect(5.0, 5.0, 1.0, 1.0)]);
        assert!(ed.history.commit(&mut ed.doc, tx).is_ok());

        // Act: undo on page 2, then on page 1.
        assert!(ed.apply(Command::Undo));
        let second_empty = ed.document().is_empty();
        assert!(!ed.can_undo());
        go(&mut ed, 0);
        let first_kept = ed.document().len();
        assert!(ed.apply(Command::Undo));

        // Assert
        assert!(second_empty);
        assert_eq!(first_kept, 1);
        assert!(ed.document().is_empty());
        go(&mut ed, 1);
        assert!(ed.can_redo());
    }

    #[test]
    fn switching_pages_is_not_an_undo_step() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);

        // Act
        go(&mut ed, 2);
        go(&mut ed, 0);
        ed.apply(Command::Undo);

        // Assert: the one undo took back the rectangle, not a switch.
        assert!(ed.document().is_empty());
        assert!(!ed.can_undo());
        assert_eq!(ed.page(), 0);
    }

    #[test]
    fn switching_pages_cancels_the_gesture() {
        // Arrange
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Rect));
        down(&mut ed, PointerButton::Left, 10.0, 10.0);
        move_to(&mut ed, 60.0, 50.0);

        // Act
        go(&mut ed, 1);
        up(&mut ed, PointerButton::Left, 60.0, 50.0);

        // Assert
        assert_eq!(ed.active_gesture(), None);
        assert!(ed.document().is_empty());
        go(&mut ed, 0);
        assert!(ed.document().is_empty());
    }

    #[test]
    fn switching_pages_commits_the_open_text() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 100.0, "abc");

        // Act
        go(&mut ed, 1);

        // Assert
        assert!(ed.text_edit().is_none());
        assert_eq!(texts(&ed), Vec::<String>::new());
        go(&mut ed, 0);
        assert_eq!(texts(&ed), vec!["abc".to_owned()]);
    }

    #[test]
    fn switching_pages_bumps_the_document_revision() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        let before = ed.document_revision();

        // Act
        go(&mut ed, 3);

        // Assert: a full re-render, not an append on the cached layer.
        assert!(ed.document_revision() > before);
        assert_eq!(ed.document_base_revision(), ed.document_revision());
    }

    #[test]
    fn clear_all_clears_only_the_current_page() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        go(&mut ed, 1);
        let tx = tx_insert(&mut ed.doc, vec![rect(5.0, 5.0, 1.0, 1.0)]);
        assert!(ed.history.commit(&mut ed.doc, tx).is_ok());

        // Act
        ed.apply(Command::ClearAll);
        go(&mut ed, 0);

        // Assert
        assert_eq!(ed.document().len(), 1);
        go(&mut ed, 1);
        assert!(ed.document().is_empty());
    }

    #[test]
    fn copy_on_one_page_pastes_on_another() {
        // Arrange
        let mut ed = editor_with(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        ed.apply(Command::SelectAll);
        ed.apply(Command::Copy);

        // Act
        go(&mut ed, 2);
        ed.apply(Command::Paste);

        // Assert
        assert_eq!(ed.document().len(), 1);
        go(&mut ed, 0);
        assert_eq!(ed.document().len(), 1);
    }

    #[test]
    fn zero_based_toggled_while_away_keeps_the_counter_above_the_first_number() {
        // Arrange: page 2 is left while zero-based, with its counter at 0.
        let mut ed = Editor::new();
        ed.apply(Command::ToggleZeroBased);
        go(&mut ed, 1);
        ed.apply(Command::ToggleNumbering);
        draw_node_with(&mut ed, Tool::Rect);
        go(&mut ed, 0);

        // Act: back to one-based, then to page 2.
        ed.apply(Command::ToggleZeroBased);
        go(&mut ed, 1);

        // Assert
        assert!(ed.helpers().next_number >= ed.helpers().first_number());
    }

    fn draw_node_with(ed: &mut Editor, tool: Tool) {
        ed.apply(Command::SetTool(tool));
        draw_node(ed, 0.0);
    }

    #[test]
    fn numbering_counts_per_page() {
        // Arrange: two nodes on page 1.
        let mut ed = numbering_editor();
        draw_node(&mut ed, 0.0);
        draw_node(&mut ed, 100.0);

        // Act: a node on page 2, then back to page 1.
        go(&mut ed, 1);
        let fresh_counter = ed.helpers().next_number;
        draw_node(&mut ed, 0.0);
        let on_second = labels(&ed);
        go(&mut ed, 0);

        // Assert
        assert_eq!(fresh_counter, FIRST_NUMBER);
        assert_eq!(on_second, vec![Some(FIRST_NUMBER)]);
        assert_eq!(ed.helpers().next_number, FIRST_NUMBER + 2);
    }

    // ---- T113 shortcut sheet -------------------------------------------------

    /// Editor with a viewport and the shortcut sheet open.
    fn sheet_editor() -> Editor {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: Vec2::new(800.0, 600.0),
        });
        typed(&mut ed, "?");
        ed
    }

    #[test]
    fn sheet_question_mark_opens_on_the_first_page() {
        // Arrange
        let mut ed = Editor::new();
        assert!(ed.sheet().is_none());

        // Act
        let redraw = ed.handle(InputEvent::Char {
            ch: '?',
            mods: NONE,
        });

        // Assert
        assert!(redraw);
        assert_eq!(ed.sheet().map(sheet::Sheet::page), Some(0));
    }

    #[test]
    fn sheet_question_mark_and_escape_close() {
        // Arrange
        let mut ed = sheet_editor();

        // Act / Assert
        typed(&mut ed, "?");
        assert!(ed.sheet().is_none());
        typed(&mut ed, "?");
        assert!(ed.sheet().is_some());
        key(&mut ed, Key::Escape);
        assert!(ed.sheet().is_none());
    }

    #[test]
    fn sheet_question_mark_types_in_a_text_block() {
        // Arrange
        let mut ed = text_editor();
        place(&mut ed, 100.0, 50.0, "a");

        // Act
        typed(&mut ed, "?");

        // Assert
        assert!(ed.sheet().is_none());
        assert_eq!(
            ed.text_edit().map(|e| e.text().to_owned()),
            Some("a?".to_owned())
        );
    }

    #[test]
    fn sheet_does_not_open_over_the_palette() {
        // Arrange
        let mut ed = palette_editor();

        // Act
        typed(&mut ed, "?");

        // Assert: the `?` went into the palette query.
        assert!(ed.sheet().is_none());
        assert_eq!(ed.palette().map(CommandPalette::query), Some("?"));
    }

    #[test]
    fn sheet_palette_chord_is_ignored_while_open() {
        // Arrange
        let mut ed = sheet_editor();

        // Act
        key_with(&mut ed, Key::P, CTRL_SHIFT);

        // Assert
        assert!(ed.palette().is_none());
        assert!(ed.sheet().is_some());
    }

    #[test]
    fn sheet_arrows_wrap_pages() {
        // Arrange
        let mut ed = sheet_editor();
        let page = |ed: &Editor| ed.sheet().map(sheet::Sheet::page);

        // Act / Assert
        key(&mut ed, Key::ArrowRight);
        assert_eq!(page(&ed), Some(1));
        key(&mut ed, Key::ArrowLeft);
        key(&mut ed, Key::ArrowLeft);
        assert_eq!(page(&ed), Some(sheet::pages().len() - 1));
        key(&mut ed, Key::ArrowRight);
        assert_eq!(page(&ed), Some(0));
    }

    #[test]
    fn sheet_digits_jump_to_a_topic() {
        // Arrange
        let mut ed = sheet_editor();

        // Act
        key(&mut ed, Key::Digit3);

        // Assert
        let topic = ed.sheet().map(|s| s.current().topic);
        assert_eq!(topic, Some(keymap::Topic::Edit));
    }

    #[test]
    fn sheet_digit_without_topic_is_ignored() {
        // Arrange
        let mut ed = sheet_editor();
        key(&mut ed, Key::ArrowRight);

        // Act
        key(&mut ed, Key::Digit9);
        key(&mut ed, Key::Digit0);

        // Assert
        assert_eq!(ed.sheet().map(sheet::Sheet::page), Some(1));
    }

    #[test]
    fn sheet_keys_and_pointer_do_not_reach_the_tools() {
        // Arrange
        let mut ed = sheet_editor();
        let before = ed.snapshot();

        // Act
        for k in [Key::R, Key::Tab, Key::Space, Key::Delete, Key::Enter] {
            key(&mut ed, k);
        }
        let pos = Vec2::new(100.0, 100.0);
        let end = Vec2::new(200.0, 160.0);
        ed.handle(InputEvent::PointerDown {
            pos,
            button: PointerButton::Left,
            mods: NONE,
        });
        ed.handle(InputEvent::PointerMove {
            pos: end,
            mods: NONE,
        });
        ed.handle(InputEvent::PointerUp {
            pos: end,
            button: PointerButton::Left,
            mods: NONE,
        });
        ed.handle(InputEvent::Scroll { pos, delta: 3.0 });

        // Assert
        assert!(ed.sheet().is_some());
        assert!(ed.document().is_empty());
        assert_eq!(ed.tool(), Tool::Pen);
        assert!(ed.active_gesture().is_none());
        assert!(ed.toolbar_visible());
        assert_eq!(ed.snapshot(), before);
    }

    #[test]
    fn sheet_ignores_commands_while_open() {
        // Arrange
        let mut ed = sheet_editor();

        // Act
        let changed = ed.apply(Command::SetTool(Tool::Rect));

        // Assert
        assert!(!changed);
        assert_eq!(ed.tool(), Tool::Pen);
        assert!(ed.sheet().is_some());
    }

    #[test]
    fn sheet_open_and_close_record_nothing() {
        // Arrange
        let mut ed = Editor::new();
        let before = ed.snapshot();

        // Act
        typed(&mut ed, "?");
        key(&mut ed, Key::ArrowRight);
        key(&mut ed, Key::Digit4);
        key(&mut ed, Key::Escape);

        // Assert
        assert!(!ed.can_undo());
        assert_eq!(ed.snapshot(), before);
    }
}
