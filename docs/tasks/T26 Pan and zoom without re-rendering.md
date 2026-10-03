---
id: T26
title: Pan and zoom without re-rendering
status: ready
wave: 14
branch: task/T26-gesture-redraw
depends_on: [T24]
adrs: ["[[ADR-T12-1 Blocking event loop with cached frame]]", "[[ADR-T24-1 Document layer keyed by a document revision]]", "[[ADR-T24-5 Redraw budgets]]"]
feature:
tutorial: "[[26 Reusing a frame during a gesture]]"
tags: [task]
---

# T26 Pan and zoom without re-rendering

## Goal

Panning and zooming a 5 000-stroke scene is smooth at any zoom level.
Today every pan or zoom event changes the camera, which is part of the
document layer's key
([[ADR-T24-1 Document layer keyed by a document revision]]), so every
event re-renders the whole layer. That takes ≈ 15 ms zoomed out and
40–60 ms at medium zoom on the target, over the 16 ms budget of
[[ADR-T24-5 Redraw budgets]].

Found during [[T24 Smooth redraws with large documents]] (2026-10-03),
from the owner's idea of deferring MSAA until a gesture ends. MSAA is
mostly GPU time and the remaining cost is CPU (vertices and uploads), so
this task skips the whole re-render during a gesture instead.

## Spec

Draft — refined in the spec step (each AC names its tests, ADRs added).

- **AC-1 — Reuse during a gesture.** While only the camera changed since
  the cached document layer, the frame draws that layer moved and scaled
  by the camera difference (one textured quad) instead of re-rendering
  it. The overlay is drawn as today. *Tests:* a pure function from the
  cached and current camera to the blit rectangle (`shell::app`), and
  `plan_redraw` returning a new `Redraw` case for camera-only changes.
- **AC-2 — Settle.** Once no camera change has happened for
  `SETTLE_MS` (≈ 100 ms), the layer is re-rendered at the current camera
  with MSAA. This needs a wake-up after a quiet period. The loop sleeps
  until an input event (ADR-T12-1). A candidate is a timer thread calling
  `miniquad::window::schedule_update`, whose thread-safety must be checked
  first. The choice is recorded in an ADR amending ADR-T12-1.
- **AC-3 — Margin.** The layer is rendered with a margin around the
  window, so short pans show shapes rather than empty edges. Size and
  memory cost are recorded in the ADR.
- **AC-4 — Any other change wins.** A document, hidden-set, size or
  underlay change during a gesture re-renders as today. *Tests:* the
  existing `plan_redraw` key tests stay green.
- **AC-5 — Budget.** With the ADR-T24-5 scene, pan and zoom frames during
  a gesture cost < 2 ms CPU at any zoom. The settle re-render is listed
  separately. Numbers go in the *Log*, and ADR-T24-5 is amended.
- **AC-6 — No regressions.** Output after settling looks the same as
  today; `scripts/check.sh` and the T14 perf tests stay green.

## Out of scope

- Cutting the cost of the settle re-render itself (geometry,
  [[T25 Linear-time selection with large documents]]).
- Retained GPU buffers or anything needing `unsafe`.

## Files owned

- `src/shell/app.rs`
- `src/shell/render.rs` (blit helpers only)
- new ADRs `ADR-T26-*`, tutorial `docs/tutorials/26 Reusing a frame during a gesture.md`
- `docs/architecture/Rendering performance options.md`,
  `docs/architecture/Architecture.md` (record what was done)

## Subtasks (one commit each)

- [ ] spec — `docs(T26): …`
- [ ] tests — `test(T26): …`
- [ ] models — `feat(T26): …`
- [ ] behaviour — `feat(T26): …`
- [ ] quality — `chore(T26): …`
- [ ] docs — `docs(T26): …`

## Learning path

Step 26 — requires step 24 (layered redraws) and step 12 (app loop).

## Log

- 2026-10-03 — Created from T24's AC-4 result (medium zoom 40–60 ms);
  not started. Best measured after T25 lands, so the settle numbers do
  not include the selection lookups.
