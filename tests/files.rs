//! The document file (T108, ADR-T108-1): the committed version-1 fixture,
//! round trips and refused input.

mod common;

use common::scenes::{self, v};
use draw::core::command::{Command, Tool};
use draw::core::editor::Editor;
use draw::core::file::{DocFile, VERSION};

const V1: &str = include_str!("files/v1.draw");

fn load(text: &str) -> Editor {
    let file = DocFile::parse(text).unwrap_or_else(|err| panic!("{err}"));
    let mut ed = Editor::new();
    ed.apply_file(file);
    ed
}

#[test]
fn v1_fixture_loads() {
    // Arrange / Act
    let mut ed = load(V1);

    // Assert: page 3 is current and holds the ellipse.
    assert_eq!(VERSION, 1);
    assert_eq!(ed.page(), 2);
    assert_eq!(ed.document().shapes().count(), 1);
    ed.apply(Command::GoToPage(0));
    assert_eq!(ed.document().shapes().count(), 2);
    assert_eq!(ed.document().locked_ids().count(), 1);
    assert!((ed.camera().zoom() - 2.0).abs() < 1e-6);
    ed.apply(Command::GoToPage(1));
    assert_eq!(ed.document().shapes().count(), 0);
}

#[test]
fn fixture_saved_again_is_the_same_bytes() {
    assert_eq!(load(V1).file().to_text(), V1);
}

fn drawn() -> Editor {
    let mut ed = scenes::editor();
    ed.apply(Command::SetTool(Tool::Rect));
    scenes::drag(&mut ed, v(100.0, 100.0), v(300.0, 200.0));
    ed.apply(Command::SelectAll);
    ed.apply(Command::LockSelection);
    ed.apply(Command::GoToPage(4));
    ed.apply(Command::SetTool(Tool::Pen));
    scenes::drag(&mut ed, v(10.0, 10.0), v(200.0, 90.0));
    ed.apply(Command::FitView);
    ed
}

#[test]
fn round_trip_gives_an_equal_document_on_every_page() {
    // Arrange
    let mut original = drawn();

    // Act
    let mut loaded = load(&original.file().to_text());

    // Assert
    assert_eq!(loaded.page(), original.page());
    assert_eq!(loaded.document(), original.document());
    assert_eq!(loaded.camera(), original.camera());
    for page in [0, 4] {
        original.apply(Command::GoToPage(page));
        loaded.apply(Command::GoToPage(page));
        assert_eq!(loaded.document(), original.document());
        assert_eq!(loaded.camera(), original.camera());
    }
}

#[test]
fn file_holds_no_history_or_settings() {
    let text = drawn().file().to_text();
    for word in ["[history]", "[editor]", "[clipboard]", "undo", "toolbar"] {
        assert!(!text.contains(word), "{word} in the file");
    }
    assert!(text.starts_with("draw-file 1\n"));
}

#[test]
fn newer_version_is_refused() {
    let text = V1.replacen("draw-file 1", "draw-file 2", 1);
    match DocFile::parse(&text) {
        Err(err) => assert!(err.to_string().contains("version"), "{err}"),
        Ok(_) => panic!("a newer file must be refused"),
    }
}

#[test]
fn version_zero_and_foreign_headers_are_refused() {
    assert!(DocFile::parse(&V1.replacen("draw-file 1", "draw-file 0", 1)).is_err());
    assert!(DocFile::parse(&V1.replacen("draw-file", "draw-snapshot", 1)).is_err());
    assert!(DocFile::parse("").is_err());
}

#[test]
fn every_truncation_of_the_fixture_is_handled() {
    for end in 0..V1.len() {
        if V1.is_char_boundary(end) {
            // Must not panic; a cut at a section edge may still be valid.
            let _ = DocFile::parse(&V1[..end]);
        }
    }
    assert!(DocFile::parse(&V1[..V1.len() - 9]).is_err());
}

#[test]
fn page_out_of_order_or_range_is_refused() {
    assert!(DocFile::parse(&V1.replacen("index 2", "index 0", 1)).is_err());
    assert!(DocFile::parse(&V1.replacen("index 2", "index 10", 1)).is_err());
    assert!(DocFile::parse(&V1.replacen("current 2", "current 10", 1)).is_err());
    assert!(DocFile::parse(&V1.replacen("count 2", "count 3", 1)).is_err());
}

#[test]
fn applying_a_file_keeps_the_editor_settings() {
    let mut ed = Editor::new();
    ed.apply(Command::SetTool(Tool::Line));
    ed.apply_file(DocFile::parse(V1).unwrap_or_else(|err| panic!("{err}")));
    assert_eq!(ed.tool(), Tool::Line);
    assert!(!ed.can_undo(), "a loaded file has no history");
}
