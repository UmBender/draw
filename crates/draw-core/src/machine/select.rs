//! The select tool's states (ADR-T153-1). Tests first; the types follow.

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
