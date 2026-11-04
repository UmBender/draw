---
id: T74
title: Editor snapshots for tests
status: todo
wave: 33
branch: task/T74-editor-snapshots
depends_on: []
adrs: []
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

To be written in the spec step. Expected acceptance criteria:

- **AC-1 Round trip.** For any editor state reachable by input,
  `from_snapshot(parse(to_text(snapshot(ed))))` equals `ed`, and
  feeding both the same further events gives equal states. *Test:*
  proptest in `tests/fuzz.rs` (random event script → snapshot → load
  → random continuation, compare).
- **AC-2 Stable text.** `to_text(parse(text)) == text` for every
  committed fixture. *Test:* `snapshot::tests::fixtures_are_canonical`.
- **AC-3 Total parser.** Any byte string parses to a value or an error
  with a line number, never a panic; loaded shapes pass the same
  invariants as built ones (finite floats, valid ids, selection ⊆
  document). *Test:* `fuzz_snapshot_parse_is_total`.
- **AC-4 Versioning.** An unknown version or section is a clear error;
  the version is a constant bumped (with an ADR) when the format
  changes. *Test:* `snapshot::tests::rejects_unknown_version`.
- **AC-5 Golden helper.** `assert_snapshot!` passes on equal state,
  fails with a text diff otherwise, and rewrites only with
  `DRAW_UPDATE_SNAPSHOTS=1`. *Test:* tests of the helper itself.
- **AC-6 Capture and load in the shell.** The debug key writes a
  snapshot that loads back to the same document and view;
  `DRAW_SNAPSHOT` opens it; a missing or bad file logs an error and
  starts empty. *Test:* unit tests of the path/env parsing; manual check
  in *Log*.
- **AC-7 Faster tests.** At least the large-scene setups in
  `tests/soak.rs`/`tests/perf.rs` load from fixtures; wall time noted in
  *Log*.

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
  `keymap.rs`, `clipboard.rs`, `tap.rs`, `tools/*` as needed
- `src/shell/app.rs`, `src/shell/input_map.rs` (debug key, `DRAW_SNAPSHOT`)
- `tests/snapshots/` (new fixtures, and an empty `regressions/` with a README), `tests/common/` (helpers),
  `tests/fuzz.rs`, `tests/soak.rs`, `tests/perf.rs`
- `docs/features/Editor snapshots.md` (new),
  `docs/decisions/ADR-T74-1 …` (format and versioning)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T74): …`
- [ ] tests — `test(T74): …`
- [ ] models — `feat(T74): …` (format, parse, write)
- [ ] behaviour — `feat(T74): …` (editor restore, shell key and env, fixtures)
- [ ] quality — `chore(T74): …`
- [ ] docs — `docs(T74): …`

## Learning path

Optional tutorial: "Writing a test from a snapshot", after the fuzz
harness tutorial.

## Log

- 2026-11-03 — note written at the owner's request.
- 2026-11-03 — once this is `done`, every bug fix starts with a snapshot-based
  regression test ([[Workflow#Bug fixes: regression step]]); the
  `tests/snapshots/regressions/` folder and the `Ctrl+Shift+S` key serve that.
