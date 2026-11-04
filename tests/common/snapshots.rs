//! Golden snapshot helpers (T74 AC-5): load a fixture from
//! `tests/snapshots/<name>.snap` and compare an editor with one.
//! `DRAW_UPDATE_SNAPSHOTS=1` rewrites the fixtures deliberately.

use std::path::{Path, PathBuf};

use draw::core::editor::Editor;
use draw::core::snapshot::Snapshot;
use draw::core::snapshot::golden::{self, Golden};

/// Set to `1` to rewrite golden files instead of comparing.
pub const UPDATE_ENV: &str = "DRAW_UPDATE_SNAPSHOTS";

/// `tests/snapshots/<name>.snap`.
pub fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.snap"))
}

/// Whether [`UPDATE_ENV`] asks for rewriting.
pub fn update_requested() -> bool {
    std::env::var(UPDATE_ENV).is_ok_and(|value| value == "1")
}

/// The editor in fixture `name`; panics with the parse error if it is bad.
pub fn load_fixture(name: &str) -> Editor {
    let path = fixture_path(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("fixture {}: {err}", path.display()));
    let snapshot =
        Snapshot::parse(&text).unwrap_or_else(|err| panic!("fixture {}: {err}", path.display()));
    Editor::from_snapshot(snapshot)
}

/// Compares `editor` with fixture `name`, or rewrites it when `update`.
/// Panics with a line diff when they differ.
pub fn assert_snapshot_with(editor: &Editor, name: &str, update: bool) {
    let path = fixture_path(name);
    let actual = editor.snapshot().to_text();
    let expected = std::fs::read_to_string(&path).ok();
    match golden::check(&actual, expected.as_deref(), update) {
        Golden::Match => {}
        Golden::Write => {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)
                    .unwrap_or_else(|err| panic!("{}: {err}", dir.display()));
            }
            std::fs::write(&path, actual).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        }
        Golden::Missing => panic!(
            "snapshot {name}: no fixture at {}; run with {UPDATE_ENV}=1 to create it",
            path.display()
        ),
        Golden::Mismatch(diff) => panic!(
            "snapshot {name}: state differs from {}\n{diff}run with {UPDATE_ENV}=1 to accept it",
            path.display()
        ),
    }
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
