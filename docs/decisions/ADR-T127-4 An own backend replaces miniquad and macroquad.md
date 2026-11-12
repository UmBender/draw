---
id: ADR-T127-4
title: An own backend replaces miniquad and macroquad
status: proposed
kind: decision
date: 2026-11-12
task: T127
builds_on: [ADR-T127-1, ADR-T127-2]
amends: []
supersedes: [ADR-0002]
reverts: []
superseded_by:
reverted_by:
tags: [adr, 2.0, backend]
---

# ADR-T127-4 An own backend replaces miniquad and macroquad

## Context

[[ADR-0002 Rust and macroquad]] picked macroquad. Owner decisions,
2026-11-12: "**X11 stays.** ... The `x11` Flatpak socket, the X11 pen
pressure path and `DRAW_BACKEND` stay" and "**The patched miniquad goes; a
backend of our own replaces it.** miniquad (26.7k lines vendored, 19 of our
files use macroquad) is the base of macroquad, so **macroquad goes too**:
the window, input, clipboard, GL context and the renderer become ours."

The audit ([[2.0 baseline]]) confirms 26770 vendored lines, 544 `unsafe`
lines and 19 files that mention macroquad. What draw really uses is one
window, lines, rectangles, triangles, one mesh type, textures, a render
target, keyboard, pointer, pen and seat events.

## Decision

- Replace `vendor/miniquad` and macroquad with `draw-sys` (X11 and Wayland
  windows, keyboard, pointer, tablet, clipboard, EGL/GLX, all over libraries
  loaded with `dlopen`) and `draw-gfx` (a retained renderer over a `Gpu`
  trait).
- Build it next to macroquad and cut over only when it passes the same
  end-to-end suite on both backends (task T149); Wayland first, then X11.
- Both backends stay supported; X11 is not scheduled for removal (this
  reverses the deprecation of [[T46 Native Wayland by default and Xwayland deprecated]],
  done by T128).
- The vendored patches (pen pressure, seats, diagnostics) are ported with
  their tests, not dropped.

This supersedes the "macroquad" half of ADR-0002 (Rust, edition and the
target machine stay).

## Alternatives considered

- **Keep the patched miniquad** — the owner decided against carrying a
  26.7k-line fork.
- **winit or SDL** — new dependency trees against
  [[ADR-T127-3 The dependency list]].
- **Pure-Rust Wayland/X11 transport** — cannot hand EGL a display handle.

## Consequences

- Retained geometry becomes possible (no re-tessellation on pan or zoom).
- This is the largest item of the epic and the likeliest to meet platform
  quirks; Wayland and X11 are separate bodies of work.

## Rollback plan

Until T149 macroquad still builds: revert the new crates' wiring in `main`.
After it, revert the cutover commits with `git revert` and record a rollback
ADR.
