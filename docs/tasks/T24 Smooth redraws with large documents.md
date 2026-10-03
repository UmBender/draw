---
id: T24
title: Smooth redraws with large documents
status: in-progress
wave: 13
branch: task/T24-layered-redraw
depends_on: [T14]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-T07-1 Screen-space tessellation in the renderer]]", "[[ADR-T12-1 Blocking event loop with cached frame]]", "[[ADR-T15-1 MSAA on the cached frame]]", "[[ADR-T14-1 Performance budgets]]", "[[ADR-T24-1 Document layer keyed by a document revision]]", "[[ADR-T24-2 Overlay drawn straight to the window]]", "[[ADR-T24-3 Batched meshes and append-only document updates]]", "[[ADR-T24-4 Strokes as one strip with sparse round joins]]"]
feature:
tutorial: "[[24 Layered redraws and measuring a frame]]"
tags: [task]
---

# T24 Smooth redraws with large documents

## Goal

A 5 000-shape session feels as smooth as an empty canvas while drawing,
moving and erasing. Today every state change re-tessellates and re-uploads
the **whole scene** into the cached frame, so the cost of one pointer move
grows with the number of shapes, not with what changed.

Found by the owner in [[T14 Performance and release validation]] (AC-3,
2026-10-02): after `Ctrl+A`, `Ctrl+D` ×9 (≈ 5 120 shapes) the app gets
clunky; CPU peaks at ≈ 10 %; idle stays ≈ 0 %. The headless hot paths are
far inside budget (hit-test ≈ 0.35 ms, culling ≈ 0.23 ms at 10 000 shapes,
[[ADR-T14-1 Performance budgets]]), so the time is in drawing. Options and
their trade-offs: [[Rendering performance options]].

## Spec

How the shell decides what to redraw:
[[ADR-T24-1 Document layer keyed by a document revision]]. Unit tests live in
`#[cfg(test)] mod tests` of the named module.

- **AC-1 — Measure first.** `DRAW_FRAME_TIMES` (env var, no new dependency)
  turns on a frame timer in `shell::app`: every re-render prints one line to
  stderr with the layers re-rendered, the CPU time spent and the shape count.
  Off by default; `1`, `on`, `true`, `yes` (any case, trimmed) turn it on.
  The timer measures CPU time to submit the frame (tessellation + batching +
  GL calls); GPU fill and MSAA resolve run asynchronously and show up in
  `perf` / CPU usage, not in the line.
  *Tests* (`shell::app`): `frame_timing_enabled_unset_is_off`,
  `frame_timing_enabled_on_values_enable`,
  `frame_timing_enabled_other_values_disable`,
  `frame_log_line_names_layers_time_and_shapes`.
  *Owner on the target:* drawing a circle, dragging a selection, erasing,
  panning and zooming with ≈ 5 000 shapes, plus `perf record`; results in the
  *Log*. Pan and zoom always re-render the document layer, so their numbers
  are also the per-event cost before this task.
- **AC-2 — Two layers.** The committed document (underlay dot grid + visible,
  non-hidden shapes) is rendered into its own cached MSAA target; the frame
  target blits it and draws the overlay (preview shapes, guides, selection,
  marquee, toolbar) on top. The document layer is re-rendered only when its
  key changes: document revision, camera, framebuffer size, hidden ids or
  grid snap (underlay).
  - Core: `Editor::document_revision() -> u64` grows whenever the document
    may have changed and stays put otherwise; `handle`/`apply` keep their
    `bool`. *Tests* (`core::editor`):
    `document_revision_new_editor_is_zero`,
    `document_revision_pen_drag_moves_keep_revision`,
    `document_revision_pen_commit_bumps_revision`,
    `document_revision_select_click_and_marquee_keep_revision`,
    `document_revision_select_all_and_copy_keep_revision`,
    `document_revision_move_drag_bumps_only_on_release`,
    `document_revision_delete_paste_duplicate_bump_revision`,
    `document_revision_undo_redo_clear_bump_revision`,
    `document_revision_noop_undo_keeps_revision`,
    `document_revision_eraser_bumps_only_on_release`,
    `document_revision_bucket_fill_bumps_revision`,
    `document_revision_view_and_settings_keep_revision`,
    `document_revision_commit_with_full_history_bumps_revision`.
  - Shell: `shell::app::plan_redraw(cached, next, dirty) -> Redraw` is a pure
    function over `LayerKey`s. *Tests* (`shell::app`):
    `plan_redraw_without_cache_is_full`,
    `plan_redraw_same_key_clean_is_none`,
    `plan_redraw_same_key_dirty_is_overlay`,
    `plan_redraw_changed_key_is_full` (each key field in turn, dirty or not).
- **AC-3 — Cheaper re-renders** (decided from the owner's AC-1 numbers,
  see *Log*): an overlay drag costs 0.15 ms CPU but feels clunky (GPU
  bound — smoother with `DRAW_MSAA=off`); a commit costs a 34 ms full
  re-render; `perf` puts ≈ 60 % of a pan session in per-triangle
  submission (`memmove` from `QuadGl::geometry`, `draw_triangle`).
  - **AC-3a — Batched meshes**
    ([[ADR-T24-3 Batched meshes and append-only document updates]]).
    `shell::render` collects triangles in a `Batch` (shared vertices for
    quads and fans) and submits them with one `draw_mesh` per chunk of at
    most `BATCH_MAX_VERTICES` / `BATCH_MAX_INDICES`, flushing before text so
    z-order holds. *Tests* (`shell::render`, through a recording sink):
    `batch_triangle_adds_three_vertices`,
    `batch_quad_shares_four_vertices`,
    `batch_fan_shares_the_center`,
    `batch_line_is_a_width_wide_quad`,
    `batch_line_of_zero_length_draws_nothing`,
    `batch_flushes_before_exceeding_limits`,
    `batch_finish_draws_the_rest_once`,
    `batch_indices_stay_inside_their_chunk` (proptest).
  - **AC-3b — Overlay straight to the window**
    ([[ADR-T24-2 Overlay drawn straight to the window]]). The cached frame
    target goes away: every woken frame blits the document layer and draws
    the overlay on the window. Only the document layer is multisampled.
    *Test:* covered by AC-2's plan tests (the plan no longer has a frame
    target); manual check in AC-5.
  - **AC-3c — Append-only updates** (ADR-T24-3). When the only document
    change since the cached layer is shapes added on top (pen, line, arrow,
    rect, ellipse, grid commits, paste, duplicate) and nothing else in the
    key changed, only the new shapes are drawn onto the cached layer.
    Core: `Editor::document_base_revision()` — the revision of the latest
    change that was not a pure append. *Tests* (`core::editor`):
    `document_base_revision_new_editor_is_zero`,
    `document_base_revision_creation_commits_keep_base`,
    `document_base_revision_paste_and_duplicate_keep_base`,
    `document_base_revision_rewrites_raise_base`.
    Shell: `plan_redraw` gains the base revision and returns
    `Redraw::Append { from }`. *Tests* (`shell::app`):
    `plan_redraw_appended_shapes_only_is_append`,
    `plan_redraw_append_after_rewrite_is_full`,
    `plan_redraw_append_with_other_change_is_full`,
    `plan_redraw_shorter_document_is_full`.
  - The document layer skips hidden shapes through a hash set
    (`O(n + h)` instead of `O(n · h)` while moving a large selection).
  - **AC-3d — Strokes as strips**
    ([[ADR-T24-4 Strokes as one strip with sparse round joins]]), decided
    from the third round of numbers (pen-only scene of 5 120 strokes:
    20–46 ms zoomed out, ≈ 210 ms at medium zoom). `Batch::polyline`
    draws a polyline of screen points as one triangle strip, 2 vertices
    per point, with mitred joins. Points closer than
    `STROKE_MIN_STEP_PX` to the last kept point are skipped. Where the
    mitre would exceed `MITER_LIMIT` half-widths the strip breaks and
    restarts. Strokes wider than `JOINT_THRESHOLD_PX` get a round disc at
    those breaks and at both ends only. `Batch::strip` continues across
    chunks, so a strip may be longer than one chunk. *Tests*
    (`shell::render`, through the recording sink):
    `batch_polyline_empty_draws_nothing`,
    `batch_polyline_single_point_is_a_dot`,
    `batch_polyline_coincident_points_are_a_dot`,
    `batch_polyline_thin_shares_two_vertices_per_point`,
    `batch_polyline_thick_adds_round_caps_only_at_ends`,
    `batch_polyline_gentle_turn_is_mitred`,
    `batch_polyline_sharp_turn_breaks_with_round_join`,
    `batch_polyline_skips_points_closer_than_min_step`,
    `batch_strip_longer_than_a_chunk_continues`,
    `batch_polyline_indices_stay_inside_their_chunk` (proptest).
- **AC-4 — Budget.** With the AC-1 scenario, an overlay-only change re-renders
  in < 2 ms and a full re-render stays < 16 ms on the target; numbers in the
  *Log*, budget in an ADR amending [[ADR-T14-1 Performance budgets]] once
  measured.
- **AC-5 — No regressions.** Output looks the same (MSAA, labels, grids,
  fills, selection, toolbar on top); manual check on the target listed in the
  *Log*. `scripts/check.sh` and the T14 perf tests stay green.

## Out of scope

- Retained GPU buffers or anything needing `unsafe` (U1–U3 in
  [[Rendering performance options]]) — would amend
  [[ADR-0014 Minimal dependencies and no unsafe]].
- New dependencies or a different renderer.
- Dirty-rectangle redraws (option 7).
- Caching snap `Targets` during a drag (≈ 1 ms at 10 000 shapes) — separate
  task if AC-1 shows it matters.
- The stuck-`Alt` after `Alt+Tab` noted in T14's log.

## Files owned

- `src/shell/app.rs`
- `src/shell/render.rs`
- `src/core/editor.rs` (change reporting only)
- `tests/perf.rs` (only to add benchmarks for new pure code)
- new ADRs `ADR-T24-*`, tutorial `docs/tutorials/24 Layered redraws and measuring a frame.md`
- `docs/architecture/Rendering performance options.md`,
  `docs/architecture/Architecture.md` (record what was done)

## Subtasks (one commit each)

- [x] spec — `docs(T24): specify layered redraw acceptance criteria`
- [x] tests — `test(T24): add failing tests for change reporting`
- [x] models — `feat(T24): add change revisions and layer types`
- [x] behaviour — `feat(T24): render the document and overlay layers separately`
- [x] quality — `chore(T24): pass clippy and rustfmt`
- [x] docs — `docs(T24): add tutorial and measurements`

Second round (AC-3, after the owner's measurements):

- [x] spec — `docs(T24): specify batching and append-only criteria`
- [x] tests — `test(T24): add failing tests for batching and appends`
- [x] models — `feat(T24): add the triangle batch and base revision`
- [x] behaviour — `feat(T24): batch meshes, append shapes, draw overlay on window`
- [x] quality — `chore(T24): pass clippy and rustfmt after batching`
- [x] docs — `docs(T24): document batching and append-only updates`

Third round (AC-3d, after the owner's pen-only measurements):

- [x] spec — `docs(T24): specify strokes as strips`
- [ ] tests — `test(T24): add failing tests for stroke strips`
- [ ] models — skipped: no new types; `Batch::polyline` is a method on
  the existing `Batch`, added with its body in the behaviour step
  (signature-only stubs would not compile cleanly with the tests anyway)
- [ ] behaviour — `feat(T24): draw strokes as strips with sparse joins`
- [ ] quality — `chore(T24): pass clippy and rustfmt after strips`
- [ ] docs — `docs(T24): document stroke strips`

## Learning path

Step 24 — requires step 12 (app loop, cached frame), step 15 (MSAA) and
step 14 (profiling).

## Log

- 2026-10-02 — Created from the T14 AC-3 finding; not started.
- 2026-10-02 — Spec refined; ADR-T24-1 added.
- 2026-10-02 — AC-1 timer, AC-2 two layers and document revision
  implemented; 13 editor + 8 shell unit tests. `scripts/check.sh` green.
  T14 perf tests (release) green: culling 0.23 ms, eraser scan 0.34 ms,
  hit-test 0.38 ms, RDP 83 µs, snap drag 1.13 ms, stroke finish 17 µs.
  Smoke run with `DRAW_FRAME_TIMES=1`: the first frame logs
  `full re-render 73.7 ms, 0 shapes` (includes creating the targets and
  first-use GL setup).
- 2026-10-02 — **Pending, owner on the target:** AC-1 scenario numbers
  (circle drag, selection drag, erase, pan, zoom with ≈ 5 000 shapes;
  `perf record`), AC-5 visual check (MSAA, labels, grids, fills, selection,
  toolbar). AC-3 is decided from those numbers. The AC-4 budget ADR
  (amending ADR-T14-1) is written once the numbers exist. Pan and zoom
  always take the `full` path, so their lines also show the per-event cost
  before T24.
- 2026-10-02 — Owner's AC-1 numbers on the target (≈ 5 600 shapes):
  circle drag `overlay re-render` 0.12–0.17 ms CPU, release
  `full re-render` 33.8 ms; still clunky while dragging, smoother with
  `DRAW_MSAA=off` (GPU fill of the 4× MSAA frame target on every move).
  `perf record` of a pan session (52 s, symbols on): `memmove` 38 %
  (vertex copies in `QuadGl::geometry`, one per `draw_triangle`),
  inlined app loop 15 %, `QuadGl::geometry` 9 %, `draw_triangle` 6 %,
  `draw_ellipse_fill` 4 %; ≈ 13 % in `ShapeId` equality from the linear
  `Document` lookups → split out as
  [[T25 Linear-time selection with large documents]]. AC-3 decided:
  AC-3a–c below; T24 stays within its files owned.
- 2026-10-02 — AC-3a–c implemented (ADR-T24-2, ADR-T24-3). Tests added
  beyond the spec list: `batch_limits_fit_a_macroquad_draw_call`
  (compile-time check of the chunk limits),
  `frame_log_line_names_appends`. `scripts/check.sh` green (588 lib
  tests); T14 perf tests (release) green; release smoke run starts and
  logs its first `full` re-render. `tests/perf.rs` unchanged (the new
  pure code is the chunking, covered by unit tests; T25 owns new
  benchmarks).
- 2026-10-02 — **Pending, owner on the target:**
  - AC-4: with ≈ 5 000 shapes and `DRAW_FRAME_TIMES=1`, a circle drag
    (`overlay` lines, and whether it now *feels* smooth), its release
    (expect one `append` line), and pan/zoom (`full` lines vs the 33.8 ms
    before). The budget ADR amending ADR-T14-1 is written from those
    numbers.
  - AC-5 visual check: committed shapes, labels, grids, fills, MSAA on
    the document. Previews and toolbar icons are now single-sampled
    (ADR-T24-2).
- 2026-10-03 — Owner's AC-4 numbers after AC-3a–c, pen-only scene of
  5 120 strokes: zoomed out, pan/zoom `full re-render` 19–46 ms
  (≈ 26 ms typical; was 33.8 ms); at medium zoom ≈ 210 ms. `perf` of the
  medium-zoom session (symbols from an unstripped build of the same
  commit): ≈ 55 % `glBufferSubData` copies (libc `memmove` + Mesa),
  ≈ 10 % building vertices (`draw_ellipse_fill`, `Batch`), ≈ 11 %
  `Document::index_of` per selected id inside `present` (T25). Strokes
  wider than 2 px on screen get a disc at every point (≈ 14 vertices per
  point instead of 4), which only happens once zoomed in. AC-3d added.
  Not in T24: drawing the cached layer moved/scaled during a gesture and
  re-rendering once input is quiet (needs a timed wake-up, amending
  ADR-T12-1) — a separate task if AC-3d is not enough.
