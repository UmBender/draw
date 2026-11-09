---
id: T80
title: Mouse cursor in end-to-end screenshots
status: done
wave: 35
branch: fix/e2e-cursor-in-screenshots
depends_on: [T75]
adrs: ["[[ADR-T80-1 Screenshots ignore the box under the pointer]]"]
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

## Spec

Before comparing, the golden's pixels are copied over a 26×26 box that
starts 2 pixels above and left of the pointer's last position
([[ADR-T80-1 Screenshots ignore the box under the pointer]]). All tests
are in `tests/e2e.rs`, `self_tests`.

- **AC-1** — A difference inside the box does not count. *Test:*
  `cursor_box_hides_a_difference_at_the_pointer`
- **AC-2** — A difference just outside the box still fails. *Test:*
  `cursor_box_keeps_a_difference_outside_it`
- **AC-3** — A box reaching past the image's edge is clipped, without a
  panic. *Test:* `cursor_box_at_the_image_edge_is_clipped`
- **AC-4** — The pointer's last position comes from the case's `set`
  steps, and is the start when there are none. *Tests:*
  `last_pointer_is_the_last_set_step`,
  `last_pointer_without_moves_is_the_start`
- **AC-5** — `scripts/e2e.sh` passes here with the goldens unchanged, and
  the screenshots of nightly run 42541491862 match them once the box is
  applied. *Test:* the runner, and a check of the saved artifacts, both
  in *Log*.
- **AC-6** — The nightly `end-to-end` job passes. *Test:* one manual run
  by the owner, which also closes AC-6 of T75.

## Out of scope

- Other differences between Mesa or sway versions.
- Changing the input the cases send, or the goldens.

## Files owned

- `tests/e2e.rs`
- `docs/features/End-to-end tests.md`
- `docs/decisions/` (the new ADR)

## Subtasks (one commit each)

- [x] spec — `docs(T80): specify the cursor box in screenshot comparison`
- [x] tests — `test(T80): add failing tests for the cursor box`
- [x] models — `feat(T80): add the cursor box types`
- [x] behaviour — `fix(T80): ignore the mouse cursor in screenshot comparison`
- [x] quality — `refactor(T80): move the screenshot call into the session`
- [x] docs — `docs(T80): record the fix`

## Learning path

None planned.

## Log

- 2026-11-08 — created from the first nightly run of the e2e job.
- 2026-11-08 — AC-1 to AC-4: the `self_tests` pass. They did not compile
  before the functions existed, and failed on the `todo!()` bodies.
- 2026-11-08 — AC-5: `scripts/e2e.sh` passes all 10 cases with the goldens
  unchanged. Applying the box to the 5 failing screenshots saved by run
  42541491862 leaves 0 pixels differing by more than 8 outside it.
  `scripts/check.sh` is green.
- 2026-11-08 — AC-6 pending owner run: the job needs one more nightly run
  on this fix, which also closes AC-6 of T75. No tutorial: none planned.
- 2026-11-08 — integrated. AC-6 waits on the nightly run from `main`.
- 2026-11-08 — AC-6 closed: nightly run on `529bab9` passed, with all 10
  cases in the `end-to-end` job. https://github.com/UmBender/draw/actions/runs/42543084405
