//! The states the user sees: what the machine is doing with the input.

use crate::core::editor::ActiveGesture;

/// The interaction region (ADR-T134-1). A modal overlay is a state that
/// captures input, not a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interaction {
    /// Nothing in progress.
    Idle,
    /// A pointer gesture is in progress.
    Gesture(ActiveGesture),
    /// A text block is open for typing.
    TextEdit,
    /// The command palette is open.
    Palette,
    /// The shortcut sheet is open.
    Sheet,
}

/// An [`Interaction`] without its data: one row of the routing table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionKind {
    /// [`Interaction::Idle`].
    Idle,
    /// [`Interaction::Gesture`].
    Gesture,
    /// [`Interaction::TextEdit`].
    TextEdit,
    /// [`Interaction::Palette`].
    Palette,
    /// [`Interaction::Sheet`].
    Sheet,
}

impl InteractionKind {
    /// Every kind, in table order.
    pub const ALL: [Self; 5] = [
        Self::Idle,
        Self::Gesture,
        Self::TextEdit,
        Self::Palette,
        Self::Sheet,
    ];

    /// The name used in the diagram.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Gesture => "Gesture",
            Self::TextEdit => "TextEdit",
            Self::Palette => "Palette",
            Self::Sheet => "Sheet",
        }
    }
}

impl Interaction {
    /// This state without its data.
    #[must_use]
    pub fn kind(self) -> InteractionKind {
        match self {
            Self::Idle => InteractionKind::Idle,
            Self::Gesture(_) => InteractionKind::Gesture,
            Self::TextEdit => InteractionKind::TextEdit,
            Self::Palette => InteractionKind::Palette,
            Self::Sheet => InteractionKind::Sheet,
        }
    }
}
