---
id: T99
title: Spacing commands
status: todo
wave: 57
branch: task/T99-spacing-commands
depends_on: []
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T99 Spacing commands

## Goal

`\:`, `\;`, `\!`, `\quad`, `\qquad`, `\ ` and `~` space a formula as in LaTeX instead of printing as text.

## Known facts (2026-11-10)

- Only `\,` exists, as the glyph U+2006 of class Ord (`src/core/tex.rs`, ADR-T36-1). The others print literally ([[TeX formulas#Limits]]).
- `\!` is a negative space, so it cannot be a glyph.

## Spec

The spec step names the test functions of each AC.

- **AC-1** — `\:` is 4/18 em, `\;` 5/18 em, `\quad` 1 em, `\qquad` 2 em, `\ ` and `~` one interword space, all scaled in scripts like other spacing.
- **AC-2** — `\!` is −3/18 em: a real negative kern that pulls its neighbours together, never below zero total width of the formula.
- **AC-3** — A spacing command never changes atom classes: `a\;+\;b` keeps the binary spacing of `+` added to the explicit spaces, as TeX does.
- **AC-4** — `\,` keeps its present look (existing tests unchanged). Spacing commands round-trip in the source text and in snapshots unchanged.
- **AC-5** — Formulas made only of spaces, or `\!` repeated, never panic and give finite boxes (fuzzed).

## Out of scope

- `\hspace`, `\kern`, `\mkern`, `\enspace`, `\thinspace`.
- Display mode.

## Files owned

- `src/core/tex.rs`, `src/core/text_layout.rs`
- `docs/features/TeX formulas.md`, `README.md` (formula table), `docs/features/Contest cheat sheet.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T99-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T99): specify spacing commands`
- [ ] tests — `test(T99): add failing tests for spacing commands`
- [ ] models — `feat(T99): …`
- [ ] behaviour — `feat(T99): …`
- [ ] quality — `chore(T99): pass clippy and rustfmt`
- [ ] docs — `docs(T99): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
