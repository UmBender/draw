---
id: T184
title: Select click shows the selection
status: in-progress
wave: 61
branch: fix/select-click-redraw
depends_on: []
adrs: []
feature: "[[Editing tools]]"
tutorial: ""
tags: [task, bug]
---

# T184 Select click shows the selection

## Goal

A plain click with the Select tool (`V`) on a shape, a text block in particular, shows the selection and its resize handles at once, so a block can be picked with one click and turned into a graph or a grid.

## Known facts (2026-11-21)

- Reported by the owner on 2.0.0: a click on a text block with the Select tool does not select it; only a small marquee drag over it does. It gets in the way of turning a text block into a graph.
- The click does select, at the press (`select::press`), but the window is not redrawn when the button is released:
  - the resize handles are hidden while a gesture is in progress (`Editor::selection_handles`);
  - a click's release leaves the selection as the press set it, so `select::release` reports no change, and `Editor::pointer_up` returns `false`;
  - the shell redraws only when the editor reports a change (`plan_redraw`), and press and release normally arrive in different frames, so the frame with the handles is never drawn.
- A marquee's release always reports a change, which is why the small drag works.
- The editor tests' `click` helper ORs the press and the release results, which hid the bug.

## Spec

- **AC-1** — The release of a Select click that leaves the selection's handles shown reports a redraw, on a text block and on any other shape.
  Tests: `select_click_release_redraws_to_show_handles`, `select_click_on_rect_release_redraws_to_show_handles` (`core::editor`).
- **AC-2** — Regression: from a snapshot with one unselected text block and the Select tool, a press then a release on the block selects it, shows the handles and the release reports a redraw.
  Test: `regression_T184_select_click_shows_handles` (`tests/snapshots.rs`), fixture `tests/snapshots/regressions/T184-select-click-shows-handles.snap`.
- **AC-3** — In the shell loop, with idle frames skipped as on the desktop and the press and the release in different frames, the release frame submits GPU work, so the handles reach the window.
  Test: `regression_T184_select_click_release_draws_the_handles` (`shell::app`, over `RecordingPlatform` and `RecordingGpu`, started from the AC-2 fixture).

## Out of scope

- What a click on an already selected text block does (it stays selected, ADR-T31-1 as changed on 2026-11-17).
- Redraw rules of other tools.

## Files owned

- `crates/draw-core/src/core/editor.rs`
- `tests/snapshots.rs`
- `tests/snapshots/regressions/T184-select-click-shows-handles.snap`
- `src/shell/app.rs` (tests only)
- `docs/features/Editing tools.md`, `docs/features/Text blocks.md` (the Select click rows)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T184): specify select click redraw acceptance criteria`
- [x] regression — `test(T184): add regression test for select click without handles`
- [x] behaviour — `fix(T184): redraw when a gesture ends with the handles shown`
- [ ] quality — `chore(T184): pass clippy and rustfmt` (skipped if nothing changes)
- [ ] docs — `docs(T184): update the select click notes`

## Log

- 2026-11-21 — written from the owner's bug report; cause found by reading the press/release path and confirmed with a failing unit test.
- 2026-11-21 — no e2e case: a screenshot case passed without the fix too. With the e2e hooks on, the app draws every idle frame (`skip_idle` is false), which hides the bug; only the desktop, which skips idle frames, shows it. The shell-loop test of AC-3 runs with idle frames skipped and fails without the fix.
