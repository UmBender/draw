---
id: ADR-T41-1
title: Integral superscript at the advance
status: accepted
kind: decision
date: 2026-10-22
task: T41
builds_on: [ADR-T33-1]
amends: [ADR-T36-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T41-1 Integral superscript at the advance

## Context

[[ADR-T36-1 Integral class with scripts beside the sign]] puts an
integral's superscript "after the italic correction as for any base", at
`advance + italic`. In 1.4.2 the upper limit of `$\int_0^\pi$` floats
about half an em right of the sign.

Latin Modern Math is an OpenType `MATH` font. There, the advance of a
large operator already **includes** its italic correction: `∫` has
advance 0.665 em, ink right edge 0.609 em and correction 0.332 em, so
the top of the sign ends inside the advance. The correction only says
how far the subscript moves left under the slant. Adding it again for
the superscript counts it twice. Italic letters are different: their
advance excludes the correction (`𝑓` ends past its advance), so `f^2`
is right at `advance + italic`.

## Decision

- On a `Class::Integral` base the superscript starts at the base's
  advance; the subscript stays at advance − italic correction (unchanged
  from ADR-T36-1). This is the OpenType `MATH` rule for operators
  without limits, and what LuaTeX and XeTeX do.
- Every other base keeps the superscript at `advance + italic`
  ([[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]).
- `scripts` takes the superscript's offset from the advance as an
  argument, next to the subscript's `tuck`, so the class decides both in
  `atom`.

## Alternatives considered

- **Zero the italic correction of the integral box and keep the tuck
  elsewhere** — the box's `italic` would then lie about the glyph, and
  the tuck would need a second source.
- **Subtract `advance − x_max` instead (superscript at the ink edge)** —
  not the font's rule; the font designer's advance already holds the
  side bearing a script needs.

## Consequences

- `\int_0^\pi`, `\oint_C^D` and the other integrals put the upper limit
  about 0.07 em (at 1.3×) right of the ink, as in inline LaTeX.
- An integral with a superscript is narrower by the enlarged correction
  (0.43 em), so text after it moves left.
- `MathBox::italic` keeps one meaning (the glyph's correction); how a
  base uses it is the class's business.

## Rollback plan

A rollback ADR plus `revert(T41)` commits restore the superscript at
`advance + italic` for integrals.
