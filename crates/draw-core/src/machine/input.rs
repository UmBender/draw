//! Everything that can happen to the machine, as one closed enum.

use std::time::Instant;

use crate::core::geom::Vec2;
use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};

/// The answer to an [`Effect`](super::Effect) that needed the outside world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reply {
    /// The shell finished the effect it was asked for (an export, a save).
    Done,
}

/// One input of the automaton.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    /// A pointer button went down.
    PointerDown {
        /// Pointer position in screen pixels.
        pos: Vec2,
        /// The button.
        button: PointerButton,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// The pointer moved.
    PointerMove {
        /// Pointer position in screen pixels.
        pos: Vec2,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// A pointer button went up.
    PointerUp {
        /// Pointer position in screen pixels.
        pos: Vec2,
        /// The button.
        button: PointerButton,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// The wheel turned.
    Scroll {
        /// Pointer position in screen pixels.
        pos: Vec2,
        /// Wheel notches.
        delta: f32,
    },
    /// A key went down.
    KeyDown {
        /// The key.
        key: Key,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// A key went up.
    KeyUp {
        /// The key.
        key: Key,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// The layout produced a character.
    Char {
        /// The character.
        ch: char,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// The pen pressure changed.
    Pressure {
        /// Tip pressure, or `None` for a mouse.
        pressure: Option<f32>,
    },
    /// Clipboard text for an open text block.
    Paste(String),
    /// Clipboard text to turn into a grid.
    PasteGrid(String),
    /// The viewport changed size.
    Resize {
        /// New size in pixels.
        size: Vec2,
    },
    /// The clock reached this instant (the only time source, T137).
    Tick(Instant),
    /// The answer to an effect.
    Reply(Reply),
}

/// An [`Input`] without its data: one column of the routing table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKind {
    /// [`Input::PointerDown`].
    PointerDown,
    /// [`Input::PointerMove`].
    PointerMove,
    /// [`Input::PointerUp`].
    PointerUp,
    /// [`Input::Scroll`].
    Scroll,
    /// [`Input::KeyDown`].
    KeyDown,
    /// [`Input::KeyUp`].
    KeyUp,
    /// [`Input::Char`].
    Char,
    /// [`Input::Pressure`].
    Pressure,
    /// [`Input::Paste`].
    Paste,
    /// [`Input::PasteGrid`].
    PasteGrid,
    /// [`Input::Resize`].
    Resize,
    /// [`Input::Tick`].
    Tick,
    /// [`Input::Reply`].
    Reply,
}

impl InputKind {
    /// Every kind, in table order.
    pub const ALL: [Self; 13] = [
        Self::PointerDown,
        Self::PointerMove,
        Self::PointerUp,
        Self::Scroll,
        Self::KeyDown,
        Self::KeyUp,
        Self::Char,
        Self::Pressure,
        Self::Paste,
        Self::PasteGrid,
        Self::Resize,
        Self::Tick,
        Self::Reply,
    ];
}

impl Input {
    /// The input for an editor event.
    #[must_use]
    pub fn from_event(event: InputEvent) -> Self {
        match event {
            InputEvent::PointerDown { pos, button, mods } => Self::PointerDown { pos, button, mods },
            InputEvent::PointerMove { pos, mods } => Self::PointerMove { pos, mods },
            InputEvent::PointerUp { pos, button, mods } => Self::PointerUp { pos, button, mods },
            InputEvent::Scroll { pos, delta } => Self::Scroll { pos, delta },
            InputEvent::KeyDown { key, mods } => Self::KeyDown { key, mods },
            InputEvent::KeyUp { key, mods } => Self::KeyUp { key, mods },
            InputEvent::Char { ch, mods } => Self::Char { ch, mods },
            InputEvent::Pressure { pressure } => Self::Pressure { pressure },
            InputEvent::Paste(text) => Self::Paste(text),
            InputEvent::PasteGrid(text) => Self::PasteGrid(text),
            InputEvent::Resize { size } => Self::Resize { size },
        }
    }

    /// The editor event for this input, or `None` for [`Input::Tick`] and
    /// [`Input::Reply`], which the editor does not know.
    #[must_use]
    pub fn to_event(&self) -> Option<InputEvent> {
        match self {
            Self::PointerDown { pos, button, mods } => Some(InputEvent::PointerDown {
                pos: *pos,
                button: *button,
                mods: *mods,
            }),
            Self::PointerMove { pos, mods } => Some(InputEvent::PointerMove {
                pos: *pos,
                mods: *mods,
            }),
            Self::PointerUp { pos, button, mods } => Some(InputEvent::PointerUp {
                pos: *pos,
                button: *button,
                mods: *mods,
            }),
            Self::Scroll { pos, delta } => Some(InputEvent::Scroll {
                pos: *pos,
                delta: *delta,
            }),
            Self::KeyDown { key, mods } => Some(InputEvent::KeyDown {
                key: *key,
                mods: *mods,
            }),
            Self::KeyUp { key, mods } => Some(InputEvent::KeyUp {
                key: *key,
                mods: *mods,
            }),
            Self::Char { ch, mods } => Some(InputEvent::Char {
                ch: *ch,
                mods: *mods,
            }),
            Self::Pressure { pressure } => Some(InputEvent::Pressure {
                pressure: *pressure,
            }),
            Self::Paste(text) => Some(InputEvent::Paste(text.clone())),
            Self::PasteGrid(text) => Some(InputEvent::PasteGrid(text.clone())),
            Self::Resize { size } => Some(InputEvent::Resize { size: *size }),
            Self::Tick(_) | Self::Reply(_) => None,
        }
    }

    /// This input without its data.
    #[must_use]
    pub fn kind(&self) -> InputKind {
        match self {
            Self::PointerDown { .. } => InputKind::PointerDown,
            Self::PointerMove { .. } => InputKind::PointerMove,
            Self::PointerUp { .. } => InputKind::PointerUp,
            Self::Scroll { .. } => InputKind::Scroll,
            Self::KeyDown { .. } => InputKind::KeyDown,
            Self::KeyUp { .. } => InputKind::KeyUp,
            Self::Char { .. } => InputKind::Char,
            Self::Pressure { .. } => InputKind::Pressure,
            Self::Paste(_) => InputKind::Paste,
            Self::PasteGrid(_) => InputKind::PasteGrid,
            Self::Resize { .. } => InputKind::Resize,
            Self::Tick(_) => InputKind::Tick,
            Self::Reply(_) => InputKind::Reply,
        }
    }
}
