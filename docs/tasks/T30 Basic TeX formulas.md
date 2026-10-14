---
id: T30
title: Basic TeX formulas
status: todo
wave: 16
branch: task/T30-tex
depends_on: [T29]
adrs: []
feature: "[[TeX formulas]]"
tutorial: "[[30 Parsing and laying out a small TeX subset]]"
tags: [task]
---

# T30 Basic TeX formulas

## Goal

Inside a text block, `$…$` is a formula written in a small TeX subset —
enough for competitive programming notes such as
`$dp_{i,j} = \min(dp_{i-1,j}, dp_{i,j-1}) + a_{i,j}$`,
`$O(n \log n)$`, `$\sum_{k=1}^{n} k^2$`, `$\frac{n(n+1)}{2}$`. The block
shows the source while edited and the typeset formula once committed.
Written in-house in `core` (no TeX engine, no new dependency), total on
every input and fuzzed.

## Supported subset (settle in the spec step)

- Superscript `^` and subscript `_`, single character or `{group}`,
  both on one base (`x_i^2`).
- `\frac{a}{b}`, `\sqrt{x}`, `\binom{n}{k}`.
- Greek letters `\alpha` … `\omega`, `\Gamma` … `\Omega`.
- Relations and operators: `\le \ge \ne \approx \equiv \cdot \times
  \div \pm \to \leftarrow \Rightarrow \iff \in \notin \subset \cup \cap
  \land \lor \neg \oplus \infty \ldots \cdots`.
- Big operators with limits: `\sum \prod \min \max \lim`, and functions
  in upright type: `\log \ln \gcd \lcm \bmod \pmod`.
- Brackets: `\lfloor \rfloor \lceil \rceil \{ \} |`, and `\text{…}`.
- `\$` for a literal dollar sign outside a formula.

Anything else is drawn as its literal source in a warning colour, never
an error and never a panic.

## Spec

To be refined in the spec step; ADR-T30-1 records the subset, the
layout model (boxes with width, ascent, descent; script scale and
shift) and the metrics source.

- **AC-1 — Parse.** `core::tex::parse` turns any string into a tree;
  unknown commands and unbalanced braces become literal nodes. Unit
  tests per construct; proptest: never panics, and printing the tree
  gives back the source.
- **AC-2 — Layout.** `core::tex::layout` turns the tree into positioned
  glyphs and rules (fraction bars, root signs) with total bounds, using
  the T29 font metrics. Tests on positions for scripts, fractions,
  roots and limits; proptest: bounds contain every glyph.
- **AC-3 — In text blocks.** Committed blocks render `$…$` typeset,
  mixed with plain text on the same line; the block's bounds, hit-test
  and snapping use the typeset size. Editing shows the source.
- **AC-4 — Font coverage.** Every symbol of the subset has a glyph in the
  embedded font (test over the symbol table); if not, ADR-T30-1 adds a
  second embedded math font and records the size cost.
- **AC-5 — Budgets.** Parse + layout cached per block (only when its text
  changes); 200 blocks with formulas stay within
  [[ADR-T24-5 Redraw budgets]]; perf test in `tests/perf.rs`.
- **AC-6 — Fuzz.** The T13 harness generates formula text; invariants
  hold.
- **AC-7 — Docs.** README section with the supported commands and an
  example. At integration the integrator adds the feature under
  `[Unreleased]` → `Added` in `CHANGELOG.md`.

## Out of scope

- Display-mode `$$…$$`, matrices, `\begin{…}` environments, macros
  (`\newcommand`), alignment, accents (`\hat`, `\bar`) — candidates for
  a follow-up if missed in practice.
- Rendering through an external TeX engine or a math crate.

## Files owned

- `src/core/tex.rs` (new, or `src/core/tex/` module), `src/core/mod.rs`,
  `src/core/text_layout.rs`
- `src/shell/render.rs`
- `assets/fonts/*` (only if AC-4 needs a math font)
- `tests/fuzz.rs`, `tests/perf.rs`, `README.md`
- new ADRs `ADR-T30-*`, feature note `docs/features/TeX formulas.md`,
  tutorial `docs/tutorials/30 Parsing and laying out a small TeX subset.md`

## Subtasks (one commit each)

- [ ] spec — `docs(T30): …`
- [ ] tests — `test(T30): …`
- [ ] models — `feat(T30): …`
- [ ] behaviour — `feat(T30): …`
- [ ] quality — `chore(T30): …`
- [ ] docs — `docs(T30): …`

## Learning path

Step 30 — requires step 29 (text blocks).

## Log

- 2026-10-14 — Created from the owner's proposal; depends on the text
  blocks and font of T29.
