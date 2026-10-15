---
id: ADR-T30-1
title: TeX subset, box layout and metrics
status: accepted
kind: decision
date: 2026-10-15
task: T30
builds_on: [ADR-0003, ADR-0014, ADR-T29-1, ADR-T29-2, ADR-T32-1, ADR-T32-2]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T30-1 TeX subset, box layout and metrics

## Context

[[T30 Basic TeX formulas]] typesets `$…$` inside text blocks — enough for
contest notes like `$dp_{i,j} = \min(dp_{i-1,j}, dp_{i,j-1}) + a_{i,j}$`.
It must live in `core` ([[ADR-0003 Headless core and thin shell]]), add
no dependency ([[ADR-0014 Minimal dependencies and no unsafe]]), be total
on every input and fuzzed. Bounds, hit-testing, culling and snapping of a
committed block must use the typeset size, so the layout has to be a pure
function of the text, like the monospaced cell layout of
[[ADR-T29-1 Text shape with monospaced layout and an embedded font]].
The only font is the embedded Adwaita Mono subset: one weight, upright,
every glyph 0.6 em wide.

## Decision

### Source syntax

- A **formula** is `$…$` on one line: a `$` opens one when an unescaped
  `$` follows on the same line with at least one character between them;
  it ends at the first such `$`. A `$` with no closing `$` on its line,
  and `$$`, are plain text. `\$` is a literal `$` inside and outside
  formulas. Plain text is otherwise drawn as typed.
- Inside a formula:
  - `^` and `_` take one argument: a character, a symbol command, or a
    `{group}`. They attach to the atom right before them (no whitespace
    between); otherwise their base is empty. One `^` and one `_` per base,
    in either order.
  - `\frac{a}{b}`, `\binom{n}{k}`, `\sqrt{x}`, `\pmod{m}` take braced or
    single-token arguments (`\frac12`); `\text{…}` takes a braced raw body
    drawn as typed.
  - Symbol commands, mapped to one character or an upright word:
    Greek `\alpha` … `\omega` (with `\varepsilon \vartheta \varpi
    \varrho \varsigma \varphi`), `\Gamma \Delta \Theta \Lambda \Xi \Pi
    \Sigma \Upsilon \Phi \Psi \Omega`; relations `\le \leq \ge \geq \ne
    \neq \approx \equiv \to \leftarrow \Rightarrow \iff \in \notin
    \subset`; binary operators `\cdot \times \div \pm \cup \cap \land
    \lor \oplus \bmod`; ordinary `\neg \infty \ldots \cdots \lfloor
    \rfloor \lceil \rceil \{ \} \$`; big operators `\sum \prod`; words
    with limits `\min \max \lim`; functions `\log \ln \gcd \lcm`.
  - Whitespace is ignored, as in TeX; `-` is drawn as `−` (U+2212).
- Anything else — an unknown command, an unmatched brace, `^`/`_` or a
  command with a missing argument, a second `^` or `_` on one base, a
  lone `\` — is a **literal** node: its source, drawn in a warning colour
  (palette red, yellow on a red block). Never an error, never a panic.

### Parse tree

`core::tex::parse(&str) -> Vec<Segment>`, with `Segment::{Text, Dollar,
Math}` and math `Node::{Space, Char, Symbol, Group, Scripts, Command,
Text, Literal}`. The tree is **lossless**: every node keeps the source it
came from (including skipped whitespace before an argument), so
`print(parse(s)) == s` for every `s` — the property test that proves the
parser drops and invents nothing.

### Layout model

TeX's box model, reduced. Units are em of the block's font size, y down,
positions are baseline-left points.

- A box has `width`, `ascent`, `descent` and its **ink**: glyphs `(char,
  position, scale, warn)` and rules (segments with a width).
- A glyph is one cell: `ADVANCE × scale` wide (0.6 em, T29), ascent
  `0.75` and descent `0.22` × scale — the font's cap height 0.735 and
  descender 0.215–0.223; brackets, bars, `∑ ∏ √` use `0.85` / `0.17`
  (their ink spans −0.16 … 0.84 em). Measured with fontdue on the
  embedded font; no per-glyph table.
- **Math axis** `0.34` em (centre of `−`), **rule** `0.07` em (`−`'s
  thickness).
- **Scripts**: scale `0.7 ×`, never below `0.5`. Superscript baseline up
  by `max(0.4 s, base ascent − 0.35 s)`, subscript down by
  `max(0.2 s, base descent − 0.05 s)`; with both, the subscript moves down
  until `0.1 s` separates them. Scripts start at the base's right edge.
- **Big operators** `∑ ∏` are drawn `1.3 ×` and centred on the axis;
  they and `min max lim` put their scripts as **limits**, centred above
  and below with a `0.1 s` gap.
- **Fractions** and **binomials**: parts at script scale, centred, the
  numerator `0.08 s` above and the denominator `0.08 s` below a bar on
  the axis (`0.1 s` side padding); a binomial has no bar and is wrapped
  in `(` `)` scaled up to its height (at most 3 ×).
- **Roots**: the root sign is four rules (tick, down stroke, up stroke,
  overbar) `0.5 s` wide, `0.1 s` above the content.
- **Spacing** between atoms (TeX classes Ord, Op, Bin, Rel, Open, Close,
  Punct): Rel gets `0.15 s` on each side, Bin `0.1 s` (a Bin at the start,
  after an operator/relation/opening or before a relation/closing is an
  Ord, so `-a` is a unary minus), an Op (function or big operator) before
  an Ord or Op gets `0.1 s`. No space inside scripts other than the Op
  space. Our monospaced glyphs already carry side bearings, so these are
  about half of TeX's.
- **Blocks**: lines split on `\n`. A line without a formula is exactly
  the T29 cell line (1.25 em, baseline 1.0 em), so text without `$`
  lays out exactly as before. A line with a formula grows to
  `max(1.0, ascent + 0.05)` above and `max(0.25, descent + 0.05)` below
  its baseline. Plain characters keep their 0.6 em cells.
- Letters stay upright: there is no italic in the embedded font, and
  function names are distinguished by spacing only.

### Where it is used

- `core::text_layout::bounds` (hence `Shape::bounds`, hit-test, culling,
  snapping) uses the typeset size when the text contains `$`;
  `text_layout::source_bounds` keeps the T29 cell box and is what the
  edit session uses ([[ADR-T29-2 Text edit mode]]: the open block shows
  its source).
- `core::text_layout::typeset(text)` memoises `parse` + `layout` per text
  in a thread-local map of `Rc<Layout>` (cleared past 4 096 entries); the
  layout is in em, so one entry serves every size, position and zoom.
  Work happens only when a text changes.
- `shell::render` draws a committed block's glyphs through the T32 atlas
  ([[ADR-T32-1 Own glyph atlas rasterized with fontdue]]), each at
  `size × scale` (so up to a few extra raster sizes per character), and
  its rules as quads of the same batch
  ([[ADR-T32-2 Shapes and glyphs share one textured batch]]). The overlay
  draws the open block with the cell layout.

### Metrics source and font

Every symbol of the table has a glyph in the embedded subset (checked by
a test over the table), so **no second font** is embedded; the binary does
not grow.

## Alternatives considered

- **A math crate or an external TeX engine** — new dependencies, far
  more than the subset needs; ruled out by the task.
- **Per-glyph metrics table from the font** — tighter boxes, but a
  table copied into `core` for 1 342 glyphs; uniform cell metrics are
  enough for a scratch canvas.
- **Formulas spanning lines, `$$…$$`** — out of scope; keeping formulas
  on one line keeps line layout and caret mapping simple.
- **Cache inside `Shape::Text`** — would change the shape model and
  every constructor; a memo keyed by text needs no model change.
- **TeX's text-style fraction rules exactly** — same idea; our numbers
  are tuned to a monospaced font with no math tables.

## Consequences

- A block with `$` costs one map lookup per `bounds` call after its first
  layout; layout itself is a few microseconds per formula.
- A typeset block can be smaller or larger than its source, so the box
  changes when editing starts and ends.
- Lines with scripts or fractions grow taller than plain lines.
- Unknown commands are visible (warning colour), so a typo is noticed.

## Rollback plan

A rollback ADR plus `revert(T30)` commits remove `core::tex`, return
`text_layout::bounds` to the cell box and the renderer to cell glyphs;
the edit session and overlay changes revert with them.
