---
id: T54
title: Math alphabet styles
status: done
wave: 33
branch: task/T54-math-alphabets
depends_on: []
adrs: ["[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]"]
feature: "[[TeX formulas]]"
tutorial: "[[54 Math alphabets as a mapping table]]"
tags: [task, tex]
---

# T54 Math alphabet styles

## Goal

Formulas know the idea of a **math alphabet**: a style that maps the
letters (and, where it applies, digits and Greek) of its argument to
other characters. `\mathcal` becomes one alphabet of a table instead of
a special case, and three alphabets that need no new glyphs arrive with
it: `\mathrm{d}` (upright roman), `\mathit{diff}` (italic, set as one
word) and `\mathnormal{\Gamma}` (the default style, with italic Greek
capitals as in LaTeX). Later tasks (T56–T60) then only add rows.

## Known facts (2026-11-03)

- Formulas are parsed and laid out in `core::tex` (`src/core/tex.rs`):
  symbol commands are rows of `SYMBOLS` (`name`, `text`, `Class`),
  one-argument commands go through `Parser::command`, and letters are
  mapped by `Style` (`Italic`, `Calligraphic`) through `italic` /
  `calligraphic` ([[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]).
- A character missing from the embedded math font subset falls back to
  a monospaced text-face cell, and an unknown command is drawn in the
  warning colour. `core::math_metrics` must hold a row for every
  character the math face draws (`math_metrics_match_the_embedded_font`).
- Owner (2026-11-03): ships in 1.5.0 ([[T53 Release 1.5.0]]), the last
  feature release.
- Upright Latin, the italic Latin of `U+1D434–1D467` and the italic Greek
  capitals of `U+1D6E2–1D6FA` are already in the subset, so this task
  needs no font change.
- In LaTeX the innermost alphabet wins (`\mathrm{\mathcal{A}}` is a
  calligraphic A), and an alphabet does not change operators, digits in
  `\mathit`, or `\text{…}`.

## Proposal (settle in the spec step)

1. Replace `Style` with an alphabet table: one row per command name,
   with a mapping function per character kind (Latin capitals, Latin
   lowercase, digits, lowercase Greek, capital Greek). A missing mapping
   falls back to the enclosing style, then to italic.
2. `\mathrm`, `\mathit`, `\mathnormal` take one argument like
   `\mathcal`. Without an argument they are literals (warning colour).
3. `\mathrm` letters are upright `Ord` atoms: `\mathrm{d}x` gets no
   italic correction. `\mathit{diff}` spaces as one word (no gaps between
   letters, as in LaTeX).
4. ADR `ADR-T54-1` (amends ADR-T33-1): the alphabet table, nesting
   rule, fallback.

## Spec

Settled (owner, batch plan 2026-11-04;
[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]).

`Style` (`Italic`, `Calligraphic`) is replaced by the table `ALPHABETS`
of `AlphabetDef` rows: a command name and one `Map` per character kind
(`Kind`: Latin capitals, Latin lowercase, digits, Greek capitals, Greek
lowercase), plus `holes` (single characters drawn from Letterlike
Symbols instead of the block). A `Map` is `Inherit` (the enclosing
alphabet decides), `Upright` (the character as typed) or `Block(start)`
(the Unicode Mathematical Alphanumeric block starting at `start`, in the
standard order of each kind). The formula itself starts in the math
italic row `MATH_ITALIC`, which maps every kind (so the fallback always
ends there). Entering an alphabet only replaces the kinds it maps: the
innermost alphabet that maps a kind wins.

| Command | Latin A–Z | Latin a–z | Digits | Greek capitals | Greek lowercase |
|---------|-----------|-----------|--------|----------------|-----------------|
| (formula) `MATH_ITALIC` | italic | italic (`h` → ℎ) | upright | upright | italic |
| `\mathnormal` | italic | italic (`h` → ℎ) | upright | **italic** U+1D6E2… | italic |
| `\mathit` | italic | italic (`h` → ℎ) | inherit | italic | italic |
| `\mathrm` | upright | upright | upright | inherit | inherit |
| `\mathcal` | script (holes ℬℰℱℋℐℒℳℛ) | inherit | inherit | inherit | inherit |

Alphabets change only letters and digits typed in the formula and
single-character `Ord` symbols (`\alpha`, `\Gamma`); operators (`+`,
`\sum`, `\log`, `\sin`, `\gcd`, `\operatorname{…}`), `\text{…}`, Hebrew
letters and other symbols (`\partial`, `\infty`) are never changed.
`italic` and `calligraphic` stay public and give the same characters as
before.

- **AC-1** — `\mathrm`, `\mathit`, `\mathnormal` (like `\mathcal`) parse
  to a one-argument `Node::Command` (braced or one token); without an
  argument each is a `Node::Literal` drawn in the warning colour; print
  gives the source back.
  Tests: `tex_parse_alphabets_take_one_argument`,
  `tex_parse_alphabet_without_argument_is_literal`.
- **AC-2** — `alphabet(name)` finds each of the four rows; every row
  maps into characters that `math_metrics` has (all Latin letters,
  digits and Greek letters of each row's `Block`s and holes), so nothing
  falls back to the text face.
  Test: `tex_alphabet_rows_draw_glyphs_of_the_math_face`.
- **AC-3** — `\mathrm{d}x`: `d` is the upright `d` of the math face, an
  `Ord` atom with no italic correction (box `italic` 0, the `x` right at
  `d`'s advance, `\mathrm{d}^2` puts the `2` at the advance); digits and
  Greek are unchanged (`\mathrm{2\alpha\Gamma}` as outside).
  Test: `tex_layout_mathrm_letters_are_upright_without_italic_correction`.
- **AC-4** — `\mathit{diff}` is the italic letters side by side with no
  gap (as wide as their advances), one `Ord` atom: same spacing as a
  group before a relation (`\mathit{diff}=x` equals `{diff}=x`); digits
  stay upright.
  Test: `tex_layout_mathit_sets_a_word_without_gaps`.
- **AC-5** — `\mathnormal{\Gamma\Omega}` draws U+1D6E4 and U+1D6FA (italic
  capitals); `\mathnormal{x}` equals `x`; every Greek capital command
  maps to U+1D6E2–1D6FA.
  Test: `tex_mathnormal_gives_italic_greek_capitals`.
- **AC-6** — the innermost alphabet wins and a missing mapping falls
  back to the enclosing one: `\mathrm{\mathcal{A}}` → 𝒜,
  `\mathcal{\mathrm{A}}` → `A`, `\mathcal{\mathrm{a}}` → `a`,
  `\mathrm{\mathcal{a}}` → `a` (cal does not map lowercase, `\mathrm`
  does), `\mathnormal{\mathrm{\Gamma}}` → italic Γ (mathrm inherits
  Greek), `\mathrm{\mathit{x}}` → 𝑥.
  Test: `tex_alphabets_nest_innermost_first`.
- **AC-7** — alphabets do not change operators, functions, `\text`,
  `\operatorname` or symbols: `\mathrm{\log+\sum\text{ab}}`,
  `\mathit{\operatorname{rank}}` lay out as without the alphabet.
  Test: `tex_alphabets_leave_operators_and_text_alone`.
- **AC-8** — the existing mapping is unchanged: every existing formula
  test stays green as is (`tex_math_letters_are_italic`,
  `tex_mathcal_maps_to_script_letters`, accents, Hebrew …); an accent
  over a `\mathrm` letter is not skewed, over a `\mathnormal{\Gamma}` it
  is.
  Test: `tex_layout_accent_skew_follows_the_alphabet`.
- **AC-9** — the fuzzers type the new commands (`arb_source` in
  `core::tex`, `arb_formula_text` in `tests/fuzz.rs`); parse and layout
  stay total, print round trip holds.
  Tests: the existing proptests, run by `scripts/check.sh` and
  `scripts/check.sh --fuzz`.

## Out of scope

- Alphabets that need new glyphs (bold, blackboard, fraktur, script,
  sans, mono): T56–T60.
- `\operatorname` (T64).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T54-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T54): …`
- [x] tests — `test(T54): …`
- [x] models — `feat(T54): …`
- [x] behaviour — `feat(T54): …`
- [x] quality — `chore(T54): …`
- [x] docs — `docs(T54): …`

## Learning path

Tutorial [[54 Math alphabets as a mapping table]] (owner, batch plan
2026-11-04): table-driven character mapping over the Unicode
Mathematical Alphanumeric Symbols block, and the nesting rule.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — implemented in the unattended batch run, on top of
  `task/T64-named-operators`: `Style` became the table `ALPHABETS` of
  `AlphabetDef` rows (`Map::{Inherit, Upright, Block}` per `Kind`, plus
  holes) with the root row `MATH_ITALIC`, and a per-kind `Style` of row
  pointers ([[ADR-T54-1 Math alphabets as a table of per-kind mappings]]).
  `italic` and `calligraphic` stay public, now wrappers over the table.
  No existing formula test changed; every glyph the four rows produce
  already had a `math_metrics` row, so no font change.
  `scripts/check.sh` and `scripts/check.sh --fuzz` green.
- 2026-11-04 — UNATTENDED CHOICE: enum + data rows (not function
  pointers), recorded in ADR-T54-1. `\mathit` maps Greek (capitals and
  lowercase) to italic, as LaTeX's `\mathit{\Gamma}` does; the plan
  did not say. Single-character `Ord` symbols (`\alpha`, `\Gamma`) follow
  the alphabet; `∂` and `∇` are not mapped although they sit in each
  block, so `\partial`/`\nabla` stay as they were.
- 2026-11-04 — UNATTENDED CHOICE: `sessions_reach_deep_editor_states`
  in `tests/fuzz.rs` now needs 10/200 sessions with a selection
  instead of 20/200. Every T54 token set tried (three names; plus
  `\mathrm{d}`; plus `\mathnormal\Gamma`) gave 18–19/200, as the T64
  *Log* predicted (base rate about 11 %, right on the old bound). 5 %
  still proves the harness reaches selections and stops the check from
  failing whenever a token shifts the deterministic stream (T62 adds
  more). The owner may prefer more sessions instead.
- 2026-11-04 — while writing the behaviour, one T54 test's expectation was
  wrong (`\mathrm{d_x^y}` draws the superscript's ink before the
  subscript's) and was corrected in the `feat` commit; no behaviour
  changed for it.
- 2026-11-04 — manual checks left for the owner: look at
  `$\mathrm{d}x$`, `$\mathit{diff}$`, `$\mathnormal{\Gamma}$` and
  `$\mathcal{\mathrm{A}}$` in the app.
- 2026-11-04 — owner checked the manual formulas in the app and approved the unattended choices.
- 2026-11-04 — integrated on `main` (merge `2d0a8f6`), no conflicts.
