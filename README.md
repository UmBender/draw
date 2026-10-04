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
- **Kanagawa Dragon** dark theme with a fixed six-colour palette.

## Install

### Requirements

- Rust **1.85** or newer (edition 2024) — install with [rustup](https://rustup.rs).
- Linux: X11, XInput and OpenGL development headers.

  ```sh
  # Fedora
  sudo dnf install libX11-devel libXi-devel mesa-libGL-devel
  # Debian / Ubuntu
  sudo apt install libx11-dev libxi-dev libgl1-mesa-dev
  ```

  On Wayland the window runs through XWayland.
- macOS and Windows build with no extra packages (macroquad supports them),
  but the app is developed and validated on Linux.

### Build and run

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

The release build is a single stripped binary of about 1 MB that opens a
window in well under 200 ms.

## Quick start

1. Press `C` and drag: an ellipse. Hold `Shift` for a circle.
2. Press `A` and drag between two circles: an arrow. Press `M` so it snaps
   to their outlines.
3. Press `N`, then draw more circles: they are numbered 1, 2, 3…
4. Press `G`, start dragging a table and tap `→` / `↓` to add columns and
   rows. Press `B` and click cells to fill them.
5. Made a mess? `Ctrl+Z`. Starting a new problem? `Ctrl+Backspace`, then
   `Shift+N`.

## Keyboard reference

Modifiers match exactly: `Shift+R` does nothing, `R` picks the rectangle.

### Tools

| Key | Tool | Key | Tool |
|-----|------|-----|------|
| `P` | Pen (smoothed) | `G` | Grid |
| `L` | Line | `V` | Select / move |
| `A` | Arrow | `E` | Eraser (on a fill: clears fills) |
| `R` | Rectangle | `B` | Bucket (shape or grid cell) |
| `C` | Circle / ellipse | `H` | Hand (pan) |

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
| `M` | Smart snap on / off | `Shift+G` | Grid snap on / off |
| `N` | Numbering on / off (also grid indices) | `Shift+N` | Restart numbering at 1 |
| `→` / `←` | Grid columns + / − | `↓` / `↑` | Grid rows + / − |

### Gestures

| Gesture | Effect |
|---------|--------|
| `Shift` while drawing | Square / circle, 45° lines and arrows |
| `Alt` while drawing | No snapping |
| `Alt` + drag selection | Duplicate while moving |
| Right drag | Temporary eraser, from any tool |

A printable one-page version lives in
[`docs/features/Contest cheat sheet.md`](docs/features/Contest%20cheat%20sheet.md).

## Recipes

- **Graph** — `N` on, `C` for nodes, `A` for directed edges (`L` for
  undirected), `M` so edges meet the nodes' outlines.
- **DP table** — `G`, size it with the arrows while dragging, `N` for
  indices, `B` to fill cells, `E` on a fill to clear it.
- **Geometry** — `Shift+G` for the snap grid, `L` and `C` with `Shift` for
  exact angles and circles.
- **Start over** — `Ctrl+Backspace`, then `Shift+N`; `Ctrl+Z` brings it
  all back.

## Configuration

There is no config file: shortcuts, palette and theme are fixed on purpose.
Two environment variables exist for troubleshooting:

| Variable | Values | Effect |
|----------|--------|--------|
| `DRAW_MSAA` | `off`/`1`, `2`, `4` (default), `8` | Anti-aliasing samples; lower it on a very weak GPU |
| `DRAW_FRAME_TIMES` | `1`, `on`, `true`, `yes` | Log how long each re-render takes |

```sh
DRAW_MSAA=off draw
```

## Limitations

- Nothing is saved or exported — by design, it is a scratch pad.
- No text tool; shapes carry numbers only.
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
