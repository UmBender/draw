//! The document file `Ctrl+S` writes and `draw file.draw` reads (T108,
//! ADR-T108-1).
//!
//! Unlike a [`crate::core::snapshot::Snapshot`] it holds the document only:
//! every page's shapes, locks and view. Every later build reads every
//! version from 1 up; `tests/files/v1.draw` must keep loading.

use std::fmt::Write as _;

use crate::core::editor::Editor;
use crate::core::snapshot::{Reader, SnapshotError, Writer};

/// First word of the first line of a document file.
pub const HEADER: &str = "draw-file";

/// The version this build writes and the newest it reads.
pub const VERSION: u32 = 1;

/// A validated document file: what [`Editor::file`] captures and
/// [`DocFile::parse`] accepts. Applying it cannot fail.
#[derive(Debug, Clone, PartialEq)]
pub struct DocFile(pub(crate) Editor);

impl DocFile {
    /// The canonical text of this file.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut w = Writer::default();
        w.out.push_str(HEADER);
        let _ = writeln!(w.out, " {VERSION}");
        self.0.write_file(&mut w);
        w.out
    }

    /// Reads a document file from `text`.
    ///
    /// # Errors
    ///
    /// A [`SnapshotError`] naming the first bad line: wrong header, a
    /// version this build does not read (older than 1 or newer than
    /// [`VERSION`]), a truncated or malformed file, or a page that breaks
    /// an invariant.
    pub fn parse(text: &str) -> Result<Self, SnapshotError> {
        let mut r = Reader::new(text);
        let mut header = r.record(HEADER)?;
        let version: u32 = header.number("version")?;
        if !(1..=VERSION).contains(&version) {
            return Err(header.error(format!(
                "unsupported file version {version} (this build reads up to version {VERSION})"
            )));
        }
        header.end()?;
        let editor = Editor::read_file(&mut r, version)?;
        r.finish()?;
        Ok(Self(editor))
    }
}
