//! Static checks of the GitHub Actions workflows (T73): they call the same
//! gates as `scripts/check.sh`, build on the declared `rust-version`, and
//! pin every action. The workflows are read as plain text, line by line,
//! so no YAML parser is needed (ADR-0014).

use std::path::Path;

const CI: &str = ".github/workflows/ci.yml";
const NIGHTLY: &str = ".github/workflows/nightly.yml";

fn read(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

/// Number of leading spaces.
fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// Lines nested under the first line equal to `header` at `level` spaces
/// of indentation, up to the next line at that level or less.
fn block<'a>(text: &'a str, level: usize, header: &str) -> Vec<&'a str> {
    let mut lines = text.lines();
    let found = lines
        .by_ref()
        .any(|line| indent(line) == level && line.trim_end() == format!("{}{header}", " ".repeat(level)));
    assert!(found, "no `{header}` at indent {level}");
    lines
        .take_while(|line| line.trim().is_empty() || indent(line) > level)
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .collect()
}

/// The lines of job `name` under `jobs:`.
fn job<'a>(text: &'a str, name: &str) -> Vec<&'a str> {
    let jobs = block(text, 0, "jobs:");
    let header = format!("  {name}:");
    let mut lines = jobs.into_iter();
    assert!(
        lines.by_ref().any(|line| line.trim_end() == header),
        "no job `{name}`"
    );
    lines.take_while(|line| indent(line) > 2).collect()
}

/// The value after `key:` on a trimmed line, without quotes or comments.
fn value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let rest = line.trim().trim_start_matches("- ").strip_prefix(key)?;
    let rest = rest.strip_prefix(':')?;
    let rest = rest.split(" #").next().unwrap_or(rest);
    Some(rest.trim().trim_matches(|c| c == '"' || c == '\''))
}

fn values<'a>(lines: &[&'a str], key: &str) -> Vec<&'a str> {
    lines.iter().filter_map(|line| value(line, key)).collect()
}

/// `1.99.1` → `[1, 99, 1]`.
fn version(text: &str) -> Vec<u32> {
    text.split('.')
        .map(|part| part.parse().unwrap_or_else(|err| panic!("version {text}: {err}")))
        .collect()
}

/// The `rust-version` of `Cargo.toml`.
fn rust_version() -> String {
    read("Cargo.toml")
        .lines()
        .find_map(|line| {
            let rest = line.strip_prefix("rust-version")?.trim_start().strip_prefix('=')?;
            Some(rest.trim().trim_matches('"').to_owned())
        })
        .unwrap_or_else(|| panic!("Cargo.toml has no rust-version"))
}

#[test]
fn ci_workflow_runs_on_push_and_pull_request() {
    // Arrange
    let ci = read(CI);

    // Act
    let on = block(&ci, 0, "on:");

    // Assert
    assert!(on.iter().any(|l| l.trim() == "push:"), "{on:?}");
    assert!(on.iter().any(|l| l.trim() == "pull_request:"), "{on:?}");
    assert!(on.iter().any(|l| l.contains("main")), "pull requests to main: {on:?}");
    assert!(ci.lines().any(|l| l.trim_end() == "concurrency:"), "older runs are cancelled");
    assert!(ci.contains("cancel-in-progress: true"));
}

#[test]
fn ci_gates_job_runs_check_script() {
    // Arrange
    let ci = read(CI);

    // Act
    let gates = job(&ci, "gates");

    // Assert
    assert!(values(&gates, "run").contains(&"scripts/check.sh"), "{gates:?}");
    assert_eq!(values(&gates, "toolchain"), ["stable"]);
    let components = values(&gates, "components").join(",");
    assert!(components.contains("rustfmt") && components.contains("clippy"), "{components}");
}

#[test]
fn ci_msrv_job_uses_cargo_rust_version() {
    // Arrange
    let ci = read(CI);
    let declared = rust_version();

    // Act
    let msrv = job(&ci, "msrv");

    // Assert
    assert_eq!(values(&msrv, "toolchain"), [declared.as_str()]);
    assert!(
        values(&msrv, "run").contains(&"cargo check --all-targets --locked"),
        "{msrv:?}"
    );
}

#[test]
fn containerfile_rust_is_at_least_rust_version() {
    // Arrange
    let declared = version(&rust_version());

    // Act
    let pinned = read("packaging/Containerfile")
        .lines()
        .find_map(|line| line.strip_prefix("ARG RUST_VERSION=").map(version))
        .unwrap_or_else(|| panic!("Containerfile has no RUST_VERSION"));

    // Assert: cargo refuses a toolchain older than rust-version.
    assert!(pinned >= declared, "{pinned:?} < {declared:?}");
}

#[test]
fn readme_build_requirement_names_rust_version() {
    // Arrange
    let expected = format!("Rust **{}** or newer", rust_version());

    // Act
    let readme = read("README.md");

    // Assert
    assert!(readme.contains(&expected), "README should say {expected}");
}

#[test]
fn nightly_runs_fuzz_and_soak_on_schedule_and_dispatch() {
    // Arrange
    let nightly = read(NIGHTLY);

    // Act
    let on = block(&nightly, 0, "on:");
    let fuzz = job(&nightly, "fuzz");
    let soak = job(&nightly, "soak");

    // Assert
    assert!(on.iter().any(|l| l.trim() == "schedule:"), "{on:?}");
    assert_eq!(values(&on, "cron").len(), 1, "{on:?}");
    assert!(on.iter().any(|l| l.trim() == "workflow_dispatch:"), "{on:?}");
    assert!(values(&fuzz, "run").contains(&"scripts/check.sh --fuzz"), "{fuzz:?}");
    let soak_run = values(&soak, "run").join(" ");
    assert!(soak_run.contains("--release --test soak"), "{soak_run}");
    assert!(soak_run.contains("--ignored") && soak_run.contains("--test-threads=1"), "{soak_run}");
}

#[test]
fn nightly_uploads_fuzz_regressions_on_failure() {
    // Arrange
    let nightly = read(NIGHTLY);

    // Act
    let fuzz = job(&nightly, "fuzz");

    // Assert
    assert!(values(&fuzz, "if").contains(&"failure()"), "{fuzz:?}");
    assert!(
        values(&fuzz, "uses").iter().any(|u| u.starts_with("actions/upload-artifact@")),
        "{fuzz:?}"
    );
    assert!(values(&fuzz, "path").contains(&"tests/fuzz.proptest-regressions"), "{fuzz:?}");
}

#[test]
fn workflow_actions_are_pinned_to_full_sha() {
    for path in [CI, NIGHTLY] {
        // Arrange
        let text = read(path);
        let lines: Vec<&str> = text.lines().collect();

        // Act
        let uses = values(&lines, "uses");

        // Assert
        assert!(!uses.is_empty(), "{path}: no actions");
        for action in uses {
            let (_, reference) = action
                .split_once('@')
                .unwrap_or_else(|| panic!("{path}: {action} has no ref"));
            assert!(
                reference.len() == 40 && reference.bytes().all(|b| b.is_ascii_hexdigit()),
                "{path}: {action} is not pinned to a commit SHA"
            );
        }
    }
}

#[test]
fn workflow_permissions_are_read_only() {
    for path in [CI, NIGHTLY] {
        // Arrange
        let text = read(path);

        // Act
        let permissions = block(&text, 0, "permissions:");

        // Assert
        assert_eq!(
            permissions.iter().map(|l| l.trim()).collect::<Vec<_>>(),
            ["contents: read"],
            "{path}"
        );
        assert!(!text.contains(": write"), "{path}: no write permission");
    }
}

#[test]
fn readme_has_ci_badge() {
    // Arrange
    let badge = "https://github.com/UmBender/draw/actions/workflows/ci.yml/badge.svg";

    // Act
    let readme = read("README.md");

    // Assert
    assert!(readme.contains(badge), "README has no CI badge");
}
