---
id: T112
title: User macros
status: in-progress
wave: 60
branch: task/T112-user-macros
depends_on: [T105]
adrs: [ADR-T112-1]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T112 User macros

## Goal

`$\newcommand{\R}{\mathbb{R}}$` in any text block defines `\R` for every formula of the drawing, with up to nine arguments (`\newcommand{\abs}[1]{\left|#1\right|}`).

## Known facts (2026-11-10)

- Formulas have no macros ([[TeX formulas#Limits]]); some commands are built-in expansions (`\choose`, physics shortcuts, ADR-T69-1).
- Formula layouts are cached per block; a macro links blocks, so changing a definition must invalidate the layouts that use it.
- Depends on T105 because both change `src/core/tex.rs`.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `\newcommand{\name}{body}` and `\newcommand{\name}[n]{body}` with `#1`…`#9` define a macro; the definition itself draws nothing.
- **AC-2** — Definitions apply to every formula in the drawing, wherever their block is; when two blocks define the same name, the one higher in the stacking order wins. `\renewcommand` is read the same way.
- **AC-3** — Editing, deleting or undoing a definition re-lays out every formula that uses it, and only those.
- **AC-4** — A built-in command cannot be redefined (the definition is ignored and the log says so). Expansion stops at a fixed depth and size, so `\newcommand{\a}{\a\a}` draws the literal source instead of hanging (fuzzed).
- **AC-5** — Sources round-trip unchanged; snapshots and files keep definitions because they are block text.

### Tests per criterion (ADR-T112-1)

- AC-1 — `tex::tests`: `newcommand_without_args_expands_in_formula`, `newcommand_with_args_fills_slots`, `definition_draws_nothing`.
- AC-2 — `tex::tests`: `table_higher_block_wins`, `renewcommand_defines_like_newcommand`; `document::tests`: `macros_follow_stacking_order`.
- AC-3 — `text_layout::tests`: `typeset_relayouts_only_users_of_changed_macro`, `typeset_notices_macro_defined_later`; `document::tests`: `macros_change_with_undo`.
- AC-4 — `tex::tests`: `builtin_cannot_be_redefined`, `self_recursive_macro_draws_literal_source`, `exponential_macro_is_bounded`; `tests/fuzz.rs`: macro sources never hang or panic.
- AC-5 — `tex::tests`: `print_parse_unchanged_by_macros`; `document::tests`: `snapshot_keeps_definitions`.

## Out of scope

- `\def`, `\let`, optional-argument defaults, `\DeclareMathOperator`.
- Macros in plain text outside formulas.

## Files owned

- `src/core/tex.rs`, `src/core/text_layout.rs`, `src/core/document.rs` (macro table), `src/shell/` (layout cache invalidation)
- `docs/features/TeX formulas.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T112-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T112): specify user macros`
- [ ] tests — `test(T112): add failing tests for user macros`
- [ ] models — `feat(T112): add the macro table and expansion`
- [ ] behaviour — `feat(T112): expand user macros in every formula`
- [ ] quality — `chore(T112): pass clippy and rustfmt`
- [ ] docs — `docs(T112): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from F6 of the [[Feature proposal]].
- Depends on T105 because both change `src/core/tex.rs`.
