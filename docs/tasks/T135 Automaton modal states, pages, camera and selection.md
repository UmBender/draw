---
id: T135
title: Automaton modal states, pages, camera and selection
status: review
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

Design: [[ADR-T135-1 Regions live in the machine module]]. The region types
`Modal`, `Pages` and `View` are defined in `machine/regions.rs`; `Editor`
holds them in place of the loose fields `palette`, `sheet`, `page`, `pages`,
`camera` and `selection`.

- **AC-1** — `Modal` (`Closed | Palette | Sheet`), `Pages` and `View` (camera and selection) are implemented in the machine; the six loose fields leave `Editor`.
  Tests: `modal_opens_one_overlay_at_a_time`, `pages_start_on_the_first_page`, `view_starts_empty_at_the_default_camera` in `machine/tests.rs`; the existing editor tests for palette, sheet, pages and camera pass unchanged.
- **AC-2** — No input reaches a tool while a modal state is current, by construction, and every modal state leaves with `Escape`.
  Tests: `every_state_leaves_with_escape` (exhaustive over `ModalKind::ALL`), `modal_captures_input`.
- **AC-3** — The snapshot fixtures load and the snapshot of the machine equals the old one.
  Fixtures unchanged; existing snapshot tests, plus `snapshot_of_machine_equals_the_editor_snapshot`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/machine/**`, `crates/draw-core/src/core/editor.rs` (removal of the migrated fields) and any `crates/draw-core/src/core/*.rs` that reads them (amended by the owner, triage 2026-11-12: `editor/**` does not exist)
- this note, `ADR-T135-1`

## Subtasks (one commit each)

- [x] spec — `docs(T135): specify the modal and view regions`
- [x] tests — `test(T135): add failing tests for the regions`
- [x] behaviour — `refactor(T135): move the modal and view regions into the machine`
- [x] quality — `chore(T135): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — resumed: the owner amended *Files owned* in the plan (triage): `core/editor.rs` and the `core/*.rs` that read the fields. ADR-T135-1 records that the types move into `machine/regions.rs` while the storage stays in the editor adapter.
- 2026-11-12 — done: `Modal`, `Pages`, `View` in `machine/regions.rs`; `Editor` uses them. Deviations: models and behaviour share one commit (the types are only useful wired in); `Page` became `pub(crate)`; the storage stays in the editor adapter (ADR-T135-1, UNATTENDED CHOICE). Snapshot fixtures unchanged, `scripts/check.sh` green.
- 2026-11-12 — (earlier, now resolved) BLOCKED (batch): *Files owned* names `crates/draw-core/src/editor/**`, which does not exist; the fields to migrate (`page`, `pages`, `camera`, `selection`, `palette`, `sheet`) live in `crates/draw-core/src/core/editor.rs` (about 10.5k lines), outside the owned files. Tried: checked the tree and the `Editor` struct. Recommended fix: amend *Files owned* to `crates/draw-core/src/core/editor.rs` (and any `core/*.rs` that reads those fields), then re-run T135.
