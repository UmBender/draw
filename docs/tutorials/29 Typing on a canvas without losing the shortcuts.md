---
title: Typing on a canvas without losing the shortcuts
step: 29
requires: ["[[05 Modelling shapes and hit-testing]]", "[[16 Growing a data model without breaking it]]", "[[08 An editor as an input-driven state machine]]", "[[11 Keymaps and gesture macros]]", "[[24 Layered redraws and measuring a frame]]"]
feature: "[[Text blocks]]"
code: ["src/core/text_layout.rs", "src/core/tools/text.rs", "src/core/editor.rs", "src/core/input.rs", "src/shell/input_map.rs", "src/shell/render.rs"]
tags: [tutorial]
---

# Typing on a canvas without losing the shortcuts

> **Learning path step 29.** Requires: steps 5/16 (shape model), 8 (editor
> and input), 11 (keymap) and 24 (layered redraw) · Next: —

## Why this matters here

draw is keyboard-first: `P`, `L`, `1`–`6`, `[`, `]`, `Tab`, `Space`
all *do* something. A text tool turns the same keys into *characters*. Get
the switch wrong and typing "pq" picks the pen and does nothing, or
`Ctrl+Backspace` in a note clears the canvas. And the core that must know
where every character sits — to hit-test, snap and place the caret — is not
allowed to talk to the font renderer.

## The concept

**Two streams from one key press.** A keyboard sends a *key* (`KeyCode::P`,
"the key labelled P") and the system's layout turns it into a *character*
(`p`, `P`, `π` with a compose key, nothing for `F5`). Shortcuts want keys;
text wants characters. miniquad delivers both: `char_event` then
`key_down_event`. So the shell forwards both, and the editor picks one per
mode:

```
              ┌── not editing ──▶ KeyDown → keymap → Command
key press ──▶ │
              └── editing ──────▶ Char → insert;  KeyDown → editing key or nothing
```

A **modal state machine** is the honest model: edit mode is a state with
its own transition table (`Esc` commits, `Ctrl+Z` cancels, a click outside
commits), not a filter bolted onto the keymap.

**Monospaced layout is arithmetic.** If every character is one cell
`0.6 em × 1.25 em`, then the box of a block is
`(longest line × 0.6, lines × 1.25) × size`, the caret at `(line, col)` is
`origin + (col × 0.6, line × 1.25) × size`, and a click maps back by
dividing and rounding. No font file is needed to answer layout questions —
only three numbers measured from it once.

**UTF-8 carets.** Rust strings are bytes; `"β"` is two of them. A caret is
a byte index that must sit on a `char` boundary, or `insert`/`remove`
panic. Moving left means "back by the length of the previous `char`", not
"minus one".

**One undo step per edit.** Typing does not touch the document: the open
block is a separate value, shown in the overlay. Only finishing it writes a
single `Insert`, `Replace` or `Remove` transaction.

## How draw implements it

- `core::input::InputEvent::Char { ch, mods }` is new; `shell::input_map`
  fills it from `char_event` and maps `Enter`, `Home`, `End`.
- `core::text_layout` holds `ADVANCE`, `LINE_HEIGHT`, `BASELINE` and pure
  functions: `bounds`, `line_col`, `caret_index`, `caret_point`,
  `caret_from_point`, `glyphs`. Proptests check that every caret it returns
  is a `char` boundary and every glyph cell is inside `bounds`.
- `core::tools::text::TextEdit` is the open block: `insert`, `backspace`,
  `delete`, `left`/`right` (step by `len_utf8`), `up`/`down` (same column,
  clamped), `home`/`end`, `tab`. `Shape::Text` comes out of `shape()`.
- `Editor::handle` checks `tools.text.edit` before the keymap:
  `text_key` handles editing keys and swallows the rest; `Char` inserts
  unless it is a control character or `Ctrl` is held. `finish_text`
  commits through `track_document`, so the document layer is redrawn once
  (step 24). `Editor::overlay` shows the open block and hides the original
  while re-editing.
- Re-editing with Select needs no clock: a click on the block that is
  *already* the only selection opens it. A double-click is exactly that.
- `shell::render::draw_text_block` draws each `Glyph` at its cell with the
  embedded font, rasterized at the nearest `TEXT_RASTER_SIZES` entry above
  the on-screen size; the caret is a steady line in the overlay, so the
  blocking loop sleeps between keys.

## Try it

1. Add a binding test: in `core::editor` tests, check that `Ctrl+A` while
   editing leaves the selection empty (it does — why? Follow `handle`).
2. Make `Ctrl+Backspace` delete the previous *word* in edit mode: add a
   `TextEdit::delete_word_back`, a unit test with `"foo βar"`, and route
   it in `text_key`.
3. Change `ADVANCE` to `0.5` and run `cargo test`: which tests catch that
   the layout no longer matches the font?

## Further reading

- [[ADR-T29-1 Text shape with monospaced layout and an embedded font]],
  [[ADR-T29-2 Text edit mode]].
- *The Rust Book*, "Storing UTF-8 Encoded Text with Strings".
- The `char_event` / `key_down_event` split in miniquad's
  `EventHandler` trait.
