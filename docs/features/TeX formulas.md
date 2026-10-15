---
title: TeX formulas
task: "[[T30 Basic TeX formulas]]"
adrs: ["[[ADR-T30-1 TeX subset, box layout and metrics]]"]
tutorial: "[[30 Parsing and laying out a small TeX subset]]"
shortcuts: ["$"]
tags: [feature]
---

# TeX formulas

## What it does

Inside a text block, `$…$` is a formula in a small TeX subset, typeset
once the block is finished: scripts, fractions, roots, binomials, sums
with limits, Greek letters and the usual relations — enough for
`$dp_{i,j} = \min(dp_{i-1,j}, dp_{i,j-1}) + a_{i,j}$` or
`$\sum_{k=1}^{n} k^2 = \frac{n(n+1)(2n+1)}{6}$`. While the block is open
you see and edit the source; plain text and formulas mix on one line.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Start / end a formula | `$` … `$` on one line |
| Literal dollar sign | `\$` (a lone `$`, as in `costs $5`, also stays plain) |
| Superscript / subscript | `x^2`, `x_i`, `x_{i,j}^{2k}` |
| Fraction, binomial, root | `\frac{a}{b}`, `\binom{n}{k}`, `\sqrt{x}` (`\frac12` works too) |
| Big operators with limits | `\sum_{i=1}^{n}`, `\prod`, `\min_i`, `\max`, `\lim` |
| Functions (upright) | `\log \ln \gcd \lcm`, `a \bmod m`, `\pmod{m}` |
| Greek | `\alpha` … `\omega`, `\Gamma` … `\Omega` |
| Relations, operators | `\le \ge \ne \approx \equiv \cdot \times \div \pm \to \leftarrow \Rightarrow \iff \in \notin \subset \cup \cap \land \lor \neg \oplus \infty \ldots \cdots` |
| Brackets | `\lfloor \rfloor \lceil \rceil \{ \} |` |
| Upright text | `\text{if } x > 0` |

Anything else is drawn as typed, in red (yellow on a red block), so a
typo shows.

## How it works

`core::tex` (`src/core/tex.rs`) parses a block into plain text and
formulas, each a lossless tree, and lays it out with a reduced TeX box
model — width, ascent, descent; scripts at 0.7×, a math axis at 0.34 em —
in em units ([[ADR-T30-1 TeX subset, box layout and metrics]]).
`core::text_layout::typeset` caches the layout per text, and
`text_layout::bounds` gives every shape query (hit-test, culling,
snapping, selection) the typeset box; the edit session keeps the source
box (`source_bounds`). `shell::render` draws the glyphs through the T32
atlas at their own scales and the fraction bars and root signs as quads
of the same batch.

## Limits

- No display mode (`$$…$$`), matrices, environments, macros, accents or
  `\left`/`\right` — brackets do not grow (only `\binom`'s do).
- One upright font: letters are not italic.
- A formula stays on one line.
- Lines with scripts or fractions are taller than plain lines.
