---
id: T106
title: Fading laser pointer
status: todo
wave: 59
branch: task/T106-laser-pointer
depends_on: []
adrs: []
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: ""
tags: [task]
---

# T106 Fading laser pointer

## Goal

Holding `Q` with the left hand turns the pen or mouse in the right hand into a laser pointer: a bright trail follows it and fades away, and nothing is drawn, selected or added to the history.

## Known facts (2026-11-10)

- `Q` is unbound as a bare key (only listed among the keys `keymap.rs` knows). The proposal had `L`; the owner chose `Q` so the left hand holds it while the right hand draws.
- draw redraws only on input ([[ADR-0006 Redraw on demand]]): a fading trail needs frames after the last event, until it has faded, then idle CPU must go back to 0 %.
- Overlays (selection outline, previews) are drawn over the cached document layer without re-rendering it (ADR-T24-5): the trail belongs there.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — While `Q` is held, pointer moves (pen or mouse, button down or not) add points to a trail instead of reaching the current tool. Releasing `Q` stops adding points; the tool, selection and any open text edit are as before. In text edit mode `Q` types a `q`.
- **AC-2** — Each trail point fades out over a fixed time (proposed about 1 s, chosen in the ADR); the trail is drawn as a thick, bright stroke in screen space, the same width at every zoom.
- **AC-3** — The trail never changes the document, the selection or the history: no undo step, no snapshot change, no autosave trigger.
- **AC-4** — Frames are requested only while a trail point is still visible; once it has faded the app goes back to waiting for events (idle 0 %). Only the overlay is redrawn, never the document layer.
- **AC-5** — Holding `Q` during a gesture (a stroke or drag in progress) ends nothing: the gesture keeps going and the trail starts with the next move after it. Fuzzed with `Q` presses and releases mixed into input sessions.

## Out of scope

- Laser colour or width settings; a laser that stays on without holding a key.
- Recording or replaying the trail; showing it in PNG export.

## Files owned

- `src/core/keymap.rs`, `src/core/editor.rs`, `src/core/input.rs`, a new `src/core/laser.rs`
- `src/shell/` (key release, frame requests, overlay drawing)
- `docs/features/Infinite canvas, pan and zoom.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T106-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T106): specify the fading laser pointer`
- [ ] tests — `test(T106): add failing tests for the laser pointer`
- [ ] models — `feat(T106): add the laser trail model`
- [ ] behaviour — `feat(T106): draw the fading trail while Q is held`
- [ ] quality — `chore(T106): pass clippy and rustfmt`
- [ ] docs — `docs(T106): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from V3 of the [[Feature proposal]]; key changed from `L` to `Q` for the left hand.
