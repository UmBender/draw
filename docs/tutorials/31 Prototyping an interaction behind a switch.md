---
title: Prototyping an interaction behind a switch
step: 31
requires: ["[[29 Typing on a canvas without losing the shortcuts]]", "[[18 A grid tool with live parameters]]", "[[19 Auto-numbering and undoable counters]]"]
feature: "[[Text in shapes (experimental)]]"
code: ["src/core/experimental.rs", "src/core/shape.rs", "src/core/text_layout.rs", "src/core/tools/text.rs", "src/core/editor.rs", "src/core/tools/select.rs", "src/core/tools/eraser.rs", "src/shell/render.rs", "src/shell/app.rs", "tests/fuzz.rs"]
tags: [tutorial]
---

# Prototyping an interaction behind a switch

> **Learning path step 31.** Requires: step 29 (text edit mode), step 18
> (grid tool), step 19 (numbering).

## Why this matters here

Typing into a DP table cell by cell sounds obviously useful, but in a
keyboard-first app every new mode can steal keys and clicks from the
old ones. The only way to know is to try it on real problems. A
**feature switch** lets the code land on `main`, tested and fuzzed,
while nobody meets it by accident, and keeps removal cheap: one
rollback ADR and a `git revert`.

## The concept

A switch has three parts:

1. **A pure parser** of the environment. `Experimental::from_env_value`
   takes `Option<&str>` rather than reading `std::env` itself, so it is
   unit-tested like any other function, and the shell reads the
   variable exactly once.
2. **One place the flag lives.** The editor already carries helper
   settings to every tool (`Helpers`, ADR-T16-3); the flag rides along,
   so no signature changes.
3. **Unreachable when off.** With the flag off, no input path creates
   the new data. The fuzzer checks this as an invariant, not just as
   one example.

Then reuse instead of invent: the prototype is T29's text edit session
with a **host**. The session does not care whether it ends in a
`Shape::Text` or in a rectangle's `text` field; only `finish_text`
does.

```text
Text tool press ──► topmost text block?  ──► TextEdit::open        (T29)
                └─► topmost shape with
                    text slot at press?  ──► TextEdit::open_in     (T31, flag on)
                └─► otherwise            ──► TextEdit::new
```

## How draw implements it

- `core::experimental` — the switch.
- `core::shape` — `TextSlot` names where text lives (interior or cell);
  `slot_text` / `with_slot_text` read and write it, mirroring the cell
  fills of step 21; `text_frame` is the box text fits into and
  `grid_cell_step` walks cells in **screen** order even when the grid was
  dragged from its bottom-right corner.
- `core::text_layout::fit` — text layout scales linearly with the font
  size, so the fitted size is one division per axis:
  `min(max_size, frame_w / w₁, frame_h / h₁)` where `w₁ × h₁` is the box
  at size 1. Centring uses the box's own centre, so formulas whose ink
  starts away from the origin still sit in the middle.
- `core::tools::text` — `TextEdit` keeps its stored `size` as the
  *largest* size and computes the shown origin and size (`placed`) on
  every query, so the caret, outline and overlay always agree.
- `core::editor` — `finish_text` turns a hosted edit into one
  `tx_replace`; `step_cell` commits and re-opens the next cell, which is
  why filling a 5 × 5 table leaves exactly 25 undo steps.
- `core::tools::select` — a press inside the only selected shape counts
  as a hit, so the second click can reach its interior.
- `tests/fuzz.rs` — `arb_host_session` draws a shape and types into it;
  every property runs with the switch on and off.

## Try it

1. Run `DRAW_EXPERIMENTAL=shape-text cargo run --release`, draw a 5 × 5
   grid (`G`), press `T`, click the top-left cell and fill it with `Tab`.
   Time yourself, then do the same with plain text blocks.
2. Change `SHAPE_TEXT_FILL` in `src/core/shape.rs` to `1.0` and run
   `cargo test text_frame` — read which assertion fails and why the
   margin exists.
3. Make `Enter` at the bottom row wrap to the top of the next column:
   change the `CellStep::Down` arm of `Shape::grid_cell_step`, then fix
   `shape_text_enter_moves_down_and_stays_at_bottom_row`.

## Further reading

- [[ADR-T31-1 Shape text prototype behind a switch]]
- Martin Fowler, *Feature Toggles (aka Feature Flags)* — release toggles
  vs. experiment toggles.
