---
title: Toolbar
task: "[[T12 App shell and toolbar]]"
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-T12-1 Blocking event loop with cached frame]]"]
tutorial: "[[12 The app loop, idle redraw and UI]]"
shortcuts: [Tab]
tags: [feature]
---

# Toolbar

## What it does

A thin strip at the left edge of the window with one button per tool, the six
palette colours, and undo/redo. The active tool and colour are outlined in the
theme's `accent` colour; undo and redo fade when there is nothing to undo or
redo. It is optional: every button has a shortcut, and `Tab` hides the strip.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Pick a tool | Click its letter (`P L A R C E B V H` — the letter is the shortcut) |
| Pick a colour | Click a swatch (keys `1`–`6`) |
| Undo / redo | Click the left / right arrow (`Ctrl+Z` / `Ctrl+Shift+Z`) |
| Show / hide the toolbar | `Tab` |

## How it works

`src/shell/toolbar.rs` is pure layout plus a thin draw function:
`layout(viewport)` returns 17 buttons of 32 × 32 px, `hit(buttons, pos)` maps
a click to a `Command`, and `route(visible, viewport, event)` decides whether
an input event goes to the canvas or to the toolbar. Any press on the visible
strip is consumed there, so a click on the toolbar never draws on the canvas.
Moves, releases and scrolls always reach the editor, so a stroke dragged over
the strip still ends normally.

The app loop (`src/shell/app.rs`) sleeps in miniquad's blocking event loop,
collects every raw event in order (`src/shell/input_map.rs`), and re-renders
the scene (toolbar included) into a cached texture only when something
changed ([[ADR-T12-1 Blocking event loop with cached frame]]).

## Limits

- No tooltips, no width or anti-tremor indicator: those live on the keyboard.
- The strip does not scroll; in a window shorter than ~620 px the last buttons
  are cut off (their shortcuts still work).
- Runs through XWayland on Wayland sessions (miniquad's X11 backend).
