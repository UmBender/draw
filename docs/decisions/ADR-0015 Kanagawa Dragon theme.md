---
id: ADR-0015
title: Kanagawa Dragon theme
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0012]
amends: [ADR-0012]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0015 Kanagawa Dragon theme

## Context

[[ADR-0012 Fixed palette and theme tokens]] defined the colour model with
provisional values. The user supplied their Alacritty theme (`theme.toml`,
Kanagawa Dragon) as the base for the UI. The source file was removed from the
repository after this decision; its full palette is preserved in [[Theme]].

## Decision

Replace the provisional values with colours taken **only** from the user's
theme, and add one UI token, `selection`, for selection/marquee highlight
(the theme defines a selection colour, so it is used as-is).

UI tokens:

| Token | Source in theme | Value |
|-------|-----------------|-------|
| `bg` | `primary.background` | `#181616` |
| `surface` | `normal.black` | `#0d0c0c` |
| `border` | `bright.black` | `#a6a69c` |
| `text` | `primary.foreground` | `#c5c9c5` |
| `accent` | `normal.blue` | `#7fa8bc` |
| `selection` | `selection.background` | `#2d4f67` |

Drawing palette (keys `1`–`6`), the theme's `normal` colours — muted, readable
on the dark background:

| Key | Name | Source | Value |
|-----|------|--------|-------|
| `1` | ink | `primary.foreground` | `#c5c9c5` |
| `2` | red | `normal.red` | `#d16961` |
| `3` | green | `normal.green` | `#8aa86e` |
| `4` | blue | `normal.blue` | `#7fa8bc` |
| `5` | yellow | `normal.yellow` | `#ceb680` |
| `6` | magenta | `normal.magenta` | `#aa88ac` |

The colour model of ADR-0012 (palette index per shape, all colours in
`core::palette`) is unchanged.

## Alternatives considered

- **Bright variants for drawing colours** — more saturated, but harsher on a
  dark background during long contests.
- **Parsing `theme.toml` at runtime** — needs a TOML dependency and a file
  that can go missing mid-contest; colours are compiled in instead.

## Consequences

- [[T02 Palette and theme tokens]] implements these values and the extra
  `selection` token.
- Changing the theme later = a new ADR amending this one + edit of
  `core::palette` and [[Theme]].

## Rollback plan

Rollback ADR reverting this one restores the provisional values of ADR-0012.
