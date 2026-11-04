---
id: ADR-T54-1
title: Math alphabets as a table of per-kind mappings
status: accepted
kind: decision
date: 2026-11-04
task: T54
builds_on: ["[[ADR-T30-1 TeX subset, box layout and metrics]]"]
amends: ["[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, tex]
---

# ADR-T54-1 Math alphabets as a table of per-kind mappings

## Context

[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]
maps formula letters with a private enum `Style { Italic, Calligraphic }`
and two functions, `italic` and `calligraphic`. [[T54 Math alphabet styles]]
adds `\mathrm`, `\mathit` and `\mathnormal`, and T56–T60 will add bold,
blackboard, fraktur, script, sans-serif and monospace. Every one of them
is a *math alphabet*: it replaces letters (and, for some, digits and
Greek) by characters of the Unicode Mathematical Alphanumeric Symbols
block, which lays out every alphabet the same way — 26 capitals, 26
lowercase, 25 Greek capitals, 25 Greek lowercase plus six variants, 10
digits — with a few holes filled from Letterlike Symbols (`ℎ`, `ℬ`,
`ℂ`, …). In LaTeX the innermost alphabet wins, and an alphabet that does
not cover a character (`\mathcal{a}`, `\mathrm{\alpha}`) leaves it as
the enclosing style draws it.

## Decision

- **Table.** `ALPHABETS: &[AlphabetDef]` has one row per command:
  `name`, one `Map` per `Kind` (`upper`, `lower`, `digit`, `greek_upper`,
  `greek_lower`) and `holes: &[(char, char)]`. A `Map` is plain data:
  - `Inherit` — the enclosing alphabet decides;
  - `Upright` — the character as typed;
  - `Block(start)` — the code point `start + index`, where `index` is the
    character's position in its kind (`A`→0 … `Z`→25; `α`→0 … `ω`→24,
    `ϵ ϑ ϰ ϕ ϱ ϖ` → 26–31, the Unicode order; `Α`→0 … `Ω`→24, skipping
    the unassigned U+03A2).
  A hole replaces the block character of one source character (`h` → ℎ
  in italic, `B` → ℬ in `\mathcal`) when the row maps that kind.
  Enum + data was chosen over a function pointer per row: rows stay
  `const`, comparable and printable in tests, and a new alphabet is one
  row of numbers.
- **Root.** A formula starts in `MATH_ITALIC` (not a command), which maps
  every kind: italic Latin and lowercase Greek, upright digits and Greek
  capitals — exactly what `italic` gave before.
- **Nesting.** The current style is a `Style` value holding, per kind,
  the row that decides it. Entering `\mathX{…}` replaces the kinds that
  row maps (`Map` other than `Inherit`) and keeps the others. So the
  innermost alphabet that maps a kind wins, a missing mapping falls back
  to the enclosing alphabet and finally to `MATH_ITALIC`. `Style` is
  `Copy` and fixed size: no allocation, no stack.
- **Reach.** An alphabet maps letters and digits typed in the formula
  and single-character `Ord` symbols (`\alpha`, `\Gamma`). Operators,
  functions, `\operatorname`, `\text`, and characters of no kind
  (Hebrew, `∂`, `∞`) are never changed.
- **Commands.** `\mathcal`, `\mathrm`, `\mathit`, `\mathnormal` take one
  argument (braced or one token); without one they are literals. An
  alphabet's argument is an `Ord` atom laid out as a group: letters sit
  side by side, so `\mathit{diff}` is one word, and `\mathrm{d}` has the
  upright glyph's italic correction, which is zero.
- `\mathrm` maps Latin and digits only (Greek inherits, as in LaTeX);
  `\mathit` maps Latin and Greek, not digits; `\mathnormal` is
  `MATH_ITALIC` with italic Greek capitals (U+1D6E2–1D6FA).
- `italic(ch)` and `calligraphic(ch)` stay public and are now thin
  wrappers over the table.

## Alternatives considered

- **A function pointer per row** (`fn(char) -> Option<char>`) — flexible,
  but every new alphabet repeats the offset arithmetic, rows cannot be
  compared, and the regular block layout is hidden.
- **Growing the `Style` enum** — one more `match` arm per alphabet and per
  kind in code instead of data; nesting would need a stack of enums.
- **A stack of active alphabets searched innermost first** — same result
  as the per-kind `Style`, but allocates and searches on every letter.
- **Mapping `∂` and `∇` too** (they sit in each block) — would change
  every existing formula with `\partial`/`\nabla`; left out.

## Consequences

- T56–T60 add rows only (`Block` starts and holes), plus the glyphs in
  the font subset and `math_metrics`.
- Output of every existing formula is unchanged (the existing tests are
  the guard); `\mathcal` keeps its behaviour, now as a table row.
- An accent skews over a letter whenever the current alphabet maps it to
  a different character, so `\hat{\mathrm{x}}` is centred and
  `\hat{\mathnormal{\Gamma}}` is skewed.

## Rollback plan

Revert the T54 `feat` commits: `Style` goes back to the two-variant enum
and `\mathrm`, `\mathit`, `\mathnormal` become unknown commands (literals)
again. Record the rollback as a new ADR.
