//! The gesture automata (ADR-T136-1): the gesture in progress is one
//! variant per tool, with its own stage, and ends in a commit or a cancel.

use crate::core::command::Tool;
use crate::core::editor::ActiveGesture;
use crate::core::geom::Aabb;
use crate::core::input::PointerButton;
use crate::core::tools::navigate::Pan;

/// How far a pressed tool gesture got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Pressed, no pointer motion yet.
    Down,
    /// The pointer moved since the press.
    Active,
}

/// How a gesture ends: the only two ways out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// The driving button was released: the tool commits.
    Commit,
    /// A command, a mode switch or Escape discarded it.
    Cancel,
}

/// A pressed tool: the driving button and the stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Press {
    /// Button whose release ends the gesture.
    pub button: PointerButton,
    /// How far the gesture got.
    pub stage: Stage,
}

impl Press {
    fn new(button: PointerButton) -> Self {
        Self {
            button,
            stage: Stage::Down,
        }
    }
}

/// The gesture in progress: one variant per tool, plus the two view
/// gestures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// Panning the view.
    Pan {
        /// Button whose release ends the gesture.
        button: PointerButton,
        /// Pan state.
        pan: Pan,
    },
    /// Moving the view by a press or drag in the minimap (ADR-T110-1).
    Minimap {
        /// Button whose release ends the gesture.
        button: PointerButton,
        /// The minimap's world box at the press, kept until the release.
        world: Aabb,
    },
    /// The pen tool.
    Pen(Press),
    /// The line tool.
    Line(Press),
    /// The arrow tool.
    Arrow(Press),
    /// The rectangle tool.
    Rect(Press),
    /// The ellipse tool.
    Ellipse(Press),
    /// The grid tool.
    Grid(Press),
    /// The text tool.
    Text(Press),
    /// The eraser (also a right drag with any tool).
    Eraser(Press),
    /// The bucket tool.
    Bucket(Press),
    /// The select tool (marquee, move and resize live in the tool).
    Select(Press),
    /// The hand tool.
    Hand(Press),
}

impl Gesture {
    /// A fresh press of `button` with `tool`.
    #[must_use]
    pub fn for_tool(button: PointerButton, tool: Tool) -> Self {
        let press = Press::new(button);
        match tool {
            Tool::Pen => Self::Pen(press),
            Tool::Line => Self::Line(press),
            Tool::Arrow => Self::Arrow(press),
            Tool::Rect => Self::Rect(press),
            Tool::Ellipse => Self::Ellipse(press),
            Tool::Grid => Self::Grid(press),
            Tool::Text => Self::Text(press),
            Tool::Eraser => Self::Eraser(press),
            Tool::Bucket => Self::Bucket(press),
            Tool::Select => Self::Select(press),
            Tool::Hand => Self::Hand(press),
        }
    }

    fn press(&self) -> Option<&Press> {
        match self {
            Self::Pan { .. } | Self::Minimap { .. } => None,
            Self::Pen(p)
            | Self::Line(p)
            | Self::Arrow(p)
            | Self::Rect(p)
            | Self::Ellipse(p)
            | Self::Grid(p)
            | Self::Text(p)
            | Self::Eraser(p)
            | Self::Bucket(p)
            | Self::Select(p)
            | Self::Hand(p) => Some(p),
        }
    }

    fn press_mut(&mut self) -> Option<&mut Press> {
        match self {
            Self::Pan { .. } | Self::Minimap { .. } => None,
            Self::Pen(p)
            | Self::Line(p)
            | Self::Arrow(p)
            | Self::Rect(p)
            | Self::Ellipse(p)
            | Self::Grid(p)
            | Self::Text(p)
            | Self::Eraser(p)
            | Self::Bucket(p)
            | Self::Select(p)
            | Self::Hand(p) => Some(p),
        }
    }

    /// The tool receiving the events, for a tool gesture.
    #[must_use]
    pub fn tool(&self) -> Option<Tool> {
        Some(match self {
            Self::Pan { .. } | Self::Minimap { .. } => return None,
            Self::Pen(_) => Tool::Pen,
            Self::Line(_) => Tool::Line,
            Self::Arrow(_) => Tool::Arrow,
            Self::Rect(_) => Tool::Rect,
            Self::Ellipse(_) => Tool::Ellipse,
            Self::Grid(_) => Tool::Grid,
            Self::Text(_) => Tool::Text,
            Self::Eraser(_) => Tool::Eraser,
            Self::Bucket(_) => Tool::Bucket,
            Self::Select(_) => Tool::Select,
            Self::Hand(_) => Tool::Hand,
        })
    }

    /// The button whose release ends the gesture.
    #[must_use]
    pub fn button(&self) -> PointerButton {
        match (self, self.press()) {
            (_, Some(press)) => press.button,
            (Self::Pan { button, .. } | Self::Minimap { button, .. }, None) => *button,
            // Unreachable by construction: only the two view gestures have
            // no press, and both matched above.
            (_, None) => PointerButton::Left,
        }
    }

    /// The stage of a tool gesture.
    #[must_use]
    pub fn stage(&self) -> Option<Stage> {
        self.press().map(|press| press.stage)
    }

    /// The gesture after pointer motion.
    #[must_use]
    pub fn moved(mut self) -> Self {
        if let Some(press) = self.press_mut() {
            press.stage = Stage::Active;
        }
        self
    }

    /// Whether a press of `button` replaces this gesture: a pen reports
    /// its tip just before a held barrel button, in the same spot, so a
    /// right or middle press takes over a left tool press that has not
    /// moved yet.
    #[must_use]
    pub fn yields_to(&self, button: PointerButton) -> bool {
        self.press().is_some_and(|press| {
            press.button == PointerButton::Left
                && press.stage == Stage::Down
                && button != PointerButton::Left
        })
    }

    /// How the release of `button` ends this gesture: a commit if it is the
    /// driving button, `None` otherwise.
    #[must_use]
    pub fn ends_on(&self, button: PointerButton) -> Option<End> {
        (self.button() == button).then_some(End::Commit)
    }

    /// The gesture as seen from outside.
    #[must_use]
    pub fn active(&self) -> ActiveGesture {
        match self.tool() {
            Some(tool) => ActiveGesture::Tool(tool),
            None => ActiveGesture::Pan,
        }
    }
}

#[cfg(test)]
mod tests;
