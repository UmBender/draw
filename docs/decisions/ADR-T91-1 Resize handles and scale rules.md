---
id: ADR-T91-1
title: Resize handles and scale rules
status: accepted
kind: decision
date: 2026-11-09
task: T91
builds_on: ["[[ADR-0004 Vector object model]]", "[[ADR-0005 Undo via transaction log]]", "[[ADR-T08-1 Tool context and gesture overlay]]", "[[ADR-T24-2 Overlay drawn straight to the window]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T91-1 Resize handles and scale rules

## Context

[[T91 Resize handles]] adds the second transform after moving. Its
spec fixes the gesture, the keys and a 4-unit minimum, but not what the
minimum means for a thin line, how one text size follows two scale
factors, or where a handle sits relative to the box being scaled.

## Decision

**Keys.** `Alt+[` scales the selection by 0.9 and `Alt+]` by 1.1, about
the centre of its geometry box. The plain and `Ctrl` brackets keep their
meaning. Any tool can use the keys, as with `Ctrl+D`.

**Handles.** Four squares of 10 px, centred on the corners of the
selection outline. They show and work only in the Select tool, with no
gesture and no open text block, because a press means something else in
the other tools. A press takes a handle only inside its square, the
nearest one if squares overlap, and never with `Alt`. A selection whose
geometry box has no width and no height has no handles, since nothing
could scale.

**The box that scales.** Scaling uses the *geometry box*: stroke points,
the two ends of a line or arrow, the corners of a rectangle, ellipse or
grid, and the layout box of a text block. Outlines, arrow heads and grid
index bands are left out: they keep their size, so including them would
move the opposite corner. The pointer's world delta is added to the
dragged corner of that box, so the handle's 4 px offset causes no jump
on press.

**One frame, cached at the press.** The pivot, the dragged corner and
the minimum scales are worked out once when the drag starts. The
preview then costs one clone-and-scale per selected shape, the same
order as a move.

**Scale factors.** One per axis. A side with no extent keeps factor 1.
With `Shift` both axes take the larger factor. Factors are clamped from
below, never flipped: at least `0.001`, and at least the selection's
minimum.

**The 4-unit minimum** comes from each shape, and the largest per axis
wins, so the group keeps its layout.

- A rectangle, ellipse or grid limits both sides of its box.
- A stroke, line or arrow limits only its longer side. A nearly flat
  line may be scaled down proportionally.
- A text block limits its font size, on both axes.
- A side already at or under 4 units cannot shrink further; a side with
  no extent is no limit.

**Text.** A block has one size, so it takes the smaller of the two
factors, and stays inside the scaled box. Text inside shapes and cells
scales its size the same way and adds no limit, as the box limits it.

**Non-finite results.** A factor that is not finite and positive, or a
result with a non-finite coordinate or text box, cancels the whole
operation: no change and no undo step.

**Stroke widths and arrow heads stay.** Snapping does not apply to a
resize.

## Alternatives considered

- **Minimum per shape, each clamped on its own** — the group would come
  apart as its small members stop.
- **Treating every axis alike for lines** — a near-horizontal line with
  a 5-unit rise could not be halved with `Shift`.
- **Geometric mean of the factors for text** — the text could leave the
  scaled box.
- **Handles in every tool** — they would do nothing where a press draws.
- **Snapping the dragged corner** — wanted only if it is missed in use;
  it would mix the two sets of rules.

## Consequences

- A selection holding one small shape stops shrinking when that shape
  reaches 4 units, however large the rest is.
- Handles cover the corners of very small selections; their press wins
  there, and `Alt`-drag or the keys still reach the shape.
- The new `select resize` snapshot record is additive: no version bump
  and no fixture changes.

## Rollback plan

Add a rollback ADR, revert the T91 commits and drop the two
`Command` variants and their bindings. No stored format depends on
this, since the document only ever holds the scaled shapes.
