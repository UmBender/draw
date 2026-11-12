//! Tests of the gesture automata (T136, AC-1 to AC-3).

use proptest::prelude::*;

use super::*;
use crate::core::command::{Command, Tool};
use crate::core::editor::{ActiveGesture, Editor};
use crate::core::geom::Vec2;
use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};

const CREATING: [Tool; 5] = [Tool::Pen, Tool::Line, Tool::Arrow, Tool::Rect, Tool::Ellipse];

fn ev_down(ed: &mut Editor, button: PointerButton, x: f32, y: f32) {
    ed.handle(InputEvent::PointerDown {
        pos: Vec2::new(x, y),
        button,
        mods: Modifiers::NONE,
    });
}

fn ev_move(ed: &mut Editor, x: f32, y: f32) {
    ed.handle(InputEvent::PointerMove {
        pos: Vec2::new(x, y),
        mods: Modifiers::NONE,
    });
}

fn ev_up(ed: &mut Editor, button: PointerButton, x: f32, y: f32) {
    ed.handle(InputEvent::PointerUp {
        pos: Vec2::new(x, y),
        button,
        mods: Modifiers::NONE,
    });
}

fn undo_depth(ed: &mut Editor) -> usize {
    let mut depth = 0;
    while ed.can_undo() {
        assert!(ed.apply(Command::Undo));
        depth += 1;
        assert!(depth < 1000, "undo never runs out");
    }
    depth
}

#[test]
fn gesture_for_tool_has_one_variant_per_tool() {
    // Arrange / Act
    let variants: Vec<_> = Tool::ALL
        .iter()
        .map(|&tool| std::mem::discriminant(&Gesture::for_tool(PointerButton::Left, tool)))
        .collect();

    // Assert: eleven tools, eleven distinct variants.
    assert_eq!(variants.len(), 11);
    for (i, a) in variants.iter().enumerate() {
        for b in &variants[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn gesture_tool_roundtrips_for_every_tool() {
    for tool in Tool::ALL {
        // Arrange / Act
        let gesture = Gesture::for_tool(PointerButton::Right, tool);

        // Assert
        assert_eq!(gesture.tool(), Some(tool), "{tool:?}");
        assert_eq!(gesture.button(), PointerButton::Right);
        assert_eq!(gesture.active(), ActiveGesture::Tool(tool));
    }
}

#[test]
fn gesture_stage_moves_from_down_to_active() {
    // Arrange
    let gesture = Gesture::for_tool(PointerButton::Left, Tool::Pen);
    assert_eq!(gesture.stage(), Some(Stage::Down));

    // Act
    let moved = gesture.moved();

    // Assert
    assert_eq!(moved.stage(), Some(Stage::Active));
    assert_eq!(moved.moved(), moved);
}

#[test]
fn gesture_yields_only_when_down_and_left() {
    // Arrange
    let down = Gesture::for_tool(PointerButton::Left, Tool::Pen);
    let moved = down.moved();
    let right = Gesture::for_tool(PointerButton::Right, Tool::Eraser);

    // Act / Assert
    assert!(down.yields_to(PointerButton::Right));
    assert!(down.yields_to(PointerButton::Middle));
    assert!(!down.yields_to(PointerButton::Left));
    assert!(!moved.yields_to(PointerButton::Right));
    assert!(!right.yields_to(PointerButton::Middle));
}

#[test]
fn gesture_release_commits_only_for_its_button() {
    // Arrange
    let gesture = Gesture::for_tool(PointerButton::Left, Tool::Rect);

    // Act / Assert
    assert_eq!(gesture.ends_on(PointerButton::Left), Some(End::Commit));
    assert_eq!(gesture.ends_on(PointerButton::Right), None);
    assert_eq!(End::Cancel, End::Cancel);
}

#[test]
fn one_undo_per_committed_gesture() {
    for tool in CREATING {
        // Arrange
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(tool));

        // Act: a drag with several moves.
        ev_down(&mut ed, PointerButton::Left, 100.0, 100.0);
        ev_move(&mut ed, 140.0, 120.0);
        ev_move(&mut ed, 200.0, 160.0);
        ev_up(&mut ed, PointerButton::Left, 240.0, 200.0);

        // Assert
        assert_eq!(ed.active_gesture(), None, "{tool:?}");
        assert_eq!(undo_depth(&mut ed), 1, "{tool:?}");
        assert_eq!(ed.document().shapes().count(), 0, "{tool:?}");
    }
}

#[test]
fn cancelled_gesture_leaves_no_transaction() {
    for tool in CREATING {
        // Arrange
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(tool));
        ev_down(&mut ed, PointerButton::Left, 100.0, 100.0);
        ev_move(&mut ed, 200.0, 160.0);

        // Act
        ed.handle(InputEvent::KeyDown {
            key: Key::Escape,
            mods: Modifiers::NONE,
        });

        // Assert
        assert_eq!(ed.active_gesture(), None, "{tool:?}");
        assert_eq!(undo_depth(&mut ed), 0, "{tool:?}");
        assert_eq!(ed.document().shapes().count(), 0, "{tool:?}");
    }
}

fn arb_pos() -> impl Strategy<Value = Vec2> {
    (0.0f32..900.0, 0.0f32..700.0).prop_map(|(x, y)| Vec2::new(x, y))
}

fn arb_button() -> impl Strategy<Value = PointerButton> {
    prop_oneof![
        Just(PointerButton::Left),
        Just(PointerButton::Middle),
        Just(PointerButton::Right),
    ]
}

fn arb_event() -> impl Strategy<Value = InputEvent> {
    let none = Modifiers::NONE;
    prop_oneof![
        6 => (arb_pos(), arb_button())
            .prop_map(move |(pos, button)| InputEvent::PointerDown { pos, button, mods: none }),
        8 => arb_pos().prop_map(move |pos| InputEvent::PointerMove { pos, mods: none }),
        6 => (arb_pos(), arb_button())
            .prop_map(move |(pos, button)| InputEvent::PointerUp { pos, button, mods: none }),
        3 => proptest::sample::select(Tool::ALL.to_vec())
            .prop_map(|tool| InputEvent::KeyDown {
                key: match tool {
                    Tool::Pen => Key::P,
                    Tool::Rect => Key::R,
                    Tool::Text => Key::T,
                    _ => Key::Digit1,
                },
                mods: Modifiers::NONE,
            }),
        2 => Just(InputEvent::KeyDown { key: Key::Escape, mods: none }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn gesture_always_ends(
        tool in proptest::sample::select(Tool::ALL.to_vec()),
        events in proptest::collection::vec(arb_event(), 0..60),
    ) {
        // Arrange
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(tool));

        // Act: any events, then the release of every button.
        for event in events {
            ed.handle(event);
        }
        for button in [PointerButton::Left, PointerButton::Middle, PointerButton::Right] {
            ev_up(&mut ed, button, 10.0, 10.0);
        }

        // Assert: nothing is left half-finished, and the history unwinds.
        prop_assert_eq!(ed.active_gesture(), None);
        undo_depth(&mut ed);
        prop_assert!(!ed.can_undo());
    }
}
