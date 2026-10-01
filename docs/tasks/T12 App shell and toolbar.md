---
id: T12
title: App shell and toolbar
status: ready
wave: 6
branch: task/T12-app-shell
depends_on: [T07, T09, T10, T11]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-0012 Fixed palette and theme tokens]]"]
feature: "[[Toolbar]]"
tutorial: "[[12 The app loop, idle redraw and UI]]"
tags: [task]
---

# T12 App shell and toolbar

## Goal

The runnable app: window, input translation, idle-friendly loop, and a minimal
toolbar styled with the theme.

## Spec

Pure, unit-tested:

- **AC-1** — `input_map`: macroquad `KeyCode` → `core::input::Key` and mouse buttons →
  `PointerButton` (pure `fn map_key(KeyCode) -> Option<Key>`). *Tests:* `input_map::tests::*`.
- **AC-2** — `toolbar::layout(viewport) -> Vec<Button>`: one button per tool + 6 colour
  swatches + undo/redo, vertical strip at the left edge, 32 px buttons. *Tests:* `toolbar::tests::layout_*`.
- **AC-3** — `toolbar::hit(layout, pos) -> Option<Command>`; clicks on the toolbar never
  reach the canvas. *Tests:* `hit_*`, `toolbar_click_not_forwarded`.

Shell (manual checklist in the task log):

- **AC-4** — Loop: collect events → `Editor::handle` → draw only when dirty
  ([[ADR-0006 Redraw on demand]]). Record which strategy works on the target
  (blocking event loop vs cached render target) in `ADR-T12-1`.
- **AC-5** — Toolbar shows the active tool and colour with the `accent` token; hidden with `Tab`.
- **AC-6** — Runs on Wayland (native or XWayland — record which) without input loss;
  idle CPU ≈ 0 % (measure with `top`, record the number).

## Out of scope

Menus, settings UI, saving.

## Files owned

`src/shell/app.rs`, `src/shell/input_map.rs`, `src/shell/toolbar.rs`,
new ADR `docs/decisions/ADR-T12-1 *.md`.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 12 — requires steps 7–11.

## Log
