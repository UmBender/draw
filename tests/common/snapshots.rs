//! Golden snapshot helpers (T74 AC-5): load a fixture from
//! `tests/snapshots/<name>.snap` and compare an editor with one.
//! `DRAW_UPDATE_SNAPSHOTS=1` rewrites the fixtures deliberately.

use std::path::PathBuf;

use draw::core::editor::Editor;

/// Set to `1` to rewrite golden files instead of comparing.
pub const UPDATE_ENV: &str = "DRAW_UPDATE_SNAPSHOTS";

/// `tests/snapshots/<name>.snap`.
pub fn fixture_path(name: &str) -> PathBuf {
    let _ = name;
    todo!()
}

/// Whether [`UPDATE_ENV`] asks for rewriting.
pub fn update_requested() -> bool {
    todo!()
}

/// The editor in fixture `name`; panics with the parse error if it is bad.
pub fn load_fixture(name: &str) -> Editor {
    let _ = name;
    todo!()
}

/// Compares `editor` with fixture `name`, or rewrites it when `update`.
/// Panics with a line diff when they differ.
pub fn assert_snapshot_with(editor: &Editor, name: &str, update: bool) {
    let _ = (editor, name, update);
    todo!()
}

/// `assert_snapshot!(editor, "name")`: compares `editor` with
/// `tests/snapshots/name.snap`; rewrites it under `DRAW_UPDATE_SNAPSHOTS=1`.
#[macro_export]
macro_rules! assert_snapshot {
    ($editor:expr, $name:expr) => {
        $crate::common::snapshots::assert_snapshot_with(
            &$editor,
            $name,
            $crate::common::snapshots::update_requested(),
        )
    };
}
