---
id: ADR-T114-2
title: SVG export is a pure writer over glyph outlines
status: accepted
kind: decision
date: 2026-11-11
task: T114
builds_on: ["[[ADR-T92-2 Ctrl+E reads back the document layer]]", "[[ADR-T30-1 TeX subset, box layout and metrics]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T114-2 SVG export is a pure writer over glyph outlines

## Context

The export is a second rendering path that must match the screen
([[T114 Export to SVG]]). The PNG export reads pixels back; SVG cannot.

## Decision

- `core::svg::to_svg(shapes, background, &dyn Outlines)` is pure: it takes
  the shapes bottom to top and a source of glyph outlines, and returns the
  text of the file. The shell provides the outlines (`shell::glyphs`, from
  the embedded fonts through ttf-parser, [[ADR-T114-1 The ttf-parser crate as a direct dependency]]).
- The world is the SVG's user space: the `viewBox` is the bounds of all
  shapes plus a margin; one world unit is one user unit (the screen at
  zoom 1).
- Text layout is not copied: blocks and shape texts go through
  `text_layout::typeset` (plain text and formulas alike) and the
  `Shape::placed_texts` fit, exactly as the renderer does. Each `Ink::Glyph`
  becomes a `<use>` of a `<path>` in `<defs>` (one per face and character,
  in font units), scaled by `size × scale ÷ units-per-em`; each `Ink::Rule`
  becomes a `<line>`.
- Strokes are the stored points: they are already smoothed when committed,
  and the renderer draws them as they are, so there is no smoothing
  function to share and `render.rs` is not touched. A stroke with pressure
  widths becomes one round-capped `<line>` per segment, at the mean width of
  its ends (the closest SVG form of a variable-width polyline).
- Labels and axis indices use the renderer's rules at zoom 1
  (`LABEL_HEIGHT_RATIO`, `LABEL_WIDTH_RATIO`, `LABEL_CHAR_ASPECT`) without
  the on-screen minimum and maximum, so every label is exported.
- Numbers are written with at most 2 decimals and no trailing zeros; a
  non-finite shape is skipped, so every number in the file is finite.
- `Ctrl+Shift+E` sets a request on the editor like `Ctrl+E`; the shell
  builds the text and writes it on a thread, to the first free
  `draw-<date>-<time>.svg` in the home directory, never replacing a file.

## Alternatives considered

- **`<text>` with a font name** — needs the fonts at the viewer.
- **Sharing the renderer's mesh code** — it is in screen pixels and tied
  to macroquad; the pure layout functions are what has to match.

## Consequences

- Outlines are anti-aliased by the viewer, so pixels differ slightly from
  the screen; geometry, colours and layout match.

## Rollback plan

Remove `core::svg`, the command and the shell code in a `revert(T114)`
commit referencing a new ADR.
