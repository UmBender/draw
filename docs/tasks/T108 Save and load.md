---
id: T108
title: Save and load
status: in-progress
wave: 60
branch: task/T108-save-and-load
depends_on: [T107, T116]
adrs: ["[[ADR-T108-1 The draw file format and its promise]]"]
feature: "[[Editor snapshots]]"
tutorial: ""
tags: [task]
---

# T108 Save and load

## Goal

`Ctrl+S` saves the drawing to a `.draw` file and `draw file.draw` opens it again, with no file dialog. The file format is versioned and stays readable by later versions.

## Known facts (2026-11-10)

- draw keeps nothing by design (README *Limitations*); the only file is the crash autosave, readable only by the same version ([[Crash recovery]], [[Editor snapshots#Limits]]).
- The snapshot format already serialises the whole editor; it has no compatibility between versions.
- `Ctrl+E` names exports `~/draw-<date>-<time>.png` ([[PNG export]]).
- The proposal rated it high risk: it reverses a stated principle and makes the format a promise through 2.0.0. The owner accepted it.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `Ctrl+S` writes the document to the file named on the command line, or else to `~/draw-<date>-<time>.draw`, which later `Ctrl+S` presses then reuse. The write is atomic (temporary file and rename) and never stalls a frame; the diagnostics log and a notice name the file.
- **AC-2** (owner) — `draw file.draw` opens that file; a missing file starts empty and is created by the first `Ctrl+S`. No file dialog.
- **AC-3** — The file holds the document only (every page's shapes, locks and view, T116), not the undo history or settings; it starts with a format name and version. A file from a newer version, or a damaged one, is refused with a notice and the canvas stays empty; nothing is overwritten.
- **AC-4** — Every later change to the shape model must read older files (a test loads a committed version-1 fixture). This is recorded as an ADR the 2.0.0 refactor (T50) must keep.
- **Tests per AC** — `core::file::tests::*` (AC-3, AC-5), `tests/files.rs::v1_fixture_loads` (AC-4), `tests/fuzz.rs::file_*` (AC-5), `keymap::tests::resolve_ctrl_s_saves` and `editor::tests::save_*` (AC-1), `shell::file::tests::*` (AC-1, AC-2: path, atomic write, arguments).
- **AC-5** — Round trip and fuzzing: any document saved and loaded is equal to the original; random and truncated files never panic.

## Out of scope

- A file dialog, `Ctrl+O`, recent files, `Save as`.
- Saving the undo history; merging files.

## Files owned

- `src/core/snapshot.rs` or a new `src/core/file.rs`, `src/shell/file.rs`, `src/shell/mod.rs`, `src/core/mod.rs`, `tests/files.rs`, `src/core/editor.rs`, `src/core/keymap.rs`, `src/core/command.rs`
- `src/main.rs`, `src/shell/` (arguments, writing off the frame)
- `docs/features/Editor snapshots.md`, `docs/features/Crash recovery.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/` fixtures
- this note, and new `ADR-T108-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T108): specify save and load`
- [ ] tests — `test(T108): add failing tests for save and load`
- [ ] models — `feat(T108): add the versioned document file format`
- [ ] behaviour — `feat(T108): save with Ctrl+S and open a file from the command line`
- [ ] quality — `chore(T108): pass clippy and rustfmt`
- [ ] docs — `docs(T108): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from K3 of the [[Feature proposal]].
- Depends on T107 so the first file version already stores the lock flag.
- 2026-11-10 — also depends on T116 so the first file version stores every page.
- 2026-11-11 — spec: ADR-T108-1. UNATTENDED CHOICE: files owned gain `src/shell/file.rs`, `src/shell/mod.rs`, `src/core/mod.rs` and `tests/files.rs`.
