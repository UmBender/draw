---
id: T116
title: Pages
status: in-progress
wave: 59
branch: task/T116-pages
depends_on: []
adrs: [ADR-T116-1]
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: ""
tags: [task]
---

# T116 Pages

## Goal

`Alt+1` to `Alt+9` switch between nine separate canvases, for example one per contest problem; each page keeps its own shapes and view, and the current page number shows in the corner.

## Known facts (2026-11-10)

- `Alt+1`…`Alt+9` are unbound (`Alt` is used with arrows, `H`, `V`, `[`, `]`).
- One document, one history, one camera today; the document layer is cached and re-rendered on change (ADR-T24-5); snapshots and the autosave store one document ([[Editor snapshots]], [[Crash recovery]]).
- T108 (save and load) depends on this task, so its first file version stores every page.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `Alt+1`…`Alt+9` show page 1…9; draw starts on page 1. Each page has its own shapes, selection and view (pan and zoom). The page number shows top-right when it is not page 1 (ADR-T116-1). Tests: `keymap::alt_digits_go_to_pages`, `editor::pages_keep_their_own_shapes_selection_and_view`, `editor::new_editor_is_on_page_one`, `editor::going_to_the_current_page_changes_nothing`, `render::page_label_*`.
- **AC-2** (owner) — Undo and redo act on the current page only: each page keeps its own history, and switching pages is never an undo step. Tests: `editor::undo_acts_on_the_current_page_only`, `editor::switching_pages_is_not_an_undo_step`.
- **AC-3** — Switching pages cancels a gesture in progress and closes an open text edit as `Esc` would; it costs one document re-render, then idle as usual. Tests: `editor::switching_pages_cancels_the_gesture`, `editor::switching_pages_commits_the_open_text`, `editor::switching_pages_bumps_the_document_revision`.
- **AC-4** — Numbering, snapping, export (`Ctrl+E`, and T114 when present), the minimap (T110) and clearing the canvas act on the current page; copy and paste work across pages (copy on one, paste on another). Tests: `editor::clear_all_clears_only_the_current_page`, `editor::copy_on_one_page_pastes_on_another`, `editor::numbering_counts_per_page`.
- **AC-5** — Snapshots and the autosave store all pages, their views and histories (format version bumped); recovery restores the page that was open. Fuzzed with page switches mixed into editing sessions. Version 2 still loads as one page. Tests: `snapshot::round_trip_keeps_every_page`, `snapshot::version_two_still_loads`, `snapshot::version_is_three`, `snapshot::rejects_bad_page_index`, fuzz `snapshot_round_trip_with_page_switches`.

## Out of scope

- More than nine pages, naming pages, moving shapes between pages other than cut and paste, an overview of all pages.

## Files owned

- `src/core/editor.rs`, `src/core/document.rs` (or a new `src/core/pages.rs`), `src/core/history.rs`, `src/core/keymap.rs`, `src/core/command.rs`, `src/core/snapshot.rs`
- `src/shell/` (page indicator, cache reset on switch)
- `docs/features/Infinite canvas, pan and zoom.md`, `docs/features/Undo and redo.md`, `docs/features/Editor snapshots.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, snapshot fixtures
- this note, and new `ADR-T116-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T116): specify pages`
- [ ] tests — `test(T116): add failing tests for pages`
- [ ] models — `feat(T116): keep nine pages with their own views and histories`
- [ ] behaviour — `feat(T116): switch pages with Alt+1 to Alt+9`
- [ ] quality — `chore(T116): pass clippy and rustfmt`
- [ ] docs — `docs(T116): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from V1 of the [[Feature proposal]].
- T108 depends on it so saved files store every page. The proposal's per-page undo rule is marked as the owner's default; revisit at batch planning if needed.
