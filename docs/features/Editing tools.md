---
title: Editing tools
task: "[[T10 Editing tools]]"
adrs: ["[[ADR-0004 Vector object model]]", "[[ADR-0005 Undo via transaction log]]", "[[ADR-T08-1 Tool context and gesture overlay]]", "[[ADR-T91-1 Resize handles and scale rules]]", "[[ADR-T91-2 Scaled text stays within the largest text size]]", "[[ADR-T100-1 Z-order keys and restack transactions]]", "[[ADR-T101-1 Align and distribute keys]]", "[[ADR-T107-1 Lock flag kept by the document]]", "[[ADR-T107-2 Ctrl+K locks and Ctrl+Shift+K unlocks]]"]
tutorial: "[[10 Selection, clipboard and fill]]"
shortcuts: ["E", "B", "V", "Ctrl+C", "Ctrl+X", "Ctrl+V", "Ctrl+D", "Ctrl+A", "Delete", "Backspace", "Alt+[", "Alt+]", "Ctrl+[", "Ctrl+]", "Ctrl+Shift+[", "Ctrl+Shift+]", "Alt+←", "Alt+→", "Alt+↑", "Alt+↓", "Alt+H", "Alt+V", "Ctrl+K", "Ctrl+Shift+K"]
tags: [feature]
---

# Editing tools

## What it does

Changes what is already on the canvas. The eraser removes whole shapes it
touches, the bucket fills a rectangle or ellipse with the current colour, and
the select tool picks, rubber-bands, moves, resizes and duplicates shapes. An internal
clipboard copies, cuts, pastes (at the cursor) and duplicates the selection.
Locking a shape keeps a background grid or diagram from being selected,
moved, resized or erased by accident, while it still takes fills and text.
Every change is a single undo step.

## How to use

Keys are bound by [[T11 Keymap and macros]] (see [[Keymap]]); this table lists
what the tools do with them.

| Action | Shortcut / gesture |
|--------|--------------------|
| Eraser tool | `E`, then drag over shapes |
| Erase from any tool | right drag |
| Bucket tool | `B`, then click inside a rectangle or ellipse |
| Select tool | `V` |
| Select the topmost shape under the pointer | click |
| Add / remove a shape from the selection | `Shift` + click |
| Clear the selection | click on empty space (or `Esc`) |
| Rubber-band select | drag on empty space (`Shift` adds to the selection) |
| Move the selection | drag a shape (an unselected shape is selected first) |
| Duplicate while moving | `Alt` + drag |
| Resize about the opposite corner | drag a corner handle (`Shift` keeps the proportions) |
| Grow / shrink by 10 % about the centre | `Alt+]` / `Alt+[` |
| Raise / lower one step, past the nearest overlapping shape | `Ctrl+]` / `Ctrl+[` |
| Bring to the top / send to the bottom | `Ctrl+Shift+]` / `Ctrl+Shift+[` |
| Align left / right / top / bottom edges (two or more shapes) | `Alt+←` / `Alt+→` / `Alt+↑` / `Alt+↓` |
| Lay out as a row / column, equal gaps, no overlap (two or more) | `Alt+H` / `Alt+V` |
| Select all | `Ctrl+A` |
| Delete selection | `Delete`, `Backspace` |
| Copy / cut / paste at cursor | `Ctrl+C` / `Ctrl+X` / `Ctrl+V` |
| Duplicate selection (16 px right and down) | `Ctrl+D` |
| Lock the selection (and deselect it) | `Ctrl+K` |
| Unlock every shape | `Ctrl+Shift+K` |

## How it works

- Tools follow the API of [[ADR-T08-1 Tool context and gesture overlay]]:
  a per-tool `State`, `on_pointer`, `preview -> Overlay` and `cancel`. Pointer
  positions are screen pixels; tolerances are screen pixels converted with
  `Camera::world_len`, so they feel the same at every zoom.
- `src/core/tools/eraser.rs` — marks every shape `Shape::hit` within
  `ERASER_TOLERANCE_PX` (6 px) of the pointer path. The path between two
  events is sampled at most one tolerance apart (capped at 1024 samples), so a
  fast flick does not jump over a thin line. Marked ids go into
  `Overlay::hidden`; the release commits one `tx_remove`.
- `src/core/tools/bucket.rs` — on press, `Document::topmost_where` finds the
  topmost closed shape that `contains` the point, and one `tx_replace` sets its
  fill. Same colour, open shapes and empty space record nothing.
- `src/core/tools/select.rs` — a small gesture enum: `Marquee` (started on
  empty space) or `Move` (started on a shape, `Alt` captured at press). A
  release closer than `CLICK_SLOP_PX` (3 px) is a click. The move preview shows
  translated clones and hides the originals (not for `Alt`); the release
  commits one transaction of `Replace` edits, or inserts the copies. A press
  on a corner handle starts a `Resize`, whose pivot, dragged corner and limits
  are fixed at the press; its preview is scaled clones in the overlay and the
  document changes on release
  ([[ADR-T91-1 Resize handles and scale rules]]).
- `Shape::scaled_about` scales points and box corners and the size of text,
  taking the smaller factor for text
  ([[ADR-T91-2 Scaled text stays within the largest text size]]). Stroke
  widths and arrow heads stay as they are.
- `src/core/clipboard.rs` — `copy` clones the selection in z-order; `paste`
  centres the copies' joint bounds on the cursor; `duplicate` copies the
  selection without touching the clipboard. `insert_copies` (also used by the
  `Alt` move) inserts with fresh ids in one step and selects them.
- Z-order ([[ADR-T100-1 Z-order keys and restack transactions]]): the
  selected shapes move as one block and keep their order. One step up
  lands the block right above the lowest unselected shape that is above
  the topmost selected one and whose bounds meet the selection's box;
  one step down mirrors it. `document::tx_restack` removes the block top
  down and inserts it bottom up, so undo restores the order exactly. A
  press that would not change the order, an empty selection or a
  gesture in progress records nothing.
- Align and distribute ([[ADR-T101-1 Align and distribute keys]],
  [[ADR-T101-2 Rows and columns without overlap]],
  [[ADR-T101-3 Compact rows and columns]]):
  `select::align_selection` and `select::distribute_selection` measure
  each selected shape by its `Shape::bounds()` (a grid with its index
  bands, a text block by its layout box) and translate it whole. Align
  moves each box to the chosen side of the selection's union box.
  Distribute sorts by the near edge (then far edge, then stacking
  order), keeps the first shape still, centres the others on its line
  across the axis and places each one equal gap after the previous one.
  The gap is always a quarter of the largest size along the axis, so
  every press gives a compact row: overlapping shapes are pushed apart
  and spread-out ones pulled together. The moves are one
  transaction of `Replace` edits, so ids, stacking and the selection
  stay. Nothing is recorded with too few shapes, when every offset is
  within `1e-5` of the box's magnitude, when a moved shape would not be
  finite, or during a gesture.
- Selection hygiene: the editor prunes the selection after undo, redo and every
  gesture (T08); every select and clipboard entry point also drops stale ids
  first, so a stale id can never be moved, copied or deleted.
- Locks ([[ADR-T107-1 Lock flag kept by the document]]): the `Document`
  keeps a set of locked ids, changed only by the reversible
  `Edit::Lock { id, locked }`, so `Ctrl+K` (`tx_lock` on the selection)
  and `Ctrl+Shift+K` (`tx_unlock_all`) are one undo step each and record
  nothing when there is nothing to do. `Document::apply` refuses to
  remove a locked shape, and `tx_remove`, `tx_clear` and `tx_restack`
  skip locked shapes. Click (`Document::topmost_unlocked_where`), marquee
  and `Ctrl+A` never pick a locked shape and `select::prune` drops locked
  ids, so every selection command (delete, cut, copy, move, resize,
  scale, align, distribute, restack, grid arrows) leaves it alone. The
  eraser ignores locked shapes in both modes. The bucket and the text
  tool still use `topmost_where`, so a locked grid keeps taking fills
  and cell text; snapping still uses every shape. Locked shapes look the
  same as the others.

## Limits

- No system clipboard: copy/paste only works inside one running instance.
- No rotation and no edge-only handles. A resize does not snap.
- A resize stops at 4 world units per shape, and the smallest selected shape
  stops the whole selection; a shape cannot be flipped.
- Handles show and work only in the Select tool.
- A one-step raise or lower only counts shapes that overlap the
  selection's whole box; with a spread-out selection a shape in a gap
  of the box counts too. Grid cells and text inside shapes have no
  order of their own.
- In text edit mode `Ctrl+]` / `Ctrl+[` resize the text, not the order.
- Align and distribute work only on the selection's own box: there is no
  aligning to the canvas or to a key shape, and no toolbar button. Graph
  edges are separate shapes, so they stay put when their nodes move.
- Distribute uses the order of the near edges before the move, and
  always centres on the first shape's line; to line up on another shape,
  align with `Alt+arrow` first.
- The eraser removes whole shapes, never parts of a stroke.
- The bucket fills only rectangles and ellipses, not regions enclosed by
  strokes or lines.
- A press-release under 3 px is a click, so moves smaller than that are not
  possible with the mouse.
- Locked shapes have no visible mark, and unlocking is all at once:
  there is no way to unlock one shape. `Ctrl+Backspace` clears only the
  unlocked shapes, and emptying a locked text block keeps its old text.
  The eraser cannot clear the fill of a locked shape; repaint it with the
  bucket instead.
