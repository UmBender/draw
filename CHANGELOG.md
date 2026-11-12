# Changelog

All notable changes to draw are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- X11 is supported again: the deprecation notice and the planned removal
  are gone, and the README and the packages say so.
- The release script `scripts/release.sh` runs the checks and the
  end-to-end suite on Wayland and on X11 before it packages.
- The source has no `dyn` outside a short allow-list, and matches on enums
  list their variants; both are checked by the build.
- The code is a cargo workspace of five crates (`draw-types`, `draw-core`,
  `draw-gfx`, `draw-sys` and `draw`); the compiler enforces where `unsafe`
  may appear, and the module paths and the binary are unchanged.
- CI measures line coverage and fails when a module drops below its
  recorded value.
- The menus have a new look, after the status bar: the tool strip is made of
  translucent rounded islands, and the command palette and the shortcut
  sheet are islands with rounded rows, key caps and topic pills. The tools
  are still the letters of their keys, and every colour comes from the
  theme, with new `MENU_*` tokens.

## [1.6.0] - 2026-11-12

### Added

- Files: `Ctrl+S` writes every page to `draw file.draw` (the file named on
  the command line) or to `~/draw-<date>-<time>.draw`, and `draw file.draw`
  opens it. Newer or damaged files are refused, never overwritten.
- Six pages: `Alt+1`…`Alt+6` switch canvases, each with its own shapes,
  view and undo history. A switcher under the undo and redo buttons, one
  per row like the colours, shows 1–6 (current filled, pages with a drawing
  marked, empty dimmed) and a click goes to the page; it needs a window
  about 1000 px tall.
- `Ctrl+Shift+E` exports the whole drawing to an SVG in `~`.
- `Ctrl+Shift+C` copies the selection to the clipboard as an image.
- User macros in formulas: `\newcommand` and `\renewcommand` with `#1`…`#9`
  in any text block apply to every formula.
- `?` opens a shortcut sheet: `←`/`→` change page, `1`–`9` jump to a topic,
  `Esc` closes it.

### Changed

- `Z` forgives sloppier strokes: wobbly or gently curved lines, circles that
  do not close, overshoot or end in a tail, rectangles with rounded or
  missing corners. It still leaves scribbles, triangles and open arcs alone,
  and a stroke that fits a rectangle and an ellipse equally well.
- The toolbar, command palette, shortcut sheet and page switcher are drawn
  in the bundled text font on whole pixels, sharper than before.
- A graph typed with `Ctrl+L` that has no cycle is drawn in layers even
  when it is a forest: each tree from its first node, side by side. Only
  a cycle sends it to the circle.
- A self-loop in a graph (`1 1 12`) is drawn as a loop on its node, with
  its weight beyond it; before, it was skipped.
- Edges between the same two nodes (`1 3 4` with `3 1 2`) are drawn side
  by side, each weight on the outer side of its line, so the arrows and
  weights no longer overlap.

### Fixed

- The command palette and the shortcut sheet showed empty boxes for the
  arrow keys and the degree sign; they now read `Left`, `Right`, `Up`,
  `Down` and `45-degree`.
- On a circle layout, the weight of an edge running exactly from the
  upper right to the lower left could land on a node; it now always sits
  above the edge.

## [1.5.0] - 2026-11-11

### Added

- A diagnostics log that is always on: `~/.local/state/draw/draw.log`
  (at most 1 MiB) records start-up, devices and anything unusual, and
  the last moments before a crash, a doubled key or a key-repeat burst
  are saved to a dump file. `Ctrl+Shift+L` saves one on demand.
  `DRAW_LOG` sets how much is logged. Typed text is never recorded.
- Editor snapshots for bug reports: `Ctrl+Shift+S` saves the whole
  editor state to `~/.local/state/draw/snapshots/`, and
  `DRAW_SNAPSHOT=<file>` starts draw from one.
- More TeX in formulas: the Hebrew letters `\aleph` `\beth` `\gimel`
  `\daleth`; the rest of the Greek letters (`\varkappa`, `\varGamma`…
  `\varOmega`, `\Alpha`…`\Chi`); trigonometric and hyperbolic functions
  (`\sin`…`\arccsc`, `\sinh`…`\csch`); named operators with limits
  (`\limsup`, `\liminf`, `\inf`, `\sup`, `\det`, `\Pr`), more named
  functions (`\exp`, `\dim`, `\ker`, …) and `\operatorname{…}` /
  `\operatorname*{…}`; and the alphabets `\mathrm`, `\mathit` and
  `\mathnormal` next to `\mathcal`.
- More math alphabets: `\mathbf`, `\boldsymbol`, `\bm`, `\mathbb`,
  `\mathds`, `\mathfrak`, `\mathscr` (capitals only: the font has no
  script lowercase), `\mathsf` and `\mathtt`.
- More symbols in formulas: set and logic symbols (`\implies`,
  `\setminus`, `\lVert`…), big operators with limits (`\bigcup`,
  `\bigoplus`, `\coprod`…), algebra and group theory (`\rtimes`,
  `\trianglelefteq`, `\Hom`…) and category theory (`\hookrightarrow`,
  `\dashv`, `\colon`…).
- Combinatorics notation: the infix `\choose`, `\brack`, `\brace`,
  `\over` and `\atop`, plus `\tbinom`, `\dbinom`, `\multiset`,
  `\underline`, `\nmid` and `\#`.
- Physics notation: `\dd`, `\dv`, `\pdv`, `\abs`, `\norm`, `\order`,
  `\vb`, `\vu`, `\curl` and `\dddot`; quantum notation: `\bra`, `\ket`,
  `\braket`, `\ketbra`, `\expval`, `\mel`, `\comm`, `\acomm`, `\dag` and
  `\Tr`.
- Build-time themes: every colour is a constant in `src/theme.rs`; edit
  it and rebuild to restyle draw.
- Text from other apps: `Ctrl+V` in an open text block or shape text
  inserts the system clipboard's text at the caret.
- Grids from text: `Ctrl+T` turns the one selected text block into a grid
  (lines are rows, whitespace separates cells), and `Ctrl+Shift+V` builds
  one from the clipboard at the cursor.
- The arrow keys add or remove columns and rows of the one selected grid
  under the Select tool, keeping its contents.
- Zero-based numbering: `Ctrl+N` switches the first number between 1 and 0
  and restarts the counter.
- Graphs from text: `Ctrl+L` turns the one selected text block of edges
  into a layered tree or a circle graph, in one undo step.
- Coordinate planes from text: `Ctrl+P` turns the one selected text block
  of `x y` lines into axes, numbered ticks and filled points.
- Labels on plotted points: anything after a point's two numbers, such as
  `2 3 $C$`, is written just up and to the right of it.
- Graph input as pasted: a first line `graph:`, `tree:`, `dgraph:`,
  `parents:` or `parents0:` says how `Ctrl+L` reads the lines below it,
  so a problem's input needs no `>` and its count line is skipped.
- Spaces around edge separators: `1 > 3` and `1 - 3` read as `1>3` and
  `1-3`.
- Formulas in graphs: `$…$` node names and weights, such as
  `$s$ $t$ $\infty$`, are typeset inside the circles and beside the
  edges.
- Spacing in formulas: `\:`, `\;`, `\!`, `\quad`, `\qquad`, `\ ` and `~`.
- Labelled arrows in formulas: `\xrightarrow`, `\xleftarrow`,
  `\xleftrightarrow`, `\xRightarrow`, `\xLeftarrow`, `\xLeftrightarrow`,
  `\xmapsto` and `\xhookrightarrow` stretch to fit a label above and an
  optional `[label]` below.
- Stacking order: `Ctrl+]` / `Ctrl+[` raise or lower the selection past
  the nearest overlapping shape, `Ctrl+Shift+]` / `Ctrl+Shift+[` send it to
  the top or bottom.
- Align and arrange: `Alt+←/→/↑/↓` align the selection's edges, and
  `Alt+H` / `Alt+V` lay it out as a compact row or column without
  overlap.
- Weighted graph input: a weight after a pair (`1>3 7`, `1-2 $w$`), `0`
  as the root in `parents:`, and the `wtree:`, `wgraph:` and `wdgraph:`
  keywords.
- Segment trees: `segtree: 8` and `Ctrl+L` draw the tree of ranges
  `[1, 8]`, `[1, 4]`, … as squares, each with an empty box under it for
  the node's value (from `[0, 7]` after `Ctrl+N`).
- Dragging the bucket across a grid fills every cell it crosses, in one
  undo step; `Esc` cancels.
- `\left` and `\right` in formulas draw brackets that grow to fit their
  content.
- `matrix`, `pmatrix`, `bmatrix`, `vmatrix` and `cases` environments in
  formulas.
- Resize handles on the corners of a selection under the Select tool, and
  `Alt+]` / `Alt+[` to grow or shrink it by 10 %.
- `Ctrl+E` exports the view to `~/draw-<date>-<time>.png`.
- Autosave for crashes: `draw --recover` brings back the last autosave and
  `draw --recover prev` the one before.
- Display formulas: `$$…$$` is a formula on its own centred line, with
  operator limits above and below and full-size fraction parts;
  `\dfrac` and `\tfrac` pick the size by hand.
- Laser pointer: hold `Q` and move the pen or mouse; a red trail follows
  it and fades within a second. Nothing is drawn.
- Lock shapes: `Ctrl+K` locks the selection so a background table or
  diagram can no longer be selected, moved or erased (it still takes
  fills and text); `Ctrl+Shift+K` unlocks everything.
- Shape recognition: `Z` right after a pen stroke turns a shaky line,
  box or circle into a line, rectangle or ellipse; `Ctrl+Z` gives the
  stroke back.
- Minimap: while you pan, a corner map shows the whole drawing and where
  you are; hold `Space` and click or drag in it to jump.
- Command palette: `Ctrl+Shift+P` lists the commands with their keys;
  type to filter, `Enter` runs one, `Esc` closes it.

### Changed

- Native Wayland is the default on Wayland sessions; the .deb and .rpm
  recommend the Wayland libraries.
- Building from source needs Rust 1.99 or newer.
- Text in rectangles, ellipses and grid cells is always on, and
  `DRAW_EXPERIMENTAL` is gone. A numbered rectangle or ellipse holds no
  text: typing there makes a free text block, and numbering a shape
  that already holds text leaves it unnumbered.

### Deprecated

- The X11 backend (XWayland on Wayland sessions and plain X11 sessions),
  removed in 2.0.0; `DRAW_BACKEND=x11` prints a notice.

### Fixed

- Long sessions no longer run out of memory: undo history is capped at
  64 MiB, so moving a large selection again and again cannot grow
  memory until the system kills draw.
- `Ctrl-C` in the terminal and `SIGTERM` close the window in order and
  note it in the diagnostics log; a second `Ctrl-C` quits at once. A
  crash signal (segfault, abort) saves a diagnostics dump first.
- A pen line drawn in one movement stays one stroke (and one undo step)
  when the tablet briefly reports the tip lifting mid-line.
- On the native Wayland backend (experimental), each key press is typed
  once: a letter could show up three times when the compositor
  re-announced the keyboard.
- The text size remembered for new blocks stays on the size ladder,
  whatever the zoom was when the edit ended: zooming far in during an
  edit no longer makes the next block huge, or leaves a snapshot that
  draw cannot load.
- On the native Wayland backend, draw no longer crashes when a keyboard
  arrives without a usable keymap: the last good keymap stays.
- While a block with a formula is edited, no second outline ends in the
  middle of its source text.

## [1.4.3] - 2026-10-22

### Fixed

- The upper limit of an integral in a formula (`$\int_0^\pi$`) sits right
  beside the top of the sign instead of half a letter width away.

## [1.4.2] - 2026-10-22

### Fixed

- Pen pressure works on the native Wayland backend
  (`DRAW_BACKEND=wayland`): strokes no longer keep a constant width on
  compositors that also send the pen as mouse input, such as Hyprland.
  The mouse is ignored there while the pen hovers over the window.

## [1.4.1] - 2026-10-22

### Fixed

- Holding a pen's barrel button and touching the tablet pans or erases
  again instead of drawing, on tablets that report the tip just before the
  button (such as the HUION 420).

## [1.4.0] - 2026-10-21

### Added

- Integrals in formulas: `\int`, `\iint`, `\iiint`, `\oint` and `\oiint`
  are drawn large like `\sum`, with their limits beside the sign
  (`$\int_0^1 f(x)\,dx$`), and `\,` is a thin space.
- The symbols most used in physics: `\partial \nabla \hbar \ell \Re \Im
  \AA \degree`, `\forall \exists \emptyset \prime`, `\vdots \ddots \vert
  \Vert`, relations such as `\propto \sim \simeq \cong \ll \gg \perp
  \mid`, arrows such as `\rightarrow \leftrightarrow \mapsto
  \rightleftharpoons`, operators such as `\mp \otimes \circ \dagger`, and
  `\langle \rangle`.
- New text blocks start at the size of the last block resized with
  `Ctrl+]` / `Ctrl+[`, so a bigger font no longer has to be set again for
  every block; `[` / `]` go back to sizing text by the stroke width.
- Accents in formulas: `\vec`, `\hat`, `\tilde`, `\dot`, `\ddot`, `\bar`
  and `\overline` sit centred over their letter or group, leaning with
  italic letters (`$\vec{F} = m\ddot{x}$`, `$\hat{n}$`, `$\overline{AB}$`).

### Changed

- Pen, line and smart snap moved to the left hand, so they can be picked
  while the right hand holds the mouse or the pen: pen `P` → `D`, line
  `L` → `W`, smart snap `M` → `X`. `P`, `L` and `M` no longer do anything.
  The toolbar shows the new keys.

## [1.3.0] - 2026-10-20

### Added

- Pen pressure: with the Pen tool, pressing harder draws a thicker line. A
  normal (half) press gives the width chosen with `[` / `]`, a light touch
  down to a quarter of it and a hard press up to twice, and the width
  changes along the stroke. Works on the X11 and native Wayland backends.
  Mouse strokes and the other tools are unchanged.
- Pen tablets on the experimental native Wayland backend
  (`DRAW_BACKEND=wayland`): the pen now draws, and both barrel buttons
  work (middle = pan, right = erase).

### Changed

- draw now builds against a patched copy of its window library, miniquad
  0.4.11, in `vendor/miniquad` (pen tablet input; changes listed in
  `vendor/miniquad/PATCHED.md`). No new runtime libraries are needed.

### Fixed

- On the native Wayland backend the second barrel button of a pen, which
  Xwayland drops, now reaches draw (as the right button, erasing).

## [1.2.0] - 2026-10-20

### Added

- Pen tablets: with a tablet attached, draw switches to pen mode (shown as
  **pen** in the toolbar) and treats a tap that slips less than 16 px as a
  clean click, so pen taps select, place text and no longer draw tiny
  shapes. `DRAW_TABLET=off` / `on` overrides the detection.

## [1.1.1] - 2026-10-15

### Fixed

- Zoom on the experimental native Wayland backend (`DRAW_BACKEND=wayland`)
  was several times faster per wheel notch than on X11; one notch now
  zooms the same step on both.

## [1.1.0] - 2026-10-15

### Added

- Text blocks: the Text tool (`T`) places free-standing text with Greek
  letters and math symbols in an embedded monospaced font. No shortcut
  fires while typing; `Esc` finishes, `Ctrl+Z` cancels the edit, `Ctrl+[` /
  `Ctrl+]` resize the block. Blocks move, copy, erase, snap and undo like
  other shapes; edit one again with a Text click or a Select double-click.
- TeX formulas in text blocks: `$…$` is typeset once the block is
  finished — superscripts and subscripts, `\frac`, `\sqrt`, `\binom`,
  `\sum` / `\prod` / `\min` / `\max` / `\lim` with limits, Greek letters,
  relations, operators, brackets and `\text{…}`. The source shows while
  editing; anything outside the subset is drawn as typed in red.
  Formulas are set in Latin Modern Math, the LaTeX font — italic
  variables, upright operators and functions, TeX's spacing — and
  `\mathcal{…}` gives calligraphic capitals.
- Text in rectangles, ellipses and grid cells (experimental, off unless
  `DRAW_EXPERIMENTAL=shape-text`): click inside a shape or cell with the
  Text tool, or click a selected shape again, to type into it; `Tab` /
  `Enter` walk grid cells. Text shrinks to fit, hides the shape's number,
  moves and copies with the shape, and the fill eraser clears it.
- Linux packages: `scripts/package.sh` builds a portable tarball (with
  `install.sh`), a `.deb`, an `.rpm` and a Flatpak bundle that run on
  glibc 2.28 and newer (Debian 10+, Ubuntu 20.04+, RHEL 8+), with a
  desktop entry, icon and metainfo, and declare the X11 and OpenGL
  libraries draw needs.
- A missing display or library prints one line naming what to install
  and exits, instead of panicking; `DRAW_SKIP_PREFLIGHT=1` skips the
  check.
- A crash writes a report with the app version and build id to stderr
  and to `~/.local/state/draw/crash.log`; release packages ship split
  debug symbols (`draw.debug`) to trace it.
- Experimental native Wayland: `DRAW_BACKEND=wayland` runs draw without
  XWayland (falls back to X11 if Wayland is unavailable). X11 stays the
  default; the Flatpak grants both sockets.

### Changed

- Number labels use the embedded monospaced font.
- Text redraws much faster: a canvas full of text blocks no longer
  hitches for a tenth of a second after zooming to a new text size, and
  every redraw of 500 blocks stays within a 60 Hz frame.

## [1.0.0] - 2026-10-04

First release.

### Added

- Infinite canvas: pan with middle drag or `Space`+drag, zoom at the cursor
  with the wheel (0.05× – 20×), fit everything with `F`, reset with `0`.
- Drawing tools: smoothed pen, line, arrow, rectangle, ellipse; `Shift`
  constrains to squares, circles and 45° angles.
- Grid tool: drag out a table and size it live with the arrow keys (up to
  64 × 64), with optional row and column indices.
- Auto-numbering: new circles, boxes and grid indices are numbered while
  numbering is on; `Shift+N` restarts at 1.
- Bucket fill for shapes and single grid cells; the eraser clears fills or
  whole objects.
- Snapping: alignment guides, outline snapping (arrows meet circles at their
  edge) and an optional snap grid; `Alt` disables snapping while drawing.
- Editing: select, move, `Alt`-drag to duplicate, copy, cut, paste,
  duplicate, select all, delete and clear canvas.
- Unlimited undo and redo of every edit, including clearing the canvas.
- Anti-tremor stroke smoothing with four levels (off, low, medium, high).
- 4× MSAA anti-aliasing (`DRAW_MSAA` to change it).
- Toolbar with clickable tools, colours and helpers, hidden with `Tab`.
- Kanagawa Dragon dark theme with a fixed six-colour palette.
- Keyboard shortcut for every tool and action.
- Idle-friendly event loop (≈ 0 % CPU while idle), layered redraws and
  frame reuse during pan and zoom, smooth with thousands of shapes on an
  8th-gen i5 iGPU (`DRAW_FRAME_TIMES=1` logs frame times).

[Unreleased]: https://github.com/UmBender/draw/compare/v1.6.0...HEAD
[1.6.0]: https://github.com/UmBender/draw/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/UmBender/draw/compare/v1.4.3...v1.5.0
[1.4.3]: https://github.com/UmBender/draw/compare/v1.4.2...v1.4.3
[1.4.2]: https://github.com/UmBender/draw/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/UmBender/draw/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/UmBender/draw/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/UmBender/draw/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/UmBender/draw/compare/v1.1.1...v1.2.0
[1.1.1]: https://github.com/UmBender/draw/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/UmBender/draw/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/UmBender/draw/releases/tag/v1.0.0
