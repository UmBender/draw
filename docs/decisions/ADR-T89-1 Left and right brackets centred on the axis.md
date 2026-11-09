---
id: ADR-T89-1
title: Left and right brackets centred on the axis
status: accepted
kind: decision
date: 2026-11-09
task: T89
builds_on: [ADR-T30-1, ADR-T33-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T89-1 Left and right brackets centred on the axis

## Context

[[T89 Growing brackets]] adds `\left` and `\right`. The font subset has no
size variants, so a bracket grows by scaling its glyph, as `\binom` does
through `delimited` ([[ADR-T30-1 TeX subset, box layout and metrics]]).
`delimited` centres its brackets on the middle of the stack and stops at
`BINOM_MAX_SCALE` (3 ×). That cap is too low for general content: a
fraction whose parts are sums already needs about 4 em.

The parser has to pair the commands. TeX requires a `\left` and its
`\right` to be in the same group, and a parser that looks ahead for each
`\left` is quadratic on input such as many unmatched `\left(`.

## Decision

### Syntax and parse tree

- `\left` and `\right` each take one delimiter, after optional whitespace:
  `( ) [ ] \{ \} | \| \langle \rangle \lfloor \rfloor \lceil \rceil`, or
  `.` for none (table `DELIMITERS`).
- A pair is `Node::LeftRight { open, body, close }`. Each `Delimiter` keeps
  its source and the whitespace before it, so the tree stays lossless.
- Pairs are found once per formula, in the pass that matches braces
  (`match_pairs`), on one stack and with one depth limit (`MAX_DEPTH`).
  A `}` drops every `\left` still open inside its group, and a `\right`
  pairs only with a `\left` directly on top of the stack, so a pair never
  crosses a group.
- A `\left` or `\right` without a partner or without a valid delimiter is a
  literal drawn in the warning colour. Only the command is literal: what
  follows it is parsed as usual.

### Layout

- The body is laid out at the current scale. The span a bracket must cover
  is symmetric about the math axis, as in TeX:
  `2 × max(ascent − axis, descent + axis)`.
- The target height is that span × `DELIM_FACTOR` (0.901, TeX's
  `\delimiterfactor`), so a slight overshoot such as a superscript does not
  grow the brackets.
- Each bracket's scale is the target over its own glyph height, clamped to
  `1 ..= LEFT_RIGHT_MAX_SCALE` (12) × the current scale. The middle of the
  scaled glyph sits on the axis.
- `.` draws nothing and takes `NULL_DELIM_SPACE` (0.12 em, TeX's
  `\nulldelimiterspace`).
- The pair is one ordinary atom. Scripts after `\right` attach to it, so
  they start at its right edge and clear its height.

## Alternatives considered

- **Reuse `delimited`** — it takes two stacked arguments and centres on the
  stack, so `\left( x^2 \right)` would shift off the axis.
- **Keep the 3 × cap** — tall content would stick out of its brackets. The
  new cap covers about ten rows of a matrix; beyond it a parenthesis is
  already over 4 em wide.
- **Look ahead from each `\left`** — quadratic on hostile input, and it
  needs its own group tracking.
- **TeX's Inner atom class** — adds a thin space beside the brackets and a
  new class to the spacing rules; for a scratch canvas tighter brackets
  are acceptable.
- **`\delimitershortfall`** — with no discrete sizes to choose from, the
  factor alone is enough.

## Consequences

- `\binom` and friends keep their behaviour and their 3 × cap.
- Scaled glyphs get heavier and wider with height, because the font has no
  size variants. Very tall brackets are drawn from a raster capped at
  `MAX_RASTER_PX`, so they can look soft.
- Taller than the cap, the content sticks out of its brackets above and
  below.
- `\middle` and other delimiter names (`\vert`, `\lvert`, …) stay out of
  the subset; after `\left` they are invalid and show in red.

## Rollback plan

A rollback ADR plus `revert(T89)` commits remove `Node::LeftRight` and
return `match_pairs` to matching braces only; `\left` and `\right` are
then literals again, as before this task.
