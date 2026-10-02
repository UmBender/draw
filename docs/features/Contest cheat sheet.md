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
| `P` | Pen (smoothed) | `G` | Grid |
| `L` | Line | `V` | Select / move |
| `A` | Arrow | `E` | Eraser (on a fill: clears fills) |
| `R` | Rectangle | `B` | Bucket (shape or grid cell) |
| `C` | Circle / ellipse | `H` | Hand (pan) |

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
| `Ctrl+Backspace` | Clear canvas (undoable) | `Esc` | Cancel / deselect |

## View

| Input | Action | Input | Action |
|-------|--------|-------|--------|
| Wheel | Zoom at cursor | `0` | Reset view |
| Middle drag, `Space`+drag | Pan | `F` | Fit to content |
| `Tab` | Toolbar on / off | | |

## Helpers

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `M` | Smart snap on / off | `Shift+G` | Grid snap on / off |
| `N` | Numbering on / off (also grid indices) | `Shift+N` | Restart numbering at 1 |
| `→` / `←` | Grid columns + / − | `↓` / `↑` | Grid rows + / − |

## Gestures

| Gesture | Effect |
|---------|--------|
| `Shift` while drawing | Square / circle, 45° lines and arrows (grid snap only) |
| `Alt` while drawing | No snapping |
| `Alt` + drag selection | Duplicate while moving |
| Right drag | Temporary eraser, from any tool |

## Recipes

- **Graph**: `N` on, `C` for nodes (numbered 1, 2, …), `A` for directed
  edges; `M` makes edges meet the circles' outlines.
- **DP table**: `G`, `→`/`↓` to size it while dragging, `N` for indices,
  `B` to fill cells, `E` on a fill to clear it.
- **Start over**: `Ctrl+Backspace`, then `Shift+N`; `Ctrl+Z` brings it back.

## Limits

Shortcuts are fixed (no config file). Printing: export this note to PDF from
Obsidian, it fits one A4 page.
