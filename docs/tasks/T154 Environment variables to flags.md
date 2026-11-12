---
id: T154
title: Environment variables to flags
status: todo
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
- Read by test code only: `DRAW_UPDATE_SNAPSHOTS`, `DRAW_E2E_RUNTIME`, `DRAW_E2E_BACKEND`.
- [[T152 Command line and DRAW_* variables audit]] has `--help`, `--version`, `--recover` and a guard test of the documentation.

## Spec

See [[ADR-T154-1 Every DRAW_ variable the binary reads becomes a flag]]. Step 1 lists every place that reads a variable or builds a command with one (`grep` over the source, `scripts/`, `.github/`, `packaging/`, `tests/`).

- **AC-1** — A pure parser turns the arguments into `Options`; a bad value or an unknown flag is refused with exit 2 and a message. *Tests:* one per flag for a good value, a bad value and a missing value; `--log` and the others also in any order; `--recover` still works.
- **AC-2** — No code in `src/` or `crates/` reads `std::env::var` for a `DRAW_` name. *Test:* a source guard `no_draw_env_in_the_binary`.
- **AC-3** — `Options` is built once before the preflight and passed down; the preflight, the log, the backend choice, the MSAA, the tablet mode and the frame timer use it. *Tests:* the existing unit tests of each, now over `Options`; `choose` of the tablet and `backend` stay pure.
- **AC-4** — `--help` lists the user flags and not the hidden ones; the README and `docs/` user text name only flags. *Test:* the guard of T152, updated.
- **AC-5** — `scripts/e2e.sh`, `tests/e2e.rs`, the CI workflow, `scripts/release.sh` and the packages pass the flags and the end-to-end suite still passes on Wayland and X11. *Tests:* `scripts/e2e.sh --backend all`.
- **AC-6** — The changelog names the change, with the old and new spelling.

## Out of scope

- New flags; changing what a setting does; the test-only variables.

## Files owned

- `src/main.rs`, `src/shell/**`, `tests/e2e.rs`, `tests/e2e/**`, `tests/common/**`
- `scripts/e2e.sh`, `scripts/release.sh`, `.github/**`, `packaging/**`
- `README.md`, `docs/features/**` (the lines naming the variables), `CHANGELOG.md` is the integrator's
- this note, `ADR-T154-1`

## Subtasks (one commit each)

- [ ] spec — `docs(T154): specify the flags`
- [ ] tests — `test(T154): add failing tests for the options`
- [ ] models — `feat(T154): parse the arguments into Options`
- [ ] behaviour — `refactor(T154): read Options instead of the environment`
- [ ] quality — `chore(T154): pass clippy and rustfmt`
- [ ] docs — `docs(T154): document the flags`

## Log

- 2026-11-12 — created after the owner chose flags over variables, with no fallback.
