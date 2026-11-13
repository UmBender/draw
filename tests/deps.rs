//! Guard: no crate of the workspace has a dependency that `ADR-T127-3` and
//! `ADR-T132-1` do not list.

use std::fs;
use std::path::PathBuf;

/// The crate whose root manifest is `Cargo.toml`, and the member crates.
const CRATES: [&str; 5] = ["draw", "draw-types", "draw-core", "draw-gfx", "draw-sys"];

/// The workspace crates; a path dependency may only name one of these.
const OWN: [&str; 5] = CRATES;

/// What each crate may depend on at run time (ADR-T132-1).
fn runtime_allowed(krate: &str) -> &'static [&'static str] {
    match krate {
        "draw" => &[
            "macroquad",
            "fontdue",
            "png",
            "ttf-parser",
            "draw-types",
            "draw-core",
            "draw-gfx",
            "draw-sys",
        ],
        "draw-core" => &["fontdue", "png", "ttf-parser", "draw-types"],
        // `draw-sys` gives the GL contexts of the `Gpu` (T145).
        "draw-gfx" => &["draw-types", "draw-sys"],
        // `draw-types` and `draw-sys` have none, and a crate not listed may not
        // have any either.
        _ => &[],
    }
}

/// What each crate may depend on for its tests (ADR-T127-3).
fn dev_allowed(_krate: &str) -> &'static [&'static str] {
    &["proptest"]
}

fn manifest_path(krate: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if krate == "draw" {
        root.join("Cargo.toml")
    } else {
        root.join("crates").join(krate).join("Cargo.toml")
    }
}

/// One dependency found in a manifest.
#[derive(Debug, PartialEq, Eq)]
struct Dep {
    name: String,
    /// `dependencies`, `dev-dependencies` or `build-dependencies`.
    table: String,
    path: bool,
}

/// The dependency kind of a table header, if it is a dependency table:
/// `[dependencies]`, `[dev-dependencies]`, `[target.x.dependencies]`, and
/// the `[dependencies.name]` form, which also gives the name.
fn table_kind(header: &str) -> Option<(String, Option<String>)> {
    let inner = header.trim_start_matches('[').trim_end_matches(']').trim();
    for kind in ["dev-dependencies", "build-dependencies", "dependencies"] {
        if inner == kind || (inner.starts_with("target.") && inner.ends_with(kind)) {
            return Some((kind.to_owned(), None));
        }
        if let Some(rest) = inner.strip_prefix(&format!("{kind}.")) {
            return Some((kind.to_owned(), Some(rest.trim_matches('"').to_owned())));
        }
    }
    None
}

/// Every dependency of a manifest, from every dependency table.
fn read_deps(manifest: &str) -> Vec<Dep> {
    let mut deps: Vec<Dep> = Vec::new();
    let mut current: Option<(String, Option<String>)> = None;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            current = table_kind(line);
            if let Some((table, Some(name))) = &current {
                deps.push(Dep {
                    name: name.clone(),
                    table: table.clone(),
                    path: false,
                });
            }
            continue;
        }
        let Some((table, named)) = &current else {
            continue;
        };
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        match named {
            Some(_) => {
                if key == "path"
                    && let Some(last) = deps.last_mut()
                {
                    last.path = true;
                }
            }
            None => deps.push(Dep {
                name: key.split('.').next().unwrap_or(key).to_owned(),
                table: table.clone(),
                path: value.contains("path"),
            }),
        }
    }
    deps
}

#[test]
fn reader_finds_every_dependency_table() {
    let manifest = r#"
[package]
name = "x"
[dependencies]
fontdue = "0.9"
inner = { path = "../inner" }
png.workspace = true
[dev-dependencies]
proptest = "1"
[target.'cfg(unix)'.dependencies]
libc = "0.2"
[dependencies.serde]
version = "1"
[lints.rust]
unsafe_code = "forbid"
"#;

    let deps = read_deps(manifest);

    let names: Vec<(&str, &str, bool)> = deps
        .iter()
        .map(|d| (d.name.as_str(), d.table.as_str(), d.path))
        .collect();
    assert_eq!(
        names,
        vec![
            ("fontdue", "dependencies", false),
            ("inner", "dependencies", true),
            ("png", "dependencies", false),
            ("proptest", "dev-dependencies", false),
            ("libc", "dependencies", false),
            ("serde", "dependencies", false),
        ]
    );
}

#[test]
fn deps_match_the_adr() {
    let mut extra = Vec::new();
    for krate in CRATES {
        let path = manifest_path(krate);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for dep in read_deps(&text) {
            let allowed = if dep.table == "dev-dependencies" {
                dev_allowed(krate)
            } else {
                runtime_allowed(krate)
            };
            if dep.table == "build-dependencies" || !allowed.contains(&dep.name.as_str()) {
                extra.push(format!("{krate}: {} in [{}]", dep.name, dep.table));
            }
        }
    }

    assert!(
        extra.is_empty(),
        "dependencies not in ADR-T127-3 / ADR-T132-1:\n{}",
        extra.join("\n")
    );
}

#[test]
fn path_dependencies_stay_between_draw_crates() {
    let mut bad = Vec::new();
    for krate in CRATES {
        let path = manifest_path(krate);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for dep in read_deps(&text).iter().filter(|d| d.path) {
            if !OWN.contains(&dep.name.as_str()) {
                bad.push(format!("{krate}: {}", dep.name));
            }
        }
    }

    assert!(
        bad.is_empty(),
        "path dependencies outside the workspace:\n{}",
        bad.join("\n")
    );
}
