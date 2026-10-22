# draw

A minimal, fast, keyboard-first scratch canvas for competitive programming.

Sketch a graph, a DP table or a geometry case while you think, then throw it
away. Nothing is saved, nothing needs configuring, and every action has a
single-key shortcut so your hands never leave the keyboard for long.

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
  (up to 64 × 64), with optional row and column indices.
- **Numbered nodes** — with numbering on, every new circle or box gets the
  next number: graphs in seconds.
- **Text blocks** — click and type notes with Greek letters and math
  symbols (`O(n log n)`, `α ≤ β`); no shortcut fires while you type, `Esc`
  finishes.
- **Formulas** — `$…$` in a text block is typeset TeX: scripts, fractions,
  roots, sums with limits (`$\sum_{k=1}^{n} k^2$`).
- **Fill** — bucket fills a shape or a single grid cell; the eraser clears
  fills or whole objects.
- **Snapping** — smart snap to alignment guides and shape outlines (arrows
  meet circles at their edge), plus an optional snap grid.
- **Editing** — select, move, `Alt`-drag to duplicate, copy / cut / paste,
  select all, delete, clear canvas.
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
sudo dnf install libX11 libXi libxkbcommon libglvnd-glx
# Debian / Ubuntu
sudo apt install libx11-6 libxi6 libxkbcommon0 libgl1
```

`SHA256SUMS` lists the checksum of every file; `draw.debug` holds the debug
symbols that turn a crash address back into a source line.

The window uses X11: on a Wayland session it runs through XWayland, which
must be installed (`xorg-x11-server-Xwayland` on Fedora, `xwayland` on
Debian / Ubuntu). If a display or a library is missing, draw says which
and what to install instead of opening a window;
`DRAW_SKIP_PREFLIGHT=1 draw` skips that check (for instance when libraries
live somewhere unusual, as on NixOS).

If draw crashes, it prints a short report and appends it to
`~/.local/state/draw/crash.log` (`$XDG_STATE_HOME/draw/crash.log` when that
is set). Please attach it to the issue.

macOS and Windows have no packages: build from source.

### Build from source

Requirements:

- Rust **1.85** or newer (edition 2024) — install with [rustup](https://rustup.rs).
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
# `--git https://github.com/UmBender/draw --tag v1.4.1` to skip cloning)
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
| `R` | Rectangle | `B` | Bucket (shape or grid cell) |
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
| `Ctrl+Backspace` | Clear canvas (undoable) | `Esc` | Cancel / deselect |

### View

| Input | Action | Input | Action |
|-------|--------|-------|--------|
| Wheel | Zoom at cursor | `0` | Reset view |
| Middle drag, `Space`+drag | Pan | `F` | Fit to content |
| `Tab` | Toolbar on / off | | |

### Helpers

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `X` | Smart snap on / off | `Shift+G` | Grid snap on / off |
| `N` | Numbering on / off (also grid indices) | `Shift+N` | Restart numbering at 1 |
| `→` / `←` | Grid columns + / − | `↓` / `↑` | Grid rows + / − |

### Gestures

| Gesture | Effect |
|---------|--------|
| `Shift` while drawing | Square / circle, 45° lines and arrows |
| `Alt` while drawing | No snapping |
| `Alt` + drag selection | Duplicate while moving |
| Right drag | Temporary eraser, from any tool |

### Typing in a text block

While a block is open every key types; only these act:

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `Esc`, `Ctrl+Enter` | Finish (one undo step) | `Ctrl+Z` | Throw the edit away |
| `Enter` | New line | `Tab` | Four spaces |
| `←` `→` `↑` `↓` | Move the caret | `Home` / `End` | Line start / end |
| `Backspace` / `Delete` | Delete a character | Click outside | Finish |
| `Ctrl+]` / `Ctrl+[` | Bigger / smaller text | | |

Edit a block again with the Text tool (click it) or Select (double-click
it). Its size starts from `[` / `]` and its colour from `1`–`6` when you
click; `Ctrl+]` / `Ctrl+[` resize it while you type. After you resize a
block, the next new blocks (and empty shapes or cells) start at that
on-screen size, even after a zoom; press `[` or `]` to go back to the
stroke-width size.

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
| `\sum_{i=1}^{n}`, `\prod`, `\min_i`, `\max`, `\lim` | Big operators with limits |
| `\int_0^1 f(x)\,dx`, `\iint`, `\iiint`, `\oint`, `\oiint` | Integrals, limits beside the sign; `\,` is a thin space |
| `\log \ln \gcd \lcm`, `a \bmod m`, `\pmod{m}` | Upright functions |
| `\alpha` … `\omega`, `\Gamma` … `\Omega` | Greek letters |
| `\mathcal{F}` | Calligraphic capitals |
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

There is no config file: shortcuts, palette and theme are fixed on purpose.
Four environment variables exist for troubleshooting, and two turn on
experimental features:

| Variable | Values | Effect |
|----------|--------|--------|
| `DRAW_MSAA` | `off`/`1`, `2`, `4` (default), `8` | Anti-aliasing samples; lower it on a very weak GPU |
| `DRAW_FRAME_TIMES` | `1`, `on`, `true`, `yes` | Log how long each re-render takes |
| `DRAW_EXPERIMENTAL` | feature names, comma-separated | Turn on experimental features (below) |
| `DRAW_BACKEND` | `x11` (default), `wayland` | Window backend on Linux; `wayland` is experimental (below) |
| `DRAW_SKIP_PREFLIGHT` | any non-empty value | Skip the startup check for a display and libraries |
| `DRAW_TABLET` | `auto` (default), `on`, `off` | Pen mode: hold pen taps still (see [Pen tablets](#pen-tablets)) |

```sh
DRAW_MSAA=off draw
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
other tools are unchanged. Pressure works on the default X11 backend and on
the native Wayland backend (`DRAW_BACKEND=wayland`), where the pen also
draws and both barrel buttons pan and erase; there the mouse is ignored
while the pen hovers over the window. Tilt, rotation and the eraser
end of the pen are not used. draw ships a patched copy of its window
library for this (`vendor/miniquad`).

### Experimental

Experimental features are off unless named in `DRAW_EXPERIMENTAL`. They
may change or disappear in any release.

- **`shape-text` — text in rectangles, ellipses and grid cells.** With the
  Text tool (`T`), click inside a rectangle, ellipse or grid cell to type
  into it; with Select, click again on the selected shape. In a grid,
  `Tab` / `Shift+Tab` go to the next / previous cell and `Enter` /
  `Shift+Enter` to the cell below / above. Text shrinks to fit, hides the
  shape's number, moves and copies with the shape; the fill eraser clears
  it.

```sh
DRAW_EXPERIMENTAL=shape-text draw
```

- **Native Wayland — `DRAW_BACKEND=wayland`.** By default draw is an X11
  app and runs on Wayland sessions through XWayland. With this variable it
  talks to the Wayland compositor directly, which can help on sessions
  without XWayland; if the Wayland connection fails it falls back to X11.
  It prints a one-line warning at start-up, is not yet tested on every
  compositor and may not be sharper under fractional scaling. Without
  `libdecor` the window may have no title bar on GNOME. Unset the
  variable to go back to X11.

```sh
DRAW_BACKEND=wayland draw
```

## Limitations

- Nothing is saved or exported — by design, it is a scratch pad.
- Text is one monospaced font with no wrapping, styling, or pasting from
  other apps; no IME input.
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
