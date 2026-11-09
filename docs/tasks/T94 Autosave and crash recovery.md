---
id: T94
title: Autosave and crash recovery
status: in-progress
wave: 51
branch: task/T94-autosave
depends_on: []
adrs: [ADR-T94-1]
feature: "[[Crash recovery]]"
tutorial: ""
tags: [task]
---

# T94 Autosave and crash recovery

## Goal

After a crash or a stray close, `draw --recover` restores the last autosave. (idea K2 of [[Feature proposal]])

## Known facts (2026-11-09)

- The owner accepted this on 2026-11-09, full save and load stay out.
- `Editor::snapshot`, `Snapshot::parse` and `Editor::from_snapshot` already ship in release, and `DRAW_SNAPSHOT` shows the start-up path (`startup_snapshot` in `shell::app`).
- `shell::crash::state_dir` gives the state directory.
- The event loop blocks until an event, so a timer would break the idle budget ([[ADR-T12-1 Blocking event loop with cached frame]]).
- Snapshots have no compatibility between versions ([[Editor snapshots#Limits]]).

## Spec

Tests live in `src/shell/autosave.rs` (`mod tests`) unless a file is named.
Design: [[ADR-T94-1 Autosave captured on input and written off the frame]].

- **AC-1** — An autosave happens on an input event at least 30 s after the last one (the session start counts as the first), only if the document changed and no gesture or text edit is open. No timer: idle stays at 0 %.
  Tests: `policy_is_not_due_before_the_interval`, `policy_is_due_after_the_interval_with_a_change`, `policy_is_not_due_without_a_document_change`, `policy_is_not_due_while_editing`, `autosaver_saves_on_input_after_the_interval`, `autosaver_waits_for_an_open_gesture`, `autosaver_waits_for_an_open_text_edit`, `autosave_uses_no_timer`.
- **AC-2** — The write is atomic (temporary file, then rename), with mode `0600`, off the render path.
  Tests: `write_atomic_leaves_only_the_file`, `write_atomic_replaces_the_old_contents`, `write_atomic_sets_mode_0600`, `write_atomic_tightens_a_stale_temporary_file`, `autosaver_writes_on_another_thread`.
- **AC-3** — The first autosave of a session moves the existing file to `autosave.prev.snap`, so a crashed session survives one normal launch.
  Tests: `first_autosave_moves_the_existing_file_to_prev`, `later_autosaves_keep_prev`, `first_autosave_without_a_file_makes_no_prev`, `failed_write_keeps_the_existing_autosave`.
- **AC-4** — `draw --recover` opens `autosave.snap`; `draw --recover prev` opens the older one.
  Tests: `launch_without_recover_is_normal`, `launch_recover_is_latest`, `launch_recover_prev_is_previous`, `launch_recover_other_is_invalid`, `recover_latest_opens_autosave`, `recover_prev_opens_the_older_file`.
- **AC-5** — A missing, corrupt or wrong-version file prints the reason and starts blank, never panics. A failed write goes to the diagnostics log and never disturbs drawing.
  Tests: `recover_missing_file_gives_the_reason`, `recover_corrupt_file_gives_the_line`, `recover_wrong_version_gives_the_reason`, `recover_without_a_state_directory_gives_the_reason`, `recover_invalid_launch_gives_the_reason`, `recover_any_bytes_never_panics`, `failed_write_is_reported_and_retried`, `autosaver_without_a_state_directory_is_inert`.
- **AC-6** — A clean exit leaves the files in place.
  Test: `dropping_the_autosaver_leaves_the_files`.
- **AC-7** — The serialisation time for the 5 120-stroke scene is measured and recorded in the ADR against the overlay budget of ADR-T24-5.
  Tests: `perf_autosave_capture_of_5120_strokes_is_under_2ms`, `perf_autosave_text_of_5120_strokes_is_measured`, `perf_autosave_with_full_history_is_measured` (`tests/perf.rs`); numbers in ADR-T94-1.
- **AC-8** — `README.md` *Limitations* and `docs/Home.md` no longer claim that nothing is ever saved.
  Test: `readme_and_home_do_not_claim_nothing_is_saved`.

## Out of scope

- Save and load, file names, or a recovery prompt at start-up.

## Files owned

- `src/main.rs`
- `src/shell/app.rs`, `src/shell/crash.rs`
- new `src/shell/autosave.rs`, `src/shell/mod.rs`
- `docs/Home.md`
- `docs/features/Crash recovery.md` (new)
- `docs/features/Undo and redo.md`
- `docs/features/Editor snapshots.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/perf.rs`, `docs/architecture/Architecture.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T94-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T94): specify autosave and crash recovery`
- [ ] tests — `test(T94): add failing tests`
- [ ] models — `feat(T94): add types`
- [ ] behaviour — `feat(T94): implement`
- [ ] quality — `chore(T94): pass clippy and rustfmt`
- [ ] docs — `docs(T94): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — UNATTENDED CHOICE: added `tests/perf.rs` (AC-7 needs a release timing) and `docs/architecture/Architecture.md` (new shell module) to *Files owned*.
- 2026-11-09 — UNATTENDED CHOICE: the plan's default serialises on the main thread only if under budget. The text takes 31 ms for 5 120 strokes, so the main thread only clones the state (1.2 ms) and the writer thread makes the text; see ADR-T94-1.
- 2026-11-09 — UNATTENDED CHOICE: the 30 s of the first autosave count from the session start, and the temporary file is named per process so two windows cannot mix their writes; see ADR-T94-1.
- 2026-11-09 — The spec step took two commits: an edit of this note failed, so the first commit holds only ADR-T94-1 although its message describes the criteria too. The second holds this note. History was left as it is.
