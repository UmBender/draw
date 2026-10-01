---
title: The app loop, idle redraw and UI
step: 12
requires: ["[[07 Rendering with macroquad and culling]]", "[[08 An editor as an input-driven state machine]]", "[[09 Building drawing tools]]", "[[10 Selection, clipboard and fill]]", "[[11 Keymaps and gesture macros]]"]
feature: "[[Toolbar]]"
code: ["src/shell/app.rs", "src/shell/input_map.rs", "src/shell/toolbar.rs"]
tags: [tutorial]
---

# The app loop, idle redraw and UI

> **Learning path step 12.** Requires: steps 7–11 · Next: step 13 (fuzzing)

## Why this matters here

A contest lasts hours on a laptop. A typical game loop redraws 60+ times a
second even when nothing moves, which burns battery for nothing. And a drawing
app that drops pointer samples between frames gives jagged strokes. This
step wires the tested core to a real window without giving up either.

## The concept

**Event-driven, not frame-driven.** Instead of "every frame, poll the mouse",
the loop is "sleep until an event arrives, apply it, redraw if it changed
something":

```
            ┌── sleep (0 % CPU) ◀────────────────┐
 OS event ──▶ collect raw events in order        │
            ▶ route: toolbar? → Command          │
            │        canvas?  → Editor::handle   │
            ▶ dirty? → re-render into texture    │
            ▶ blit texture (1 quad) ─────────────┘
```

Two details make it work:

1. **Collect events, don't poll state.** Polled state (`mouse_position()`,
   `is_mouse_button_pressed()`) only shows the last value per frame. Ten
   motion samples between two frames become one, and a click shorter than a
   frame can vanish. An *event subscriber* replays every raw event in order.
2. **A woken frame must still produce an image.** macroquad clears the
   window at the start of every frame, so "do nothing when clean" would show
   a blank window. Rendering the scene into an off-screen texture when it is
   dirty, and blitting that texture otherwise, keeps clean frames almost free.

**UI as pure functions.** The toolbar is a function from the viewport size to
a list of rectangles, plus a hit-test from a point to a command. Neither
touches the GPU, so both are unit-tested like any other core code. Only the
final `draw` call is untested glue.

## How draw implements it

- `shell::app::window_conf` returns `WindowSettings`; its `From` impl builds
  macroquad's `Conf` with `blocking_event_loop` and every `update_on` trigger,
  so pointer, wheel and key events wake the loop.
- `shell::input_map::Collector` implements miniquad's `EventHandler`.
  `run` calls `repeat_all_miniquad_input(&mut collector, subscriber)` each
  frame and drains `InputEvent`s. Positions are divided by the DPI scale
  (`to_logical`); pointer events get the modifiers of the modifier keys held
  (`HeldMods`, a bitmask so releasing left `Ctrl` keeps right `Ctrl` held).
- `shell::toolbar::route` sends a press on the visible strip to
  `hit` (→ `Route::Apply(command)`) or drops it (`Route::Swallow`); everything
  else is `Route::Forward`.
- `CachedFrame::present` (in `app.rs`) recreates the render target when the
  framebuffer size changes and calls `draw_scene` only when dirty. The blit
  uses `flip_y` because GL textures start at the bottom row.

## Try it

1. Run `cargo run --release`, leave it idle, and watch `top -p $(pgrep -x draw)`:
   CPU stays at 0 %.
2. In `input_map.rs`, make `mouse_motion_event` push only every second
   sample. Draw a fast circle and compare the shape. Revert.
3. Add a test that `route` forwards a `Scroll` over the toolbar, then change
   `route` to swallow it and watch the test fail.

## Further reading

- miniquad `conf::Platform::blocking_event_loop` and `window::schedule_update`.
- "Immediate-mode vs retained-mode GUI" — the toolbar is retained layout,
  immediate drawing.
