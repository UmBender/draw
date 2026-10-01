---
title: Infinite canvas, pan and zoom
task: "[[T03 Camera]]"
adrs: ["[[ADR-0013 World-space widths and zoom limits]]", "[[ADR-0004 Vector object model]]", "[[ADR-T08-1 Tool context and gesture overlay]]"]
tutorial: "[[03 Cameras - world space vs screen space]]"
shortcuts: ["Mouse wheel", "Middle drag", "Space + drag", "H", "0", "F"]
tags: [feature]
---

# Infinite canvas, pan and zoom

## What it does

The canvas has no edges: you can pan in any direction and zoom from 5 % to
2000 %. Zooming is anchored at the cursor, so whatever is under the mouse stays
under the mouse. One key resets the view, another frames all content.

> Camera model by [[T03 Camera]]; input wiring (wheel, drags, `0`, `F`) by
> [[T08 Editor core and input model]]. The keys themselves are bound by
> [[T11 Keymap and macros]].

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Zoom towards cursor | Mouse wheel (×1.15 per notch) |
| Pan | Middle drag, `Space` + drag, or `H` (hand tool) + drag |
| Reset view (origin, zoom 1) | `0` |
| Fit view to all content | `F` |

Bindings are defined in [[Keymap]]. Panning works from **any tool** and
never switches it: release the middle button or `Space` and you are back to
drawing. Releasing `Space` mid-drag does not stop the pan; releasing the
button does.

## How it works

`src/core/camera.rs` holds `Camera`, the affine map between *world space*
(where shapes live, unbounded `f32` coordinates) and *screen space* (window
pixels, origin top-left):

```text
screen = (world − offset) · zoom        world = screen / zoom + offset
```

- `offset` is the world point at the screen's top-left corner; `zoom` is pixels
  per world unit, always in `[ZOOM_MIN, ZOOM_MAX] = [0.05, 20.0]`
  ([[ADR-0013 World-space widths and zoom limits]]). Both fields are private,
  so the invariant cannot be broken from outside.
- `pan_by_screen(delta)` moves content by exactly `delta` pixels
  (`offset −= delta / zoom`).
- `zoom_at(cursor, notches)` multiplies zoom by `ZOOM_STEP^notches`
  (`ZOOM_STEP = 1.15`), clamps it, then recomputes `offset` so the world point
  under the cursor is unchanged — even when the clamp kicks in.
- `fit(bounds, viewport, margin_px)` centres a world box and picks the largest
  zoom that shows it with a pixel margin; `reset()` returns to offset 0, zoom 1.
- `visible_world_rect(viewport)` gives the world box on screen (for culling in
  [[T07 Renderer]]); `world_len` / `screen_len` convert pixel tolerances and
  widths (hit-testing, smoothing epsilon, `width_px / zoom` for new strokes).
- Every mutator ignores non-finite input and refuses a result that would
  overflow, so camera state is always finite — an invariant the fuzzer
  ([[T13 Fuzz harness]]) checks.

### Input wiring (`src/core/editor.rs`, `src/core/tools/navigate.rs`)

- `Editor::handle(InputEvent::Scroll { pos, delta })` calls
  `navigate::zoom(camera, pos, delta)`; `delta` is wheel notches (positive
  zooms in). Non-finite positions or deltas are dropped.
- A `PointerDown` starts a **pan gesture** when the button is middle, or left
  with the hand tool or with `Space` held. `navigate::Pan::drag` pans by the
  pointer delta since the previous event, so content sticks to the pointer.
- Only the button that started a gesture ends it; other buttons are ignored
  while it runs ([[ADR-T08-1 Tool context and gesture overlay]]).
- `Command::ResetView` calls `Camera::reset`; `Command::FitView` fits the union
  of all shape bounds into the viewport (last `Resize`) with a 32 px margin
  (`FIT_MARGIN_PX`), or resets on an empty document.
- Every handler returns whether the camera actually changed, so the shell can
  skip redraws ([[ADR-0006 Redraw on demand]]).

## Limits

- No rotation, no animated or inertial zoom/pan (out of scope for T03).
- Coordinates are `f32`: far from the origin (≳ 10⁶ world units) precision
  drops to visible jitter at high zoom. Not a problem for scratch diagrams.
- Zoom is limited to 5 %–2000 %.
