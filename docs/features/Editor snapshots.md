---
title: Editor snapshots
task: "[[T74 Editor snapshots for tests]]"
adrs: ["[[ADR-T74-1 Line-based editor snapshot format]]"]
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

This is a debugging and testing tool, not a way to save drawings.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Save a snapshot of the running app | `Ctrl+Shift+S`: writes `$XDG_STATE_HOME/draw/snapshots/snapshot-<UTC>.snap` (default `~/.local/state/draw/…`), prints the path and logs it |
| Start the app from a snapshot | `DRAW_SNAPSHOT=<file> draw`; a missing or bad file prints and logs an error with the line, and the app starts empty |
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
  editor like the dump chord; `startup_snapshot` reads `DRAW_SNAPSHOT`.
- Fuzzed: round trips continue equal under random sessions (also with
  text in shapes and the pen tap filter), and random or mutated text
  never panics the parser or the loaded editor (`tests/fuzz.rs`).

## Limits

- Not saved: the keymap (a `fn` pointer; a loaded editor uses the
  default), the document revisions (a load is a fresh start), the
  history's change count and byte estimate (recomputed, so byte-budget
  eviction right after a load can differ slightly).
- No compatibility between format versions: a new `VERSION` means
  regenerating fixtures and regression snapshots.
- A loaded snapshot replaces the `DRAW_EXPERIMENTAL` and tablet
  settings with the ones it was saved with.
- Two snapshots saved in the same second overwrite each other.
- No full 64 MiB history fixture (too large for the repository); the
  history unit tests cover the byte budget.
