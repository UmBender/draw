---
id: T19
title: Auto-numbering
status: ready
wave: 9
branch: task/T19-numbering
depends_on: [T16, T17]
adrs: ["[[ADR-T16-1 Grid shape and shape labels]]"]
feature: "[[Numbered nodes]]"
tutorial: "[[19 Auto-numbering and undoable counters]]"
tags: [task]
---

# T19 Auto-numbering

## Goal

Drawing trees and graphs is fast: with numbering on, every new circle or
square gets the next number written inside it (1, 2, 3, …).

## Spec

To be refined at the spec step; starting point:

- **AC-1** — `N` toggles numbering. While on, each committed `Ellipse` or
  `Rect` gets `label = Some(counter)` and the counter increments; other shapes
  are unaffected. The preview shows the number that will be used.
  *Tests:* `numbering::tests::new_circle_gets_next_number`,
  `rect_gets_next_number`, `line_gets_no_label`, `preview_shows_next_number`.
- **AC-2** — Counter starts at 1; `Shift+N` resets it to 1.
  *Tests:* `numbering::tests::starts_at_one`, `reset_restarts_at_one`.
- **AC-3** — Undo of a numbered shape rolls the counter back so the next
  shape reuses the number; redo moves it forward again. The exact rule (e.g.
  counter derived from the history versus stored with the transaction) is
  decided at the spec step. *Tests:* `editor::tests::undo_rolls_counter_back`,
  `redo_restores_counter`.
- **AC-4** — Copy/paste/duplicate keep labels as they are (no renumbering).
  *Test:* `clipboard` behaviour via an editor-level test.
- **AC-5** — Fuzz: labels are always `≥ 1` and only on Rect/Ellipse.
  *Test:* `tests/fuzz.rs` invariant from T16.

ADR to write at the spec step if the counter/undo rule needs one
(`ADR-T19-1`).

## Out of scope

Editing a label after creation, letters or custom start values, numbering
grid cells.

## Files owned

`src/core/numbering.rs`, `src/core/tools/shape_tool.rs`, `src/core/editor.rs`
(counter and undo hook only), new `ADR-T19-*`.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 19 — requires steps 6 and 16.

## Log
