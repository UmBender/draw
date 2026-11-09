---
title: Feature proposal
task: "[[T72 Feature proposal]]"
date: 2026-11-09
status: awaiting-verdicts
tags: [proposal]
---

# Feature proposal

Which features should draw add before it stops adding features?

[[T53 Release 1.5.0]] is the last feature release, and
[[T50 Version 2.0.0 refactor]] adds none (owner, 2026-11-02). So an
accepted idea has to ship in 1.5.0, unless that decision changes. Each
accepted idea becomes a task that T53 waits for.

**Recommendation:** accept the five ideas in the
[short list](#recommended-short-list). Four are small and one is
medium, and together they turn pasted contest input into a table or a
graph. By estimate, they use about 17 KiB of the 128 KiB left in the
binary budget.

## Limits every idea is measured against

| Limit | Value | Today | Source |
|-------|-------|-------|--------|
| Shipped binary | under 2 MiB (2 097 152 bytes) | 1 966 008 bytes, **about 128 KiB free** | `scripts/package.sh` |
| Hit-test, eraser scan, culling, stroke finish, RDP | under 1 ms each | met | [[ADR-T14-1 Performance budgets]] |
| One snapped pointer move | under 4 ms | ≈ 1 ms | [[ADR-T14-1 Performance budgets]] |
| Overlay-only re-render | under 2 ms | met | [[ADR-T24-5 Redraw budgets]] |
| Full re-render | under 16 ms | see the ADR | [[ADR-T24-5 Redraw budgets]] |
| Idle CPU | ≈ 0 %: sleeps until an event | met | [[ADR-T12-1 Blocking event loop with cached frame]] |
| Runtime dependencies | each new one needs an ADR | macroquad, fontdue | [[ADR-0014 Minimal dependencies and no unsafe]], [[ADR-T32-1 Own glyph atlas rasterized with fontdue]] |
| Key bindings | each change needs an ADR | see [[Keymap]] | [[Keymap]] |

There is no start-up budget. The only measurement is 83–147 ms from
launch to window ([[T14 Performance and release validation]]), taken
before the fonts were added.

The binary and frame-time figures below are **estimates**, not
measurements. The task for each accepted idea measures its real cost.

Unbound keys today: bare `I J K L M O P Q U Y Z 7 8 9`, and most
`Ctrl` and `Alt` chords. `Ctrl+Shift+S` and `Ctrl+Shift+L` are taken.

## How the ideas are scored

- **Value** 1–5 for a competitive programmer during a contest.
- **Cost** = size (S 1, M 2, L 4) + risk to the *minimal, fast,
  keyboard-first* goal (low 0, medium 1, high 2).
- **Score** = value ÷ cost.

Value and risk are judgement calls, so treat a small difference in
score as a tie.

## Ranking

Fill *Verdict* with `accepted`, `later` or `rejected`.

| # | Idea | Value | Size | Risk | Cost | Score | Verdict |
|---|------|-------|------|------|------|-------|---------|
| 1 | C1 Grid from text | 5 | S | low | 1 | 5.00 | pending |
| 2 | K1 Paste text from the clipboard | 4 | S | low | 1 | 4.00 | pending |
| 3 | E1 Resize a committed grid | 4 | S | low | 1 | 4.00 | pending |
| 4 | F1 Growing brackets | 3 | S | low | 1 | 3.00 | pending |
| 5 | E2 Zero-based numbering | 3 | S | low | 1 | 3.00 | pending |
| 6 | E3 Bucket drag across cells | 3 | S | low | 1 | 3.00 | pending |
| 7 | C2 Graph or tree from an edge list | 5 | M | low | 2 | 2.50 | pending |
| 8 | E4 Resize handles | 4 | M | low | 2 | 2.00 | pending |
| 9 | F4 Labelled extensible arrows | 2 | S | low | 1 | 2.00 | pending |
| 10 | F5 Spacing commands | 2 | S | low | 1 | 2.00 | pending |
| 11 | E5 Align and distribute | 2 | S | low | 1 | 2.00 | pending |
| 12 | E6 Z-order keys | 2 | S | low | 1 | 2.00 | pending |
| 13 | K4 Export the view to PNG | 3 | S | medium | 2 | 1.50 | pending |
| 14 | C4 Coordinate plane from points | 3 | M | low | 2 | 1.50 | pending |
| 15 | F2 Matrices and cases | 3 | M | low | 2 | 1.50 | pending |
| 16 | K2 Autosave and crash recovery | 4 | M | medium | 3 | 1.33 | pending |
| 17 | C3 Segment tree drawing | 3 | M | medium | 3 | 1.00 | pending |
| 18 | F3 Display mode | 2 | M | low | 2 | 1.00 | pending |
| 19 | V3 Fading laser pointer | 2 | S | medium | 2 | 1.00 | pending |
| 20 | E8 Lock | 1 | S | low | 1 | 1.00 | pending |
| 21 | K3 Save and load | 3 | M | high | 4 | 0.75 | pending |
| 22 | V5 Ruler and protractor | 2 | M | medium | 3 | 0.67 | pending |
| 23 | E7 Group and ungroup | 2 | M | medium | 3 | 0.67 | pending |
| 24 | I2 Shape recognition | 2 | M | medium | 3 | 0.67 | pending |
| 25 | V4 Presenter mode | 1 | S | medium | 2 | 0.50 | pending |
| 26 | E9 Find text | 1 | M | low | 2 | 0.50 | pending |
| 27 | K5 Export to SVG | 2 | L | medium | 5 | 0.40 | pending |
| 28 | K6 Copy the selection as an image | 2 | L | medium | 5 | 0.40 | pending |
| 29 | V1 Pages | 2 | L | medium | 5 | 0.40 | pending |
| 30 | V2 Minimap | 1 | M | medium | 3 | 0.33 | pending |
| 31 | E10 Command palette | 1 | M | medium | 3 | 0.33 | pending |
| 32 | F6 User macros | 1 | M | medium | 3 | 0.33 | pending |
| 33 | I1 Pressure highlighter | 1 | M | medium | 3 | 0.33 | pending |
| 34 | F7 Commutative diagrams | 1 | L | high | 6 | 0.17 | pending |
| 35 | I3 Handwriting to TeX | 1 | L | high | 6 | 0.17 | pending |

## Recommended short list

| Build order | Idea | Size | Binary (estimate) | Why |
|-------------|------|------|-------------------|-----|
| 1 | K1 Paste text from the clipboard | S | 2 KiB | The problem statement's input gets onto the canvas. C1 and C2 build on it. |
| 2 | C1 Grid from text | S | 3 KiB | A DP table or a sample matrix appears at once instead of cell by cell. |
| 3 | E1 Resize a committed grid | S | 3 KiB | Needing one more column no longer means redrawing the table. |
| 4 | E2 Zero-based numbering | S | 1 KiB | Node numbers can match 0-indexed code, as grid indices already do. |
| 5 | C2 Graph or tree from an edge list | M | 8 KiB | Draws the most common contest picture from the input itself. |

C2 ranks seventh and is on the list ahead of F1 and E3, which score
higher. It is the only remaining idea with value 5, and F1 and E3 are
small improvements to things that already work. If you want a sixth,
take F1 or E3, in either order.

## Decisions for the owner

1. **Does crash recovery fit "Nothing is saved"?** [[Home]] and the
   README both say nothing is saved, by design. K2 only keeps a hidden
   recovery file, so it bends that promise. K3 ends it. A crash in the
   middle of a contest is the one loss draw can't undo today, so the
   recommendation is **K2 yes, K3 no**. K2 is left off the short list
   only because this decision is the owner's.
2. **Is five about right?** Every accepted idea delays 1.5.0 and adds
   code that 2.0.0 has to carry through its refactor. Fewer is fine.

## Ideas

## Keeping work

### K1 Paste text from the clipboard

- **Value** — 4. Sample input is on the screen in another window.
- **Problem** — Text must be retyped from the problem statement.
- **In use** — `Ctrl+V` while a text block or shape text is open pastes at the caret. Line breaks are kept.
- **Size** — S. miniquad already has `clipboard_get`; the edit session needs an insert and a size cap.
- **Cost** — Start-up: none; frame time: none, with pastes capped at 64 KiB; binary: about 2 KiB.
- **Risk** — low. It only adds a key to text edit mode.
- **Extends** — [[Text blocks#Limits]] (no pasting from the system clipboard).
- **Dependency** — None.
- **Budgets** — None at risk.

### K2 Autosave and crash recovery

- **Value** — 4. A crash during a contest currently loses everything.
- **Problem** — The work is only in memory, so a crash or a stray close destroys it.
- **In use** — Nothing to do while drawing. After a crash, `draw --recover` opens the last autosave.
- **Size** — M. The snapshot code already ships in release builds; the work is the trigger, an atomic write and the recovery path.
- **Cost** — Start-up: none unless `--recover` is used; frame time: one serialisation at most every 30 s, triggered by an input event, so idle stays at 0 %; binary: about 4 KiB.
- **Risk** — medium. It bends "nothing is saved", and disk writes must never stall a frame.
- **Extends** — [[Undo and redo#Limits]] (history lives in memory only), using [[Editor snapshots]].
- **Dependency** — None.
- **Budgets** — Overlay-only re-render (ADR-T24-5) at risk on large documents unless serialisation and the write leave the frame.

### K3 Save and load

- **Value** — 3. Useful after a contest, rarely during one.
- **Problem** — A drawing cannot be kept or reopened.
- **In use** — `Ctrl+S` writes the file named on the command line, or a timestamped file in the home directory. `draw file.draw` opens one. No file dialog.
- **Size** — M. Reuses the snapshot format, but saved files need the version compatibility snapshots do not have.
- **Cost** — Start-up: only when a file is opened; frame time: none except on save; binary: about 6 KiB.
- **Risk** — high. It reverses a stated principle and makes the file format a promise to keep through 2.0.0.
- **Extends** — [[Editor snapshots#Limits]] (no compatibility between versions) and README *Limitations*.
- **Dependency** — None.
- **Budgets** — None at risk.

### K4 Export the view to PNG

- **Value** — 3. Good for editorials and sharing, not for solving.
- **Problem** — A diagram cannot leave the app except by screenshot.
- **In use** — `Ctrl+E` writes `draw-<time>.png` of the current view to the home directory.
- **Size** — S. Reads the cached frame back and writes one file.
- **Cost** — Start-up: none; frame time: none except on export; binary: 30–90 KiB with the PNG encoder that macroquad pulls in, about 1 KiB with a small uncompressed writer.
- **Risk** — medium. macroquad's `export_png` panics if the write fails, and with `panic = "abort"` that loses the drawing, so it can't be used as is.
- **Extends** — [[Infinite canvas, pan and zoom]] and README *Limitations* (nothing is exported).
- **Dependency** — None; `image` and `png` are already in the tree through macroquad.
- **Budgets** — Binary budget at risk if the full encoder is linked; the uncompressed writer avoids it.

### K5 Export to SVG

- **Value** — 2. Only matters when publishing a diagram.
- **Problem** — PNG does not scale for printed notes.
- **In use** — `Ctrl+Shift+E` writes `draw-<time>.svg`.
- **Size** — L. Formulas need glyph outlines or embedded fonts to look the same elsewhere.
- **Cost** — Start-up: none; frame time: none except on export; binary: about 20 KiB.
- **Risk** — medium. A second rendering path that must match the first.
- **Extends** — [[Shapes]], [[TeX formulas]] and README *Limitations*.
- **Dependency** — None.
- **Budgets** — None at risk.

### K6 Copy the selection as an image

- **Value** — 2. Faster sharing than K4, but rarely needed in a contest.
- **Problem** — Pasting a diagram into a chat needs a file first.
- **In use** — `Ctrl+Shift+C` puts the selection on the system clipboard as a PNG.
- **Size** — L. miniquad's clipboard carries text only, so the vendored Wayland code needs a binary MIME path.
- **Cost** — Start-up: none; frame time: none except on copy; binary: 30–90 KiB for the encoder.
- **Risk** — medium. More patches to the vendored backend just before the 2.0.0 refactor.
- **Extends** — [[Editing tools#Limits]] (no system clipboard).
- **Dependency** — None, but a larger patch to the vendored miniquad ([[ADR-T35-1 Vendored miniquad with a pen tablet patch]]).
- **Budgets** — Binary budget at risk.

## Contest drawing

### C1 Grid from text

- **Value** — 5. DP tables, matrices and sample grids are everyday contest drawings.
- **Problem** — A grid's cells are filled one at a time, which is slow enough to skip.
- **In use** — Select a text block and press `Ctrl+T`: lines become rows, whitespace separates cells, and the block becomes a grid in one undo step. `Ctrl+Shift+V` does the same from the clipboard at the cursor. One line gives an array strip, and indices follow `N` as usual.
- **Size** — S. A tokeniser plus building the existing `Shape::Grid` with `texts`; capped at the 64 × 64 grid limit.
- **Cost** — Start-up: none; frame time: none beyond an ordinary grid; binary: about 3 KiB.
- **Risk** — low. It is one command that builds an existing shape.
- **Extends** — [[Grid tool]] and [[Text in shapes]].
- **Dependency** — None.
- **Budgets** — None at risk.

### C2 Graph or tree from an edge list

- **Value** — 5. Graphs and trees are the most frequent contest drawing.
- **Problem** — Placing nodes and connecting them by hand takes minutes.
- **In use** — Type or paste edges (`1-2 2-3`, `1>2` for directed, `u v w` lines for weights) into a text block, then press `Ctrl+L`. Numbered circles and arrows replace the block in one undo step. A tree gets layers from its first node; anything else goes on a circle.
- **Size** — M. A parser, two layouts, and building existing shapes.
- **Cost** — Start-up: none; frame time: none beyond the same shapes drawn by hand; binary: about 8 KiB.
- **Risk** — low. It produces ordinary shapes that all the tools can edit.
- **Extends** — [[Numbered nodes]] and [[Shapes]].
- **Dependency** — None.
- **Budgets** — None at risk.

### C3 Segment tree drawing

- **Value** — 3. Useful for one family of problems.
- **Problem** — Interval labels on a segment tree are tedious to draw.
- **In use** — Type `seg 8` in a text block and press `Ctrl+L`. A tree with `[l, r]` labels replaces it.
- **Size** — M. A new layout with text labels on nodes.
- **Cost** — Start-up: none; frame time: none beyond the same shapes; binary: about 4 KiB.
- **Risk** — medium. It opens the door to one template per data structure.
- **Extends** — [[Text in shapes]] and [[Numbered nodes]].
- **Dependency** — None.
- **Budgets** — None at risk.

### C4 Coordinate plane from points

- **Value** — 3. Helps geometry problems, which are a minority.
- **Problem** — Sample points must be plotted by hand and by eye.
- **In use** — Select a text block of `x y` lines and press `Ctrl+P`. Axes, tick labels and points replace it, scaled to fit.
- **Size** — M. Scale and tick selection, built from existing lines and text.
- **Cost** — Start-up: none; frame time: none beyond the same shapes; binary: about 5 KiB.
- **Risk** — low. It produces ordinary shapes.
- **Extends** — [[Snapping]] and [[Shapes]].
- **Dependency** — None.
- **Budgets** — None at risk.

## Formulas

### F1 Growing brackets

- **Value** — 3. Brackets around fractions and sums look wrong today.
- **Problem** — Only `\binom` and its relatives have brackets that grow.
- **In use** — `$\left( \frac{a}{b} \right)$`, as in LaTeX, with `.` for an invisible side.
- **Size** — S. `core::tex` already scales brackets for `\binom`; this adds parsing and any bracket pair.
- **Cost** — Start-up: none; frame time: none, layout is cached; binary: about 2 KiB and no new glyphs.
- **Risk** — low. It is contained in `core::tex`.
- **Extends** — [[TeX formulas#Limits]] (no `\left`/`\right`).
- **Dependency** — None.
- **Budgets** — None at risk.

### F2 Matrices and cases

- **Value** — 3. Matrix exponentiation and piecewise recurrences are common.
- **Problem** — A formula is one line, so a matrix cannot be written.
- **In use** — `$\begin{pmatrix} a & b \\ c & d \end{pmatrix}$`, also `bmatrix` and `cases`.
- **Size** — M. Environments, cell alignment, and a parser change. Needs F1 for the brackets.
- **Cost** — Start-up: none; frame time: none, layout is cached; binary: about 6 KiB.
- **Risk** — low. It is contained in `core::tex`.
- **Extends** — [[TeX formulas#Limits]] (no matrices or environments).
- **Dependency** — None.
- **Budgets** — None at risk.

### F3 Display mode

- **Value** — 2. Nicer sums, but inline ones are readable.
- **Problem** — Big operators are always inline size, with limits at the side.
- **In use** — `$$\sum_{k=1}^{n} k$$` on its own line, with limits above and below.
- **Size** — M. Display style through layout; the operators are scaled because the font subset has no display variants.
- **Cost** — Start-up: none; frame time: none, layout is cached; binary: about 3 KiB, as long as the glyphs are scaled.
- **Risk** — low. It is contained in `core::tex`.
- **Extends** — [[TeX formulas#Limits]] (no display mode).
- **Dependency** — None.
- **Budgets** — Binary budget at risk only if display glyphs are added; the full font is about 730 KB ([[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]).

### F4 Labelled extensible arrows

- **Value** — 2. Mostly for maths-heavy notes.
- **Problem** — Arrows have a fixed length and cannot carry a label.
- **In use** — `$A \xrightarrow{f} B$` and `\xleftarrow`.
- **Size** — S. A rule, a head glyph and a script above.
- **Cost** — Start-up: none; frame time: none, layout is cached; binary: about 1 KiB.
- **Risk** — low. It is contained in `core::tex`.
- **Extends** — [[TeX formulas#Limits]] (no labelled extensible arrows).
- **Dependency** — None.
- **Budgets** — None at risk.

### F5 Spacing commands

- **Value** — 2. Small, but a surprise today because they print literally.
- **Problem** — Only `\,` exists; `\;`, `\quad` and `\!` show up as text.
- **In use** — `\: \; \! \quad \qquad`, as in LaTeX.
- **Size** — S. A few table entries.
- **Cost** — Start-up: none; frame time: none; binary: under 1 KiB.
- **Risk** — low. It is contained in `core::tex`.
- **Extends** — [[TeX formulas#Limits]] (only `\,` exists).
- **Dependency** — None.
- **Budgets** — None at risk.

### F6 User macros

- **Value** — 1. Formulas in a scratch pad are short.
- **Problem** — Repeated notation must be retyped.
- **In use** — `$\newcommand{\R}{\mathbb{R}}$` in a block; applies to later blocks in the session.
- **Size** — M. Expansion, argument handling and invalidating the layout cache.
- **Cost** — Start-up: none; frame time: more cache misses when a macro changes; binary: about 4 KiB.
- **Risk** — medium. Hidden state links blocks to each other, and expansion needs a loop guard.
- **Extends** — [[TeX formulas#Limits]] (no macros).
- **Dependency** — None.
- **Budgets** — None at risk.

### F7 Commutative diagrams

- **Value** — 1. Very rare in competitive programming, and arrows on the canvas already do it.
- **Problem** — `tikz-cd` diagrams cannot be typeset.
- **In use** — `$\begin{tikzcd} A \arrow[r] & B \end{tikzcd}$`.
- **Size** — L. A 2D layout engine and arrow routing.
- **Cost** — Start-up: none; frame time: none, layout is cached; binary: about 15 KiB.
- **Risk** — high. A large subsystem for a niche use.
- **Extends** — [[TeX formulas#Limits]] (no commutative diagrams).
- **Dependency** — None.
- **Budgets** — None at risk.

## Canvas

### V1 Pages

- **Value** — 2. The infinite canvas and `F` already separate problems.
- **Problem** — Sketches for problems A, B and C share one space.
- **In use** — `Alt+1` to `Alt+9` switch pages; each has its own view.
- **Size** — L. Document, history, snapshot and render cache all change.
- **Cost** — Start-up: none; frame time: one full re-render per switch; binary: about 8 KiB.
- **Risk** — medium. Undo across pages needs new rules, and it touches the core just before the refactor.
- **Extends** — [[Infinite canvas, pan and zoom]] and [[Undo and redo]].
- **Dependency** — None.
- **Budgets** — None at risk.

### V2 Minimap

- **Value** — 1. `F` shows everything in one key.
- **Problem** — It is easy to lose your place on a large canvas.
- **In use** — `M` toggles a small overview in a corner.
- **Size** — M. A second view of the document, with its own cache.
- **Cost** — Start-up: none; frame time: an extra pass on each document change; binary: about 4 KiB.
- **Risk** — medium. Permanent UI, and work on every edit.
- **Extends** — [[Infinite canvas, pan and zoom]].
- **Dependency** — None.
- **Budgets** — Full re-render (ADR-T24-5) at risk on large documents.

### V3 Fading laser pointer

- **Value** — 2. Only for teaching or streaming.
- **Problem** — There is no way to point at something without drawing on it.
- **In use** — Hold `L` and move: a trail follows the cursor and fades.
- **Size** — S. An overlay trail with a timer.
- **Cost** — Start-up: none; frame time: redraws while the trail fades; binary: about 2 KiB.
- **Risk** — medium. It needs an animation clock in an app built to sleep between events.
- **Extends** — [[Infinite canvas, pan and zoom]] and [[ADR-0006 Redraw on demand]].
- **Dependency** — None.
- **Budgets** — Idle CPU only while the trail is fading; overlay-only re-render should hold.

### V4 Presenter mode

- **Value** — 1. `Tab` and `0` already cover most of it.
- **Problem** — The toolbar and zoom are distracting when presenting.
- **In use** — `P` hides the toolbar and locks the zoom.
- **Size** — S. A flag on the view and toolbar.
- **Cost** — Start-up: none; frame time: none; binary: under 1 KiB.
- **Risk** — medium. A new mode that can trap a user who pressed it by accident.
- **Extends** — [[Toolbar]] and [[Infinite canvas, pan and zoom]].
- **Dependency** — None.
- **Budgets** — None at risk.

### V5 Ruler and protractor

- **Value** — 2. Helps a few geometry problems.
- **Problem** — Lengths and angles can only be judged by eye.
- **In use** — Hold `U` over a line to read its length and the angle to the last one.
- **Size** — M. Overlay readouts and picking the shapes.
- **Cost** — Start-up: none; frame time: a hit-test per move while held; binary: about 3 KiB.
- **Risk** — medium. World units mean little on a free canvas, so it may not earn its key.
- **Extends** — [[Snapping]] and [[Drawing tools]].
- **Dependency** — None.
- **Budgets** — None at risk.

## Editing

### E1 Resize a committed grid

- **Value** — 4. Needing one more column is routine.
- **Problem** — A finished grid cannot change size, so it is redrawn with its contents retyped.
- **In use** — Select one grid, then the arrow keys add or remove columns and rows, as they do while drawing one. Text and fills stay in their cells; each press is one undo step.
- **Size** — S. A grid edit command that keeps `texts` and fills. The keys already exist.
- **Cost** — Start-up: none; frame time: none; binary: about 3 KiB.
- **Risk** — low. It reuses a gesture people already know.
- **Extends** — [[Grid tool#Limits]] (a committed grid cannot be resized).
- **Dependency** — None.
- **Budgets** — None at risk.

### E2 Zero-based numbering

- **Value** — 3. Much contest code is 0-indexed.
- **Problem** — Node numbers start at 1 while grid indices start at 0, and the counter cannot start anywhere else.
- **In use** — `Ctrl+N` switches the first number between 0 and 1 and restarts the counter.
- **Size** — S. A setting read by `core::numbering`, saved in snapshots.
- **Cost** — Start-up: none; frame time: none; binary: about 1 KiB.
- **Risk** — low. One key on an existing feature.
- **Extends** — [[Numbered nodes#Limits]] (no custom start).
- **Dependency** — None.
- **Budgets** — None at risk.

### E3 Bucket drag across cells

- **Value** — 3. Marking a DP path or a visited region is common.
- **Problem** — Each cell takes its own click.
- **In use** — With the bucket, press in a cell and drag: every cell passed over fills, in one undo step.
- **Size** — S. Extends the bucket's gesture; one hit-test against the grid per move.
- **Cost** — Start-up: none; frame time: one cell lookup per move; binary: about 1 KiB.
- **Risk** — low. It extends an existing gesture.
- **Extends** — [[Cell fill#Limits]] (no flood fill across several cells).
- **Dependency** — None.
- **Budgets** — None at risk.

### E4 Resize handles

- **Value** — 4. Getting a shape's size wrong currently means redrawing it.
- **Problem** — Moving is the only transform.
- **In use** — Drag a corner handle of the selection, or press `Alt+[` and `Alt+]` to scale it by 10 %.
- **Size** — M. Scaling for each of the shape kinds, handle hit-tests and undo.
- **Cost** — Start-up: none; frame time: handles drawn in the overlay; binary: about 6 KiB.
- **Risk** — low. It is expected behaviour, and handles appear only with a selection.
- **Extends** — [[Editing tools#Limits]] (no resize or rotate handles).
- **Dependency** — None.
- **Budgets** — None at risk.

### E5 Align and distribute

- **Value** — 2. Smart snap already handles most alignment while drawing.
- **Problem** — Nodes drawn quickly end up uneven.
- **In use** — With a selection, `Alt+arrow` aligns to that side, and `Alt+H` and `Alt+V` space items evenly.
- **Size** — S. Translations computed from bounds.
- **Cost** — Start-up: none; frame time: none; binary: about 2 KiB.
- **Risk** — low. It is only a command on the selection.
- **Extends** — [[Snapping#Limits]] (moving a selection does not snap).
- **Dependency** — None.
- **Budgets** — None at risk.

### E6 Z-order keys

- **Value** — 2. It matters when a fill hides something.
- **Problem** — Stacking order is fixed by drawing order, and the shape on top takes the bucket click.
- **In use** — `Ctrl+]` and `Ctrl+[` move the selection up or down one step; with `Shift`, to the top or bottom.
- **Size** — S. Reordering in the document, with undo.
- **Cost** — Start-up: none; frame time: one full re-render per press; binary: about 2 KiB.
- **Risk** — low. It is only a command on the selection.
- **Extends** — [[Cell fill#Limits]] (a shape on top takes the bucket click) and [[Shapes]].
- **Dependency** — None.
- **Budgets** — None at risk.

### E7 Group and ungroup

- **Value** — 2. Select-all and marquee cover most uses.
- **Problem** — A composite drawing has to be reselected each time.
- **In use** — `Ctrl+G` groups the selection, `Ctrl+Shift+G` ungroups.
- **Size** — M. Group ids across the document, selection, history and snapshot.
- **Cost** — Start-up: none; frame time: a group lookup per selection hit; binary: about 4 KiB.
- **Risk** — medium. A new relation between shapes that every tool must respect.
- **Extends** — [[Editing tools]].
- **Dependency** — None.
- **Budgets** — Hit-test (ADR-T14-1) should hold if the lookup stays constant-time.

### E8 Lock

- **Value** — 1. Rarely worth a key.
- **Problem** — A background grid gets moved or erased by accident.
- **In use** — `Ctrl+K` locks the selection, `Ctrl+Shift+K` unlocks everything.
- **Size** — S. A flag that selection and the eraser skip.
- **Cost** — Start-up: none; frame time: none; binary: about 1 KiB.
- **Risk** — low. It is only a flag.
- **Extends** — [[Editing tools]].
- **Dependency** — None.
- **Budgets** — None at risk.

### E9 Find text

- **Value** — 1. Scratch drawings are small enough to see.
- **Problem** — Text on a large canvas is hard to locate.
- **In use** — `Ctrl+F`, type, and `Enter` jumps to the next match.
- **Size** — M. A new input prompt plus search.
- **Cost** — Start-up: none; frame time: a linear scan per keystroke; binary: about 3 KiB.
- **Risk** — low. It stays out of the way until called.
- **Extends** — [[Text blocks]].
- **Dependency** — None.
- **Budgets** — None at risk.

### E10 Command palette

- **Value** — 1. The app has few commands and a one-page cheat sheet.
- **Problem** — New users cannot discover the shortcuts.
- **In use** — `Ctrl+Shift+P`, type a name, press `Enter`.
- **Size** — M. An input prompt, list drawing and fuzzy match.
- **Cost** — Start-up: none; frame time: overlay only while open; binary: about 5 KiB.
- **Risk** — medium. A second way to do everything, in an app whose point is single keys.
- **Extends** — [[Shortcuts and macros]] and [[Contest cheat sheet]].
- **Dependency** — None.
- **Budgets** — None at risk.

## Input

### I1 Pressure highlighter

- **Value** — 1. The six colours and fills already mark things.
- **Problem** — Marking over existing ink hides it.
- **In use** — `Y` selects a translucent pen whose width follows pressure.
- **Size** — M. Translucency through the batch and the cached layer.
- **Cost** — Start-up: none; frame time: blending over overlapping strokes; binary: about 2 KiB.
- **Risk** — medium. All colours are opaque by decision, and this changes that.
- **Extends** — [[Pen tablets]] and [[Theme and palette#Limits]] (all colours are opaque).
- **Dependency** — None, but it needs an ADR amending [[ADR-0012 Fixed palette and theme tokens]].
- **Budgets** — Full re-render (ADR-T24-5) at risk with many translucent strokes.

### I2 Shape recognition

- **Value** — 2. `C` and `R` are one key away.
- **Problem** — A shaky hand draws poor circles.
- **In use** — Right after a stroke, `Q` replaces it with the best-fitting line, rectangle or ellipse. One undo step.
- **Size** — M. Fitting and thresholds that take tuning.
- **Cost** — Start-up: none; frame time: none, it runs on the key press; binary: about 4 KiB.
- **Risk** — medium. A wrong guess costs more than redrawing.
- **Extends** — [[Anti-tremor strokes]] and [[Drawing tools]].
- **Dependency** — None.
- **Budgets** — None at risk.

### I3 Handwriting to TeX

- **Value** — 1. Typing TeX is faster for people who already know it.
- **Problem** — Formulas need the keyboard.
- **In use** — Select handwritten strokes and press `Ctrl+Shift+T`; a formula replaces them.
- **Size** — L. It needs a recognition model and an inference runtime.
- **Cost** — Start-up: model load; frame time: long inference pauses; binary: tens of MiB for the model.
- **Risk** — high. It contradicts *minimal*, *fast* and *few dependencies* at once.
- **Extends** — [[Pen tablets]] and [[TeX formulas]].
- **Dependency** — Yes: an inference runtime and a model, needing an ADR against [[ADR-0014 Minimal dependencies and no unsafe]].
- **Budgets** — Binary budget broken by a wide margin.
