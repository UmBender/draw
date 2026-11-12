---
id: T135
title: Automaton modal states, pages, camera and selection
status: todo
wave: 67
branch: task/T135-automaton-modal-states-pages-camera-and-
depends_on: [T133, T134]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T135 Automaton modal states, pages, camera and selection

## Goal

The modal overlays and the view regions move into the machine: a modal is a state that captures input, not a flag.

## Known facts (2026-11-12)

- The bugs this removes: the laser with an open palette, undo with an open sheet, `?` closing the sheet during fuzz.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `Interaction::{Idle, Palette, Sheet}`, `Page`, `Camera` and `Selection` regions are implemented in the machine; the matching fields leave `Editor`.
  Tests: the existing editor tests for palette, sheet, pages and camera pass through the machine unchanged.
- **AC-2** — No input reaches a tool while a modal state is current, by construction, and every modal state leaves with `Escape`.
  Tests: `every_state_leaves_with_escape` (exhaustive over the modal states), `modal_captures_input`.
- **AC-3** — The snapshot fixtures load and the snapshot of the machine equals the old one.
  Fixtures unchanged.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/machine/**`, `crates/draw-core/src/editor/**` (removal of the migrated fields)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T135): specify the modal and view regions`
- [ ] tests — `test(T135): add failing tests for the regions`
- [ ] behaviour — `refactor(T135): move the modal and view regions into the machine`
- [ ] quality — `chore(T135): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
