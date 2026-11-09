---
id: ADR-T92-2
title: Ctrl+E reads back the document layer
status: accepted
kind: decision
date: 2026-11-09
task: T92
builds_on: ["[[ADR-0003 Headless core and thin shell]]", "[[ADR-T11-1 Exact modifier matching]]", "[[ADR-T15-1 MSAA on the cached frame]]", "[[ADR-T24-1 Document layer keyed by a document revision]]", "[[ADR-T24-2 Overlay drawn straight to the window]]", "[[ADR-T26-1 Reuse the document layer during pan and zoom]]", "[[ADR-T29-2 Text edit mode]]", "[[ADR-T32-1 Own glyph atlas rasterized with fontdue]]", "[[ADR-T92-1 The png crate as a direct dependency]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T92-2 Ctrl+E reads back the document layer

## Context

The export must show the drawing as seen in the view, without the toolbar,
selection or previews, and must not disturb an open text block or a gesture.
Only the shell can read pixels, but whether an export is allowed depends on
editor state.

## Decision

- **Key.** `Ctrl+E` is `Command::ExportView` in `core::keymap`. `E` alone stays
  the eraser ([[ADR-T11-1 Exact modifier matching]]).
- **Request.** The command only records a request, which the shell collects
  with `Editor::take_export_request`. It changes nothing else: it is ignored
  during a text edit or a gesture, and never commits or cancels either.
- **Key repeat.** `Collector` drops auto-repeats of the chord, so holding the
  keys writes one file.
- **Source.** The shell reads the resolved texture of the document layer, which
  holds the background, the underlay and the shapes, and nothing of the
  overlay ([[ADR-T24-2 Overlay drawn straight to the window]]).
- **Fresh layer.** The request forces a full re-render, so a layer still being
  blitted from an older camera
  ([[ADR-T26-1 Reuse the document layer during pan and zoom]]) is never
  exported. If the glyph atlas overflowed in that render, the export waits for
  the next frame's complete layer
  ([[ADR-T32-1 Own glyph atlas rasterized with fontdue]]).
- **Crop.** The margin around the window is cut off, the rows are put top-down
  and every pixel is made opaque, in the pure `shell::export::view_pixels`.
- **File.** `~/draw-<UTC date>-<time>.png`, opened with `create_new`, so an
  existing file is never overwritten and the next `-1`, `-2` name is tried. A
  write that fails half-way removes its partial file.
- **Thread.** Read-back, encoding and writing run on the main thread: this
  happens once per key press, and the GL read has to be there anyway.

## Alternatives considered

- **A shell-only chord, like the dump and snapshot chords** — those are
  debugging aids that work in every mode. This one is a user command with a
  mode rule, which belongs in the keymap and can be tested and fuzzed in core.
- **Reading the window's framebuffer** — it includes the overlay.
- **Rendering again into a window-sized target** — a second target and pass
  for what the layer already holds.

## Consequences

- The underlay dot grid is exported while grid snap is on, because it is part
  of what the view shows.
- The export has the window's size in physical pixels, so a HiDPI window gives
  a larger file.
- A very large window can hold one frame for the time it takes to encode.

## Rollback plan

Remove the binding, `Command::ExportView` and `shell::export`, in a
`revert(T92)` commit referencing a new ADR.
