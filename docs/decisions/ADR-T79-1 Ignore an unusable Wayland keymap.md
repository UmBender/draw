---
id: ADR-T79-1
title: Ignore an unusable Wayland keymap
status: accepted
kind: decision
date: 2026-11-08
task: T79
builds_on: ["[[ADR-T35-1 Vendored miniquad with a pen tablet patch]]", "[[ADR-T47-1 Flight recorder and capped session log]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T79-1 Ignore an unusable Wayland keymap

## Context

`keyboard_handle_keymap` in the vendored miniquad maps the event's `fd`
and asserts that `mmap` worked. A `no_keymap` event has size 0, so the
`mmap` fails and the assert panics inside a Wayland callback. A compositor
sends that event when it adopts a keyboard that has no keymap yet, which
anything creating and destroying virtual keyboards can cause
([[ADR-T75-1 Headless sway session for end-to-end tests]]).

The handler also ignores `format`, and stores a null keymap or state
without checking. Every later key, modifier or leave event then hands
that null to libxkbcommon. The same null is there before the first keymap
arrives, because both pointers start null.

## Decision

The handler treats a keymap as unusable when the format is not `XKB_V1`,
the size is 0, `mmap` fails, `xkb_keymap_new_from_string` returns null or
`xkb_state_new` returns null.

It builds the new keymap and state **before** it releases the old ones.
An unusable keymap changes nothing: draw keeps the last good pair. The
`fd` is closed and the mapping released on every path.

An ignored keymap is recorded as a new `BackendEvent::KeymapIgnored`
with its format and size, and logged at `info`. It is an expected event
on a seat with virtual keyboards, so it must not count as an anomaly: a
`WARN` line would also fail the e2e exit check.

While no keymap has ever arrived, key and modifier events return early,
key repeat does nothing, and leave still clears the focus but skips the
modifier reset.

Every change is marked `draw patch (ADR-T79-1)` and listed in
`vendor/miniquad/PATCHED.md`. No `unsafe` is added: the changes are in
the bodies of functions that are already `unsafe`.

## Alternatives considered

- **Clear the keymap on `no_keymap`** — what the event literally says,
  but keys would stop working until the next keymap, and every use of the
  pointers would then need to cope with a keymap going away mid-session.
- **Catch the panic at the callback** — unwinding across `extern "C"`
  aborts, and the file descriptor would still leak.
- **Log at `warn`** — fails the e2e exit check on every swap of a
  virtual keyboard, which is normal behaviour.

## Consequences

- A seat that swaps keyboards no longer takes draw down.
- Keys that arrive between a `no_keymap` and the next keymap translate
  with the previous keyboard's layout. The compositor sends no key from a
  keyboard without a keymap, so this is only the last good guess.
- The log shows `keymap ignored format F size S`, which tells the
  `no_keymap` case (format 0) from a failed parse (format 1, size above 0).

## Rollback plan

Remove the code marked `draw patch (ADR-T79-1)`, the `KeymapIgnored`
variant and the regression case in a `revert(T79)` commit under a new ADR
with `kind: rollback`.
