---
tags: [architecture]
task: "[[T11 Keymap and macros]]"
status: implemented
---

# Keymap

Bindings implemented by [[T11 Keymap and macros]] (`src/core/keymap.rs`); modifiers
match exactly ([[ADR-T11-1 Exact modifier matching]]). Changes require an ADR.

## Tools (single key, no modifier)

| Key | Tool |
|-----|------|
| `P` | Pen (freehand, smoothed) |
| `L` | Line |
| `A` | Arrow |
| `R` | Rectangle |
| `C` | Circle / ellipse |
| `E` | Eraser (removes whole objects) |
| `B` | Bucket (fills the clicked rectangle/ellipse) |
| `V` | Select / move |
| `H` | Hand (pan) |
| `G` | Grid (table of cells) |

## Style

| Key | Action |
|-----|--------|
| `1`–`6` | Pick colour from the palette |
| `[` / `]` | Thinner / thicker stroke |
| `S` | Cycle anti-tremor strength (off → low → medium → high) |

## Edit

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z`, `Ctrl+Y` | Redo |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copy / cut / paste at cursor |
| `Ctrl+D` | Duplicate selection (macro: copy + paste with offset) |
| `Ctrl+A` | Select all |
| `Delete`, `Backspace` | Delete selection |
| `Ctrl+Backspace` | Clear canvas (undoable) |
| `Esc` | Cancel current gesture / clear selection |

## View

| Input | Action |
|-------|--------|
| Mouse wheel | Zoom towards cursor |
| Middle drag, `Space` + drag | Pan (from any tool) |
| `0` | Reset view (origin, zoom 1) |
| `F` | Fit view to all content |
| `Tab` | Show / hide toolbar |

## Helpers

Added by [[T16 Shape model v2 and helper skeleton]]
([[ADR-T16-2 Helper key bindings]]). None of these cancels a gesture.

| Key | Action |
|-----|--------|
| `M` | Smart snap on / off |
| `Shift+G` | Grid snap on / off |
| `N` | Auto-numbering on / off |
| `Shift+N` | Restart numbering at 1 |
| `→` / `←` | Grid: one more / one less column (1–64, live while dragging) |
| `↓` / `↑` | Grid: one more / one less row (1–64, live while dragging) |

## Gesture macros

| Gesture | Action |
|---------|--------|
| `Shift` while drawing | Constrain: square/circle, 45° lines and arrows |
| Right drag | Temporary eraser, from any tool |
| `Alt` + drag selection | Duplicate while moving |
