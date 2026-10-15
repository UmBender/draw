---
id: T32
title: Fast text rendering
status: in-progress
wave: 16
branch: task/T32-fast-text
depends_on: [T29]
adrs: ["[[ADR-T32-1 Own glyph atlas rasterized with fontdue]]", "[[ADR-T32-2 Shapes and glyphs share one textured batch]]"]
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

Decisions: [[ADR-T32-1 Own glyph atlas rasterized with fontdue]] (own
atlas, direct `fontdue` pinned to macroquad's 0.9, page size and limits,
clearing) and [[ADR-T32-2 Shapes and glyphs share one textured batch]]
(white texel, page-bound chunks). Tests live in `src/shell/glyphs.rs`,
`src/shell/render.rs` and `src/shell/app.rs` (`#[cfg(test)]`, no GL
context needed) and `tests/perf.rs`.

- **AC-1 — Measured.** `DRAW_FRAME_TIMES=1` re-render lines carry the
  CPU time spent on text and the number of glyphs rasterized in that
  frame: `draw: full re-render 12.345 ms (text 3.210 ms, 45 rasterized),
  5120 shapes`; without text stats the line is unchanged.
  Before numbers on the target are in the *Log* (2026-10-14).
  Tests: `app::tests::frame_log_line_names_layers_time_and_shapes`
  (unchanged lines), `frame_log_line_with_text_stats_names_them`.
- **AC-2 — Glyph quads.** `glyphs::glyph_quad(baseline, &entry, scale)`
  maps a glyph's baseline point (screen), its atlas entry and the
  logical-pixels-per-texel scale to a screen quad and UVs;
  `glyphs::raster_px(raster, dpi)` is `ceil(raster × dpi)` clamped to
  `1..=MAX_RASTER_PX`; `glyphs::glyph_scale(px, raster_px)` is
  `px / raster_px`. Tests: `glyph_quad_sits_on_baseline_after_bearing`,
  `glyph_quad_scales_bitmap_by_scale`, `glyph_quad_uvs_cover_entry_texels`,
  `glyph_quad_empty_bitmap_is_none`, `glyph_quad_non_finite_is_none`,
  `raster_px_scales_by_dpi_and_clamps`,
  `glyph_scale_at_dpi_two_keeps_logical_size`.
- **AC-3 — Atlas.** `ShelfPacker` places rectangles `GAP` texels apart,
  inside the page, deterministically; `GlyphAtlas` rasterizes each
  `(char, raster px)` once, reserves the white block first on every page,
  grows to at most `MAX_PAGES` pages, then reports an overflow, and
  `clear` resets it. Tests: `packer_first_rect_is_at_gap`,
  `packer_rejects_rect_larger_than_page`, `packer_full_page_returns_none`,
  proptests `packer_never_overlaps_or_leaves_page`,
  `packer_is_deterministic`; `atlas_embedded_font_loads`,
  `atlas_rasterizes_each_key_once`, `atlas_space_has_no_bitmap`,
  `atlas_new_page_starts_with_white_block`,
  `atlas_bitmap_matches_fontdue`, `atlas_full_reports_overflow_not_growth`,
  `atlas_clear_resets_entries_and_overflow`; proptest
  `atlas_entries_stay_inside_their_page`.
- **AC-4 — Batched drawing.** `render::push_text_block` and
  `render::push_text_centered` add glyph quads to a `Batch` from an
  atlas; text blocks, number labels and grid indices are drawn through
  them. Shapes and glyphs share chunks; a chunk is flushed only when full
  or when a glyph needs another page. Tests (`render.rs`):
  `batch_shape_vertices_use_white_uv`,
  `batch_glyphs_and_shapes_share_a_chunk`,
  `batch_glyph_on_other_page_flushes`,
  `push_text_block_10k_glyphs_fill_whole_chunks`,
  `push_text_block_keeps_z_order_with_shapes`,
  `push_text_block_skips_whitespace_and_tiny_text`,
  `push_text_centered_centres_on_rect`; existing `batch_*` tests stay
  green. `app::tests::glyph_overflow_rerenders_once` for the
  clear-and-re-render rule.
- **AC-5 — Same look.** Pixel comparison of the document layer before and
  after the change at 0.05×, 1× and 20× zoom, with Latin, Greek and
  math characters, number labels and grid indices (scene and result in
  the *Log*).
- **AC-6 — Budgets.** On the target with 512 blocks: full re-render
  < 16 ms zoomed out and at medium zoom, including the first render at a
  new raster size ([[ADR-T24-5 Redraw budgets]]); overlay-only frames
  with an open block < 2 ms ([[ADR-T26-2 Gesture frame budget]]).
  `tests/perf.rs`: `perf_tessellate_10k_glyphs_is_under_1ms`. Release
  binary still under 2 MB.
- **AC-7 — No regressions.** `scripts/check.sh` green; the T14/T29 perf
  tests stay green.

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
- 2026-10-14 — Before numbers, measured on the target (i5-8250U, Wayland,
  1280×800 and 944×1024 windows, DPI 1, MSAA 4) with a throw-away
  harness (not committed) that builds the owner's scene — 8 blocks of 20
  characters doubled six times with `Ctrl+A`, `Ctrl+D`: 512 blocks,
  9 728 characters — and times 41 full re-renders per zoom, splitting off
  the time inside text drawing:

  | zoom (raster) | first render at the size | steady median | text in steady |
  |---|---|---|---|
  | 0.3 (12 px) | 80–161 ms | 10.9–12.1 ms | 7.6–7.9 ms |
  | 0.5 (12 px, already cached) | 12.1–12.7 ms | 10.0–10.6 ms | 7.0–7.6 ms |
  | 1 (20 px) | 92–133 ms | 10.9–13.2 ms | 7.5–8.8 ms |
  | 2 (40 px) | 124–127 ms | 7.4–11.3 ms | 5.0–7.3 ms |

  The 68 ms of T29 was a first render at a new size: macroquad re-uploads
  its whole atlas texture after each newly cached glyph. Steady renders
  are within 16 ms but text is ~70 % of them (≈ 0.8 µs per character).
  Before images for AC-5 captured with the same harness.
- 2026-10-14 — Spec: ADR-T32-1, ADR-T32-2; ACs name their tests.
