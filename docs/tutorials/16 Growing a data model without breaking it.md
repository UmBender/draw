---
title: Growing a data model without breaking it
step: 16
requires: ["[[05 Modelling shapes and hit-testing]]", "[[11 Keymaps and gesture macros]]", "[[13 Property-based testing and fuzzing]]"]
feature:
code: ["src/core/shape.rs", "src/core/editor.rs", "src/core/tools/mod.rs", "src/core/keymap.rs", "src/shell/render.rs", "tests/fuzz.rs"]
tags: [tutorial]
---

# Growing a data model without breaking it

> **Learning path step 16.** Requires: steps 5, 11 and 13 · Next: step 17

## Why this matters here

Three features arrive at once: snapping (T17), a grid tool (T18) and
auto-numbering (T19). They are meant to be built **in parallel**, each by
someone who owns only one or two files. But all three need changes to shared
types: a new shape kind, a new field on two shapes, new commands, new keys,
new toolbar buttons. If each task made those changes itself, the three
branches would conflict in `shape.rs`, `editor.rs` and `keymap.rs` on the
first day.

T16 makes every shared change once, ahead of time, and leaves behind
**stubs** — functions with the final signature that do nothing yet. The
feature tasks then only fill in bodies.

## The concept

**The compiler as a to-do list.** In Rust, adding a variant to an enum
breaks every exhaustive `match` on it, and adding a field to a struct variant
breaks every literal that builds it. That is a feature: `cargo build` lists
every place that has to decide what a grid or a label means. Avoid `_ =>`
arms on your own enums so this list stays complete.

**Skeleton, then flesh.** A stub has the real signature and the most
harmless behaviour:

```rust
pub fn snap_end(start: Vec2, end: Vec2, view: &ToolView<'_>) -> Snapped {
    let _ = (start, view);
    Snapped { point: end, guides: Vec::new() } // identity until T17
}
```

Callers, routing and tests can be written against it today.

**Minimise blast radius.** Each new field costs one edit per struct literal.
T16 needed helper settings in every tool, and `ToolCtx`/`ToolView` literals
live in five test fixtures. Instead of a new field there, the settings ride
inside `DrawStyle`, which the fixtures already build with
`DrawStyle::default()` — zero fixture edits
([[ADR-T16-3 Helper settings and hooks]]). Same idea with
`..Overlay::default()`: a literal written that way survives new fields.

**Fuzz the new invariants immediately.** The type system already guarantees
labels only exist on rectangles and ellipses (the field is only there).
What it cannot express — "grid dims are in `1..=64`", "labels are ≥ 1" — goes
into the fuzzer's invariant check, so T18 and T19 inherit a safety net.

## How draw implements it

- `core::shape` — `Shape::Grid { a, b, cols, rows, style }` and
  `label: Option<u32>` on `Rect`/`Ellipse`
  ([[ADR-T16-1 Grid shape and shape labels]]). `grid_lines` is the one
  iterator both hit-testing (`Shape::hit`) and drawing use, and it clamps the
  dimensions so a bad value can never divide by zero or loop forever.
  `label()`/`with_label()` mirror the existing `fill()`/`with_fill()`.
- `core::editor` — `Helpers` (snap flags, numbering counter, grid size)
  lives in `DrawStyle`. `apply_helper` handles the helper commands and, unlike
  `SetTool`, never cancels the gesture, so arrow keys resize a grid *during*
  the drag. `step_cells` does the clamped arithmetic in `i64`, so even
  `GridCols(i32::MIN)` is safe.
- `core::keymap` — new rows in `BINDINGS`, including the first `Shift`-only
  chords via `KeyChord::shift` ([[ADR-T16-2 Helper key bindings]]).
- `core::{snap, numbering, tools::grid}` — stubs with `//!` docs naming the
  task that fills them.
- `shell::render` — draws grids as axis-aligned bands and labels with
  `draw_text_ex`. `label_size` fits the number in the box (ellipses use the
  inscribed box, `label_area`), hides it under 8 px; `label_raster` snaps the
  font size to 16/32/64/128 and scales, because macroquad caches glyphs per
  font size and zooming would otherwise fill the cache.
  `draw_underlay`/`draw_guides` are empty hooks already called by the app.
- `tests/fuzz.rs` — `arb_grid_drag` selects the grid tool and presses arrow
  keys in the middle of a drag; `check_invariants` checks dims and labels.

## Try it

1. Temporarily add another variant, `Shape::Dot { p: Vec2, style: Style }`,
   and run `cargo build`. Count the errors: that is the full list of
   decisions a new shape needs. Remove it again.
2. In `editor::step_cells`, replace the `i64` arithmetic with
   `(*cells as i32 + delta) as u32` and run
   `cargo test grid_dims_clamped`. Which input breaks it, and how?
3. Change `LABEL_MIN_PX` to `20.0`, run the app, number a few circles
   (after T19) and zoom out: labels vanish earlier.

## Further reading

- [[Parallel Execution]] — why tasks own files.
- Rust reference, *Struct expressions* — functional update syntax
  (`..Default::default()`).
