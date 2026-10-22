---
title: Accents that sit on their letters
step: 39
requires: ["[[33 Typesetting with a proportional math font]]", "[[36 Operators whose limits sit beside them]]"]
feature: "[[TeX formulas]]"
code: ["src/core/tex.rs"]
tags: [tutorial]
---

# Accents that sit on their letters

> **Learning path step 39.** Requires: step 33 (a proportional math
> font), step 36 (operators whose limits sit beside them) · Next: —

## Why this matters here

`\vec{F}`, `\dot{x}`, `\hat{n}` are everywhere in physics notes. An
accent looks wrong as soon as it is a little off: too high and it floats,
too low and it touches the letter, centred on the advance of an italic
`𝑥` and it leans left of the letter's top. LaTeX gets this right with
data the font provides — combining accent glyphs and a *top-accent
attachment point* per letter. Our font subset has neither, so the task is
to get close with what is there.

## The concept

**1. An accent is a box over a box.** Lay out the argument; put the mark
a fixed gap above its ascent; the result has the argument's width and
descent and an ascent that reaches the mark's top. Scripts then attach
to the accented box like to any letter, and a superscript rises with the
taller box.

```text
      ˙            ← mark, its ink bottom ACCENT_GAP above…
   ───────  ascent ← …the argument's ascent
     𝑥
   ───────  baseline
   |width|         ← unchanged: the accent never widens the letter
```

**2. Centre on the ink, then skew for the slant.** Over an upright `2`
the middle of the ink is the right place. An italic letter leans right,
so its *top* is right of its middle by roughly `tan(slant) × height / 2`.
The font's `MATH` table stores the exact point per glyph; without it we
move the centre right by `ACCENT_SKEW × ascent` (0.1 per em of height),
close to TeX's own skew kerns for `𝑥`, `𝑣` and `𝐹`. Only a single
slanted letter is skewed — `\hat{xy}` or `\hat{2}` is centred.

**3. Reuse glyphs the font already has.** Combining accents (U+0300…)
are not in the subset, but their spacing cousins are: `^ ~ · ¨ →`. They
sit at various heights in the font, so the layout aligns their *ink
bottom*, not their baseline, to the gap. Bars are rules, like fraction
bars: `\bar` is a fixed short rule (LaTeX's macron), `\overline` spans
the whole argument.

## How draw implements it

- `ACCENTS` in `src/core/tex.rs` maps each name to a `Mark`:
  `Glyph(ch, scale)`, `Bar` or `Overline`
  ([[ADR-T39-1 Accents from spacing glyphs with a slant skew]]).
- `Parser::command` treats any `ACCENTS` name like `\sqrt`: one argument
  (a `{group}`, a symbol or one character), otherwise a literal. The
  node is an ordinary `Node::Command`, so `print` needs no change.
- `accented` lays out the argument, finds its ink centre with
  `MathBox::ink_span`, adds the skew when `slanted` says the argument is
  one letter the style maps to an italic glyph, and places the mark:
  a glyph with its ink bottom at `-(ascent + ACCENT_GAP)`, or a rule
  whose lower edge is there.
- Tests: `tex_layout_accent_sits_above_its_argument`,
  `tex_layout_accent_is_centred_on_an_upright_argument`,
  `tex_layout_accent_over_an_italic_letter_is_skewed`,
  `tex_layout_bar_is_a_short_rule`,
  `tex_layout_overline_spans_its_argument`,
  `tex_layout_accent_keeps_the_argument_width_and_descent`,
  `tex_layout_accent_takes_scripts`; the bounds proptests type accents,
  and `shell::render`'s glyph test checks every accent glyph is in the
  font.

## Try it

1. Set `ACCENT_SKEW` to `0.0` and type `$\hat{x}\ \hat{F}$`: the hats
   drift left of the letters' tops.
   `tex_layout_accent_over_an_italic_letter_is_skewed` fails.
2. Add `\check` (`ˇ`) — it is not in the font subset. Check
   `core::math_metrics::get('\u{2C7}')`; the glyph test in
   `shell::render` would catch the missing glyph.
3. Make `\bar` span its argument like `\overline` and compare
   `$\bar{AB}$` with LaTeX's output.

## Further reading

- *The TeXbook*, Appendix G, rule 12 (accents) and the `\skewchar` kerns.
- OpenType `MATH` table: `MathTopAccentAttachment` and `AccentBaseHeight`.
- [[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]
