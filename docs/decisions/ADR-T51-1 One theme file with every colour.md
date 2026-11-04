---
id: ADR-T51-1
title: One theme file with every colour
status: accepted
kind: decision
date: 2026-11-04
task: T51
builds_on: ["[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-0015 Kanagawa Dragon theme]]"]
amends: ["[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-0015 Kanagawa Dragon theme]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, theme]
---

# ADR-T51-1 One theme file with every colour

## Context

[[ADR-0012 Fixed palette and theme tokens]] put the six drawing colours and
six UI tokens in `core::palette`, and [[ADR-0015 Kanagawa Dragon theme]]
gave them their values. Since then the shell grew colours that are not in
`core::palette`: alphas tuned where they are used
(`render::DOT_GRID_ALPHA`, `app::MARQUEE_FILL_ALPHA`,
`toolbar::DISABLED_ALPHA`) and colours derived from tokens or from the
palette (`render::warn_color`, `render::label_color`, the caret on a
filled shape). A token is also shared by places a user may want to
colour apart (`accent` is the selection outline, the guides, the marquee
outline and the active toolbar button).

[[T51 Build-time themes]] wants every colour draw shows in **one** source
file that someone who does not know the code (a desktop customiser) can
edit and rebuild. No argument, environment variable, Cargo feature or
config file chooses a theme; one theme ships, with the Kanagawa Dragon
values unchanged, and the shipped picture must stay identical to the
pixel.

## Decision

### The file

`src/theme.rs`, a module at the crate root (`draw::theme`), so it is the
first thing a customiser sees in `src/`. It holds nothing but `pub const`
items, grouped by where they show, each with a one-line comment. No
types, no functions, no macroquad, no `use` outside its tests.

- A colour is a `u32` written `0xRRGGBB`, so `#7fa8bc` from any palette
  becomes `0x7fa8bc`.
- An opacity is a **separate `u8`**: `0` invisible, `255` opaque. Only
  the three translucent colours have one. `0xRRGGBBAA` was rejected: most
  colours are opaque and an eight-digit value that is mostly `ff` is
  easier to get wrong.

### Groups and constants

| Group | Constant | Value | Was |
|-------|----------|-------|-----|
| Canvas | `CANVAS` | `0x181616` | `THEME.bg` |
| Canvas | `TEXT_ON_FILL` | `0x181616` | `THEME.bg` in `label_color` and the caret on a fill |
| Drawing colours (keys 1–6) | `INK`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `MAGENTA` | ADR-0015 values | `PALETTE` |
| Selection and marquee | `SELECTION_OUTLINE` | `0x7fa8bc` | `THEME.accent` in `draw_selection` |
| Selection and marquee | `MARQUEE_OUTLINE` | `0x7fa8bc` | `THEME.accent` |
| Selection and marquee | `MARQUEE_FILL`, `MARQUEE_FILL_ALPHA` | `0x2d4f67`, `63` | `THEME.selection` × `0.25` |
| Snapping | `GUIDE` | `0x7fa8bc` | `THEME.accent` in `draw_guides` |
| Snapping | `DOT_GRID`, `DOT_GRID_ALPHA` | `0xa6a69c`, `89` | `THEME.border` × `0.35` |
| Formulas | `FORMULA_SOURCE`, `FORMULA_SOURCE_ALT` | `0xd16961`, `0xceb680` | `PALETTE[1]`, `PALETTE[4]` in `warn_color` |
| Toolbar | `TOOLBAR_BACKGROUND` | `0x0d0c0c` | `THEME.surface` |
| Toolbar | `TOOLBAR_BORDER` | `0xa6a69c` | `THEME.border` |
| Toolbar | `TOOLBAR_TEXT` | `0xc5c9c5` | `THEME.text` |
| Toolbar | `TOOLBAR_ACTIVE` | `0x7fa8bc` | `THEME.accent` |
| Toolbar | `TOOLBAR_ACTIVE_FILL` | `0x2d4f67` | `THEME.selection` in `draw_toggle` |
| Toolbar | `TOOLBAR_DISABLED_ALPHA` | `76` | `DISABLED_ALPHA = 0.3` |

### Derived colours, case by case

- **Marquee fill, dot grid**: become theme constants (colour + alpha).
  They no longer follow `selection` / `border`; a customiser sets them
  where they show.
- **Text, numbers and caret on a filled shape** (`label_color`, caret):
  the rule stays (outline colour on an empty shape, a fixed colour on a
  filled one); its fixed colour is `TEXT_ON_FILL`.
- **Caret elsewhere**: keeps the text block's drawing colour; no constant.
- **Literal formula source** (`warn_color`): the rule stays (one colour,
  another when the text is already drawn in it); its inputs are
  `FORMULA_SOURCE` and `FORMULA_SOURCE_ALT` instead of palette slots 2 and
  5.
- **Disabled undo/redo**: the rule stays (the button's colour, faded); the
  fade is `TOOLBAR_DISABLED_ALPHA`, applied as `a × alpha / 255` in
  integer arithmetic by `Rgba::faded`.

### Identical pixels

macroquad turns a vertex colour into bytes by truncation
(`(c × 255) as u8`), and `k / 255` round-trips for every byte `k`. The old
alphas give bytes `trunc(0.25 × 255) = 63`, `trunc(0.35 × 255) = 89` and
`trunc(0.3 × 255) = 76`, so the `u8` alphas above draw the same bytes. A
test pins this against macroquad's `Vertex::new`.

### What stays where

- `core::palette` keeps `Rgba`, `ColorId`, `PALETTE_LEN`, `PALETTE` (now
  built from the six `theme` drawing constants) and `palette()`. It gains
  `Rgba::from_hex_alpha`, `Rgba::faded`, `Rgba::NO_TINT`,
  `Rgba::relative_luminance` and `contrast_ratio`. The `Theme` struct and
  `THEME` const are removed: the shell reads `crate::theme` directly.
- `ColorId` keeps six colours in key order (ADR-0012).
- Not colours, so not in the theme: the glyph atlas texels (white with
  coverage as alpha, a mask the vertex colour multiplies) and the tint the
  cached canvas layer is drawn with, `Rgba::NO_TINT` (every channel at
  `u8::MAX`, i.e. "leave the texture as it is").

### Guard and sanity tests

- A test in `theme::tests` scans every `.rs` file under `src/` except
  `src/theme.rs`, skipping `//` comments and `#[cfg(test)]` items, and
  fails with `file:line` on any of: a six- or eight-digit hex literal
  (no `_`), a `Color::new` / `Color::from_rgba` / `from_hex` /
  `from_hex_alpha` / `color_u8!` call whose first argument is a number, a
  `Color { … }` or `Rgba { … }` literal, an alpha edit (`.a =`, `.a *=`,
  `.a +=`, `.a -=`, `.with_alpha(`) and a macroquad named colour
  (`macroquad::color::WHITE`, `colors::…`). Only `src/theme.rs` is
  allow-listed.
- Sanity tests a customiser may rely on, separate from the test pinning
  the Kanagawa values: the six drawing colours are distinct, and `INK` and
  `SELECTION_OUTLINE` have a WCAG 2 contrast ratio of at least **3:1**
  against `CANVAS` (the WCAG level for graphics and large text). The
  marquee fill is a translucent tint over the canvas and is not checked.

## Alternatives considered

- **Keep tokens shared by several places** (`accent` for selection,
  guides, marquee and toolbar): fewer constants, but a customiser cannot
  colour the guides apart from the selection without reading code.
- **`src/core/theme.rs`**: tidy, but one level deeper than the file a
  customiser looks for first.
- **`Rgba` consts in the theme file** (`Rgba::from_hex(0x…)`): typed, but
  the file then needs a `use` and a constructor call per line; plain
  integers read like a palette.
- **`f32` opacities** (`0.25`): match the old code, but no palette tool
  writes them, and they still become a byte on the GPU.
- **Runtime themes** (file, env var, flag): out of scope by the owner's
  decision; see [[ADR-0015 Kanagawa Dragon theme]] for why colours are
  compiled in.

## Consequences

- Changing any colour is one edit in `src/theme.rs` and a rebuild; the
  guard test stops new code from hard-coding one again.
- `core::palette` depends on `crate::theme` (pure consts), not the other
  way round; the core stays free of macroquad.
- A customiser who breaks contrast gets a failing sanity test that says
  which pair is too close, and may change the threshold knowingly.
- `docs/architecture/Theme.md` points at `src/theme.rs`.

## Rollback plan

A new ADR reverting this one; `git revert` the T51 commits. `THEME` and
the shell's local alphas come back from history.
