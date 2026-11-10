---
id: T103
title: Labelled extensible arrows
status: todo
wave: 58
branch: task/T103-labelled-arrows
depends_on: [T99]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T103 Labelled extensible arrows

## Goal

`\xrightarrow{f}` and its relatives draw an arrow that stretches to fit a label above it, and an optional `[below]` label under it.

## Known facts (2026-11-10)

- Arrows are fixed-size glyphs; `\xrightarrow` prints literally ([[TeX formulas#Limits]]).
- Fractions already draw rules; scripts already set text in script size.
- Branches from T99 only because both edit `src/core/tex.rs`.

## Spec

The spec step names the test functions of each AC.

- **AC-1** (owner) — `\xrightarrow`, `\xleftarrow`, `\xleftrightarrow`, `\xRightarrow`, `\xLeftarrow`, `\xLeftrightarrow`, `\xmapsto` and `\xhookrightarrow` take a required `{above}` and an optional `[below]` argument: `\xrightarrow[n\to\infty]{f}`.
- **AC-2** — Labels are set in script style, centred above / below the arrow; the arrow is at least as long as the fixed arrow and grows to the wider label plus padding on both sides.
- **AC-3** — The arrow is a relation (`Rel` spacing) sitting on the math axis like `\rightarrow`.
- **AC-4** — Empty labels `\xrightarrow{}` give a plain arrow of minimum length; a missing `{…}` is invalid input shown in red, as other malformed commands are.
- **AC-5** — Fuzzed; boxes stay finite, nested labels (`\xrightarrow{\xrightarrow{a}}`) work.

## Out of scope

- `\overset`, `\underset`, `\stackrel`, `\overrightarrow`.
- Commutative diagrams.

## Files owned

- `src/core/tex.rs`, `src/core/text_layout.rs`, `src/shell/render.rs` (only if a rule needs drawing support)
- `docs/features/TeX formulas.md`, `README.md` (formula table), `docs/features/Contest cheat sheet.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T103-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T103): specify labelled extensible arrows`
- [ ] tests — `test(T103): add failing tests for labelled extensible arrows`
- [ ] models — `feat(T103): …`
- [ ] behaviour — `feat(T103): …`
- [ ] quality — `chore(T103): pass clippy and rustfmt`
- [ ] docs — `docs(T103): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
