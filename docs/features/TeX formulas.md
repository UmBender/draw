---
title: TeX formulas
task: "[[T30 Basic TeX formulas]]"
adrs: ["[[ADR-T30-1 TeX subset, box layout and metrics]]", "[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]", "[[ADR-T36-1 Integral class with scripts beside the sign]]", "[[ADR-T39-1 Accents from spacing glyphs with a slant skew]]", "[[ADR-T41-1 Integral superscript at the advance]]", "[[ADR-T64-1 Operatorname with a star parsed as one command]]", "[[ADR-T54-1 Math alphabets as a table of per-kind mappings]]", "[[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]", "[[ADR-T56-1 Accent skew follows a slant flag per math alphabet]]", "[[ADR-T59-1 One script alphabet for mathscr and mathcal]]", "[[ADR-T60-1 Bold sans-serif inside a bold alphabet]]", "[[ADR-T65-1 Wide relations padded with a thick space]]", "[[ADR-T71-1 Infix fraction commands split their group]]", "[[ADR-T69-1 Physics shortcuts expand to their long forms]]", "[[ADR-T89-1 Left and right brackets centred on the axis]]", "[[ADR-T90-1 Environments as one table node]]", "[[ADR-T99-1 Spacing commands as kerns outside the atom list]]", "[[ADR-T103-1 Extensible arrows from the arrow glyph and a ruled shaft]]"]
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

Combinatorics reads as in LaTeX. Besides `\binom{n}{k}` (and its
`\tbinom` / `\dbinom` forms, drawn the same since there is no display
mode), TeX's infix forms work: `${n \choose k}$`, the Stirling numbers
`${n \brack k}$` and `${n \brace k}$`, and the older `{a \over b}` (a
fraction, like `\frac{a}{b}`) and `{a \atop b}` (the same without the
bar). Each splits its `{group}` — or the whole formula — in two at the
command; a second one in the same group is drawn in red, as TeX rejects
it, so write `{a \over b} \over c` with braces. `\multiset{n}{k}` is
the multiset coefficient in double parentheses. Falling and rising
factorials are `$x^{\underline{k}}$` and `$x^{\overline{k}}$`;
`$a \nmid b$` is "does not divide", and cardinalities are `\#A` or
`\lvert A \rvert`. (Since [[T71 Combinatorics notation]],
[[ADR-T71-1 Infix fraction commands split their group]].)

Algebra and group theory read as in a first-year text: normal subgroups
`$N \trianglelefteq G$` (also `\lhd \unlhd \rhd \unrhd \triangleleft
\triangleright \trianglerighteq`, and `\ntriangleleft \ntriangleright`)
are relations; semidirect products `$N \rtimes H$`, `\ltimes`, the
wreath product `\wr` and `\odot \ominus \oslash \boxplus \boxtimes
\amalg \diamond` are binary operators. `\triangleleft` and
`\triangleright` draw the same `⊲ ⊳` as `\lhd \rhd`. The usual operator
names are upright words, spaced exactly like `\operatorname{…}`:
`$\Hom(A, B)$`, `\End \Aut \Inn \Gal \Ker \im \coker \id \sgn \tr
\rank`. There is no `\char` (it is a TeX primitive): write
`$\operatorname{char} k$`. Quotients, cosets, generated subgroups and
indices need nothing new: `$G/N$`, `$gH$`, `$\langle g \rangle$`,
`$[G:H]$`. (Since [[T67 Algebra and group theory symbols]].)

Category theory has its arrows: `$f\colon A \to B$` (`\colon` is
punctuation, so there is no space before it and a thin space after it,
unlike a typed `:`), monos and epis `$A \hookrightarrow B$`,
`\twoheadrightarrow \rightarrowtail` (and `\hookleftarrow
\twoheadleftarrow \leftarrowtail`), long arrows `\longrightarrow
\longleftarrow \longleftrightarrow \longmapsto \Longrightarrow
\Longleftarrow \Longleftrightarrow`, parallel arrows `\rightrightarrows
\leftleftarrows \leftrightarrows \rightleftarrows`, diagonal arrows
`\nearrow \searrow \swarrow \nwarrow` and `\Uparrow \Downarrow
\Updownarrow \updownarrow`, all relations. Adjunctions are
`$F \dashv G$`, isomorphisms `$A \cong B$`. `\Ob`, `\Mor` and `\op` are
upright names like `\operatorname{…}`: `$\Ob(\mathcal{C})$`,
`$\mathcal{C}^{\op}$`. Named categories have no command of their own:
write `$\mathbf{Set}$`, `$\mathbf{Grp}$`, `$\mathbf{Ab}$`,
`$\mathbf{Top}$`. `\Longrightarrow` is a plain relation (`\implies`
draws the same arrow with wider space). (Since
[[T68 Category theory symbols]].)

Classical physics has its notation: the jerk `$\vb{F} = m\dddot{x}$`
(`\dddot` is a third accent beside `\dot` and `\ddot`), the
d'Alembertian `$\Box\phi = 0$` (`\square` is the same `□`),
`\triangle`, angles `$\angle ABC$`, and the units `\ohm` (`Ω`) and
`\micro` (`µ`), which stay upright in every alphabet (`\degree` and
`\AA` were already there). The most used shortcuts of the LaTeX
`physics` package work and draw exactly their long form:
`$\dv{f}{x}$` is `\frac{\mathrm{d}f}{\mathrm{d}x}` and `\dv{x}` the
operator `\frac{\mathrm{d}}{\mathrm{d}x}`; `\pdv` the same with `∂`;
`$\int f\dd{x}$` puts a thin space and an upright `d` before `x`;
`\abs{x}`, `\norm{v}` and `\order{h^2}` are `\lvert x\rvert`,
`\lVert v\rVert` and `\mathcal{O}(h^2)`; `\vb{F}` is `\mathbf{F}` and
`\vu{n}` the unit vector `\hat{\mathbf{n}}`; `\grad` is `∇` and
`$\curl\vb{A}$` is `\nabla\times\mathbf{A}`. The second argument of
`\dv`, `\pdv` (and the one of `\dd`) is taken only when it is a
`{group}`, as in `physics`. `\div` stays `÷` (in `physics` it is the
divergence; write `\nabla\cdot`), and there is no `\laplacian` (write
`\nabla^2`). Brackets keep their normal size. (Since
[[T69 Classical physics notation]],
[[ADR-T69-1 Physics shortcuts expand to their long forms]].)

Quantum mechanics has the Dirac notation of `physics`, through the same
expansion table: `$\bra{\psi}$` is `\langle \psi\rvert`, `$\ket{0}$`
is `\lvert 0\rangle`, `$\braket{\phi}{\psi}$` is
`\langle \phi\vert \psi\rangle` (and `\braket{a}` is `⟨a|a⟩`),
`$\ketbra{0}{1}$` is `|0⟩⟨1|`, `$\expval{H}$` is `\langle H\rangle`
(`\expval{H}{\psi}` is `⟨ψ|H|ψ⟩`) and `$\mel{n}{H}{m}$` is
`\langle n\vert H\vert m\rangle`. The inner bars are ordinary, with no
space around them, as `physics` draws them. Commutators
`$\comm{A}{B}$` and anticommutators `$\acomm{A}{B}$` are `[A,B]` and
`\{A,B\}`; the adjoint is `$A^\dag$` (`\dag` is `†`, as `\dagger`), the
trace `$\Tr\rho$` beside `\tr`, and products of kets use `\otimes`:
`$\ket{0}\otimes\ket{1}$`. Brackets keep their normal size. (Since
[[T70 Quantum notation]].)

Brackets grow to fit what they enclose with `\left` and `\right`:
`$\left( \frac{a}{b} \right)^2$`, `$\left[ \sum_{i=1}^{n} x_i \right]$`.
Each takes one of `( ) [ ] \{ \} | \| \langle \rangle \lfloor \rfloor
\lceil \rceil`, the two sides need not match, and `.` draws nothing on
its side, as in `$\left. \frac{x}{2} \right|_0^1$`. Pairs nest, and a
script after `\right` attaches to the closing bracket. Short content
keeps the brackets at their normal size. A `\left` without its `\right`
in the same group, or the reverse, is drawn in red, and so is one
without a valid delimiter. The font has no size variants, so a tall
bracket is the glyph scaled up, up to 12 times. (Since
[[T89 Growing brackets]],
[[ADR-T89-1 Left and right brackets centred on the axis]].)

Matrices and piecewise definitions are environments, written on one line
with `&` between cells and `\\` between rows:
`$\begin{pmatrix} a & b \\ c & d \end{pmatrix}$`, `bmatrix`, `vmatrix`,
plain `matrix` without brackets, and
`$f(n) = \begin{cases} 1 & n = 0 \\ n f(n-1) & n > 0 \end{cases}$`.
Matrix columns are centred, `cases` columns start at the left edge behind
one growing brace, and the block is centred on the math axis. A short row
leaves its missing cells empty, a trailing `\\` adds no row, and cells
may hold any formula, even another matrix. An unknown environment, a
missing or misspelt `\end`, or more than 64 rows or columns shows
`\begin{…}` and `\end{…}` in red, with what is between them drawn as
usual. (Since [[T90 Matrices and cases]],
[[ADR-T90-1 Environments as one table node]].)

Spacing commands space a formula as in LaTeX: `\,` a thin space (1/6
em), `\:` a medium space (4/18 em), `\;` a thick space (5/18 em),
`\quad` one em, `\qquad` two, and `\ ` and `~` one interword space
(1/3 em). `\!` is a negative thin space (−3/18 em) that pulls its
neighbours together: `$a\quad b\!c\;d$`. They shrink in scripts like
the other spaces, and they are not atoms, so `$a\;+\;b$` keeps the
space of the binary `+` and adds the explicit spaces to it, as TeX
does. A run of `\!` stops at the start of its formula or group: a
formula never gets narrower than nothing, and `\!` at the very start
does nothing. (Since [[T99 Spacing commands]],
[[ADR-T99-1 Spacing commands as kerns outside the atom list]].)

Arrows stretch to fit a label: `$A \xrightarrow{f} B$`, and with an
optional label under the arrow, `$A \xrightarrow[n\to\infty]{f_n} B
\xleftarrow{g} C$`. The eight commands are `\xrightarrow \xleftarrow
\xleftrightarrow \xRightarrow \xLeftarrow \xLeftrightarrow \xmapsto
\xhookrightarrow`. Labels are set at script size, centred above and
under the arrow; the arrow is as long as the wider label plus half an
em on each side, and never shorter than its plain arrow. With empty
labels (`\xrightarrow{}`) it is exactly the plain arrow. It is a
relation on the math axis, spaced like `\rightarrow`, and labels may
hold any formula, another arrow too. A stretched arrow is the font's
arrow head with a ruled shaft (two rules for the double arrows, a bar
for `\xmapsto`, a drawn hook for `\xhookrightarrow`). The `{…}` label
is required (a single token such as `\xrightarrow f` works, as for
other commands); without it, or with a `[` that has no `]`, the command
is drawn in red. (Since [[T103 Labelled extensible arrows]],
[[ADR-T103-1 Extensible arrows from the arrow glyph and a ruled shaft]].)

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Start / end a formula | `$` … `$` on one line |
| Literal dollar sign | `\$` (a lone `$`, as in `costs $5`, also stays plain) |
| Superscript / subscript | `x^2`, `x_i`, `x_{i,j}^{2k}` |
| Fraction, binomial, root | `\frac{a}{b}`, `\binom{n}{k}`, `\sqrt{x}` (`\frac12` works too) |
| Infix fractions | `{n \choose k}`, `{n \brack k}`, `{n \brace k}`, `{a \over b}`, `{a \atop b}` (split their group) |
| More binomials | `\tbinom{n}{k}`, `\dbinom{n}{k}` (as `\binom`), `\multiset{n}{k}` (double parentheses) |
| Combinatorics | `x^{\underline{k}}`, `x^{\overline{k}}`, `a \nmid b`, `\#A`, `\lvert A \rvert` |
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
| Overline, underline | `\overline{AB}`, `\underline{AB}` (as wide as their argument) |
| Relations, operators | `\le \ge \ne \approx \equiv \cdot \times \div \pm \to \leftarrow \Rightarrow \iff \in \notin \subset \cup \cap \land \lor \neg \oplus \infty \ldots \cdots` |
| Brackets | `\lfloor \rfloor \lceil \rceil \{ \} | \langle \rangle` |
| Growing brackets | `\left( \frac{a}{b} \right)^2`, with `( ) [ ] \{ \} | \| \langle \rangle \lfloor \rfloor \lceil \rceil`, or `.` for none |
| Matrices | `\begin{pmatrix} a & b \\ c & d \end{pmatrix}`, also `matrix`, `bmatrix`, `vmatrix` |
| Piecewise definitions | `\begin{cases} 1 & n = 0 \\ 2 & n > 0 \end{cases}` |
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
| Group relations | `N \trianglelefteq G`, `\lhd \unlhd \rhd \unrhd \triangleleft \triangleright \trianglerighteq \ntriangleleft \ntriangleright` |
| Algebra operators | `N \rtimes H`, `\ltimes \wr \odot \ominus \oslash \boxplus \boxtimes \amalg \diamond` |
| Algebra names (upright) | `\Hom(A, B)`, `\End \Aut \Inn \Gal \Ker \im \coker \id \sgn \tr \rank`; `\operatorname{char}` |
| Category theory | `f\colon A \to B`, `F \dashv G`, `\Ob \Mor`, `\mathcal{C}^{\op}`, `\mathbf{Set}` |
| Monos, epis, long arrows | `\hookrightarrow \twoheadrightarrow \rightarrowtail` (and `\hookleftarrow \twoheadleftarrow \leftarrowtail`), `\longrightarrow \longleftarrow \longleftrightarrow \longmapsto \Longrightarrow \Longleftarrow \Longleftrightarrow` |
| Parallel, diagonal, double arrows | `\rightrightarrows \leftleftarrows \leftrightarrows \rightleftarrows`, `\nearrow \searrow \swarrow \nwarrow`, `\Uparrow \Downarrow \Updownarrow \updownarrow` |
| Labelled arrows (stretch to the label) | `A \xrightarrow{f} B`, `\xrightarrow[n\to\infty]{f_n}`, `\xleftarrow \xleftrightarrow \xRightarrow \xLeftarrow \xLeftrightarrow \xmapsto \xhookrightarrow` |
| Classical physics | `\dddot{x}`, `\Box \square \triangle \angle`, units `\ohm \micro \degree \AA`, `\grad \curl` |
| Physics shortcuts | `\dd{x}`, `\dv{f}{x}`, `\dv{x}`, `\pdv{f}{x}`, `\pdv{x}`, `\abs{x}`, `\norm{v}`, `\order{h^2}`, `\vb{F}`, `\vu{n}` (each draws its long form) |
| Quantum notation | `\bra{\psi}`, `\ket{0}`, `\braket{\phi}{\psi}`, `\ketbra{0}{1}`, `\expval{H}`, `\mel{n}{H}{m}`, `\comm{A}{B}`, `\acomm{A}{B}`, `A^\dag`, `\Tr\rho` |
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
Shortcut commands (`\dv`, `\abs`, …) are rows of the expansion table
`EXPANSIONS`: required arguments, optional `{group}` arguments and a
template of subset source per argument count. The parse tree keeps the
command as typed; at layout `expand` parses the template, fills its
`#n` slots with the argument nodes and splices the result among the
neighbours, so a shortcut draws its long form's ink
([[ADR-T69-1 Physics shortcuts expand to their long forms]]).
`core::text_layout::typeset` caches the layout per text, and
`text_layout::bounds` gives every shape query (hit-test, culling,
snapping, selection) the typeset box; the edit session keeps the source
box (`source_bounds`). `shell::render` draws the glyphs through the T32
atlas — keyed by face, one font per face — at their own scales and the
fraction bars, root signs and arrow shafts as quads
of the same batch.

## Limits

- No display mode (`$$…$$`) or macros. The only environments are
  `matrix`, `pmatrix`, `bmatrix`, `vmatrix` and `cases`, up to 64 rows
  and columns: no `array` with a column spec, `smallmatrix` or `dcases`.
  A matrix of more than about a dozen rows is taller than its brackets,
  which stop growing at 12 times their size.
- Brackets grow only with `\left`/`\right`, around matrices and `cases`,
  and in `\binom`, `\multiset` and the
  infix `\choose \brack \brace`. No `\middle`, and `\vert`, `\lvert` and
  the like are not delimiters after `\left` (use `|` or `\|`). No
  `\genfrac`, `\above` or `\…withdelims`.
- Accents do not grow: no `\widehat`, `\widetilde`, `\overrightarrow`,
  and no `\underbrace`. An accent takes
  one letter, a symbol or a `{group}`, not another accent without braces
  (`\hat{\vec{x}}` works, `\hat\vec x` does not). Their horizontal
  placement over italic letters is approximated (the font's top-accent
  points are not embedded).
- No `\limits` / `\nolimits`; integrals and big operators are always
  the inline size.
- No `\DeclareMathOperator`, `\varliminf`, `\varlimsup`, `\injlim` or
  `\projlim`; write `\operatorname{…}` instead.
- No `\hspace`, `\kern`, `\mkern`, `\enspace` or `\thinspace`: the
  spacing commands are `\, \: \; \! \quad \qquad`, `\ ` and `~`. A `\!`
  cannot hang left of its formula.
- Of the `physics` package only the shortcuts listed exist: no
  `\laplacian`, no divergence `\div` (it stays `÷`), no `[n]` order
  (`\dv[2]{f}{x}`; write `\frac{\mathrm{d}^2 f}{\mathrm{d}x^2}`), no
  starred forms, no `\qty`, `\eval`, `\vdot`, `\cross`, `\fdv`, `\var`
  or `\mqty`.
- No commutative diagrams (`tikz-cd`). Of the extensible arrows only the
  eight `\x…` commands listed exist (no `\xtwoheadrightarrow`,
  `\xlongequal` or harpoons), and no `\overset`, `\underset`,
  `\stackrel` or `\overrightarrow`. The hook of a stretched
  `\xhookrightarrow` is drawn with rules, not the font's curve.
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
- A formula stays on one line, matrices and `cases` included.
- Hebrew is there as math letters only: Hebrew text (`א`–`ת` in
  `\text{…}` or plain text) and right-to-left layout are not.
- Lines with scripts or fractions are taller than plain lines.
