---
title: TeX formulas
task: "[[T30 Basic TeX formulas]]"
adrs: ["[[ADR-T30-1 TeX subset, box layout and metrics]]", "[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]", "[[ADR-T36-1 Integral class with scripts beside the sign]]", "[[ADR-T39-1 Accents from spacing glyphs with a slant skew]]", "[[ADR-T41-1 Integral superscript at the advance]]", "[[ADR-T64-1 Operatorname with a star parsed as one command]]", "[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]", "[[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]", "[[ADR-T56-1 Accent skew follows a slant flag per math alphabet]]", "[[ADR-T59-1 One script alphabet for mathscr and mathcal]]", "[[ADR-T60-1 Bold sans-serif inside a bold alphabet]]", "[[ADR-T65-1 Wide relations padded with a thick space]]"]
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

Every Greek letter LaTeX writes is there: the lowercase letters and
their variants (`\varepsilon \vartheta \varkappa \varpi \varrho
\varsigma \varphi`), the capitals, the amsmath italic capitals
`$\varGamma \ne \Gamma$` … `\varOmega` (italic even inside `\mathrm`),
and the capitals that look like Latin letters under their unicode-math
names, `\Alpha \Beta \Epsilon \Zeta \Eta \Iota \Kappa \Mu \Nu \Omicron
\Rho \Tau \Chi` — upright like `\Gamma`, italic in `\mathnormal`.
(Since [[T62 Complete Greek letters]].)

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

Math alphabets change the letters of their argument, as in LaTeX:
`$\mathrm{d}x$` is an upright roman `d` (no italic correction, so the
`x` follows right after it), `$\mathit{diff}$` is an italic word with no
gaps between the letters, and `$\mathnormal{\Gamma}$` is the italic
capital Γ. `\mathcal` is one of them. The innermost alphabet wins
(`\mathrm{\mathcal{A}}` is a calligraphic 𝒜), and what an alphabet does
not cover is drawn as around it: `\mathcal{a}` is the italic 𝑎,
`\mathrm{\alpha}` the italic α (`\mathrm` leaves Greek alone, as in
LaTeX), and digits in `\mathit` stay upright. Operators, functions,
`\operatorname`, `\text{…}` and symbols such as `\partial` or `\aleph`
are never changed. (Since [[T54 Math alphabet styles]],
[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]; tutorial
[[54 Math alphabets as a mapping table]].)

Bold is two more alphabets. `$\mathbf{v}$` is upright bold: Latin
letters, digits and Greek capitals (`\mathbf{\Gamma}` → 𝚪), while
lowercase Greek stays italic, as in LaTeX. `$\boldsymbol{\alpha}$` and
its short form `$\bm{x}$` are bold italic, Greek lowercase and capitals
included, with bold digits. An accent is centred over an upright bold
letter and skewed over a bold italic one. Only letters and digits can be
bold: the font has no bold operators, and `\boldmath` (a LaTeX
declaration valid only outside math mode) is drawn as an unknown
command. (Since [[T56 Bold math]],
[[ADR-T56-1 Accent skew follows a slant flag per math alphabet]].)

Blackboard bold names number sets: `$\mathbb{R}$`, `$\mathbb{Z}_p$`,
`$\mathbb{F}_q$`. Capitals, lowercase and digits are double-struck
(`ℂ ℍ ℕ ℙ ℚ ℝ ℤ` come from Letterlike Symbols, as Unicode places them);
Greek stays as outside, and an accent is centred. `$\mathds{1}_A$`
(dsfont's indicator function) is the same alphabet. draw accepts more
than amssymb, whose `\mathbb` has capitals only; there are no `\R` or
`\N` shortcuts. (Since [[T57 Blackboard bold]],
[[ADR-T57-1 One double-struck alphabet for mathbb and mathds]].)

Fraktur letters name Lie algebras, ideals and groups: `$\mathfrak{g}$`,
`$\mathfrak{p}$`, `$\mathfrak{S}_n$`. Capitals and lowercase are
fraktur (`ℭ ℌ ℑ ℜ ℨ` come from Letterlike Symbols); digits and Greek
stay as outside (Unicode has no fraktur for them), and an accent is
centred. `\Re` and `\Im` still draw `ℜ` and `ℑ`. There is no bold
fraktur: `\mathbf{\mathfrak{g}}` is a plain fraktur g (the innermost
alphabet wins). (Since [[T58 Fraktur letters]].)

Script capitals name sheaves, Lagrangians and transforms:
`$\mathscr{L}$`, `$\mathscr{F}$`, `$\mathscr{H}$`. `\mathscr` is the
`\mathcal` alphabet under the rsfs name: Latin Modern Math has a single
script style, so in draw the two commands look the same (in LaTeX rsfs
is rounder). The font has no script lowercase either, so
`\mathscr{l}`, like `\mathcal{l}`, is the italic 𝑙 of the text around
it; digits and Greek stay as outside, and an accent is skewed as over
any slanted letter. (Since [[T59 Script letters]],
[[ADR-T59-1 One script alphabet for mathscr and mathcal]].)

Sans-serif and typewriter letters name categories, matrices and code:
`$\mathsf{G}$`, `$\mathsf{Set}$`, `$\mathtt{x1}$`. Capitals,
lowercase and digits are sans-serif (`𝖦 𝗑 𝟣`) or monospace (`𝚡 𝟷`),
upright, so an accent is centred; Greek stays as outside. `\mathsf`
inside a bold alphabet, or a bold alphabet inside `\mathsf`, is bold
sans, upright: `\mathbf{\mathsf{x}}`, `\bm{\mathsf{x}}` and
`\mathsf{\mathbf{x}}` all draw `𝘅`. Anything in between (`\mathrm`)
breaks the chain, and `\mathtt` has no bold (Unicode has none):
`\mathbf{\mathtt{x}}` is `𝚡`. (Since
[[T60 Sans-serif and typewriter letters]],
[[ADR-T60-1 Bold sans-serif inside a bold alphabet]].)

Sets and logic read as in LaTeX: `$A \setminus B$`,
`$A \subseteq B \cup C$`, `$\Gamma \vdash p$`, `$p \veebar q$` (xor),
`$\lVert v \rVert$`, `$\lvert x \rvert$` and, with
`\langle u, v \rangle`, inner products. `\implies` and `\impliedby`
are spaced as amsmath does: a thick space each side on top of the
relation's own, so `$p \implies q$` is wider than
`$p \Rightarrow q$`, in scripts too. `\lvert \rvert \lVert \rVert` are
opening and closing bars, so a minus right after `\lvert` is a sign.
`\varnothing` draws the same `∅` as `\emptyset`, and `\barwedge` is
`⊼` (the font subset has no other glyph for them). (Since
[[T65 Set and logic symbols]],
[[ADR-T65-1 Wide relations padded with a thick space]].)

Big set and logic operators take limits above and below like `\sum`:
`$\bigcup_{i=1}^{n} A_i$`, `$\bigcap_{k} B_k$`, `$\bigoplus_{i} V_i$`,
`$\bigwedge_{i} p_i$`, and also `\bigvee \bigotimes \bigodot \biguplus
\bigsqcup \coprod`. They are drawn at the same enlarged size as `\sum`,
centred on the axis. (Since [[T66 Big set and logic operators]].)

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Start / end a formula | `$` … `$` on one line |
| Literal dollar sign | `\$` (a lone `$`, as in `costs $5`, also stays plain) |
| Superscript / subscript | `x^2`, `x_i`, `x_{i,j}^{2k}` |
| Fraction, binomial, root | `\frac{a}{b}`, `\binom{n}{k}`, `\sqrt{x}` (`\frac12` works too) |
| Big operators with limits | `\sum_{i=1}^{n}`, `\prod`, `\min_i`, `\max`, `\lim` |
| Big set and logic operators | `\bigcup_{i=1}^{n} A_i`, `\bigcap \bigwedge \bigvee \bigoplus \bigotimes \bigodot \biguplus \bigsqcup \coprod` (limits above and below) |
| Named operators with limits | `\inf \sup \liminf \limsup \det \Pr \gcd`, as in `\limsup_{n\to\infty}` |
| Integrals (limits beside) | `\int_0^1`, `\iint`, `\iiint`, `\oint`, `\oiint` |
| Functions (upright) | `\log \ln \lcm \exp \lg \deg \dim \ker \hom \arg`, `a \bmod m`, `\pmod{m}` |
| Any operator name | `\operatorname{rank}`, `\operatorname*{argmin}_x` (`*`: limits below) |
| Trigonometric, hyperbolic | `\sin \cos \tan \cot \sec \csc`, `\arcsin \arccos \arctan`, `\sinh \cosh \tanh \coth`; not in LaTeX's base set: `\arccot \arcsec \arccsc \sech \csch` |
| Greek | `\alpha` … `\omega` (italic), `\Gamma` … `\Omega` (upright) |
| Greek variants | `\varepsilon \vartheta \varkappa \varpi \varrho \varsigma \varphi`; italic capitals `\varGamma` … `\varOmega`; `\Alpha \Beta \Epsilon \Zeta \Eta \Iota \Kappa \Mu \Nu \Omicron \Rho \Tau \Chi` (upright, unicode-math names) |
| Hebrew (set theory, upright) | `\aleph \beth \gimel \daleth`, as in `\aleph_0 < 2^{\aleph_0}` |
| Calligraphic capitals | `\mathcal{F}`, `\mathcal A` |
| Script capitals | `\mathscr{L}`, `\mathscr{F}` (the same letters as `\mathcal`) |
| Math alphabets | `\mathrm{d}x` (upright roman), `\mathit{diff}` (italic word), `\mathnormal{\Gamma}` (italic Greek capitals); nested, the innermost wins |
| Bold | `\mathbf{v}` (upright bold, also digits and Greek capitals), `\boldsymbol{\alpha}`, `\bm{x}` (bold italic) |
| Blackboard bold | `\mathbb{R}`, `\mathbb{Z}_p`, `\mathds{1}_A` (capitals, lowercase and digits) |
| Fraktur | `\mathfrak{g}`, `\mathfrak{p}`, `\mathfrak{S}_n` (capitals and lowercase) |
| Sans-serif, typewriter | `\mathsf{G}`, `\mathtt{x1}` (Latin and digits; `\mathbf{\mathsf{x}}` is bold sans) |
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
| Sets | `\setminus \sqcup \sqcap \uplus`, `\supset \subseteq \supseteq \subsetneq \supsetneq \nsubseteq \ni`, `\complement \varnothing` |
| Logic | `\lnot \veebar \barwedge \top \bot \nexists`, `\vdash \models`, `\implies \impliedby` (spaced wider) |
| Absolute value, norm | `\lvert x \rvert`, `\lVert v \rVert` |
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
([[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]])
through the alphabet table `ALPHABETS`: one row per command with a
mapping per kind of character (Latin, digits, Greek), resolved innermost
first down to math italic
([[ADR-T54-1 Math alphabets as a table of per-kind mappings]]); a
`slant` flag per row decides whether an accent is skewed
([[ADR-T56-1 Accent skew follows a slant flag per math alphabet]]).
The subset holds the whole Mathematical Alphanumeric Symbols block (bold,
script capitals, fraktur, double-struck, sans-serif, monospace, Greek and
digit styles; Latin Modern Math has no script lowercase), long and
supplemental arrows, the n-ary operators `⨀ ⨁ ⨂ ⨄ ⨆`, `⨿`, geometric
shapes and combining marks, about 270 KB and 1762 metrics rows
([[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]).
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
- No `\limits` / `\nolimits`; integrals and big operators are always
  the inline size.
- No `\DeclareMathOperator`, `\varliminf`, `\varlimsup`, `\injlim` or
  `\projlim`; write `\operatorname{…}` instead.
- Of the spacing commands only `\,` exists (`\;`, `\quad`, `\!` are
  literals); `\longrightarrow` and `\triangle` have no command yet
  (their glyphs are in the font subset since T55).
- No `\not` prefix: only the negated relations listed (`\notin`,
  `\nsubseteq`, `\ne`) exist. `\varnothing` looks like `\emptyset`.
- `\mathcal` and `\mathscr` have capitals only and look the same (the
  font has one script style and no script lowercase); no bold script,
  no bold fraktur, no bold monospace, no sans-serif Greek or italic
  sans, no `\R` /
  `\N` shortcuts and no double-struck Greek or italic (`ℾ ⅅ ⅆ`). No bold operators, fraction bars or roots, no `\pmb` and
  no `\boldmath`. `\mathit{diff}` has no text-italic ligatures or kerning
  (the letters are the math italic ones), and `\mathrm` has no upright
  lowercase Greek.
- No `\digamma` / `\Digamma`: the font subset has no digamma.
- A character the math font lacks (e.g. CJK in `\text{…}`) falls back to
  a monospaced cell.
- A formula stays on one line.
- Hebrew is there as math letters only: Hebrew text (`א`–`ת` in
  `\text{…}` or plain text) and right-to-left layout are not.
- Lines with scripts or fractions are taller than plain lines.
