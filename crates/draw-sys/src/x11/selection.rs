//! The pure part of the X11 selections (T177): event decoders, the target
//! tables and the `INCR` sender and receiver. No `unsafe`, no server.

/// A payload above this many bytes is sent with `INCR`.
pub const INCR_THRESHOLD: usize = 256 * 1024;
/// The size of one `INCR` chunk.
pub const INCR_CHUNK: usize = 128 * 1024;
/// The most an incoming `INCR` transfer may hold.
pub const MAX_INCOMING: usize = 64 * 1024 * 1024;

/// What a selection request asks for, by the target atoms draw knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    /// `TARGETS`: the list of targets served.
    Targets,
    /// `UTF8_STRING`.
    Utf8,
    /// `STRING` (served as the same UTF-8 bytes).
    String,
    /// `TEXT` (served as the same UTF-8 bytes).
    Text,
    /// `text/plain;charset=utf-8`.
    PlainUtf8,
    /// `image/png`.
    Png,
    /// Any other target.
    Other,
}

/// What draw holds in a selection it owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// Text.
    Text(String),
    /// A PNG image.
    Png(Vec<u8>),
}

/// The answer to one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The targets served, for `TARGETS`.
    Targets(Vec<TargetKind>),
    /// The bytes, typed by the target they answer.
    Data(Vec<u8>, TargetKind),
    /// The target is not served.
    Refuse,
}

impl Payload {
    /// The answer to a request for `kind`.
    #[must_use]
    pub fn answer(&self, kind: TargetKind) -> Answer {
        match (self, kind) {
            (_, TargetKind::Targets) => Answer::Targets(match self {
                Self::Text(_) => vec![
                    TargetKind::Targets,
                    TargetKind::Utf8,
                    TargetKind::String,
                    TargetKind::Text,
                    TargetKind::PlainUtf8,
                ],
                Self::Png(_) => vec![TargetKind::Targets, TargetKind::Png],
            }),
            (
                Self::Text(_),
                TargetKind::Utf8 | TargetKind::String | TargetKind::Text | TargetKind::PlainUtf8,
            )
            | (Self::Png(_), TargetKind::Png) => Answer::Data(self.bytes().to_vec(), kind),
            _ => Answer::Refuse,
        }
    }

    /// The bytes of the payload.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Text(text) => text.as_bytes(),
            Self::Png(png) => png,
        }
    }
}

/// A `SelectionRequest`: another client asks for our selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Request {
    /// The window to answer.
    pub requestor: u64,
    /// The selection atom.
    pub selection: u64,
    /// The target atom.
    pub target: u64,
    /// The property to write the answer to (`0`: obsolete client).
    pub property: u64,
    /// The request time.
    pub time: u64,
}

/// The selection events the platform reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelEvent {
    /// `SelectionRequest`.
    Request(Request),
    /// `SelectionNotify`: the answer to our request; `property` is `0`
    /// when the owner refused.
    Notify {
        /// The selection atom.
        selection: u64,
        /// The target atom.
        target: u64,
        /// The property holding the data, or `0`.
        property: u64,
    },
    /// `SelectionClear`: we lost `selection`.
    Clear {
        /// The selection atom.
        selection: u64,
    },
    /// `PropertyNotify`.
    Property {
        /// The window of the property.
        window: u64,
        /// The property atom.
        atom: u64,
        /// Deleted (`true`) or a new value.
        deleted: bool,
    },
}

/// Decodes a selection or property event; `None` for any other event or a
/// short buffer. Never panics.
#[must_use]
pub fn decode_selection(raw: &[u8]) -> Option<SelEvent> {
    let kind = read_i32(raw, 0)?;
    match kind {
        SELECTION_REQUEST => Some(SelEvent::Request(Request {
            requestor: read_u64(raw, 40)?,
            selection: read_u64(raw, 48)?,
            target: read_u64(raw, 56)?,
            property: read_u64(raw, 64)?,
            time: read_u64(raw, 72)?,
        })),
        SELECTION_NOTIFY => Some(SelEvent::Notify {
            selection: read_u64(raw, 40)?,
            target: read_u64(raw, 48)?,
            property: read_u64(raw, 56)?,
        }),
        SELECTION_CLEAR => Some(SelEvent::Clear {
            selection: read_u64(raw, 40)?,
        }),
        PROPERTY_NOTIFY => Some(SelEvent::Property {
            window: read_u64(raw, 32)?,
            atom: read_u64(raw, 40)?,
            deleted: read_i32(raw, 56)? == 1,
        }),
        _ => None,
    }
}

const PROPERTY_NOTIFY: i32 = 28;
const SELECTION_CLEAR: i32 = 29;
const SELECTION_REQUEST: i32 = 30;
const SELECTION_NOTIFY: i32 = 31;

fn read_i32(raw: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_ne_bytes(raw.get(at..at + 4)?.try_into().ok()?))
}

fn read_u64(raw: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_ne_bytes(raw.get(at..at + 8)?.try_into().ok()?))
}

/// Sends a payload in chunks, then an empty chunk.
#[derive(Debug)]
pub struct IncrSender {
    data: Vec<u8>,
    sent: usize,
    finished: bool,
}

impl IncrSender {
    /// A sender of `data`.
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            sent: 0,
            finished: false,
        }
    }

    /// The next chunk: data chunks of at most [`INCR_CHUNK`], then one
    /// empty chunk, then `None`.
    pub fn next_chunk(&mut self) -> Option<&[u8]> {
        if self.finished {
            return None;
        }
        let start = self.sent;
        let end = (start + INCR_CHUNK).min(self.data.len());
        self.sent = end;
        if start == end {
            self.finished = true;
        }
        self.data.get(start..end)
    }
}

/// What a received chunk did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrStep {
    /// More chunks are due.
    More,
    /// The empty chunk arrived: the data is complete.
    Done,
    /// The data passed [`MAX_INCOMING`]: give up.
    Overflow,
}

/// Joins `INCR` chunks.
#[derive(Debug, Default)]
pub struct IncrReceiver {
    buf: Vec<u8>,
}

impl IncrReceiver {
    /// An empty receiver.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes one chunk.
    pub fn push(&mut self, chunk: &[u8]) -> IncrStep {
        if chunk.is_empty() {
            return IncrStep::Done;
        }
        if self.buf.len() + chunk.len() > MAX_INCOMING {
            return IncrStep::Overflow;
        }
        self.buf.extend_from_slice(chunk);
        IncrStep::More
    }

    /// The bytes received so far.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.buf
    }
}
