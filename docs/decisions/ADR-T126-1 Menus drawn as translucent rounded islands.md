---
id: ADR-T126-1
title: Menus drawn as translucent rounded islands
status: accepted
kind: decision
date: 2026-11-12
task: T126
builds_on: ["[[ADR-T51-1 Build-time theme]]", "[[ADR-T123-1 Menus use the bundled font on whole pixels]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T126-1 Menus drawn as translucent rounded islands

## Context

The owner finds the menus ugly but likes their content and navigation, and asked for a look closer to their waybar, keeping the palette and the letter icons. They chose direction A of three mockups.

## Decision

- **Islands.** The tool strip is four islands (tools, colours, helpers, history) plus a fifth for the page buttons when the window is tall enough. An island is a rounded rectangle of radius 8 px, filled with the island colour at 75 % opacity and edged by a 1 px ring in the border colour at 80 %. The palette and the sheet are single islands at 92 % opacity.
- **Geometry.** Outer margin 3 px, border 1 px, padding 3 px, 3 px between islands: the buttons keep their 32 px, sit 7 px from the window edge and the strip is 46 px wide. The four main islands end at 784 px, so they fit the default 800 px window as the flat strip did.
- **Looks.** An active tool, helper or topic: rounded 6 px fill in the active fill colour, text in the bright accent. Colours: 20 px rounded swatches, the current one with a 2 px accent ring. Rows of the palette and the sheet: 6 px rounded; the selected row in the active fill. Each key of a chord is a small key cap (key cap fill, 1 px border).
- **Drawing.** macroquad has no rounded rectangles. They are built from triangles that do not overlap (a centre and two side rectangles, four corner fans, and for a ring bars and corner strips), so the translucent fill has no seams. The module is `shell::ui_shapes`.
- **Theme.** New tokens in `theme.rs`: `MENU_ISLAND` (+ alpha), `MENU_PANEL_ALPHA`, `MENU_BORDER` (+ alpha), `MENU_KEY_FILL`, `MENU_MUTED`, `MENU_DIM_TEXT`, `MENU_ACTIVE_BRIGHT`, with the Kanagawa Dragon values of the waybar. A rebuild with other values restyles every menu, as for the existing tokens.

## Alternatives considered

- **B, a flat rail** and **C, dots** — the other two proposals, not chosen.
- **A full-height strip with rounded groups inside** — a wider band that hides more canvas.
- **Overlapping circles for corners** — they double-blend a translucent fill.

## Consequences

- The strip is 6 px wider than before (46 px).
- About 100 to 300 more triangles per frame for the menus; they are drawn only when visible.

## Rollback plan

Restore the previous `draw` functions of the three menus and the 40 px strip; this ADR is then reverted by a new ADR.
