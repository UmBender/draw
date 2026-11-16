---
id: T172
title: Retained renderer in the app
status: todo
wave: 72
branch: task/T172-retained-renderer-in-the-app
depends_on: [T146, T173, T179, T180]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T172 Retained renderer in the app

## Goal

The app draws through the retained renderer of [[T146 Retained renderer in draw-gfx]]: shapes, text, formulas, grids and the minimap, with the camera as a uniform, and the frame budgets are met on the target.

## Known facts (2026-11-15)

- Created by the owner when T146 was scoped to `draw-gfx` (2026-11-15): T146 AC-2's app part (text, formulas, grids and the minimap through the same path; e2e goldens) and T146 AC-4 (budgets on the target) moved here.
- Removes the document layer re-render, the settle re-render and the blit of [[T26 Pan and zoom without re-rendering]].

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Shapes, text, formulas, grids and the minimap draw through the retained renderer; pan and zoom re-tessellate nothing. Tests: draw-call and vertex counts on the recorder.
- **AC-2** — The formula and e2e goldens stay equal within the pixel tolerance. `scripts/e2e.sh --backend all`.
- **AC-3** — The [[ADR-T24-5 Redraw budgets]] budgets are met on the target (overlay under 2 ms, full frame under 16 ms at any zoom), measured with `--frame-times` on the perf scenes and recorded in the Log. The measurement is the owner's, on the desktop GPU (headless sway renders with pixman); an unattended run stops at `review` with AC-1 and AC-2 done.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `src/shell/**` (the drawing path), `tests/perf.rs`, `tests/e2e*`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T172): specify the wiring`
- [ ] tests — `test(T172): add failing tests`
- [ ] behaviour — `refactor(T172): draw the app through the retained renderer`
- [ ] quality — `chore(T172): pass clippy and rustfmt`
- [ ] docs — `docs(T172): record the frame times`

## Log

- 2026-11-15 — created by the owner from T146 AC-2 (app part) and AC-4.
- 2026-11-16 — split by the owner after the batch blocker (recorded on the task branch): tessellator parity went to [[T179 Tessellator parity with the app's drawing]], retained text and formulas to [[T180 Text and formulas on the retained path]]; T172 keeps the wiring (`ChunkCache` in the app frame, minimap and grids, removal of the T26 layer re-render, settle and blit) and AC-3.
