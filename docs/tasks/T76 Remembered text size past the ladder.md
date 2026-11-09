---
id: T76
title: Remembered text size past the ladder
status: done
wave: 34
branch: fix/remembered-text-size
depends_on: [T74]
adrs: [ADR-T76-1]
feature: "[[Text blocks]]"
tutorial: ""
tags: [task]
---

# T76 Remembered text size past the ladder

## Goal

The text size draw remembers for new blocks always stays on the size
ladder, whatever the zoom was when the edit ended. Every snapshot draw
writes can be read back, and `scripts/check.sh --fuzz` stops failing at
random on `text size out of range`.

## Known facts (2026-11-04)

- **Symptom:** `snapshot_round_trip_continues_equal` and its pen-tap and
  shape-text variants fail with `own text rejected: line N: text size out
  of range`. Found by the `--fuzz` runs of
  [[T71 Combinatorics notation]] and [[T67 Algebra and group theory symbols]],
  no formula involved. It only shows when the random inputs hit it, so a
  green `--fuzz` run does not mean it is gone.
- **Reproduced on `main` at `baa7ca5`:** the T71 seed fails both
  `snapshot_round_trip_continues_equal` and
  `pen_tap_snapshot_round_trip_continues_equal`.
- **Seed:** `stash@{0}`, `batch-T71-2026-11-04 fuzz seed: snapshot text
  size out of range` (one line in `tests/fuzz.proptest-regressions`).
  T67's seed was deleted, so this is the only copy.
- **Cause:** `Editor::finish_text` (`src/core/editor.rs`) remembers
  `edit.size * camera.zoom()` when an edit was resized
  ([[ADR-T37-1 Remember the last text size]]). `TextEdit::step_size`
  only leaves `size` on a ladder rung at the zoom of the key press, so
  zooming between `Ctrl+[` and the end of the edit multiplies the size.
  `Ctrl+[` to 16 px at zoom 1, then zooming to `ZOOM_MAX` (20), remembers
  320 px. The reader in `src/core/tools/text.rs` refuses anything above
  `MAX_TEXT_SIZE_PX` (76.76).
- **The reader is right.** `MAX_TEXT_SIZE_PX` says what the editor should
  be able to set; the bug is the value written. It is also visible without
  snapshots: the next block starts at 320 px on screen, which is what
  ADR-T37-1 set out to avoid ("unreadably small or huge"). The same path
  goes low: a block resized at zoom 1 and finished at `ZOOM_MIN` remembers
  a size below the smallest rung, which the reader accepts.

## Proposal (settle in the spec step)

Clamp the remembered on-screen size in `finish_text` to the ladder's
range, `text_size_px(MIN_WIDTH_PX)`..=`text_size_px(MAX_WIDTH_PX)`. The
block itself keeps its world size; only the memory is clamped. Clamping
both ends amends ADR-T37-1, so it needs a new ADR.

Rejected: raising the reader's limit to `MAX_TEXT_SIZE × ZOOM_MAX`. The
snapshot would load, but draw would keep starting new blocks at a huge
size.

## Spec

Settled by [[ADR-T76-1 Remembered text size stays on the ladder]]: the
remembered on-screen size is clamped to
`text_size_px(MIN_WIDTH_PX)`..=`text_size_px(MAX_WIDTH_PX)` in
`Editor::finish_text`, at both ends. The block keeps its world size.

- **AC-1** — Resizing a block, zooming to `ZOOM_MAX` and ending the edit
  remembers at most the top rung, which never exceeds the reader's
  `MAX_TEXT_SIZE_PX`, and the snapshot taken then parses and round-trips.
  The block keeps its world size. *Test:*
  `core::editor::tests::regression_t76_remembered_size_past_the_ladder`,
  from the snapshot `regressions/T76-remembered-size-past-the-ladder`. The
  name is lower-case because `regression_T76_…` fails `non_snake_case`
  under `-D warnings`.
- **AC-2** — The same at `ZOOM_MIN`: the memory is at least the lowest
  rung. *Test:* `core::editor::tests::text_size_memory_stays_on_the_ladder_at_min_zoom`
- **AC-3** — A remembered size within the ladder's range is unchanged by
  the fix, so the next block keeps its on-screen size across zoom. *Tests:*
  the existing `core::editor::tests::text_size_memory_keeps_on_screen_size_across_zoom`,
  which ends the edit at zoom 1, and
  `core::editor::tests::text_size_memory_inside_the_ladder_is_not_clamped`,
  which ends it at another zoom.
- **AC-4** — The reader still rejects a remembered size above
  `MAX_TEXT_SIZE_PX`. *Test:* `core::tools::text::tests::read_snapshot_rejects_size_past_the_ladder`
- **AC-5** — The T71 seed is committed in `tests/fuzz.proptest-regressions`
  and `scripts/check.sh --fuzz` is green with it. *Test:* the
  `tests/fuzz.rs` snapshot round-trip properties.

## Out of scope

- The size of the block being edited, or of blocks already in the
  document.
- Snapshots already saved with a size past the limit: they stay rejected,
  with no migration.
- The `blit_rect_maps_corners_through_cameras` flake noted in
  [[T57 Blackboard bold]]: a separate bug.

## Files owned

- `src/core/editor.rs`
- `src/core/tools/text.rs` (test only)
- `tests/fuzz.proptest-regressions`
- `tests/snapshots/regressions/T76-remembered-size-past-the-ladder.snap`
- `docs/decisions/` (the new ADR)

## Subtasks (one commit each)

- [x] spec — `docs(T76): specify the remembered text size clamp`
- [x] regression — `test(T76): add regression test for the remembered text size`
- [x] behaviour — `fix(T76): keep the remembered text size on the ladder`
- [x] quality — `chore(T76): pass clippy and rustfmt`
- [x] docs — `docs(T76): record the clamp and close the task`

## Learning path

None planned; the ADR is enough.

## Log

- 2026-11-04 — created from the `--fuzz` failures of T67 and T71. Apply
  the seed from `stash@{0}` in the regression step, and drop the stash
  only once it is committed.
- 2026-11-08 — spec settled. There are no new types, so the models step
  produces nothing and is skipped.
- 2026-11-08 — regression: the snapshot was written from a test replaying
  the real events. Before the fix the memory reached 320 px at `ZOOM_MAX`
  and 1 px at `ZOOM_MIN`, and the T71 seed failed both snapshot round-trip
  properties with `text size out of range`. All pass after the fix. The
  seed is committed; its stash is left for the owner to drop.
- 2026-11-08 — AC-3 and AC-4 pass before and after the fix: they pin
  behaviour that must not change. No feature or tutorial note: the learning
  path plans none, and the ADR records the rule.
- 2026-11-08 — **separate bug found, not fixed here.** The first
  `scripts/check.sh --fuzz` failed the three `*undo_all_then_redo_all_is_symmetric`
  properties: a long formula session leaves `ink Glyph { ch: 'a', pos: (250.8, 1.0) }`
  outside the block's box, `max.y` 22.5 (`tests/fuzz.rs:749`, the T30 ink
  invariant). It fails the same with the clamp removed, so T76 does not
  cause it, and the failing code is outside *Files owned*. Seed:
  `cc 1bf62a4f742068dd71bc2aa699366205a90af4d8b775a11fcd964bfa8086e409`.
  It needs its own bug-fix task.
- 2026-11-08 — UNATTENDED CHOICE: that seed is not committed, because it
  would keep the gates red until its bug is fixed. The whole line is in the
  stash `batch-T76-2026-11-08 fuzz seed: text ink outside its box`, as T71
  did. The stash numbers moved by one: the T71 seed is no longer
  `stash@{0}`, so find it by its message before dropping it.
- 2026-11-08 — AC-5: with the T71 seed committed, `scripts/check.sh --fuzz`
  then passed every gate and 20000 cases. The ink bug comes from random
  input, so a later `--fuzz` run can still hit it. Left for the owner:
  `Ctrl+[` a block, zoom far in, end the edit, start a new block: it starts
  at the top rung at most.
- 2026-11-08 — integrated. Still open for the owner: the hand check
  above, and dropping the T71 seed stash (find it by its message). The
  ink bug is now [[T78 Formula ink outside its text box]].
