---
title: Editor snapshots
task: "[[T74 Editor snapshots for tests]]"
adrs: ["[[ADR-T74-1 Line-based editor snapshot format]]", "[[ADR-T107-1 Lock flag kept by the document]]"]
tutorial: ""
shortcuts: ["Ctrl+Shift+S"]
tags: [feature, testing]
---

# Editor snapshots

## What it does

The whole state of the editor (document, undo history, view, tools,
an open text block, a drag in progress, the pen tap filter) can be
saved to a small text file and loaded back exactly. Tests start from a
committed snapshot instead of replaying thousands of events, and a bug
seen in the app becomes a regression test: save the state just before
it, commit the file, and replay the trigger in a test.

This is a debugging and testing tool, not a way to save drawings. The
crash autosave uses the same format ([[Crash recovery]]).

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Save a snapshot of the running app | `Ctrl+Shift+S`: writes `$XDG_STATE_HOME/draw/snapshots/snapshot-<UTC>.snap` (default `~/.local/state/draw/…`), prints the path and logs it |
| Start the app from a snapshot | `draw --snapshot <file>` (a flag for tests, not in `--help`); a missing or bad file prints and logs an error with the line, and the app starts empty |
| Load a fixture in a test | `common::snapshots::load_fixture("text-mid-edit")` (from `tests/snapshots/`) |
| Compare a state with a fixture | `assert_snapshot!(editor, "name")`: panics with a line diff when they differ |
| Rewrite fixtures on purpose | `DRAW_UPDATE_SNAPSHOTS=1 cargo test --test snapshots -- --test-threads=1` |

Committed fixtures, each rebuilt from events in `tests/common/scenes.rs`
by `tests/snapshots.rs`:

| Fixture | State |
|---------|-------|
| `empty` | a new editor |
| `pen-mid-stroke` | a pen stroke being drawn, with pressure |
| `select-mid-drag` | two shapes selected and being moved |
| `text-mid-edit` | an open block, caret inside `$…$` |
| `history-at-limit` | a full undo stack (500) and three redo entries |
| `strokes-1000` | 1 000 pen strokes (the soak large-selection scene) |
| `text-500` | 500 typed blocks and an open one (the perf typing scene) |

Bug snapshots go in `tests/snapshots/regressions/` (see its README and
[[Workflow#Bug fixes: regression step]]).

## How it works

- `src/core/snapshot.rs`: the format (ADR-T74-1). One record per line,
  fixed sections (`[editor]`, `[document]`, `[history]`, `[clipboard]`,
  `[tools]`) and keys in a fixed order, so the text is canonical.
  Floats use `{:?}` and come back bit for bit; strings are quoted and
  escaped. `Writer`/`Line` write tokens, `Reader`/`Tokens` read them,
  and the `Snap` trait gives each value its token form.
- Version 2 ([[ADR-T107-1 Lock flag kept by the document]]): the
  `[document]` section ends with `locked <id>…` (locked ids in z-order)
  when any shape is locked, and the history has `lock <id>` and
  `unlock <id>` edits. A locked id must be in the document, once; the
  selection and the select and eraser gesture ids must not be locked.
- Every state type writes and reads its own records in its module
  (`tools/*`, `tap.rs`, `smoothing.rs`), so no private field becomes
  public. Reading checks the editor invariants: finite shapes, ids in
  the document, carets on character boundaries, values in range, no
  text in shapes with the switch off. Live drag positions may be
  infinite (the tools drop such shapes on commit) and are read without
  that check.
- `Editor::snapshot`, `Snapshot::{to_text, parse}`,
  `Editor::from_snapshot`. `Editor: PartialEq` compares the saved
  state.
- `golden::{check, diff}` in core decide match/write/mismatch; the file
  IO and `DRAW_UPDATE_SNAPSHOTS` are in `tests/common/snapshots.rs`.
- Shell (`src/shell/app.rs`): `SNAPSHOT_CHORD` is caught before the
  editor like the dump chord; `startup_snapshot` reads the `--snapshot` path.
- Fuzzed: round trips continue equal under random sessions (also with
  text in shapes and the pen tap filter), and random or mutated text
  never panics the parser or the loaded editor (`tests/fuzz.rs`).

## Document files

T108 ([[ADR-T108-1 The draw file format and its promise]]) adds the file
a user keeps, in the same text syntax but with a different header,
`draw-file 1`, and only the document: for every non-empty page its view,
number counter and `[document]` (shapes, text, locks), with the current
page named. No history, selection, clipboard, tools or settings.

| Action | How |
|--------|-----|
| Save | `Ctrl+S`: to the file named on the command line, else a new `~/draw-<date>-<time>.draw` that later presses reuse; written through a temporary file and a rename on a thread; the path is printed and logged |
| Open | `draw file.draw`; a missing file starts empty and the first save creates it |
| Bad file | damaged, truncated or newer: printed and logged with the line, the canvas starts empty and the file is never overwritten (the next save goes to a new default file) |

`Ctrl+S` does nothing while a text block is open (every key types there):
finish it first. Compatibility promise: every later build reads version 1
and up; `tests/files/v1.draw` must keep loading, and the 2.0.0 refactor
(T50) must keep it. Code: `src/core/file.rs`, `src/shell/file.rs`; tests:
`tests/files.rs` and `file_*` in `tests/fuzz.rs`.

## Limits

- Not saved: the keymap (a `fn` pointer; a loaded editor uses the
  default), the document revisions (a load is a fresh start), the
  history's change count and byte estimate (recomputed, so byte-budget
  eviction right after a load can differ slightly).
- Version 3 (T116) stores every page: the `[editor]` sections describe the
  current page, a final `[pages]` section names it and counts the stored
  pages, and each stored page has a `[page]` section with its view, number
  counter, selection, document and history. Version 2 still loads, as one
  page; recovery restores the page that was open.
- No other compatibility between format versions: a new `VERSION` means
  regenerating fixtures and regression snapshots.
- A loaded snapshot replaces the old experimental switch and tablet
  settings with the ones it was saved with.
- Two snapshots saved in the same second overwrite each other.
- No full 64 MiB history fixture (too large for the repository); the
  history unit tests cover the byte budget.
