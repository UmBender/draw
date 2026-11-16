---
id: ADR-T177-1
title: X11 selections served from the poll loop, INCR both ways, quit signals by a flag
status: accepted
kind: decision
date: 2026-11-15
task: T177
builds_on: [ADR-T127-2, ADR-T142-1, ADR-T48-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T177-1 X11 selections served from the poll loop, INCR both ways, quit signals by a flag

## Context

`X11Platform` needs the clipboard and primary selections (text and
`image/png`) and the quit behaviour of [[ADR-T48-1 Signal handling in the vendored miniquad]]
without miniquad. `Platform::clipboard_text` is synchronous.

## Decision

- No thread: the owner of a selection answers `SelectionRequest` inside
  `poll`, and `clipboard_text` / `clipboard_png` / `primary_text` read by
  `XConvertSelection` and then pump the event queue for at most one second.
  Events that are not about the selection are stashed and replayed by the
  next `poll`, so no key or pointer event is lost.
- Targets served: `TARGETS`, `UTF8_STRING`, `STRING`, `TEXT`,
  `text/plain;charset=utf-8` for text; `image/png` for a PNG. Anything else
  is refused (property `None`). Setting text replaces a PNG and the other
  way round.
- A payload above 256 KiB is sent with `INCR` in 128 KiB chunks, driven by
  `PropertyNotify(Delete)` of the requestor's property; a zero-length chunk
  ends it. Incoming `INCR` is reassembled the same way and capped at
  64 MiB (more is dropped, the read returns `None`).
- The pure parts (event decoders, the `INCR` sender and receiver, the
  target tables) live in `x11::selection` without `unsafe`, and are fuzzed.
- `SIGINT` and `SIGTERM` set an atomic flag through `signal(2)` from glibc
  (declared in one `unsafe extern` block with a `TEST:` marker); `poll`
  turns the flag into `close_requested`, the same state a window-manager
  close sets, so the window and GL context are torn down in order.
  `x11::signal::received` returns the signal number so `main` (T178) can
  end the process by that signal as T48 does.

## Alternatives considered

- **A thread serving selections** — needs a second display connection and
  locking; the loop already polls every frame.
- **`xcb` for selections** — Xlib is the owner's choice (T177 note).

## Consequences

A selection is served only while the loop polls (as in the former
backend). A client that waits more than one second for us gets nothing.

## Rollback plan

Revert the T177 commits; the clipboard is then empty as after T176.
