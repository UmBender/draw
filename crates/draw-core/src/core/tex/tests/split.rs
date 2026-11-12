//! Tests of the TeX subset.

// ---- T139 module split (AC-1, AC-2) ----

/// Every `.rs` file under `core/tex/`, recursively.
fn tex_sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        panic!("{} is not readable", dir.display());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            tex_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn modules_stay_under_the_limit() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/core/tex");
    let mut files = Vec::new();
    tex_sources(&root, &mut files);
    for name in ["parse", "layout", "metrics", "glyphs", "macros"] {
        assert!(
            files.iter().any(|f| f.ends_with(format!("{name}.rs"))),
            "module {name} is missing"
        );
    }
    for file in files {
        let Ok(source) = std::fs::read_to_string(&file) else {
            panic!("{} is not readable utf-8", file.display());
        };
        let lines = source.lines().count();
        assert!(lines <= 3000, "{} has {lines} lines", file.display());
    }
}
