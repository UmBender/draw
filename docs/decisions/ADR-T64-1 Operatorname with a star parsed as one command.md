---
id: ADR-T64-1
title: Operatorname with a star parsed as one command
status: accepted
kind: decision
date: 2026-11-04
task: T64
builds_on: ["[[ADR-T30-1 TeX subset, box layout and metrics]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T64-1 Operatorname with a star parsed as one command

## Context

[[T64 Named operators with limits]] adds `\operatorname{…}` (an upright
operator word, spaced like `\log`) and `\operatorname*{…}` (the same word
taking limits, like `\max`). The parser of [[ADR-T30-1 TeX subset, box layout and metrics]]
reads a command name as a run of ASCII letters or one other character,
so the `*` is not part of the name, and no command so far had a starred
form. The body is a word, not math: `\text{…}` already parses a raw,
balanced `{…}` body. Every node must print back to its exact source.

## Decision

- `\operatorname` parses to a new node `Node::OperatorName { star, lead,
  body }`. The lexer is unchanged: `Parser::command` reads the name, then
  takes a `*` only if it comes **directly** after `operatorname` (no
  space between). The body is read raw like `\text` (whitespace `lead`,
  then a balanced `{…}` group, kept as a string).
- Layout: the body is drawn upright from the math face (as `\text`
  draws it) with class `Class::Op`, or `Class::Limits` when starred. So
  `\operatorname{log}` lays out exactly as `\log`, and
  `\operatorname*{max}_k` exactly as `\max_k`.
- Without a `{…}` body, the command is a literal in the warning colour:
  `\operatorname`, or `\operatorname*` with its star, since the star was
  typed as part of the command. The cursor goes back to just after the
  name (and star), so what follows parses normally.
- `\operatorname *{x}` (space before the star) is the unstarred command
  without a body: the literal `\operatorname`, then a space, `*` and a
  group, as in LaTeX where the star must follow at once.

## Alternatives considered

- **Lexing `*` as part of every command name** — a lexer change that
  would turn `\cdot*` or `\alpha*` into unknown commands; only one command
  needs a star.
- **Reusing `Node::Command` with a parsed argument** — the body would be
  math (italic letters, spacing between them, `-` as minus); LaTeX sets
  the operator name as a word.
- **Reusing `Node::Text` with a flag** — `\text` is an ordinary atom and
  prints as `\text`; a separate node keeps print and class simple.
- **Star as a separate `Char('*')` node when the body is missing** — the
  star would be drawn as a normal asterisk, hiding that the command was
  not understood.

## Consequences

- A new `Node` variant; every match on `Node` (all in `core::tex`) gains
  an arm. The print round trip (`print(&parse(s)) == s`) covers it.
- Later algebra names (`\Hom \End …`, T66/T67) can be defined on top of
  the same layout.
- `\DeclareMathOperator` and other starred commands stay out.

## Rollback plan

Revert the T64 `feat` commits: the `OperatorName` variant and its arms in
`Parser::command`, `print_node` and `atom` go away and `\operatorname`
becomes an unknown command (a literal) again. Record the rollback as a
new ADR.
