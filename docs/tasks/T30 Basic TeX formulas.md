---
id: T30
title: Basic TeX formulas
status: review
wave: 17
branch: task/T30-tex
depends_on: [T29, T32]
adrs: ["[[ADR-T30-1 TeX subset, box layout and metrics]]"]
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

[[ADR-T30-1 TeX subset, box layout and metrics]] records the subset, the
parse tree, the layout model (boxes with width, ascent, descent; script
scale and shift), the metrics source and the per-text cache.

Summary of the subset as specified (details in the ADR):

- A formula is `$…$` on one line, non-empty, ended by the first
  unescaped `$`; a `$` with no closing `$` on its line, or `$$`, is plain
  text. `\$` is a literal `$` inside and outside formulas.
- Inside a formula: `^`/`_` with a single character, a symbol command or a
  `{group}`, both on one base in either order; `\frac`, `\sqrt`,
  `\binom` (braced or single-token arguments), `\pmod{…}`, `\text{…}`;
  the symbol table of the goal (plus aliases `\leq \geq \neq`); `\{`,
  `\}`, `\$`; whitespace is ignored as in TeX; `-` is drawn as `−`.
- Anything else — unknown commands, unmatched braces, a script or command
  with a missing argument, a second superscript/subscript on one base —
  is a literal node drawn as its source in a warning colour.

Acceptance criteria and the tests that prove them:

- **AC-1 — Parse.** `core::tex::parse` turns any string into
  `Vec<Segment>`; `core::tex::print` gives the source back.
  Tests (`src/core/tex.rs`): `tex_parse_plain_text_is_one_text_segment`,
  `tex_parse_dollar_pair_is_a_formula`,
  `tex_parse_escaped_dollar_is_a_dollar_segment`,
  `tex_parse_unclosed_or_empty_dollar_is_text`,
  `tex_parse_formula_does_not_cross_a_line_break`,
  `tex_parse_scripts_bind_to_the_previous_atom`,
  `tex_parse_script_takes_one_token_or_a_group`,
  `tex_parse_commands_take_braced_or_single_token_arguments`,
  `tex_parse_symbols_come_from_the_table`,
  `tex_parse_text_command_keeps_its_raw_body`,
  `tex_parse_unknown_command_is_literal`,
  `tex_parse_unbalanced_braces_are_literal`,
  `tex_parse_missing_argument_is_literal`,
  `tex_parse_double_superscript_is_literal`; proptest
  `tex_parse_never_panics_and_prints_back_the_source`.
- **AC-2 — Layout.** `core::tex::layout` turns segments into a `Layout`:
  positioned glyphs (char, baseline-left position, scale, warning flag) and
  rules (segments with a width) in em units from the block's top-left, with
  its width and height; `core::tex::layout_math` lays out one formula as a
  `MathBox` (width, ascent, descent). Metrics: the T29 cell (`ADVANCE`
  0.6 em) and the font's math constants of ADR-T30-1.
  Tests (`src/core/tex.rs`): `tex_layout_char_is_one_cell`,
  `tex_layout_superscript_is_raised_and_scaled`,
  `tex_layout_subscript_is_lowered_and_scaled`,
  `tex_layout_sub_and_sup_share_the_base`,
  `tex_layout_frac_centres_parts_around_a_bar_on_the_axis`,
  `tex_layout_sqrt_draws_a_root_sign_over_the_content`,
  `tex_layout_binom_has_parens_and_no_bar`,
  `tex_layout_sum_limits_are_centred_above_and_below`,
  `tex_layout_relations_and_binary_operators_get_space`,
  `tex_layout_minus_is_the_math_minus`,
  `tex_layout_literal_is_warned`,
  `tex_layout_plain_text_matches_the_cell_layout`,
  `tex_layout_tall_formula_grows_its_line`,
  `tex_layout_mixes_text_and_formula_on_one_line`; proptests
  `tex_layout_bounds_contain_every_ink` and `tex_layout_is_finite`.
- **AC-3 — In text blocks.** `text_layout::bounds` (and so
  `Shape::bounds`, hit-test, culling and snapping) uses the typeset size
  when the text has a `$`; `text_layout::source_bounds` is the T29 cell
  box, used by the edit session. The renderer draws committed blocks
  typeset, the open block as source.
  Tests: `layout_bounds_use_the_typeset_size_with_formulas`,
  `layout_source_bounds_ignore_formulas`,
  `layout_text_shape_hits_and_snaps_on_the_typeset_box`
  (`src/core/text_layout.rs`); `text_edit_bounds_are_the_source_cells`
  (`src/core/tools/text.rs`);
  `push_text_block_formula_draws_glyphs_and_rules`,
  `push_text_block_literal_uses_the_warning_colour`,
  `push_text_source_draws_formulas_as_typed` (`src/shell/render.rs`).
- **AC-4 — Font coverage.** Every symbol of the table (and `−`) has a
  glyph in the embedded font — checked, so no second font is needed.
  Test: `tex_symbols_all_have_glyphs_in_the_embedded_font`
  (`src/shell/render.rs`).
- **AC-5 — Budgets.** Parse + layout are cached per text
  (`text_layout::typeset`), recomputed only for a new text.
  Tests: `layout_typeset_is_cached_per_text` (`src/core/text_layout.rs`);
  `perf_tex_layout_200_blocks_is_under_2ms` (cold) and
  `perf_tessellate_200_formula_blocks_is_under_1ms` (cached) in
  `tests/perf.rs`; the [[ADR-T24-5 Redraw budgets]] manual check with
  `DRAW_FRAME_TIMES=1` on the target is recorded in the *Log*.
- **AC-6 — Fuzz.** `tests/fuzz.rs`: text sessions type formula snippets;
  committed blocks have finite bounds that contain their typeset ink
  (`check_invariants`), and `fuzz_tex_parse_and_layout_are_total`.
- **AC-7 — Docs.** README section *Formulas in text blocks* with the
  supported commands and an example. At integration the integrator adds
  the feature under `[Unreleased]` → `Added` in `CHANGELOG.md`.

## Out of scope

- Display-mode `$$…$$`, matrices, `\begin{…}` environments, macros
  (`\newcommand`), alignment, accents (`\hat`, `\bar`) — candidates for
  a follow-up if missed in practice.
- Rendering through an external TeX engine or a math crate.

## Files owned

- `src/core/tex.rs` (new, or `src/core/tex/` module), `src/core/mod.rs`,
  `src/core/text_layout.rs`, `src/core/tools/text.rs` (edit session uses
  the source box — added in the spec step)
- `src/shell/render.rs`, `src/shell/app.rs` (overlay draws the open block
  as source — added in the spec step)
- `assets/fonts/*` (only if AC-4 needs a math font)
- `tests/fuzz.rs`, `tests/perf.rs`, `README.md`
- new ADRs `ADR-T30-*`, feature note `docs/features/TeX formulas.md`,
  tutorial `docs/tutorials/30 Parsing and laying out a small TeX subset.md`

## Subtasks (one commit each)

- [x] spec — `docs(T30): specify TeX formula acceptance criteria`
- [x] tests — `test(T30): add failing tests for TeX parsing, layout and rendering`
- [x] models — `feat(T30): add TeX segment, node, ink and layout types`
- [x] behaviour — `feat(T30): implement TeX parsing, box layout and typeset rendering`
- [x] quality — `chore(T30): pass clippy, rustfmt and rustdoc`
- [x] docs — `docs(T30): add feature note and tutorial`

## Learning path

Step 30 — requires step 29 (text blocks).

## Log

- 2026-10-14 — Created from the owner's proposal; depends on the text
  blocks and font of T29.
- 2026-10-14 — Now depends on [[T32 Fast text rendering]] (wave 17):
  T29's per-character text drawing missed the full re-render budget, and
  formulas should use T32's batched glyph path; both tasks own
  `src/shell/render.rs` and `tests/perf.rs`.
- 2026-10-15 — Spec step: *Files owned* amended with the owner's approval
  — `src/core/tools/text.rs` (the edit session's box must stay the source
  box) and `src/shell/app.rs` (the overlay must draw the open block as
  source); both owners are done and T31 runs in a later wave. AC-4
  checked: every subset symbol is in the embedded font, no second font.
- 2026-10-15 — Behaviour step: two spec tests were wrong and were fixed
  with the code — a `\` right before the closing `$` escapes it, so the
  unknown control symbol test uses `\,`; `\min_i` matched the `i` of
  "min", now `\min_k`.
- 2026-10-15 — Budgets (AC-5), release build on the development machine
  (not the target): `perf_tex_layout_200_blocks_is_under_2ms` median
  0.89 ms (cold parse + layout of 200 formula blocks);
  `perf_tessellate_200_formula_blocks_is_under_1ms` 0.22 ms;
  `perf_text_layout_and_cull_500_blocks_is_under_1ms` 0.12 ms and
  `perf_tessellate_10k_glyphs_is_under_1ms` 0.50 ms (plain text, unchanged
  path). `perf_undo_delete_5k_of_10k_is_under_1ms` is flaky on this
  machine on `main` too (0.66–1.59 ms) and does not touch text.
  The [[ADR-T24-5 Redraw budgets]] manual check with `DRAW_FRAME_TIMES=1`
  on the target is **pending the owner** (needs the target's GPU).
- 2026-10-15 — `scripts/check.sh` and `scripts/check.sh --fuzz` green.
  Status → review.
