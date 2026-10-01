---
id: T08
title: Editor core and input model
status: ready
wave: 4
branch: task/T08-editor
depends_on: [T03, T06]
adrs: ["[[ADR-0003 Headless core and thin shell]]"]
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: "[[08 An editor as an input-driven state machine]]"
tags: [task]
---

# T08 Editor core and input model

## Goal

The `Editor`: the single object the shell talks to. Defines the input model and
the command set, routes events to tools, implements navigation and the
non-tool commands. Leaves **stub signatures** for wave-5 tasks so they can be
built in parallel without touching shared files.

## Spec

- **AC-1** — `core::input`: `Modifiers { shift, ctrl, alt }`, `PointerButton { Left, Middle, Right }`,
  `Key` (letters, digits, `Delete`, `Backspace`, `Escape`, `Space`, `Tab`,
  `BracketLeft`, `BracketRight`), `InputEvent { PointerDown, PointerMove, PointerUp, Scroll, KeyDown, KeyUp, Resize }`
  with screen-space positions. *Test:* compile + `input::tests::modifiers_default_none`.
- **AC-2** — `core::command::Command`: `SetTool(Tool)`, `SetColor(ColorId)`, `WidthUp`, `WidthDown`,
  `CycleSmoothing`, `Undo`, `Redo`, `Copy`, `Cut`, `Paste`, `Duplicate`, `SelectAll`,
  `DeleteSelection`, `ClearAll`, `Cancel`, `ResetView`, `FitView`, `ToggleToolbar`.
- **AC-3** — `Tool { Pen, Line, Arrow, Rect, Ellipse, Eraser, Bucket, Select, Hand }`.
- **AC-4** — `Editor::handle(InputEvent) -> bool` (true = needs redraw) and
  `Editor::apply(Command) -> bool`. Non-finite positions are dropped at the boundary.
  *Tests:* `editor::tests::non_finite_pointer_ignored`.
- **AC-5** — Navigation (`tools::navigate`): scroll zooms at cursor; drag with `Hand`,
  middle button, or `Space` held pans from any tool. *Tests:* `scroll_zooms_at_cursor`,
  `middle_drag_pans_from_any_tool`, `space_drag_pans`.
- **AC-6** — Routing (`tools::mod`): left-button gestures go to the active tool;
  right-button drag goes to the eraser from any tool (gesture macro);
  `KeyDown` goes through `keymap::resolve` then `apply`. *Tests:* `right_drag_routes_to_eraser`,
  `keydown_uses_keymap` (with a test keymap stub).
- **AC-7** — Commands implemented here: `SetTool` (cancels gesture), `SetColor`,
  `WidthUp/Down` (clamped 1–32 px), `CycleSmoothing`, `Undo`, `Redo`, `ClearAll`,
  `Cancel`, `ResetView`, `FitView`, `ToggleToolbar`. *Tests:* one per command.
- **AC-8** — Read API for shell and fuzzer: `document()`, `camera()`, `tool()`, `style()`,
  `smoothing()`, `selection() -> &[ShapeId]`, `selection_bounds()`, `preview() -> Option<Shape>`,
  `toolbar_visible()`, `can_undo()`, `can_redo()`.
- **AC-9** — Stub signatures (no-op bodies, return `false`/`None`) with the exact API the
  wave-5 tasks implement: `tools::{pen, shape_tool, eraser, bucket, select}::{on_pointer, preview, cancel}`,
  `clipboard::{copy, cut, paste, duplicate}`, `tools::select::{select_all, delete_selection}`,
  `keymap::resolve(key: Key, mods: Modifiers) -> Option<Command>`.

## Out of scope

Behaviour of pen/shape/eraser/bucket/select/clipboard/keymap (T09–T11).

## Files owned

`src/core/input.rs`, `src/core/command.rs`, `src/core/editor.rs`,
`src/core/tools/mod.rs`, `src/core/tools/navigate.rs`;
stub signatures only in `src/core/tools/{pen,shape_tool,eraser,bucket,select}.rs`,
`src/core/clipboard.rs`, `src/core/keymap.rs`.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 8 — requires steps 3 and 6.

## Log
