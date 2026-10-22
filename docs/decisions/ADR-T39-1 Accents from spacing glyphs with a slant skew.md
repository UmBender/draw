---
id: ADR-T39-1
title: Accents from spacing glyphs with a slant skew
status: accepted
kind: decision
date: 2026-10-21
task: T39
builds_on: [ADR-T30-1, ADR-T33-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T39-1 Accents from spacing glyphs with a slant skew

## Context

Physics notes need `\vec`, `\hat`, `\tilde`, `\dot`, `\ddot`, `\bar` and
`\overline`. LaTeX draws them from the font's combining accents and
places them with the `MATH` table's top-accent attachment points. The
embedded Latin Modern Math subset has neither: no combining marks
(`U+0300–036F`, `U+20D0–20FF`), no spacing modifiers, and the `MATH`
table is dropped (ADR-T33-1). It does have spacing forms with rows in
`core::math_metrics`: `^ ~ ¨ ¯ · →`.

## Decision

- An accent is a `Node::Command` with one argument, like `\sqrt`; a table
  `tex::ACCENTS` maps each name to a `Mark`: a math-face glyph at a
  fraction of the argument's scale (`→` at `VEC_SCALE` = 0.5, the others
  at 1), a short rule (`\bar`, `ACCENT_BAR` long, the macron's ink width)
  or a rule from the argument's left edge to its advance (`\overline`).
  Rules are `RULE` thick, like fraction bars.
- The mark's ink bottom sits `ACCENT_GAP` above the argument's ascent;
  glyphs and `\bar` are centred on the argument's ink span.
- When the argument is one italic letter, the centre moves right by
  `ACCENT_SKEW × ascent` (0.1 per em of height): the top of a slanted
  letter lies right of its middle. This stands in for the dropped
  top-accent attachment and is close to TeX's skew kerns for the usual
  letters (`x`, `v`, `F`).
- The accented box keeps the argument's width, descent and italic
  correction and grows its ascent to the mark's top. It is an ordinary
  atom, so scripts attach to it as to any nucleus.

## Alternatives considered

- **A new `Node::Accent` variant** — the same data as a one-argument
  command; it would duplicate printing and parsing for no gain.
- **Grow the font subset with combining accents and the `MATH` table** —
  exact placement, but it regenerates the font and the metrics table and
  adds a table parser; out of scope for the task.
- **`\bar` as a rule as wide as the argument** — that is `\overline`;
  LaTeX's `\bar` is a fixed-width macron, so `\bar{AB}` stays short.
- **The `¯` glyph for `\bar`** — a rule of the same width draws the same
  ink and shares the bar thickness with fractions.

## Consequences

- Accents work with no font change and no new node kind; the shell draws
  them with the glyphs and rules it already batches.
- The skew is an approximation: letters with unusual tops (`A`, `f`) may
  look slightly off-centre compared with LaTeX.
- Marks do not widen: an accent wider than its argument (`\vec{i}`)
  overhangs its neighbours, as in TeX.

## Rollback plan

Remove the `ACCENTS` table and its arms in `Parser::command` and the
layout's command match; the commands become literals again. A later ADR
that brings the `MATH` table back would supersede the skew.
