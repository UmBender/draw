# Changelog

All notable changes to draw are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/UmBender/draw/compare/v1.4.3...HEAD
[1.4.3]: https://github.com/UmBender/draw/compare/v1.4.2...v1.4.3
[1.4.2]: https://github.com/UmBender/draw/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/UmBender/draw/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/UmBender/draw/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/UmBender/draw/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/UmBender/draw/compare/v1.1.1...v1.2.0
[1.1.1]: https://github.com/UmBender/draw/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/UmBender/draw/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/UmBender/draw/releases/tag/v1.0.0
