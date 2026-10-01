---
id: ADR-T12-1
title: Blocking event loop with cached frame
status: accepted
kind: decision
date: 2026-10-01
task: T12
builds_on: [ADR-0006, ADR-0002]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T12-1 Blocking event loop with cached frame

## Context

[[ADR-0006 Redraw on demand]] left two options open: miniquad's blocking event
loop, or caching the scene in a render target. Reading macroquad 0.4 showed
that both are needed together:

- With `blocking_event_loop`, the process sleeps until an input event (or a
  resize) arrives, so idle CPU is ~0 %. But every woken frame still runs
  macroquad's `begin_frame`, which **clears the back buffer**. Skipping all
  drawing on a clean frame would show an empty window.
- Many woken frames change nothing (hovering, releasing a modifier), so
  redrawing every shape on them is wasted work.

Separately, macroquad's polled input (`mouse_position`, `is_mouse_button_pressed`)
keeps only the last state per frame: pointer samples between frames and clicks
shorter than a frame would be lost, which hurts pen strokes.

## Decision

- Window config uses `macroquad::conf::Conf` with
  `platform.blocking_event_loop = true` and every `update_on` trigger enabled
  (mouse motion, buttons, wheel, keys). Resizes wake the loop by themselves.
- The scene (background, shapes, overlay, selection, toolbar) is rendered into
  an off-screen render target the size of the framebuffer **only when dirty**
  (an input changed editor state, or the window size or DPI changed). Every
  frame then blits that texture with one quad.
- Input comes from a macroquad input subscriber
  (`input::utils::repeat_all_miniquad_input`) feeding `shell::input_map::Collector`,
  which yields every raw event in order.
- Linux backend: miniquad's default `X11Only`, i.e. **XWayland** on a Wayland
  session. Its native Wayland backend is newer and less tested; XWayland gives
  the same input fidelity for a single window.

## Alternatives considered

- **Blocking loop, redraw everything on each woken frame** — simplest, but
  re-tessellates every shape on each mouse move over an idle canvas.
- **Render target with a vsync loop** — cheap frames, but still wakes 60+
  times a second while idle.
- **Native Wayland backend** — revisit if XWayland misbehaves on the target.

## Consequences

- Idle: the process blocks in the event loop; no frames are produced.
- One extra full-screen texture in GPU memory, recreated on resize.
- Rendering into a texture needs `flip_y` when blitting (GL texture origin).

## Rollback plan

Only `shell::app` changes: drop the render target and draw directly every
frame (keeping the blocking loop), or turn `blocking_event_loop` off to fall
back to a vsync loop. Record the change as a new ADR.
