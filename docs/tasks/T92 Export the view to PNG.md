---
id: T92
title: Export the view to PNG
status: todo
wave: 51
branch: task/T92-png-export
depends_on: []
adrs: [ADR-T92-1]
feature: "[[PNG export]]"
tutorial: ""
tags: [task]
---

# T92 Export the view to PNG

## Goal

`Ctrl+E` writes the current view to a PNG file. (idea K4 of [[Feature proposal]])

## Known facts (2026-11-09)

- macroquad's `Image::export_png` panics when the write fails, and `panic = "abort"` would lose the drawing: do not use it.
- Owner's decision (2026-11-09): use the `png` crate directly. It is already in `Cargo.lock` through macroquad, so `ADR-T92-1` amends [[ADR-0014 Minimal dependencies and no unsafe]].
- Read-back gives rows bottom-up, as `export_png` shows.

## Spec

The spec step names the test for each criterion.

- **AC-1** — `Ctrl+E` writes the drawing as seen in the view, without the toolbar, selection or previews, to `~/draw-<UTC date>-<time>.png`.
- **AC-2** — An existing file is not overwritten: a `-1`, `-2` suffix is added.
- **AC-3** — A failed write (no `HOME`, read-only disk) never panics and goes to the diagnostics log; the path goes to the log on success too.
- **AC-4** — Rows are in the right order. *Test:* a pure function that encodes RGBA to PNG bytes, decoded back in a test.
- **AC-5** — e2e case: `Ctrl+E` produces a decodable PNG of the window's size.
- **AC-6** — `Ctrl+E` is ignored during text edit and a gesture.

## Out of scope

- Exporting the whole document beyond the view.
- SVG.
- A file dialog.

## Files owned

- `Cargo.toml`, `Cargo.lock`
- `src/shell/app.rs`, `src/shell/render.rs`, `src/shell/input_map.rs`
- new `src/shell/export.rs`, `src/shell/mod.rs`
- `tests/e2e/**`
- `docs/features/PNG export.md` (new)
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T92-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T92): specify export the view to png`
- [ ] tests — `test(T92): add failing tests`
- [ ] models — `feat(T92): add types`
- [ ] behaviour — `feat(T92): implement`
- [ ] quality — `chore(T92): pass clippy and rustfmt`
- [ ] docs — `docs(T92): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
