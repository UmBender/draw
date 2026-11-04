---
id: T74
title: Editor snapshots for tests
status: done
wave: 33
branch: task/T74-editor-snapshots
depends_on: []
adrs: ["[[ADR-T74-1 Line-based editor snapshot format]]"]
feature: "[[Editor snapshots]]"
tutorial: ""
tags: [task, testing]
---

# T74 Editor snapshots for tests

## Goal

The whole state of the headless `Editor` can be saved to a small text
file (a *snapshot*) and loaded back exactly. A test can then start from a
known state, such as a 5 000-stroke scene, a text block being edited with
the cursor in a formula, a half-finished drag, or a full undo history,
instead of rebuilding it from thousands of synthetic events each time.
When the owner hits an edge case in the running app, one key saves a
snapshot, and that file becomes a regression test. This lets the repo
test far more situations: fixture-based unit tests, "replay from here"
fuzzing, and golden comparisons of state after an input script.

## Known facts (2026-11-03)

- All app logic is in `core::editor::Editor` (`src/core/editor.rs:160`),
  already headless and driven by `Editor::handle(InputEvent)`. Its state:
  `doc`, `history`, `camera`, `selection`, `clipboard`, `tools`
  (per-tool states), `tool`, `style`, `smoothing`, `toolbar_visible`,
  `viewport`, `cursor`, `space_held`, `gesture` (in-progress pan or tool
  drag), `reedit`, `keymap` (macros), `tap` (pen tap filter), `pressure`,
  `doc_revision`, `doc_base_revision`.
- Today tests rebuild state by replaying events (`tests/soak.rs`
  `editor()`/`drag()`, `tests/fuzz.rs`, `tests/perf.rs`). Large scenes
  take seconds in debug builds, and some states (mid-gesture, a given
  tap-filter phase) are awkward to reach.
- No file format exists: documents live only in memory
  ([[T72 Feature proposal]] lists save/load as an idea).
- ADR-0014: minimal dependencies, so no `serde`. The format is written
  and parsed by hand in `core`, which is pure and fuzzed.
- Shell runtime toggles use env vars (`DRAW_MSAA`, `DRAW_FRAME_TIMES`)
  and debug keys (`Ctrl+Shift+L` dumps the T47 log).

## Proposal (settle in the spec step)

1. **Format** `core::snapshot`: a versioned, line-based UTF-8 text
   (`draw-snapshot 1` header), readable in diffs and editable by hand for
   tiny fixtures. Floats are written with round-trip precision (`{:?}`)
   so loading gives back the same bits. One section per field group:
   document (shapes with ids), history stacks, camera, tools and style,
   selection, clipboard, keymap, transient input state (gesture, tap,
   pressure, cursor, space).
2. **API**: `Editor::snapshot(&self) -> Snapshot`,
   `Snapshot::to_text`, `Snapshot::parse(&str) -> Result<Snapshot,
   SnapshotError>`, `Editor::from_snapshot(Snapshot) -> Result<Editor,
   …>`. Errors give a line number; bad input never panics.
   Revisions are not saved. Loading starts a fresh revision, so the
   shell re-renders everything.
3. **Equality**: `Editor` (or a `Snapshot`) gets a structural
   comparison, so a test can assert "after these events the state equals
   `expected.snap`", with a readable diff of the text on failure.
4. **Test helpers** (`tests/common/snapshots.rs` or a `core::testing`
   module behind `#[cfg(test)]`/a feature): `load_fixture("name")`, and
   `assert_snapshot!(editor, "name")` comparing to
   `tests/snapshots/name.snap`, with `DRAW_UPDATE_SNAPSHOTS=1` to rewrite
   golden files deliberately.
5. **Capture from the app**: a debug key (e.g. `Ctrl+Shift+S`, check for
   clashes in the keymap) writes
   `$XDG_STATE_HOME/draw/snapshots/<timestamp>.snap` and logs the path in
   the T47 log. `DRAW_SNAPSHOT=<file>` starts the app from a snapshot, so
   the owner can reproduce an edge case by hand too.
6. **Starter fixtures**: empty, 5 000 strokes (the perf scene), 500
   text blocks with formulas, a text block mid-edit, a mid-drag selection
   move, a full byte-budget history; port the slow setups in
   `tests/soak.rs` and `tests/perf.rs` to them where it shortens the
   tests.

## Spec

Settled 2026-11-03 ([[ADR-T74-1 Line-based editor snapshot format]]):

- `core::snapshot`: `VERSION = 1`, `Snapshot`, `SnapshotError { line,
  message }`, `Writer`/`Reader` token helpers, `Editor::snapshot`,
  `Snapshot::{to_text, parse}`, `Editor::from_snapshot` (infallible:
  `parse` validates). `Editor: PartialEq` ignores the keymap, the
  revisions and `History::{changes, bytes}`.
- Golden helper, pure part in `core::snapshot::golden`: `diff(expected,
  actual) -> String` (line diff) and `check(actual, expected: Option<&str>,
  update) -> Golden` (`Match`, `Write`, `Mismatch(diff)`, `Missing`).
  File IO and `DRAW_UPDATE_SNAPSHOTS` in `tests/common/snapshots.rs`:
  `load_fixture(name) -> Editor`, `assert_snapshot!(editor, name)`
  against `tests/snapshots/<name>.snap`.
- Shell: `Ctrl+Shift+S` (`SNAPSHOT_CHORD`, caught before the editor like
  `Ctrl+Shift+L`) writes `<state dir>/snapshots/snapshot-<UTC stamp>.snap`
  and logs the path at `INFO`; `DRAW_SNAPSHOT=<file>` starts from it, and
  a missing or bad file logs an `ERROR` and starts empty. Pure helpers:
  `snapshot_path(state_dir, unix_secs)`, `startup_snapshot(value,
  read) -> (Option<Snapshot>, Option<String>)`.
- Fixtures (`tests/snapshots/`): `empty`, `strokes-1000` (the soak
  large-selection scene), `text-500` (the perf typing scene),
  `text-mid-edit` (open block, caret inside `$…$`), `select-mid-drag`
  (move in progress), `pen-mid-stroke` (live smoother),
  `history-at-limit` (`HISTORY_LIMIT` undo entries). A full 64 MiB
  byte-budget history is not committed (too large for the repo); the
  T49 unit tests cover it.
- `tests/snapshots/regressions/README.md` explains the bug-fix flow.

Acceptance criteria:

- **AC-1 Round trip.** For any state reachable by input,
  `from_snapshot(parse(to_text(snapshot(ed))))` equals `ed`, and the same
  further events keep them equal. *Tests:* `tests/fuzz.rs`
  `snapshot_round_trip_continues_equal` (proptest: session → snapshot →
  load → continuation), also with shape text and pen taps on;
  `snapshot::tests::round_trip_mid_gesture_states`.
- **AC-2 Stable text.** `to_text(parse(text)) == text` for every
  committed fixture. *Test:* `snapshot::tests::fixtures_are_canonical`.
- **AC-3 Total parser.** Any input parses or fails with a line number,
  never panics; loaded state keeps the invariants. *Tests:*
  `tests/fuzz.rs` `snapshot_parse_is_total` (arbitrary text) and
  `snapshot_mutated_text_is_total` (mutated valid text, loaded and fed
  events, invariants checked); unit tests
  `parse_rejects_non_finite_float`, `parse_rejects_duplicate_id`,
  `parse_rejects_selection_outside_document`,
  `parse_rejects_caret_off_char_boundary`, `parse_error_names_line`,
  `strings_round_trip_with_escapes`.
- **AC-4 Versioning.** *Tests:* `snapshot::tests::rejects_unknown_version`,
  `rejects_unknown_section`, `rejects_unknown_key`.
- **AC-5 Golden helper.** *Tests:* `snapshot::golden::tests::
  check_matches_equal_text`, `check_mismatch_has_line_diff`,
  `check_writes_only_when_updating`, `check_missing_without_update`;
  `tests/snapshots.rs` `assert_snapshot_passes_on_fixture`,
  `assert_snapshot_panics_with_diff_on_change`.
- **AC-6 Shell.** *Tests:* `shell::app::tests::snapshot_chord_is_ctrl_shift_s`,
  `snapshot_path_is_in_state_dir`, `startup_snapshot_unset_is_none`,
  `startup_snapshot_missing_file_reports_error`,
  `startup_snapshot_bad_file_reports_line`,
  `startup_snapshot_loads_valid_file`; manual check in *Log*.
- **AC-7 Faster tests.** `tests/soak.rs` large-selection test and
  `tests/perf.rs` typing test load `strokes-1000` and `text-500`;
  `tests/snapshots.rs` fixture tests rebuild each fixture from events and
  compare (the large ones `#[ignore]`d, run them to regenerate). Wall
  time before/after in *Log*.

## Out of scope

- A user-facing save/open of documents (that is a feature for
  [[T72 Feature proposal]] to accept; it may reuse this format later, but
  snapshots carry transient input state a document file should not).
- Backward compatibility across format versions (fixtures are
  regenerated with `DRAW_UPDATE_SNAPSHOTS=1`).
- Snapshots of shell or GPU state (caches, glyph atlas, backend).

## Files owned

- `src/core/snapshot.rs` (new) and its `mod` line in `src/core/mod.rs`
- `src/core/editor.rs` (snapshot/restore, equality); `Eq`/accessor
  derives in `document.rs`, `history.rs`, `shape.rs`, `camera.rs`,
  `keymap.rs`, `clipboard.rs`, `tap.rs`, `tools/*` as needed, and
  `smoothing.rs` (the live pen stroke holds a `Smoother`; added in the
  spec step)
- `docs/architecture/Architecture.md` (module map row for `core::snapshot`)
- `src/shell/app.rs`, `src/shell/input_map.rs` (debug key, `DRAW_SNAPSHOT`)
- `tests/snapshots/` (new fixtures, and an empty `regressions/` with a README), `tests/common/` (helpers),
  `tests/snapshots.rs` (new: golden helper and fixture tests; added in the spec step),
  `tests/fuzz.rs`, `tests/soak.rs`, `tests/perf.rs`
- `docs/features/Editor snapshots.md` (new),
  `docs/decisions/ADR-T74-1 …` (format and versioning)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T74): …`
- [x] tests — `test(T74): …`
- [x] models — `feat(T74): …` (format, parse, write)
- [x] behaviour — `feat(T74): …` (editor restore, shell key and env, fixtures)
- [x] quality — `chore(T74): …`
- [x] docs — `docs(T74): …`

## Learning path

Optional tutorial: "Writing a test from a snapshot", after the fuzz
harness tutorial.

## Log

- 2026-11-03 — note written at the owner's request.
- 2026-11-03 — once this is `done`, every bug fix starts with a snapshot-based
  regression test ([[Workflow#Bug fixes: regression step]]); the
  `tests/snapshots/regressions/` folder and the `Ctrl+Shift+S` key serve that.
- 2026-11-03 — spec: the keymap (a `fn` pointer), revisions and
  `History::{changes, bytes}` are not saved (ADR-T74-1). `smoothing.rs`
  and `tests/snapshots.rs` added to *Files owned*.
- 2026-11-03 — tests: red at compile time (no snapshot API, no
  `Editor: PartialEq`, no shell helpers, no golden macro). Two test fixes
  in later steps: `AssertUnwindSafe` around the `catch_unwind` closure
  (models; `Document` holds a `OnceCell`), and `history-at-limit` leaves
  `HISTORY_LIMIT - 3` undo entries, not `HISTORY_LIMIT` (behaviour).
- 2026-11-03 — bug found by the round-trip fuzz at 5 000 cases: a grid
  or shape drag can hold an infinite world point (zoomed out, huge
  screen position) until the commit drops the shape; the parser
  rejected it. Live drag positions (shape, grid, select) are now read
  without the finite check; shapes keep it. The two shrunk seeds stay in
  `tests/fuzz.proptest-regressions`. Afterwards the five snapshot
  properties passed at `PROPTEST_CASES=20000`.
- 2026-11-03 — AC-7: the setups were not slow. In a debug build,
  rebuilding `strokes-1000` from events takes ≈ 0.04 s and loading it
  ≈ 0.03 s; `soak_large_selection_moves_keep_history_within_budget` takes
  1.7 s before and after (its 600 selection moves dominate). The perf
  typing test (release, ignored) loads `text-500`; median 346 ns. The
  two large fixture tests are therefore not `#[ignore]`d: every
  fixture is rebuilt from events and compared on each `cargo test`
  (0.06 s for the whole `tests/snapshots.rs`). The value of the fixtures
  is reaching mid-gesture states and regressions, not speed.
- 2026-11-03 — sizes: `strokes-1000.snap` 324 KB, `text-500.snap` 84 KB,
  `history-at-limit.snap` 52 KB, the rest under 1 KB. No full byte-budget
  history fixture (64 MiB).
- 2026-11-03 — no tutorial (frontmatter `tutorial: ""`; the optional
  "Writing a test from a snapshot" is left for later);
  `tests/snapshots/regressions/README.md` gives the steps.
- 2026-11-03 — `src/shell/input_map.rs` needed no change: the collector
  already delivers `Ctrl+Shift+S` as a `KeyDown`.
- 2026-11-04 — `scripts/check.sh --fuzz` found two more bugs at 20 000
  cases: an open text block at an infinite world origin was rejected on
  load, and a loaded text size beyond the editor's range (1e30) let typing
  overflow the block's bounds. Regression tests
  `core::snapshot::tests::regression_T74_*` committed red (the grid-drag
  one passes; that bug was fixed in the behaviour step before the
  regression step existed for it), then `fix(T74)`: text origins read
  loosely, text sizes capped at `snapshot::MAX_TEXT_SIZE` (76 px at zoom
  0.05, + 1 %). `scripts/check.sh --fuzz` green afterwards.
- 2026-11-04 — AC-6 manual, part 1: `DRAW_SNAPSHOT` with a missing file
  and with `[nonsense]` print and log `ERROR … line 2: unknown section …;
  starting empty`; with `tests/snapshots/text-mid-edit.snap` the log says
  `INFO started from DRAW_SNAPSHOT snapshot`. Part 2, for the owner:
  press `Ctrl+Shift+S` in the app, check the printed path, and reopen it
  with `DRAW_SNAPSHOT` to see the same view and open block.
- 2026-11-04 — AC-6 manual, part 2: owner checked `Ctrl+Shift+S` and
  reopening the file in the app; works.
- 2026-11-04 — integrated on `main` (merge `3d52522`), no conflicts.
