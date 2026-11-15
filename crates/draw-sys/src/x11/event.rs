//! A safe decoder of the raw 192-byte Xlib `XEvent` (T175 AC-2, AC-4).
//!
//! Offsets are those of the LP64 Xlib structs (`XConfigureEvent`,
//! `XClientMessageEvent`); `xlib` asserts that `long` has 8 bytes.

/// Size of the Xlib `XEvent` union on LP64.
pub const EVENT_BYTES: usize = 192;

/// An X event reduced to what the window part reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XEvent {
    /// `ConfigureNotify`: the window now has this size in pixels.
    Configure {
        /// New width, `0` if the server sent a negative value.
        width: u32,
        /// New height, `0` if the server sent a negative value.
        height: u32,
    },
    /// `ClientMessage`.
    ClientMessage {
        /// The message type atom.
        message_type: u64,
        /// The first `long` of the data.
        first_long: u64,
    },
    /// Any other event, by its type code (`0` for a truncated event).
    Other(i32),
}

/// The atoms that make a `ClientMessage` a window-manager close request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseAtoms {
    /// `WM_PROTOCOLS`.
    pub protocols: u64,
    /// `WM_DELETE_WINDOW`.
    pub delete_window: u64,
}

/// Decodes one event. Never panics: a short or unknown event is
/// [`XEvent::Other`].
#[must_use]
pub fn decode(raw: &[u8]) -> XEvent {
    todo!("decode {raw:?}")
}

/// Whether `event` is the window manager asking to close the window.
#[must_use]
pub fn is_close(event: XEvent, atoms: CloseAtoms) -> bool {
    todo!("is_close {event:?} {atoms:?}")
}
