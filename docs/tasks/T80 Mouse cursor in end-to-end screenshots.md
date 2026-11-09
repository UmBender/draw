---
id: T80
title: Mouse cursor in end-to-end screenshots
status: todo
wave: 35
branch: fix/e2e-cursor-in-screenshots
depends_on: [T75]
adrs: []
feature: "[[End-to-end tests]]"
tutorial: ""
tags: [task, testing]
---

# T80 Mouse cursor in end-to-end screenshots

## Goal

The end-to-end screenshot comparison gives the same answer whether or not
the compositor's capture includes the mouse cursor, so the suite passes on
this machine and on the nightly runner with one set of goldens.

## Known facts (2026-11-08)

- **Symptom:** nightly run 42541491862, job `end-to-end`, failed the 5
  cases with screenshots: 149–151 pixels off by more than 8 (allowed: 64).
  All 10 editor snapshots matched.
- **Every differing pixel is the cursor:** a box of at most 14×21 at the
  case's last pointer position, starting one pixel above it. The golden
  shows no cursor there and the runner's screenshot does.
- **Versions:** the runner has sway 1.9, wlroots 0.17 and grim 1.4; this
  machine has sway 1.11, wlroots 0.19 and grim 1.5. Which one draws the
  cursor into a capture is probably down to that version gap; this has
  not been proven. `ubuntu-latest` is due to move to Ubuntu 26, so the
  fix must not depend on either behaviour.
- `grim` runs without `-c`, so neither machine asks for the cursor.

## Proposal (settle in the spec step)

`compare_images` ignores a small box around the last pointer position.
Goldens do not change. Rejected: regenerating the goldens on the runner
(the failure moves to this machine) and raising the pixel allowance to
about 160 (hides real rendering changes).

## Spec

Finalised in the spec step.

## Out of scope

- Other differences between Mesa or sway versions.
- Changing the input the cases send, or the goldens.

## Files owned

- `tests/e2e.rs`
- `docs/features/End-to-end tests.md`
- `docs/decisions/` (the new ADR)

## Subtasks (one commit each)

- [ ] spec — `docs(T80): specify the cursor box in screenshot comparison`
- [ ] tests — `test(T80): add failing tests for the cursor box`
- [ ] models — `feat(T80): add the cursor box types`
- [ ] behaviour — `fix(T80): ignore the mouse cursor in screenshot comparison`
- [ ] quality — `chore(T80): pass clippy and rustfmt`
- [ ] docs — `docs(T80): record the fix`

## Learning path

None planned.

## Log

- 2026-11-08 — created from the first nightly run of the e2e job.
