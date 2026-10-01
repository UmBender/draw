//! Tools: each module handles pointer gestures for one tool.
//!
//! Routing between tools is owned by T08; the tool-facing API is fixed by
//! ADR-T08-1 so tools can be built in parallel:
//!
//! - each tool module keeps its gesture state in its own `State` type, held by
//!   the editor in [`ToolStates`];
//! - tools see editor state through a [`ToolCtx`] (mutable) or a
//!   [`ToolView`] (read-only) and receive input as [`Pointer`]s;
//! - every tool exposes `on_pointer`, `preview` (an [`Overlay`]) and `cancel`.
//!
//! [`navigate`] (pan and zoom) is handled by the editor itself, not through
//! this API.

pub mod bucket;
pub mod eraser;
pub mod grid;
pub mod navigate;
pub mod pen;
pub mod select;
pub mod shape_tool;

use crate::core::camera::Camera;
use crate::core::clipboard::Clipboard;
use crate::core::command::Tool;
use crate::core::document::{Document, ShapeId, Transaction};
use crate::core::editor::DrawStyle;
use crate::core::geom::{Aabb, Vec2};
use crate::core::history::History;
use crate::core::input::Modifiers;
use crate::core::shape::Shape;
use crate::core::smoothing::SmoothingLevel;

/// Stage of a pointer gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// The button went down: the gesture starts.
    Down,
    /// The pointer moved while the button is held.
    Move,
    /// The button was released: the gesture ends.
    Up,
}

/// Pointer input as a tool receives it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pointer {
    /// Gesture stage.
    pub phase: Phase,
    /// Position in screen pixels; always finite.
    pub pos: Vec2,
    /// Modifiers held.
    pub mods: Modifiers,
}

/// What an in-progress gesture draws on top of the document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Overlay {
    /// Shapes drawn above the document (stroke being drawn, moved copies).
    pub shapes: Vec<Shape>,
    /// Document shapes not to draw (being erased or moved).
    pub hidden: Vec<ShapeId>,
    /// Selection marquee in world coordinates.
    pub marquee: Option<Aabb>,
    /// Alignment guides as world-space segments (ADR-T16-3).
    pub guides: Vec<[Vec2; 2]>,
}

impl Overlay {
    /// Whether there is nothing to draw or hide.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
            && self.hidden.is_empty()
            && self.marquee.is_none()
            && self.guides.is_empty()
    }
}

/// Mutable access to editor state for a tool handling input or a command.
#[derive(Debug)]
pub struct ToolCtx<'a> {
    /// The document; mutate it only through [`ToolCtx::commit`].
    pub doc: &'a mut Document,
    /// Undo history.
    pub history: &'a mut History,
    /// Selected shape ids, in selection order.
    pub selection: &'a mut Vec<ShapeId>,
    /// Internal clipboard.
    pub clipboard: &'a mut Clipboard,
    /// The camera (tools never move it).
    pub camera: &'a Camera,
    /// The tool the gesture belongs to (`Eraser` for a right drag).
    pub tool: Tool,
    /// Current drawing style.
    pub style: DrawStyle,
    /// Current anti-tremor level.
    pub smoothing: SmoothingLevel,
    /// Last known pointer position in screen pixels.
    pub cursor: Vec2,
}

impl ToolCtx<'_> {
    /// Applies `tx` to the document as one undo step. Returns whether the
    /// document changed (`false` for an empty or rejected transaction).
    pub fn commit(&mut self, tx: Transaction) -> bool {
        !tx.is_empty() && self.history.commit(self.doc, tx).is_ok()
    }

    /// Read-only view of the same state.
    #[must_use]
    pub fn view(&self) -> ToolView<'_> {
        ToolView {
            doc: self.doc,
            camera: self.camera,
            selection: self.selection,
            tool: self.tool,
            style: self.style,
            smoothing: self.smoothing,
            cursor: self.cursor,
        }
    }
}

/// Read-only access to editor state, for previews.
#[derive(Debug, Clone, Copy)]
pub struct ToolView<'a> {
    /// The document.
    pub doc: &'a Document,
    /// The camera.
    pub camera: &'a Camera,
    /// Selected shape ids.
    pub selection: &'a [ShapeId],
    /// The tool the gesture belongs to.
    pub tool: Tool,
    /// Current drawing style.
    pub style: DrawStyle,
    /// Current anti-tremor level.
    pub smoothing: SmoothingLevel,
    /// Last known pointer position in screen pixels.
    pub cursor: Vec2,
}

/// Gesture state of every pointer tool.
#[derive(Debug, Clone, Default)]
pub struct ToolStates {
    /// Pen.
    pub pen: pen::State,
    /// Line, arrow, rectangle and ellipse.
    pub shape: shape_tool::State,
    /// Grid.
    pub grid: grid::State,
    /// Eraser (also used by right drags).
    pub eraser: eraser::State,
    /// Bucket.
    pub bucket: bucket::State,
    /// Select / move.
    pub select: select::State,
}

impl ToolStates {
    /// Routes `pointer` to the tool `ctx.tool`. Returns whether a redraw is
    /// needed. [`Tool::Hand`] is handled by the editor and ignored here.
    pub fn on_pointer(&mut self, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
        match ctx.tool {
            Tool::Pen => pen::on_pointer(&mut self.pen, ctx, pointer),
            Tool::Line | Tool::Arrow | Tool::Rect | Tool::Ellipse => {
                shape_tool::on_pointer(&mut self.shape, ctx, pointer)
            }
            Tool::Grid => grid::on_pointer(&mut self.grid, ctx, pointer),
            Tool::Eraser => eraser::on_pointer(&mut self.eraser, ctx, pointer),
            Tool::Bucket => bucket::on_pointer(&mut self.bucket, ctx, pointer),
            Tool::Select => select::on_pointer(&mut self.select, ctx, pointer),
            Tool::Hand => false,
        }
    }

    /// The overlay of the gesture of `view.tool`.
    #[must_use]
    pub fn preview(&self, view: &ToolView<'_>) -> Overlay {
        match view.tool {
            Tool::Pen => pen::preview(&self.pen, view),
            Tool::Line | Tool::Arrow | Tool::Rect | Tool::Ellipse => {
                shape_tool::preview(&self.shape, view)
            }
            Tool::Grid => grid::preview(&self.grid, view),
            Tool::Eraser => eraser::preview(&self.eraser, view),
            Tool::Bucket => bucket::preview(&self.bucket, view),
            Tool::Select => select::preview(&self.select, view),
            Tool::Hand => Overlay::default(),
        }
    }

    /// Discards the gesture of `tool` without touching the document. Returns
    /// whether a redraw is needed.
    pub fn cancel(&mut self, tool: Tool) -> bool {
        match tool {
            Tool::Pen => pen::cancel(&mut self.pen),
            Tool::Line | Tool::Arrow | Tool::Rect | Tool::Ellipse => {
                shape_tool::cancel(&mut self.shape)
            }
            Tool::Grid => grid::cancel(&mut self.grid),
            Tool::Eraser => eraser::cancel(&mut self.eraser),
            Tool::Bucket => bucket::cancel(&mut self.bucket),
            Tool::Select => select::cancel(&mut self.select),
            Tool::Hand => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_with_guides_is_not_empty() {
        // Arrange
        let overlay = Overlay {
            guides: vec![[Vec2::ZERO, Vec2::new(10.0, 0.0)]],
            ..Overlay::default()
        };

        // Act / Assert
        assert!(!overlay.is_empty());
        assert!(Overlay::default().is_empty());
    }
}
