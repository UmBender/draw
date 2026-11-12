//! Static checks of the GitHub Actions workflows (T73): they call the same
//! gates as `scripts/check.sh`, build on the declared `rust-version`, and
//! pin every action. The workflows are read as plain text, line by line,
//! so no YAML parser is needed (ADR-0014).
//!
//! Also runs `scripts/check.sh` with stand-ins for `cargo`, `systemd-run`
//! and `systemctl`, to check the limits every step gets (T77).

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

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
    let found = lines.by_ref().any(|line| {
        indent(line) == level && line.trim_end() == format!("{}{header}", " ".repeat(level))
    });
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
        .map(|part| {
            part.parse()
                .unwrap_or_else(|err| panic!("version {text}: {err}"))
        })
        .collect()
}

/// The `rust-version` of `Cargo.toml`.
fn rust_version() -> String {
    read("Cargo.toml")
        .lines()
        .find_map(|line| {
            let rest = line
                .strip_prefix("rust-version")?
                .trim_start()
                .strip_prefix('=')?;
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
    assert!(
        on.iter().any(|l| l.contains("main")),
        "pull requests to main: {on:?}"
    );
    assert!(
        ci.lines().any(|l| l.trim_end() == "concurrency:"),
        "older runs are cancelled"
    );
    assert!(ci.contains("cancel-in-progress: true"));
}

#[test]
fn ci_gates_job_runs_check_script() {
    // Arrange
    let ci = read(CI);

    // Act
    let gates = job(&ci, "gates");

    // Assert
    assert!(
        values(&gates, "run").contains(&"scripts/check.sh"),
        "{gates:?}"
    );
    assert_eq!(values(&gates, "toolchain"), ["stable"]);
    let components = values(&gates, "components").join(",");
    assert!(
        components.contains("rustfmt") && components.contains("clippy"),
        "{components}"
    );
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
    assert!(
        on.iter().any(|l| l.trim() == "workflow_dispatch:"),
        "{on:?}"
    );
    assert!(
        values(&fuzz, "run").contains(&"scripts/check.sh --fuzz"),
        "{fuzz:?}"
    );
    let soak_run = values(&soak, "run").join(" ");
    assert!(soak_run.contains("--release --test soak"), "{soak_run}");
    assert!(
        soak_run.contains("--ignored") && soak_run.contains("--test-threads=1"),
        "{soak_run}"
    );
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
        values(&fuzz, "uses")
            .iter()
            .any(|u| u.starts_with("actions/upload-artifact@")),
        "{fuzz:?}"
    );
    assert!(
        values(&fuzz, "path").contains(&"tests/fuzz.proptest-regressions"),
        "{fuzz:?}"
    );
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

/// Records one line per step: its arguments and what it runs under.
const FAKE_CARGO: &str = r#"#!/usr/bin/env bash
printf '%s | jobs=%s | threads=%s | nice=%s | io=%s | scope=%s\n' "$*" \
    "${CARGO_BUILD_JOBS:-}" "${RUST_TEST_THREADS:-}" "$(nice)" "$(ionice)" \
    "${GATE_SCOPE:-}" >>"$GATE_LOG"
[[ "$1" != "${FAKE_CARGO_FAILS:-}" ]] || exit 101
if [[ "$1" == llvm-cov && -n "${FAKE_SUMMARY:-}" ]]; then
    while (($#)); do
        if [[ "$1" == --output-path ]]; then cp "$FAKE_SUMMARY" "$2"; fi
        shift
    done
fi
"#;

/// Runs the command after `--`, passing its own options on to it.
const FAKE_SYSTEMD_RUN: &str = r#"#!/usr/bin/env bash
[[ -z "${FAKE_SCOPE_FAILS:-}" ]] || exit 1
options=()
while (($#)) && [[ "$1" != -- ]]; do options+=("$1"); shift; done
shift
GATE_SCOPE="${options[*]}" exec "$@"
"#;

/// Answers `show` with the result the test asked for.
const FAKE_SYSTEMCTL: &str = r#"#!/usr/bin/env bash
[[ " $* " != *" show "* ]] || printf '%s\n' "${FAKE_SCOPE_RESULT:-success}"
"#;

/// The `cargo` subcommands of `check.sh --fuzz`, in order.
const GATES: [&str; 5] = ["fmt", "clippy", "test", "doc", "test"];

/// The directory of the stand-ins. They are written once, before any
/// test starts a process: a script still open for writing in a forked
/// child could not be executed (`ETXTBSY`).
fn fake_tools() -> &'static Path {
    static TOOLS: OnceLock<PathBuf> = OnceLock::new();
    TOOLS.get_or_init(|| {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("gate-tools");
        std::fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display()));
        for (name, body) in [
            ("cargo", FAKE_CARGO),
            ("systemd-run", FAKE_SYSTEMD_RUN),
            ("systemctl", FAKE_SYSTEMCTL),
        ] {
            let path = dir.join(name);
            std::fs::write(&path, body).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        }
        dir
    })
}

/// What one run of `check.sh --fuzz` did.
struct GateRun {
    code: Option<i32>,
    /// Standard output followed by standard error.
    output: String,
    /// One recorded line per `cargo` step.
    steps: Vec<String>,
}

/// Runs `check.sh --fuzz` with the stand-ins, in an environment holding
/// none of the limits, so that they can only come from the script.
fn run_gates(test: &str, env: &[(&str, &str)]) -> GateRun {
    let tools = fake_tools();
    let log = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("gate-{test}.log"));
    if log.exists() {
        std::fs::remove_file(&log).unwrap_or_else(|err| panic!("{}: {err}", log.display()));
    }
    let path = std::env::var("PATH").unwrap_or_else(|err| panic!("PATH: {err}"));
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/check.sh");

    let result = Command::new(script)
        .arg("--fuzz")
        .env("PATH", format!("{}:{path}", tools.display()))
        .env("GATE_LOG", &log)
        .env_remove("CARGO_BUILD_JOBS")
        .env_remove("RUST_TEST_THREADS")
        .envs(env.iter().copied())
        .output()
        .unwrap_or_else(|err| panic!("check.sh: {err}"));

    let steps = std::fs::read_to_string(&log).unwrap_or_default();
    GateRun {
        code: result.status.code(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ),
        steps: steps.lines().map(str::to_owned).collect(),
    }
}

/// The value of `name` in a recorded step.
fn field<'a>(step: &'a str, name: &str) -> &'a str {
    let prefix = format!("{name}=");
    step.split(" | ")
        .find_map(|part| part.strip_prefix(prefix.as_str()))
        .unwrap_or_else(|| panic!("no {name} in {step}"))
}

/// The `cargo` subcommand of a recorded step.
fn subcommand(step: &str) -> &str {
    step.split(' ').next().unwrap_or_default()
}

fn subcommands(run: &GateRun) -> Vec<&str> {
    run.steps.iter().map(|step| subcommand(step)).collect()
}

#[test]
fn gates_limit_threads() {
    // Arrange, Act
    let run = run_gates("threads", &[]);

    // Assert
    assert_eq!(run.code, Some(0), "{}", run.output);
    assert_eq!(subcommands(&run), GATES);
    for step in &run.steps {
        assert_eq!(field(step, "jobs"), "2", "{step}");
        assert_eq!(field(step, "threads"), "3", "{step}");
    }
}

#[test]
fn gates_run_under_a_memory_cap() {
    // Arrange, Act
    let run = run_gates("cap", &[]);

    // Assert
    assert_eq!(run.code, Some(0), "{}", run.output);
    assert_eq!(subcommands(&run), GATES);
    for step in &run.steps {
        let scope: Vec<&str> = field(step, "scope").split(' ').collect();
        for option in [
            "--user",
            "--scope",
            "MemoryMax=1536M",
            "MemorySwapMax=0",
            "CPUQuota=300%",
        ] {
            assert!(scope.contains(&option), "{option} missing: {step}");
        }
    }
}

#[test]
fn gates_run_at_low_priority() {
    // Arrange, Act
    let run = run_gates("priority", &[]);

    // Assert
    assert_eq!(run.code, Some(0), "{}", run.output);
    assert_eq!(subcommands(&run), GATES);
    for step in &run.steps {
        assert_eq!(field(step, "nice"), "19", "{step}");
        assert_eq!(field(step, "io"), "idle", "{step}");
        let scope: Vec<&str> = field(step, "scope").split(' ').collect();
        for option in ["CPUWeight=1", "IOWeight=1"] {
            assert!(scope.contains(&option), "{option} missing: {step}");
        }
    }
}

#[test]
fn gates_warn_without_a_memory_cap() {
    // Arrange, Act
    let run = run_gates("no-cap", &[("FAKE_SCOPE_FAILS", "1")]);

    // Assert
    assert_eq!(run.code, Some(0), "{}", run.output);
    assert_eq!(subcommands(&run), GATES);
    for step in &run.steps {
        assert_eq!(field(step, "scope"), "", "{step}");
        assert_eq!(field(step, "jobs"), "2", "{step}");
        assert_eq!(field(step, "threads"), "3", "{step}");
        assert_eq!(field(step, "nice"), "19", "{step}");
        assert_eq!(field(step, "io"), "idle", "{step}");
    }
    assert!(
        run.output
            .lines()
            .any(|l| l.contains("memory cap") && l.contains("not enforced")),
        "{}",
        run.output
    );
}

#[test]
fn gates_name_step_and_limit_when_out_of_memory() {
    // Arrange, Act
    let run = run_gates(
        "out-of-memory",
        &[
            ("FAKE_CARGO_FAILS", "clippy"),
            ("FAKE_SCOPE_RESULT", "oom-kill"),
        ],
    );

    // Assert
    assert_eq!(run.code, Some(101), "{}", run.output);
    assert_eq!(subcommands(&run), ["fmt", "clippy"]);
    assert!(
        run.output
            .lines()
            .any(|l| l.contains("clippy") && l.contains("1536M") && l.contains("memory limit")),
        "{}",
        run.output
    );
}

#[test]
fn gates_blame_no_limit_for_an_ordinary_failure() {
    // Arrange, Act
    let run = run_gates("ordinary-failure", &[("FAKE_CARGO_FAILS", "clippy")]);

    // Assert
    assert_eq!(run.code, Some(101), "{}", run.output);
    assert_eq!(subcommands(&run), ["fmt", "clippy"]);
    assert!(!run.output.contains("memory limit"), "{}", run.output);
}

// ---- coverage ratchet (T130) ----

/// A `cargo llvm-cov --json` summary: `(file, lines, covered)`.
fn summary_json(files: &[(&str, u32, u32)]) -> String {
    let files: Vec<String> = files
        .iter()
        .map(|(name, count, covered)| {
            format!(
                r#"{{"filename":"/work/draw/{name}","summary":{{"lines":{{"count":{count},"covered":{covered}}}}}}}"#
            )
        })
        .collect();
    format!(r#"{{"data":[{{"files":[{}]}}]}}"#, files.join(","))
}

/// What one run of `coverage.sh` did.
struct CoverageRun {
    code: Option<i32>,
    /// Standard output followed by standard error.
    output: String,
    /// The recorded `cargo` steps.
    steps: Vec<String>,
}

/// Runs `coverage.sh` with the stand-ins. `summary` is the JSON the fake
/// `cargo llvm-cov` writes (when `use_flag` is false) or the file passed
/// with `--summary`; `baseline` is the content of the baseline file.
fn run_coverage(test: &str, summary: &str, baseline: &str, use_flag: bool) -> CoverageRun {
    let tools = fake_tools();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("coverage-{test}"));
    std::fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display()));
    let write = |name: &str, text: &str| {
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        path
    };
    let summary_path = write("summary.json", summary);
    let baseline_path = write("baseline.toml", baseline);
    let log = dir.join("cargo.log");
    if log.exists() {
        std::fs::remove_file(&log).unwrap_or_else(|err| panic!("{}: {err}", log.display()));
    }
    let path = std::env::var("PATH").unwrap_or_else(|err| panic!("PATH: {err}"));
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/coverage.sh");

    let mut command = Command::new("bash");
    command
        .arg(script)
        .arg("--baseline")
        .arg(&baseline_path)
        .env("PATH", format!("{}:{path}", tools.display()))
        .env("GATE_LOG", &log)
        .env("FAKE_SUMMARY", &summary_path)
        .env_remove("CARGO_BUILD_JOBS")
        .env_remove("RUST_TEST_THREADS");
    if use_flag {
        command.arg("--summary").arg(&summary_path);
    }
    let result = command
        .output()
        .unwrap_or_else(|err| panic!("coverage.sh: {err}"));

    let steps = std::fs::read_to_string(&log).unwrap_or_default();
    CoverageRun {
        code: result.status.code(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ),
        steps: steps.lines().map(str::to_owned).collect(),
    }
}

/// Two `core` files at 90 % together, one `shell` file at 50 %, and a
/// file outside `src/` that is ignored.
fn sample_summary() -> String {
    summary_json(&[
        ("src/core/a.rs", 100, 90),
        ("src/core/tools/b.rs", 100, 90),
        ("src/shell/c.rs", 200, 100),
        ("tests/ci.rs", 50, 50),
    ])
}

#[test]
fn coverage_summary_is_parsed() {
    // Arrange, Act
    let run = run_coverage(
        "parsed",
        &sample_summary(),
        "core = 90.0\nshell = 50.0\n",
        true,
    );

    // Assert
    assert_eq!(run.code, Some(0), "{}", run.output);
    let lines: Vec<&str> = run.output.lines().map(str::trim).collect();
    assert!(lines.contains(&"core 90.0"), "{}", run.output);
    assert!(lines.contains(&"shell 50.0"), "{}", run.output);
    assert!(!run.output.contains("tests"), "{}", run.output);
    assert!(run.steps.is_empty(), "--summary must skip cargo");
}

#[test]
fn coverage_script_runs_under_the_caps() {
    // Arrange, Act
    let run = run_coverage(
        "caps",
        &sample_summary(),
        "core = 90.0\nshell = 50.0\n",
        false,
    );

    // Assert
    assert_eq!(run.code, Some(0), "{}", run.output);
    assert_eq!(run.steps.len(), 1, "{:?}", run.steps);
    let step = &run.steps[0];
    assert_eq!(subcommand(step), "llvm-cov");
    assert_eq!(field(step, "jobs"), "2");
    assert_eq!(field(step, "threads"), "3");
    assert_eq!(field(step, "nice"), "19");
    assert!(field(step, "scope").contains("MemoryMax=1536M"), "{step}");
}

#[test]
fn ratchet_fails_when_a_module_falls() {
    // Arrange: core is 90.0, the baseline says 91.0.
    let run = run_coverage(
        "falls",
        &sample_summary(),
        "core = 91.0\nshell = 50.0\n",
        true,
    );

    // Assert
    assert_eq!(run.code, Some(1), "{}", run.output);
    assert!(
        run.output.contains("core") && run.output.contains("fell"),
        "{}",
        run.output
    );
}

#[test]
fn ratchet_asks_to_raise_the_baseline() {
    // Arrange: shell is 50.0, the baseline says 49.0.
    let run = run_coverage(
        "raise",
        &sample_summary(),
        "core = 90.0\nshell = 49.0\n",
        true,
    );

    // Assert
    assert_eq!(run.code, Some(1), "{}", run.output);
    assert!(
        run.output.contains("shell") && run.output.contains("coverage-baseline.toml"),
        "{}",
        run.output
    );
}

#[test]
fn ratchet_passes_within_the_tolerance() {
    // Arrange: 0.5 below and 0.5 above are both accepted.
    let run = run_coverage(
        "tolerance",
        &sample_summary(),
        "core = 90.5\nshell = 49.5\n",
        true,
    );

    // Assert
    assert_eq!(run.code, Some(0), "{}", run.output);
}

#[test]
fn ratchet_fails_for_a_module_without_a_baseline() {
    // Arrange, Act
    let run = run_coverage("missing", &sample_summary(), "core = 90.0\n", true);

    // Assert
    assert_eq!(run.code, Some(1), "{}", run.output);
    assert!(
        run.output.contains("shell") && run.output.contains("no baseline"),
        "{}",
        run.output
    );
}

#[test]
fn ci_coverage_job_runs_the_ratchet() {
    // Arrange
    let text = read(CI);

    // Act
    let coverage = job(&text, "coverage");

    // Assert
    assert!(
        values(&coverage, "run").contains(&"bash scripts/coverage.sh"),
        "the coverage job must run scripts/coverage.sh"
    );
}

#[test]
fn quality_gates_name_the_coverage_ratchet() {
    // Arrange, Act
    let gates = read("docs/process/Quality Gates.md");

    // Assert
    assert!(gates.contains("scripts/coverage.sh"), "gate not listed");
    assert!(
        gates.contains("coverage-baseline.toml"),
        "baseline not named"
    );
    let done = gates
        .split("## Definition of Done")
        .nth(1)
        .unwrap_or_default();
    assert!(
        done.contains("coverage"),
        "Definition of Done does not name coverage"
    );
}
