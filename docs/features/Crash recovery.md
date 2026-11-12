---
title: Crash recovery
task: "[[T94 Autosave and crash recovery]]"
adrs: ["[[ADR-T94-1 Autosave captured on input and written off the frame]]"]
tutorial: ""
shortcuts: []
tags: [feature]
---

# Crash recovery

## What it does

While you draw, draw keeps an autosave of the whole editor: the drawing,
its undo history, the view and the tools. After a crash or a stray close,
`draw --recover` brings it back. There is no prompt at start-up and no way
to pick a file: a normal launch always starts blank. The autosave is not
the drawing file you keep: that is `Ctrl+S` and `draw file.draw`
([[Editor snapshots#Document files]]); they are separate and work together.

## How to use

| Action | Command |
|--------|---------|
| Open the last autosave | `draw --recover` |
| Open the one before it | `draw --recover prev` |

The files are `autosave.snap` and `autosave.prev.snap` in the state
directory (`$XDG_STATE_HOME/draw`, default `~/.local/state/draw`), readable
only by you. A crashed session survives one normal launch: the first
autosave of the new session moves its file to `autosave.prev.snap`.

A missing, corrupt or wrong-version file prints the reason and starts
blank. Closing the window leaves the files in place.

## How it works

- `src/shell/autosave.rs` ([[ADR-T94-1 Autosave captured on input and written off the frame]]).
  `Autosaver::on_input` runs once per frame that had input; there is no
  timer, so an idle window stays at 0 % CPU.
- `Policy` saves when at least 30 s have passed since the last capture (the
  session start counts as the first), the document revision changed, and no
  gesture or text edit is open.
- The main thread only clones the editor (`Editor::snapshot`, 1.2 ms for
  5 120 strokes, within the 2 ms of [[ADR-T24-5 Redraw budgets]]). A
  short-lived thread makes the text and writes it.
- `write_atomic` writes a temporary file of this process with mode `0600`,
  syncs it, rotates the old file on the session's first save, then renames.
  A failure goes to the diagnostics log and is tried again 30 s later.
- `Launch::from_args` reads `--recover`; `recover` turns the file into a
  `Snapshot` or a reason. `--recover` wins over `DRAW_SNAPSHOT`.

## Limits

- Up to 30 s of work can be lost, and more if the window got no input after
  the last change: only an input event can start a save.
- Autosaves follow [[Editor snapshots#Limits]]: a build with a new snapshot
  version cannot read an older file. Tablet settings come from the file.
- With a full 64 MiB undo history a capture takes about 59 ms (one long
  frame, at most every 30 s), and the file can reach about 77 MiB.
- Two open windows share the files: the last write wins, and each session's
  first autosave rotates.
- A `kill -9` during a write can leave a small `autosave.snap.tmp.<pid>`
  behind. It is never read.
- This is not save and load: there are no file names and no way to choose
  where it goes.
