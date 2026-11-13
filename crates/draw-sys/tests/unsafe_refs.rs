//! Every `unsafe` block, `unsafe fn`, `unsafe impl` and `unsafe extern` in
//! `crates/*/src` names its test with a `// TEST: <fn_name>` comment on the
//! same line or in the 8 lines above (ADR-T142-1). The named function must
//! exist somewhere in the repository.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Lines above an `unsafe` use where its marker may sit.
const WINDOW: usize = 8;

/// Keywords that make a following `unsafe` an unsafe item.
const ITEM_KEYWORDS: [&str; 4] = ["fn", "impl", "extern", "trait"];

fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The code part of a line: everything before a `//` comment.
fn code_of(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

/// Whether the code part of a line holds an `unsafe` block or item.
fn has_unsafe_use(code: &str) -> bool {
    code.match_indices("unsafe").any(|(pos, _)| {
        let bytes = code.as_bytes();
        let before_ok = pos == 0 || !is_ident(bytes[pos - 1]);
        let after = &code[pos + "unsafe".len()..];
        let after_ok = after.as_bytes().first().is_none_or(|b| !is_ident(*b));
        let rest = after.trim_start();
        let item = ITEM_KEYWORDS.iter().any(|kw| {
            rest.strip_prefix(kw)
                .is_some_and(|tail| tail.as_bytes().first().is_none_or(|b| !is_ident(*b)))
        });
        before_ok && after_ok && (rest.starts_with('{') || item)
    })
}

/// The test named by a `TEST:` marker on line `index` or the lines above.
fn marker_for(lines: &[&str], index: usize) -> Option<String> {
    let start = index.saturating_sub(WINDOW);
    lines[start..=index].iter().rev().find_map(|line| {
        let (_, tail) = line.split_once("TEST:")?;
        let name: String = tail
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        (!name.is_empty()).then_some(name)
    })
}

/// A problem found in one source text: line number (1-based) and why.
fn problems(source: &str, known: &HashSet<String>) -> Vec<(usize, String)> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !has_unsafe_use(code_of(line)) {
            continue;
        }
        match marker_for(&lines, index) {
            None => found.push((index + 1, "no `// TEST: <name>` marker".to_string())),
            Some(name) if !known.contains(&name) => {
                found.push((index + 1, format!("marker names unknown test `{name}`")));
            }
            Some(_) => {}
        }
    }
    found
}

/// The source before its `#[cfg(test)]` module: test code calls unsafe
/// functions but defines no primitive.
fn production_part(source: &str) -> &str {
    source
        .find("\n#[cfg(test)]")
        .map_or(source, |end| &source[..end])
}

/// Every `fn <name>` in a source text.
fn fn_names(source: &str, into: &mut HashSet<String>) {
    for (pos, _) in source.match_indices("fn ") {
        if pos > 0 && is_ident(source.as_bytes()[pos - 1]) {
            continue;
        }
        let name: String = source[pos + 3..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            into.insert(name);
        }
    }
}

fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_unsafe_block_has_a_test_reference() {
    let root = root();
    let mut known = HashSet::new();
    let mut everything = Vec::new();
    rust_files(&root.join("crates"), &mut everything);
    rust_files(&root.join("tests"), &mut everything);
    rust_files(&root.join("src"), &mut everything);
    for file in &everything {
        fn_names(&fs::read_to_string(file).unwrap_or_default(), &mut known);
    }

    let mut report = Vec::new();
    for file in everything.iter().filter(|f| {
        let text = f.to_string_lossy();
        text.contains("/src/") && text.contains("crates/")
    }) {
        let source = fs::read_to_string(file).unwrap_or_default();
        for (line, why) in problems(production_part(&source), &known) {
            report.push(format!("{}:{line}: {why}", file.display()));
        }
    }
    assert!(
        report.is_empty(),
        "unsafe without a test:\n{}",
        report.join("\n")
    );
}

fn known(names: &[&str]) -> HashSet<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

#[test]
fn marker_missing_is_reported() {
    let source = "fn f() {\n    let x = unsafe { g() };\n}\n";

    let found = problems(source, &known(&["t"]));

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, 2);
}

#[test]
fn marker_naming_unknown_test_is_reported() {
    let source = "// TEST: nope\nunsafe fn f() {}\n";

    let found = problems(source, &known(&["t"]));

    assert_eq!(found.len(), 1);
    assert!(found[0].1.contains("nope"));
}

#[test]
fn marker_on_previous_lines_is_accepted() {
    let source = "// TEST: t\n// SAFETY: fine\nunsafe impl Send for X {}\nlet y = unsafe { g() }; // TEST: t\n";

    let found = problems(source, &known(&["t"]));

    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn test_module_is_not_scanned() {
    let source = "fn a() {}\n#[cfg(test)]\nmod tests { fn t() { unsafe { g() }; } }\n";

    let found = problems(production_part(source), &known(&[]));

    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn unsafe_in_comment_is_ignored() {
    let source = "// unsafe { nothing }\nlet a = 1; // unsafe fn no\n#![allow(unsafe_code)]\nlet unsafe_len = 2;\n";

    let found = problems(source, &known(&[]));

    assert!(found.is_empty(), "{found:?}");
}
