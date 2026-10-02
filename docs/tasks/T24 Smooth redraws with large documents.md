---
id: T24
title: Smooth redraws with large documents
status: review
wave: 13
branch: task/T24-layered-redraw
depends_on: [T14]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-T07-1 Screen-space tessellation in the renderer]]", "[[ADR-T12-1 Blocking event loop with cached frame]]", "[[ADR-T15-1 MSAA on the cached frame]]", "[[ADR-T14-1 Performance budgets]]", "[[ADR-T24-1 Document layer keyed by a document revision]]"]
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
- **AC-3 — Cheaper full re-renders.** Conditional on AC-1: only if the
  owner's numbers show pan/zoom still slow. Each option kept only if the
  timer shows a gain. Decided in the *Log*.
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
