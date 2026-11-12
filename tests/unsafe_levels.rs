//! Guard: the workspace has the five crates of `ADR-T132-1`, each crate root
//! declares its `unsafe` level, and `#![allow(unsafe_code)]` appears only in
//! the files that ADR lists.

use std::fs;
use std::path::{Path, PathBuf};

/// The member crates, under `crates/`.
const MEMBERS: [&str; 4] = ["draw-types", "draw-core", "draw-gfx", "draw-sys"];

/// Crate roots that declare `forbid(unsafe_code)` (ADR-T132-1).
const FORBID: [&str; 4] = [
    "crates/draw-types/src/lib.rs",
    "crates/draw-core/src/lib.rs",
    "src/lib.rs",
    "src/main.rs",
];

/// Crate roots that declare `deny(unsafe_code)` (ADR-T132-1).
const DENY: [&str; 2] = ["crates/draw-gfx/src/lib.rs", "crates/draw-sys/src/lib.rs"];

/// Where `#![allow(unsafe_code)]` may appear: a path ending in `/` lists
/// every file under it (ADR-T132-1).
const ALLOW_LIST: [&str; 2] = ["crates/draw-sys/src/", "crates/draw-gfx/src/gpu/gl.rs"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Lines (1-based) of an attribute that allows `unsafe_code`. Only lines
/// that start with `#[allow(` or `#![allow(` count, so comments and string
/// literals that mention it do not.
fn allow_unsafe_lines(source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let line = line.trim_start();
            (line.starts_with("#![allow(") || line.starts_with("#[allow("))
                && line.contains("unsafe_code")
        })
        .map(|(n, _)| n + 1)
        .collect()
}

/// Whether `source` has the inner attribute `#![<level>(unsafe_code)]`.
fn declares(source: &str, level: &str) -> bool {
    let want = format!("#![{level}(unsafe_code)]");
    source.lines().any(|line| line.trim() == want)
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn is_listed(file: &str) -> bool {
    ALLOW_LIST
        .iter()
        .any(|entry| file == *entry || (entry.ends_with('/') && file.starts_with(entry)))
}

#[test]
fn scanner_finds_allow_attribute() {
    let source = "#![deny(unsafe_code)]\n#![allow(unsafe_code)]\n    #[allow(unsafe_code)]\n#[allow(dead_code, unsafe_code)]\n";

    assert_eq!(allow_unsafe_lines(source), vec![2, 3, 4]);
}

#[test]
fn scanner_ignores_comments_and_strings() {
    let source = "// #![allow(unsafe_code)]\nlet s = \"#![allow(unsafe_code)]\";\n/// #[allow(unsafe_code)]\n";

    assert_eq!(allow_unsafe_lines(source), Vec::<usize>::new());
}

#[test]
fn workspace_lists_the_five_crates() {
    let manifest = read("Cargo.toml");

    assert!(manifest.contains("[workspace]"), "no [workspace] table");
    for name in MEMBERS {
        let path = format!("crates/{name}");
        assert!(
            manifest.contains(&format!("\"{path}\"")),
            "{path} is not a member"
        );
        let own = read(&format!("{path}/Cargo.toml"));
        assert!(
            own.contains(&format!("name = \"{name}\"")),
            "{path}: wrong name"
        );
        assert!(
            own.contains("workspace = true"),
            "{path}: lints are not shared"
        );
    }
    assert!(manifest.contains("name = \"draw\""));
}

#[test]
fn crate_roots_declare_their_level() {
    for file in FORBID {
        assert!(
            declares(&read(file), "forbid"),
            "{file} must declare forbid(unsafe_code)"
        );
    }
    for file in DENY {
        assert!(
            declares(&read(file), "deny"),
            "{file} must declare deny(unsafe_code)"
        );
    }
}

#[test]
fn allow_unsafe_only_in_listed_files() {
    let mut files = Vec::new();
    rust_files(&root().join("src"), &mut files);
    rust_files(&root().join("crates"), &mut files);
    assert!(
        files.len() > MEMBERS.len(),
        "found only {} files",
        files.len()
    );

    let mut found = Vec::new();
    for file in &files {
        let shown = file
            .strip_prefix(root())
            .unwrap_or(file)
            .display()
            .to_string();
        let source = fs::read_to_string(file).unwrap_or_else(|e| panic!("{shown}: {e}"));
        for line in allow_unsafe_lines(&source) {
            if !is_listed(&shown) {
                found.push(format!("{shown}:{line}"));
            }
        }
    }

    assert!(
        found.is_empty(),
        "unsafe_code is allowed only in the files of ADR-T132-1:\n{}",
        found.join("\n")
    );
}
