---
id: ADR-T29-1
title: Text shape with monospaced layout and an embedded font
status: accepted
kind: decision
date: 2026-10-14
task: T29
builds_on: [ADR-0003, ADR-0013, ADR-0014, ADR-T07-1]
amends: [ADR-0004, ADR-T16-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T29-1 Text shape with monospaced layout and an embedded font

## Context

[[T29 Text blocks]] adds free-standing text to the canvas. Text blocks must
be ordinary shapes — bounds, hit-testing, culling, snapping, fit-to-content
— and all of that lives in `core`, which must stay free of macroquad
([[ADR-0003 Headless core and thin shell]]). Those queries need text
metrics. macroquad's built-in font is ASCII-only, while contest notes need
Greek letters and math symbols (and [[T30 Basic TeX formulas]] needs more).
[[ADR-0014 Minimal dependencies and no unsafe]] rules out a new crate, and
the release binary must stay under 2 MB (about 1 MB today).

## Decision

- **Model.** New variant `Shape::Text { origin, text, size, style }`
  ([[ADR-0004 Vector object model]] amended again, like
  [[ADR-T16-1 Grid shape and shape labels]]): `origin` is the top-left
  corner of the block in world units, `text` its UTF-8 content (lines split
  on `\n`), `size` the font size (em height) in world units, `style.color`
  its colour. `style.width` is unused and set to 0. A text block is an
  **open** shape: no interior, no fill; it is hit anywhere inside its box
  (plus the tolerance), like a solid object.
- **Size.** A new block's size is `TEXT_BASE_PX + TEXT_PX_PER_WIDTH ×
  width_px` screen pixels (12 + 2 × the `[` / `]` width: 14 px to 76 px,
  18 px at the default width) divided by the zoom at creation, so text
  zooms with the canvas like widths do
  ([[ADR-0013 World-space widths and zoom limits]]).
- **Layout in `core`** (`core::text_layout`). Every `char` is one cell of a
  monospaced grid: `ADVANCE` = 0.6 em wide, `LINE_HEIGHT` = 1.25 em tall,
  baseline `BASELINE` = 1.0 em below the top of its line. Bounds, caret
  positions, point-to-caret mapping and glyph positions are pure functions
  of `(origin, text, size)`; the renderer draws each glyph at its cell, so
  what is tested is what is drawn. Characters without a glyph show the
  font's replacement box; wide (CJK) characters overlap their neighbour —
  out of scope.
- **Font.** `assets/fonts/AdwaitaMono-Regular-subset.ttf`, a subset of
  **Adwaita Mono Regular 34.1** (Iosevka-based, SIL OFL 1.1, no reserved
  font names), embedded with `include_bytes!` and loaded with macroquad's
  own TTF loader — no new dependency. Every glyph is exactly 600/1000 em
  wide; hhea ascent 965, descent 215, line gap 70 give the 1.25 em line
  and the 1.0 em baseline above. The subset keeps U+0020–007E,
  U+00A0–017F (Latin-1, Latin Extended-A), U+0370–03FF (Greek),
  U+2000–206F, U+2070–209F, U+2100–214F, U+2190–21FF (arrows),
  U+2200–22FF (math operators), U+2300–23FF (floor/ceil), U+25A0–25FF,
  U+27E6–27FF and U+FFFD: 1 342 glyphs in **124 KB**, built with
  `pyftsubset --layout-features='' --no-hinting --desubroutinize`. The
  licence is `assets/fonts/OFL.txt`.
- **Labels** of rectangles, ellipses and grid axes move to the same font
  for one consistent look; their sizing rules (ADR-T16-1) are unchanged —
  `LABEL_CHAR_ASPECT` 0.6 already matched a monospaced digit.
- **Rendering.** Text is drawn in screen space
  ([[ADR-T07-1 Screen-space tessellation in the renderer]]): on-screen size
  `size × zoom`, hidden below `TEXT_MIN_PX` (4 px, where it is unreadable
  anyway) and rasterized at the smallest of `TEXT_RASTER_SIZES`
  (12 … 256 px) at least that size, then scaled down — sharp from 0.05× to
  20× and at most twelve glyph-cache entries per character.

## Alternatives considered

- **Proportional font with metrics in `core`** — needs a per-glyph width
  table in `core` copied from the font, or the font parser in `core`;
  more code, harder caret tests, no gain for contest notes.
- **Measure in the shell, cache sizes in the shape** — bounds would depend
  on the renderer; the fuzzer and every core query would need a stub.
- **DejaVu Sans Mono** — wide coverage but Bitstream Vera licence, not OFL.
- **Noto Sans Mono** — OFL, 596 KB, lacks `∑` and `∫`.
- **Full Adwaita Mono** — 1.4 MB, the binary would pass 2 MB.

## Consequences

- Every exhaustive `match` on `Shape` grows an arm.
- The binary grows by about 124 KB.
- Any monospaced font with the same 0.6 em advance can replace this one by
  changing only the asset and the three metrics constants.
- [[T30 Basic TeX formulas]] reuses the cell metrics and the subset's math
  ranges.

## Rollback plan

A rollback ADR plus `revert(T29)` commits remove the variant, the layout
module, the asset and the label font switch; ADR-0004 and ADR-T16-1 stay
as they were.
