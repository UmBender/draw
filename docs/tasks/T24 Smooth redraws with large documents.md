---
id: T24
title: Smooth redraws with large documents
status: ready
wave: 13
branch: task/T24-layered-redraw
depends_on: [T14]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-T07-1 Screen-space tessellation in the renderer]]", "[[ADR-T12-1 Blocking event loop with cached frame]]", "[[ADR-T15-1 MSAA on the cached frame]]", "[[ADR-T14-1 Performance budgets]]"]
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

Draft — refined in the spec step (each AC names its tests, ADRs added).

- **AC-1 — Measure first.** A frame timer in the shell (debug-only or behind
  an env var, no new dependency) logs re-render time and shape count. Record
  on the target, before any change: drawing a circle, dragging a selection,
  erasing, panning and zooming with ≈ 5 000 shapes, plus where `perf record`
  puts the time (tessellation in `shell::render`, `QuadGl` batching/upload,
  or driver fill/MSAA resolve). Results in the *Log*; they decide which of
  AC-2–AC-4 are needed.
- **AC-2 — Two layers** (option 1). The committed document is rendered into
  its own cached target; a change that only touches the overlay (preview,
  marquee, guides, selection box, toolbar hover) blits that layer and draws
  the overlay on top. The document layer is re-rendered only when the
  document, the hidden set (`Overlay::hidden`), the camera or the viewport
  changes. Core reports *what* changed — preferably additively (e.g.
  revision counters for document and view on `Editor`) so `handle`/`apply`
  keep their `bool` and existing callers do not change. *Tests:* core unit
  tests that each command/gesture bumps exactly the right revision; shell
  logic that picks the layer to re-render is a pure function with unit
  tests.
- **AC-3 — Cheaper full re-renders**, if AC-1 shows pan/zoom are still slow
  (they always re-render the document layer): cheaper tessellation (option
  4: lower ellipse segment counts, one mesh per shape via `draw_mesh`)
  and/or capping re-renders at the display refresh (option 6). Each one
  kept only if the AC-1 timer shows a gain.
- **AC-4 — Budget.** With the AC-1 scenario, an overlay-only change re-renders
  in < 2 ms and a full re-render stays < 16 ms on the target; numbers in the
  *Log*, budget in an ADR amending [[ADR-T14-1 Performance budgets]].
- **AC-5 — No regressions.** Output looks the same (MSAA, labels, grids,
  fills, selection); manual check on the target listed in the *Log*.
  `scripts/check.sh` and the T14 perf tests stay green.

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

- [ ] spec — `docs(T24): specify layered redraw acceptance criteria`
- [ ] tests — `test(T24): add failing tests for change reporting`
- [ ] models — `feat(T24): add change revisions and layer types`
- [ ] behaviour — `feat(T24): render the document and overlay layers separately`
- [ ] quality — `chore(T24): pass clippy and rustfmt`
- [ ] docs — `docs(T24): add tutorial and measurements`

## Learning path

Step 24 — requires step 12 (app loop, cached frame), step 15 (MSAA) and
step 14 (profiling).

## Log

- 2026-10-02 — Created from the T14 AC-3 finding; not started.
