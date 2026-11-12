---
id: T117
title: Looser shape recognition
status: done
wave: 60
branch: task/T117-looser-recognition
depends_on: []
adrs: []
feature: "[[Drawing tools]]"
tutorial: ""
tags: [task]
---

# T117 Looser shape recognition

## Goal

`Z` after a quick, careless stroke still gives a line, rectangle or ellipse. Today the thresholds of [[ADR-T109-1 Shape recognition on Z]] are tight, so a hurried stroke often stays as drawn; since the rectangle and ellipse tools already exist for careful shapes, `Z` is only worth using when it forgives a sloppy one.

## Known facts (2026-11-11)

- Reported by the owner after trying 1.5.0: recognition is too sensitive. They reach for `Z` to be fast, and when it does nothing the stroke has to be redrawn.
- Thresholds are `pub const`s in `src/core/recognize.rs`: line straightness `0.8` and deviation `0.05`, closing gap `0.15`, minimum turn `0.8`, corner tolerance `0.08`, mean / max residual `0.035` / `0.1` (of the box diagonal).
- The current design is a veto chain: any one limit failing means no fit. A sloppy stroke fails one limit narrowly and gets nothing.
- The hard rule from T109 stays: a stroke that is clearly not a line, box or circle (scribble, zigzag, spiral, triangle, figure eight) must stay a stroke, and a wrong guess costs more than redrawing.
- Draw has no rotation, so the result is always the axis-aligned box of the stroke.

## Decisions

The owner did not answer the questions; the recommended option of each is taken (ADR-T117-1):

1. **Approach** — (b): each closed candidate is scored, and the best one wins when it clears a looser bar.
2. **Closing** — a gap up to `0.3` of the length, and strokes that overshoot or overlap their start; a short tail (the worst `4 %` of the samples) does not count against the shape.
3. **Corners** — a rectangle needs three of the four box corners (within `0.12` of the diagonal); an ellipse allows at most one (within `0.06`).
4. **Lines** — straightness `0.75` and deviation `0.1` of the chord.

The residual limits go to `0.05` (mean) and `0.14` (largest) of the box diagonal, the turn to `0.7` of a full turn. When a rectangle and an ellipse both fit, the one with the score below `0.9` of the other wins; otherwise the stroke stays as drawn.

1. **Approach.** (a) Loosen the constants only; (b) score each candidate shape and take the best one when it clears a looser bar (recommended: a near-miss on one limit no longer vetoes, while a bad fit on several still does); (c) a second, looser pass when the strict one finds nothing.
2. **Closing.** Accept a gap up to about `0.3` of the length, and strokes that overshoot or overlap their start (recommended: yes to both).
3. **Corners.** Rounded or three-and-a-half-cornered rectangles become a rectangle when the box fits better than the ellipse; today they fall to nothing.
4. **Lines.** A mildly curved or wobbly stroke becomes a line when its ends are far apart compared with its bend.

## Spec

- **AC-1** — A corpus of sloppy samples (hurried circles that overshoot or do not close, rounded and lopsided rectangles, wobbly lines, strokes with a tail) each becomes the expected shape. The corpus is generated deterministically in the tests, with a stated amount of noise per family.
  Tests: `recognize::tests::fit_sloppy_*` (one per family: hurried circle, circle that overshoots, circle that stops short, circle with a tail, squashed ellipse, lopsided rectangle, rectangle that stops short, rectangle that overshoots, rounded rectangle, wobbly line, curved line).
- **AC-2** — The T109 rejects stay rejected: zigzag, spiral, triangle, figure eight, open arcs, an `L`, scribbles, and the degenerate strokes.
  Tests: the existing `fit_scribbles_gives_nothing`, `fit_open_curves_never_close`, `fit_degenerate_strokes_gives_nothing`, plus `fit_loose_still_rejects_the_near_misses` (L, U, open C, half circle, hooked line, wave, sloppy triangle) and `fit_loose_still_rejects_noisy_scribbles`.
- **AC-3** — A table in the tests lists, for each family, the noise level where it stops being recognised, so the tolerance is measured and a later change shows up as a diff rather than a feeling.
  Test: `recognize::tests::tolerance_table_matches_the_adr`.
- **AC-4** — Ambiguity is decided by fit, never by a fixed order: a stroke that fits both a rectangle and an ellipse becomes the one with the smaller residual; a near tie keeps the stroke.
  Tests: `fit_rounded_square_picks_the_better_fit`, `fit_near_tie_gives_nothing` (a circle that squares off through the superellipse exponents goes ellipse, nothing, rect).
- **AC-5** — Everything else of T109 is unchanged: one undo step, the stroke's colour and width, `Z` only right after a stroke. Fuzzed as before, with sloppier generated strokes added.
  Tests: the T109 editor tests unchanged; the recognition fuzz of `tests/fuzz.rs` keeps passing.

## Out of scope

- Triangles, arrows, polygons; recognising while drawing without a key.
- A user setting for the tolerance (the owner may ask for one after trying it).

## Files owned

- `src/core/recognize.rs`, `src/core/editor.rs` (tests only, if needed)
- `docs/features/Drawing tools.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T117-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T117): specify looser shape recognition`
- [x] tests — `test(T117): add failing tests for looser shape recognition`
- [x] models — `feat(T117): add the shape scores`
- [x] behaviour — `feat(T117): accept sloppier strokes on Z`
- [x] quality — `chore(T117): pass clippy and rustfmt`
- [x] docs — `docs(T117): update the feature note`

## Learning path

None: it tunes an existing feature.

## Log

- 2026-11-11 — written with the owner after trying 1.5.0.
- 2026-11-12 — done with the recommended options; `scripts/check.sh --fuzz` run below. The tolerance table is in the ADR and pinned by `tolerance_table_matches_the_adr`. Not tried by the owner's hand yet: the corpus is generated, so real strokes may still miss.
