---
id: ADR-T59-1
title: One script alphabet for mathscr and mathcal
status: accepted
kind: decision
date: 2026-11-04
task: T59
builds_on: ["[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]", "[[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T59-1 One script alphabet for mathscr and mathcal

## Context

[[T59 Script letters]] adds `\mathscr`. In LaTeX, `\mathscr` (rsfs) is
a rounder, more formal script than `\mathcal`, and it is capitals only
too. Unicode has one script alphabet: capitals `U+1D49C–1D4B5` (eight
filled earlier in Letterlike Symbols) and lowercase `U+1D4B6–1D4CF`
(holes `ℯ ℊ ℴ`); it tells chancery and roundhand apart only with the
variation selectors `U+FE00` / `U+FE01`.

Facts about Latin Modern Math 1.959, the formula font (measured from
the T55 subset, whose ranges cover all of `U+1D400–1D7FF` and
`U+2100–214F`, and from the T55 plan check of the original font):

- It has one script style: no variation sequences (no cmap format 14),
  and its GSUB has only `aalt dtls ssty`, no stylistic set.
- It has the script capitals (all 26 with the eight Letterlike holes)
  and the bold script capitals `U+1D4D0–1D4E9`.
- It has **no script lowercase**: none of `U+1D4B6–1D4CF`, none of
  `ℯ U+212F`, `ℊ U+210A`, `ℴ U+2134`, and no bold script lowercase
  `U+1D4EA–1D503`. `core::math_metrics` has no row for any of them.

The batch plan asked for lowercase script, and its shared rule says: a
command that needs a glyph the T55 subset lacks is dropped and recorded
in *Out of scope*, not faked with a fallback cell. Only T55 changes the
subset, and a second font is out of scope.

## Decision

- `\mathscr` is the `\mathcal` row of `ALPHABETS` under its own name
  (`..MATHCAL`), as `\mathds` is of `\mathbb`: Latin capitals from
  U+1D49C with the eight holes, every other kind inherited, slanted
  (`slant: true`, ADR-T56-1).
- `\mathscr` and `\mathcal` capitals look the same: the font has no
  second script style to pick.
- Lowercase script is dropped: `\mathscr{l}` draws what `\mathcal{l}`
  draws, the lowercase of the surrounding style (math italic `𝑙`), from
  the math face, by the T54 inherit rule. No fallback cell, no warning
  colour.
- A test pins the missing glyphs (no `math_metrics` row for script
  lowercase or `ℯ ℊ ℴ`), so a font that gains them fails it and the
  lowercase map can be added then.

## Alternatives considered

- **Map lowercase to `U+1D4B6…` anyway** — every letter would be a
  missing glyph: a monospaced fallback cell, against the plan's rule.
- **Block the task** — the capitals, `\mathscr{L}` and `\mathscr{F}`,
  are the common use and the font draws them; the lowercase gap is a
  font fact the owner can lift later (a font with script lowercase, or
  rsfs-style glyphs, is a new task).
- **Draw lowercase with `ℓ` or bold script** — not script lowercase,
  and only for one letter or a different weight.

## Consequences

- No new field or code path: one more row in the table.
- `\mathscr{l}` is an italic l, not a script l, until the font changes.
- Bold script capitals are in the font but unused: `\mathbf{\mathscr{L}}`
  keeps the innermost rule of T54 (a plain script L).

## Rollback plan

Revert the T59 `feat` commits: `\mathscr` becomes an unknown command
again. Record the rollback as a new ADR.
