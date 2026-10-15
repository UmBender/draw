---
title: Text blocks
task: "[[T29 Text blocks]]"
adrs: ["[[ADR-T29-1 Text shape with monospaced layout and an embedded font]]", "[[ADR-T29-2 Text edit mode]]", "[[ADR-T29-3 Resize text while typing]]"]
tutorial: "[[29 Typing on a canvas without losing the shortcuts]]"
shortcuts: ["T", "Esc", "Ctrl+Enter", "Enter", "Tab", "Ctrl+Z", "Ctrl+[", "Ctrl+]"]
tags: [feature]
---

# Text blocks

## What it does

Write notes straight on the canvas: a complexity, a recurrence, `n ≤ 2·10⁵`,
`dp[i][j]`, Greek letters and math symbols. Click with the Text tool and
type; the block zooms with the canvas, stays sharp at every zoom, and moves,
copies, erases, snaps and undoes like any other shape. While you type no
shortcut fires — `P` is a `p`, `1` is a `1` — and one `Esc` gives the keys
back.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Text tool | `T` or the toolbar |
| New block | Text tool, click where the top-left corner goes |
| Finish the block | `Esc`, `Ctrl+Enter`, or click elsewhere (with the Text tool that click starts the next block) |
| New line | `Enter` |
| Indent | `Tab` (four spaces) |
| Move the caret | `←` `→` `↑` `↓`, `Home`, `End`, or click inside the block |
| Delete | `Backspace`, `Delete` |
| Throw the edit away | `Ctrl+Z` while typing |
| Edit a block again | Text tool: click it · Select tool: double-click it (or click it when it is already selected) |
| Text size | `[` / `]` before clicking (the stroke width ladder: 14–76 px) |
| Bigger / smaller while typing | `Ctrl+]` / `Ctrl+[` (one step of the same ladder) |
| Text colour | `1`–`6` before clicking |

Finishing an edit is one undo step: `Ctrl+Z` afterwards removes the new
block or restores the old text. A block left empty (or only spaces)
disappears. With grid or smart snap on, the clicked corner snaps like the
start of a shape, and other shapes snap to a block's edges and centre.

## How it works

`Shape::Text { origin, text, size, style }` lives in the document
(`src/core/shape.rs`). Its layout is a monospaced grid computed in
`src/core/text_layout.rs` — every character is 0.6 × 1.25 em — so bounds,
hit-testing, caret placement and snapping are pure, tested and fuzzed
([[ADR-T29-1 Text shape with monospaced layout and an embedded font]]).
The block being typed is a `TextEdit` in `src/core/tools/text.rs`, outside
the document until it is committed; the editor sends it typed characters
(`InputEvent::Char`) and editing keys and keeps every key away from the
keymap ([[ADR-T29-2 Text edit mode]]). The shell draws each glyph at its
cell from an embedded 124 KB subset of Adwaita Mono, rasterized near its
on-screen size (`src/shell/render.rs`); the number labels use the same font.
While typing, only the overlay is redrawn and the caret does not blink, so
the app stays idle between keys.

## Limits

- One monospaced font, one style; no word wrap, alignment, rich text or
  font choice.
- No pasting from the system clipboard and no IME composition (dead keys
  work through the keyboard layout).
- Characters outside the font's Latin, Greek and math ranges show a
  replacement box; wide (CJK) characters overlap their neighbour.
- Text inside rectangles and grid cells is
  [[T31 Text in rectangles and grid cells]]; formulas are
  [[T30 Basic TeX formulas]].
