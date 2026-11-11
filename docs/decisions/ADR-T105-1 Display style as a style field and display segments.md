---
id: ADR-T105-1
title: Display style as a style field and display segments
status: accepted
kind: decision
date: 2026-11-10
task: T105
builds_on: ["[[ADR-T30-1 TeX subset, box layout and metrics]]", "[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]", "[[ADR-T71-1 Infix fraction commands split their group]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T105-1 Display style as a style field and display segments

## Context

T105 adds LaTeX's display style: `$$…$$` formulas, larger big operators,
fraction parts at full size, and the commands `\dfrac`, `\tfrac`,
`\dbinom`, `\tbinom`, `\displaystyle` and `\textstyle`. Until now a block
had one kind of formula (`Segment::Math`), every big operator was drawn
at `BIG_OP_SCALE` (1.3), and fraction parts were always at script scale.
The font subset has no display-size operator glyphs
([[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]).

## Decision

- **Parsing.** A new segment `Segment::Display(Vec<Node>)` holds a
  `$$…$$` formula. A `$$` opens one only when a closing `$$` follows on
  the same line, with no single unescaped `$` and at least one character
  in between; `\$` inside stays a literal command, as in `$…$`. Otherwise
  the first `$` is plain text and parsing goes on at the second, exactly
  as before (so `$$x$` is still `$` then `$x$`). Display formulas do not
  cross a line break, like inline ones.
- **Style.** The private layout `Style` (the alphabets of
  ADR-T54-1) gains a `display: bool` field, carried down the layout with
  the alphabets. A formula starts with it off (`$…$`) or on (`$$…$$`).
  No global state.
- **Effects of display style.**
  - Big operators and integrals are the same glyph at
    `DISPLAY_OP_SCALE` = **1.8** instead of `BIG_OP_SCALE` (1.3); limits
    stay above and below `\sum`-like operators and beside integrals.
  - Fraction, binomial and infix (`\over`, `\choose`, …) parts are laid
    out at the fraction's own scale in text style (display off); in text
    style they are at script scale, as now.
  - Scripts, extensible-arrow labels and environment cells are always in
    text style. Root, accent, `\left…\right`, `\underline`, `\pmod` and
    alphabet bodies keep the style they are in.
- **Commands.** `\dfrac` and `\tfrac` are fractions and `\dbinom` and
  `\tbinom` binomials whose parts are laid out as display (`d`) or text
  (`t`) parts, whatever the outside style. `\displaystyle` and
  `\textstyle` parse to a new node `Node::MathStyle(bool)` (true for
  display), printed back as typed. In layout it is not an atom (like a
  kern, ADR-T99-1): it switches the style of the rest of its list, which
  is the rest of its group, argument or formula. As a script base it is
  an empty ordinary box.
- **Block layout.** A display formula ends the current line if that line
  has any content, sits on a line of its own, and the next character that
  is not a line break starts a new line; a `\n` right after it only ends
  its line, so `a\n$$x$$\nb` is three lines, as is `a $$x$$ b`. Text
  after a display formula keeps its spaces as typed (the cell grid is
  kept). Each display line is centred on the block's width (the widest
  line, display lines included), its italic overhangs inside the line as
  for inline formulas.

## Alternatives considered

- **A `display` flag on `Segment::Math`** — every `match` on segments
  would need to look at it, and existing tests and fuzz checks that build
  `Segment::Math` would change shape; a separate variant keeps inline
  formulas untouched.
- **A style node that wraps the rest of its group** (`Node::Styled {
  display, rest }`) — mirrors TeX's scope literally, but splits the
  atom list so spacing between the switch's neighbours would change and
  the parser would need a second kind of group. A switch node read while
  walking the list does the same with no new nesting.
- **Display-size glyph variants from the full font** — out of scope:
  binary size (ADR-T55-1).
- **Scale 1.6 or 2.0** — 1.8 is about the ratio of Computer Modern's
  display to text `\sum` (about 1.4 times the text size against our 1.3
  text scale, which is already enlarged); it reads as clearly larger
  without the limits drifting far from the sign.
- **Centring on the widest non-display line** — a display formula wider
  than the text would then hang out of the block box.

## Consequences

- Inline formulas lay out exactly as before, except that `\dbinom`
  inline now has full-size parts (it drew like `\binom` before).
- A display block is taller; its lines change when the text around the
  formula is edited, which only the layout cache sees (ADR-T30-1).
- A single `$` inside a would-be display formula makes the whole thing
  fall back to inline parsing; this is documented in the feature note.

## Rollback plan

Revert the T105 commits: `Segment::Display`, `Node::MathStyle`, the
`display` field and `DISPLAY_OP_SCALE` go together, and `$$…$$` reads as
before. Record it as a rollback ADR.
