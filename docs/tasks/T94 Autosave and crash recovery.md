---
id: T94
title: Autosave and crash recovery
status: todo
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

The spec step names the test for each criterion.

- **AC-1** — An autosave happens on an input event at least 30 s after the last one, only if the document changed and no gesture or text edit is open. No timer: idle stays at 0 %.
- **AC-2** — The write is atomic (temporary file, then rename), with mode `0600`, off the render path.
- **AC-3** — The first autosave of a session moves the existing file to `autosave.prev.snap`, so a crashed session survives one normal launch.
- **AC-4** — `draw --recover` opens `autosave.snap`; `draw --recover prev` opens the older one.
- **AC-5** — A missing, corrupt or wrong-version file prints the reason and starts blank, never panics. A failed write goes to the diagnostics log and never disturbs drawing.
- **AC-6** — A clean exit leaves the files in place.
- **AC-7** — The serialisation time for the 5 120-stroke scene is measured and recorded in the ADR against the overlay budget of ADR-T24-5.
- **AC-8** — `README.md` *Limitations* and `docs/Home.md` no longer claim that nothing is ever saved.

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
