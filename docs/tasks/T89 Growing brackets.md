---
id: T89
title: Growing brackets
status: todo
wave: 51
branch: task/T89-growing-brackets
depends_on: []
adrs: [ADR-T89-1]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T89 Growing brackets

## Goal

`\left` and `\right` draw brackets that grow to fit what they enclose. (idea F1 of [[Feature proposal]])

## Known facts (2026-11-09)

- `delimited` in `src/core/tex.rs` already scales a bracket glyph for `\binom`, up to `BINOM_MAX_SCALE`.
- The font subset has no size variants, so glyphs are scaled as `\binom` does.

## Spec

The spec step names the test for each criterion.

- **AC-1** — `\left( … \right)` scales both brackets to the enclosed height, for `( ) [ ] \{ \} | \| \langle \rangle \lfloor \rfloor \lceil \rceil`.
- **AC-2** — `.` draws nothing on that side.
- **AC-3** — An unmatched `\left` or `\right` is drawn in red, as other invalid input is, and never panics.
- **AC-4** — Pairs nest, and scripts on the closing bracket attach to it.
- **AC-5** — Short content leaves the brackets at normal size.

## Out of scope

- `\middle`.
- Adding size variants to the font.

## Files owned

- `src/core/tex.rs`
- `docs/features/TeX formulas.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T89-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T89): specify growing brackets`
- [ ] tests — `test(T89): add failing tests`
- [ ] models — `feat(T89): add types`
- [ ] behaviour — `feat(T89): implement`
- [ ] quality — `chore(T89): pass clippy and rustfmt`
- [ ] docs — `docs(T89): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
