---
id: T146
title: Retained renderer in draw-gfx
status: in-progress
wave: 71
branch: task/T146-retained-renderer-in-draw-gfx
depends_on: [T145, T135, T136, T153, T161]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T146 Retained renderer in draw-gfx

## Goal

Shapes are tessellated once, kept in GPU buffers per chunk and drawn with the camera in the vertex shader.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §6. Removes the document layer re-render, the settle re-render and the blit of [[T26 Pan and zoom without re-rendering]].
- The risk is cache invalidation, so every cached geometry is compared with a recomputed one.

## Spec

Design (new modules in `draw-gfx`, which gains a path dependency on `draw-core`, a workspace crate, so no new external dependency):

- `tessellate`: `Mesh { vertices, indices }` in **world** coordinates (`Vertex` of the `Gpu` trait, `uv` 0, straight colour from the palette) and `tessellate(&Shape) -> Mesh`. Non-finite shapes give an empty mesh. Text shapes give an empty mesh here (glyph primitives belong to the app wiring, T172). `Mesh::geometry()` gives a `Geometry::Inline` that T173 can upload unchanged.
- `chunks`: shapes in z-order are grouped in fixed chunks of `CHUNK_LEN` (64) consecutive shapes, which keeps the z-order exact. A chunk's key is its list of `(ShapeId, revision)`; `ChunkCache::update` rebuilds a chunk only if its key changed. The caller supplies the revision (it grows on each edit of a shape).

- **AC-1** — Chunks keyed by shape id and revision; an edit rebuilds only the chunks it touches.
  Tests: `edit_rebuilds_only_its_chunk`, `unchanged_update_tessellates_nothing`, `insert_at_end_rebuilds_only_the_last_chunk`, proptest `cached_geometry_equals_recomputed` over edit sequences (replace, insert, remove).
- **AC-2** — A camera change re-tessellates nothing (the chunks are in world space); culling picks chunks by world bounds.
  Tests: `pan_and_zoom_tessellate_nothing`, `visible_chunks_follow_the_view`, `tessellate_*` per shape kind (`tessellate_line_covers_its_width`, `tessellate_filled_rect_has_fill_and_outline`, `tessellate_non_finite_is_empty`). The camera as a `Gpu` uniform is [[T173 Camera uniform and analytic anti-aliasing over Gpu]]; the app path and the e2e goldens are [[T172 Retained renderer in the app]].
- **AC-3** — moved to [[T173 Camera uniform and analytic anti-aliasing over Gpu]] (analytic anti-aliasing).
- **AC-4** — The tessellation time is a benchmark in `tests/perf.rs` (`perf_tessellation_of_5k_strokes_is_measured`, 5120 strokes), recorded in the Log. The ADR-T24-5 budgets in the app are [[T172 Retained renderer in the app]].

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.
- The `Gpu` trait and its interpreters ([[T173 Camera uniform and analytic anti-aliasing over Gpu]]).
- Wiring the renderer into `src/shell`, the e2e goldens and the frame budgets in the app ([[T172 Retained renderer in the app]]).

## Files owned

- `crates/draw-gfx/src/**`, `crates/draw-gfx/tests/**`, `crates/draw-gfx/Cargo.toml` (`proptest` as a dev-dependency, already in the workspace), `tests/perf.rs`
- this note and `ADR-T146-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T146): specify the retained renderer`
- [ ] tests — `test(T146): add failing tests for the chunks`
- [ ] models — `feat(T146): add the chunk and geometry types`
- [ ] behaviour — `feat(T146): implement the retained renderer`
- [ ] quality — `chore(T146): pass clippy and rustfmt`
- [ ] docs — `docs(T146): record the tessellation benchmark`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-15 — scoped by the owner to `draw-gfx`: tested on the recording `Gpu` and headless GL; the app wiring, the e2e goldens and the frame budgets (AC-2 app part, AC-4 budgets) moved to [[T172 Retained renderer in the app]].
- 2026-11-15 — BLOCKED (batch): AC-2 and AC-3 need the camera as a uniform and an analytic-AA fragment shader, but `Command`/`View`/`Geometry` in the `Gpu` trait carry no camera, and any change to them breaks the other interpreters (`src/shell/gpu_mq.rs`, `src/shell/recording.rs`, `src/shell/render_ctx.rs`, `tests/recorder.rs`), which are outside *Files owned*. `crates/draw-gfx/tests/**` and `crates/draw-gfx/Cargo.toml` (a `proptest` dev-dependency for AC-1's `cached_geometry_equals_recomputed`) are also not owned. Nothing was implemented. Recommended fix: owner decides one of (a) add the files above plus `crates/draw-gfx/tests/**` and `Cargo.toml` to *Files owned*, with an ADR amending ADR-T140-1 that adds a camera (`origin`, `scale`) to `Command::BeginPass` and a `fwidth`/SDF feather in `gl.rs`; or (b) split the pure part (chunks, revisions, invalidation, tessellation, benchmark: AC-1 and AC-4) from the GPU part (AC-2, AC-3) into two tasks.
- 2026-11-15 — unblocked by the owner: split; T146 keeps the pure part in `draw-gfx` (chunks, revisions, invalidation, tessellation, the proptest, the benchmark: AC-1, AC-2 without the uniform, AC-4), and the camera uniform and the analytic anti-aliasing go to [[T173 Camera uniform and analytic anti-aliasing over Gpu]]. No change to the `Gpu` trait here.
- 2026-11-15 — spec, tests, models, behaviour and quality steps committed: `crates/draw-gfx/src/{tessellate,chunks}.rs`, `crates/draw-gfx/tests/chunks.rs`, and `perf_tessellation_of_5k_strokes_is_measured` in `tests/perf.rs`. All of `cargo test -p draw-gfx` and clippy pass.
- 2026-11-15 — BLOCKED (batch): `scripts/check.sh` is red only in `tests/deps.rs` (`deps_match_the_adr`): it allows `draw-gfx` only `draw-types` and `draw-sys` (ADR-T132-1 / ADR-T127-3), but tessellating a `Shape` needs `draw-core`, which `draw-gfx/Cargo.toml` now depends on (the crate graph has no cycle: `draw-core` depends on `draw-types` only). `tests/deps.rs` is not in *Files owned*. Not done: the docs step (benchmark numbers in the Log; the benchmark itself is written but not run yet). Recommended fix: owner allows editing `tests/deps.rs` (line 28: `"draw-gfx" => &["draw-types", "draw-sys", "draw-core"]`) and an ADR amending ADR-T132-1 to permit `draw-gfx → draw-core`; then run `cargo test --release --test perf -- --ignored perf_tessellation --nocapture`, record the numbers here, tick the subtasks and set `status: review`. The alternative is a shape-agnostic `draw-gfx` fed meshes by the shell, which changes the AC-1 design.
