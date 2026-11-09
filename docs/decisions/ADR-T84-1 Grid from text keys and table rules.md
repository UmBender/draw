---
id: ADR-T84-1
title: Grid from text keys and table rules
status: accepted
kind: decision
date: 2026-11-09
task: T84
builds_on: [ADR-T83-1, ADR-T11-1, ADR-T16-1, ADR-T22-1, ADR-T31-1, ADR-T47-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T84-1 Grid from text keys and table rules

## Context

A selected text block, or the system clipboard, should become a grid in
one key: lines are rows, whitespace separates cells
([[T84 Grid from text]]). New keys need an ADR ([[Keymap]]). The clipboard
only exists in the running app, `core` knows nothing about the diagnostics
log, and [[ADR-T83-1 Paste text through an input event]] fixed that
`InputEvent::Paste` with no open edit does nothing.

## Decision

- **Keys.** `Ctrl+T` is bound to `Command::GridFromText`. `Ctrl+Shift+V` is
  not in the keymap: `Command` is `Copy` and has no text, so the shell
  reads the clipboard, as for `Ctrl+V` in a text block.
- **Event.** `InputEvent::PasteGrid(String)` carries the text. The shell
  sends it for a press of exactly `Ctrl+Shift+V` with no text edit open
  (`input_map::wants_grid_clipboard`) and an empty or unavailable clipboard
  sends nothing. With an edit open the event does nothing.
- **One pure parser.** `tools::grid::TextTable::parse` serves both keys.
  Lines split on `\n` and `\r`, cells on Unicode whitespace; control
  characters inside a token are dropped; blank lines are ignored; the
  widest row gives the column count and shorter rows leave empty cells.
- **Refusals.** More than 64 columns or rows, or more than 64 KiB of text
  (`PASTE_MAX_BYTES`), is refused whole, never cut: a table with a part
  missing looks right and is wrong. The 64 KiB is checked on the raw text,
  so nothing the parser sees was cut.
- **Reasons reach the log through the editor.** `Editor::take_notice`
  hands the shell the reason of the latest refusal, and the shell writes it
  to the diagnostics log at `warn`. The notice is transient: it is not part
  of the saved state or of `Editor`'s equality.
- **Nothing happens on a refusal.** The checks run before the gesture is
  cancelled, so a refused or empty key leaves the editor as it was.
- **Placement.** `Ctrl+T` puts the grid's corner at the block's top-left,
  replacing the block in place: same id, same z-order, still selected, its
  colour kept, one undo step. `Ctrl+Shift+V` puts the corner at the cursor,
  on top, in the current colour, and selects it.
- **Cell size.** Every cell has the same size. Its width is the widest
  token's box and its height the tallest token's, both divided by
  `SHAPE_TEXT_FILL` so that the text fits at full size; the width is never
  less than the height, so a matrix of digits gets square cells.
- **Text size.** `Ctrl+T` keeps the block's own size. `Ctrl+Shift+V` uses
  the size a new text block would get.
- **Axis indices** follow numbering, as for any new grid
  ([[ADR-T22-1 Numbering toggle drives grid indices]]).

## Alternatives considered

- **Overload `InputEvent::Paste` outside an edit** — contradicts
  ADR-T83-1 and would make the same event mean two things.
- **Centre the pasted grid on the cursor, like `Ctrl+V` for shapes** — a
  big table would start off screen; the top-left is where reading starts,
  and it matches `Ctrl+T`.
- **Reuse `text::clean_paste`** — it cuts at 64 KiB and turns tabs into
  spaces, which would hide what the limit is there to refuse.
- **Pass a logging callback into `core`** — `core` would own a side
  channel the fuzzer and the snapshots cannot see; a value the shell takes
  is simpler.

## Consequences

- Formulas containing spaces are split like any text, as the task states.
- The editor holds one reason at a time, so the shell takes it after every
  event.
- A block over 64 KiB cannot be turned into a grid. A real 64 × 64 table
  averages 16 bytes per cell at that size.

## Rollback plan

Remove `Command::GridFromText`, its binding, `InputEvent::PasteGrid`,
`TextTable`, `Editor::take_notice` and the shell branch with `revert(T84)`
commits and a rollback ADR. No stored format changes.
