# draw

[![CI](https://github.com/UmBender/draw/actions/workflows/ci.yml/badge.svg)](https://github.com/UmBender/draw/actions/workflows/ci.yml)

A minimal, fast, keyboard-first scratch canvas for competitive programming.

Sketch a graph, a DP table or a geometry case while you think, then throw it
away. There are no files to manage, nothing needs configuring, and every
action has a single-key shortcut so your hands never leave the keyboard for
long. If it crashes, `draw --recover` brings your drawing back.

- **Never breaks during a contest** — few features, one dependency
  ([macroquad](https://github.com/not-fl3/macroquad)), no `unsafe`, fuzzed
  core.
- **Fast on weak hardware** — sleeps until you touch it (≈ 0 % CPU idle),
  redraws only what changed, stays smooth with thousands of shapes on an
  8th-gen i5 iGPU.
- **Keyboard first** — every tool and action has a shortcut.
- **Shaky-hand friendly** — strokes are smoothed and simplified as you draw.

## Features

- **Infinite canvas** — pan with middle drag or `Space`+drag, zoom at the
  cursor with the wheel (0.05× – 20×), fit everything with `F`.
- **Drawing tools** — smoothed pen, line, arrow, rectangle, ellipse; `Shift`
  constrains to squares, circles and 45° angles.
- **Grid tool** — drag out a table, size it live with the arrow keys
  (up to 64 × 64), with optional row and column indices. Select a finished
  table and the arrows add or remove its columns and rows.
- **Numbered nodes** — with numbering on, every new circle or box gets the
  next number: graphs in seconds.
- **Text blocks** — click and type notes with Greek letters and math
  symbols (`O(n log n)`, `α ≤ β`); no shortcut fires while you type, `Esc`
  finishes.
- **Text in shapes** — type inside a rectangle, a circle or a grid cell
  (a DP table, named graph nodes); text shrinks to fit and moves with the
  shape. A numbered node keeps its number instead.
- **Formulas** — `$…$` in a text block is typeset TeX: scripts, fractions,
  roots, sums with limits (`$\sum_{k=1}^{n} k^2$`).
- **Fill** — bucket fills a shape or a grid cell, and a drag fills every
  cell it crosses; the eraser clears fills or whole objects.
- **Snapping** — smart snap to alignment guides and shape outlines (arrows
  meet circles at their edge), plus an optional snap grid.
- **Editing** — select, move, resize by the corner handles or with
  `Alt+[` / `Alt+]`, `Alt`-drag to duplicate, copy / cut / paste, select
  all, delete, clear canvas, raise / lower with `Ctrl+]` / `Ctrl+[`
  (`Shift` for top / bottom), align edges with `Alt+←/→/↑/↓`, lay out
  as a row / column without overlap with `Alt+H` / `Alt+V`.
- **Lock shapes** — `Ctrl+K` locks the selection so a background table or
  diagram can no longer be selected, moved or erased; it still takes fills
  and text. `Ctrl+Shift+K` unlocks everything.
- **Unlimited undo / redo** of every edit, including clearing the canvas.
- **Anti-tremor** — four smoothing levels (off, low, medium, high).
- **Anti-aliasing** — 4× MSAA by default.
- **Toolbar** — clickable tools, colours and helpers, hidden with `Tab`.
- **Pen tablets** — draw with a pen; pressure sets the stroke width, and
  taps are clean clicks while a tablet is attached.
- **Kanagawa Dragon** dark theme with a fixed six-colour palette.

## Install

### Prebuilt packages

Linux x86_64 packages run on any distribution with glibc 2.28 or newer
(RHEL / AlmaLinux 8, Debian 10, Ubuntu 20.04 and later). Each one adds draw
to the application launcher with its icon.

| Package | Install | Remove |
|---------|---------|--------|
| `draw-<version>-1.x86_64.rpm` | `sudo dnf install ./draw-*.rpm` | `sudo dnf remove draw` |
| `draw_<version>-1_amd64.deb` | `sudo apt install ./draw_*.deb` | `sudo apt remove draw` |
| `draw-<version>-x86_64.flatpak` | `flatpak install --user ./draw-*.flatpak` | `flatpak uninstall io.github.UmBender.draw` |
| `draw-<version>-x86_64-linux.tar.gz` | `tar xzf draw-*.tar.gz && draw-*/install.sh` | `draw-*/install.sh --uninstall` |

The `.rpm` and `.deb` pull in the libraries the window needs. The tarball
installs into `~/.local` (`--prefix DIR` for another place) and needs them
installed already:

```sh
# Fedora / RHEL
sudo dnf install libX11 libXi libxkbcommon libglvnd-glx libwayland-client libwayland-egl libdecor
# Debian / Ubuntu
sudo apt install libx11-6 libxi6 libxkbcommon0 libgl1 libwayland-client0 libwayland-egl1 libdecor-0-0
```

`SHA256SUMS` lists the checksum of every file; `draw.debug` holds the debug
symbols that turn a crash address back into a source line.

On a Wayland session the window talks to the compositor directly; the
`.rpm` and `.deb` recommend the Wayland libraries for it. Without them draw
falls back to X11 through XWayland. On an X11 session draw uses X11, which
is deprecated and will be removed in draw 2.0.0 (see
[Window backend](#window-backend)). If a display or a library is missing,
draw says which and what to install instead of opening a window;
`DRAW_SKIP_PREFLIGHT=1 draw` skips that check (for instance when libraries
live somewhere unusual, as on NixOS).

If draw crashes, it prints a short report and appends it to
`~/.local/state/draw/crash.log` (`$XDG_STATE_HOME/draw/crash.log` when that
is set). Please attach it to the issue.

macOS and Windows have no packages: build from source.

### Build from source

Requirements:

- Rust **1.99** or newer (edition 2024) — install with [rustup](https://rustup.rs).
- Linux: X11, XInput and OpenGL development headers.

  ```sh
  # Fedora
  sudo dnf install libX11-devel libXi-devel mesa-libGL-devel
  # Debian / Ubuntu
  sudo apt install libx11-dev libxi-dev libgl1-mesa-dev
  ```

- macOS and Windows build with no extra packages (macroquad supports them),
  but the app is developed and validated on Linux.

```sh
git clone https://github.com/UmBender/draw.git
cd draw
cargo run --release
```

Or install the binary into `~/.cargo/bin`:

```sh
cargo install --path .
draw
```

#### System-wide with `cargo install`

To install draw for every user under `/usr/local`, build as yourself and
copy the result as root. Do not run `cargo` with `sudo`: root usually has
no Rust toolchain, and the build would leave root-owned files in
`target/` and `~/.cargo`.

```sh
# From a clone (or replace `--path .` with
# `--git https://github.com/UmBender/draw --tag v1.4.3` to skip cloning)
cargo install --path . --locked --root "$PWD/target/install"

sudo install -Dm755 target/install/bin/draw /usr/local/bin/draw
# Optional: application launcher entry and icon
sudo install -Dm644 packaging/io.github.UmBender.draw.desktop \
    /usr/local/share/applications/io.github.UmBender.draw.desktop
sudo install -Dm644 packaging/icons/draw.svg \
    /usr/local/share/icons/hicolor/scalable/apps/io.github.UmBender.draw.svg
```

Cargo warns that `target/install/bin` is not in `PATH`; that is expected,
as the binary is copied out of it. Use `--root /opt/draw` and `/opt/draw/bin/draw` (or any prefix) the same
way. To upgrade, run the same commands again. To remove it:

```sh
sudo rm -f /usr/local/bin/draw \
    /usr/local/share/applications/io.github.UmBender.draw.desktop \
    /usr/local/share/icons/hicolor/scalable/apps/io.github.UmBender.draw.svg
```

The release build is a single stripped binary of about 1 MB that opens a
window in well under 200 ms.

To build the release packages yourself, run `scripts/package.sh`: it needs
only `podman` (plus `flatpak-builder` for the Flatpak bundle) and writes
everything to `dist/`.

## Quick start

1. Press `C` and drag: an ellipse. Hold `Shift` for a circle.
2. Press `A` and drag between two circles: an arrow. Press `X` so it snaps
   to their outlines.
3. Press `N`, then draw more circles: they are numbered 1, 2, 3…
4. Press `G`, start dragging a table and tap `→` / `↓` to add columns and
   rows. Press `B` and click cells to fill them.
5. Press `T`, click next to the table and type a note; `Enter` starts a
   new line, `Esc` finishes.
6. Made a mess? `Ctrl+Z`. Starting a new problem? `Ctrl+Backspace`, then
   `Shift+N`.

## Keyboard reference

Modifiers match exactly: `Shift+R` does nothing, `R` picks the rectangle.

### Tools

| Key | Tool | Key | Tool |
|-----|------|-----|------|
| `D` | Pen (smoothed) | `G` | Grid |
| `W` | Line | `V` | Select / move |
| `A` | Arrow | `E` | Eraser (on a fill: clears fills) |
| `R` | Rectangle | `B` | Bucket (shape, grid cell; drag across cells) |
| `C` | Circle / ellipse | `H` | Hand (pan) |
| `T` | Text | | |

### Style

| Key | Action |
|-----|--------|
| `1` `2` `3` `4` `5` `6` | Ink · red · green · blue · yellow · magenta |
| `[` / `]` | Thinner / thicker stroke |
| `S` | Anti-tremor: off → low (default) → medium → high |

### Edit

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `Ctrl+Z` | Undo | `Ctrl+Shift+Z`, `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy | `Ctrl+X` | Cut |
| `Ctrl+V` | Paste at cursor | `Ctrl+D` | Duplicate selection |
| `Ctrl+A` | Select all | `Del`, `Backspace` | Delete selection |
| `Ctrl+T` | Text block to grid | `Ctrl+Shift+V` | Grid from clipboard |
| `Ctrl+L` | Text block to graph | `Ctrl+P` | Text block to plot |
| `Ctrl+Backspace` | Clear canvas (undoable) | `Esc` | Cancel / deselect |
| `Ctrl+K` | Lock selection | `Ctrl+Shift+K` | Unlock every shape |
| `Alt+]` | Grow selection 10 % | `Alt+[` | Shrink selection 10 % |
| `Ctrl+]` | Raise selection one step | `Ctrl+[` | Lower selection one step |
| `Ctrl+Shift+]` | Selection to the top | `Ctrl+Shift+[` | Selection to the bottom |
| `Alt+←` | Align left edges | `Alt+→` | Align right edges |
| `Alt+↑` | Align top edges | `Alt+↓` | Align bottom edges |
| `Alt+H` | Lay out as a row, no overlap | `Alt+V` | Lay out as a column, no overlap |

### View

| Input | Action | Input | Action |
|-------|--------|-------|--------|
| Wheel | Zoom at cursor | `0` | Reset view |
| Middle drag, `Space`+drag | Pan | `F` | Fit to content |
| `Tab` | Toolbar on / off | `Ctrl+E` | Export the view to PNG in `~` |

### Helpers

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `X` | Smart snap on / off | `Shift+G` | Grid snap on / off |
| `N` | Numbering on / off (also grid indices) | `Shift+N` | Restart numbering at the first number |
| `Ctrl+N` | First number 1 / 0, restarts the counter | | |
| `→` / `←` | Grid columns + / − (of the one selected grid, if any) | `↓` / `↑` | Grid rows + / − (same) |

### Gestures

| Gesture | Effect |
|---------|--------|
| `Shift` while drawing | Square / circle, 45° lines and arrows |
| `Alt` while drawing | No snapping |
| `Alt` + drag selection | Duplicate while moving |
| Drag a corner handle (Select) | Resize; `Shift` keeps the proportions |
| Right drag | Temporary eraser, from any tool |

### Typing in a text block

While a block is open every key types; only these act:

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `Esc`, `Ctrl+Enter` | Finish (one undo step) | `Ctrl+Z` | Throw the edit away |
| `Enter` | New line | `Tab` | Four spaces |
| `←` `→` `↑` `↓` | Move the caret | `Home` / `End` | Line start / end |
| `Backspace` / `Delete` | Delete a character | Click outside | Finish |
| `Ctrl+]` / `Ctrl+[` | Bigger / smaller text | `Ctrl+V` | Paste text from the system clipboard |

Edit a block again with the Text tool (click it) or Select (double-click
it). Its size starts from `[` / `]` and its colour from `1`–`6` when you
click; `Ctrl+]` / `Ctrl+[` resize it while you type. After you resize a
block, the next new blocks (and empty shapes or cells) start at that
on-screen size, even after a zoom; press `[` or `]` to go back to the
stroke-width size.

### Typing in a shape

With the Text tool (`T`), click inside a rectangle, ellipse or grid cell
to type into it; with Select, click again on the selected shape. All the
keys above work, and in a grid cell:

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Next / previous cell (wraps around the grid) |
| `Enter` / `Shift+Enter` | Cell below / above |

Text shrinks to fit the shape, moves and copies with it, and works with
formulas; the fill eraser clears it. A **numbered** rectangle or circle
holds no text, its number is its content: clicking inside one with the
Text tool starts a free text block there. Grids keep text in their cells
even with indices on.

### Formulas in text blocks

Write `$…$` inside a text block for a formula in a small TeX subset. It is
typeset in Latin Modern Math — the LaTeX look, with italic variables —
once the block is finished, and shows its source while you edit:

```text
$dp_{i,j} = \min(dp_{i-1,j}, dp_{i,j-1}) + a_{i,j}$ in $O(nm)$
```

| Write | For |
|-------|-----|
| `x^2`, `x_i`, `x_{i,j}^{2k}` | Superscript, subscript (one character or `{group}`) |
| `\frac{a}{b}`, `\binom{n}{k}`, `\sqrt{x}` | Fraction, binomial, root |
| `\left( \frac{a}{b} \right)^2`, `\left. x \right|` | Brackets that grow to fit (`.` draws nothing) |
| `\begin{pmatrix} a & b \\ c & d \end{pmatrix}`, `matrix`, `bmatrix`, `vmatrix` | Matrices, on one line |
| `\begin{cases} 1 & n = 0 \\ 2 & n > 0 \end{cases}` | Piecewise definitions |
| `{n \choose k}`, `{n \brack k}`, `{n \brace k}`, `{a \over b}`, `{a \atop b}` | Infix forms: split their `{group}` (or the formula) in two |
| `\tbinom{n}{k}`, `\dbinom{n}{k}`, `\multiset{n}{k}` | Binomials (`\multiset` in double parentheses) |
| `x^{\underline{k}}`, `x^{\overline{k}}`, `a \nmid b`, `\#A`, `\lvert A \rvert` | Falling and rising factorials, does not divide, cardinality |
| `\sum_{i=1}^{n}`, `\prod`, `\min_i`, `\max`, `\lim` | Big operators with limits |
| `\bigcup_{i=1}^{n} A_i`, `\bigcap \bigwedge \bigvee \bigoplus \bigotimes \bigodot \biguplus \bigsqcup \coprod` | Big set and logic operators, limits above and below like `\sum` |
| `\int_0^1 f(x)\,dx`, `\iint`, `\iiint`, `\oint`, `\oiint` | Integrals, limits beside the sign; `\,` is a thin space |
| `a\,b \: c \; d \quad e \qquad f`, `a\!b`, `a\ b`, `a~b` | Spaces: thin, medium, thick, 1 em, 2 em; `\!` is negative; `\ ` and `~` one word space |
| `\inf \sup \liminf \limsup \det \Pr \gcd` | Named operators with limits below (`\limsup_{n\to\infty} a_n`) |
| `\log \ln \lcm \exp \lg \deg \dim \ker \hom \arg`, `a \bmod m`, `\pmod{m}` | Upright functions |
| `\operatorname{rank}`, `\operatorname*{argmin}_x` | Any other operator name; with `*` it takes limits |
| `\sin \cos \tan \cot \sec \csc`, `\arcsin \arccos \arctan`, `\sinh \cosh \tanh \coth` | Trigonometric and hyperbolic functions (`\sin^2\theta`) |
| `\arccot \arcsec \arccsc \sech \csch` | More functions, not in LaTeX's base set |
| `\alpha` … `\omega`, `\Gamma` … `\Omega` | Greek letters |
| `\varepsilon \vartheta \varkappa \varpi \varrho \varsigma \varphi` | Variant Greek letters |
| `\varGamma` … `\varOmega` | Italic Greek capitals |
| `\Alpha \Beta \Epsilon \Zeta \Eta \Iota \Kappa \Mu \Nu \Omicron \Rho \Tau \Chi` | Greek capitals that look like Latin letters (unicode-math names) |
| `\aleph \beth \gimel \daleth` | Hebrew letters (`\aleph_0`) |
| `\mathcal{F}` | Calligraphic capitals |
| `\mathrm{d}x`, `\mathit{diff}`, `\mathnormal{\Gamma}` | Upright roman letters, an italic word, italic Greek capitals (the innermost alphabet wins) |
| `\mathbf{v}`, `\boldsymbol{\alpha}`, `\bm{x}` | Bold: upright bold letters, digits and Greek capitals; bold italic Latin and Greek |
| `\mathbb{R}`, `\mathbb{Z}_p`, `\mathds{1}_A` | Blackboard bold capitals, lowercase and digits (`\mathds` is the same) |
| `\mathfrak{g}`, `\mathfrak{p}`, `\mathfrak{S}_n` | Fraktur capitals and lowercase |
| `\mathscr{L}`, `\mathscr{F}` | Script capitals (the same letters as `\mathcal`; the font has no script lowercase) |
| `\mathsf{G}`, `\mathtt{x1}` | Sans-serif and typewriter letters and digits (`\mathbf{\mathsf{x}}` is bold sans) |
| `\vec{F}`, `\hat{n}`, `\tilde{x}`, `\dot{x}`, `\ddot{x}`, `\bar{v}`, `\overline{AB}` | Accents centred over their argument |
| `\le \ge \ne \approx \equiv \cdot \times \div \pm \to \leftarrow \Rightarrow \iff \in \notin \subset \cup \cap \land \lor \neg \oplus \infty \ldots \cdots` | Relations and operators |
| `\lfloor \rfloor \lceil \rceil \{ \} \| \langle \rangle` | Brackets |
| `\partial \nabla` | Calculus, vectors |
| `\hbar \ell \Re \Im \AA \degree` | Letters, constants |
| `\forall \exists \emptyset \prime` | Logic, sets |
| `\vdots \ddots \vert \Vert` | Dots, bars |
| `\propto \sim \simeq \cong \ll \gg \lesssim \gtrsim \doteq \coloneqq \parallel \perp \mid` | More relations |
| `\rightarrow \leftrightarrow \Leftarrow \Leftrightarrow \uparrow \downarrow \mapsto \rightleftharpoons` | More arrows |
| `\mp \otimes \wedge \vee \circ \ast \star \bullet \dagger` | More binary operators |
| `\setminus \sqcup \sqcap \uplus`, `\supset \subseteq \supseteq \subsetneq \supsetneq \nsubseteq \ni`, `\complement \varnothing` | Sets (`A \setminus B`) |
| `\lnot \veebar \barwedge \top \bot \nexists`, `\vdash \models`, `\implies \impliedby` | Logic; `\implies` is spaced wider, as in amsmath |
| `\lvert x \rvert`, `\lVert v \rVert` | Absolute value and norm bars |
| `N \trianglelefteq G`, `\lhd \unlhd \rhd \unrhd \triangleleft \triangleright \trianglerighteq \ntriangleleft \ntriangleright` | Normal subgroup relations |
| `N \rtimes H`, `\ltimes \wr \odot \ominus \oslash \boxplus \boxtimes \amalg \diamond` | Semidirect, wreath and other algebra operators |
| `\Hom(A, B)`, `\End \Aut \Inn \Gal \Ker \im \coker \id \sgn \tr \rank` | Algebra operator names (upright); `\operatorname{char}` for the characteristic |
| `f\colon A \to B`, `F \dashv G`, `\Ob \Mor`, `\mathcal{C}^{\op}` | Category theory (`\colon` is punctuation); categories as `\mathbf{Set}` |
| `\hookrightarrow \twoheadrightarrow \rightarrowtail \hookleftarrow \twoheadleftarrow \leftarrowtail` | Monos and epis |
| `\longrightarrow \longleftarrow \longleftrightarrow \longmapsto \Longrightarrow \Longleftarrow \Longleftrightarrow` | Long arrows |
| `\rightrightarrows \leftleftarrows \leftrightarrows \rightleftarrows`, `\nearrow \searrow \swarrow \nwarrow`, `\Uparrow \Downarrow \Updownarrow \updownarrow` | Parallel, diagonal and double arrows |
| `A \xrightarrow{f} B`, `\xrightarrow[n\to\infty]{f_n}`, `\xleftarrow \xleftrightarrow \xRightarrow \xLeftarrow \xLeftrightarrow \xmapsto \xhookrightarrow` | Arrows that stretch to a label above (and an optional `[label]` below) |
| `\vb{F} = m\dddot{x}`, `\Box \square \triangle \angle`, `\ohm \micro`, `\grad \curl` | Classical physics: triple dot, d'Alembertian, angle, units, gradient and curl |
| `\dd{x}`, `\dv{f}{x}`, `\pdv{f}{x}`, `\dv{x}`, `\abs{x}`, `\norm{v}`, `\order{h^2}`, `\vu{n}` | `physics` shortcuts, drawn as their long form (`\frac{\mathrm{d}f}{\mathrm{d}x}`, …); `\div` stays `÷` |
| `\bra{\psi}`, `\ket{0}`, `\braket{\phi}{\psi}`, `\ketbra{0}{1}`, `\expval{H}`, `\mel{n}{H}{m}` | Dirac notation (`⟨φ\|ψ⟩`); brackets keep their normal size |
| `\comm{A}{B}`, `\acomm{A}{B}`, `A^\dag`, `\Tr\rho`, `\ket{0}\otimes\ket{1}` | Commutators `[A,B]`, anticommutators `{A,B}`, adjoint and trace |
| `\text{…}` | Upright text inside a formula |
| `\$` | A literal dollar sign (a lone `$` also stays plain) |

A formula stays on one line. Anything outside the subset is drawn as
typed in red, so a typo shows.

A printable one-page version lives in
[`docs/features/Contest cheat sheet.md`](docs/features/Contest%20cheat%20sheet.md).

## Recipes

- **Graph** — `N` on, `C` for nodes, `A` for directed edges (`W` for
  undirected), `X` so edges meet the nodes' outlines.
- **DP table** — `G`, size it with the arrows while dragging, `N` for
  indices, `B` to fill cells, `E` on a fill to clear it.
- **Geometry** — `Shift+G` for the snap grid, `W` and `C` with `Shift` for
  exact angles and circles.
- **Start over** — `Ctrl+Backspace`, then `Shift+N`; `Ctrl+Z` brings it
  all back.

## Configuration

There is no config file: shortcuts are fixed on purpose, and colours are
set at build time (see [Theme](#theme)).
Five environment variables exist for troubleshooting:

| Variable | Values | Effect |
|----------|--------|--------|
| `DRAW_MSAA` | `off`/`1`, `2`, `4` (default), `8` | Anti-aliasing samples; lower it on a very weak GPU |
| `DRAW_FRAME_TIMES` | `1`, `on`, `true`, `yes` | Log how long each re-render takes |
| `DRAW_BACKEND` | `wayland` (default on Wayland sessions), `x11` | Window backend on Linux; `x11` is deprecated and removed in 2.0.0 (see [Window backend](#window-backend)) |
| `DRAW_SKIP_PREFLIGHT` | any non-empty value | Skip the startup check for a display and libraries |
| `DRAW_TABLET` | `auto` (default), `on`, `off` | Pen mode: hold pen taps still (see [Pen tablets](#pen-tablets)) |

```sh
DRAW_MSAA=off draw
```

### Theme

Every colour is a constant in [`src/theme.rs`](src/theme.rs), grouped by
where it shows (canvas, drawing colours `1`–`6`, selection and marquee,
snapping, formulas, toolbar). There is no option, variable or file for
colours: edit the file and rebuild.

1. Change a value — `#RRGGBB` from any palette is written `0xRRGGBB`; an
   `_ALPHA` is an opacity from `0` (invisible) to `255` (opaque).
2. `cargo test theme` — checks the six drawing colours are distinct and
   that `INK` and `SELECTION_OUTLINE` stand out from `CANVAS` (3:1
   contrast). Update `theme_matches_kanagawa_dragon`, which pins the
   shipped values.
3. `cargo build --release` (or `scripts/package.sh` for the packages).

Example, a lighter background with darker ink — a light background needs
a dark ink:

```rust
pub const CANVAS: u32 = 0xf2ecbc;
pub const TEXT_ON_FILL: u32 = 0xf2ecbc;
pub const INK: u32 = 0x1f1f28;
```

### Window backend

On a Wayland session draw is a native Wayland app; if the connection to
the compositor fails or its libraries are missing, it falls back to X11
through XWayland. Without `libdecor` the window may have no title bar on
GNOME.

**X11 is deprecated and will be removed in draw 2.0.0**, both XWayland on
Wayland sessions and plain X11 sessions; 2.0.0 needs a Wayland session.
Until then `DRAW_BACKEND=x11` still forces X11 (for instance if text looks
blurrier under fractional scaling), and every start on X11 prints one line
saying so.

```sh
DRAW_BACKEND=x11 draw
```

### Pen tablets

A pen tablet (Wacom, Huion, XP-Pen, …) works like a mouse: the tip draws
with the current tool, hovering moves the cursor, and the barrel buttons do
what the middle (pan) and right (erase) mouse buttons do, if the compositor
passes them on.

A pen tip slips a little while it touches down and lifts, so a tap would
reach the tools as a tiny drag. When a tablet is attached, draw finds it in
`/proc/bus/input/devices`, prints one line naming it, and shows **pen** at
the bottom of the toolbar. In this pen mode a tap that stays within 16 px of
where it landed is a clean click: it selects with Select, places the caret
with Text, and draws nothing with Rect or Ellipse. Strokes start once the
pen has moved 16 px and then catch up. A mouse on the same machine gets the
same 16 px, so `DRAW_TABLET=off` turns pen mode off and `DRAW_TABLET=on`
forces it on.

```sh
DRAW_TABLET=off draw
```

Map the tablet to **one monitor** in your compositor, otherwise strokes are
stretched and taps slip further. On Hyprland put `output = current` (or a
monitor name) in the `input:tablet` section; on X11 use
`xinput map-to-output <pen device> <output>`.

**Pressure.** With the Pen tool, pressing harder draws a thicker line: a
normal (half) press gives the width chosen with `[` / `]`, a light touch
down to a quarter of it, a hard press up to twice. Mouse strokes and the
other tools are unchanged. Pressure works on native Wayland, where the pen
also draws and both barrel buttons pan and erase and the mouse is ignored
while the pen hovers over the window, and on the deprecated X11 backend.
Tilt, rotation and the eraser end of the pen are not used. draw ships a
patched copy of its window library for this (`vendor/miniquad`).

## Limitations

- There are no files to open or save — by design, it is a scratch pad.
  `Ctrl+E` exports the view to a PNG. The only thing kept is an autosave
  for crashes: `draw --recover` brings back the last one, and
  `draw --recover prev` the one before. It is written at most every 30 s,
  on input after a change, to the state directory
  (`~/.local/state/draw/`), and only works with the same version of draw.
- Text is one monospaced font with no wrapping or styling; no IME input.
- Shortcuts cannot be remapped.

## Development

```text
src/core/    pure logic (no macroquad): geometry, camera, shapes, document,
             undo history, tools, snapping, keymap — unit-tested and fuzzed
src/shell/   macroquad window, input translation, rendering, toolbar
tests/       fuzz harness (proptest) and performance budgets
docs/        Obsidian vault: architecture, decisions, features, tutorials
```

The core consumes input events and exposes state; the shell only translates
input and draws. See [`docs/architecture/Architecture.md`](docs/architecture/Architecture.md).

### Quality gates

```sh
scripts/check.sh          # fmt, clippy -D warnings, tests, rustdoc
scripts/check.sh --fuzz   # plus 20 000 random input sequences
```

Performance budgets (hit-test, culling, stroke finish, snapping on a
10 000-shape document) run in release mode:

```sh
cargo test --release --test perf -- --ignored --nocapture --test-threads=1
```

### Documentation

[`docs/`](docs/Home.md) is an [Obsidian](https://obsidian.md) vault and the
project's source of truth:

- [Task Board](docs/tasks/Task%20Board.md) — every task, from bootstrap to release.
- [Decision Log](docs/decisions/Decision%20Log.md) — append-only architecture decision records.
- [Feature Index](docs/features/Feature%20Index.md) — one note per user-visible feature.
- [Learning Path](docs/tutorials/Learning%20Path.md) — tutorials that rebuild the app step by step.

### Contributing

Work follows the process in [`docs/process/Workflow.md`](docs/process/Workflow.md):
one branch per task, specs and failing tests committed before the code, and
`scripts/check.sh` green before merging. Commit messages follow
[`docs/process/Commit Convention.md`](docs/process/Commit%20Convention.md).

## License

[MIT](LICENSE) © 2026 Gustavo Bender
