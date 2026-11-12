//! Snapshot fixtures and the golden helper (T74, ADR-T74-1).
//!
//! Every fixture in `tests/snapshots/` is rebuilt here from the events in
//! `tests/common/scenes.rs` and compared with the committed file. To
//! regenerate the fixtures after a deliberate change:
//!
//! ```sh
//! DRAW_UPDATE_SNAPSHOTS=1 cargo test --test snapshots -- --test-threads=1
//! ```

mod common;

use common::scenes;
use common::snapshots::{assert_snapshot_with, load_fixture};
use draw::core::command::Command;
use draw::core::editor::Editor;
use draw::core::history::HISTORY_LIMIT;

#[test]
fn fixture_empty() {
    assert_snapshot!(scenes::empty(), "empty");
}

#[test]
fn fixture_pen_mid_stroke() {
    let ed = scenes::pen_mid_stroke();
    assert!(ed.active_gesture().is_some(), "stroke in progress");
    assert_snapshot!(ed, "pen-mid-stroke");
}

#[test]
fn fixture_select_mid_drag() {
    let ed = scenes::select_mid_drag();
    assert_eq!(ed.selection().len(), 2);
    assert!(ed.active_gesture().is_some(), "move in progress");
    assert_snapshot!(ed, "select-mid-drag");
}

#[test]
fn fixture_text_mid_edit() {
    let ed = scenes::text_mid_edit();
    assert!(ed.text_edit().is_some(), "block open");
    assert_snapshot!(ed, "text-mid-edit");
}

#[test]
fn fixture_history_at_limit() {
    assert_snapshot!(scenes::history_at_limit(), "history-at-limit");
}

#[test]
fn fixture_strokes_1000() {
    let ed = scenes::strokes_1000();
    assert_eq!(ed.document().len(), scenes::LARGE_STROKES);
    assert_snapshot!(ed, "strokes-1000");
}

#[test]
fn fixture_text_500() {
    let ed = scenes::text_500();
    assert!(ed.text_edit().is_some(), "a block is open");
    assert_snapshot!(ed, "text-500");
}

#[test]
fn loaded_fixture_continues_like_the_built_state() {
    // Arrange
    let mut built = scenes::history_at_limit();
    let mut loaded = load_fixture("history-at-limit");

    // Act
    let mut undone = 0;
    while built.apply(Command::Undo) {
        assert!(loaded.apply(Command::Undo));
        undone += 1;
    }

    // Assert
    assert!(!loaded.apply(Command::Undo));
    assert_eq!(undone, HISTORY_LIMIT - 3, "full stack minus the 3 undone");
    assert_eq!(loaded, built);
}

#[test]
fn large_fixtures_load() {
    // Arrange / Act
    let strokes = load_fixture("strokes-1000");
    let text = load_fixture("text-500");

    // Assert
    assert_eq!(strokes.document().len(), scenes::LARGE_STROKES);
    assert!(text.text_edit().is_some());
    assert!(text.document().len() >= scenes::TEXT_BLOCKS / 2);
}

// AC-5: the helper itself.

#[test]
fn assert_snapshot_passes_on_fixture() {
    // Arrange / Act / Assert: no panic.
    assert_snapshot_with(&Editor::new(), "empty", false);
}

#[test]
fn assert_snapshot_panics_with_diff_on_change() {
    // Arrange
    let mut changed = Editor::new();
    changed.apply(Command::ToggleToolbar);

    // Act: never rewrite here, even under DRAW_UPDATE_SNAPSHOTS.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_snapshot_with(&changed, "empty", false);
    }));

    // Assert
    let payload = result
        .err()
        .unwrap_or_else(|| panic!("a changed state matched"));
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default();
    assert!(message.contains("empty"), "{message}");
    assert!(message.contains("- toolbar 1"), "{message}");
    assert!(message.contains("+ toolbar 0"), "{message}");
    assert!(message.contains("DRAW_UPDATE_SNAPSHOTS=1"), "{message}");
}

#[test]
fn assert_snapshot_panics_on_missing_fixture() {
    // Arrange / Act
    let result =
        std::panic::catch_unwind(|| assert_snapshot_with(&Editor::new(), "no-such-fixture", false));

    // Assert
    assert!(result.is_err());
}

// ---- T120: weight label on diagonal edges -----------------------------------

/// The editor just before `Ctrl+L`: a text block holding the owner's edge
/// list, plus `4 5` so that a cycle keeps the circle layout.
fn weight_label_scene() -> Editor {
    use draw::core::command::Tool;
    let mut ed = scenes::editor();
    // Two steps up the width ladder: the text size grows with the width, and
    // at this size the `1` of `1>5 1` lands on a node.
    for _ in 0..2 {
        scenes::key(&mut ed, draw::core::input::Key::BracketRight);
    }
    ed.apply(Command::SetTool(Tool::Text));
    scenes::click(&mut ed, scenes::v(100.0, 100.0));
    scenes::type_text(&mut ed, "8>2 $\\pi$ 1>3 4 5>7 2 3>4 1 6>3 2 1>5 1 4>5 3");
    scenes::key(&mut ed, draw::core::input::Key::Escape);
    ed.apply(Command::SetTool(Tool::Select));
    ed.apply(Command::SelectAll);
    ed
}

#[test]
#[allow(non_snake_case)] // `regression_<ID>_<slug>` (Workflow)
fn regression_fixture_T120_is_the_built_scene() {
    assert_snapshot_with(
        &weight_label_scene(),
        "regressions/T120-weight-label-diagonal",
        common::snapshots::update_requested(),
    );
}

#[test]
#[allow(non_snake_case)] // `regression_<ID>_<slug>` (Workflow)
fn regression_T120_weight_label_diagonal() {
    use draw::core::input::{InputEvent, Key, Modifiers};
    use draw::core::shape::Shape;

    // Arrange: the state just before `Ctrl+L`.
    let mut ed = load_fixture("regressions/T120-weight-label-diagonal");

    // Act
    ed.handle(InputEvent::KeyDown {
        key: Key::L,
        mods: Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    });

    // Assert: no weight sits on top of a node, whatever the edge's angle.
    let shapes: Vec<&Shape> = ed.document().shapes().map(|(_, s)| s).collect();
    let circles: Vec<_> = shapes
        .iter()
        .filter(|s| matches!(s, Shape::Ellipse { .. }))
        .map(|s| s.bounds())
        .collect();
    let weights: Vec<_> = shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text { .. }))
        .map(|s| s.bounds())
        .collect();
    assert_eq!(circles.len(), 8, "the graph was drawn");
    assert_eq!(weights.len(), 7, "one weight per edge");
    for weight in &weights {
        for circle in &circles {
            assert!(!weight.intersects(circle), "{weight:?} overlaps {circle:?}");
        }
    }
}

// ---- T121: self-loops and opposite edges ------------------------------------

/// The editor just before `Ctrl+L`: a text block holding the owner's
/// `dgraph:` with a self-loop (`1 1 12`) and opposite edges (`1 3`, `3 1`).
fn loops_and_lanes_scene() -> Editor {
    use draw::core::command::Tool;
    let mut ed = scenes::editor();
    ed.apply(Command::SetTool(Tool::Text));
    scenes::click(&mut ed, scenes::v(100.0, 100.0));
    for line in [
        "dgraph:", "1 2 1", "1 3 4", "1 1 12", "3 4 1", "3 1 2", "4 5 1", "5 1 10",
    ] {
        scenes::type_text(&mut ed, line);
        scenes::key(&mut ed, draw::core::input::Key::Enter);
    }
    scenes::key(&mut ed, draw::core::input::Key::Escape);
    ed.apply(Command::SetTool(Tool::Select));
    ed.apply(Command::SelectAll);
    ed
}

#[test]
#[allow(non_snake_case)] // `regression_<ID>_<slug>` (Workflow)
fn regression_fixture_T121_is_the_built_scene() {
    assert_snapshot_with(
        &loops_and_lanes_scene(),
        "regressions/T121-dgraph-loops-and-opposite-edges",
        common::snapshots::update_requested(),
    );
}

#[test]
#[allow(non_snake_case)] // `regression_<ID>_<slug>` (Workflow)
fn regression_T121_dgraph_loops_and_opposite_edges() {
    use draw::core::input::{InputEvent, Key, Modifiers};
    use draw::core::shape::Shape;

    // Arrange: the state just before `Ctrl+L`.
    let mut ed = load_fixture("regressions/T121-dgraph-loops-and-opposite-edges");

    // Act
    ed.handle(InputEvent::KeyDown {
        key: Key::L,
        mods: Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    });

    // Assert: the loop is drawn, and every weight is readable.
    let shapes: Vec<&Shape> = ed.document().shapes().map(|(_, s)| s).collect();
    let circles: Vec<_> = shapes
        .iter()
        .filter(|s| matches!(s, Shape::Ellipse { .. }))
        .map(|s| s.bounds())
        .collect();
    let weights: Vec<_> = shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text { .. }))
        .map(|s| s.bounds())
        .collect();
    assert_eq!(circles.len(), 5, "the graph was drawn");
    assert_eq!(
        shapes
            .iter()
            .filter(|s| matches!(s, Shape::Stroke { .. }))
            .count(),
        1,
        "the self-loop 1 1 is drawn"
    );
    assert_eq!(weights.len(), 7, "one weight per edge, loop included");
    for (i, weight) in weights.iter().enumerate() {
        for circle in &circles {
            assert!(!weight.intersects(circle), "{weight:?} overlaps {circle:?}");
        }
        for other in &weights[i + 1..] {
            assert!(!weight.intersects(other), "{weight:?} overlaps {other:?}");
        }
    }
}
