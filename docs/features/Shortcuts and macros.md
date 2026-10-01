---
title: Shortcuts and macros
task: "[[T11 Keymap and macros]]"
adrs: ["[[ADR-T11-1 Exact modifier matching]]"]
tutorial: "[[11 Keymaps and gesture macros]]"
shortcuts: [P, L, A, R, C, E, B, V, H, "1-6", "[", "]", S, Ctrl+Z, Ctrl+Shift+Z, Ctrl+Y, Ctrl+C, Ctrl+X, Ctrl+V, Ctrl+D, Ctrl+A, Delete, Backspace, Ctrl+Backspace, Esc, "0", F, Tab]
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
| Pen / line / arrow | `P` / `L` / `A` |
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

## Limits

- Bindings are fixed at compile time; there is no user configuration.
- Keys are matched by position-independent `Key` values from the shell, so
  layouts where `[`/`]` need a modifier (`AltGr`) may not reach those
  bindings.
- A tool key pressed while `Shift` is held is ignored.
