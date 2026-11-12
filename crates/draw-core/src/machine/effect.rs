//! What the machine asks the outside world to do, as data.

use std::time::Instant;

use crate::core::geom::Aabb;

/// What to do with the system clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardOp {
    /// Put this text on the clipboard.
    Write(String),
    /// Read the clipboard; the answer comes back as an input.
    Read,
}

/// One effect of a step; the shell interprets them (ADR-T134-1).
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// The view changed: draw it again.
    Redraw,
    /// Use the system clipboard.
    Clipboard(ClipboardOp),
    /// Save the document (`Ctrl+S`).
    Save,
    /// Export the view to a PNG (`Ctrl+E`).
    ExportPng,
    /// Export the document to an SVG (`Ctrl+Shift+E`).
    ExportSvg,
    /// Copy this world box as an image (`Ctrl+Shift+C`).
    CopyImage(Aabb),
    /// A key refused to act: say why in the log.
    Notify(String),
    /// Send a `Tick` at this instant.
    Schedule(Instant),
}
