//! Fuzz harness (ADR-0009): drives the headless `Editor` with random input
//! sessions and checks the invariants listed in `docs/architecture/Architecture.md`.
//!
//! `cargo test` runs a quick pass; `scripts/check.sh --fuzz` runs 20 000 cases.
//! Shrunk failures are stored in `tests/fuzz.proptest-regressions` and replayed
//! on every run.

use std::collections::HashSet;

use draw::core::camera::{ZOOM_MAX, ZOOM_MIN};
use draw::core::command::{Command, Tool};
use draw::core::document::ShapeId;
use draw::core::editor::Editor;
use draw::core::geom::Vec2;
use draw::core::history::HISTORY_LIMIT;
use draw::core::input::{InputEvent, Key, Modifiers, PointerButton};
use draw::core::keymap::BINDINGS;
use draw::core::numbering::FIRST_NUMBER;
use draw::core::shape::{GRID_MAX_CELLS, Shape};
use draw::core::smoothing::{Smoother, SmoothingParams, simplify_rdp};
use proptest::prelude::*;
use proptest::sample::select;
use proptest::strategy::ValueTree;
use proptest::test_runner::{FileFailurePersistence, TestRunner};

/// Upper bound on events in one session.
const MAX_SESSION_LEN: usize = 500;

/// Every `Key`, in declaration order.
const ALL_KEYS: [Key; 47] = [
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
    Key::G,
    Key::H,
    Key::I,
    Key::J,
    Key::K,
    Key::L,
    Key::M,
    Key::N,
    Key::O,
    Key::P,
    Key::Q,
    Key::R,
    Key::S,
    Key::T,
    Key::U,
    Key::V,
    Key::W,
    Key::X,
    Key::Y,
    Key::Z,
    Key::Digit0,
    Key::Digit1,
    Key::Digit2,
    Key::Digit3,
    Key::Digit4,
    Key::Digit5,
    Key::Digit6,
    Key::Digit7,
    Key::Digit8,
    Key::Digit9,
    Key::Delete,
    Key::Backspace,
    Key::Escape,
    Key::Space,
    Key::Tab,
    Key::BracketLeft,
    Key::BracketRight,
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::ArrowUp,
    Key::ArrowDown,
];

/// The arrow keys: grid columns and rows.
const ARROW_KEYS: [Key; 4] = [Key::ArrowLeft, Key::ArrowRight, Key::ArrowUp, Key::ArrowDown];

/// Every `PointerButton`.
const ALL_BUTTONS: [PointerButton; 3] = [
    PointerButton::Left,
    PointerButton::Middle,
    PointerButton::Right,
];

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// A coordinate: mostly on-screen, sometimes `0`, `NaN`, `±∞`, huge or any bit
/// pattern.
fn arb_coord() -> impl Strategy<Value = f32> {
    prop_oneof![
        12 => -100.0f32..2_100.0,
        1 => Just(0.0f32),
        1 => Just(f32::NAN),
        1 => Just(f32::INFINITY),
        1 => Just(f32::NEG_INFINITY),
        1 => prop_oneof![1e30f32..f32::MAX, -f32::MAX..-1e30f32],
        1 => any::<f32>(),
    ]
}

/// A screen position built from two [`arb_coord`]s.
fn arb_pos() -> impl Strategy<Value = Vec2> {
    (arb_coord(), arb_coord()).prop_map(|(x, y)| Vec2::new(x, y))
}

/// A finite, on-screen position.
fn arb_screen_pos() -> impl Strategy<Value = Vec2> {
    (0.0f32..2_000.0, 0.0f32..1_200.0).prop_map(|(x, y)| Vec2::new(x, y))
}

/// Any of the 8 modifier combinations.
fn arb_mods() -> impl Strategy<Value = Modifiers> {
    any::<(bool, bool, bool)>().prop_map(|(shift, ctrl, alt)| Modifiers { shift, ctrl, alt })
}

/// Any pointer button.
fn arb_button() -> impl Strategy<Value = PointerButton> {
    select(ALL_BUTTONS.to_vec())
}

/// Any key.
fn arb_key() -> impl Strategy<Value = Key> {
    select(ALL_KEYS.to_vec())
}

/// A scroll delta: usually a few notches, sometimes extreme or non-finite.
fn arb_scroll_delta() -> impl Strategy<Value = f32> {
    prop_oneof![
        6 => -3.0f32..3.0,
        1 => arb_coord(),
    ]
}

/// Any single input event, every variant reachable.
fn arb_event() -> impl Strategy<Value = InputEvent> {
    prop_oneof![
        (arb_pos(), arb_button(), arb_mods())
            .prop_map(|(pos, button, mods)| InputEvent::PointerDown { pos, button, mods }),
        (arb_pos(), arb_mods()).prop_map(|(pos, mods)| InputEvent::PointerMove { pos, mods }),
        (arb_pos(), arb_button(), arb_mods())
            .prop_map(|(pos, button, mods)| InputEvent::PointerUp { pos, button, mods }),
        (arb_pos(), arb_scroll_delta()).prop_map(|(pos, delta)| InputEvent::Scroll { pos, delta }),
        (arb_key(), arb_mods()).prop_map(|(key, mods)| InputEvent::KeyDown { key, mods }),
        (arb_key(), arb_mods()).prop_map(|(key, mods)| InputEvent::KeyUp { key, mods }),
        arb_pos().prop_map(|size| InputEvent::Resize { size }),
    ]
}

/// A realistic drag: down, a wobbly path of moves (rarely a wild point), up.
fn arb_gesture() -> impl Strategy<Value = Vec<InputEvent>> {
    let step = prop_oneof![
        12 => (-40.0f32..40.0, -40.0f32..40.0).prop_map(|(x, y)| Vec2::new(x, y)),
        1 => arb_pos(),
    ];
    // Mostly plain left drags: they reach the active tool.
    let button = prop_oneof![
        6 => Just(PointerButton::Left),
        1 => arb_button(),
    ];
    let mods = prop_oneof![
        4 => Just(Modifiers::NONE),
        1 => arb_mods(),
    ];
    (
        button,
        mods,
        arb_screen_pos(),
        prop::collection::vec(step, 0..30),
        any::<bool>(),
    )
        .prop_map(|(button, mods, start, steps, release)| {
            let mut events = vec![InputEvent::PointerDown {
                pos: start,
                button,
                mods,
            }];
            let mut pos = start;
            for step in steps {
                // Wild points replace the position; normal steps walk from it.
                pos = if step.is_finite() && step.length() <= 60.0 {
                    pos + step
                } else {
                    step
                };
                events.push(InputEvent::PointerMove { pos, mods });
            }
            if release {
                events.push(InputEvent::PointerUp { pos, button, mods });
            }
            events
        })
}

/// A burst of bound key chords: tool switches, undo/redo, clipboard, view.
fn arb_command_burst() -> impl Strategy<Value = Vec<InputEvent>> {
    let chord = select(
        BINDINGS
            .iter()
            .map(|&(chord, _, _)| chord)
            .collect::<Vec<_>>(),
    );
    prop::collection::vec(chord, 1..8).prop_map(|chords| {
        chords
            .into_iter()
            .map(|chord| InputEvent::KeyDown {
                key: chord.key,
                mods: chord.mods,
            })
            .collect()
    })
}

/// Holding `Space` around a left drag (space-pan), released or not.
fn arb_space_drag() -> impl Strategy<Value = Vec<InputEvent>> {
    (arb_gesture(), any::<bool>()).prop_map(|(gesture, release)| {
        let space = |down: bool| {
            let (key, mods) = (Key::Space, Modifiers::NONE);
            if down {
                InputEvent::KeyDown { key, mods }
            } else {
                InputEvent::KeyUp { key, mods }
            }
        };
        let mut events = vec![space(true)];
        events.extend(gesture);
        if release {
            events.push(space(false));
        }
        events
    })
}

/// A grid-tool drag with arrow keys pressed while the button is held.
fn arb_grid_drag() -> impl Strategy<Value = Vec<InputEvent>> {
    let arrow = select(ARROW_KEYS.to_vec()).prop_map(|key| InputEvent::KeyDown {
        key,
        mods: Modifiers::NONE,
    });
    (arb_gesture(), prop::collection::vec(arrow, 0..12), any::<u64>()).prop_map(
        |(gesture, arrows, seed)| {
            let mut events = vec![InputEvent::KeyDown {
                key: Key::G,
                mods: Modifiers::NONE,
            }];
            // Spread the arrows over the drag, after the press.
            let len = gesture.len().max(1) as u64;
            let mut at: Vec<usize> = (0..arrows.len() as u64)
                .map(|i| (1 + (seed.wrapping_add(i * 7919) % len)) as usize)
                .collect();
            at.sort_unstable();
            let mut arrows = arrows.into_iter();
            for (i, event) in gesture.into_iter().enumerate() {
                events.push(event);
                while at.first() == Some(&(i + 1)) {
                    at.remove(0);
                    events.extend(arrows.next());
                }
            }
            events.extend(arrows);
            events
        },
    )
}

/// A session of 1..=500 events, biased towards realistic gestures.
fn arb_session() -> impl Strategy<Value = Vec<InputEvent>> {
    let chunk = prop_oneof![
        6 => arb_gesture(),
        1 => arb_grid_drag(),
        3 => arb_command_burst(),
        1 => arb_space_drag(),
        3 => arb_event().prop_map(|event| vec![event]),
        1 => arb_screen_pos().prop_map(|size| vec![InputEvent::Resize { size }]),
    ];
    prop::collection::vec(chunk, 1..60).prop_map(|chunks| {
        let mut events: Vec<InputEvent> = chunks.into_iter().flatten().collect();
        events.truncate(MAX_SESSION_LEN);
        events
    })
}

/// Smoothing parameters, sane or arbitrary (negative, NaN, ∞).
fn arb_smoothing_params() -> impl Strategy<Value = SmoothingParams> {
    let value = prop_oneof![3 => 0.0f32..10.0, 1 => arb_coord()];
    (value.clone(), value.clone(), value).prop_map(|(min_dist_px, alpha, epsilon_px)| {
        SmoothingParams {
            min_dist_px,
            alpha,
            epsilon_px,
        }
    })
}

/// Default config (honours `PROPTEST_CASES`), with shrunk failures stored in
/// `tests/fuzz.proptest-regressions` next to this file.
fn config() -> ProptestConfig {
    ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..ProptestConfig::default()
    }
}

// ---------------------------------------------------------------------------
// Invariants
// ---------------------------------------------------------------------------

/// Snapshot of the document: ids and shapes, bottom to top.
fn shapes_of(editor: &Editor) -> Vec<(ShapeId, Shape)> {
    editor
        .document()
        .shapes()
        .map(|(id, shape)| (id, shape.clone()))
        .collect()
}

/// Checks every editor invariant from the architecture note.
fn check_invariants(editor: &Editor, step: usize) -> Result<(), TestCaseError> {
    let cells = 1..=GRID_MAX_CELLS;
    for (id, shape) in editor.document().shapes() {
        prop_assert!(
            shape.is_finite(),
            "step {step}: shape {id:?} not finite: {shape:?}"
        );
        if let Shape::Grid { cols, rows, .. } = shape {
            prop_assert!(
                cells.contains(cols) && cells.contains(rows),
                "step {step}: grid {id:?} has {cols} × {rows} cells"
            );
        }
        // Only Rect and Ellipse carry a label field; a label is a number ≥ 1.
        if let Some(label) = shape.label() {
            prop_assert!(
                label >= FIRST_NUMBER,
                "step {step}: shape {id:?} has label {label}"
            );
        }
    }

    let helpers = editor.helpers();
    prop_assert!(
        cells.contains(&helpers.grid_cols) && cells.contains(&helpers.grid_rows),
        "step {step}: editor grid dims {} × {}",
        helpers.grid_cols,
        helpers.grid_rows
    );
    prop_assert!(
        helpers.next_number >= FIRST_NUMBER,
        "step {step}: numbering counter {}",
        helpers.next_number
    );

    let camera = editor.camera();
    let zoom = camera.zoom();
    prop_assert!(
        (ZOOM_MIN..=ZOOM_MAX).contains(&zoom),
        "step {step}: zoom {zoom} out of range"
    );
    prop_assert!(
        camera.offset().is_finite(),
        "step {step}: camera offset not finite: {:?}",
        camera.offset()
    );

    let mut seen = HashSet::new();
    for &id in editor.selection() {
        prop_assert!(
            editor.document().get(id).is_some(),
            "step {step}: selected {id:?} not in document"
        );
        prop_assert!(seen.insert(id), "step {step}: {id:?} selected twice");
    }
    Ok(())
}

/// Runs `session`, checking invariants after every event.
fn run_session(session: &[InputEvent]) -> Result<Editor, TestCaseError> {
    let mut editor = Editor::new();
    check_invariants(&editor, 0)?;
    for (i, &event) in session.iter().enumerate() {
        editor.handle(event);
        check_invariants(&editor, i + 1)?;
    }
    Ok(editor)
}

// ---------------------------------------------------------------------------
// AC-1: strategy coverage
// ---------------------------------------------------------------------------

/// Special coordinate classes the strategies must produce.
const COORD_CLASSES: [&str; 5] = ["zero", "nan", "+inf", "-inf", "huge"];

/// The special class of `v`, if any.
fn coord_class(v: f32) -> Option<&'static str> {
    if v.is_nan() {
        Some("nan")
    } else if v == f32::INFINITY {
        Some("+inf")
    } else if v == f32::NEG_INFINITY {
        Some("-inf")
    } else if v.abs() > 1e30 {
        Some("huge")
    } else if v == 0.0 {
        Some("zero")
    } else {
        None
    }
}

#[test]
fn strategies_generate_all_variants() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_event();
    let mut variants = HashSet::new();
    let mut keys = HashSet::new();
    let mut buttons = HashSet::new();
    let mut mods_seen = HashSet::new();
    let mut coords = HashSet::new();

    // Act
    for _ in 0..20_000 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_event failed to generate a value");
        };
        let event = tree.current();
        let (variant, pos) = match event {
            InputEvent::PointerDown { pos, button, mods } => {
                buttons.insert(button);
                mods_seen.insert(mods);
                (0, Some(pos))
            }
            InputEvent::PointerMove { pos, mods } => {
                mods_seen.insert(mods);
                (1, Some(pos))
            }
            InputEvent::PointerUp { pos, button, mods } => {
                buttons.insert(button);
                mods_seen.insert(mods);
                (2, Some(pos))
            }
            InputEvent::Scroll { pos, delta } => {
                coords.extend(coord_class(delta));
                (3, Some(pos))
            }
            InputEvent::KeyDown { key, mods } => {
                keys.insert(key);
                mods_seen.insert(mods);
                (4, None)
            }
            InputEvent::KeyUp { key, mods } => {
                keys.insert(key);
                mods_seen.insert(mods);
                (5, None)
            }
            InputEvent::Resize { size } => (6, Some(size)),
        };
        variants.insert(variant);
        if let Some(pos) = pos {
            coords.extend(coord_class(pos.x));
            coords.extend(coord_class(pos.y));
        }
    }

    // Assert
    assert_eq!(variants.len(), 7, "event variants seen: {variants:?}");
    assert_eq!(keys.len(), ALL_KEYS.len(), "keys seen: {keys:?}");
    assert_eq!(
        buttons.len(),
        ALL_BUTTONS.len(),
        "buttons seen: {buttons:?}"
    );
    assert_eq!(
        mods_seen.len(),
        8,
        "modifier combinations seen: {mods_seen:?}"
    );
    assert_eq!(
        coords.len(),
        COORD_CLASSES.len(),
        "special coordinates seen: {coords:?}"
    );
}

#[test]
fn grid_drags_press_arrows_while_the_grid_tool_drags() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_grid_drag();
    let mut live = 0;

    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_grid_drag failed to generate a value");
        };
        let mut editor = Editor::new();
        let mut changed_live = false;

        // Act
        for event in tree.current() {
            let during_grid_drag = editor.active_gesture()
                == Some(draw::core::editor::ActiveGesture::Tool(Tool::Grid));
            let is_arrow = matches!(event, InputEvent::KeyDown { key, .. } if ARROW_KEYS.contains(&key));
            changed_live |= editor.handle(event) && during_grid_drag && is_arrow;
        }
        live += usize::from(changed_live);
    }

    // Assert: dims really change in the middle of grid drags.
    assert!(live >= 50, "sessions resizing a grid mid-drag: {live}/200");
}

#[test]
fn session_strategy_respects_length_bounds() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_session();

    for _ in 0..200 {
        // Act
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_session failed to generate a value");
        };
        let session = tree.current();

        // Assert
        assert!((1..=MAX_SESSION_LEN).contains(&session.len()));
    }
}

#[test]
fn sessions_reach_deep_editor_states() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_session();
    let (mut with_shapes, mut with_selection, mut zoomed, mut with_redo) = (0, 0, 0, 0);

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_session failed to generate a value");
        };
        let mut editor = Editor::new();
        let (mut shapes, mut selection, mut zoom, mut redo) = (false, false, false, false);
        for event in tree.current() {
            editor.handle(event);
            shapes |= !editor.document().is_empty();
            selection |= !editor.selection().is_empty();
            zoom |= !draw::core::geom::approx_eq(editor.camera().zoom(), 1.0, 1e-6);
            redo |= editor.can_redo();
        }
        with_shapes += usize::from(shapes);
        with_selection += usize::from(selection);
        zoomed += usize::from(zoom);
        with_redo += usize::from(redo);
    }

    // Assert: a harness that never reaches these states proves nothing.
    assert!(
        with_shapes >= 100,
        "sessions with shapes: {with_shapes}/200"
    );
    assert!(
        with_selection >= 20,
        "sessions with a selection: {with_selection}/200"
    );
    assert!(zoomed >= 20, "sessions that zoomed: {zoomed}/200");
    assert!(with_redo >= 20, "sessions that could redo: {with_redo}/200");
}

// ---------------------------------------------------------------------------
// AC-2, AC-3: editor sessions
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn editor_never_panics_and_keeps_invariants(session in arb_session()) {
        // Arrange / Act / Assert: invariants are checked after every event.
        run_session(&session)?;
    }

    #[test]
    fn undo_all_then_redo_all_is_symmetric(session in arb_session()) {
        // Arrange
        let mut editor = run_session(&session)?;
        editor.apply(Command::Cancel);
        // Leftover redo entries would make "redo all" overshoot the snapshot.
        while editor.can_redo() {
            prop_assert!(editor.apply(Command::Redo), "redo reported no change");
        }
        let before = shapes_of(&editor);

        // Act: undo everything.
        let mut undos = 0usize;
        while editor.can_undo() {
            prop_assert!(editor.apply(Command::Undo), "undo reported no change");
            check_invariants(&editor, session.len() + undos)?;
            undos += 1;
            prop_assert!(undos <= HISTORY_LIMIT, "more undos than the history limit");
        }

        // Assert: empty unless the history was capped.
        if undos < HISTORY_LIMIT {
            prop_assert!(
                editor.document().is_empty(),
                "{} shapes left after {undos} undos",
                editor.document().len()
            );
        }

        // Act: redo everything.
        for _ in 0..undos {
            prop_assert!(editor.apply(Command::Redo), "redo reported no change");
            check_invariants(&editor, session.len() + undos)?;
        }

        // Assert
        prop_assert!(!editor.can_redo());
        prop_assert_eq!(shapes_of(&editor), before);
    }
}

// ---------------------------------------------------------------------------
// AC-4: smoothing
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn smoother_never_panics_on_arbitrary_points(
        params in arb_smoothing_params(),
        px_to_world in arb_coord(),
        points in prop::collection::vec(arb_pos(), 0..200),
    ) {
        // Arrange
        let mut smoother = Smoother::new(params, px_to_world);

        // Act
        for &p in &points {
            smoother.push(p);
            prop_assert!(smoother.points().iter().all(|q| q.is_finite()));
        }
        let finished = smoother.finish();

        // Assert
        prop_assert!(finished.iter().all(|q| q.is_finite()), "{finished:?}");
    }

    #[test]
    fn simplify_rdp_never_panics_on_arbitrary_points(
        points in prop::collection::vec(arb_pos(), 0..200),
        eps in arb_coord(),
    ) {
        // Arrange
        let finite: Vec<Vec2> = points.iter().copied().filter(|p| p.is_finite()).collect();

        // Act
        let _ = simplify_rdp(&points, eps);
        let simplified = simplify_rdp(&finite, eps);

        // Assert: finite, and a subsequence of the input.
        prop_assert!(simplified.iter().all(|p| p.is_finite()));
        let mut rest = finite.iter();
        for p in &simplified {
            prop_assert!(rest.any(|q| q == p), "{p:?} is not from the input in order");
        }
    }
}
