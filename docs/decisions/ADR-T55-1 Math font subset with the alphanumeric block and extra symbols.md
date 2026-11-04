---
id: ADR-T55-1
title: Math font subset with the alphanumeric block and extra symbols
status: accepted
kind: decision
date: 2026-11-04
task: T55
builds_on: [ADR-T14-1, ADR-T32-1]
amends: [ADR-T33-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T55-1 Math font subset with the alphanumeric block and extra symbols

## Context

[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]
embeds a subset of Latin Modern Math 1.959 with only the italic Latin,
script capitals and italic Greek from the Mathematical Alphanumeric
Symbols block, and sets a size of "about 140 KB, under 200 KB". The TeX
series T56–T71 adds `\mathbf`, `\boldsymbol`, `\mathbb`, `\mathfrak`,
`\mathscr`, `\mathsf`, `\mathtt`, long arrows, big operators such as
`\bigoplus`, `\amalg`, geometric shapes such as `\square`, and `\dddot`.
Their glyphs are not in the subset. Growing the subset once, in one
task, keeps those tasks to command tables and keeps the font and its
metrics table changing in one place.

## Decision

- The subset keeps every range of ADR-T33-1 and adds `U+1D400–1D7FF`
  (the whole Mathematical Alphanumeric Symbols block, which contains the
  three ranges ADR-T33-1 listed), `U+27F0–27FF` (long arrows),
  `U+2900–297F` (supplemental arrows), `U+2A00–2A0C` (n-ary operators),
  `U+2A3F` (`⨿`), `U+25A0–25FF` (geometric shapes) and `U+20D0–20DF`
  (combining marks for symbols). Whole blocks are listed even where the
  font has only a few of their characters: the subsetter keeps what the
  font has, and the command stays short.
- What the font really has is listed in the spec of
  [[T55 Math font subset for alphabets and symbols]]. The ones to note:
  no script lowercase (plain or bold) and no bold digamma in the
  alphanumeric block; no `U+2AEB`, no `U+2AA6`; no variation sequences;
  GSUB features `aalt dtls ssty` only.
- **Size budget: under 300 KB** (was under 200 KB). Measured: 270 232
  bytes (was 139 336). `math_font_is_embedded_and_small` checks it.
- `core::math_metrics::TABLE` is regenerated with
  `scripts/math_metrics.py`: **1762 rows** (was 888). Rows that were in
  the table keep their values; only new rows appear.
- The rebuild command lives in `assets/fonts/README.md`;
  `python3 -m fontTools.subset` is the same program as `pyftsubset`. The
  command with the old ranges reproduces the old subset byte for byte.

### Performance

The glyph atlas ([[ADR-T32-1 Own glyph atlas rasterized with fontdue]])
rasterizes glyphs on first use, so its pages and uploads do not grow
with the subset: a formula that uses only the old glyphs fills the atlas
exactly as before. Start-up parses one font about twice as large (fontdue
reads every outline once). [[ADR-T14-1 Performance budgets]] sets no
start-up budget; its `tests/perf.rs` budgets, which load both fonts the
way the app does, stay green (numbers in the task *Log*).

## Alternatives considered

- **Only the styles T56–T71 use, range by range** — a slightly smaller
  file (the sans-serif bold italic Greek and monospace alone are a few
  KB), but each later task that finds a missing glyph would reopen the
  font, and the range list would be long and easy to get wrong.
- **The full font (about 730 KB)** — most of it is glyph variants and
  scripts formulas never use.
- **Keeping the 200 KB budget** — not possible with the bold, fraktur
  and double-struck alphabets.

## Consequences

- The binary grows by about 130 KB.
- Later TeX tasks add commands only; a command whose glyph the font
  lacks is left out, not drawn from another font.
- `math_metrics.rs` doubles in length; it stays generated and checked by
  `math_metrics_match_the_embedded_font`.

## Rollback plan

A rollback ADR plus `revert(T55)` commits restore the 139 KB subset, the
888-row table, the README ranges and the 200 KB guard. Tasks T56–T71
that use the new glyphs would have to be reverted first.
