//! The select tool's states (ADR-T153-1): idle, or one of three gestures.
//! Positions are screen pixels. Every transition is a `match` on all four
//! variants.

use crate::core::document::ShapeId;
use crate::core::geom::Vec2;

/// Smallest factor a resize ever applies: keeps every factor positive on
/// the axes that set no limit of their own.
pub(crate) const MIN_FACTOR: f32 = 1e-3;

/// What a resize scales, worked out once at the press (ADR-T91-1). World
/// units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeFrame {
    /// The corner that stays put.
    pub pivot: Vec2,
    /// The corner being dragged.
    pub corner: Vec2,
    /// Smallest factor per axis the selection allows.
    pub floor: Vec2,
}

/// The select tool's state: no gesture, or the gesture in progress.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Select {
    /// No gesture.
    #[default]
    Idle,
    /// Rubber-band selection started on empty space.
    Marquee {
        /// Press position.
        start: Vec2,
        /// Latest pointer position.
        current: Vec2,
        /// `Shift` was held: add to the selection instead of replacing it.
        additive: bool,
    },
    /// Dragging shapes started on a shape.
    Move {
        /// Press position.
        start: Vec2,
        /// Latest pointer position.
        current: Vec2,
        /// The shape under the press.
        clicked: ShapeId,
        /// Shapes being moved (the selection at press time).
        ids: Vec<ShapeId>,
        /// `Alt` was held: insert moved copies, keep the originals.
        duplicate: bool,
    },
    /// Dragging a corner handle of the selection.
    Resize {
        /// Press position.
        start: Vec2,
        /// Latest pointer position.
        current: Vec2,
        /// `Shift` is held: keep the proportions.
        proportional: bool,
        /// Shapes being resized (the selection at press time).
        ids: Vec<ShapeId>,
        /// Pivot, dragged corner and limits.
        frame: ResizeFrame,
    },
}

impl ResizeFrame {
    /// The factors per axis for the dragged corner moved by the world
    /// `delta`, or `None` if they are not finite. A side with no extent
    /// keeps factor 1; `proportional` gives both axes the larger factor.
    /// Never flips and never goes below the limits (ADR-T91-1).
    #[must_use]
    pub fn factors(&self, delta: Vec2, proportional: bool) -> Option<(f32, f32)> {
        if !delta.is_finite() {
            return None;
        }
        let extent = self.corner - self.pivot;
        let ratio = |side: f32, moved: f32| (side.abs() > 0.0).then(|| (side + moved) / side);
        let (rx, ry) = (ratio(extent.x, delta.x), ratio(extent.y, delta.y));
        if rx.is_some_and(|r| !r.is_finite()) || ry.is_some_and(|r| !r.is_finite()) {
            return None;
        }
        if proportional {
            let larger = match (rx, ry) {
                (Some(x), Some(y)) => x.max(y),
                (Some(r), None) | (None, Some(r)) => r,
                (None, None) => return None,
            };
            let s = larger.max(self.floor.x.max(self.floor.y)).max(MIN_FACTOR);
            return Some((s, s));
        }
        let clamp = |r: Option<f32>, floor: f32| r.map_or(1.0, |r| r.max(floor).max(MIN_FACTOR));
        Some((clamp(rx, self.floor.x), clamp(ry, self.floor.y)))
    }
}

impl Select {
    /// Moves the pointer of the gesture, with `shift` as now held. Returns
    /// whether anything it draws changed.
    pub fn drag(&mut self, pos: Vec2, shift: bool) -> bool {
        match self {
            Self::Idle => false,
            Self::Marquee { current, .. } | Self::Move { current, .. } => {
                let moved = current.distance(pos) > 0.0;
                *current = pos;
                moved
            }
            Self::Resize {
                current,
                proportional,
                ..
            } => {
                let changed = current.distance(pos) > 0.0 || *proportional != shift;
                *current = pos;
                *proportional = shift;
                changed
            }
        }
    }

    /// Leaves the gesture, returning it and going `Idle`.
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    /// Discards the gesture. Returns whether there was one.
    pub fn cancel(&mut self) -> bool {
        match self.take() {
            Self::Idle => false,
            Self::Marquee { .. } | Self::Move { .. } | Self::Resize { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::ShapeId;
    use crate::core::geom::Vec2;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn frame() -> ResizeFrame {
        ResizeFrame {
            pivot: v(0.0, 0.0),
            corner: v(10.0, 10.0),
            floor: v(0.1, 0.1),
        }
    }

    fn marquee() -> Select {
        Select::Marquee {
            start: v(1.0, 1.0),
            current: v(1.0, 1.0),
            additive: false,
        }
    }

    fn moving() -> Select {
        Select::Move {
            start: v(1.0, 1.0),
            current: v(1.0, 1.0),
            clicked: ShapeId(1),
            ids: vec![ShapeId(1)],
            duplicate: false,
        }
    }

    fn resizing() -> Select {
        Select::Resize {
            start: v(1.0, 1.0),
            current: v(1.0, 1.0),
            proportional: false,
            ids: vec![ShapeId(1)],
            frame: frame(),
        }
    }

    #[test]
    fn select_default_is_idle() {
        assert_eq!(Select::default(), Select::Idle);
    }

    #[test]
    fn select_idle_drag_stays_idle() {
        let mut state = Select::Idle;
        assert!(!state.drag(v(5.0, 5.0), false));
        assert_eq!(state, Select::Idle);
    }

    #[test]
    fn select_marquee_drag_follows_pointer() {
        let mut state = marquee();
        assert!(state.drag(v(4.0, 6.0), false));
        assert!(matches!(state, Select::Marquee { current, .. } if current == v(4.0, 6.0)));
        assert!(!state.drag(v(4.0, 6.0), false), "same spot is no change");
    }

    #[test]
    fn select_move_drag_follows_pointer() {
        let mut state = moving();
        assert!(state.drag(v(9.0, 2.0), false));
        assert!(matches!(state, Select::Move { current, .. } if current == v(9.0, 2.0)));
    }

    #[test]
    fn select_resize_drag_updates_proportional() {
        let mut state = resizing();
        assert!(state.drag(v(1.0, 1.0), true), "shift alone is a change");
        assert!(matches!(
            state,
            Select::Resize {
                proportional: true,
                ..
            }
        ));
    }

    #[test]
    fn select_cancel_from_each_state_is_idle() {
        for (mut state, left) in [
            (Select::Idle, false),
            (marquee(), true),
            (moving(), true),
            (resizing(), true),
        ] {
            assert_eq!(state.cancel(), left);
            assert_eq!(state, Select::Idle);
        }
    }

    #[test]
    fn select_take_leaves_idle_and_returns_the_state() {
        let mut state = moving();
        assert_eq!(state.take(), moving());
        assert_eq!(state, Select::Idle);
    }
}
