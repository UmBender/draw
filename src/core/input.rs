//! Input model: pointer, key and scroll events in screen space.
//!
//! The shell translates window-system input into [`InputEvent`]s and feeds them
//! to [`Editor::handle`](crate::core::editor::Editor::handle). Positions are
//! screen pixels with the origin at the top-left of the viewport. Events may
//! carry non-finite numbers; the editor drops those at the boundary.

use crate::core::geom::Vec2;

/// Modifier keys held during an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    /// `Shift` is held.
    pub shift: bool,
    /// `Ctrl` is held.
    pub ctrl: bool,
    /// `Alt` is held.
    pub alt: bool,
}

impl Modifiers {
    /// No modifier held.
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
    };
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// Primary button: drives the active tool.
    Left,
    /// Middle button: pans from any tool.
    Middle,
    /// Secondary button: erases from any tool.
    Right,
}

/// A keyboard key the editor understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)] // letters and digits are self-describing
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    /// `Delete` (forward delete).
    Delete,
    /// `Backspace`.
    Backspace,
    /// `Esc`.
    Escape,
    /// Space bar; held, it turns left drags into pans.
    Space,
    /// `Tab`.
    Tab,
    /// `[`.
    BracketLeft,
    /// `]`.
    BracketRight,
}

/// One input event, positions in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    /// A button was pressed at `pos`.
    PointerDown {
        /// Pointer position.
        pos: Vec2,
        /// Pressed button.
        button: PointerButton,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// The pointer moved to `pos`.
    PointerMove {
        /// Pointer position.
        pos: Vec2,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// A button was released at `pos`.
    PointerUp {
        /// Pointer position.
        pos: Vec2,
        /// Released button.
        button: PointerButton,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// The wheel turned by `delta` notches with the pointer at `pos`
    /// (positive zooms in).
    Scroll {
        /// Pointer position.
        pos: Vec2,
        /// Wheel notches; fractional for touchpads.
        delta: f32,
    },
    /// A key was pressed (or auto-repeated).
    KeyDown {
        /// The key.
        key: Key,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// A key was released.
    KeyUp {
        /// The key.
        key: Key,
        /// Modifiers held.
        mods: Modifiers,
    },
    /// The viewport now measures `size` pixels.
    Resize {
        /// Width and height in pixels.
        size: Vec2,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_default_none() {
        // Arrange / Act
        let mods = Modifiers::default();

        // Assert
        assert!(!mods.shift && !mods.ctrl && !mods.alt);
    }

    #[test]
    fn modifiers_none_is_default() {
        assert_eq!(Modifiers::NONE, Modifiers::default());
    }
}
