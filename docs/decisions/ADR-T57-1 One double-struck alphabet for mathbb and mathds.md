---
id: ADR-T57-1
title: One double-struck alphabet for mathbb and mathds
status: accepted
kind: decision
date: 2026-11-04
task: T57
builds_on: ["[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]", "[[ADR-T56-1 Accent skew follows a slant flag per math alphabet]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T57-1 One double-struck alphabet for mathbb and mathds

## Context

[[T57 Blackboard bold]] adds blackboard bold. In LaTeX two packages
provide it: amssymb's `\mathbb` has capitals only (`\mathbb{1}` is an
error or a plain 1), and dsfont's `\mathds` has capitals plus `1`, `h`
and `k`, mostly for the indicator function `\mathds{1}_A`. Unicode, and
Latin Modern Math (in the subset since T55), have double-struck
capitals, lowercase and digits, with seven capitals filled earlier in
Letterlike Symbols: `ℂ ℍ ℕ ℙ ℚ ℝ ℤ`.

## Decision

- `\mathbb` and `\mathds` are the same row of `ALPHABETS` (`\mathds` is
  `..MATHBB` with its own name), as `\bm` is of `\boldsymbol`.
- The row maps Latin capitals from U+1D538, lowercase from U+1D552 and
  digits from U+1D7D8; Greek is inherited (Unicode has no double-struck
  Greek run). The seven capitals above are holes.
- draw accepts more than either package: lowercase and every digit, not
  only dsfont's `1 h k`. A formula valid in LaTeX draws the same; one
  that LaTeX rejects (`\mathbb{x}`) draws a double-struck letter instead
  of an error.
- The row is upright (`slant: false`, ADR-T56-1): an accent is centred.

## Alternatives considered

- **amssymb exactly (capitals only)** — `\mathbb{1}` would draw an
  italic-style plain 1, and the common `\mathds{1}` would need a second
  row anyway.
- **Two rows, `\mathds` with only `1 h k`** — needs per-character
  mapping the table does not have (ADR-T54-1 maps whole kinds); the
  extra glyphs are harmless.

## Consequences

- No new field or code path: two rows in the table.
- Owner decision (batch plan): no `\R`, `\N` shortcuts.

## Rollback plan

Revert the T57 `feat` commits: both commands become unknown again.
Record the rollback as a new ADR.
