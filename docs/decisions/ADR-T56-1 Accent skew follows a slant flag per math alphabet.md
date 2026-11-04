---
id: ADR-T56-1
title: Accent skew follows a slant flag per math alphabet
status: proposed
kind: decision
date: 2026-11-04
task: T56
builds_on: ["[[ADR-T39-1 Accents from spacing glyphs with a slant skew]]"]
amends: ["[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T56-1 Accent skew follows a slant flag per math alphabet

## Context

[[ADR-T54-1 Math alphabets as a table of per-kind mappings]] skews an
accent over one letter whenever the current alphabet maps it to a
different character. Until T56 every alphabet that changed a character
was slanted (math italic, `\mathit`, `\mathnormal`, `\mathcal`) and
`\mathrm` kept the character as typed. [[T56 Bold math]] adds `\mathbf`,
whose letters come from the *upright* bold block: with the T54 rule
`\hat{\mathbf{v}}` would have its hat pushed right, while LaTeX centres
it (the bold roman font has no skew character). Later alphabets
(blackboard, fraktur, sans-serif, monospace) are upright too.

## Decision

- `AlphabetDef` gains `slant: bool`, whether the glyphs the row maps
  lean right.
- An accent over one letter is skewed when the row that draws the letter
  maps it to another character **and** that row is slanted.
- Existing rows: `MATH_ITALIC`, `\mathit`, `\mathnormal`, `\mathcal`
  are slanted, `\mathrm` is not (it maps to the character itself, so
  nothing changes). New rows: `\mathbf` upright, `\boldsymbol`/`\bm`
  slanted.
- Because the flag belongs to the row that *decides* a kind, an
  inherited kind keeps its row's slant: `\mathbf{\hat{\alpha}}` draws
  math-italic α and stays skewed.

## Alternatives considered

- **Keep the T54 rule** — every bold capital and Latin letter would get
  a misplaced accent.
- **Infer the slant from the block start** (a list of italic starts) —
  hidden knowledge outside the table; each new alphabet would need to
  touch it.
- **A slant per kind** (`Map::Slanted(start)` vs `Map::Block(start)`) —
  more precise than needed: no alphabet mixes slanted and upright blocks
  among the kinds it maps.

## Consequences

- T57–T60 set `slant` on their rows (`false` for blackboard, fraktur,
  sans-serif upright and monospace; `true` for script and slanted sans).
- Output of every existing formula is unchanged
  (`tex_layout_accent_skew_follows_the_alphabet` and the T39 accent
  tests are the guard).

## Rollback plan

Revert the T56 `feat` commits: the field goes away and the T54 rule
returns; `\mathbf` would then skew accents. Record the rollback as a new
ADR.
