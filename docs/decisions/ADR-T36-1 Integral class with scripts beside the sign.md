---
id: ADR-T36-1
title: Integral class with scripts beside the sign
status: accepted
kind: decision
date: 2026-10-21
task: T36
builds_on: [ADR-T30-1, ADR-T33-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T36-1 Integral class with scripts beside the sign

## Context

The TeX subset ([[ADR-T30-1 TeX subset, box layout and metrics]]) has
two operator classes with limits: `BigOp` (`∑ ∏`, enlarged, limits above
and below) and `Limits` (`min max lim`). In inline LaTeX an integral is
enlarged like `∑` but its limits are scripts to the right of the sign.
The sign is slanted: Latin Modern Math gives `∫` an italic correction of
0.332 em, which [[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]
already uses to place superscripts.

## Decision

- A new `Class::Integral`. `\int ∫`, `\iint ∬`, `\iiint ∭`, `\oint ∮`,
  `\oiint ∯` have it.
- The glyph is drawn like a `BigOp`: at `BIG_OP_SCALE`, its middle on
  the axis, but its box **keeps the italic correction** of the enlarged
  glyph.
- `_` and `^` on an integral are scripts (the `scripts` rules of
  ADR-T33-1), not limits; the subscript moves left by the italic
  correction, under the slant, as the OpenType `MATH` specification does
  for operators without limits. The superscript starts after the italic
  correction as for any base.
- For spacing and for the unary-binary rule an integral is an operator,
  like `BigOp` and `Limits` (thin space before an ordinary atom).
- `\,` is a table symbol drawn as U+2006 (1/6 em, the TeX thin space),
  so it needs no new node kind.

## Alternatives considered

- **A `limits: bool` flag on `BigOp`** — every `match` on the class
  would read a field as well; a class keeps the table a flat
  `(name, text, class)` row and makes `\bigcup`-style operators a
  one-line addition either way.
- **Subscript at the advance** — leaves a gap under the slant that grows
  with `BIG_OP_SCALE`; the sign's foot curls left, so moving the
  subscript left by the italic correction keeps it clear of the ink.
- **A `Node::Kern` for `\,`** — exact TeX semantics (a kern, not an
  atom), but a new node kind in parse, print and layout for one space.

## Consequences

- `\int_0^1 f(x)\,dx` typesets as in inline LaTeX.
- `\limits` / `\nolimits` and display-size integrals remain out of the
  subset.
- `\,` is an ordinary atom: between two operators it adds its width to
  the operator space instead of standing alone.

## Rollback plan

A rollback ADR plus `revert(T36)` commits remove the class and the
rows; `\int` becomes a literal again.
