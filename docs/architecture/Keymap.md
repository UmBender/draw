---
tags: [architecture]
task: "[[T11 Keymap and macros]]"
status: implemented
---

# Keymap

Bindings implemented by [[T11 Keymap and macros]] (`src/core/keymap.rs`); modifiers
match exactly ([[ADR-T11-1 Exact modifier matching]]). Pen, line and smart
snap sit under the left hand ([[ADR-T38-1 Left-hand keys for pen, line and smart snap]]). Changes require an ADR.

## Tools (single key, no modifier)

| Key | Tool |
|-----|------|
| `D` | Pen (freehand, smoothed) |
| `W` | Line |
| `A` | Arrow |
| `R` | Rectangle |
| `C` | Circle / ellipse |
| `E` | Eraser (removes whole objects; pressed on a fill, clears fills instead — [[ADR-T21-2 Eraser mode chosen on press]]) |
| `B` | Bucket (fills the clicked rectangle/ellipse or grid cell; a drag fills every cell of that grid it crosses — [[ADR-T88-1 Bucket drag previews cells and commits on release]]) |
| `V` | Select / move |
| `H` | Hand (pan) |
| `G` | Grid (table of cells) |
| `T` | Text blocks ([[T29 Text blocks]]) |

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
| `Ctrl+T` | Replace the only selected text block by a grid of its lines and words ([[ADR-T84-1 Grid from text keys and table rules]]) |
| `Ctrl+Shift+V` | Build a grid from the system clipboard at the cursor; read by the shell, not in the keymap table ([[ADR-T84-1 Grid from text keys and table rules]]) |
| `Ctrl+L` | Replace the only selected text block by the graph or tree of its edges ([[ADR-T87-1 Graph from an edge list key and layout rules]]) |
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
| `X` | Smart snap on / off |
| `Shift+G` | Grid snap on / off |
| `N` | Auto-numbering on / off (circles and squares get numbers, new grids get axis indices — [[ADR-T22-1 Numbering toggle drives grid indices]]) |
| `Shift+N` | Restart numbering at the first number |
| `Ctrl+N` | First number 1 / 0, restarts the counter ([[ADR-T86-1 Zero-based numbering switch]]) |
| `→` / `←` | Grid: one more / one less column (1–64, live while dragging); with the Select tool and one grid selected, of that grid ([[ADR-T85-1 Arrows resize the one selected grid]]) |
| `↓` / `↑` | Grid: one more / one less row (1–64, live while dragging); with the Select tool and one grid selected, of that grid |

## Text edit mode

Added by [[T29 Text blocks]] ([[ADR-T29-2 Text edit mode]]). While a text
block is open **no binding above fires**: every key types its character
(from the keyboard layout), and only these keys act.

| Key | Action |
|-----|--------|
| `Esc`, `Ctrl+Enter` | Commit the block (one undo step) and leave |
| `Ctrl+Z` | Cancel the edit (new block vanishes, old text stays) |
| `Enter` | New line |
| `Tab` | Four spaces |
| `←` / `→` | Previous / next character |
| `↑` / `↓` | Previous / next line, same column |
| `Home` / `End` | Start / end of the line |
| `Backspace` / `Delete` | Delete the character before / after the caret |
| `Ctrl+]` / `Ctrl+[` | Bigger / smaller text, one step of the 14–76 px ladder ([[ADR-T29-3 Resize text while typing]]) |
| `Ctrl+V` | Paste text from the system clipboard at the caret ([[ADR-T83-1 Paste text through an input event]]) |

A press outside the block (that does not pan) commits it too; toolbar
buttons commit it before acting, except Undo, which cancels it.

## Gesture macros

| Gesture | Action |
|---------|--------|
| `Shift` while drawing | Constrain: square/circle, 45° lines and arrows |
| Right drag | Temporary eraser, from any tool |
| `Alt` + drag selection | Duplicate while moving |
