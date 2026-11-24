---
id: T192
title: Windows platform in main and the e2e suite
status: todo
wave: 81
branch: task/T192-windows-platform-in-main-and-the-e2e-suite
depends_on: [T189, T191]
adrs: []
feature: "[[Windows]]"
tutorial: ""
tags: [task, windows]
---

# T192 Windows platform in main and the e2e suite

## Goal

`draw.exe` runs the app on Windows: `main` runs
`app::run::<Win32Platform, D3d11Gpu>`, the process behaves like a
Windows program (no console window, clean exit on log-off, crash dump),
and the end-to-end suite passes on the Windows CI job and joins the
release gate.

## Known facts (2026-11-24)

- Decisions: [[Windows port proposal]] §3.3, §3.6, §3.9 and §4.
- `main` keeps the `match` over backends with no `dyn`
  ([[ADR-T131-1 No dyn outside an allow-list]]); on Windows there is one
  arm.
- `#![cfg_attr(windows, windows_subsystem = "windows")]` hides the
  console; `--help`, `--version` and `--from-md` attach to the parent
  console with `AttachConsole(ATTACH_PARENT_PROCESS)`.
- The first call is `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32)`.
- Native faults go to `SetUnhandledExceptionFilter`: a diagnostics line
  and a minidump in `%LOCALAPPDATA%\draw\crash.dmp`, with `dbghelp`
  loaded from System32 only. The panic hook stays
  ([[ADR-T27-3 Crash report from a panic hook and split debug symbols]]).
- The application manifest (Per-Monitor V2, `supportedOS`, UTF-8 code
  page, long paths) is embedded through MSVC linker arguments from a new
  `build.rs`, which does nothing when the target is not Windows, so the
  Linux clippy of T187 still runs.
- Preflight on Windows checks the version (10 1809 or later) and that a
  D3D11 device or WARP can be created. A hidden debugging flag
  `--gpu warp` forces WARP (no environment variables:
  `tests/no_draw_env.rs`).
- The e2e suite of [[T75 End-to-end test suite]] drives the app in a
  headless sway and takes screenshots with `grim`. On Windows the
  harness drives `draw.exe` with `SendInput` and reads snapshots and
  pixels through the app's own e2e hooks (`--e2e-dump`, `--e2e-idle`
  and a new frame read-back hook). Same case folders, per-backend pixel
  goldens where D3D11 and GL differ.
- The release gate of [[T129 Release gate with end-to-end on X11 and Wayland]]
  adds the Windows job's result for the tagged commit.
- Windows tests run only on the Windows CI job (proposal, question 7).

## Spec

Draft; the spec step names the final tests.

- **AC-1** — `main` runs the Windows arm; `--backend` on Windows warns
  and is ignored. Tests: `main_runs_the_windows_arm` (source test),
  `backend_flag_is_ignored_on_windows`.
- **AC-2** — Console: only `--help`, `--version` and `--from-md` attach
  to the parent console. Tests: `console_attach_only_for_cli_modes`
  (pure), `draw_exe_prints_the_version_in_a_console` (Windows).
- **AC-3** — `draw.exe` opens, draws and closes with exit code 0;
  log-off (`WM_ENDSESSION`) flushes the autosave.
  Tests (Windows): `draw_exe_opens_and_closes_cleanly`,
  `end_session_flushes_the_autosave`.
- **AC-4** — A native fault writes the diagnostics line and the
  minidump. Test (Windows): `native_fault_writes_a_minidump`.
- **AC-5** — The manifest is embedded: Per-Monitor V2 and the UTF-8
  code page are active in `draw.exe`.
  Test (Windows): `draw_exe_has_the_manifest`.
- **AC-6** — Preflight refuses Windows older than 10 1809 and falls back
  to WARP when no hardware device works. Tests:
  `windows_preflight_needs_1809` (pure), `windows_preflight_falls_back_to_warp`.
- **AC-7** — The e2e cases pass on Windows. Test: `e2e_windows_every_case`
  (ignored outside its script, like `e2e_every_case`).
- **AC-8** — `scripts/release.sh` refuses a tag whose Windows job did
  not pass. Test: `release_requires_the_windows_job` (`tests/ci.rs`).

Manual check (the owner's friend): the CI build of the branch starts
from Explorer and from a terminal; drawing, shortcuts, save and reopen,
PNG and SVG export into `Documents\draw`.

## Out of scope

- Installer, signing, icon, hardening flags
  ([[T193 Windows installer, signing and packages]]).

## Files owned

- `src/main.rs`, `build.rs` (new), `packaging/windows/draw.manifest` (new)
- `src/shell/preflight.rs`, `src/shell/crash.rs`, `src/shell/options.rs`,
  `src/shell/backend.rs`, `src/shell/gpu_native.rs`,
  `src/shell/diag/e2e.rs`, `src/shell/app.rs` (end of session, frame hook)
- `crates/draw-sys/src/win32/crash.rs`, `crates/draw-sys/src/win32/console.rs` (new),
  `crates/draw-sys/src/win32/platform.rs`
- `tests/e2e_windows.rs` (new), `tests/common/**`, `tests/e2e/**`
  (per-backend goldens), `tests/ci.rs`, `tests/options.rs`
- `scripts/e2e.ps1` (new), `scripts/release.sh`,
  `.github/workflows/ci.yml`, `.github/workflows/nightly.yml`
- `docs/features/Windows.md` (new feature note),
  `docs/features/End-to-end tests.md` (the Windows harness)
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T192): specify the Windows app and e2e`
- [ ] tests — `test(T192): add failing tests for the Windows app`
- [ ] models — `feat(T192): add the Windows arm, console and crash types`
- [ ] behaviour — `feat(T192): run draw on Windows with its e2e suite`
- [ ] quality — `chore(T192): pass clippy and rustfmt`
- [ ] docs — `docs(T192): add the Windows feature note`

## Log

- 2026-11-24 — written from the Windows port proposal.
