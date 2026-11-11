//! Editor commands triggered by keys and toolbar buttons.
//!
//! A [`Command`] is a discrete action with no position; the keymap and the
//! toolbar both produce them and
//! [`Editor::apply`](crate::core::editor::Editor::apply) executes them.

use crate::core::palette::ColorId;

/// A pointer tool: what a left-button drag does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tool {
    /// Freehand, smoothed strokes.
    #[default]
    Pen,
    /// Straight line.
    Line,
    /// Line with an arrow head.
    Arrow,
    /// Rectangle.
    Rect,
    /// Ellipse / circle.
    Ellipse,
    /// Table of `cols × rows` cells.
    Grid,
    /// Free-standing text blocks (ADR-T29-2).
    Text,
    /// Removes whole shapes it touches.
    Eraser,
    /// Fills the clicked rectangle or ellipse.
    Bucket,
    /// Select and move.
    Select,
    /// Pan the view.
    Hand,
}

impl Tool {
    /// Every tool once, in toolbar order.
    pub const ALL: [Self; 11] = [
        Self::Pen,
        Self::Line,
        Self::Arrow,
        Self::Rect,
        Self::Ellipse,
        Self::Grid,
        Self::Text,
        Self::Eraser,
        Self::Bucket,
        Self::Select,
        Self::Hand,
    ];
}

/// A side of a box, for [`Command::Align`]. The world is `y`-down: the
/// top is the smaller `y`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// Smallest `x`.
    Left,
    /// Largest `x`.
    Right,
    /// Smallest `y`.
    Top,
    /// Largest `y`.
    Bottom,
}

/// A direction along which shapes are spaced, for [`Command::Distribute`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Along `x`: gaps between right and left edges.
    Horizontal,
    /// Along `y`: gaps between bottom and top edges.
    Vertical,
}

/// A discrete editor action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    /// Switch the active tool (cancels the current gesture).
    SetTool(Tool),
    /// Pick the drawing colour.
    SetColor(ColorId),
    /// Next thicker stroke width.
    WidthUp,
    /// Next thinner stroke width.
    WidthDown,
    /// Cycle the anti-tremor level.
    CycleSmoothing,
    /// Undo the last action.
    Undo,
    /// Redo the last undone action.
    Redo,
    /// Copy the selection to the clipboard.
    Copy,
    /// Copy the selection, then delete it.
    Cut,
    /// Paste the clipboard at the cursor.
    Paste,
    /// Duplicate the selection with a small offset.
    Duplicate,
    /// Select every shape.
    SelectAll,
    /// Replace the only selected text block by a grid of its lines and
    /// words (ADR-T84-1).
    GridFromText,
    /// Replace the only selected text block by the graph or tree its edges
    /// describe (ADR-T87-1).
    GraphFromText,
    /// Grow the selection by 10 % about its centre.
    ScaleUp,
    /// Shrink the selection by 10 % about its centre.
    ScaleDown,
    /// Replace the only selected text block by a coordinate plane of the
    /// points it lists (ADR-T93-1).
    PlotFromText,
    /// Move the selection one step up the stacking order, past the nearest
    /// unselected shape above it that overlaps its bounds (ADR-T100-1).
    RaiseSelection,
    /// Move the selection one step down the stacking order, past the
    /// nearest unselected shape below it that overlaps its bounds
    /// (ADR-T100-1).
    LowerSelection,
    /// Move the selection to the top of the stacking order (ADR-T100-1).
    SelectionToTop,
    /// Move the selection to the bottom of the stacking order
    /// (ADR-T100-1).
    SelectionToBottom,
    /// Move each selected shape so its bounds touch that side of the
    /// selection's box (ADR-T101-1).
    Align(Side),
    /// Space three or more selected shapes with equal gaps along the axis,
    /// the outer two fixed (ADR-T101-1).
    Distribute(Axis),
    /// Delete the selected shapes.
    DeleteSelection,
    /// Remove every unlocked shape (undoable, ADR-T107-1).
    ClearAll,
    /// Lock the selected shapes and clear the selection (ADR-T107-2).
    LockSelection,
    /// Unlock every locked shape (ADR-T107-2).
    UnlockAll,
    /// Cancel the gesture, or clear the selection when idle.
    Cancel,
    /// Back to the origin at zoom 1.
    ResetView,
    /// Frame all content.
    FitView,
    /// Show page `n` (`0` is page 1, up to 8): its own shapes, selection,
    /// view and history (ADR-T116-1).
    GoToPage(usize),
    /// Show or hide the toolbar.
    ToggleToolbar,
    /// Ask the shell to write the view to a PNG file (ADR-T92-2).
    ExportView,
    /// Turn smart snapping (round, sizes, alignment) on or off.
    ToggleSmartSnap,
    /// Turn snapping to the world grid on or off.
    ToggleGridSnap,
    /// Turn auto-numbering of new rectangles and ellipses on or off.
    ToggleNumbering,
    /// Restart auto-numbering at the current first number.
    ResetNumbering,
    /// Switch the first number between 1 and 0 and restart the counter
    /// there (ADR-T86-1).
    ToggleZeroBased,
    /// Add this many columns to new grids (clamped, live during a drag), or
    /// to the grid that is the whole selection under the Select tool
    /// (ADR-T85-1).
    GridCols(i32),
    /// Add this many rows to new grids (clamped, live during a drag), or
    /// to the grid that is the whole selection under the Select tool
    /// (ADR-T85-1).
    GridRows(i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_default_is_pen() {
        assert_eq!(Tool::default(), Tool::Pen);
    }

    #[test]
    fn tool_all_lists_each_tool_once() {
        // Arrange
        let all = Tool::ALL;

        // Act
        let unique: std::collections::HashSet<Tool> = all.iter().copied().collect();

        // Assert
        assert_eq!(all.len(), 11);
        assert!(all.contains(&Tool::Grid));
        assert!(all.contains(&Tool::Text));
        assert_eq!(unique.len(), all.len());
    }
}
