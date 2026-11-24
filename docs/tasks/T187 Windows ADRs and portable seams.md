---
id: T187
title: Windows ADRs and portable seams
status: ready
wave: 77
branch: task/T187-windows-adrs-and-portable-seams
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, windows]
---

# T187 Windows ADRs and portable seams

## Goal

The decisions of the [[Windows port proposal]] are accepted ADRs; the
workspace compiles for `x86_64-pc-windows-msvc`, its portable tests pass
on a Windows CI job, and `check.sh` lints the Windows target from Linux.
No window yet. Linux behaves exactly as before.

## Known facts (2026-11-24)

- The owner accepted the proposal on 2026-11-24 (§8a): Windows 10 1809
  or later, `x86_64`, saves and exports in `Documents\draw`, per-user
  MSI with the `.draw` association, signed releases, Windows on ARM
  later.
- Linux-only today (measured): `draw-sys` `wayland/`, `x11/`, `egl.rs`,
  `glx.rs`, `loader.rs`, `signal.rs`; `draw-gfx/src/gpu/gl.rs`;
  `src/shell/gpu_native.rs`; `run_wayland` and `run_x11` in `main.rs`;
  the `std::os::unix` use of `autosave.rs`; `preflight.rs`; tests that
  use `std::os::unix` or start sway and Xvfb (`tests/e2e.rs`,
  `crates/draw-sys/tests/wayland*.rs`, `x11*.rs`, `egl.rs`, `glx.rs`,
  `loader.rs`, among others).
- Seven sites read `HOME` or `XDG_STATE_HOME`: `app.rs` (save, PNG and
  SVG export, state), `crash.rs`, `diag/mod.rs`.
- The root crate is `#![forbid(unsafe_code)]`: the Known Folders call
  (`SHGetKnownFolderPath`) lives in `draw-sys`, behind a safe function.
- Git for Windows checks text out with CRLF by default: without a
  `.gitattributes`, the snapshot goldens and fixtures change on the
  Windows runner and the tests fail.
- **Owner step before the run**: `rustup target add x86_64-pc-windows-msvc`
  on the Linux machine, for the new clippy gate (an unattended session
  may not install toolchains).
- **Open question 7 of the proposal**: Windows tests run only in CI; this
  task's Windows job is proven by pushing the branch.

## Spec

Draft; the spec step names the final tests.

ADRs written in the spec step (one per decision of the proposal's §3):
D3D11 as the Windows `Gpu` with WARP and offline HLSL (§3.1); the
`windows` crate on the Windows target only, amending ADR-T127-3 (§3.2);
system DLLs linked and the DLL search path (§3.3); the window thread,
an exception to the 2.0 rule on threads (§3.4); `shell::paths` with
Known Folders (§3.8); Linux-only scoping of ADR-T142-1, ADR-T145-1,
ADR-T48-1, ADR-T27-1, ADR-T27-2 and ADR-T77-1 (§6).

- **AC-1** — The workspace type-checks and lints for Windows from Linux:
  `check.sh` runs
  `cargo clippy --target x86_64-pc-windows-msvc --workspace --all-targets -- -D warnings`,
  and says how to install the target when it is missing.
  Test: `check_sh_lints_the_windows_target` (`tests/ci.rs`).
- **AC-2** — `shell::paths` gives the state and files folders: on Linux
  exactly today's rules; on Windows `%LOCALAPPDATA%\draw` and
  `Documents\draw` from Known Folders.
  Tests: `linux_state_dir_follows_xdg_state_home_then_home`,
  `linux_files_dir_is_home`, `windows_dirs_come_from_known_folders`
  (Windows job).
- **AC-3** — No source outside `shell::paths` reads `HOME` or
  `XDG_STATE_HOME`. Test: `home_and_xdg_are_read_only_in_paths`
  (source guard).
- **AC-4** — The `windows` crate is a dependency of `draw-sys` and
  `draw-gfx` for the Windows target only, pinned; Linux builds do not
  compile it. Test: `windows_crate_only_on_the_windows_target`
  (`tests/deps.rs`).
- **AC-5** — `.gitattributes` keeps LF on every text file and marks the
  binary goldens. Test: `gitattributes_keeps_lf_and_binary_goldens`.
- **AC-6** — A `windows` job in `ci.yml` runs fmt, clippy and
  `cargo test --workspace --all-targets` on `windows-latest`;
  `scripts/check.ps1` runs the same gates as `check.sh` except the
  Linux-only ones. Tests: `ci_has_a_windows_job_with_the_gates`,
  `check_ps1_and_check_sh_run_the_same_gates`.
- **AC-7** — On Windows `draw --help`, `--version` and `--from-md` work;
  starting the app itself prints that the Windows window is not ready
  yet and exits with an error (until
  [[T192 Windows platform in main and the e2e suite]]).
  Tests: the existing `main.rs` and `tests/options.rs` tests, run by
  the Windows job; `windows_without_a_window_backend_exits_with_a_message`.

## Out of scope

- Window, GPU, input, clipboard, pen (T188–T191); `main` wiring and e2e
  (T192); packaging (T193).
- `--backend` values for Windows: T192.

## Files owned

- The ADRs this task writes
- `.gitattributes` (new)
- `crates/draw-sys/Cargo.toml`, `crates/draw-gfx/Cargo.toml`, `Cargo.lock`
- `crates/draw-sys/src/lib.rs` (the `cfg` of the module lines),
  `crates/draw-sys/src/win32/mod.rs` and `crates/draw-sys/src/win32/folders.rs` (new)
- `crates/draw-gfx/src/lib.rs`, `crates/draw-gfx/src/gpu.rs` (the `cfg` of the module lines)
- `src/main.rs`, `src/shell/mod.rs`, `src/shell/paths.rs` (new),
  `src/shell/app.rs`, `src/shell/crash.rs`, `src/shell/diag/mod.rs`,
  `src/shell/autosave.rs`, `src/shell/preflight.rs`,
  `src/shell/gpu_native.rs`, `src/shell/backend.rs` (`cfg` and path calls only)
- `tests/*.rs` and `crates/*/tests/*.rs` (`cfg` lines only), `tests/ci.rs`, `tests/deps.rs`
- `scripts/check.sh`, `scripts/check.ps1` (new), `.github/workflows/ci.yml`
- `docs/process/Quality Gates.md` (the Windows gates)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T187): specify the Windows seams and write the ADRs`
- [ ] tests — `test(T187): add failing tests for the Windows seams`
- [ ] models — `feat(T187): add shell::paths and the Windows module`
- [ ] behaviour — `feat(T187): build and test the workspace on Windows`
- [ ] quality — `chore(T187): pass clippy and rustfmt`
- [ ] docs — `docs(T187): update the notes and the quality gates`

## Log

- 2026-11-24 — written from the Windows port proposal, after the owner's
  decisions.
