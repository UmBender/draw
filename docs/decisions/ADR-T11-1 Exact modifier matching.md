---
id: ADR-T11-1
title: Exact modifier matching
status: accepted
kind: decision
date: 2026-10-01
task: T11
builds_on: [ADR-0003]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T11-1 Exact modifier matching

## Context

[[Keymap]] binds bare keys (`C` ellipse, `Backspace` delete) and the same keys
with modifiers (`Ctrl+C` copy, `Ctrl+Backspace` clear canvas). `Shift` is also
a gesture modifier (constrain while drawing), so it is often held when a key
is pressed. The keymap must decide what a chord with extra modifiers means.

## Decision

A binding fires only when the held modifiers (`shift`, `ctrl`, `alt`) are
**exactly** the binding's modifiers. Bindings live in one `const` table,
`keymap::BINDINGS`, and `resolve` is a linear lookup in it. Chords not in the
table resolve to `None`.

## Alternatives considered

- **Ignore extra modifiers on bare keys** (`Shift+R` → Rectangle) — hides
  mistakes such as `Ctrl+Shift+C` silently becoming `C`, and makes the
  precedence between `Ctrl+Z` and `Ctrl+Shift+Z` depend on lookup order.
- **A `match` instead of a table** — fast, but the toolbar and docs could not
  list the bindings without a second, drifting copy.

## Consequences

- Every chord has one meaning; tests can enumerate all key × modifier
  combinations.
- Pressing a tool key while holding `Shift` does nothing; users release
  `Shift` first.
- Lookup is linear over ~35 rows, negligible next to a frame.

## Rollback plan

Change the comparison in `keymap::resolve` (single function) and add a new ADR
that supersedes this one; update `modifiers_disambiguate`.
