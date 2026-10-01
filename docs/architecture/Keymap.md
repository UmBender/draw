---
tags: [architecture]
task: "[[T11 Keymap and macros]]"
status: planned
---

# Keymap

Planned bindings. [[T11 Keymap and macros]] implements them; changes after that
require an ADR.

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

## Gesture macros

| Gesture | Action |
|---------|--------|
| `Shift` while drawing | Constrain: square/circle, 45° lines and arrows |
| Right drag | Temporary eraser, from any tool |
| `Alt` + drag selection | Duplicate while moving |
