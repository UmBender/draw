---
id: T08
title: Editor core and input model
status: review
wave: 4
branch: task/T08-editor
depends_on: [T03, T06]
adrs: ["[[ADR-0003 Headless core and thin shell]]", "[[ADR-T08-1 Tool context and gesture overlay]]"]
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

Tool-facing API (context, gesture state, overlay) is fixed by
[[ADR-T08-1 Tool context and gesture overlay]].

- **AC-1** — `core::input`: `Modifiers { shift, ctrl, alt }`, `PointerButton { Left, Middle, Right }`,
  `Key` (letters `A`–`Z`, digits `Digit0`–`Digit9`, `Delete`, `Backspace`, `Escape`, `Space`, `Tab`,
  `BracketLeft`, `BracketRight`), `InputEvent { PointerDown, PointerMove, PointerUp, Scroll, KeyDown, KeyUp, Resize }`
  with screen-space positions.
  *Tests:* `input::tests::modifiers_default_none`, `input::tests::modifiers_none_is_default`.
- **AC-2** — `core::command::Command`: `SetTool(Tool)`, `SetColor(ColorId)`, `WidthUp`, `WidthDown`,
  `CycleSmoothing`, `Undo`, `Redo`, `Copy`, `Cut`, `Paste`, `Duplicate`, `SelectAll`,
  `DeleteSelection`, `ClearAll`, `Cancel`, `ResetView`, `FitView`, `ToggleToolbar`.
  *Test:* compile (used by every `editor::tests` command test).
- **AC-3** — `Tool { Pen, Line, Arrow, Rect, Ellipse, Eraser, Bucket, Select, Hand }`, default `Pen`;
  `Tool::ALL` lists every tool once. *Tests:* `command::tests::tool_default_is_pen`,
  `command::tests::tool_all_lists_each_tool_once`.
- **AC-4** — `Editor::handle(InputEvent) -> bool` (true = needs redraw) and
  `Editor::apply(Command) -> bool`. Non-finite positions, scroll deltas and viewport
  sizes are dropped at the boundary (return `false`, no state change).
  *Tests:* `editor::tests::non_finite_pointer_ignored`, `non_finite_scroll_ignored`,
  `non_finite_resize_ignored`, `resize_sets_viewport`.
- **AC-5** — Navigation (`tools::navigate`): scroll zooms at cursor (`delta` = notches, positive
  zooms in); drag with `Hand`, middle button, or `Space` held pans from any tool; releasing
  `Space` mid-drag does not stop the drag. *Tests:* `editor::tests::scroll_zooms_at_cursor`,
  `middle_drag_pans_from_any_tool`, `space_drag_pans`, `hand_tool_drag_pans`,
  `navigate::tests::pan_moves_by_pointer_delta`.
- **AC-6** — Routing (`tools::mod`): left-button gestures go to the active tool;
  right-button drag goes to the eraser from any tool (gesture macro);
  a second button pressed during a gesture is ignored; `PointerUp` of another button
  does not end the gesture; `KeyDown` goes through the editor's keymap (default
  `keymap::resolve`, replaceable via `Editor::with_keymap`) then `apply`.
  *Tests:* `editor::tests::right_drag_routes_to_eraser`, `left_drag_routes_to_active_tool`,
  `other_button_does_not_end_gesture`, `keydown_uses_keymap`, `space_key_not_resolved`.
- **AC-7** — Commands implemented here: `SetTool` (cancels gesture), `SetColor`,
  `WidthUp/Down` (ladder `1 2 3 4 6 8 12 16 24 32` px, clamped at both ends, default 3),
  `CycleSmoothing`, `Undo`, `Redo` (both cancel the gesture and prune the selection),
  `ClearAll` (one undo step, no-op on an empty document), `Cancel` (gesture first,
  else clears selection), `ResetView`, `FitView` (fits document bounds into the viewport,
  resets on an empty document), `ToggleToolbar`. Commands delegated to wave-5 stubs:
  `Copy`, `Cut`, `Paste`, `Duplicate`, `SelectAll`, `DeleteSelection`.
  *Tests:* `editor::tests::set_tool_*`, `set_color_*`, `width_up_*`, `width_down_*`,
  `cycle_smoothing_*`, `undo_*`, `redo_*`, `clear_all_*`, `cancel_*`, `reset_view_*`,
  `fit_view_*`, `toggle_toolbar_*`.
- **AC-8** — Read API for shell and fuzzer: `document()`, `camera()`, `tool()`, `style()`
  (`DrawStyle { color, width_px }`), `smoothing()`, `selection() -> &[ShapeId]`,
  `selection_bounds()`, `preview() -> Option<Shape>`, `overlay() -> Overlay`,
  `toolbar_visible()`, `can_undo()`, `can_redo()`, `viewport()`.
  *Tests:* `editor::tests::new_editor_defaults`, `selection_bounds_*`.
- **AC-9** — Stub signatures (no-op bodies, return `false`/empty) with the exact API the
  wave-5 tasks implement: `tools::{pen, shape_tool, eraser, bucket, select}::{on_pointer, preview, cancel}`
  over a per-tool `State` and a `ToolCtx`, `clipboard::{copy, cut, paste, duplicate}`,
  `tools::select::{select_all, delete_selection}`,
  `keymap::resolve(key: Key, mods: Modifiers) -> Option<Command>`.
  *Test:* `editor::tests::stub_commands_do_not_panic`.

## Out of scope

Behaviour of pen/shape/eraser/bucket/select/clipboard/keymap (T09–T11).

## Files owned

`src/core/input.rs`, `src/core/command.rs`, `src/core/editor.rs`,
`src/core/tools/mod.rs`, `src/core/tools/navigate.rs`;
stub signatures only in `src/core/tools/{pen,shape_tool,eraser,bucket,select}.rs`,
`src/core/clipboard.rs`, `src/core/keymap.rs`.

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

Step 8 — requires steps 3 and 6.

## Log

- 2026-10-01 — Spec refined; tool-facing API recorded in
  [[ADR-T08-1 Tool context and gesture overlay]]. Deviation from the original
  AC-8/AC-9: `preview()` is kept but the full gesture output is
  `overlay() -> Overlay { shapes, hidden, marquee }` so the eraser (hidden
  shapes) and select (moved shapes, marquee) of T10 fit; tools take a per-tool
  `State`, a `ToolCtx` / `ToolView` and a `Pointer`.
- Additions beyond the spec: `Editor::with_keymap`, `active_gesture()`,
  `viewport()`, `Tool::ALL`, `DrawStyle { color, width_px }`, width ladder
  constants, `Clipboard` type (stub, owned by T10).
- Selection pruning after undo/redo/clear/edit commands/tool gestures lives in
  the editor, so T10's AC-7 holds without T10 editing `editor.rs`.
- Behaviour commit also fixed one test expectation: `Shape::bounds` includes
  half the outline width.
- Quality: rustfmt + removal of one unneeded `allow`; `scripts/check.sh` green
  (252 unit tests).
- Next for the integrator: Keymap note says `0` resets the view — T11 must map
  `Digit0` to `ResetView`, and `Digit1`–`Digit6` to `SetColor`.
