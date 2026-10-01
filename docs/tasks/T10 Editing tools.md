---
id: T10
title: Editing tools
status: in-progress
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

Positions reach tools in screen pixels (ADR-T08-1); tolerances below are screen
pixels converted with `Camera::world_len` at gesture time. A press-release whose
pointer moved less than `select::CLICK_SLOP_PX` (3 px) is a **click**.

- **AC-1** — Eraser: every shape hit (`Shape::hit`, tolerance
  `eraser::ERASER_TOLERANCE_PX` = 6 px) during a drag is removed; points between
  two pointer events are sampled at most one tolerance apart so fast drags do not
  skip shapes. The whole drag is one transaction (empty drag = no undo entry);
  the preview lists marked shapes in `Overlay::hidden`; `cancel` drops the marks.
  *Tests:* `eraser::tests::down_on_shape_marks_it_hidden`,
  `drag_marks_every_shape_touched`, `fast_drag_samples_between_events`,
  `release_removes_marked_as_one_step`, `drag_over_nothing_records_no_step`,
  `cancel_drops_marks_and_keeps_document`, `tolerance_is_in_screen_pixels`.
- **AC-2** — Bucket: a press sets `fill = current colour` on the topmost
  `Rect`/`Ellipse` that `contains` the point (one undo step); same colour = no-op
  (no undo entry); empty space or open shapes = no-op.
  *Tests:* `bucket::tests::click_inside_rect_fills_with_current_color`,
  `click_inside_ellipse_fills`, `click_fills_topmost_closed_shape`,
  `same_color_is_noop`, `empty_space_is_noop`, `open_shape_is_not_filled`,
  `fill_is_undoable`.
- **AC-3** — Select: click selects the topmost hit shape (empty = clear);
  `Shift`-click toggles the hit shape; drag on empty space = marquee (world
  `Overlay::marquee` while dragging; on release selects shapes whose bounds
  intersect it, added to the selection with `Shift`).
  *Tests:* `select::tests::click_selects_topmost_shape`,
  `click_on_empty_clears_selection`, `click_on_selected_in_group_selects_only_it`,
  `click_shift_toggles_shape`, `marquee_selects_intersecting_shapes`,
  `marquee_preview_shows_world_rect`, `marquee_with_shift_adds_to_selection`.
- **AC-4** — Move: a drag starting on a shape moves the selection (a shape not
  yet selected becomes the selection first); while dragging the preview shows
  translated copies and hides the originals; release commits one `Replace`
  transaction. `Alt` held at press duplicates: the originals stay, translated
  copies are inserted (one undo step) and selected.
  *Tests:* `move_drag_translates_selection`, `move_is_one_undo_step`,
  `move_preview_shows_translated_and_hides_originals`,
  `move_drag_on_unselected_shape_moves_it`, `move_cancel_keeps_document`,
  `alt_drag_duplicates`.
- **AC-5** — `select_all` selects every shape (returns whether it changed);
  `delete_selection` removes the selection as one transaction and clears it.
  *Tests:* `select_all_selects_every_shape`, `select_all_twice_reports_no_change`,
  `delete_removes_selection_as_one_step`, `delete_with_empty_selection_is_noop`.
- **AC-6** — Clipboard (internal): `copy` stores clones of the selected shapes in
  z-order (empty selection keeps the clipboard); `paste` inserts them with their
  bounds centred at the cursor, with new ids, and selects them; `cut` = copy +
  delete; `duplicate` copies the *selection* (not the clipboard, which is left
  untouched) offset by `clipboard::DUPLICATE_OFFSET_PX` (16 px) right and down,
  and selects the copies. Each mutating command is one undo step.
  *Tests:* `clipboard::tests::copy_stores_selected_in_z_order`,
  `copy_with_empty_selection_keeps_clipboard`, `paste_centres_at_cursor`,
  `paste_assigns_new_ids_and_selects`, `paste_empty_clipboard_is_noop`,
  `paste_is_one_undo_step`, `cut_copies_and_deletes_in_one_step`,
  `duplicate_offsets_by_screen_pixels`, `duplicate_leaves_clipboard`.
- **AC-7** — Selection never contains ids missing from the document. The editor
  prunes after undo/redo (T08, `editor::tests::undo_prunes_selection`); in
  addition every select/clipboard entry point drops stale ids before acting, so
  a stale selection can never be moved, copied or deleted.
  *Test:* `select::tests::selection_pruned_after_undo`.

## Out of scope

System clipboard integration, resize/rotate handles.

## Files owned

`src/core/tools/eraser.rs`, `src/core/tools/bucket.rs`, `src/core/tools/select.rs`,
`src/core/clipboard.rs`

## Subtasks (one commit each)

- [x] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 10 — requires step 8.

## Log
