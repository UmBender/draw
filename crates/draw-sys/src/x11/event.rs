//! A safe decoder of the raw 192-byte Xlib `XEvent` (T175 AC-2, AC-4).
//!
//! Offsets are those of the LP64 Xlib structs (`XConfigureEvent`,
//! `XClientMessageEvent`); `xlib` asserts that `long` has 8 bytes.

/// Size of the Xlib `XEvent` union on LP64.
pub const EVENT_BYTES: usize = 192;

const KEY_PRESS: i32 = 2;
const KEY_RELEASE: i32 = 3;
const BUTTON_PRESS: i32 = 4;
const BUTTON_RELEASE: i32 = 5;
const MOTION_NOTIFY: i32 = 6;
const FOCUS_IN: i32 = 9;
const FOCUS_OUT: i32 = 10;
/// `GenericEvent`: its first field (the type) is 35.
pub const GENERIC_EVENT: i32 = 35;
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
    /// `KeyPress` or `KeyRelease`.
    Key {
        /// Pressed (`true`) or released.
        down: bool,
        /// The X keycode.
        keycode: u32,
        /// Modifier mask and keyboard group.
        state: u32,
    },
    /// `ButtonPress` or `ButtonRelease`.
    Button {
        /// Pressed (`true`) or released.
        down: bool,
        /// The X button number (4 to 7 are the wheel).
        button: u32,
        /// X in window pixels.
        x: i32,
        /// Y in window pixels.
        y: i32,
    },
    /// `MotionNotify`.
    Motion {
        /// X in window pixels.
        x: i32,
        /// Y in window pixels.
        y: i32,
    },
    /// `FocusIn` (`true`) or `FocusOut`.
    Focus(bool),
    /// `GenericEvent`: the extension opcode and event type; the data is
    /// fetched from the display.
    Generic {
        /// The extension's major opcode.
        extension: i32,
        /// The extension's event type.
        evtype: i32,
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

fn read_u32(raw: &[u8], at: usize) -> Option<u32> {
    let bytes = raw.get(at..at + 4)?;
    Some(u32::from_ne_bytes(bytes.try_into().ok()?))
}

fn decode_known(raw: &[u8], kind: i32) -> Option<XEvent> {
    match kind {
        KEY_PRESS | KEY_RELEASE => Some(XEvent::Key {
            down: kind == KEY_PRESS,
            keycode: read_u32(raw, 84)?,
            state: read_u32(raw, 80)?,
        }),
        BUTTON_PRESS | BUTTON_RELEASE => Some(XEvent::Button {
            down: kind == BUTTON_PRESS,
            button: read_u32(raw, 84)?,
            x: read_i32(raw, 64)?,
            y: read_i32(raw, 68)?,
        }),
        MOTION_NOTIFY => Some(XEvent::Motion {
            x: read_i32(raw, 64)?,
            y: read_i32(raw, 68)?,
        }),
        FOCUS_IN | FOCUS_OUT => Some(XEvent::Focus(kind == FOCUS_IN)),
        GENERIC_EVENT => Some(XEvent::Generic {
            extension: read_i32(raw, 32)?,
            evtype: read_i32(raw, 36)?,
        }),
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
