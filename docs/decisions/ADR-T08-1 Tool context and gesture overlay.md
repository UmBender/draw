---
id: ADR-T08-1
title: Tool context and gesture overlay
status: accepted
kind: decision
date: 2026-10-01
task: T08
builds_on: [ADR-0003, ADR-0005]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T08-1 Tool context and gesture overlay

## Context

Wave 5 ([[T09 Creation tools]], [[T10 Editing tools]], [[T11 Keymap and macros]])
runs in parallel and may only edit its own tool files, never `editor.rs`. So the
editor must fix, up front, how a tool sees editor state, where a tool keeps its
gesture state, and how it shows what it is doing. The task note planned
`preview() -> Option<Shape>`, which cannot express an eraser hiding shapes, a
selection being moved (many shapes) or a marquee rectangle.

## Decision

- **Gesture state lives in the tool module.** Each tool module exports
  `#[derive(Default)] pub struct State` (owned and filled by its task); the
  editor holds one of each and never looks inside.
- **Tools receive a `ToolCtx<'_>`** (in `tools::mod`): mutable borrows of
  document, history, selection and clipboard; read-only camera; the active
  `Tool`, `DrawStyle`, `SmoothingLevel` and last cursor position (screen). Its
  helper `commit(tx)` records one undo step and returns whether anything changed.
- **Pointer input reaches tools as `Pointer { phase, pos, mods }`**, `pos` in
  screen pixels and always finite (filtered by the editor).
- **Uniform tool API:** `on_pointer(&mut State, &mut ToolCtx, Pointer) -> bool`,
  `preview(&State, &ToolCtx) -> Overlay`, `cancel(&mut State) -> bool`.
  Clipboard and select commands take `&mut ToolCtx` (plus `&mut State` where
  needed) and return `bool`.
- **`Overlay { shapes, hidden, marquee }`** is what a gesture draws on top of the
  document: extra shapes, document ids to skip, and a world-space marquee.
  `Editor::overlay()` exposes it; `Editor::preview()` remains as the first overlay
  shape for the simple case.
- **The editor prunes the selection** after undo, redo and clear, so tools never
  observe stale ids.
- **The keymap is a function pointer** (`keymap::resolve` by default),
  replaceable with `Editor::with_keymap` so routing is testable before T11.

## Alternatives considered

- **A `Tool` trait object per tool** — dynamic dispatch and boxing for nine
  fixed tools; a `match` in `tools::mod` is simpler and exhaustive.
- **Tools return edit requests the editor applies** — the editor would have to
  know every tool's needs (marquee, move, duplicate) up front.
- **Editor-owned gesture state** — every wave-5 task would need to edit
  `editor.rs`, breaking parallel execution.

## Consequences

- Wave-5 tasks need no file outside their own; T12's renderer draws
  `document` minus `overlay.hidden`, then `overlay.shapes` and `overlay.marquee`.
- `preview()` allocates an `Overlay` per call; it is called once per redrawn
  frame only (ADR-0006), so this is negligible.

## Rollback plan

Narrow `Overlay` back to `Option<Shape>` in `tools::mod` and the tool modules;
tasks using `hidden`/`marquee` then need a follow-up ADR.
