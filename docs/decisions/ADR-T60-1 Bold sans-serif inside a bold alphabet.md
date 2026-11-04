---
id: ADR-T60-1
title: Bold sans-serif inside a bold alphabet
status: accepted
kind: decision
date: 2026-11-04
task: T60
builds_on: ["[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]", "[[ADR-T56-1 Accent skew follows a slant flag per math alphabet]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T60-1 Bold sans-serif inside a bold alphabet

## Context

[[T60 Sans-serif and typewriter letters]] adds `\mathsf` (sans-serif
Latin `U+1D5A0…`, digits `U+1D7E2…`) and `\mathtt` (monospace Latin
`U+1D670…`, digits `U+1D7F6…`). Unicode also has bold sans-serif
(`U+1D5D4…`, digits `U+1D7EC…`) and bold italic sans; it has no bold
monospace. Latin Modern Math 1.959 has all of these glyphs (rows in
`core::math_metrics`).

ADR-T54-1 says the innermost alphabet that maps a kind wins:
`\mathbf{\mathfrak{g}}` is a plain fraktur g. The owner (batch plan,
2026-11-04) wants `\mathsf` inside a bold alphabet (`\mathbf`,
`\boldsymbol`, `\bm`) to draw **bold sans**, upright in all three cases
(LaTeX's `\mathsf` is upright), and `\mathsf{\mathbf{x}}` to give the
same result. `\mathtt` is unchanged under bold.

## Decision

- `AlphabetDef` gains a `bold: bool` field: `true` for the bold weights
  (`\mathbf`, `\boldsymbol`, `\bm` and the internal bold-sans row),
  `false` for every other row (`\mathbb` is not a weight).
- An internal row `MATHSF_BOLD` (not in `ALPHABETS`, so not a command;
  its name is `mathsf`) maps Latin to `U+1D5D4…` / `U+1D5EE…` and
  digits to `U+1D7EC…`, Greek inherited, `slant: false`.
- The exception lives in `Style::with`, per kind, and only for sans:
  - entering `\mathsf` where the current row is bold → bold sans;
  - entering a bold alphabet where the current row is sans (regular or
    bold) → bold sans;
  - in every other case the innermost alphabet wins, as before.
- So `\mathbf{\mathsf{x}}`, `\bm{\mathsf{x}}`, `\boldsymbol{\mathsf{x}}`,
  `\mathsf{\mathbf{x}}` and `\mathsf{\bm{x}}` all draw `𝘅`;
  `\mathbf{\mathsf{\mathbf{x}}}` too. Greek is not sans and follows the
  bold alphabet as before. `\mathrm` (or any other alphabet) in between
  breaks the chain: innermost wins.

## Alternatives considered

- **Innermost wins (regular sans under bold)** — simpler, but the owner
  chose bold sans; LaTeX with `\boldmath` or `\bm` also gives bold sans.
- **A `bolded: Option<&AlphabetDef>` field on every row** — general, but
  the bold-sans row would need to point at itself (a const cycle), and
  only one row uses it.
- **Bold italic sans for `\boldsymbol` / `\bm`** — `\mathsf` is upright
  in LaTeX; the owner chose upright in all three cases.

## Consequences

- One new field (`bold`), one internal row and a three-line rule; every
  T54–T59 nesting result is unchanged (the rule only fires when one side
  is a sans row).
- Bold italic sans (`U+1D63C…`) stays unused.
- Accents over bold sans are centred (`slant: false`).

## Rollback plan

Revert the T60 `feat` commits: `\mathsf` and `\mathtt` become unknown
commands again and the `bold` field goes away. Record the rollback as a
new ADR.
