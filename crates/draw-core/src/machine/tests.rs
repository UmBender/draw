//! Tests of the control automaton (T134, AC-1 to AC-4).

use std::time::Instant;

use proptest::prelude::*;

use super::*;
use crate::core::editor::Editor;
use crate::core::geom::Vec2;
use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};

const CTRL: Modifiers = Modifiers {
    shift: false,
    ctrl: true,
    alt: false,
};
const CTRL_SHIFT: Modifiers = Modifiers {
    shift: true,
    ctrl: true,
    alt: false,
};

fn key(key: Key, mods: Modifiers) -> Input {
    Input::from_event(InputEvent::KeyDown { key, mods })
}

fn down(x: f32, y: f32) -> Input {
    Input::from_event(InputEvent::PointerDown {
        pos: Vec2::new(x, y),
        button: PointerButton::Left,
        mods: Modifiers::NONE,
    })
}

fn arb_mods() -> impl Strategy<Value = Modifiers> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(shift, ctrl, alt)| Modifiers {
        shift,
        ctrl,
        alt,
    })
}

fn arb_pos() -> impl Strategy<Value = Vec2> {
    (0.0f32..900.0, 0.0f32..700.0).prop_map(|(x, y)| Vec2::new(x, y))
}

fn arb_button() -> impl Strategy<Value = PointerButton> {
    prop_oneof![
        4 => Just(PointerButton::Left),
        1 => Just(PointerButton::Middle),
        1 => Just(PointerButton::Right),
    ]
}

fn arb_key() -> impl Strategy<Value = Key> {
    prop_oneof![
        Just(Key::Escape),
        Just(Key::Enter),
        Just(Key::Backspace),
        Just(Key::Delete),
        Just(Key::Space),
        Just(Key::Tab),
        Just(Key::ArrowLeft),
        Just(Key::ArrowDown),
        Just(Key::A),
        Just(Key::C),
        Just(Key::D),
        Just(Key::K),
        Just(Key::P),
        Just(Key::S),
        Just(Key::T),
        Just(Key::V),
        Just(Key::Z),
        Just(Key::Digit1),
    ]
}

fn arb_event() -> impl Strategy<Value = InputEvent> {
    prop_oneof![
        6 => (arb_pos(), arb_button(), arb_mods())
            .prop_map(|(pos, button, mods)| InputEvent::PointerDown { pos, button, mods }),
        10 => (arb_pos(), arb_mods()).prop_map(|(pos, mods)| InputEvent::PointerMove { pos, mods }),
        6 => (arb_pos(), arb_button(), arb_mods())
            .prop_map(|(pos, button, mods)| InputEvent::PointerUp { pos, button, mods }),
        2 => (arb_pos(), -3.0f32..3.0).prop_map(|(pos, delta)| InputEvent::Scroll { pos, delta }),
        8 => (arb_key(), arb_mods()).prop_map(|(key, mods)| InputEvent::KeyDown { key, mods }),
        3 => (arb_key(), arb_mods()).prop_map(|(key, mods)| InputEvent::KeyUp { key, mods }),
        6 => (
            prop_oneof![Just('?'), proptest::char::range('a', 'z'), Just('x')],
            arb_mods()
        )
            .prop_map(|(ch, mods)| InputEvent::Char { ch, mods }),
        1 => proptest::option::of(0.0f32..1.0)
            .prop_map(|pressure| InputEvent::Pressure { pressure }),
        1 => "[a-z ]{0,8}".prop_map(InputEvent::Paste),
        1 => "[a-z \n]{0,8}".prop_map(InputEvent::PasteGrid),
        1 => (arb_pos()).prop_map(|size| InputEvent::Resize { size }),
        2 => Just(InputEvent::KeyDown { key: Key::P, mods: CTRL_SHIFT }),
        1 => Just(InputEvent::KeyDown { key: Key::Escape, mods: Modifiers::NONE }),
    ]
}

fn run(inputs: &[Input]) -> (Machine, Vec<Vec<Effect>>) {
    let mut machine = Machine::new();
    let mut all = Vec::new();
    for input in inputs {
        let (next, effects) = step(machine, input.clone());
        machine = next;
        all.push(effects);
    }
    (machine, all)
}

#[test]
fn step_ctrl_s_becomes_a_save_effect() {
    // Arrange / Act
    let (_, effects) = step(Machine::new(), key(Key::S, CTRL));

    // Assert
    assert!(effects.contains(&Effect::Save));
}

#[test]
fn step_tick_is_ignored() {
    // Arrange
    let machine = Machine::new();

    // Act
    let (next, effects) = step(machine.clone(), Input::Tick(Instant::now()));

    // Assert
    assert_eq!(effects, Vec::<Effect>::new());
    assert_eq!(next, machine);
}

#[test]
fn step_reply_is_ignored() {
    let machine = Machine::new();

    let (next, effects) = step(machine.clone(), Input::Reply(Reply::Done));

    assert_eq!(effects, Vec::<Effect>::new());
    assert_eq!(next, machine);
}

#[test]
fn interaction_of_follows_the_editor() {
    // Arrange
    let machine = Machine::new();
    assert_eq!(machine.interaction(), Interaction::Idle);

    // Act / Assert: palette
    let (machine, _) = step(machine, key(Key::P, CTRL_SHIFT));
    assert_eq!(machine.interaction(), Interaction::Palette);
    let (machine, _) = step(machine, key(Key::Escape, Modifiers::NONE));
    assert_eq!(machine.interaction(), Interaction::Idle);

    // Sheet
    let (machine, _) = step(
        machine,
        Input::from_event(InputEvent::Char {
            ch: '?',
            mods: Modifiers::NONE,
        }),
    );
    assert_eq!(machine.interaction(), Interaction::Sheet);
    let (machine, _) = step(machine, key(Key::Escape, Modifiers::NONE));

    // Gesture
    let (machine, _) = step(machine, down(100.0, 100.0));
    assert!(matches!(machine.interaction(), Interaction::Gesture(_)));
}

#[test]
fn table_lists_every_cell() {
    // Arrange / Act
    let cells = transitions();

    // Assert
    assert_eq!(
        cells.len(),
        InteractionKind::ALL.len() * InputKind::ALL.len()
    );
}

#[test]
fn table_ignores_only_tick_and_reply() {
    for cell in transitions() {
        let ignored = cell.route == Route::Ignore;
        let quiet = matches!(cell.input, InputKind::Tick | InputKind::Reply);
        assert_eq!(ignored, quiet, "{:?} in {:?}", cell.input, cell.from);
    }
}

#[test]
fn table_has_no_wildcard_arm() {
    // Arrange
    let source = include_str!("table.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    // Act / Assert
    assert!(
        !production.contains("_ =>"),
        "the routing table must decide every cell"
    );
}

#[test]
fn diagram_matches_the_table() {
    // Arrange
    let note = include_str!("../../../../docs/architecture/Control automaton.md");
    let (Some(begin), Some(end)) = (note.find(DIAGRAM_BEGIN), note.find(DIAGRAM_END)) else {
        panic!("the note must hold both diagram markers");
    };

    // Act
    let embedded = note[begin + DIAGRAM_BEGIN.len()..end].trim();

    // Assert
    assert_eq!(embedded, diagram().trim());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn step_is_deterministic(events in proptest::collection::vec(arb_event(), 0..60)) {
        let inputs: Vec<Input> = events.into_iter().map(Input::from_event).collect();

        let (first, first_effects) = run(&inputs);
        let (second, second_effects) = run(&inputs);

        prop_assert_eq!(first, second);
        prop_assert_eq!(first_effects, second_effects);
    }

    #[test]
    fn machine_and_editor_agree(events in proptest::collection::vec(arb_event(), 0..80)) {
        let mut editor = Editor::new();
        let mut machine = Machine::new();
        for event in events {
            let redraw = editor.handle(event.clone());
            let (next, effects) = step(machine, Input::from_event(event));
            machine = next;
            prop_assert_eq!(effects.contains(&Effect::Redraw), redraw);
            prop_assert_eq!(machine.editor(), &editor);
            prop_assert_eq!(machine.editor().snapshot(), editor.snapshot());
        }
    }
}
