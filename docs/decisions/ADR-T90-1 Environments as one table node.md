---
id: ADR-T90-1
title: Environments as one table node
status: accepted
kind: decision
date: 2026-11-09
task: T90
builds_on: [ADR-T30-1, ADR-T89-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T90-1 Environments as one table node

## Context

[[T90 Matrices and cases]] adds `matrix`, `pmatrix`, `bmatrix`, `vmatrix`
and `cases`. A formula is one line of source and the parse tree must stay
lossless ([[ADR-T30-1 TeX subset, box layout and metrics]]).
[[ADR-T89-1 Left and right brackets centred on the axis]] already pairs
`\left` and `\right` in one linear pass and grows brackets about the math
axis.

`&` and `\\` have to end a cell only when they belong to that environment,
and a script such as `x^&` must not swallow the separator. Trying to parse
a body and re-parsing it as plain nodes on failure would be exponential on
nested environments.

## Decision

### Syntax and parse tree

- An environment is `Node::Environment { name, lead, end_lead, rows }`, with
  `rows` a list of rows of cells of nodes. `lead` and `end_lead` are the
  whitespace before each `{name}`. The separators are implied by the shape
  of `rows`, so the tree stays lossless.
- The table `ENVIRONMENTS` holds each name, its brackets, its column
  alignment and its row stretch.
- `\begin` and `\end` pair in `match_pairs`, on the stack and depth limit
  that `{` and `\left` already use. An `\end` pairs only with a `\begin` of
  the same name directly on top of the stack, so an environment never
  crosses a group or a `\left` pair.
- The cells are found **before** any of them is parsed: one walk over the
  body that jumps over each matched group, `\left` pair and nested
  environment. A separator inside one of those belongs to it, and each
  cell is then parsed bounded by its own end, so `x^&` leaves a literal
  `^`. The walk visits each character once per formula.
- An environment that is unknown, has no matching `\end` or goes over
  `ENVIRONMENT_MAX_CELLS` (64) rows or columns is invalid: `\begin{name}`
  and `\end{name}` are literals in the warning colour, and what lies
  between them is parsed as usual. A `\begin` without a `{name}` of ASCII
  letters is a literal on its own.
- A blank last row, as after a trailing `\\`, is kept by the parser for
  printing but neither counted against the limit nor drawn, as in LaTeX.

### Layout

- Each cell is laid out once at the current scale. A column is as wide as
  its widest cell. A short row leaves its missing cells empty.
- Columns are separated by `ARRAY_COLUMN_GAP` (1 em, amsmath's two
  `\arraycolsep` and the `\quad` of `cases`), with no padding outside.
- A row is at least LaTeX's strut tall, `ARRAY_ROW_ASCENT` (0.84 em) over
  `ARRAY_ROW_DESCENT` (0.36 em), and grows with its cells; rows touch.
  `cases` multiplies the strut by `CASES_STRETCH` (1.2), as amsmath does.
- The block is centred on the math axis.
- The brackets go through the code of `\left` and `\right`, so a `pmatrix`
  draws exactly what `\left( \begin{matrix} … \end{matrix} \right)` draws.
  `matrix` has none; `cases` is `\{` on the left and `.` on the right.
- The environment is one ordinary atom, like a `\left` pair.

## Alternatives considered

- **Split on separators while parsing the list** — a script or command
  argument would take a `&` as its argument, and `list` would need to know
  whether it is inside an environment.
- **Parse the body, fall back on failure** — exponential on nested
  environments that fail, such as an outer one over the cell limit.
- **Only the word `\begin` in red** — the name after it would be drawn in
  black italics, so a misspelt environment would not read as an error.
- **A higher bracket cap for matrices** — the owner's spec says brackets
  use T89's scaling.

## Consequences

- Plain `&` and `\\` outside an environment keep their meaning: `&` an
  ordinary character and `\\` a red literal.
- The brackets stop at 12 ×, about 14 em, so a matrix of more than about a
  dozen rows sticks out of them, as [[ADR-T89-1 Left and right brackets centred on the axis]]
  states for tall content.
- A `&` inside a `\left` pair that spans a cell stays an ordinary character.
- No `array` with a column spec, no `smallmatrix`, no `dcases`, no
  source spread over several text lines.

## Rollback plan

A rollback ADR plus `revert(T90)` commits remove `Node::Environment` and
`ENVIRONMENTS` and return `match_pairs` to braces and `\left`; `\begin` and
`\end` are then literals again.
