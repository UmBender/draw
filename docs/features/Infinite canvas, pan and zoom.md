---
title: Infinite canvas, pan and zoom
task: "[[T03 Camera]]"
adrs: ["[[ADR-0013 World-space widths and zoom limits]]", "[[ADR-0004 Vector object model]]", "[[ADR-T08-1 Tool context and gesture overlay]]", "[[ADR-T106-1 Laser pointer in front of the editor]]", "[[ADR-T106-2 Laser key, fade and look]]", "[[ADR-T110-1 Minimap geometry, routing and overview cache]]"]
tutorial: "[[03 Cameras - world space vs screen space]]"
shortcuts: ["Mouse wheel", "Middle drag", "Space + drag", "Space + click in the minimap", "H", "0", "F", "Hold Q"]
tags: [feature]
---

# Infinite canvas, pan and zoom

## What it does

The canvas has no edges: you can pan in any direction and zoom from 5 % to
2000 %. Zooming is anchored at the cursor, so whatever is under the mouse stays
under the mouse. One key resets the view, another frames all content.

> Camera model by [[T03 Camera]]; input wiring (wheel, drags, `0`, `F`) by
> [[T08 Editor core and input model]]. The keys themselves are bound by
> [[T11 Keymap and macros]]. The laser pointer is by
> [[T106 Fading laser pointer]].

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Zoom towards cursor | Mouse wheel (×1.15 per notch) |
| Pan | Middle drag, `Space` + drag, or `H` (hand tool) + drag |
| Reset view (origin, zoom 1) | `0` |
| Fit view to all content | `F` |
| Laser pointer (fading trail, draws nothing) | Hold `Q` and move the pen or mouse |
| See the whole drawing (minimap) | Hold `Space`, or pan with the middle button |
| Jump to a place | `Space` + click or drag in the minimap |

Bindings are defined in [[Keymap]]. Panning works from **any tool** and
never switches it: release the middle button or `Space` and you are back to
drawing. Releasing `Space` mid-drag does not stop the pan; releasing the
button does.

### Minimap

While `Space` is held, and while you pan with the middle button, a small
panel in the bottom-right corner shows every shape and a frame for the part
of the canvas on screen. The frame moves as you pan. With `Space` held,
click in the panel to centre the view on that point, or drag there to move
the view continuously; these presses never draw. The minimap goes away when
you release `Space` (or the button). It does not show for a hand-tool drag
or while a text block is open, where `Space` types a space
([[T110 Minimap]]).

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

### Laser pointer (`src/core/laser.rs`, `src/shell/app.rs`)

- The shell hands every event to `core::laser::Laser::route` before the
  toolbar and the editor ([[ADR-T106-1 Laser pointer in front of the editor]]).
  `Q` without `Ctrl`/`Alt` holds it; while held and no gesture is in
  progress, pointer moves add screen-space points and presses/releases
  are swallowed. A gesture already running keeps its events; the trail
  starts after it. In text edit mode `Q` goes on and types `q`.
- Points carry the time the shell passes in (a monotonic clock started at
  launch); each fades linearly over 1 s and is then dropped; at most 1024
  are kept ([[ADR-T106-2 Laser key, fade and look]]).
- The trail is drawn last, 6 px wide in `theme::RED`, over the cached
  document layer (an overlay-only redraw). The loop asks for frames only
  while a point is fading, then sleeps again.
- The editor never sees the laser: no document revision, no undo step, no
  snapshot field, no autosave trigger.

### Minimap (`src/core/minimap.rs`, `src/shell/minimap.rs`)

Decisions: [[ADR-T110-1 Minimap geometry, routing and overview cache]].

- `core::minimap::panel_rect` places a 200 × 150 px panel 12 px from the
  right and bottom edges; a viewport too small for it has no minimap.
- `Fit` maps a world box into a pixel rectangle: scaled uniformly to fit
  and centred. It computes in `f64` and clamps results to the target, so
  huge or zero-sized boxes stay finite. `Minimap` is the panel plus the
  `Fit` of the *world box* — the union of all shape bounds and the visible
  rectangle — into the panel's drawing area.
- `Editor::minimap` returns it only while it shows. `Editor::handle`
  returns `true` when it appears or goes, so a `Space` press that changes
  nothing else still redraws the overlay; a repeated `Space` does not.
- A left press on the panel with `Space` held starts `Gesture::Minimap`
  instead of a pan. Press, moves and release centre the view on
  `Minimap::world_at(pointer)`. The gesture keeps the world box of its
  press, so the minimap does not rescale under the pointer during the drag.
- The editor caches the content bounds per document revision and refreshes
  them only while the minimap shows.
- The shell draws a schematic of the shapes (1 px lines, fills, text blocks
  as bars, tiny shapes as dots) into its own render target, keyed by
  `OverviewKey`: document revision, shape count, content bounds and target
  size. The camera is not in the key. Each frame the overlay draws the
  panel, that texture where `Minimap::place` puts it, and the view frame.
  Panning changes only the placement and the frame.
## Pages (T116)

`Alt+1`…`Alt+9` show one of nine canvases, for example one per contest
problem. Each page has its own shapes, selection, view (pan and zoom),
undo history and auto-number counter; the tool, colour, width, helper
switches and the clipboard are shared, so a copy on page 1 pastes on
page 3. The page number shows top-right, small, only when it is not
page 1. Pages are created lazily (empty until visited). Switching cancels
a gesture, closes an open text edit like `Esc`, is never an undo step and
costs one re-render of the document layer. Export, snapping, numbering
and `Ctrl+Backspace` act on the current page. See [[ADR-T116-1 Nine pages swapped into the editor]].

## Limits

- The minimap is schematic: no text glyphs or labels, every line 1 px. Its
  size and corner are fixed, and it is not part of the PNG export.
- When the view is far from the content, the content shrinks to a small
  marker in the minimap.
- No rotation, no animated or inertial zoom/pan (out of scope for T03).
- Coordinates are `f32`: far from the origin (≳ 10⁶ world units) precision
  drops to visible jitter at high zoom. Not a problem for scratch diagrams.
- Zoom is limited to 5 %–2000 %.
