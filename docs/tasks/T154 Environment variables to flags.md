---
id: T154
title: Environment variables to flags
status: review
wave: 72
branch: task/T154-environment-variables-to-flags
depends_on: [T152, T129]
adrs: [ADR-T154-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T154 Environment variables to flags

## Goal

The binary reads nothing from the environment: every `DRAW_*` variable it read is a flag with a checked value, parsed once into a typed `Options` value.

## Known facts (2026-11-12)

- Variables read by the binary: `DRAW_LOG`, `DRAW_MSAA`, `DRAW_FRAME_TIMES`, `DRAW_BACKEND`, `DRAW_SKIP_PREFLIGHT`, `DRAW_TABLET`, `DRAW_SNAPSHOT`, `DRAW_E2E_DUMP`, `DRAW_E2E_IDLE`, `DRAW_E2E_CLIPBOARD`; the code is in `src/main.rs`, `src/shell/{app,backend,tablet,preflight}.rs` and `src/shell/diag/**`.
- Read by test code and scripts only, so they stay: `DRAW_UPDATE_SNAPSHOTS`, `DRAW_E2E_RUNTIME`, `DRAW_E2E_BACKEND`, `DRAW_E2E_CASE`, `DRAW_E2E_IN_SCOPE`, `DRAW_RELEASE`, `DRAW_RELEASE_FAIL`.
- `scripts/audit/frames.sh` runs the app with `DRAW_FRAME_TIMES=1` and the flatpak manifest comment names `DRAW_BACKEND=x11`.
- [[T152 Command line and DRAW_* variables audit]] has `--help`, `--version`, `--recover` and a guard test of the documentation.

## Spec

See [[ADR-T154-1 Every DRAW_ variable the binary reads becomes a flag]]. Step 1 lists every place that reads a variable or builds a command with one (`grep` over the source, `scripts/`, `.github/`, `packaging/`, `tests/`).

Design: `src/shell/options.rs` holds `Options` and the pure `parse(args) -> Parsed` (`Run(Options)`, `Help`, `Version`, `Refused(String)`). A flag takes its value as `--flag value` or `--flag=value`. The words left after the flags go to `Launch::from_args` (`--recover [prev]`) and `path_from_args` unchanged. `Options` is built in `main` and passed down: `diag::init`, `preflight::run`, `backend::choose`, `tablet::choose`, `app::window_conf` and `app::run` take it, and `backend::from_env`, `tablet::from_env`, `Hooks::from_env` and `diag::init_from_env` are removed.

- **AC-1** — The parser turns the arguments into `Options`; a bad or missing value and an unknown flag are refused. *Tests* (`options::tests`): `defaults_without_arguments`, `log_takes_every_level`, `log_refuses_an_unknown_level`, `msaa_takes_off_and_the_counts`, `msaa_refuses_other_counts`, `backend_takes_x11_and_wayland`, `backend_refuses_an_unknown_name`, `tablet_takes_auto_on_off`, `tablet_refuses_an_unknown_mode`, `frame_times_and_skip_preflight_take_no_value`, `a_flag_without_its_value_is_refused`, `value_with_equals_sign_is_accepted`, `flags_in_any_order_and_with_a_file_and_recover`, `hidden_flags_take_paths`, `unknown_flag_is_refused`, `help_and_version_win`; in `main`, a refused parse exits 2 (the T152 test, updated).
- **AC-2** — No code in `src/` or `crates/` reads a `DRAW_` variable. *Test:* `tests/no_draw_env.rs::the_binary_reads_no_draw_variable` (a source guard over `src/**` and `crates/**`).
- **AC-3** — `Options` is built once and passed down. *Tests:* `backend::choose` over `Option<Backend>` and the session (`choose_defaults_to_wayland_in_a_wayland_session`, `choose_honours_the_flag`), `tablet::choose` over `TabletMode` and the device list (`choose_on_forces_pen_mode`, `choose_off_turns_it_off`, `choose_auto_detects_a_pen`), `preflight` with `skip` as a bool (`check_skips_when_asked`, hint names `--skip-preflight`), `app::startup_snapshot` over a path (`startup_snapshot_reads_the_path`), `e2e::Hooks::from_paths`, `diag::init` with the level from `Options`.
- **AC-4** — `--help` lists the user flags and not the hidden ones, and the user documentation names no `DRAW_` variable. *Tests:* `help_lists_the_flags`, `help_hides_the_test_flags`, `readme_names_no_draw_variable` (replaces the guard of T152).
- **AC-5** — `tests/e2e.rs`, `scripts/audit/frames.sh` and the packaging text pass the flags; the end-to-end suite passes on Wayland and X11. *Tests:* `scripts/e2e.sh --backend all`; `e2e_command_passes_flags_not_variables` in `tests/e2e.rs`.
- **AC-6** — The README's Debugging section lists the flags; the changelog (integrator) names the change with the old and new spelling.

## Out of scope

- New flags; changing what a setting does; the test-only variables.

## Files owned

- `src/main.rs`, `src/shell/**`, `tests/e2e.rs`, `tests/e2e/**`, `tests/common/**`
- `scripts/audit/frames.sh`, `.github/**`, `packaging/**`, `tests/no_draw_env.rs`, `tests/options.rs`, `tests/skeleton.rs` (one call of `window_conf`), `docs/architecture/Architecture.md` (two lines)
- `README.md`, `docs/features/**` (the lines naming the variables), `CHANGELOG.md` is the integrator's
- this note, `ADR-T154-1`

## Subtasks (one commit each)

- [x] spec — `docs(T154): specify the flags`
- [x] tests — `test(T154): add failing tests for the options`
- [x] models — `feat(T154): parse the arguments into Options`
- [x] behaviour — `refactor(T154): read Options instead of the environment`
- [x] quality — `chore(T154): pass clippy and rustfmt`
- [x] docs — `docs(T154): document the flags`

## Log

- 2026-11-12 — created after the owner chose flags over variables, with no fallback.
- 2026-11-12 — spec: parser design and test names; `scripts/audit/frames.sh` and `tests/no_draw_env.rs` added to the files owned (the first still runs the app with the variable); `scripts/e2e.sh` and `scripts/release.sh` only use variables the binary does not read, so they are not touched.
- 2026-11-12 — done. `src/shell/options.rs` holds `Options` and `parse`; `main` builds it once, picks the backend and the pen mode, and passes them to `diag::init`, `preflight::run`, `window_conf` and `app::run`. Removed: `DRAW_*` constants, `backend::from_env`, `tablet::from_env`, `Hooks::from_env`, `diag::init_from_env`, `parse_level`, `msaa_samples`, `frame_timing_enabled`, the `Choice` warnings of backend and tablet. A bad value is now refused (exit 2) instead of warned about and replaced by the default. `scripts/check.sh` green; `scripts/e2e.sh --backend all` green on Wayland and X11 with the flags.
- 2026-11-12 — deviations: models and a first part of behaviour (`Level::from_name`, `Backend::from_name`) share the parser commit's neighbours; the module unit tests whose signatures changed (`backend`, `tablet`, `preflight`, `app`, `diag`, `e2e`) were adapted in the behaviour commit, not the test commit; `tests/skeleton.rs` and `Architecture.md` touched (added to the files owned); `--recover` followed by a flag now reads no value (`--recover --log warn`), where it used to take the next word whatever it was.
- 2026-11-12 — not changed on purpose: the tutorials and the older ADRs, tasks and the 2.0 baseline still name the variables as history. `scripts/e2e.sh`, `scripts/release.sh` and `scripts/package.sh` only use `DRAW_E2E_*` and `DRAW_RELEASE*` variables that the binary does not read. Owner: try `draw --frame-times`, `draw --backend x11`, `draw --msaa 3` (refused, exit 2) and `draw --help`.
