---
id: ADR-T112-1
title: User macros as source expansion with a document-wide table
status: accepted
kind: decision
date: 2026-11-11
task: T112
builds_on: [ADR-T69-1, ADR-T105-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T112-1 User macros as source expansion with a document-wide table

## Context

`\newcommand` in one block must define a command for every formula of the
drawing. Formula layouts are cached per text (`text_layout::typeset`), the
parser keeps every node's source so `print(parse(text)) == text`
(ADR-T30-1), and `typeset(text)` has no access to the document.

## Decision

- **Expansion is a source-to-source pass before parsing.**
  `tex::expand_macros(text, table)` rewrites each formula's characters:
  definitions are removed (they draw nothing), user commands are replaced
  by their body with `#1`…`#9` filled by the arguments (a `{group}` or one
  token). `parse` and `print` are untouched, so sources round-trip (AC-5).
- **The table is `tex::MacroTable`**, built from the `Shape::Text` blocks
  of the document in stacking order, bottom first, so a higher block wins
  (AC-2); `\renewcommand` reads the same as `\newcommand`. Definitions
  are read only inside formulas. A name that is built in (symbol, accent,
  expansion, environment, arrow, alphabet, kern, infix or a structural
  command) is never defined: the definition is skipped and the name is
  listed in `MacroTable::ignored` (the project has no logger, so nothing
  is printed) (AC-4).
- **`Document` owns the table** and rebuilds it after a transaction that
  adds, removes or replaces a text mentioning `newcommand` (before or
  after), and when built from parts. It then hands a copy to
  `text_layout::set_macros`, a thread-local like the layout cache, because
  `typeset(text)` is called from the shell and the editor with only the
  text. The last document changed on the thread wins; the app has one.
- **Cache validity is per macro.** Each cached layout records the
  user-defined names its formulas mention, directly or through bodies,
  with the hash of their definition (0 when undefined). A hit is valid
  while every recorded hash equals the current one, so a change re-lays
  out only the blocks that use it, and a name defined later is noticed
  (AC-3).
- **Limits:** expansion depth 32, at most 10 000 expansion steps and
  10 000 output characters per formula. Past any of them the formula is
  parsed from its original source, so `\newcommand{\a}{\a\a}` draws
  `\a` literally (AC-4).
- A macro used with fewer arguments than it takes is left as typed.

## Alternatives considered

- **Expand at node level** — arguments and `#n` are already tokens there,
  but nodes must round-trip to source and a body can start a command that
  takes the following text; source expansion is TeX's own model.
- **Pass the table to `typeset`** — touches every caller in shell and
  editor for no gain over one thread-local.
- **One global version in the cache key** — simpler but re-lays out every
  formula when any macro changes, against AC-3.

## Consequences

- A definition typed in the block being edited applies once the edit is
  committed (the table follows the document, not the edit buffer).
- Text inside shapes (rectangles, cells) does not define macros but uses
  them.
- Two documents on one thread share the last table; tests that need
  their own call `set_macros`.

## Rollback plan

Revert the T112 commits; `parse`, `print` and snapshots are unchanged, so
nothing else depends on it.
