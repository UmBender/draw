---
id: ADR-T123-1
title: Menus use the bundled font on whole pixels
status: accepted
kind: decision
date: 2026-11-12
task: T123
builds_on: ["[[ADR-T29-1 Embedded text font]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T123-1 Menus use the bundled font on whole pixels

## Context

The overlays drew with macroquad's built-in font at fractional positions: rough glyphs that shimmered, and no arrows (T122). The owner asked for better fonts.

## Decision

- Menu text uses the bundled Adwaita Mono subset, loaded once through macroquad's `load_ttf_font_from_bytes`, behind `shell::ui_text`. If loading fails the built-in font is used.
- Every string is drawn at a position rounded to a whole physical pixel (`round(v · dpi) / dpi`).

## Alternatives considered

- **A second, proportional UI font** — adds a binary asset for little gain; the canvas font is already bundled.
- **Only rounding positions** — keeps the rough built-in glyphs.

## Consequences

- Menus match the canvas text. No new dependency or asset.

## Rollback plan

Call `draw_text` again in the four call sites and drop `ui_text`.
