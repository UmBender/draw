---
id: T32
title: Fast text rendering
status: ready
wave: 16
branch: task/T32-fast-text
depends_on: [T29]
adrs: []
feature:
tutorial: "[[32 Batching glyphs into the document mesh]]"
tags: [task]
---

# T32 Fast text rendering

## Goal

A page full of text redraws as fast as a page full of strokes. On the
target, a full re-render of 512 text blocks (~10 000 characters) took
**68 ms** (owner check in [[T29 Text blocks]], 2026-10-14) against the
16 ms of [[ADR-T24-5 Redraw budgets]]. Pan and zoom stay smooth thanks to
[[ADR-T26-3 Reuse the layer only when re-rendering is slow]], but every
settle, commit or undo with much text on screen costs a visible hitch, and
[[T30 Basic TeX formulas]] adds 200 formula blocks on top. After this task,
glyphs are tessellated into the same batched meshes as shapes
([[ADR-T24-3 Batched meshes and append-only document updates]]) instead of
one `draw_text_ex` call per character, and text looks the same as before.

## Design points (settle in the spec step)

- **Measure first.** Confirm where the 68 ms goes before choosing a fix:
  time the text part of a full re-render separately (e.g. a second
  `DRAW_FRAME_TIMES` figure for glyphs), and check whether that sample was
  the first render at a new raster size (glyph rasterization) or a steady
  one. The suspected cause is per-call overhead inside macroquad's
  `draw_text_ex` (locks, hash lookups, one quad built per call) — about
  6.8 µs per character.
- **Own glyph atlas** (proposal, ADR-T32-1). macroquad does not expose its
  font atlas, so batching needs our own: rasterize each `(char, raster
  size)` once with `fontdue` (already compiled into the binary through
  macroquad, so ≈ 0 bytes of binary growth), pack it into one texture
  (shelf packer), and emit a textured quad per glyph into a text `Batch`
  drawn with one `draw_mesh` per chunk. A direct `fontdue` dependency
  needs ADR-T32-1 under [[ADR-0014 Minimal dependencies and no unsafe]]
  and must pin the version macroquad already uses (no second copy).
  Alternatives to record: caching each block as a small render target;
  drawing one line per `draw_text_ex` call (likely too little gain).
- **Same look.** Same font, metrics ([[ADR-T29-1 Text shape with monospaced layout and an embedded font]]),
  `TEXT_RASTER_SIZES`, `TEXT_MIN_PX`, colours and z-order (text above the
  shape drawn before it, below the next). Number labels and grid indices
  use the same path.
- **Bounded memory.** The atlas grows by pages up to a fixed limit; when
  full it is cleared and rebuilt on the next full render (at most twelve
  raster sizes × the glyphs on screen). Record the limit in the ADR.
- **Append-only updates** (ADR-T24-3) keep working: committing a new text
  block draws only that block onto the layer.

## Spec

To be refined in the spec step; every AC names its tests there.

- **AC-1 — Measured.** The frame log separates glyph time from the rest
  of a full re-render; the owner's before numbers (512 blocks, zoomed out
  and medium zoom, steady and first-at-new-size) are in the *Log* before
  any fix lands.
- **AC-2 — Glyph quads.** A pure function maps a laid-out glyph, its atlas
  entry, the raster scale and the camera to a screen quad with UVs;
  unit-tested for position (baseline, bearing), scale and DPI.
- **AC-3 — Atlas.** Packing never overlaps, never leaves the texture, and
  is deterministic; a full atlas is cleared, not grown without bound;
  property tests over random glyph sizes.
- **AC-4 — Batched drawing.** Text blocks, labels and grid indices are
  drawn through text batches: at most one draw call per chunk of
  `BATCH_MAX_*` and per atlas texture, never one per character; z-order
  with shapes unchanged.
- **AC-5 — Same look.** Manual side-by-side check against T29 at 0.05×,
  1× and 20× zoom, with Latin, Greek and math characters (screenshots in
  the *Log*).
- **AC-6 — Budgets.** On the target with 512 blocks: full re-render
  < 16 ms zoomed out and at medium zoom ([[ADR-T24-5 Redraw budgets]]),
  `moved` frames still < 2 ms ([[ADR-T26-2 Gesture frame budget]]),
  typing frames < 2 ms. New CPU perf test in `tests/perf.rs`:
  tessellating 10 000 glyphs < 1 ms. Binary still under 2 MB.
- **AC-7 — No regressions.** `scripts/check.sh` and the T14/T29 perf tests
  stay green.

## Out of scope

- Text features (sizes, fonts, wrapping) and formulas
  ([[T30 Basic TeX formulas]]).
- The toolbar's letters (a few dozen glyphs, drawn on the overlay).
- GPU-side techniques (SDF fonts, shaders).

## Files owned

- `src/shell/render.rs`, `src/shell/app.rs`, `src/shell/mod.rs`,
  `src/shell/glyphs.rs` (new: rasterization, atlas, glyph quads)
- `Cargo.toml`, `Cargo.lock` (only the direct `fontdue` dependency, if
  ADR-T32-1 picks it)
- `tests/perf.rs`
- new ADRs `ADR-T32-*`, tutorial
  `docs/tutorials/32 Batching glyphs into the document mesh.md`;
  `docs/features/Text blocks.md` (*How it works* only) and
  `docs/architecture/Architecture.md` if the render path changes

## Subtasks (one commit each)

- [ ] spec — `docs(T32): …`
- [ ] tests — `test(T32): …`
- [ ] models — `feat(T32): …`
- [ ] behaviour — `feat(T32): …`
- [ ] quality — `chore(T32): …`
- [ ] docs — `docs(T32): …`

## Learning path

Step 32 — requires step 24 (layered redraw and batched meshes) and step
29 (text blocks).

## Log

- 2026-10-14 — Created after the owner's T29 check: 512 text blocks,
  `moved` frames 0.48–1.36 ms, one `full` re-render 68 ms. Scheduled
  before [[T30 Basic TeX formulas]]: both own `src/shell/render.rs` and
  `tests/perf.rs`, and T30's formulas should use the batched glyph path.
