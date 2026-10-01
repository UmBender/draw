---
title: An editor as an input-driven state machine
step: 8
requires: ["[[03 Cameras - world space vs screen space]]", "[[06 Undo and redo with a transaction log]]"]
feature: "[[Infinite canvas, pan and zoom]]"
code: ["src/core/input.rs", "src/core/command.rs", "src/core/editor.rs", "src/core/tools/mod.rs", "src/core/tools/navigate.rs"]
tags: [tutorial]
---

# An editor as an input-driven state machine

> **Learning path step 8.** Requires: [[03 Cameras - world space vs screen space]],
> [[06 Undo and redo with a transaction log]] · Next: *9 Building drawing tools*

## Why this matters here

The window system throws a stream of low-level events at us: button down,
move, move, move, button up, key down. The same event means different things
depending on what came before — a move draws, pans or does nothing. If every
tool decided that on its own, rules like "middle drag pans from any tool" or
"`Esc` cancels whatever is happening" would be copied nine times and drift.

## The concept

**One object, one entry point, explicit state.** All input goes through one
function, `handle(event) -> bool`. The editor keeps a small amount of state
that decides what an event means:

```
            PointerDown(middle) / Down(left)+Space / Down(left)+Hand
   Idle ─────────────────────────────────────────────────────▶ Pan
    │  ▲                                                        │
    │  └──────────────── PointerUp(same button) ────────────────┘
    │  ▲
    │  └──────────────── PointerUp(same button) / Cancel ───────┐
    │                                                           │
    └── PointerDown(left) → Tool(active)  /  Down(right) → Tool(Eraser)
```

- **Idle** — no button held. Moves only update the cursor.
- **Pan** — moves pan the camera.
- **Tool(t)** — moves go to tool `t`.

Two more ideas keep the machine small:

1. **Commands are data.** Discrete actions (`Undo`, `SetTool(Rect)`,
   `ResetView`) are values of the `Command` enum. Keys become commands through
   a *keymap* function; toolbar buttons produce the same values. So
   `apply(Command)` is the only place an action is implemented, whoever
   triggered it.
2. **Validate at the boundary.** Events from outside may carry `NaN` or `∞`.
   The editor drops them on entry, so nothing inside ever has to check again.

The `bool` return value is the "dirty flag" of
[[ADR-0006 Redraw on demand]]: the shell redraws only when something visible
changed.

## How draw implements it

- `src/core/input.rs` defines `InputEvent`, `Key`, `PointerButton` and
  `Modifiers`. Positions are screen pixels.
- `src/core/command.rs` defines `Tool` and `Command`.
- `Editor::handle` (`src/core/editor.rs`) matches on the event.
  `track_cursor` sanitizes the position (`Vec2::sanitize`) and returns `None`
  for non-finite input, which ends the match arm with `false`.
- The private `Gesture` enum is the state: `Pan { button, pan }` or
  `Tool { button, tool }`. Storing the **button** makes "only the button that
  started the gesture ends it" a single `if b == button` guard in
  `pointer_up`. `pointer_down` ignores a second button while a gesture runs.
- `Space` is special: `KeyDown(Space)` sets `space_held` instead of going
  through the keymap, so holding it turns the next left drag into a pan.
- `Command::Cancel` first calls `cancel_gesture`; only when idle does it
  clear the selection — one key, the most local meaning first.
- Tools are reached through `ToolStates` in `src/core/tools/mod.rs`, which
  matches on the tool and calls `pen::on_pointer`, `eraser::on_pointer`, …
  Each tool gets a `ToolCtx` — borrowed slices of editor state — and keeps its
  own gesture data in its module's `State` type
  ([[ADR-T08-1 Tool context and gesture overlay]]). That split lets the
  creation and editing tools be written in parallel without touching
  `editor.rs`.
- The keymap is a plain function pointer (`fn(Key, Modifiers) ->
  Option<Command>`). Tests swap it with `Editor::with_keymap(test_keymap)` to
  check routing before the real keymap exists.

## Try it

1. Add `InputEvent::Scroll { pos, delta: f32::NAN }` to
   `editor::tests::non_finite_scroll_ignored` in a loop over other bad
   positions (`±∞`, `f32::MAX * 2.0`). Do they all return `false`?
2. Change `pointer_down` so a second button *replaces* the running gesture.
   Which test fails, and what would a user notice (hint: press right while
   middle-panning)?
3. Write a test `width_up_then_down_returns_to_default` using
   `Command::WidthUp` and `Command::WidthDown`.

## Further reading

- Robert Nystrom, *Game Programming Patterns* — chapters "Command" and
  "State".
- [[ADR-0003 Headless core and thin shell]] — why the editor never sees
  macroquad types.
