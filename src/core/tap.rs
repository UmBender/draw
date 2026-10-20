#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::Vec2;
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use proptest::prelude::*;

    const NONE: Modifiers = Modifiers::NONE;
    const SLOP: f32 = 16.0;

    fn down_at(x: f32, y: f32, button: PointerButton) -> InputEvent {
        InputEvent::PointerDown {
            pos: Vec2::new(x, y),
            button,
            mods: NONE,
        }
    }

    fn move_at(x: f32, y: f32) -> InputEvent {
        InputEvent::PointerMove {
            pos: Vec2::new(x, y),
            mods: NONE,
        }
    }

    fn up_at(x: f32, y: f32, button: PointerButton) -> InputEvent {
        InputEvent::PointerUp {
            pos: Vec2::new(x, y),
            button,
            mods: NONE,
        }
    }

    /// Feeds `events` through `filter` and collects what comes out.
    fn run(filter: &mut TapFilter, events: &[InputEvent]) -> Vec<InputEvent> {
        events
            .iter()
            .flat_map(|&event| filter.filter(event).into_iter().flatten())
            .collect()
    }

    fn arb_coord() -> impl Strategy<Value = f32> {
        prop_oneof![
            8 => -100.0f32..100.0,
            1 => Just(f32::NAN),
            1 => Just(f32::INFINITY),
        ]
    }

    fn arb_button() -> impl Strategy<Value = PointerButton> {
        prop_oneof![
            Just(PointerButton::Left),
            Just(PointerButton::Middle),
            Just(PointerButton::Right),
        ]
    }

    fn arb_pos() -> impl Strategy<Value = Vec2> {
        (arb_coord(), arb_coord()).prop_map(|(x, y)| Vec2::new(x, y))
    }

    fn arb_event() -> impl Strategy<Value = InputEvent> {
        prop_oneof![
            1 => (arb_pos(), arb_button()).prop_map(|(pos, button)| InputEvent::PointerDown {
                pos,
                button,
                mods: NONE
            }),
            4 => arb_pos().prop_map(|pos| InputEvent::PointerMove { pos, mods: NONE }),
            1 => (arb_pos(), arb_button()).prop_map(|(pos, button)| InputEvent::PointerUp {
                pos,
                button,
                mods: NONE
            }),
            1 => arb_pos().prop_map(|pos| InputEvent::Scroll { pos, delta: 1.0 }),
            1 => Just(InputEvent::KeyDown {
                key: Key::ArrowRight,
                mods: NONE
            }),
            1 => Just(InputEvent::Char { ch: 'x', mods: NONE }),
        ]
    }

    /// The presses and releases of `events`, in order, with their buttons.
    fn buttons(events: &[InputEvent]) -> Vec<(bool, PointerButton)> {
        events
            .iter()
            .filter_map(|event| match *event {
                InputEvent::PointerDown { button, .. } => Some((true, button)),
                InputEvent::PointerUp { button, .. } => Some((false, button)),
                _ => None,
            })
            .collect()
    }

    // ---- AC-1 off ----------------------------------------------------------

    proptest! {
        #[test]
        fn tap_filter_off_passes_every_event_unchanged(
            slop in prop_oneof![
                Just(0.0f32),
                Just(-5.0),
                Just(f32::NAN),
                Just(f32::INFINITY),
                Just(f32::NEG_INFINITY),
            ],
            events in prop::collection::vec(arb_event(), 0..40),
        ) {
            // Arrange
            let mut filter = TapFilter::new(slop);

            // Act
            let out = run(&mut filter, &events);

            // Assert
            prop_assert!(filter.slop().abs() < f32::EPSILON);
            prop_assert_eq!(out, events);
        }
    }

    #[test]
    fn pen_tap_slop_is_positive_and_finite() {
        assert!(PEN_TAP_SLOP_PX.is_finite() && PEN_TAP_SLOP_PX > 0.0);
        assert!((TapFilter::new(PEN_TAP_SLOP_PX).slop() - PEN_TAP_SLOP_PX).abs() < 1e-6);
    }

    // ---- AC-1 on -----------------------------------------------------------

    #[test]
    fn tap_filter_tap_within_slop_is_a_click_at_the_press_point() {
        // Arrange
        let mut filter = TapFilter::new(SLOP);

        // Act
        let out = run(
            &mut filter,
            &[
                down_at(100.0, 100.0, PointerButton::Left),
                move_at(108.0, 104.0),
                move_at(110.0, 109.0),
                up_at(111.0, 110.0, PointerButton::Left),
            ],
        );

        // Assert
        assert_eq!(
            out,
            vec![
                down_at(100.0, 100.0, PointerButton::Left),
                up_at(100.0, 100.0, PointerButton::Left),
            ]
        );
    }

    #[test]
    fn tap_filter_moves_within_slop_are_held_back() {
        // Arrange
        let mut filter = TapFilter::new(SLOP);
        let pressed = run(&mut filter, &[down_at(0.0, 0.0, PointerButton::Left)]);

        // Act: 16 px straight down is still inside (the edge counts).
        let held = run(&mut filter, &[move_at(3.0, 4.0), move_at(0.0, 16.0)]);

        // Assert
        assert_eq!(pressed, vec![down_at(0.0, 0.0, PointerButton::Left)]);
        assert!(held.is_empty(), "{held:?}");
    }

    #[test]
    fn tap_filter_leaving_the_slop_flushes_the_last_held_move() {
        // Arrange
        let mut filter = TapFilter::new(SLOP);
        run(
            &mut filter,
            &[
                down_at(0.0, 0.0, PointerButton::Left),
                move_at(5.0, 0.0),
                move_at(10.0, 0.0),
            ],
        );

        // Act
        let out = run(&mut filter, &[move_at(20.0, 0.0)]);

        // Assert
        assert_eq!(out, vec![move_at(10.0, 0.0), move_at(20.0, 0.0)]);
    }

    #[test]
    fn tap_filter_drag_passes_through_to_the_release() {
        // Arrange
        let mut filter = TapFilter::new(SLOP);
        run(
            &mut filter,
            &[down_at(0.0, 0.0, PointerButton::Left), move_at(30.0, 0.0)],
        );

        // Act: back inside the circle no longer holds anything.
        let dragged = run(
            &mut filter,
            &[move_at(2.0, 0.0), up_at(1.0, 0.0, PointerButton::Left)],
        );
        // A new press starts a new hold.
        let next = run(
            &mut filter,
            &[
                down_at(50.0, 50.0, PointerButton::Left),
                move_at(55.0, 50.0),
                up_at(56.0, 50.0, PointerButton::Left),
            ],
        );

        // Assert
        assert_eq!(
            dragged,
            vec![move_at(2.0, 0.0), up_at(1.0, 0.0, PointerButton::Left)]
        );
        assert_eq!(
            next,
            vec![
                down_at(50.0, 50.0, PointerButton::Left),
                up_at(50.0, 50.0, PointerButton::Left),
            ]
        );
    }

    #[test]
    fn tap_filter_other_button_during_hold_starts_the_drag() {
        // Arrange
        let mut filter = TapFilter::new(SLOP);
        run(
            &mut filter,
            &[down_at(0.0, 0.0, PointerButton::Left), move_at(4.0, 0.0)],
        );

        // Act
        let other = run(&mut filter, &[down_at(5.0, 0.0, PointerButton::Right)]);
        let rest = run(
            &mut filter,
            &[move_at(6.0, 0.0), up_at(6.0, 0.0, PointerButton::Left)],
        );

        // Assert
        assert_eq!(
            other,
            vec![move_at(4.0, 0.0), down_at(5.0, 0.0, PointerButton::Right)]
        );
        assert_eq!(
            rest,
            vec![move_at(6.0, 0.0), up_at(6.0, 0.0, PointerButton::Left)]
        );
    }

    #[test]
    fn tap_filter_keys_and_non_finite_events_pass_without_changing_state() {
        // Arrange
        let mut filter = TapFilter::new(SLOP);
        run(
            &mut filter,
            &[down_at(0.0, 0.0, PointerButton::Left), move_at(4.0, 0.0)],
        );
        let others = [
            InputEvent::KeyDown {
                key: Key::ArrowRight,
                mods: NONE,
            },
            InputEvent::KeyUp {
                key: Key::ArrowRight,
                mods: NONE,
            },
            InputEvent::Char {
                ch: 'a',
                mods: NONE,
            },
            InputEvent::Scroll {
                pos: Vec2::new(4.0, 0.0),
                delta: 1.0,
            },
            InputEvent::Resize {
                size: Vec2::new(800.0, 600.0),
            },
            move_at(f32::NAN, 0.0),
            up_at(f32::INFINITY, 0.0, PointerButton::Left),
            down_at(0.0, f32::NAN, PointerButton::Right),
        ];

        // Act
        let passed = run(&mut filter, &others);
        let tap = run(&mut filter, &[up_at(9.0, 0.0, PointerButton::Left)]);

        // Assert: still holding, so the release is a click.
        assert_eq!(passed.len(), others.len());
        for (a, b) in passed.iter().zip(&others) {
            // `assert_eq!` would fail on NaN; compare the debug text.
            assert_eq!(format!("{a:?}"), format!("{b:?}"));
        }
        assert_eq!(tap, vec![up_at(0.0, 0.0, PointerButton::Left)]);
    }

    proptest! {
        #[test]
        fn tap_filter_keeps_presses_and_releases_in_order(
            events in prop::collection::vec(arb_event(), 0..60),
        ) {
            // Arrange
            let mut filter = TapFilter::new(SLOP);

            // Act
            let out = run(&mut filter, &events);

            // Assert: same presses and releases, in the same order; nothing
            // but held moves is dropped.
            prop_assert_eq!(buttons(&out), buttons(&events));
            let not_moves = |list: &[InputEvent]| {
                list.iter()
                    .filter(|e| !matches!(e, InputEvent::PointerMove { .. }))
                    .count()
            };
            prop_assert_eq!(not_moves(&out), not_moves(&events));
            prop_assert!(out.len() <= events.len());
        }
    }
}
