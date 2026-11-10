---
title: Contest cheat sheet
task: "[[T14 Performance and release validation]]"
adrs: ["[[ADR-T11-1 Exact modifier matching]]", "[[ADR-T16-2 Helper key bindings]]"]
tutorial: "[[14 Profiling and shipping a release build]]"
shortcuts: []
tags: [feature]
---

# Contest cheat sheet

One page to print and keep next to the keyboard. Source of truth:
`core::keymap::BINDINGS` and [[Keymap]]. Modifiers match exactly:
`Shift+R` does nothing.

## Tools

| Key | Tool | Key | Tool |
|-----|------|-----|------|
| `D` | Pen (smoothed) | `G` | Grid |
| `W` | Line | `V` | Select / move |
| `A` | Arrow | `E` | Eraser (on a fill: clears fills) |
| `R` | Rectangle | `B` | Bucket (shape, grid cell; drag across cells) |
| `C` | Circle / ellipse | `H` | Hand (pan) |
| `T` | Text (click, type, `Esc`) | | |

## Style

| Key | Action |
|-----|--------|
| `1` `2` `3` `4` `5` `6` | Ink · red · green · blue · yellow · magenta |
| `[` / `]` | Thinner / thicker stroke |
| `S` | Anti-tremor: off → low (default) → medium → high |

## Edit

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `Ctrl+Z` | Undo | `Ctrl+Shift+Z`, `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy | `Ctrl+X` | Cut |
| `Ctrl+V` | Paste at cursor | `Ctrl+D` | Duplicate selection |
| `Ctrl+A` | Select all | `Del`, `Backspace` | Delete selection |
| `Ctrl+T` | Text block to grid | `Ctrl+Shift+V` | Grid from clipboard |
| `Ctrl+L` | Text block to graph | `Ctrl+P` | Text block to plot |
| `Ctrl+Backspace` | Clear canvas (undoable) | `Esc` | Cancel / deselect |
| `Alt+]` | Grow selection 10 % | `Alt+[` | Shrink selection 10 % |
| `Ctrl+]` | Raise selection one step | `Ctrl+[` | Lower selection one step |
| `Ctrl+Shift+]` | Selection to the top | `Ctrl+Shift+[` | Selection to the bottom |
| `Alt+←` | Align left edges | `Alt+→` | Align right edges |
| `Alt+↑` | Align top edges | `Alt+↓` | Align bottom edges |
| `Alt+H` | Space horizontally, equal gaps | `Alt+V` | Space vertically, equal gaps |

## View

| Input | Action | Input | Action |
|-------|--------|-------|--------|
| Wheel | Zoom at cursor | `0` | Reset view |
| Middle drag, `Space`+drag | Pan | `F` | Fit to content |
| `Tab` | Toolbar on / off | `Ctrl+E` | Export the view to PNG in `~` |

## Helpers

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `X` | Smart snap on / off | `Shift+G` | Grid snap on / off |
| `N` | Numbering on / off (also grid indices) | `Shift+N` | Restart numbering at the first number |
| `Ctrl+N` | First number 1 / 0, restarts the counter | | |
| `→` / `←` | Grid columns + / − (of the one selected grid, if any) | `↓` / `↑` | Grid rows + / − (same) |

## Gestures

| Gesture | Effect |
|---------|--------|
| `Shift` while drawing | Square / circle, 45° lines and arrows (grid snap only) |
| `Alt` while drawing | No snapping |
| `Alt` + drag selection | Duplicate while moving |
| Drag a corner handle (Select) | Resize; `Shift` keeps the proportions |
| Right drag | Temporary eraser, from any tool |

## Typing in a text block

Every key types; only these act: `Esc` / `Ctrl+Enter` finish · `Ctrl+Z`
cancel · `Enter` new line · `Tab` 4 spaces · arrows, `Home`, `End` move ·
`Backspace`, `Delete` delete · `Ctrl+]` / `Ctrl+[` bigger / smaller ·
`Ctrl+V` paste text from the system clipboard.
Edit again: Text click, or Select double-click.

## Typing in a shape

`T` and click inside a box, circle or grid cell (or Select, click it
twice). In a cell: `Tab` / `Shift+Tab` next / previous, `Enter` /
`Shift+Enter` below / above. Numbered nodes keep their number: a click
inside one starts a free text block.

## Recipes

- **Graph**: `N` on, `C` for nodes (numbered 1, 2, …), `A` for directed
  edges; `X` makes edges meet the circles' outlines.
- **DP table**: `G`, `→`/`↓` to size it while dragging, `N` for indices,
  `B` to fill cells, `E` on a fill to clear it.
- **Start over**: `Ctrl+Backspace`, then `Shift+N`; `Ctrl+Z` brings it back.

## Limits

Shortcuts are fixed (no config file). Printing: export this note to PDF from
Obsidian, it fits one A4 page.
