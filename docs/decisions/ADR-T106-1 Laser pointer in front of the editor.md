---
id: ADR-T106-1
title: Laser pointer in front of the editor
status: accepted
kind: decision
date: 2026-11-10
task: T106
builds_on: [ADR-0006, ADR-T24-2, ADR-T24-5, ADR-T74-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T106-1 Laser pointer in front of the editor

## Context

[[T106 Fading laser pointer]]: holding `Q` turns pointer moves into a
fading trail that never reaches the tools, the document or the history.
The trail needs time to fade, and `core` has no clock. The editor's held
keys (`Space`) are saved state: a snapshot taken while one is held keeps
it, and the fuzzer checks that a snapshot loaded mid-session continues
exactly like the original ([[ADR-T74-1 Line-based editor snapshot format]]). A `Q` flag
inside the editor would have to join the snapshot text, which means a
format version bump that makes every saved fixture, e2e golden and the
user's autosave unreadable, for a state nobody wants to restore.

## Decision

- A new pure module `core::laser` holds a `Laser`: whether `Q` is held
  and the trail, a bounded queue of screen-space points stamped with a
  time in seconds that the caller passes in.
- The shell routes every input event through `Laser::route` **before**
  the toolbar and the editor, telling it whether a text block is open and
  whether a gesture is in progress. `route` answers `Consumed` or
  `Forward`:
  - `Q` down with no `Ctrl` and no `Alt`, while no text block is open,
    starts holding (auto-repeat keeps holding); `Q` up stops it. Both are
    consumed. In text edit mode `Q` is forwarded, so the block types `q`.
  - While held and no gesture is in progress, pointer moves add a point
    and pointer presses and releases are swallowed. Every other event
    (keys, wheel, pressure, characters, resize, paste) is forwarded.
  - While held during a gesture, pointer events are forwarded, so the
    gesture keeps going; the trail starts with the first move after it
    ends, as a new piece not joined to older points.
- The editor never sees `Q` or the trail: no editor field, no snapshot
  change, no undo step, no document revision, and autosave sees no
  change. `Q` stays unbound in the keymap (like `Space`).
- The shell keeps a monotonic clock (`Instant` since start) and calls
  `Laser::tick(now)` every frame: it drops faded points and says whether
  the overlay must be redrawn. While `Laser::is_fading()` the shell asks
  for another frame (`schedule_update`); once the last point is gone it
  stops, so idle goes back to 0 % ([[ADR-0006 Redraw on demand]]). The
  trail is drawn last over the overlay; the document layer is reused
  (an `Overlay` redraw, [[ADR-T24-2 Overlay drawn straight to the window]]).

## Alternatives considered

- **A `laser_held` field in the editor** — routing would live with the
  rest of the routing, but the flag must then be saved (format bump,
  every autosave lost) or the snapshot continuation check fails as soon
  as a cut falls between `Q` down and `Q` up.
- **A clock in the editor (a `Tick` input event)** — puts time into the
  saved state and into every fuzz session for a purely visual effect.

## Consequences

- The laser is tested headless in `core::laser`, including against a
  real `Editor` behind it, and fuzzed in `tests/fuzz.rs` with `Q` presses
  and releases mixed into sessions.
- A pointer move swallowed by the laser does not update the editor's
  cursor; the next forwarded move does (paste at cursor may use the spot
  before the laser until then).
- Like `Space`, a `Q` release lost to a focus change leaves the laser
  held until `Q` is pressed and released again.

## Rollback plan

Remove `core::laser`, its routing call and drawing in `shell::app`, and
the `Q` row in the docs; record it in a new rollback ADR.
