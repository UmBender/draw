---
title: Shortcuts and macros
task: "[[T11 Keymap and macros]]"
adrs: ["[[ADR-T11-1 Exact modifier matching]]", "[[ADR-T38-1 Left-hand keys for pen, line and smart snap]]"]
tutorial: "[[11 Keymaps and gesture macros]]"
shortcuts: [D, W, A, R, C, E, B, V, H, "1-6", "[", "]", S, Ctrl+Z, Ctrl+Shift+Z, Ctrl+Y, Ctrl+C, Ctrl+X, Ctrl+V, Ctrl+D, Ctrl+A, Delete, Backspace, Ctrl+Backspace, Esc, "0", F, Tab]
tags: [feature]
---

# Shortcuts and macros

## What it does

Every tool, colour, edit and view action has a single-key or `Ctrl` shortcut,
so a solution sketch never needs the mouse to leave the canvas. A few gesture
macros combine a modifier or button with a drag for the most common
two-step actions.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Pen / line / arrow | `D` / `W` / `A` |
| Rectangle / circle-ellipse | `R` / `C` |
| Eraser / bucket | `E` / `B` |
| Select / hand | `V` / `H` |
| Pick colour | `1`–`6` |
| Thinner / thicker stroke | `[` / `]` |
| Cycle anti-tremor | `S` |
| Undo / redo | `Ctrl+Z` / `Ctrl+Shift+Z` or `Ctrl+Y` |
| Copy / cut / paste at cursor | `Ctrl+C` / `Ctrl+X` / `Ctrl+V` |
| Duplicate selection | `Ctrl+D` |
| Select all | `Ctrl+A` |
| Delete selection | `Delete`, `Backspace` |
| Clear canvas (undoable) | `Ctrl+Backspace` |
| Cancel gesture / clear selection | `Esc` |
| Reset view / fit content | `0` / `F` |
| Show / hide toolbar | `Tab` |
| Command palette | `Ctrl+Shift+P` (type to filter, `↑`/`↓`, `Enter` runs, `Esc` closes) |
| Shortcut sheet | `?` (`←`/`→` pages, `1`–`9` topic, `?` or `Esc` closes) |
| Zoom towards cursor | Mouse wheel |
| Pan from any tool | Middle drag, `Space` + drag |
| Constrain (square, circle, 45°) | Hold `Shift` while drawing |
| Temporary eraser | Right drag, from any tool |
| Duplicate while moving | `Alt` + drag selection |

## How it works

`src/core/keymap.rs` holds one `const` table, `BINDINGS`, of
`(KeyChord, Command, description)` rows. `keymap::resolve(key, mods)` looks
the chord up in that table; the editor calls it on every `KeyDown` and passes
the resulting `Command` to `Editor::apply`. Modifiers must match exactly
([[ADR-T11-1 Exact modifier matching]]), so `Ctrl+C` copies and `C` picks the
ellipse tool, while `Shift+R` does nothing. The descriptions let the toolbar
and docs list the bindings from the same source.

`Space`, the mouse wheel and the gesture macros are not keymap entries: the
editor handles them directly in its input state machine (T08), and the tools
read `Shift`/`Alt` from pointer events (T09, T10).

### How the menus look

The tool strip, the command palette and the shortcut sheet are drawn as
translucent rounded islands with a thin border, after the owner's status
bar: four islands in the strip (tools, colours, helpers, undo and redo)
and a fifth for the page buttons when the window is tall enough; the
active tool, the selected palette row and the current sheet topic sit on
the blue selection fill with the bright accent text; the palette shows each
key of a chord as a key cap. The tools stay the letters of their keys. All
colours are theme tokens ([[Themes]]), and the rounded shapes are built
from non-overlapping triangles in `shell::ui_shapes`
([[ADR-T126-1 Menus drawn as translucent rounded islands]]).

### Command palette

`Ctrl+Shift+P` opens a prompt over the canvas listing every keymap command
with its key. Typing keeps the commands whose description contains the
letters in order (case-insensitive), best match first; `Enter` runs the
highlighted one exactly as its key would. While it is open no key or
pointer event reaches the tools. It is a modal editor state with no
history entry ([[ADR-T111-1 Command palette as a modal editor state]]); the
rows come from `BINDINGS`, so a new binding appears in it automatically.

### Shortcut sheet

`?` opens a paged cheat sheet over the canvas, one topic per page in the
order of [[Contest cheat sheet]]: Tools, Style, Edit, View, Helpers, Gestures,
Typing in a text block, Typing in a shape. `←`/`→` flip pages (wrapping),
`1`–`9` jump to a topic, `?` or `Esc` closes it. A topic with more than 14
rows is split ("Edit 1/2"), so a page fits the default 1280×800 window. In a
text block `?` types a `?`. Key rows come from `BINDINGS` through
`keymap::topic_of`, an exhaustive `match` on the command, so the sheet cannot
drift from the real keys; gesture and typing rows are one table in
`core/sheet.rs`. Like the palette it is a modal editor state with no history
entry ([[ADR-T113-1 Shortcut sheet as a second modal overlay]]).

## Limits

- Bindings are fixed at compile time; there is no user configuration.
- Keys are matched by position-independent `Key` values from the shell, so
  layouts where `[`/`]` need a modifier (`AltGr`) may not reach those
  bindings.
- A tool key pressed while `Shift` is held is ignored.
