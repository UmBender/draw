---
title: PNG export
task: "[[T92 Export the view to PNG]]"
adrs: ["[[ADR-T92-1 The png crate as a direct dependency]]", "[[ADR-T92-2 Ctrl+E reads back the document layer]]"]
tutorial: ""
shortcuts: ["Ctrl+E"]
tags: [feature]
---

# PNG export

## What it does

`Ctrl+E` writes what the canvas shows to a new PNG file in your home directory.
The toolbar, the selection and any preview are left out. Nothing is ever
overwritten, and the file's path goes to the diagnostics log.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Export the view to `~/draw-<UTC date>-<time>.png` | `Ctrl+E` |

A second export in the same second gets `-1`, then `-2`. While a text block is
open or a gesture is in progress, `Ctrl+E` does nothing: finish first. Holding
the keys writes one file.

## How it works

`Ctrl+E` is `Command::ExportView`, which only records a request in the
`Editor`. The shell collects it each frame, re-renders the document layer in
full, reads the layer back and hands the pixels to `src/shell/export.rs`, which
crops the margin, puts the rows top-down, encodes the PNG and creates the file.
See [[ADR-T92-1 The png crate as a direct dependency]] and
[[ADR-T92-2 Ctrl+E reads back the document layer]].

## Limits

- Only the view is exported, not the whole document.
- The image has the window's size in physical pixels, so a HiDPI window gives
  a larger image.
- The dot grid is in the file while grid snap is on.
- There is no file dialog, and no SVG.
- Without `HOME`, or on a read-only disk, nothing is written and the reason
  goes to the diagnostics log.
- Under Flatpak the home directory may be sandboxed.
