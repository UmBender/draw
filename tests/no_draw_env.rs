//! Guard: the binary reads nothing from the environment under a `DRAW_`
//! name (T154, ADR-T154-1). Every setting is a flag.

use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn the_binary_reads_no_draw_variable() {
    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    rust_files(&root.join("crates"), &mut files);

    // Act: a string literal that starts with `DRAW_` is a variable name
    let mut hits = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_default();
        for (index, line) in text.lines().enumerate() {
            if line.contains("\"DRAW_") {
                hits.push(format!("{}:{}: {}", file.display(), index + 1, line.trim()));
            }
        }
    }

    // Assert
    assert!(
        hits.is_empty(),
        "DRAW_ variables read:\n{}",
        hits.join("\n")
    );
}
