---
id: ADR-T26-1
title: Reuse the document layer during pan and zoom
status: accepted
kind: decision
date: 2026-10-03
task: T26
builds_on: [ADR-T12-1, ADR-T15-1, ADR-T24-1, ADR-T24-2]
amends: [ADR-T12-1, ADR-T24-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T26-1 Reuse the document layer during pan and zoom

## Context

The camera is part of the document layer's key
([[ADR-T24-1 Document layer keyed by a document revision]]), so every pan
or zoom event re-renders the whole layer: ≈ 15 ms zoomed out and 40–60 ms
at medium zoom with 5 120 strokes on the target
([[ADR-T24-5 Redraw budgets]]). Panning and zooming change nothing but an
affine map with uniform scale, so the cached layer already holds the
right picture, moved and scaled.

A re-render must still happen once the gesture is over, at full quality.
The loop sleeps until an input event
([[ADR-T12-1 Blocking event loop with cached frame]]), so something has to
wake it after a quiet period. Reading miniquad 0.4.11
(`native/linux_x11.rs`, the backend we use):

- `window::schedule_update` locks the global display mutex and sends a
  request on a channel. The loop only drains that channel at the top of
  an iteration, **before** it blocks in `XNextEvent`. A request sent from
  another thread while the loop sleeps is not seen until the next X event,
  so a timer thread does not wake it.
- The loop's condition is `native_display().try_lock().unwrap()`. A timer
  thread holding that mutex at the wrong moment would make the main thread
  panic.
- Called from the main thread during a frame, `schedule_update` sets
  `update_requested`, and the next iteration does not block.

## Decision

- **`Redraw::Moved`**: `plan_redraw` returns it when the cached and the
  next key differ only in the camera. The frame blits the cached layer to
  `blit_rect(cached camera, current camera, layer rect)`, a pure map of
  the layer's corners through both cameras, with a linear filter; the
  overlay is drawn as today. Any other difference (revision, length,
  size, hidden ids, underlay) still gives `Full`, also together with a
  camera change.
- **Settle**: the shell remembers when the camera last changed. `settle`
  turns `Moved` into `Full` once the camera has been still for
  `SETTLE` = 100 ms; the re-render uses MSAA and the current camera as
  before.
- **Wake-up by polling, main thread only**: while a frame is `Moved`, it
  calls `miniquad::window::schedule_update`, so the loop runs another
  frame instead of sleeping. Polling frames cost one blit plus the
  overlay, are paced by the swap interval (vsync), and stop with the
  settle re-render, after which the loop blocks again. No thread is
  started.
- **Margin**: the layer covers the window plus `layer_margin(size)`
  physical pixels on every side: ⅛ of the longer window side, reduced so
  that no layer side exceeds `MAX_LAYER_SIDE` = 8192. It is rendered with
  `layer_camera`, the view camera shifted by the margin, so the same
  shapes and dots land in the margin. At rest the layer is blitted 1:1 at
  `−margin` with the nearest filter, so the picture is unchanged.
  `render::draw_underlay` takes the camera explicitly for this.
- Area and memory cost of the margin: (1 + 2·⅛·L/W)·(1 + 2·⅛·L/H) of the
  window. At 2560 × 1600 the margin is 320 px, the layer 3200 × 2240
  (1.75×): ≈ 143 MB for the 4× MSAA target and its resolved texture
  (≈ 82 MB before). At 1280 × 800: 160 px, 1600 × 1120.

## Alternatives considered

- **Timer thread calling `schedule_update`** — the candidate in the task
  note. Rejected: on X11 it does not wake a sleeping loop and can make
  the loop's `try_lock().unwrap()` panic (see *Context*).
- **Turning off `blocking_event_loop` during a gesture** — a conf setting,
  fixed at start-up.
- **Defer only MSAA during a gesture** — the remaining cost is CPU
  (tessellation, vertex upload), not fill; it would still re-render.
- **Retained GPU buffers / world-space meshes** — out of scope (needs
  `unsafe` or a per-shape mesh cache, [[Rendering performance options]]
  U2 and option 5).
- **No margin** — every pan shows background at the edge until it
  settles. A larger margin costs memory and makes every full re-render
  cover more shapes.

## Consequences

- A pan or zoom event costs one blit and the overlay, independent of the
  document. The layer is blurry or pixelated (zoom) or cut at its edge
  (pans beyond the margin, zooming out) for up to `SETTLE` after the last
  event; beyond the layer the window shows the background colour.
- Every full or append re-render covers the margin too, so it can draw
  more shapes than before (≤ 1.75× the area).
- After a gesture, the loop spins for ≈ 100 ms (≈ 6 frames at 60 Hz),
  then sleeps; idle CPU stays ≈ 0.
- Without vsync the polling frames are not paced; they are still only
  blits and stop after `SETTLE`.

## Rollback plan

`shell::app`: make `plan_redraw` return `Full` for camera changes again
and set the margin to 0; `blit_rect`, `settle` and the wake-up then never
act. Record as a rollback ADR reverting this one.
