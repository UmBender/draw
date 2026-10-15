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
use draw::core::experimental::Experimental;
use draw::core::geom::Vec2;
use draw::core::history::HISTORY_LIMIT;
use draw::core::input::{InputEvent, Key, Modifiers, PointerButton};
use draw::core::keymap::BINDINGS;
use draw::core::numbering::FIRST_NUMBER;
use draw::core::shape::{GRID_MAX_CELLS, Shape, TextSlot};
use draw::core::smoothing::{Smoother, SmoothingParams, simplify_rdp};
use draw::core::tex;
use draw::core::text_layout::typeset;
use proptest::prelude::*;
use proptest::sample::select;
use proptest::strategy::ValueTree;
use proptest::test_runner::{FileFailurePersistence, TestRunner};

/// Upper bound on events in one session.
const MAX_SESSION_LEN: usize = 500;

/// Every `Key`, in declaration order.
const ALL_KEYS: [Key; 50] = [
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
    Key::Enter,
    Key::Home,
    Key::End,
];

/// Keys that edit a text block (T29).
const TEXT_KEYS: [Key; 10] = [
    Key::Enter,
    Key::Backspace,
    Key::Delete,
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::ArrowUp,
    Key::ArrowDown,
    Key::Home,
    Key::End,
    Key::Tab,
];

/// The arrow keys: grid columns and rows.
const ARROW_KEYS: [Key; 4] = [
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::ArrowUp,
    Key::ArrowDown,
];

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

/// A typed character: mostly letters, sometimes line breaks, spaces,
/// multi-byte or control characters, or any `char`.
fn arb_char() -> impl Strategy<Value = char> {
    prop_oneof![
        8 => proptest::char::range('a', 'z'),
        1 => Just('\n'),
        1 => Just(' '),
        1 => select(vec!['β', '∑', '≤', 'é', '𝔸']),
        1 => select(vec!['\u{8}', '\r', '\t', '\u{1b}', '\u{7f}']),
        1 => any::<char>(),
    ]
}

/// Formula-ish source (T30, T33): TeX tokens, `$`s, braces and any `char`.
fn arb_formula_text() -> impl Strategy<Value = String> {
    let token = prop_oneof![
        6 => select(vec![
            "$", "\\$", "{", "}", "^", "_", "\\", " ", "x", "i", "2", "-", "+", "=", "(",
            ")", ",", "|", "\\frac", "\\sqrt", "\\binom", "\\text", "\\pmod",
            "\\alpha", "\\Omega", "\\sum", "\\prod", "\\min", "\\lim", "\\log",
            "\\le", "\\cdot", "\\lfloor", "\\{", "\\}", "\\foo",
            // T33: script capitals and italic overhangs at both ends.
            "\\mathcal", "A", "f", "j", "\\varphi",
        ])
        .prop_map(str::to_owned),
        1 => arb_char().prop_map(String::from),
    ];
    prop::collection::vec(token, 0..24).prop_map(|tokens| tokens.concat())
}

/// A formula typed into a block: `$…$` around formula-ish source, or
/// the source alone.
fn arb_formula() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => arb_formula_text().prop_map(|src| format!("${src}$")),
        1 => arb_formula_text(),
    ]
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
        (arb_char(), arb_mods()).prop_map(|(ch, mods)| InputEvent::Char { ch, mods }),
    ]
}

/// A Text-tool session (T29): `T`, a click, then typing and editing keys,
/// sometimes ended with `Esc`, `Ctrl+Z`, `Ctrl+Enter` or another click.
fn arb_text_session() -> impl Strategy<Value = Vec<InputEvent>> {
    let none = Modifiers::NONE;
    let step = prop_oneof![
        8 => arb_char().prop_map(move |ch| vec![InputEvent::Char { ch, mods: none }]),
        2 => arb_formula().prop_map(move |src| {
            src.chars().map(|ch| InputEvent::Char { ch, mods: none }).collect()
        }),
        3 => select(TEXT_KEYS.to_vec()).prop_map(move |key| vec![InputEvent::KeyDown { key, mods: none }]),
        1 => (arb_key(), arb_mods()).prop_map(|(key, mods)| vec![InputEvent::KeyDown { key, mods }]),
        1 => arb_screen_pos().prop_map(move |pos| vec![
            InputEvent::PointerDown { pos, button: PointerButton::Left, mods: none },
            InputEvent::PointerUp { pos, button: PointerButton::Left, mods: none },
        ]),
    ];
    let end = prop_oneof![
        3 => Just(vec![InputEvent::KeyDown { key: Key::Escape, mods: none }]),
        1 => Just(vec![InputEvent::KeyDown {
            key: Key::Z,
            mods: Modifiers { ctrl: true, ..none },
        }]),
        1 => Just(vec![InputEvent::KeyDown {
            key: Key::Enter,
            mods: Modifiers { ctrl: true, ..none },
        }]),
        1 => Just(Vec::new()),
    ];
    (arb_screen_pos(), prop::collection::vec(step, 0..30), end).prop_map(
        move |(pos, steps, end)| {
            let mut events = vec![
                InputEvent::KeyDown {
                    key: Key::T,
                    mods: none,
                },
                InputEvent::PointerDown {
                    pos,
                    button: PointerButton::Left,
                    mods: none,
                },
                InputEvent::PointerUp {
                    pos,
                    button: PointerButton::Left,
                    mods: none,
                },
            ];
            events.extend(steps.into_iter().flatten());
            events.extend(end);
            events
        },
    )
}

/// A shape-text session (T31): draw a rectangle, ellipse or grid, switch
/// to the Text tool, click inside it, then type, walk cells with `Tab` /
/// `Enter` (with or without `Shift`) and end like a text session.
fn arb_host_session() -> impl Strategy<Value = Vec<InputEvent>> {
    let none = Modifiers::NONE;
    let shift = Modifiers {
        shift: true,
        ..none
    };
    let step = prop_oneof![
        6 => arb_char().prop_map(move |ch| vec![InputEvent::Char { ch, mods: none }]),
        1 => arb_formula().prop_map(move |src| {
            src.chars().map(|ch| InputEvent::Char { ch, mods: none }).collect()
        }),
        3 => (select(vec![Key::Tab, Key::Enter]), any::<bool>()).prop_map(move |(key, s)| {
            vec![InputEvent::KeyDown { key, mods: if s { shift } else { none } }]
        }),
        2 => select(TEXT_KEYS.to_vec()).prop_map(move |key| vec![InputEvent::KeyDown { key, mods: none }]),
        1 => (-1.0f32..1.0, -1.0f32..1.0).prop_map(|(x, y)| vec![
            InputEvent::PointerDown { pos: Vec2::new(x, y), button: PointerButton::Left, mods: Modifiers::NONE },
            InputEvent::PointerUp { pos: Vec2::new(x, y), button: PointerButton::Left, mods: Modifiers::NONE },
        ]),
    ];
    let end = prop_oneof![
        3 => Just(vec![InputEvent::KeyDown { key: Key::Escape, mods: none }]),
        1 => Just(vec![InputEvent::KeyDown {
            key: Key::Z,
            mods: Modifiers { ctrl: true, ..none },
        }]),
        1 => Just(Vec::new()),
    ];
    (
        select(vec![Key::R, Key::C, Key::G]),
        (50.0f32..700.0, 50.0f32..500.0),
        (60.0f32..300.0, 60.0f32..300.0),
        prop::collection::vec(step, 0..30),
        end,
    )
        .prop_map(move |(tool, (x, y), (w, h), steps, end)| {
            let at = |dx: f32, dy: f32| Vec2::new(x + dx * w, y + dy * h);
            let press = |pos| InputEvent::PointerDown {
                pos,
                button: PointerButton::Left,
                mods: none,
            };
            let release = |pos| InputEvent::PointerUp {
                pos,
                button: PointerButton::Left,
                mods: none,
            };
            let mut events = vec![
                InputEvent::KeyDown {
                    key: tool,
                    mods: none,
                },
                press(at(0.0, 0.0)),
                InputEvent::PointerMove {
                    pos: at(0.5, 0.5),
                    mods: none,
                },
                release(at(1.0, 1.0)),
                InputEvent::KeyDown {
                    key: Key::T,
                    mods: none,
                },
                press(at(0.45, 0.45)),
                release(at(0.45, 0.45)),
            ];
            for step in steps {
                for event in step {
                    // Clicks are offsets inside the shape.
                    events.push(match event {
                        InputEvent::PointerDown { pos, .. } => {
                            press(at(0.5 + pos.x * 0.4, 0.5 + pos.y * 0.4))
                        }
                        InputEvent::PointerUp { pos, .. } => {
                            release(at(0.5 + pos.x * 0.4, 0.5 + pos.y * 0.4))
                        }
                        other => other,
                    });
                }
            }
            events.extend(end);
            events
        })
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
    (
        arb_gesture(),
        prop::collection::vec(arrow, 0..12),
        any::<u64>(),
    )
        .prop_map(|(gesture, arrows, seed)| {
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
        })
}

/// A session of 1..=500 events, biased towards realistic gestures.
fn arb_session() -> impl Strategy<Value = Vec<InputEvent>> {
    let chunk = prop_oneof![
        6 => arb_gesture(),
        1 => arb_grid_drag(),
        3 => arb_command_burst(),
        1 => arb_space_drag(),
        2 => arb_text_session(),
        1 => arb_host_session(),
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
            // T21: fills inside the grid, sorted by (row, col), unique.
            let fills = shape.cell_fills();
            prop_assert!(
                fills.iter().all(|f| f.col < *cols && f.row < *rows),
                "step {step}: grid {id:?} fill outside {cols} × {rows}: {fills:?}"
            );
            prop_assert!(
                fills
                    .windows(2)
                    .all(|w| (w[0].row, w[0].col) < (w[1].row, w[1].col)),
                "step {step}: grid {id:?} fills not sorted/unique: {fills:?}"
            );
        }
        // T29: text blocks have a positive size and visible text.
        if let Shape::Text { text, size, .. } = shape {
            prop_assert!(*size > 0.0, "step {step}: text {id:?} has size {size}");
            prop_assert!(
                !text.trim().is_empty(),
                "step {step}: text {id:?} is blank: {text:?}"
            );
        }
        // T30: a text block's box is finite and holds its typeset ink.
        if let Shape::Text {
            origin, text, size, ..
        } = shape
        {
            let b = shape.bounds();
            prop_assert!(
                b.min.is_finite() && b.max.is_finite(),
                "step {step}: text {id:?} bounds {b:?}"
            );
            let slack = 1e-3 * size + 1e-5 * (origin.x.abs() + origin.y.abs());
            let room = b.expand(slack);
            for ink in &typeset(text).ink {
                let ib = ink.bounds();
                let (lo, hi) = (*origin + ib.min * *size, *origin + ib.max * *size);
                prop_assert!(
                    room.contains(lo) && room.contains(hi),
                    "step {step}: text {id:?} ink {ink:?} outside {b:?}"
                );
            }
        }
        // T31: shape text only with the switch on; non-blank, sized,
        // cell texts inside the grid, sorted by (row, col), unique.
        let own = shape.slot_text(TextSlot::Shape);
        let cell_texts = shape.cell_texts();
        if !editor.experimental().shape_text {
            prop_assert!(
                own.is_none() && cell_texts.is_empty(),
                "step {step}: shape {id:?} has text with the switch off"
            );
        }
        for content in own.into_iter().chain(cell_texts.iter().map(|c| &c.content)) {
            prop_assert!(
                !content.text.trim().is_empty() && content.size.is_finite() && content.size > 0.0,
                "step {step}: shape {id:?} has bad text {content:?}"
            );
        }
        if let Shape::Grid { cols, rows, .. } = shape {
            prop_assert!(
                cell_texts.iter().all(|c| c.col < *cols && c.row < *rows),
                "step {step}: grid {id:?} text outside {cols} × {rows}"
            );
            prop_assert!(
                cell_texts
                    .windows(2)
                    .all(|w| (w[0].row, w[0].col) < (w[1].row, w[1].col)),
                "step {step}: grid {id:?} texts not sorted/unique"
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
    run_session_with(session, Experimental::default())
}

/// [`run_session`] on an editor with the `experimental` switches.
fn run_session_with(
    session: &[InputEvent],
    experimental: Experimental,
) -> Result<Editor, TestCaseError> {
    let mut editor = Editor::new().with_experimental(experimental);
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
            InputEvent::Char { mods, .. } => {
                mods_seen.insert(mods);
                (7, None)
            }
        };
        variants.insert(variant);
        if let Some(pos) = pos {
            coords.extend(coord_class(pos.x));
            coords.extend(coord_class(pos.y));
        }
    }

    // Assert
    assert_eq!(variants.len(), 8, "event variants seen: {variants:?}");
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
            let is_arrow =
                matches!(event, InputEvent::KeyDown { key, .. } if ARROW_KEYS.contains(&key));
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

#[test]
fn sessions_reach_text_edit_states() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_session();
    let (mut editing, mut committed) = (0, 0);

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_session failed to generate a value");
        };
        let mut editor = Editor::new();
        let (mut edit, mut text) = (false, false);
        for event in tree.current() {
            editor.handle(event);
            edit |= editor.text_edit().is_some_and(|e| !e.text().is_empty());
            text |= editor
                .document()
                .shapes()
                .any(|(_, shape)| matches!(shape, Shape::Text { .. }));
        }
        editing += usize::from(edit);
        committed += usize::from(text);
    }

    // Assert
    assert!(editing >= 40, "sessions typing into a block: {editing}/200");
    assert!(
        committed >= 20,
        "sessions with a text block: {committed}/200"
    );
}

#[test]
fn switch_on_sessions_reach_shape_text() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_host_session();
    let (mut own, mut cells) = (0, 0);

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_host_session failed to generate a value");
        };
        let mut editor = Editor::new().with_experimental(SHAPE_TEXT_ON);
        editor.handle(InputEvent::Resize {
            size: Vec2::new(1024.0, 768.0),
        });
        for event in tree.current() {
            editor.handle(event);
        }
        let shapes = || editor.document().shapes().map(|(_, shape)| shape);
        own += usize::from(shapes().any(|s| s.slot_text(TextSlot::Shape).is_some()));
        cells += usize::from(shapes().any(|s| s.cell_texts().len() >= 2));
    }

    // Assert
    assert!(own >= 30, "sessions with rectangle/ellipse text: {own}/200");
    assert!(
        cells >= 15,
        "sessions with two or more cell texts: {cells}/200"
    );
}

#[test]
fn sessions_reach_committed_formulas() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_text_session();
    let mut formulas = 0;

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_text_session failed to generate a value");
        };
        let mut editor = Editor::new();
        for event in tree.current() {
            editor.handle(event);
        }
        let has_formula = editor.document().shapes().any(|(_, shape)| {
            matches!(shape, Shape::Text { text, .. }
                if tex::parse(text).iter().any(|s| matches!(s, tex::Segment::Math(_))))
        });
        formulas += usize::from(has_formula);
    }

    // Assert
    assert!(
        formulas >= 10,
        "sessions with a committed formula: {formulas}/200"
    );
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
        check_undo_redo_symmetric(&session, Experimental::default())?;
    }

    #[test]
    fn shape_text_sessions_never_panic_and_keep_invariants(session in arb_session()) {
        run_session_with(&session, SHAPE_TEXT_ON)?;
    }

    #[test]
    fn shape_text_undo_all_then_redo_all_is_symmetric(session in arb_session()) {
        check_undo_redo_symmetric(&session, SHAPE_TEXT_ON)?;
    }

    #[test]
    fn switch_off_sessions_never_create_shape_text(session in arb_session()) {
        let editor = run_session(&session)?;
        let texts = editor.document().shapes().any(|(_, shape)| {
            shape.slot_text(TextSlot::Shape).is_some() || !shape.cell_texts().is_empty()
        });
        prop_assert!(!texts);
    }
}

/// Shape text on (T31).
const SHAPE_TEXT_ON: Experimental = Experimental { shape_text: true };

/// Runs `session`, undoes everything (the document must be empty unless the
/// history was capped) and redoes everything (the document must be back).
fn check_undo_redo_symmetric(
    session: &[InputEvent],
    experimental: Experimental,
) -> Result<(), TestCaseError> {
    // Arrange
    let mut editor = run_session_with(session, experimental)?;
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
    Ok(())
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

proptest! {
    #![proptest_config(config())]

    /// T30 AC-6: parsing and layout are total on formula-ish text.
    #[test]
    fn fuzz_tex_parse_and_layout_are_total(src in arb_formula()) {
        // Act
        let segments = tex::parse(&src);
        let layout = tex::layout(&segments);

        // Assert
        prop_assert_eq!(tex::print(&segments), src);
        prop_assert!(layout.width.is_finite() && layout.height.is_finite());
        let room = layout.bounds().expand(1e-3);
        for ink in &layout.ink {
            let ib = ink.bounds();
            prop_assert!(room.contains(ib.min) && room.contains(ib.max), "{ink:?}");
        }
    }
}
