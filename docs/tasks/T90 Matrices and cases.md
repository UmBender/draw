---
id: T90
title: Matrices and cases
status: todo
wave: 52
branch: task/T90-matrices
depends_on: [T89]
adrs: [ADR-T90-1]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T90 Matrices and cases

## Goal

Formulas can contain matrices and piecewise definitions. (idea F2 of [[Feature proposal]])

## Known facts (2026-11-09)

- A formula is one line of source; the box layout already stacks fractions.

## Spec

The spec step names the test for each criterion.

- **AC-1** — `matrix`, `pmatrix`, `bmatrix`, `vmatrix` and `cases` environments parse, with `&` between cells and `\\` between rows.
- **AC-2** — Matrix columns are centred; `cases` columns are left-aligned with one growing brace.
- **AC-3** — The block is vertically centred on the math axis, and brackets use T89's scaling.
- **AC-4** — An unknown environment or missing `\end` is drawn in red and never panics.
- **AC-5** — Rows of different lengths leave empty cells.

## Out of scope

- Source spread over several text lines.
- `array` with a column spec.

## Files owned

- `src/core/tex.rs`
- `docs/features/TeX formulas.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T90-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T90): specify matrices and cases`
- [ ] tests — `test(T90): add failing tests`
- [ ] models — `feat(T90): add types`
- [ ] behaviour — `feat(T90): implement`
- [ ] quality — `chore(T90): pass clippy and rustfmt`
- [ ] docs — `docs(T90): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
