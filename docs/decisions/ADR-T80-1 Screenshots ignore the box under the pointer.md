---
id: ADR-T80-1
title: Screenshots ignore the box under the pointer
status: accepted
kind: decision
date: 2026-11-08
task: T80
builds_on: []
amends: ["[[ADR-T75-1 Headless sway session for end-to-end tests]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T80-1 Screenshots ignore the box under the pointer

## Context

The first nightly run of the end-to-end suite failed every case with a
screenshot, on the mouse cursor alone: the runner's capture (sway 1.9,
wlroots 0.17, grim 1.4) shows it, and the goldens made here (sway 1.11,
wlroots 0.19, grim 1.5) do not. `grim` runs without `-c` on both. The
runner's image will change again when `ubuntu-latest` moves on, so the
comparison cannot rely on either behaviour.

## Decision

Before comparing, `tests/e2e.rs` copies the golden's pixels over a box in
the screenshot: 26×26 pixels, starting 2 pixels above and to the left of
the pointer's last position in the case. The harness knows that position
from the case's own steps. `compare_images`, the tolerances and the
goldens do not change.

## Alternatives considered

- **Regenerate the goldens on the runner** — the failure moves to this
  machine.
- **Raise the pixel allowance to about 160** — hides real rendering
  changes anywhere in the window.
- **Move the pointer to a corner before the screenshot** — sends the app
  one more motion event, which can change what it draws.
- **Hide the cursor in sway** — `seat hide_cursor` works on a timeout,
  which the suite's no-sleeps rule excludes.

## Consequences

- The pixels under the pointer are not checked, in the 5 screenshot cases.
  Their state is still checked by the editor snapshot.
- A golden refreshed on a machine whose capture has the cursor stores it,
  and the comparison still ignores it.

## Rollback plan

Remove the copy in `compare_screenshot` in a `revert(T80)` commit under a
new ADR with `kind: rollback`, and pin the runner image instead.
