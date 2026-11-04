---
id: ADR-T69-1
title: Physics shortcuts expand to their long forms
status: accepted
kind: decision
date: 2026-11-04
task: T69
builds_on: ["[[ADR-T30-1 TeX subset, box layout and metrics]]", "[[ADR-T39-1 Accents from spacing glyphs with a slant skew]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T69-1 Physics shortcuts expand to their long forms

## Context

[[T69 Classical physics notation]] adds the most used shortcuts of the
LaTeX `physics` package: `\dd`, `\dv`, `\pdv`, `\abs`, `\norm`,
`\order`, `\vb`, `\vu`, `\grad`, `\curl`. Each one is only a shorter way
to write a formula draw already lays out: `\dv{f}{x}` is
`\frac{\mathrm{d}f}{\mathrm{d}x}`, `\abs{x}` is `\lvert x\rvert`.
Some take a second argument only when it is there (`\dv{x}` and
`\dv{f}{x}`). [[T70 Quantum notation]] needs the same thing for
`\bra`, `\ket`, `\braket`, `\comm` and friends. Every node must still
print back to its exact source.

## Decision

- One table, `EXPANSIONS`, of `ExpansionDef { name, required,
  templates }`: the command takes `required` arguments of any form (a
  group, a symbol, one character, as `\frac` does), then up to
  `templates.len() - 1` **optional** arguments, each taken only when a
  `{group}` follows (whitespace allowed before it). The template used is
  `templates[optional arguments given]`.
- A template is TeX source of the subset with `#1` … `#9` slots. The
  parser keeps the shortcut as an ordinary `Node::Command` (so it
  prints back unchanged). At layout, `expand` parses the template and
  replaces each `#n` (inside groups and command arguments too) by the
  nodes of argument `n`; the user's nodes are never re-parsed.
- In a list the expansion is **spliced** into its neighbours, so a
  shortcut draws exactly the ink of its long form, spacing included
  (`\curl\vb{A}` is `\nabla\times\mathbf{A}`, a binary `×`). As the
  base of a script the expansion is one ordinary atom, like a group.
- A shortcut without its required arguments is a literal in the warning
  colour, like any command.
- The supported set and its long forms:

  | Shortcut | Long form |
  |----------|-----------|
  | `\dd`, `\dd{x}` | `\,\mathrm{d}`, `\,\mathrm{d}x` |
  | `\dv{x}`, `\dv{f}{x}` | `\frac{\mathrm{d}}{\mathrm{d}x}`, `\frac{\mathrm{d}f}{\mathrm{d}x}` |
  | `\pdv{x}`, `\pdv{f}{x}` | `\frac{\partial}{\partial x}`, `\frac{\partial f}{\partial x}` |
  | `\abs{x}` | `\lvert x\rvert` |
  | `\norm{v}` | `\lVert v\rVert` |
  | `\order{h^2}` | `\mathcal{O}(h^2)` |
  | `\vb{F}` | `\mathbf{F}` |
  | `\vu{n}` | `\hat{\mathbf{n}}` |
  | `\curl` | `\nabla\times` |

  `\grad` is a plain symbol row (`∇`, ordinary), like `\nabla`.
- Left out, and why:
  - `\div` stays `÷` (TeX's meaning); `physics` redefines it as the
    divergence, which would change existing formulas. `\laplacian` is
    left out by the owner.
  - Brackets do not grow in draw, so `\abs`, `\norm` and `\order` draw
    brackets at normal size; `\qty`, `\pqty`, `\bqty`, `\Bqty`, `\vqty`
    and `\eval` exist only to size brackets.
  - The optional `[n]` order (`\dv[2]{f}{x}`, `\dd[3]{x}`) and starred
    forms (`\dv*`): a bracket argument the parser does not know.
  - `\vdot`, `\cross`, `\fdv`, `\var`, `\mqty` (matrices), `\ev`,
    `\expval`, the bra–ket commands (T70 adds those through this table),
    `\Tr` (T70), `\Re`/`\Im` as `physics` redefines them.

## Alternatives considered

- **A layout function per command** (as `\binom` has) — ten functions
  that each rebuild by hand what `\frac`, `\mathbf` and the bracket rows
  already do, with a test burden to keep them equal.
- **Rewriting the source text before parsing** — breaks the print round
  trip and the caret positions of the editor, which read the source.
- **Expanding in the parser into a new node holding both source and
  expansion** — a new `Node` variant and a second tree to keep in sync;
  expanding at layout keeps the parse tree as typed.
- **Laying out the expansion as one group** — `\curl\vb{A}` would lose
  the binary-operator space after `×`, and `\abs{x}` the bracket
  classes; splicing gives the long form's ink exactly.

## Consequences

- `\dv{f}{x}` and `\frac{\mathrm{d}f}{\mathrm{d}x}` draw the same ink;
  every shortcut is tested against its long form.
- A new command of this kind is one table row plus a test, which is how
  T70 adds the quantum notation.
- The template is parsed again at each layout; templates are a few
  characters, and layouts are cached by the shell.

## Rollback plan

Revert the T69 `feat` commits: `EXPANSIONS`, `expand` and the parser arm
go away, and the shortcuts become unknown commands (literals) again.
Record the rollback as a new ADR.
