---
id: ADR-T93-1
title: Coordinate plane key and plotting rules
status: accepted
kind: decision
date: 2026-11-09
task: T93
builds_on: [ADR-T87-1, ADR-T84-1, ADR-T11-1, ADR-T47-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T93-1 Coordinate plane key and plotting rules

## Context

A selected text block of points should become axes, numbered ticks and
plotted points in one key ([[T93 Coordinate plane from points]]). It is the
same select-a-block-and-press-a-key pattern as
[[ADR-T84-1 Grid from text keys and table rules]] and
[[ADR-T87-1 Graph from an edge list key and layout rules]]. New keys need an
ADR ([[Keymap]]).

## Decision

- **Key.** `Ctrl+P` is bound to `Command::PlotFromText`, next to `Ctrl+T`
  (grid) and `Ctrl+L` (graph).
- **Grammar**, one pure parser `tools::layout::PointList::parse`: each
  non-blank line is exactly two whitespace-separated numbers `x y`, read as
  `f64`. Integers, decimals, negatives and exponents are numbers.
- **Refusals**, whole and never cut: a line that is not two numbers, a value
  that is not finite or above `1e300` in size, more than 1 000 points, or more
  than 64 KiB of text. The reason goes to the log through
  `Editor::take_notice`, and names the line number, not its text
  ([[ADR-T47-1 Flight recorder and capped session log]]). Blank text does
  nothing and logs nothing.
- **One step for both axes.** The step is the smallest of 1, 2 or 5 × 10ⁿ
  that is at least the larger axis span divided by 8. Both axes use it, with
  the same length per unit, so the plane is not distorted.
- **Frame.** Each axis runs from the last multiple of the step at or below
  its smallest value to the first at or above its largest. An axis whose
  values sit on one multiple runs one step either side of it.
- **Degenerate data.** A span that is zero, below `1e-290`, or below `1e-9`
  of the largest coordinate has no usable scale of its own. The step then
  covers `max(1, largest coordinate / 10)`, so one point or equal coordinates
  still give a valid plot.
- **Axes.** Each axis is drawn through the origin when that is in the frame,
  otherwise on the frame edge nearest the origin (`0.clamp(lo, hi)`), so the
  edge shows which way the origin lies.
- **Ticks and numbers.** Every multiple of the step in the frame has a short
  tick across its axis. `x` numbers are centred below their tick, `y` numbers
  to the left of theirs. When both axes pass through the origin the `y` zero
  is left out, so the origin is numbered once. Numbers are built from integers
  (`tick × mantissa`, shifted by the exponent), so they never show float
  noise; beyond `10⁶` or below `10⁻⁶` steps they use `15e-9` form.
- **Sizes follow the block's text size.** One step is the larger of 3 em and
  the widest number plus 1 em. A tick is 0.5 em long and a point is a filled
  circle of radius 0.2 em.
- **Result.** Axes, then ticks, then numbers, then points replace the block
  at its place in the z-order through `layout::replace_block`, in its colour
  and the current line width, as one undo step. The new shapes are selected.
  The top-left of everything drawn is the block's top-left. `y` grows upward.

## Alternatives considered

- **A frame around only the data, without rounding to the step** — the first
  and last ticks would float inside the axes, and the origin test would
  depend on rounding.
- **A separate scale per axis** — ruled out by the task: distances must read
  the same in both directions, as in geometry problems.
- **Axes always at the bottom and left** — hides which way the origin is when
  every value is negative.
- **Number the origin on both axes** — two zeros next to each other.
- **Accept any finite `f64`** — the span of `±1e308` overflows; a limit no
  contest reaches is simpler than scaling around it.

## Consequences

- Points can cover numbers when axes run through the data.
- Mixed magnitudes (`1e15` next to `1`) are drawn with the larger scale, so
  the small differences collapse.
- `Editor` shares one path for block-to-shapes keys; a new layout only adds
  its parser and its `shapes`.

## Rollback plan

Remove `Command::PlotFromText`, its binding and `PointList` with `revert(T93)`
commits and a rollback ADR. No stored format changes.
