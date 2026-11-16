---
id: ADR-T170-1
title: Wayland pen frames as raw events, clipboard served from writer threads
status: accepted
kind: decision
date: 2026-11-16
task: T170
builds_on: [ADR-T35-1, ADR-T40-1, ADR-T127-2, ADR-T169-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T170-1 Wayland pen frames as raw events, clipboard served from writer threads

## Context

The miniquad patches turned `zwp_tablet_tool_v2` events into pointer events
at each `frame`, and dropped `wl_pointer` input while a tool was in
proximity. The clipboard needs file descriptors (pipes) in both
directions, and a `wl_data_source` has to answer `send` events while the
single-threaded loop goes on.

## Decision

- **Pen:** the listeners only record into a pure `PenTool` (module
  `wayland/pen.rs`, no `unsafe`). Its `frame` returns the `RawEvent`s: a
  `Tablet` sample, then the pointer event it belongs to (`PointerMove` or
  `Button`), as the vendored code pushed a sample before each event. A tip
  change is a `Left` button only when the state changes (this is the tip
  bounce fix); `BTN_STYLUS`/`BTN_STYLUS2` are `Middle`/`Right` (the barrel
  fix). `RawEvent::Tablet` has no tilt or tool field: the tool kind and tilt
  are kept in `PenTool` for later and do not leave the backend.
- **Shadowing:** `Pens::shadowed` uses `pointer_shadowed_by_pen`; the
  pointer listeners drop motion and buttons while it is true (ADR-T40-1).
- **Clipboard read:** the offers' MIME types are collected on the offer
  proxy; `selection` keeps the current offer. `clipboard_text` makes a
  pipe, asks `receive`, closes its write end, flushes and reads with a
  deadline and a size limit. When the process owns the selection it
  answers from its own copy and never reads its own pipe.
- **Clipboard write:** a `wl_data_source` offers the text MIME types or
  `image/png`. Each `send` event moves the descriptor and a shared copy of
  the payload to a short-lived thread that writes it, so a large transfer
  or a slow reader never blocks the loop. `cancelled` drops the source.
- **Serial:** `set_selection` uses the last serial seen on the keyboard,
  the pointer or a tool.
- **Unsafe:** a `pipe2` declaration and `OwnedFd::from_raw_fd`, each with a
  `// TEST:` and `// SAFETY:` marker (ADR-T127-2).

## Alternatives considered

- **Writing inside the `send` callback** — a 10 MiB image to a slow reader
  stalls the frame loop.
- **Adding tilt and tool kind to `RawEvent::Tablet`** — the editor does not
  use them; a wider event is a later task.

## Consequences

- The pen and transfer decisions are tested without a compositor and
  fuzzed. Threads are the only concurrency in the backend, and they own
  only a descriptor and an `Arc`.

## Rollback plan

Revert the T170 commits; `WaylandPlatform` goes back to an empty
clipboard and no pen.
