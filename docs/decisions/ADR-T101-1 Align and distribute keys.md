---
id: ADR-T101-1
title: Align and distribute keys
status: accepted
kind: decision
date: 2026-11-10
task: T101
builds_on: [ADR-T11-1, ADR-T29-2, ADR-T85-1, ADR-T91-1, ADR-T100-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T101-1 Align and distribute keys

## Context

Smart snap aligns shapes while they are drawn, but a selection that is
moved never snaps ([[Snapping#Limits]]): lining up a row of graph nodes
or spacing array cells evenly took many careful drags. The owner chose
`Alt+←/→/↑/↓` to align the selection to that side of its box and
`Alt+H` / `Alt+V` to space it with equal gaps, the outer shapes fixed.
Plain arrows resize the one selected grid under the Select tool
(ADR-T85-1); `Alt+]` / `Alt+[` scale the selection (ADR-T91-1); `Alt`
+ drag duplicates. `Alt+arrow`, `Alt+H` and `Alt+V` were unbound. The
shell passes `Alt` through for every key (it already delivers `Alt+]`),
so the chords reach the core on X11 and Wayland alike.

## Decision

- **Keys.** `Alt+←` → `Command::Align(Side::Left)`, `Alt+→` →
  `Align(Side::Right)`, `Alt+↑` → `Align(Side::Top)`, `Alt+↓` →
  `Align(Side::Bottom)`, `Alt+H` → `Command::Distribute(Axis::Horizontal)`,
  `Alt+V` → `Distribute(Axis::Vertical)`, with exact modifiers
  (ADR-T11-1). `Side` and `Axis` live in `core::command`. In text edit
  mode key presses never reach the keymap (ADR-T29-2), so `Alt+arrow`
  keeps moving the caret there.
- **Boxes.** Every shape moves as a whole by a translation, measured by
  its `Shape::bounds()` — the same boxes whose union the Select tool
  outlines. A grid (with its index bands) is one shape; a text block
  moves by its layout box (ADR-T29-1).
- **Align.** The selection box is the union of the selected shapes'
  bounds. `Left` moves each shape so its left edge meets the box's left
  edge; `Right`, `Top` (smaller `y`, the world is `y`-down) and `Bottom`
  likewise. Needs two or more selected shapes.
- **Distribute.** Needs three or more. The shapes are sorted by their
  left edge (top edge for `Vertical`), ties by right (bottom) edge, then
  by stacking order. The first and last of that order stay put; the gap
  is `(last.min − first.max − Σ middle widths) / (n − 1)`, and each
  middle shape is placed `gap` after the previous one's far edge. The
  gap may be negative when the shapes overlap. The other axis is not
  touched.
- **Transaction.** One `Replace` edit per shape that moves, in one
  transaction: one undo step, ids and stacking unchanged, so the
  selection stays selected. A shape counts as not moving when its offset
  is within `1e-5` of the box's magnitude (at least `1e-5` world units)
  — float rounding never adds an undo step on a second press. With no
  shape to move the transaction is empty and [[History]] records
  nothing.
- **Guards.** The commands do nothing during a gesture (it is not
  cancelled), with too few selected shapes, or when a moved shape would
  not be finite (the whole command is then dropped). The logic lives in
  `core::tools::select` beside scaling and reuses its move helpers.

## Alternatives considered

- **Distribute by centres** — equal centre spacing is simpler, but with
  shapes of different sizes the visible gaps differ; the owner asked for
  equal gaps.
- **Keep the shapes with the extreme far edge fixed** (the right-most
  right edge) — differs from the sort order only for nested shapes, and
  then the middle placement can overrun the fixed shape; sorting once
  and fixing its ends is the usual "distribute spacing".
- **Geometry boxes** (`geometry_bounds`, without stroke width) — what
  resizing uses, but alignment should match the outline the user sees,
  and shapes of different widths would then look misaligned.
- **Cancel the gesture and act** (as `run_edit` does for scaling) — a
  key pressed mid-drag would silently drop the drag; restack set the
  precedent of ignoring the key (ADR-T100-1).

## Consequences

- Graph edges are separate shapes: aligning the nodes leaves the edges
  where they were (out of scope of T101).
- Each command holds two clones of every moved shape in the undo stack;
  the byte budget (ADR-T49-1) counts them.

## Rollback plan

Remove the six bindings, `Command::Align` / `Command::Distribute`,
`Side` / `Axis` and the select-tool functions in a new ADR with
`git revert` of the T101 commits.
