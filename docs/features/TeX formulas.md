---
title: TeX formulas
task: "[[T30 Basic TeX formulas]]"
adrs: ["[[ADR-T30-1 TeX subset, box layout and metrics]]", "[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]", "[[ADR-T36-1 Integral class with scripts beside the sign]]", "[[ADR-T39-1 Accents from spacing glyphs with a slant skew]]", "[[ADR-T41-1 Integral superscript at the advance]]", "[[ADR-T64-1 Operatorname with a star parsed as one command]]"]
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

Formulas are set in **Latin Modern Math**, the font of LaTeX: italic
variables, upright digits, operators and function names, TeX's spacing,
and calligraphic capitals with `\mathcal`. Plain text around them stays
in the monospaced text font. (Since [[T33 Math font for formulas]];
tutorial [[33 Typesetting with a proportional math font]].)

Integrals — `$\int_0^1 f(x)\,dx$`, `\oint`, `\iint` — are enlarged like
`\sum` with their limits as scripts beside the slanted sign, as in inline
LaTeX — the upper limit right beside the top of the sign since
[[T41 Integral superscript at the advance]] — and the symbols most used in physics are there too:
Schrödinger's `$i\hbar\,\partial_t\psi = H\psi$`, Maxwell's
`$\nabla \times E = -\partial_t B$`, bra–kets `$\langle a \mid b \rangle$`.
(Since [[T36 Integrals in formulas]]; tutorial
[[36 Operators whose limits sit beside them]].)

Accents sit centred on their letter, as in LaTeX: Newton's
`$\vec{F} = m\ddot{x}$`, a unit vector `$\hat{n}$`, an average
`$\bar{v}$`, a segment `$\overline{AB}$`. Over a slanted letter the
accent moves right with the slant, and scripts attach after the accented
letter (`$\vec{F}_x$`, `$\dot{x}^2$`). (Since [[T39 Accents in formulas]];
tutorial [[39 Accents that sit on their letters]].)

The Hebrew letters of set theory are there for cardinals:
`$\aleph_0 < 2^{\aleph_0}$`, `\beth_1`, `\gimel`, `\daleth` — upright,
as in LaTeX, and never changed by `\mathcal`. (Since
[[T61 Hebrew letters]].)

Trigonometric and hyperbolic functions are upright words with the thin
operator space of `\log`: `$\sin^2\theta + \cos^2\theta = 1$`,
`$\tan x$`, `$\arctan x$`, `$\cosh t$`; a power sits beside the word,
not above it. Besides LaTeX's thirteen (`\sin \cos \tan \cot \sec \csc
\arcsin \arccos \arctan \sinh \cosh \tanh \coth`), five common names
that are *not in LaTeX's base set* work too: `\arccot \arcsec \arccsc
\sech \csch` (in LaTeX they would need `\operatorname`). (Since
[[T63 Trigonometric and hyperbolic functions]].)

The other named operators of LaTeX are upright words too. Those that
take limits put a subscript below the word, like `\min`:
`$\inf_{x\in S} f(x)$`, `\sup`, `\det`, `\Pr`, `\gcd` and
`$\limsup_{n\to\infty} a_n$` (`\liminf` and `\limsup` are drawn
"lim inf", "lim sup" with a thin space between the words). `\exp \lg
\deg \dim \ker \hom \arg` keep their scripts beside them, like `\log`.
Any other name is `\operatorname{rank}` (scripts beside) or
`\operatorname*{argmin}_x` (with the star it takes limits); the body is
a raw word, like `\text{…}`, and the star counts only directly after
`\operatorname`. (Since [[T64 Named operators with limits]],
[[ADR-T64-1 Operatorname with a star parsed as one command]].)

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Start / end a formula | `$` … `$` on one line |
| Literal dollar sign | `\$` (a lone `$`, as in `costs $5`, also stays plain) |
| Superscript / subscript | `x^2`, `x_i`, `x_{i,j}^{2k}` |
| Fraction, binomial, root | `\frac{a}{b}`, `\binom{n}{k}`, `\sqrt{x}` (`\frac12` works too) |
| Big operators with limits | `\sum_{i=1}^{n}`, `\prod`, `\min_i`, `\max`, `\lim` |
| Named operators with limits | `\inf \sup \liminf \limsup \det \Pr \gcd`, as in `\limsup_{n\to\infty}` |
| Integrals (limits beside) | `\int_0^1`, `\iint`, `\iiint`, `\oint`, `\oiint` |
| Functions (upright) | `\log \ln \lcm \exp \lg \deg \dim \ker \hom \arg`, `a \bmod m`, `\pmod{m}` |
| Any operator name | `\operatorname{rank}`, `\operatorname*{argmin}_x` (`*`: limits below) |
| Trigonometric, hyperbolic | `\sin \cos \tan \cot \sec \csc`, `\arcsin \arccos \arctan`, `\sinh \cosh \tanh \coth`; not in LaTeX's base set: `\arccot \arcsec \arccsc \sech \csch` |
| Greek | `\alpha` … `\omega` (italic), `\Gamma` … `\Omega` (upright) |
| Hebrew (set theory, upright) | `\aleph \beth \gimel \daleth`, as in `\aleph_0 < 2^{\aleph_0}` |
| Calligraphic capitals | `\mathcal{F}`, `\mathcal A` |
| Accents | `\vec{F}`, `\hat{n}`, `\tilde{x}`, `\dot{x}`, `\ddot{x}`, `\bar{v}` (one letter or `{group}`) |
| Overline | `\overline{AB}` (as wide as its argument) |
| Relations, operators | `\le \ge \ne \approx \equiv \cdot \times \div \pm \to \leftarrow \Rightarrow \iff \in \notin \subset \cup \cap \land \lor \neg \oplus \infty \ldots \cdots` |
| Brackets | `\lfloor \rfloor \lceil \rceil \{ \} | \langle \rangle` |
| Calculus, vectors | `\partial \nabla` |
| Letters, constants | `\hbar \ell \Re \Im \AA \degree` |
| Logic, sets | `\forall \exists \emptyset \prime` |
| Dots, bars | `\vdots \ddots \vert \Vert` |
| More relations | `\propto \sim \simeq \cong \ll \gg \lesssim \gtrsim \doteq \coloneqq \parallel \perp \mid` |
| More arrows | `\rightarrow \leftrightarrow \Leftarrow \Leftrightarrow \uparrow \downarrow \mapsto \rightleftharpoons` |
| More binary operators | `\mp \otimes \wedge \vee \circ \ast \star \bullet \dagger` |
| Thin space | `\,` (as in `f(x)\,dx`) |
| Upright text | `\text{if } x > 0` |

Anything else is drawn as typed, in red (yellow on a red block), so a
typo shows.

## How it works

`core::tex` (`src/core/tex.rs`) parses a block into plain text and
formulas, each a lossless tree, and lays it out with a reduced TeX box
model — width, ascent, descent; scripts at 0.7×, a math axis at 0.25 em —
in em units ([[ADR-T30-1 TeX subset, box layout and metrics]]). Each
glyph carries a face: formula glyphs are in the math face and measured
with `core::math_metrics`, a table generated from the embedded Latin
Modern Math subset, so `core` stays font-free; letters map to the
Mathematical Alphanumeric Symbols (`x` → `𝑥`, `\mathcal{A}` → `𝒜`)
([[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]).
`core::text_layout::typeset` caches the layout per text, and
`text_layout::bounds` gives every shape query (hit-test, culling,
snapping, selection) the typeset box; the edit session keeps the source
box (`source_bounds`). `shell::render` draws the glyphs through the T32
atlas — keyed by face, one font per face — at their own scales and the
fraction bars and root signs as quads
of the same batch.

## Limits

- No display mode (`$$…$$`), matrices, environments, macros or
  `\left`/`\right` — brackets do not grow (only `\binom`'s do).
- Accents do not grow: no `\widehat`, `\widetilde`, `\overrightarrow`,
  and no under-accents (`\underline`, `\underbrace`). An accent takes
  one letter, a symbol or a `{group}`, not another accent without braces
  (`\hat{\vec{x}}` works, `\hat\vec x` does not). Their horizontal
  placement over italic letters is approximated (the font's top-accent
  points are not embedded).
- No `\limits` / `\nolimits`; integrals are always the inline size.
- No `\DeclareMathOperator`, `\varliminf`, `\varlimsup`, `\injlim` or
  `\projlim`; write `\operatorname{…}` instead.
- Of the spacing commands only `\,` exists (`\;`, `\quad`, `\!` are
  literals); `\longrightarrow`, `\implies` and `\triangle` are not in
  the font subset.
- `\mathcal` has capitals only (as in TeX); no `\mathbb`, `\mathfrak`
  or bold math.
- A character the math font lacks (e.g. CJK in `\text{…}`) falls back to
  a monospaced cell.
- A formula stays on one line.
- Hebrew is there as math letters only: Hebrew text (`א`–`ת` in
  `\text{…}` or plain text) and right-to-left layout are not.
- Lines with scripts or fractions are taller than plain lines.
