//! The pure decisions of clipboard transfers (T170, ADR-T170-1): which MIME
//! type to read, what a source offers and serves, and bounded reads and
//! writes on a pipe. No compositor and no `unsafe`.

use std::io::{self, Read, Write};

/// The most bytes a clipboard read takes.
pub const MAX_TRANSFER: usize = 64 * 1024 * 1024;

/// What the process puts on the clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// Plain text.
    Text(String),
    /// A PNG image.
    Png(Vec<u8>),
}

/// The text MIME type to read among `offered`, best first.
#[must_use]
pub fn pick_text_mime(offered: &[String]) -> Option<&str> {
    let _ = offered;
    todo!()
}

/// Whether `offered` has `image/png`.
#[must_use]
pub fn has_png(offered: &[String]) -> bool {
    let _ = offered;
    todo!()
}

/// The MIME types a source for `payload` offers.
#[must_use]
pub fn source_mimes(payload: &Payload) -> &'static [&'static str] {
    let _ = payload;
    todo!()
}

/// Whether a source for `payload` serves `mime`.
#[must_use]
pub fn serves(payload: &Payload, mime: &str) -> bool {
    let _ = (payload, mime);
    todo!()
}

/// The bytes a source sends for `payload`.
#[must_use]
pub fn payload_bytes(payload: &Payload) -> &[u8] {
    match payload {
        Payload::Text(text) => text.as_bytes(),
        Payload::Png(png) => png,
    }
}

/// Reads at most `max` bytes.
///
/// # Errors
/// The reader's error.
pub fn read_limited(reader: impl Read, max: usize) -> io::Result<Vec<u8>> {
    let _ = (reader, max);
    todo!()
}

/// Writes all of `data`; a reader that went away is not an error.
///
/// # Errors
/// Any error but a closed pipe.
pub fn write_payload(writer: impl Write, data: &[u8]) -> io::Result<()> {
    let _ = (writer, data);
    todo!()
}
