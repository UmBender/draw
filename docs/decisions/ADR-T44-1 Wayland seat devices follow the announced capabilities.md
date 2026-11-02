---
id: ADR-T44-1
title: Wayland seat devices follow the announced capabilities
status: accepted
kind: decision
date: 2026-11-02
task: T44
builds_on: ["[[ADR-T28-1 Opt-in native Wayland backend]]"]
amends: ["[[ADR-T35-1 Vendored miniquad with a pen tablet patch]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T44-1 Wayland seat devices follow the announced capabilities

## Context

The native Wayland backend of miniquad 0.4.11 asks the seat for a new
`wl_keyboard`, `wl_pointer` and `wl_touch`, each with the shared
listener, **every** time `wl_seat.capabilities` arrives. It never checks
for an existing device and never releases one. The protocol lets a
compositor send `capabilities` again whenever the seat's devices change.
After two re-sends, three keyboards feed one listener and every key
reaches draw three times: the owner saw `aaa` for `a`, and one
`Backspace` removed all three ([[T44 One keyboard per Wayland seat]]).

The protocol also says that a client must ask for a new device once a
capability comes back, so keeping the first device forever would not
work either.

## Decision

The vendored backend keeps at most one device per capability. On each
`capabilities` event it creates a device that is announced and missing,
releases one whose capability is gone, and keeps the rest. A pure helper,
`miniquad::seat::device_change`, makes that call and is tested from draw.
Releasing uses the `release` request on seats of version 3 and later
(`miniquad::seat::has_release_request`), and destroys the proxy only on
older ones. Whatever hangs off a released device goes with it: key
repeat for the keyboard, the cursor-shape device and pointer lock for the
pointer, touch points for touch.

This is a seat fix rather than a pen fix, but it is kept in the same
vendored copy, marked `draw patch (ADR-T35-1)` and listed in
`vendor/miniquad/PATCHED.md`. That widens ADR-T35-1 from "pen tablet
patch" to "pen tablet and Wayland input patches".

## Alternatives considered

- **Create once, never release** — fixes the duplicates but a keyboard
  or mouse that is re-plugged after its capability went away would never
  be read again.
- **Drop duplicate events in the shell** — the shell cannot tell a
  duplicate from a fast repeat, and the pointer buttons would still
  double.
- **Wait for upstream** — the bug is in the released 0.4.11 code; the
  copy is already vendored.

## Consequences

- Each key and button reaches draw once, however often the compositor
  re-announces the seat.
- The trigger seen by the owner was not reproduced, so if duplicated
  input comes back with this fix, the cause is elsewhere.
- One more change to re-apply when upgrading miniquad.

## Rollback plan

`git revert` the T44 commits in `vendor/miniquad` and `src/shell`, and
record the rollback as a new ADR that reverts this one.
