//! Snapshot fixtures and the golden helper (T74, ADR-T74-1).
//!
//! Every fixture in `tests/snapshots/` is rebuilt here from the events in
//! `tests/common/scenes.rs` and compared with the committed file. To
//! regenerate the fixtures after a deliberate change:
//!
//! ```sh
//! DRAW_UPDATE_SNAPSHOTS=1 cargo test --release --test snapshots -- --include-ignored
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
#[ignore = "slow in debug: run with --include-ignored to check or regenerate"]
fn fixture_strokes_1000() {
    let ed = scenes::strokes_1000();
    assert_eq!(ed.document().len(), scenes::LARGE_STROKES);
    assert_snapshot!(ed, "strokes-1000");
}

#[test]
#[ignore = "slow in debug: run with --include-ignored to check or regenerate"]
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
    assert_eq!(undone, HISTORY_LIMIT);
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
    let payload = result.err().unwrap_or_else(|| panic!("a changed state matched"));
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
