//! Guard: no dynamic dispatch (`dyn`) in `src/` or `vendor/` outside the
//! allow-list of `ADR-T131-1`.

use std::fs;
use std::path::{Path, PathBuf};

/// The ADR that holds the allow-list.
const ADR: &str = "docs/decisions/ADR-T131-1 No dyn outside an allow-list.md";

/// Source with comments and string or character literals blanked out, so
/// line numbers are kept and only code remains.
fn code_only(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    let blank = |c: char| if c == '\n' { '\n' } else { ' ' };
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(blank(chars[i]));
                    i += 1;
                }
            }
        } else if c == 'r' && matches!(next, Some('"' | '#')) && is_raw_start(&chars, i) {
            let mut j = i + 1;
            let mut hashes = 0;
            while chars[j] == '#' {
                hashes += 1;
                j += 1;
            }
            j += 1; // opening quote
            let closing: Vec<char> = std::iter::once('"')
                .chain(std::iter::repeat_n('#', hashes))
                .collect();
            while j < chars.len() && chars[j..].get(..closing.len()) != Some(&closing[..]) {
                j += 1;
            }
            let end = (j + closing.len()).min(chars.len());
            out.extend(chars[i..end].iter().map(|&c| blank(c)));
            i = end;
        } else if c == '"' {
            out.push(' ');
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    out.push(blank(chars[i]));
                    i += 1;
                }
                out.push(blank(chars[i]));
                i += 1;
            }
            out.push(' ');
            i += 1;
        } else if c == '\'' && is_char_literal(&chars, i) {
            let mut j = i + 1;
            if chars[j] == '\\' {
                j += 1;
            }
            while j < chars.len() && chars[j] != '\'' {
                j += 1;
            }
            let end = (j + 1).min(chars.len());
            out.extend(chars[i..end].iter().map(|&c| blank(c)));
            i = end;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// `r"`, `r#"`, `r##"` … at `i`, and not the tail of an identifier.
fn is_raw_start(chars: &[char], i: usize) -> bool {
    if i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_') {
        return false;
    }
    let mut j = i + 1;
    while chars.get(j) == Some(&'#') {
        j += 1;
    }
    chars.get(j) == Some(&'"')
}

/// A quote at `i` that opens a character literal, not a lifetime.
fn is_char_literal(chars: &[char], i: usize) -> bool {
    matches!(
        (chars.get(i + 1), chars.get(i + 2)),
        (Some('\\'), _) | (Some(_), Some('\''))
    )
}

/// Line numbers (1-based) of every `dyn` keyword in the code of `source`.
fn dyn_uses(source: &str) -> Vec<usize> {
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    code_only(source)
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            line.match_indices("dyn").any(|(at, _)| {
                let before = line[..at].chars().next_back();
                let after = line[at + 3..].chars().next();
                !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
            })
        })
        .map(|(n, _)| n + 1)
        .collect()
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
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

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Files of the allow-list: the first backticked path of each list item in
/// the `## Allow-list` section of the ADR.
fn allow_list() -> Vec<String> {
    let text = fs::read_to_string(root().join(ADR)).unwrap_or_else(|e| panic!("{ADR}: {e}"));
    let mut in_section = false;
    let mut files = Vec::new();
    for line in text.lines() {
        if line.starts_with("## ") {
            in_section = line.trim() == "## Allow-list";
        } else if in_section && line.starts_with("- `") {
            let rest = &line[3..];
            if let Some(end) = rest.find('`') {
                files.push(rest[..end].to_owned());
            }
        }
    }
    files
}

/// Path of `file` relative to the repository, with `/` separators.
fn relative(file: &Path) -> String {
    file.strip_prefix(root())
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn scanner_finds_dyn() {
    // Arrange
    let source = "fn a(x: &dyn Fn()) {}\nlet b: Box<dyn Any + Send>;\nfn c() {}\n";

    // Act
    let lines = dyn_uses(source);

    // Assert
    assert_eq!(lines, vec![1, 2]);
}

#[test]
fn scanner_ignores_comments_and_strings() {
    // Arrange
    let source = "// a dyn Trait\n/* dyn\n dyn */\nlet s = \"Box<dyn Any>\";\n\
                  let r = r#\"dyn \"x\" dyn\"#;\nlet c = 'd'; let dynamic = 1; let a_dyn = 2;\n";

    // Act
    let lines = dyn_uses(source);

    // Assert
    assert!(lines.is_empty(), "{lines:?}");
}

#[test]
fn scanner_keeps_lifetimes_and_code_after_literals() {
    // Arrange
    let source = "fn f<'a>(x: &'a str, y: &'a dyn T) {}\nlet s = \"x\"; let t: &dyn T;\n";

    // Act
    let lines = dyn_uses(source);

    // Assert
    assert_eq!(lines, vec![1, 2]);
}

#[test]
fn allow_list_is_read_from_the_adr() {
    // Arrange / Act
    let list = allow_list();

    // Assert
    assert!(list.iter().any(|f| f == "src/shell/crash.rs"), "{list:?}");
}

#[test]
fn guard_finds_no_dyn_outside_allow_list() {
    // Arrange
    let allowed = allow_list();
    let mut files = Vec::new();
    rust_files(&root().join("src"), &mut files);
    rust_files(&root().join("vendor"), &mut files);
    // The pure logic moved to `crates/` (ADR-T132-1); keep it guarded.
    rust_files(&root().join("crates"), &mut files);
    files.sort();

    // Act
    let mut found = Vec::new();
    for file in &files {
        let shown = relative(file);
        if allowed.contains(&shown) {
            continue;
        }
        let source = fs::read_to_string(file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        for line in dyn_uses(&source) {
            found.push(format!("{shown}:{line}"));
        }
    }

    // Assert
    assert!(
        found.is_empty(),
        "dyn belongs in the allow-list of ADR-T131-1 or nowhere:\n{}",
        found.join("\n")
    );
}

#[test]
fn allow_list_entries_must_exist() {
    // Arrange
    let allowed = allow_list();

    // Act
    let stale: Vec<&String> = allowed
        .iter()
        .filter(|file| {
            fs::read_to_string(root().join(file))
                .map_or(true, |source| dyn_uses(&source).is_empty())
        })
        .collect();

    // Assert
    assert!(
        stale.is_empty(),
        "remove from the allow-list of ADR-T131-1 (missing file or no dyn left): {stale:?}"
    );
}
