---
id: T10
title: Editing tools
status: ready
wave: 5
branch: task/T10-editing-tools
depends_on: [T08]
adrs: ["[[ADR-0004 Vector object model]]", "[[ADR-0005 Undo via transaction log]]"]
feature: "[[Editing tools]]"
tutorial: "[[10 Selection, clipboard and fill]]"
tags: [task]
---

# T10 Editing tools

## Goal

Eraser, bucket, select/move and the clipboard (copy, cut, paste, duplicate).

## Spec

- **AC-1** — Eraser: every shape hit (tolerance 6 px screen) during a drag is removed;
  the whole drag is one transaction; preview hides shapes marked for removal.
  *Tests:* `eraser::tests::*`.
- **AC-2** — Bucket: click sets `fill = current colour` on the topmost `Rect`/`Ellipse`
  that `contains` the point; same colour = no-op (no undo entry); empty space = no-op.
  *Tests:* `bucket::tests::*`.
- **AC-3** — Select: click selects the topmost hit shape (empty = clear); `Shift`-click
  toggles; drag on empty space = marquee (selects shapes whose bounds intersect).
  *Tests:* `select::tests::click_*`, `marquee_*`.
- **AC-4** — Move: drag starting on a selected shape moves the selection; preview while
  dragging; one `Replace` transaction on release. `Alt`-drag duplicates and moves the
  copies. *Tests:* `move_*`, `alt_drag_duplicates`.
- **AC-5** — `select_all`, `delete_selection` (one transaction). *Tests:* `select_all_*`, `delete_*`.
- **AC-6** — Clipboard: `copy` stores clones of selected shapes; `paste` inserts them
  centred at the cursor with new ids and selects them; `cut` = copy + delete;
  `duplicate` = paste at +16 px screen offset. Each is one undo step.
  *Tests:* `clipboard::tests::*`.
- **AC-7** — Selection never contains ids missing from the document (pruned after undo/redo).
  *Test:* `selection_pruned_after_undo`.

## Out of scope

System clipboard integration, resize/rotate handles.

## Files owned

`src/core/tools/eraser.rs`, `src/core/tools/bucket.rs`, `src/core/tools/select.rs`,
`src/core/clipboard.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 10 — requires step 8.

## Log
