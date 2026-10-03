---
title: Reusing a frame during a gesture
step: 26
requires: ["[[12 The app loop, idle redraw and UI]]", "[[24 Layered redraws and measuring a frame]]"]
feature:
code: ["src/shell/app.rs", "src/shell/render.rs"]
tags: [tutorial]
---

# Reusing a frame during a gesture

> **Learning path step 26.** Requires: steps 12 and 24 · Next: —

## Why this matters here

After step 24, drawing on a 5 000-stroke page was smooth, but panning and
zooming were not. The camera is part of the document layer's cache key, so
every scroll notch or drag event re-rendered the whole layer: ≈ 15 ms
zoomed out, 40–60 ms at medium zoom on the target. Yet a pan or zoom does
not change *what* is drawn, only *where*. The picture we already have is
right, just moved and scaled.

## The concept

**A camera is an affine map.** `screen = (world − offset) · zoom`. Two
cameras differ by a translation and a uniform scale, so mapping a point
from the old screen to the new one is
`new.world_to_screen(old.screen_to_world(p))`. Map the two corners of the
cached image and you have the rectangle to draw it into:

```
 cached layer at camera A              window at camera B
┌───────────────────────┐            ┌───────────────────┐
│ margin                │   blit to  │   ┌───────────────┼──┐
│   ┌───────────────┐   │ ─────────▶ │   │ layer, scaled │  │
│   │ window at A   │   │  A→B rect  │   │ and moved     │  │
│   └───────────────┘   │            └───┼───────────────┘  │
└───────────────────────┘                └──────────────────┘
```

**Cheap now, exact later.** A scaled bitmap is blurry and the edge of the
image may show. That is fine while the view is moving, as long as the
exact picture comes back once it stops. "Once it stops" needs a clock:
remember when the camera last changed and re-render after a quiet period
(a *debounce*).

**Waking a sleeping loop.** The app sleeps until an input event arrives
(step 12), but the debounce needs a frame *without* input. The obvious
answer, a timer thread, does not work here: miniquad's X11 loop only reads
requests from other threads *between* events, so a request sent while it
sleeps waits for the next mouse move. Reading the library's source showed
it. The fix that works is to keep asking for one more frame from the main
thread while the picture is provisional. That costs a handful of cheap
frames (one blit each) per gesture, then the loop sleeps again.

**Only when it pays.** Reusing the frame trades sharpness for speed. On a
page with a few shapes a full re-render takes a millisecond, so the trade
is all loss: blur and missing edges for nothing. Decide from the measured
cost, not from a guess like "more than N shapes": the same shapes cost
several times more at one zoom than at another. The last full re-render's
time is a cheap, honest predictor of the next one.

**Margins.** Render a bit more than the window so short pans uncover real
content instead of background. The margin costs memory and makes every
full re-render cover more area, so it is a trade-off, not a free win.

## How draw implements it

All in `src/shell/app.rs`, as pure functions plus `CachedFrame::present`:

- `plan_redraw` returns `Redraw::Moved` when the cached and the new
  `LayerKey` differ *only* in the camera. Any other difference, with or
  without a camera change, is still `Full`.

  | Situation | Result |
  |-----------|--------|
  | No cache yet | `Full` |
  | Only appends, view unchanged | `Append { from }` |
  | Only the camera changed | `Moved` |
  | Any other key change | `Full` |
  | Same key, input changed something | `Overlay` |
  | Nothing changed | `None` |

- `reuse_if_slow(redraw, last_full)` turns `Moved` back into `Full` while
  the last full re-render took less than `REUSE_ABOVE` (8 ms) or was never
  measured. `present` times every `Full` re-render into `last_full`.
- `settle(redraw, quiet)` turns `Moved` into `Full` once the camera has
  been still for `SETTLE` (100 ms). `present` tracks `last_camera` and
  `changed_at` to compute `quiet`.
- On a `Moved` frame, `present` clears the window to the background,
  calls `miniquad::window::schedule_update()` so the loop runs again, and
  blits the layer to `blit_rect(cached camera, current camera, layer)` with
  the filter from `blit_filter` (linear while scaled, nearest at rest).
- `layer_margin(size)` is ⅛ of the longer window side in physical pixels,
  capped so the layer stays within `MAX_LAYER_SIDE`; `layer_size` adds it on
  both sides. The layer is rendered with `layer_camera`, the view camera
  panned by the margin, which is why `render::draw_underlay` now takes the
  camera instead of reading `editor.camera()`. At rest `blit_rect` returns
  the layer's own rectangle, `−margin` to `window + margin`, a 1:1 blit.

## Try it

1. `DRAW_FRAME_TIMES=1 ./target/release/draw`, build the 5 000-stroke scene
   (ten strokes, `Ctrl+A`, `Ctrl+D` ×9), zoom in and pan. You see a run of
   `moved` lines, then one `full` line about 100 ms after you stop.
2. Set `SETTLE` to 1 s. Zoom in one notch and watch the picture stay
   blurry for a second before it sharpens.
3. Make `layer_margin` return 0 and pan a little: the window edge shows
   background until the view settles.
4. Remove the `schedule_update()` call. Pan and stop: the picture stays
   blurry until you move the mouse, because nothing wakes the loop.
5. On a page with ten shapes, pan with `DRAW_FRAME_TIMES=1`: every line
   is `full`, because a re-render is far below `REUSE_ABOVE`. Set
   `REUSE_ABOVE` to `Duration::ZERO` and pan again to see the blur that
   was not worth it.
6. Replace the call by a thread that sleeps 100 ms and then calls
   `schedule_update()`. Does the picture sharpen without a mouse move?
   Read `miniquad`'s `linux_x11.rs` main loop to see why not.

## Further reading

- [[ADR-T26-1 Reuse the document layer during pan and zoom]] — the
  decision and the miniquad findings.
- [[ADR-T26-2 Gesture frame budget]],
  [[ADR-T26-3 Reuse the layer only when re-rendering is slow]].
- [[Rendering performance options]] — what is done and what is still open.
