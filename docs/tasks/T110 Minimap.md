---
id: T110
title: Minimap
status: todo
wave: 59
branch: task/T110-minimap
depends_on: []
adrs: []
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: ""
tags: [task]
---

# T110 Minimap

## Goal

While panning — `Space` held, or the middle button dragging — a small overview of the whole drawing shows in a corner, with a frame for the current view that follows the pan, so you can see where you are moving to. Clicking in it while `Space` is held jumps the view there.

## Known facts (2026-11-10)

- `Space` + drag and middle-button drag pan the view; `F` fits all content in the window ([[Infinite canvas, pan and zoom]]).
- The proposal had an `M` toggle; the owner chose to show the minimap only while panning, with no toggle.
- The document layer is cached and re-rendered only on change (ADR-T24-5); idle CPU is 0 % ([[ADR-0006 Redraw on demand]]).
- The proposal's risk: permanent UI and work on every edit, so the overview must cost nothing while hidden.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — The minimap shows in the bottom-right corner while `Space` is held and while a middle-button pan is in progress, and hides as soon as both end. It shows every shape scaled to fit, and a frame for the visible area that moves with the pan. In text edit mode `Space` types a space and shows nothing.
- **AC-2** (owner) — While `Space` is held, a click in the minimap centres the view on that point and dragging there moves the view continuously; those clicks never reach the canvas tools. Outside the minimap, `Space` + drag pans as before.
- **AC-3** — While hidden it costs nothing. While shown, it is drawn from its own cache, re-rendered only when the document changed since the last time it was shown; the view frame is an overlay, so panning does not re-render it.
- **AC-4** — An empty document shows an empty minimap; huge or far-apart content keeps every coordinate finite (fuzzed).

## Out of scope

- A key that keeps the minimap on; choosing the corner or size; showing it in PNG export.

## Files owned

- a new `src/core/minimap.rs` (geometry), `src/core/editor.rs`, `src/core/input.rs`
- `src/shell/` (overview cache, input routing, drawing)
- `docs/features/Infinite canvas, pan and zoom.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T110-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T110): specify the minimap`
- [ ] tests — `test(T110): add failing tests for the minimap`
- [ ] models — `feat(T110): add the minimap geometry`
- [ ] behaviour — `feat(T110): show the minimap while panning`
- [ ] quality — `chore(T110): pass clippy and rustfmt`
- [ ] docs — `docs(T110): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from V2 of the [[Feature proposal]].
- 2026-11-10 — owner: show it while `Space` is held (panning) instead of an `M` toggle.
