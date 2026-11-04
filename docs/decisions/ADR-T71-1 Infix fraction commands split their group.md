---
id: ADR-T71-1
title: Infix fraction commands split their group
status: accepted
kind: decision
date: 2026-11-04
task: T71
builds_on: ["[[ADR-T30-1 TeX subset, box layout and metrics]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T71-1 Infix fraction commands split their group

## Context

[[T71 Combinatorics notation]] adds TeX's generalized fractions:
`n \choose k` (a binomial), `n \brack k` and `n \brace k` (Stirling
numbers), and, with the same machinery, `a \over b` and `a \atop b`.
They are infix: in TeX each one turns everything before it in the
enclosing group (or the whole formula) into the numerator and
everything after it into the denominator. The parser of
[[ADR-T30-1 TeX subset, box layout and metrics]] reads a list of atoms
left to right and only knows prefix commands with arguments. Every node
must print back to its exact source.

## Decision

- A new node `Node::Infix { name, left, right }`: the five names
  `choose`, `brack`, `brace`, `over`, `atop` (`INFIX` table), with the
  nodes before the command (`left`) and after it (`right`) in the same
  list. It prints as `left`, `\name`, `right`.
- The lexer is unchanged. `Parser::list` (the loop that reads a group
  or the formula) checks, before each atom, whether a `\` starts an
  infix name. The first one ends the left side: the nodes read so far
  become `left`, and the list goes on with an empty output for the
  right side, so a `^` right after the command has no base, as in TeX.
  At the end of the list the whole list is the one `Infix` node.
- A second infix command in the same list is not split again: it is an
  unknown command, a literal in the warning colour (TeX reports
  "Ambiguous; you need another { and }"). Nested groups split on their
  own: `{a \over b} \over c` is fine.
- Layout, at the sizes `\frac` and `\binom` use:
  - `\over` is `\frac` (bar on the axis), `\atop` the same stack
    without the bar;
  - `\choose` is `\binom`; `\brack` and `\brace` are the same stack in
    `[ ]` and `{ }`, scaled to its height like `\binom`'s parentheses.
- An infix node is an ordinary atom (`Class::Ord`), like `\frac`.

## Alternatives considered

- **A second pass over the finished node list** — the right side's
  first `^` would already have attached to the last node of the left
  side, which TeX does not do.
- **Lexing infix commands as a special token** — a lexer change for
  five names; checking in `Parser::list` is enough.
- **Splitting at the last infix command, or nesting them** — TeX
  rejects two in one group; drawing the second one as an unknown command
  shows the mistake.
- **Leaving `\over` and `\atop` out** — the machinery is the same and
  old TeX sources use them.

## Consequences

- A new `Node` variant: every match on `Node` (all in `core::tex`) gains
  an arm; the print round trip and the fuzzer cover it.
- `\frac{a}{b}` and `{a \over b}` draw the same ink, as do
  `\binom{n}{k}` and `{n \choose k}`.
- `\above`, `\abovewithdelims`, `\overwithdelims` and the other
  primitives stay unknown.

## Rollback plan

Revert the T71 `feat` commits: the `Infix` variant, its check in
`Parser::list` and its arms in `print_node` and `atom` go away, and the
five names become unknown commands (literals) again. Record the
rollback as a new ADR.
