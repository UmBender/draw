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
    pub const ALL: [Self; 9] = [
        Self::Pen,
        Self::Line,
        Self::Arrow,
        Self::Rect,
        Self::Ellipse,
        Self::Eraser,
        Self::Bucket,
        Self::Select,
        Self::Hand,
    ];
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
    /// Delete the selected shapes.
    DeleteSelection,
    /// Remove every shape (undoable).
    ClearAll,
    /// Cancel the gesture, or clear the selection when idle.
    Cancel,
    /// Back to the origin at zoom 1.
    ResetView,
    /// Frame all content.
    FitView,
    /// Show or hide the toolbar.
    ToggleToolbar,
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
        assert_eq!(all.len(), 9);
        assert_eq!(unique.len(), all.len());
    }
}
