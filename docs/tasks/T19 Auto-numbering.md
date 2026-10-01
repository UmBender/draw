---
id: T19
title: Auto-numbering
status: done
wave: 9
branch: task/T19-numbering
depends_on: [T16, T17]
adrs: ["[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T19-1 Numbering counter and undo]]"]
feature: "[[Numbered nodes]]"
tutorial: "[[19 Auto-numbering and undoable counters]]"
tags: [task]
---

# T19 Auto-numbering

## Goal

Drawing trees and graphs is fast: with numbering on, every new circle or
square gets the next number written inside it (1, 2, 3, …).

## Spec

Counter rule: [[ADR-T19-1 Numbering counter and undo]] — the counter is
derived from label counts before and after each step.

- **AC-1** — With numbering on, `numbering::label_new` gives a `Rect` or
  `Ellipse` `label = Some(next_number)`; every other shape, or any shape
  with numbering off, is returned unchanged. The shape tool labels both the
  committed shape and its preview.
  *Tests:* `numbering::tests::new_circle_gets_next_number`,
  `numbering::tests::rect_gets_next_number`,
  `numbering::tests::line_gets_no_label`,
  `numbering::tests::numbering_off_gives_no_label`,
  `shape_tool::tests::preview_shows_next_number`,
  `shape_tool::tests::commit_labels_ellipse`,
  `editor::tests::numbered_shapes_count_up`,
  `editor::tests::numbering_off_leaves_counter`,
  `editor::tests::stray_click_keeps_counter`.
- **AC-2** — The counter starts at `FIRST_NUMBER` (1); `Shift+N` resets it
  to 1 and the next shape gets 1 again.
  *Tests:* `numbering::tests::starts_at_one`,
  `editor::tests::reset_restarts_at_one`.
- **AC-3** — Undo of a numbered shape rolls the counter back by one so the
  next shape reuses the number; redo moves it forward again. Counts, not
  presence, drive the rules (works after a reset with duplicate numbers).
  Undo/redo of unnumbered changes leave the counter alone.
  *Tests:* `numbering::tests::advance_when_count_grows`,
  `numbering::tests::advance_saturates`,
  `numbering::tests::roll_back_when_count_shrinks`,
  `numbering::tests::roll_back_stops_at_first`,
  `numbering::tests::label_count_counts_matches`,
  `editor::tests::undo_rolls_counter_back`,
  `editor::tests::redo_restores_counter`,
  `editor::tests::undo_after_reset_rolls_back_duplicate`,
  `editor::tests::undo_of_unnumbered_keeps_counter`.
- **AC-4** — Copy/paste/duplicate keep labels as they are and do not move
  the counter.
  *Test:* `editor::tests::duplicate_keeps_labels_and_counter`.
- **AC-5** — Fuzz: labels are always `≥ 1` and only on Rect/Ellipse, the
  counter is always `≥ 1`. Covered by the existing T16 invariant in
  `tests/fuzz.rs` (random keys include `N`/`Shift+N`); T19 makes it exercise
  real labels. *Test:* `fuzz::editor_never_panics_and_keeps_invariants`
  (existing).

## Out of scope

Editing a label after creation, letters or custom start values, numbering
grid cells.

## Files owned

`src/core/numbering.rs`, `src/core/tools/shape_tool.rs`, `src/core/editor.rs`
(counter and undo hook only), new `ADR-T19-*`.

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

Step 19 — requires steps 6 and 16.

## Log

- 2026-10-01 — spec: AC-1…AC-5 with test names; ADR-T19-1 records the
  counter rule (label counts before/after a gesture end, undo or redo;
  `History` untouched).
- tests: `numbering` unit tests, shape tool `preview_shows_next_number`,
  `commit_labels_ellipse`, `commit_leaves_arrow_unlabelled`, editor tests
  for counting up, reset, undo/redo and clipboard.
- models: `label_count`, `advance`, `roll_back` signatures.
- behaviour: shape tool labels preview and commit via `label_new`; editor
  hooks in `pointer_up`, `Undo`, `Redo` (`advance_numbering`).
- quality: `scripts/check.sh` green with no changes, so no quality commit;
  `PROPTEST_CASES=20000 cargo test --release --test fuzz` green.
- AC-5 needed no new test: the T16 fuzz invariant (labels and counter ≥ 1,
  labels only on Rect/Ellipse) already runs with random `N`/`Shift+N`.
- Files touched: `src/core/numbering.rs`, `src/core/tools/shape_tool.rs`,
  `src/core/editor.rs`, plus this note, ADR-T19-1, the feature and tutorial
  notes and `Architecture.md` (module map row for `numbering`).
