//! The modal and view regions of the machine (ADR-T135-1): the overlays,
//! the pages, and the camera and selection of the current page.

use crate::core::camera::Camera;
use crate::core::command_palette::CommandPalette;
use crate::core::document::ShapeId;
use crate::core::editor::{PAGE_COUNT, Page};
use crate::core::sheet::Sheet;

/// Which modal overlay is open: an [`Interaction`](super::Interaction) that
/// captures every input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModalKind {
    /// The command palette.
    Palette,
    /// The shortcut sheet.
    Sheet,
}

impl ModalKind {
    /// Every modal state.
    pub const ALL: [Self; 2] = [Self::Palette, Self::Sheet];
}

/// The modal region: at most one overlay, so two are never open together
/// by construction.
#[derive(Debug, Clone, Default)]
pub enum Modal {
    /// No overlay.
    #[default]
    Closed,
    /// The command palette is open.
    Palette(CommandPalette),
    /// The shortcut sheet is open.
    Sheet(Sheet),
}

impl Modal {
    /// Opens the command palette, closing any other overlay.
    pub fn open_palette(&mut self) {
        *self = Self::Palette(CommandPalette::new());
    }

    /// Opens the shortcut sheet, closing any other overlay.
    pub fn open_sheet(&mut self) {
        *self = Self::Sheet(Sheet::new());
    }

    /// Closes the open overlay, if any.
    pub fn close(&mut self) {
        *self = Self::Closed;
    }

    /// The open overlay's kind, if any.
    #[must_use]
    pub fn kind(&self) -> Option<ModalKind> {
        match self {
            Self::Closed => None,
            Self::Palette(_) => Some(ModalKind::Palette),
            Self::Sheet(_) => Some(ModalKind::Sheet),
        }
    }

    /// The open palette, if that is the open overlay.
    #[must_use]
    pub fn palette(&self) -> Option<&CommandPalette> {
        match self {
            Self::Palette(p) => Some(p),
            Self::Closed | Self::Sheet(_) => None,
        }
    }

    /// The open sheet, if that is the open overlay.
    #[must_use]
    pub fn sheet(&self) -> Option<&Sheet> {
        match self {
            Self::Sheet(s) => Some(s),
            Self::Closed | Self::Palette(_) => None,
        }
    }

    pub(crate) fn palette_mut(&mut self) -> Option<&mut CommandPalette> {
        match self {
            Self::Palette(p) => Some(p),
            Self::Closed | Self::Sheet(_) => None,
        }
    }

    pub(crate) fn sheet_mut(&mut self) -> Option<&mut Sheet> {
        match self {
            Self::Sheet(s) => Some(s),
            Self::Closed | Self::Palette(_) => None,
        }
    }
}

/// The page region: the current page's index and the slots of the other
/// pages (ADR-T116-1). A slot of a page never touched is `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pages {
    pub(crate) current: usize,
    pub(crate) slots: Vec<Option<Box<Page>>>,
}

impl Pages {
    /// The first page current, every slot empty.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: 0,
            slots: vec![None; PAGE_COUNT],
        }
    }

    /// The current page's index.
    #[must_use]
    pub fn current(&self) -> usize {
        self.current
    }

    /// How many slots hold a page.
    #[must_use]
    pub fn stored(&self) -> usize {
        self.slots.iter().flatten().count()
    }
}

impl Default for Pages {
    fn default() -> Self {
        Self::new()
    }
}

/// The view region: the camera and the selection of the current page.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct View {
    pub(crate) camera: Camera,
    pub(crate) selection: Vec<ShapeId>,
}

impl View {
    /// The camera.
    #[must_use]
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// The selected shape ids.
    #[must_use]
    pub fn selection(&self) -> &[ShapeId] {
        &self.selection
    }
}
