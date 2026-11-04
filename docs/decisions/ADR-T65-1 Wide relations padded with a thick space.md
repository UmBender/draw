---
id: ADR-T65-1
title: Wide relations padded with a thick space
status: accepted
kind: decision
date: 2026-11-04
task: T65
builds_on: ["[[ADR-T30-1 TeX subset, box layout and metrics]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T65-1 Wide relations padded with a thick space

## Context

[[T65 Set and logic symbols]] adds `\implies` (`⟹`) and `\impliedby`
(`⟸`). amsmath defines them as `\;\Longrightarrow\;` and
`\;\Longleftarrow\;`: a relation with an explicit thick space (5 mu) on
each side, so `p \implies q` is wider than `p \Longrightarrow q`. The
owner (batch plan, 2026-11-04) wants this wider spacing.

`core::tex` spaces atoms by class only (`spacing`), and a `SymbolDef`
row has a name, a text and a class. Relations get no space in scripts.

## Decision

- A constant `WIDE_REL_SPACE` (5/18 em, TeX's thick space) and a list
  `WIDE_RELATIONS` (`implies`, `impliedby`) in `core::tex`.
- `symbol_box` pads the glyph of a listed symbol with a gap of
  `WIDE_REL_SPACE × scale` on each side. The atom stays `Rel`, so the
  class spacing is added on top, as in amsmath.
- The padding is drawn at every size, scripts included: amsmath's `\;`
  is in mu, which scales with the style, while TeX drops only the
  class spacing of relations in scripts.

## Alternatives considered

- **A `wide` field on `SymbolDef`** — every one of the ~200 rows would
  change for two symbols.
- **A new `Class`** — every match on `Class` (unary binary rule,
  spacing, alphabets) would need a case; the atom is still a relation.
- **Space characters in the row's text** (`" ⟹ "`) — the text would no
  longer be one character, so it would be drawn as an upright word in
  the text face, and Unicode has no exact 5/18 em space.
- **Plain `Rel` spacing** — simpler, but not what the owner chose, and
  not what LaTeX draws.

## Consequences

- Any later "spaced" relation (e.g. `\iff` if ever made to match
  amsmath) is one entry in `WIDE_RELATIONS`.
- `\iff` keeps its current plain `Rel` spacing (unchanged here).

## Rollback plan

Remove `WIDE_RELATIONS` and the padding in `symbol_box`: `\implies` and
`\impliedby` get plain relation spacing. Record it as a new ADR.
