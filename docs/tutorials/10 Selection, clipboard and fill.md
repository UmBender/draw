---
title: Selection, clipboard and fill
step: 10
requires: ["[[08 An editor as an input-driven state machine]]"]
feature: "[[Editing tools]]"
code: ["src/core/tools/eraser.rs", "src/core/tools/bucket.rs", "src/core/tools/select.rs", "src/core/clipboard.rs"]
tags: [tutorial]
---

# Selection, clipboard and fill

> **Learning path step 10.** Requires: [[08 An editor as an input-driven state machine]]
> · Next: *11 Keymaps and macros*

## Why this matters here

Drawing tools only add shapes. Editing tools change shapes that already exist,
which raises three questions every editor has to answer: *which* shape did the
user mean, *what* do they see while dragging, and *how* does the change undo
as one step even though the pointer sent a hundred events?

## The concept

**Pick, preview, commit.**

1. **Pick** — hit-testing in world space with a tolerance given in *screen*
   pixels. A 6 px tolerance is `6 / zoom` world units, so a thin line is as
   easy to grab zoomed out as zoomed in. Shapes are searched top down, so the
   one drawn last wins.
2. **Preview** — while the button is held, nothing in the document changes.
   The tool only describes an *overlay*: shapes to draw on top, ids to hide, a
   marquee rectangle. Cancelling is then trivial: forget the gesture.
3. **Commit** — on release the tool builds one `Transaction` and hands it to
   the history. One gesture, one undo step.

```
 Down ──▶ pick ──▶ Move … Move ──▶ Up ──▶ one Transaction
           │        (overlay only)          (history.commit)
           └── nothing hit ──▶ marquee
```

**Click versus drag.** A real hand never releases exactly where it pressed.
A *slop* distance (3 px here) separates a click from a drag: under it the
gesture is a click (select this shape), over it a drag (move or marquee).

**Sampling a fast drag.** Pointer events arrive at the frame rate, not
continuously; a fast flick can be 80 px between events. Testing only the event
positions would skip a thin line in between, so the eraser walks the segment
between two events in steps no longer than its tolerance.

## How draw implements it

- `tools/select.rs` — `press` hit-tests with `Document::topmost_where` and
  `Shape::hit`, then arms `Gesture::Move` (on a shape) or `Gesture::Marquee`
  (on empty space). `drag_to` only updates `current`. `release` decides with
  `is_click`: a click selects only the clicked shape; a move builds one
  `Edit::Replace` per shape; `Alt` (remembered from the press) calls
  `clipboard::insert_copies` instead. `preview` turns the same state into an
  `Overlay`.
- `tools/eraser.rs` — `mark_along` lerps from the last position to the new one
  in `ceil(distance / 6 px)` steps and marks every shape hit; `preview` returns
  the marks as `hidden`; the `Up` phase commits `tx_remove` once.
- `tools/bucket.rs` — one press, one `tx_replace` with `Shape::with_fill`;
  filling with the colour already there records nothing, so undo never has
  empty steps.
- `clipboard.rs` — `paste` computes the union of the copies' bounds and moves
  its centre to `Camera::screen_to_world(cursor)`. `insert_copies` collects
  the new ids from the `Edit::Insert`s of the transaction *before* committing
  it, then selects them.

## Try it

1. Change `CLICK_SLOP_PX` in `tools/select.rs` to `0.0` and run
   `cargo test select`. Which tests still pass, and why does
   `click_on_selected_in_group_selects_only_it` depend on the slop being
   positive?
2. Set `MAX_SAMPLES` in `tools/eraser.rs` to `1` and run
   `cargo test fast_drag_samples_between_events`: with a single step the
   eraser only tests the two end points and misses the line.
3. Add a test that pastes twice and checks the second paste gets ids different
   from the first.

## Further reading

- [[ADR-T08-1 Tool context and gesture overlay]] — why tools return an overlay
  instead of mutating the document while dragging.
- [[ADR-0005 Undo via transaction log]] — why one gesture is one transaction.
