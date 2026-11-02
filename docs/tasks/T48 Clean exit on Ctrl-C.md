---
id: T48
title: Clean exit on Ctrl-C
status: todo
wave: 32
branch: fix/clean-exit-on-sigint
depends_on: [T47]
adrs: []
feature: ""
tutorial: ""
tags: [task]
---

# T48 Clean exit on Ctrl-C

## Goal

Stopping draw with `Ctrl-C` in the terminal (`SIGINT`), or with
`SIGTERM`, ends the process cleanly with exit code 130 / 143 instead of a
segmentation fault.

## Known facts (2026-11-02)

- Owner's report: `Ctrl-C` in the terminal that started draw ends in a
  segfault.
- draw installs no signal handler; the only hook is the panic hook of
  ADR-T27-3, which does not see signals. `panic = "abort"` in release.
- `unsafe_code = "forbid"` in draw ([[ADR-0014 Minimal dependencies and
  no unsafe]]), so a handler cannot live in draw's crate itself; the
  vendored miniquad (which already has `unsafe` FFI) or a tiny
  dependency would host it.
- Not yet known: whether it happens on both backends (X11 and
  `DRAW_BACKEND=wayland`), in the Flatpak and in `target/release`, and
  where it crashes (likely the default `SIGINT` killing the process while
  a GL/EGL or Wayland thread is still running, or atexit handlers
  tearing down the GL driver under a live context).

## Proposal (settle in the spec step)

1. Reproduce on both backends and get a backtrace (`coredumpctl`,
   `gdb`, `addr2line` with `draw.debug`).
2. Catch `SIGINT`/`SIGTERM` (self-pipe or an atomic flag checked by the
   event loop) and turn them into the normal quit path
   (`miniquad::window::order_quit`), so the window and GL context are
   torn down in order. A second `Ctrl-C` exits at once.
3. Log the signal through [[T47 Always-on diagnostics log]], and on
   `SIGSEGV`/`SIGABRT` write the flight recorder before dying.
4. ADR on where the `unsafe` signal code lives.

## Spec

To be written once the crash site is known.

## Out of scope

- Saving the document on exit (draw has no file format).

## Files owned

- `src/main.rs`, `src/shell/app.rs`
- `vendor/miniquad/` (if the handler lives there, `PATCHED.md`)
- new ADR `ADR-T48-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T48): …`
- [ ] tests — `test(T48): …`
- [ ] models — `feat(T48): …`
- [ ] behaviour — `fix(T48): …`
- [ ] quality — `chore(T48): …`
- [ ] docs — `docs(T48): …`

## Learning path

Possibly a tutorial on async-signal-safe handlers.

## Log

- 2026-11-02 — note written from the owner's report; not reproduced yet.
