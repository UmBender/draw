---
id: T115
title: Copy the selection as an image
status: todo
wave: 59
branch: task/T115-copy-as-image
depends_on: []
adrs: []
feature: "[[Editing tools]]"
tutorial: ""
tags: [task]
---

# T115 Copy the selection as an image

## Goal

`Ctrl+Shift+C` puts the selected shapes on the system clipboard as a PNG image, so a diagram can be pasted straight into a chat or document, on Wayland and X11.

## Known facts (2026-11-10)

- `Ctrl+C` copies inside draw only; `Ctrl+Shift+C` is unbound. The system clipboard is used for text only (`Ctrl+V` in a text block, `Ctrl+Shift+V` for a grid), through miniquad's text clipboard.
- miniquad's clipboard carries text only: offering `image/png` needs a binary MIME path in the vendored miniquad, on Wayland (`wl_data_source`) and X11 (selection with a `image/png` target) ([[ADR-T35-1 Vendored miniquad with a pen tablet patch]]).
- `Ctrl+E` already encodes PNG with the `png` crate (ADR-T92-1); the document layer can be rendered off-screen.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `Ctrl+Shift+C` renders the selected shapes alone, cropped to their box plus a small margin, on the canvas background, at the current zoom (at least 1:1), and offers it on the clipboard as `image/png`. A notice confirms it; with nothing selected it does nothing.
- **AC-2** — Pasting into another app (a browser chat, an image editor) gives the image, on native Wayland and on X11; draw keeps serving the clipboard until another app takes it.
- **AC-3** — The image is limited to a maximum size (set in the ADR) so a huge selection cannot exhaust memory; larger ones are scaled down and the log says so.
- **AC-4** — The core part (crop box, size limit, pixel layout) is unit-tested and fuzzed; the backend patch is recorded in `vendor/miniquad/PATCHED.md` and checked by hand on both backends.

## Out of scope

- Copying as SVG or as text; pasting images into draw.

## Files owned

- `src/core/editor.rs`, `src/core/keymap.rs`, `src/core/command.rs`, a new `src/core/snapshot_image.rs` (crop and limits)
- `src/shell/export.rs`, `src/shell/render.rs`, `vendor/miniquad/` (binary clipboard), `vendor/miniquad/PATCHED.md`
- `docs/features/Editing tools.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T115-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T115): specify copying the selection as an image`
- [ ] tests — `test(T115): add failing tests for copying the selection as an image`
- [ ] models — `feat(T115): crop and size the selection image`
- [ ] behaviour — `feat(T115): offer the selection as a PNG on the clipboard`
- [ ] quality — `chore(T115): pass clippy and rustfmt`
- [ ] docs — `docs(T115): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from K6 of the [[Feature proposal]].
- Risk: the largest part is the vendored miniquad patch; check on Hyprland and X11 by hand.
