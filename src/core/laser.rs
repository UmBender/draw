//! The fading laser pointer (T106, ADR-T106-1, ADR-T106-2).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::{Command, Tool};
    use crate::core::editor::Editor;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};

    const EPS: f32 = 1e-5;
    const IDLE: EditorState = EditorState {
        editing: false,
        gesture: false,
    };

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn q_down() -> InputEvent {
        InputEvent::KeyDown {
            key: Key::Q,
            mods: Modifiers::NONE,
        }
    }

    fn q_up() -> InputEvent {
        InputEvent::KeyUp {
            key: Key::Q,
            mods: Modifiers::NONE,
        }
    }

    fn mv(x: f32, y: f32) -> InputEvent {
        InputEvent::PointerMove {
            pos: v(x, y),
            mods: Modifiers::NONE,
        }
    }

    fn down(x: f32, y: f32) -> InputEvent {
        InputEvent::PointerDown {
            pos: v(x, y),
            button: PointerButton::Left,
            mods: Modifiers::NONE,
        }
    }

    fn up(x: f32, y: f32) -> InputEvent {
        InputEvent::PointerUp {
            pos: v(x, y),
            button: PointerButton::Left,
            mods: Modifiers::NONE,
        }
    }

    /// A laser holding `Q` with points at `(10, 0)`, `(20, 0)`, … at times
    /// `0.0`, `0.1`, ….
    fn held_with(points: usize) -> Laser {
        let mut laser = Laser::new();
        laser.route(&q_down(), IDLE, 0.0);
        for i in 0..points {
            let x = 10.0 * (i + 1) as f32;
            laser.route(&mv(x, 0.0), IDLE, 0.1 * i as f64);
        }
        laser
    }

    /// Routes `event` through `laser` and hands it to `editor` unless the
    /// laser consumed it, as the shell does.
    fn feed(laser: &mut Laser, editor: &mut Editor, event: InputEvent, now: f64) {
        if laser.route(&event, EditorState::of(editor), now) == Routed::Forward {
            editor.handle(event);
        }
    }

    fn editor() -> Editor {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: v(800.0, 600.0),
        });
        ed
    }

    /// An editor with one line from `(100, 100)` to `(200, 150)`, selected.
    fn editor_with_selected_line() -> Editor {
        let mut ed = editor();
        ed.apply(Command::SetTool(Tool::Line));
        ed.handle(down(100.0, 100.0));
        ed.handle(mv(200.0, 150.0));
        ed.handle(up(200.0, 150.0));
        ed.apply(Command::SelectAll);
        assert_eq!(ed.selection().len(), 1);
        ed
    }

    // AC-1 ------------------------------------------------------------------

    #[test]
    fn route_q_down_holds_and_is_consumed() {
        // Arrange
        let mut laser = Laser::new();

        // Act
        let routed = laser.route(&q_down(), IDLE, 0.0);

        // Assert
        assert_eq!(routed, Routed::Consumed);
        assert!(laser.is_held());
        assert_eq!(laser.points().len(), 0);
    }

    #[test]
    fn route_move_while_held_adds_a_point() {
        // Arrange
        let mut laser = Laser::new();
        laser.route(&q_down(), IDLE, 0.0);

        // Act
        let routed = laser.route(&mv(30.0, 40.0), IDLE, 0.25);

        // Assert
        assert_eq!(routed, Routed::Consumed);
        let points: Vec<_> = laser.points().collect();
        assert_eq!(points.len(), 1);
        assert!(points[0].pos.approx_eq(v(30.0, 40.0), EPS));
        assert!((points[0].at - 0.25).abs() < 1e-9);
    }

    #[test]
    fn route_move_without_q_is_forwarded() {
        // Arrange
        let mut laser = Laser::new();

        // Act
        let routed = laser.route(&mv(30.0, 40.0), IDLE, 0.0);

        // Assert
        assert_eq!(routed, Routed::Forward);
        assert!(!laser.is_held());
        assert_eq!(laser.points().len(), 0);
    }

    #[test]
    fn route_q_up_stops_adding_points() {
        // Arrange
        let mut laser = held_with(2);

        // Act
        let released = laser.route(&q_up(), IDLE, 0.2);
        let moved = laser.route(&mv(99.0, 99.0), IDLE, 0.3);

        // Assert
        assert_eq!(released, Routed::Consumed);
        assert_eq!(moved, Routed::Forward);
        assert!(!laser.is_held());
        assert_eq!(laser.points().len(), 2);
    }

    #[test]
    fn route_presses_while_held_are_swallowed() {
        // Arrange
        let mut laser = held_with(0);

        // Act / Assert
        for button in [
            PointerButton::Left,
            PointerButton::Middle,
            PointerButton::Right,
        ] {
            let mods = Modifiers::NONE;
            let pos = v(5.0, 5.0);
            let pressed = laser.route(&InputEvent::PointerDown { pos, button, mods }, IDLE, 0.0);
            let released = laser.route(&InputEvent::PointerUp { pos, button, mods }, IDLE, 0.0);
            assert_eq!(pressed, Routed::Consumed, "{button:?} press");
            assert_eq!(released, Routed::Consumed, "{button:?} release");
        }
        assert_eq!(laser.points().len(), 0);
    }

    #[test]
    fn route_q_repeat_keeps_one_piece() {
        // Arrange
        let mut laser = held_with(1);

        // Act: auto-repeat sends more key-downs while held.
        let repeated = laser.route(&q_down(), IDLE, 0.05);
        laser.route(&mv(50.0, 0.0), IDLE, 0.1);

        // Assert
        assert_eq!(repeated, Routed::Consumed);
        let points: Vec<_> = laser.points().collect();
        assert!(points[1].joined, "a repeat must not start a new piece");
    }

    #[test]
    fn route_ctrl_or_alt_q_is_forwarded() {
        for mods in [
            Modifiers {
                ctrl: true,
                ..Modifiers::NONE
            },
            Modifiers {
                alt: true,
                ..Modifiers::NONE
            },
        ] {
            // Arrange
            let mut laser = Laser::new();

            // Act
            let routed = laser.route(&InputEvent::KeyDown { key: Key::Q, mods }, IDLE, 0.0);

            // Assert
            assert_eq!(routed, Routed::Forward, "{mods:?}");
            assert!(!laser.is_held(), "{mods:?}");
        }
    }

    #[test]
    fn route_shift_q_holds() {
        // Arrange
        let mut laser = Laser::new();
        let mods = Modifiers {
            shift: true,
            ..Modifiers::NONE
        };

        // Act
        let routed = laser.route(&InputEvent::KeyDown { key: Key::Q, mods }, IDLE, 0.0);

        // Assert
        assert_eq!(routed, Routed::Consumed);
        assert!(laser.is_held());
    }

    #[test]
    fn route_other_events_while_held_are_forwarded() {
        // Arrange
        let mut laser = held_with(0);
        let mods = Modifiers::NONE;
        let events = [
            InputEvent::KeyDown { key: Key::Z, mods },
            InputEvent::KeyUp { key: Key::Z, mods },
            InputEvent::Scroll {
                pos: v(1.0, 1.0),
                delta: 1.0,
            },
            InputEvent::Char { ch: 'a', mods },
            InputEvent::Pressure {
                pressure: Some(0.5),
            },
            InputEvent::Paste("x".to_owned()),
            InputEvent::PasteGrid("x".to_owned()),
            InputEvent::Resize {
                size: v(10.0, 10.0),
            },
        ];

        // Act / Assert
        for event in events {
            assert_eq!(laser.route(&event, IDLE, 0.0), Routed::Forward, "{event:?}");
        }
        assert!(laser.is_held());
    }

    #[test]
    fn route_q_in_text_edit_is_forwarded() {
        // Arrange
        let mut laser = Laser::new();
        let editing = EditorState {
            editing: true,
            gesture: false,
        };

        // Act
        let pressed = laser.route(&q_down(), editing, 0.0);
        let released = laser.route(&q_up(), editing, 0.0);

        // Assert
        assert_eq!(pressed, Routed::Forward);
        assert_eq!(released, Routed::Forward);
        assert!(!laser.is_held());
    }

    #[test]
    fn laser_in_front_of_editor_q_types_q_in_text_edit() {
        // Arrange
        let mut laser = Laser::new();
        let mut ed = editor();
        ed.apply(Command::SetTool(Tool::Text));
        ed.handle(down(100.0, 100.0));
        ed.handle(up(100.0, 100.0));
        assert!(ed.text_edit().is_some(), "block open");

        // Act: the layout sends the character before the key.
        let mods = Modifiers::NONE;
        feed(&mut laser, &mut ed, InputEvent::Char { ch: 'q', mods }, 0.0);
        feed(&mut laser, &mut ed, q_down(), 0.0);
        feed(&mut laser, &mut ed, mv(150.0, 150.0), 0.1);
        feed(&mut laser, &mut ed, q_up(), 0.2);

        // Assert
        let edit = ed.text_edit().expect("block still open");
        assert_eq!(edit.text(), "q");
        assert!(!laser.is_held());
        assert_eq!(laser.points().len(), 0);
    }

    #[test]
    fn laser_in_front_of_editor_keeps_tool_and_selection() {
        // Arrange
        let mut laser = Laser::new();
        let mut ed = editor_with_selected_line();
        ed.apply(Command::SetTool(Tool::Rect));
        let selection = ed.selection().to_vec();

        // Act: a drag with Q held would draw a rectangle without it.
        feed(&mut laser, &mut ed, q_down(), 0.0);
        feed(&mut laser, &mut ed, down(300.0, 300.0), 0.0);
        feed(&mut laser, &mut ed, mv(400.0, 380.0), 0.1);
        feed(&mut laser, &mut ed, up(400.0, 380.0), 0.2);
        feed(&mut laser, &mut ed, q_up(), 0.3);

        // Assert
        assert_eq!(ed.tool(), Tool::Rect);
        assert_eq!(ed.selection(), selection.as_slice());
        assert_eq!(ed.document().len(), 1);
        assert!(ed.active_gesture().is_none());
        assert_eq!(laser.points().len(), 1);
    }

    // AC-2 ------------------------------------------------------------------

    #[test]
    fn alpha_fades_linearly_over_the_fade_time() {
        assert!(approx_eq(alpha(0.0), 1.0, EPS));
        assert!(approx_eq(alpha(-1.0), 1.0, EPS));
        assert!(approx_eq(alpha(FADE_SECS * 0.25), 0.75, EPS));
        assert!(approx_eq(alpha(FADE_SECS * 0.5), 0.5, EPS));
        assert!(approx_eq(alpha(FADE_SECS), 0.0, EPS));
        assert!(approx_eq(alpha(FADE_SECS * 3.0), 0.0, EPS));
        assert!(approx_eq(alpha(f64::NAN), 0.0, EPS));
        assert!(approx_eq(alpha(f64::INFINITY), 0.0, EPS));
    }

    #[test]
    fn fade_width_and_buffer_are_as_recorded() {
        // ADR-T106-2.
        assert!((FADE_SECS - 1.0).abs() < 1e-12);
        assert!(approx_eq(WIDTH_PX, 6.0, EPS));
        assert_eq!(MAX_POINTS, 1024);
    }

    #[test]
    fn segments_join_points_of_one_hold() {
        // Arrange
        let laser = held_with(3);

        // Act
        let segments: Vec<_> = laser.segments(0.2).collect();

        // Assert: a dot at the first point, then two pieces.
        assert_eq!(segments.len(), 3);
        assert!(segments[0].from.approx_eq(v(10.0, 0.0), EPS));
        assert!(segments[0].to.approx_eq(v(10.0, 0.0), EPS));
        assert!(segments[1].from.approx_eq(v(10.0, 0.0), EPS));
        assert!(segments[1].to.approx_eq(v(20.0, 0.0), EPS));
        assert!(segments[2].from.approx_eq(v(20.0, 0.0), EPS));
        assert!(segments[2].to.approx_eq(v(30.0, 0.0), EPS));
    }

    #[test]
    fn segments_restart_after_release() {
        // Arrange
        let mut laser = held_with(2);
        laser.route(&q_up(), IDLE, 0.2);
        laser.route(&q_down(), IDLE, 0.3);
        laser.route(&mv(500.0, 500.0), IDLE, 0.3);

        // Act
        let segments: Vec<_> = laser.segments(0.3).collect();

        // Assert: the new point is a dot, not joined to (20, 0).
        let last = segments.last().expect("segments");
        assert!(last.from.approx_eq(v(500.0, 500.0), EPS));
        assert!(last.to.approx_eq(v(500.0, 500.0), EPS));
        assert!(!laser.points().last().expect("points").joined);
    }

    #[test]
    fn segment_takes_the_older_points_alpha() {
        // Arrange: points at 0.0 and 0.1.
        let laser = held_with(2);

        // Act
        let segments: Vec<_> = laser.segments(0.5).collect();

        // Assert
        assert!(approx_eq(segments[0].alpha, 0.5, EPS));
        assert!(approx_eq(segments[1].alpha, 0.5, EPS));
    }

    #[test]
    fn tick_drops_points_older_than_the_fade() {
        // Arrange: points at 0.0, 0.1 and 0.2.
        let mut laser = held_with(3);

        // Act
        laser.tick(1.15);

        // Assert
        let points: Vec<_> = laser.points().collect();
        assert_eq!(points.len(), 1);
        assert!(points[0].pos.approx_eq(v(30.0, 0.0), EPS));
        let segments: Vec<_> = laser.segments(1.15).collect();
        assert_eq!(segments.len(), 1, "the survivor shows as a dot");
        assert!(segments[0].from.approx_eq(v(30.0, 0.0), EPS));
    }

    #[test]
    fn trail_is_bounded() {
        // Arrange
        let mut laser = held_with(0);

        // Act: all at the same time, so none fades.
        for i in 0..MAX_POINTS + 10 {
            laser.route(&mv(i as f32, 0.0), IDLE, 0.0);
        }

        // Assert: the oldest went first.
        assert_eq!(laser.points().len(), MAX_POINTS);
        let first = laser.points().next().expect("points");
        assert!(first.pos.approx_eq(v(10.0, 0.0), EPS));
    }

    #[test]
    fn trail_stays_in_screen_space_when_zooming() {
        // Arrange
        let mut laser = Laser::new();
        let mut ed = editor();
        feed(&mut laser, &mut ed, q_down(), 0.0);
        feed(&mut laser, &mut ed, mv(100.0, 100.0), 0.0);

        // Act
        let zoom = InputEvent::Scroll {
            pos: v(400.0, 300.0),
            delta: 3.0,
        };
        feed(&mut laser, &mut ed, zoom, 0.1);

        // Assert: the editor zoomed, the point did not move.
        assert!(ed.camera().zoom() > 1.0);
        let point = laser.points().next().expect("point");
        assert!(point.pos.approx_eq(v(100.0, 100.0), EPS));
    }

    #[test]
    fn route_non_finite_position_or_time_adds_no_point() {
        // Arrange
        let mut laser = held_with(0);

        // Act
        let nan_pos = laser.route(&mv(f32::NAN, 0.0), IDLE, 0.0);
        let inf_pos = laser.route(&mv(0.0, f32::INFINITY), IDLE, 0.0);
        let nan_time = laser.route(&mv(1.0, 1.0), IDLE, f64::NAN);
        let inf_time = laser.route(&mv(1.0, 1.0), IDLE, f64::INFINITY);

        // Assert
        for routed in [nan_pos, inf_pos, nan_time, inf_time] {
            assert_eq!(routed, Routed::Consumed);
        }
        assert_eq!(laser.points().len(), 0);
    }

    // AC-3 ------------------------------------------------------------------

    #[test]
    fn laser_in_front_of_editor_changes_no_state() {
        // Arrange
        let mut laser = Laser::new();
        let mut ed = editor_with_selected_line();
        ed.apply(Command::SetTool(Tool::Pen));
        let before = ed.snapshot().to_text();
        let revision = ed.document_revision();

        // Act: hover, press, drag, release and right drag, all with Q held.
        feed(&mut laser, &mut ed, q_down(), 0.0);
        feed(&mut laser, &mut ed, mv(10.0, 10.0), 0.0);
        feed(&mut laser, &mut ed, down(150.0, 125.0), 0.05);
        for i in 0..20 {
            let x = 150.0 + 5.0 * i as f32;
            feed(&mut laser, &mut ed, mv(x, 125.0), 0.05 + 0.01 * f64::from(i));
        }
        feed(&mut laser, &mut ed, up(250.0, 125.0), 0.3);
        let right = |down: bool| {
            let (pos, button, mods) = (v(150.0, 125.0), PointerButton::Right, Modifiers::NONE);
            if down {
                InputEvent::PointerDown { pos, button, mods }
            } else {
                InputEvent::PointerUp { pos, button, mods }
            }
        };
        feed(&mut laser, &mut ed, right(true), 0.31);
        feed(&mut laser, &mut ed, mv(120.0, 110.0), 0.32);
        feed(&mut laser, &mut ed, right(false), 0.33);
        feed(&mut laser, &mut ed, q_up(), 0.4);

        // Assert
        assert_eq!(ed.snapshot().to_text(), before);
        assert_eq!(ed.document_revision(), revision);
        assert!(laser.points().len() > 20);
    }

    // AC-4 ------------------------------------------------------------------

    #[test]
    fn tick_redraws_while_points_are_alive() {
        // Arrange
        let mut laser = held_with(1);

        // Act
        let redraw = laser.tick(0.5);

        // Assert
        assert!(redraw);
        assert!(laser.is_fading());
    }

    #[test]
    fn tick_after_the_fade_redraws_once_then_idles() {
        // Arrange
        let mut laser = held_with(1);

        // Act
        let last = laser.tick(FADE_SECS);
        let fading = laser.is_fading();
        let after = laser.tick(FADE_SECS + 0.5);

        // Assert: one frame erases the trail, then nothing.
        assert!(last, "the frame that erases the trail");
        assert!(!fading);
        assert!(!after);
        assert!(!laser.is_fading());
        assert_eq!(laser.segments(FADE_SECS + 0.5).count(), 0);
    }

    #[test]
    fn empty_trail_never_asks_for_frames() {
        // Arrange: held, but nothing moved.
        let mut laser = held_with(0);

        // Act / Assert
        assert!(!laser.tick(0.0));
        assert!(!laser.is_fading());
        assert!(!Laser::new().tick(5.0));
    }

    #[test]
    fn tick_non_finite_time_clears_the_trail() {
        // Arrange
        let mut laser = held_with(2);

        // Act
        let redraw = laser.tick(f64::NAN);

        // Assert
        assert!(redraw);
        assert!(!laser.is_fading());
        assert_eq!(laser.points().len(), 0);
    }

    // AC-5 ------------------------------------------------------------------

    #[test]
    fn route_during_a_gesture_forwards_pointer_events() {
        // Arrange
        let mut laser = held_with(0);
        let busy = EditorState {
            editing: false,
            gesture: true,
        };

        // Act / Assert
        assert_eq!(laser.route(&mv(1.0, 1.0), busy, 0.0), Routed::Forward);
        assert_eq!(laser.route(&up(1.0, 1.0), busy, 0.0), Routed::Forward);
        assert_eq!(laser.points().len(), 0);
        assert!(laser.is_held());
    }

    #[test]
    fn route_first_move_after_a_gesture_starts_a_new_piece() {
        // Arrange: a trail, then a gesture while still held.
        let mut laser = held_with(2);
        let busy = EditorState {
            editing: false,
            gesture: true,
        };
        laser.route(&mv(300.0, 0.0), busy, 0.2);
        laser.route(&up(300.0, 0.0), busy, 0.2);

        // Act
        laser.route(&mv(310.0, 0.0), IDLE, 0.3);

        // Assert
        let last = laser.points().last().expect("points");
        assert!(last.pos.approx_eq(v(310.0, 0.0), EPS));
        assert!(!last.joined);
    }

    #[test]
    fn laser_in_front_of_editor_q_mid_stroke_finishes_the_stroke() {
        // Arrange: a pen stroke in progress.
        let mut laser = Laser::new();
        let mut ed = editor();
        ed.apply(Command::SetTool(Tool::Pen));
        feed(&mut laser, &mut ed, down(100.0, 100.0), 0.0);
        feed(&mut laser, &mut ed, mv(120.0, 110.0), 0.01);

        // Act: Q goes down mid-stroke; the stroke goes on and ends.
        feed(&mut laser, &mut ed, q_down(), 0.02);
        for i in 0..10 {
            let x = 130.0 + 10.0 * i as f32;
            feed(&mut laser, &mut ed, mv(x, 115.0), 0.03 + 0.01 * f64::from(i));
        }
        assert!(ed.active_gesture().is_some(), "the stroke keeps going");
        feed(&mut laser, &mut ed, up(220.0, 115.0), 0.2);
        feed(&mut laser, &mut ed, mv(300.0, 300.0), 0.25);

        // Assert
        assert_eq!(ed.document().len(), 1, "the stroke was committed");
        assert!(ed.active_gesture().is_none());
        let points: Vec<_> = laser.points().collect();
        assert_eq!(points.len(), 1, "the trail starts after the stroke");
        assert!(points[0].pos.approx_eq(v(300.0, 300.0), EPS));
        assert!(!points[0].joined);
    }
}
