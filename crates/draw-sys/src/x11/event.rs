//! A safe decoder of the raw 192-byte Xlib `XEvent` (T175 AC-2, AC-4).
//!
//! Offsets are those of the LP64 Xlib structs (`XConfigureEvent`,
//! `XClientMessageEvent`); `xlib` asserts that `long` has 8 bytes.

/// Size of the Xlib `XEvent` union on LP64.
pub const EVENT_BYTES: usize = 192;

const CONFIGURE_NOTIFY: i32 = 22;
const CLIENT_MESSAGE: i32 = 33;

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

fn read_i32(raw: &[u8], at: usize) -> Option<i32> {
    let bytes = raw.get(at..at + 4)?;
    Some(i32::from_ne_bytes(bytes.try_into().ok()?))
}

fn read_u64(raw: &[u8], at: usize) -> Option<u64> {
    let bytes = raw.get(at..at + 8)?;
    Some(u64::from_ne_bytes(bytes.try_into().ok()?))
}

fn decode_known(raw: &[u8], kind: i32) -> Option<XEvent> {
    match kind {
        CONFIGURE_NOTIFY => Some(XEvent::Configure {
            width: u32::try_from(read_i32(raw, 56)?).unwrap_or(0),
            height: u32::try_from(read_i32(raw, 60)?).unwrap_or(0),
        }),
        CLIENT_MESSAGE => Some(XEvent::ClientMessage {
            message_type: read_u64(raw, 40)?,
            first_long: read_u64(raw, 56)?,
        }),
        _ => None,
    }
}

/// Decodes one event. Never panics: a short or unknown event is
/// [`XEvent::Other`].
#[must_use]
pub fn decode(raw: &[u8]) -> XEvent {
    let Some(kind) = read_i32(raw, 0) else {
        return XEvent::Other(0);
    };
    decode_known(raw, kind).unwrap_or(XEvent::Other(kind))
}

/// Whether `event` is the window manager asking to close the window.
#[must_use]
pub fn is_close(event: XEvent, atoms: CloseAtoms) -> bool {
    matches!(
        event,
        XEvent::ClientMessage { message_type, first_long }
            if message_type == atoms.protocols && first_long == atoms.delete_window
    )
}
