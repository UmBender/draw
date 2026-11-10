---
id: T99
title: Spacing commands
status: in-progress
wave: 57
branch: task/T99-spacing-commands
depends_on: []
adrs: ["[[ADR-T99-1 Spacing commands as kerns outside the atom list]]"]
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

Design: [[ADR-T99-1 Spacing commands as kerns outside the atom list]].

- **AC-1** — `\:` is 4/18 em, `\;` 5/18 em, `\quad` 1 em, `\qquad` 2 em, `\ ` and `~` one interword space (1/3 em, `INTERWORD_SPACE`), all scaled in scripts like other spacing.
  Tests: `tex_parse_spacing_commands_are_kerns`, `tex_layout_spacing_commands_have_their_widths`, `tex_layout_spacing_commands_scale_in_scripts`.
- **AC-2** — `\!` is −3/18 em: a real negative kern that pulls its neighbours together, never below zero total width of the formula.
  Tests: `tex_layout_negative_space_pulls_neighbours_together`, `tex_layout_negative_space_never_makes_width_negative`.
- **AC-3** — A spacing command never changes atom classes: `a\;+\;b` keeps the binary spacing of `+` added to the explicit spaces, as TeX does.
  Tests: `tex_layout_spacing_commands_keep_atom_classes`.
- **AC-4** — `\,` keeps its present look (existing tests unchanged). Spacing commands round-trip in the source text and in snapshots unchanged.
  Tests: existing `tex_layout_thin_space_is_a_sixth_of_an_em`, `tex_parse_spacing_commands_print_back`, the proptest `tex_parse_never_panics_and_prints_back_the_source` (with spacing tokens added). Snapshots store the block's source text, so the source round trip covers them.
- **AC-5** — Formulas made only of spaces, or `\!` repeated, never panic and give finite boxes (fuzzed).
  Tests: `tex_layout_only_spacing_commands_is_finite`, the proptests `tex_layout_is_finite` and `tex_layout_bounds_contain_every_ink` and `tests/fuzz.rs` with the spacing tokens added.

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
- 2026-11-10 — UNATTENDED CHOICE: the interword space of `\ ` and `~` is 1/3 em (Computer Modern's), and `\,` stays the U+2006 glyph; see ADR-T99-1.
- 2026-11-10 — UNATTENDED CHOICE: the existing test `tex_parse_unknown_command_is_literal` used `\;` as its unknown control symbol; it now uses `\@`, which stays outside the subset.
