---
id: ADR-T99-1
title: Spacing commands as kerns outside the atom list
status: proposed
kind: decision
date: 2026-11-10
task: T99
builds_on: ["[[ADR-T30-1 TeX subset, box layout and metrics]]", "[[ADR-T36-1 Integral class with scripts beside the sign]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T99-1 Spacing commands as kerns outside the atom list

## Context

T99 adds `\:`, `\;`, `\!`, `\quad`, `\qquad`, `\ ` and `~`. The only
spacing command so far, `\,`, is a glyph (U+2006) of class Ord in the
symbol table (ADR-T36-1). That cannot work for `\!`, which is negative,
and a glyph of class Ord changes the classes its neighbours see: in TeX
`a\;+\;b` keeps the binary spacing of `+` because glue is not an atom.

## Decision

- A new node `Node::Kern(String)` holds the source of a spacing command
  (`\:`, `\;`, `\!`, `\quad`, `\qquad`, `\ `, `~`), printed back as is.
  The table `KERNS` gives each source its width in em: 4/18, 5/18,
  −3/18, 1, 2, [`INTERWORD_SPACE`] and [`INTERWORD_SPACE`].
- `INTERWORD_SPACE` is 1/3 em, the interword space of Computer Modern
  (`\fontdimen2` of cmr10, 3.33 pt at 10 pt): the formula font is a
  Computer Modern design, while the block's monospaced cell is far wider.
- The layout collects kerns apart from the atoms: each atom carries the
  kerns typed before it, and a row ends with the kerns after its last
  atom. The class rules (unary binary operators, the space table) look
  only at atoms, so a kern never changes a class, as in TeX. A kern's
  width is multiplied by the scale, like the other spaces.
- A row never moves its cursor left of its own start: a negative kern is
  cut to what the row has laid out so far. So every box, and the whole
  formula, keeps a width of at least zero and no ink starts left of it.
- `\,` stays the U+2006 glyph: its tests stay unchanged (AC-4).
- A kern on its own (a script's base, `x^\;`) is an empty box of its
  width, class Ord.

## Alternatives considered

- **Kerns as symbols of the table** — `Node::Symbol` means "draws a
  glyph or word of a class"; a kern draws nothing and has no class.
- **Move `\,` onto the kern** — cleaner, but changes its class
  behaviour (`a\,+b`) and its width (the glyph advance, not 1/6 em
  exactly); kept as is to leave the present look and tests alone.
- **Interword space as the monospaced cell** (`ADVANCE`) — about twice
  TeX's, far too wide among math-font glyphs.
- **Let negative kerns hang left of the formula** — TeX does, but the
  ink would leave the block's bounds.

## Consequences

- `\!` at the very start of a formula or group does nothing.
- Every exhaustive match on `Node` has one more arm.

## Rollback plan

Remove `Node::Kern`, `KERNS` and the kern collection in the layout;
the commands go back to literals. Record it in a new ADR.
