---
id: T71
title: Combinatorics notation
status: done
wave: 50
branch: task/T71-combinatorics
depends_on: [T55]
adrs: ["[[ADR-T71-1 Infix fraction commands split their group]]"]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T71 Combinatorics notation

## Goal

Combinatorics notation beyond `\binom`: `\tbinom`/`\dbinom`, the infix
`n \choose k`, Stirling numbers `n \brack k` and `n \brace k`,
multisets `\multiset{n}{k}` (double parentheses), falling and rising
factorials `x^{\underline{k}}` and `x^{\overline{k}}`, `\nmid`,
`\#` and `\lvert A \rvert` for cardinalities.

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
- `\binom` exists with scaled parentheses (`binomial`). `\overline`
  exists (ADR-T39-1). `\underline` does not: the feature note lists it
  under *Limits*.
- No display mode in draw, so `\dbinom` and `\tbinom` would draw like
  `\binom`.
- `\choose`, `\brack`, `\brace` are infix (they split their group in
  two), which the parser does not do yet.

## Proposal (settle in the spec step)

1. `\tbinom`, `\dbinom` as `\binom`. `\multiset` is not standard LaTeX
   (it comes from a common macro): the spec decides whether to add it.
2. Infix `\choose`, `\brack`, `\brace`: the parser splits the enclosing
   group (or the formula) at the command, the way `\over` works in TeX.
   ADR `ADR-T71-1`.
3. `\underline{…}`: a rule under the argument (the mirror of
   `\overline`).
4. `\nmid` (`∤`, `Rel`), `\#` (`#`, `Ord`).

## Spec

Owner decisions (batch plan, 2026-11-04): `\tbinom`/`\dbinom` draw as
`\binom`; infix `\choose` `\brack` `\brace`, **and `\over` `\atop`**,
split their enclosing group (or the formula) as TeX's `\over` does
([[ADR-T71-1 Infix fraction commands split their group]]);
`\multiset{n}{k}` is added (double parentheses); `\underline{…}`
mirrors `\overline`; `\nmid` (`∤` U+2224, `Rel`) and `\#` (`#`, `Ord`).
Two infix commands in one group: the first splits it, the rest are
literals in the warning colour. Every glyph drawn (`( ) [ ] { } # ∤`)
has a `core::math_metrics` row (checked: ASCII rows and `U+2224`).

| Source | Draws |
|--------|-------|
| `{a \over b}` | the ink of `\frac{a}{b}` |
| `{a \atop b}` | the same stack, no bar |
| `{n \choose k}`, `\tbinom{n}{k}`, `\dbinom{n}{k}` | the ink of `\binom{n}{k}` |
| `{n \brack k}` | the `\binom` stack in `[ ]` |
| `{n \brace k}` | the `\binom` stack in `{ }` |
| `\multiset{n}{k}` | the `\binom` stack in `(( ))` |
| `\underline{x}` | a rule under the argument, as wide as it |
| `\nmid`, `\#` | `∤` (`Rel`), `#` (`Ord`) |

- **AC-1** Each of `\choose \brack \brace \over \atop` splits its group:
  `{n \choose k}` is one group holding one `Node::Infix` whose `left`
  is `n` and a space and whose `right` is a space and `k`; it prints
  back unchanged —
  `tex_parse_infix_commands_split_their_group`.
- **AC-2** At the top level an infix command splits the whole formula,
  and inside an argument its argument (`\frac{a \over b}{c}`) —
  `tex_parse_infix_at_top_level_splits_the_formula`,
  `tex_parse_infix_inside_an_argument_splits_the_argument`.
- **AC-3** A second infix command in the same list is a literal in the
  right side, drawn in the warning colour; a nested group splits on its
  own — `tex_parse_second_infix_in_a_group_is_literal`,
  `tex_parse_infix_in_nested_groups_splits_each`.
- **AC-4** A script right after the infix command has no base (the left
  side's last atom does not take it) —
  `tex_parse_script_after_infix_has_no_base`.
- **AC-5** `{a \over b}` draws the same ink and box as `\frac{a}{b}`;
  `{a \atop b}` the same glyphs with no rule; `{n \choose k}`,
  `\tbinom{n}{k}` and `\dbinom{n}{k}` the same ink and box as
  `\binom{n}{k}` — `tex_layout_over_matches_frac`,
  `tex_layout_atop_is_frac_without_bar`,
  `tex_layout_choose_and_tbinom_dbinom_match_binom`.
- **AC-6** `{n \brack k}` and `{n \brace k}` draw `[`…`]` and `{`…`}`
  of the math face around the stack (scaled above 1, both the same),
  no rule, the box from the opening bracket to the closing one's
  advance — `tex_layout_brack_and_brace_use_their_brackets`.
- **AC-7** `\multiset{n}{k}` draws two `(` before and two `)` after the
  stack, the box as wide as all four plus the stack, and it prints
  back; with one argument it is a literal —
  `tex_layout_multiset_has_double_parentheses`,
  `tex_parse_multiset_without_two_arguments_is_literal`.
- **AC-8** `\underline{arg}` draws one rule from 0 to the argument's
  width, below the argument's descent, keeps width, ascent and italic,
  and the box's descent reaches the rule; `x^{\underline{k}}` has the
  rule in the superscript — `tex_layout_underline_spans_its_argument_below`,
  `tex_layout_falling_factorial_underlines_the_superscript`.
- **AC-9** `\nmid` and `\#` are symbols with that text and class,
  drawn from the math face — `tex_parse_nmid_and_hash_have_their_text_and_class`.
- **AC-10** The fuzzer types every new command (`tests/fuzz.rs`); the
  existing print round trip property covers the new node.

Implementation notes: `\underline` is a one-argument command laid out
by its own function, not a row of `ACCENTS` (the accent tests assume a
mark above the argument). `\multiset` puts the two parentheses of each
side next to each other at the scale `\binom` gives them.

## Out of scope

- `\above`, `\abovewithdelims`, `\atopwithdelims`, `\overwithdelims`,
  `\genfrac`, `\underbrace`.
- Generating-function notation beyond what is already there
  (`[x^n] F(x)` needs nothing new).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T71-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T71): …`
- [x] tests — `test(T71): …`
- [x] models — `feat(T71): …`
- [x] behaviour — `feat(T71): …`
- [x] quality — `chore(T71): …` (rustfmt only)
- [x] docs — `docs(T71): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — implemented (batch run). `Node::Infix` and the `INFIX`
  table; `Parser::list` splits at the first infix command, no lexer
  change, no existing test changed (ADR-T71-1). `binomial` now goes
  through a general `delimited` (brackets side by side), used by
  `\brack`, `\brace` and `\multiset`; `\underline` is its own
  one-argument command, not an `ACCENTS` row. `scripts/check.sh` green.
- 2026-11-04 — UNATTENDED CHOICE: two infix commands in one group — the
  first splits it, the rest are literals in the warning colour (plan
  default). `\multiset`'s two parentheses each side touch (no negative
  kern), at the size `\binom` gives its own.
- 2026-11-04 — `scripts/check.sh --fuzz` found a failure **outside
  T71**: `snapshot_round_trip_continues_equal` — a text-tool session
  with Ctrl+[ and a zoom scroll (no formula) remembers a text size of
  320 px that the snapshot reader rejects ("text size out of range",
  `core::tools::text` / `core::snapshot`, files not owned here). The
  new proptest seed was stashed, not committed
  (`git stash list`: `batch-T71-2026-11-04 fuzz seed: snapshot text size
  out of range`) so the regular gates stay green; it needs a `fix/`
  task. Manual checks for the owner: `${n \choose k}$`,
  `${n \brace k}$`, `$x^{\underline{k}}$`.
- 2026-11-04 — integrated on `main` (merge `3c5fa88`), no conflicts; the owner's manual checks and review of the unattended choices are still open.
