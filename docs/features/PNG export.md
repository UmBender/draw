---
title: PNG export
task: "[[T92 Export the view to PNG]]"
adrs: ["[[ADR-T92-1 The png crate as a direct dependency]]", "[[ADR-T92-2 Ctrl+E reads back the document layer]]", "[[ADR-T114-1 The ttf-parser crate as a direct dependency]]", "[[ADR-T114-2 SVG export is a pure writer over glyph outlines]]"]
tutorial: ""
shortcuts: ["Ctrl+E", "Ctrl+Shift+E"]
tags: [feature]
---

# PNG export

## What it does

`Ctrl+E` writes what the canvas shows to a new PNG file in your home directory.
The toolbar, the selection and any preview are left out. Nothing is ever
overwritten, and the file's path goes to the diagnostics log.

`Ctrl+Shift+E` writes the whole drawing, not only the view, as an SVG: a
vector image that stays sharp at any size, with the canvas colour behind it.
Text and formulas are glyph outlines, so the file needs none of draw's fonts.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Export the view to `~/draw-<UTC date>-<time>.png` | `Ctrl+E` |
| Export the whole drawing to `~/draw-<UTC date>-<time>.svg` | `Ctrl+Shift+E` |

A second export in the same second gets `-1`, then `-2`. While a text block is
open or a gesture is in progress, both do nothing: finish first. Holding
the keys writes one file.

## How it works

`Ctrl+E` is `Command::ExportView`, which only records a request in the
`Editor`. The shell collects it each frame, re-renders the document layer in
full, reads the layer back and hands the pixels to `src/shell/export.rs`, which
crops the margin, puts the rows top-down, encodes the PNG and creates the file.
See [[ADR-T92-1 The png crate as a direct dependency]] and
[[ADR-T92-2 Ctrl+E reads back the document layer]].

`Ctrl+Shift+E` is `Command::ExportSvg`, another request on the `Editor`. The
shell builds the file's text with `core::svg::to_svg` and writes it on a
thread, to a temporary file first and then to the final name, so the name never
holds a partial file. The writer is pure; the glyph outlines come from the
embedded fonts through `shell::glyphs::FontOutlines` (ttf-parser). Text goes
through the same layout the renderer uses, so formulas match the screen. Each
glyph is one `<path>` in `<defs>`, reused with `<use>`. See
[[ADR-T114-1 The ttf-parser crate as a direct dependency]] and
[[ADR-T114-2 SVG export is a pure writer over glyph outlines]].

Goldens are in `tests/svg/`; rewrite them on purpose with
`cargo test --test svg -- --ignored regenerate`.

## Limits

- The PNG has only the view; the SVG has every shape.
- The PNG has the window's size in physical pixels, so a HiDPI window gives
  a larger image.
- The dot grid is in the PNG while grid snap is on; the SVG has none.
- A stroke drawn with pen pressure is exported as one round-capped segment per
  pair of points, each at the mean width of its ends: the closest SVG form.
- Labels and axis indices are exported at the size they have at zoom 1, whether
  or not they are big enough to show on screen.
- No selectable text in the SVG, no choice of file name, no PDF.
- Without `HOME`, or on a read-only disk, nothing is written and the reason
  goes to the diagnostics log.
- Under Flatpak the home directory may be sandboxed.
