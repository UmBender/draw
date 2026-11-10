---
id: ADR-T100-1
title: Z-order keys and restack transactions
status: accepted
kind: decision
date: 2026-11-10
task: T100
builds_on: [ADR-0005, ADR-T06-1, ADR-T11-1, ADR-T29-3]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T100-1 Z-order keys and restack transactions

## Context

Stacking is drawing order: index 0 of the [[Document]] is drawn first and
the top shape takes the bucket click. There was no way to change it
without deleting and redrawing. The owner chose `Ctrl+]` / `Ctrl+[` for
one step and `Ctrl+Shift+]` / `Ctrl+Shift+[` for top / bottom. In text
edit mode `Ctrl+]` / `Ctrl+[` already resize the open block
([[ADR-T29-3 Resize text while typing]]); `Alt+]` / `Alt+[` scale the
selection (ADR-T91-1).

## Decision

- **Keys.** `Ctrl+]` → `Command::RaiseSelection`, `Ctrl+[` →
  `Command::LowerSelection`, `Ctrl+Shift+]` → `Command::SelectionToTop`,
  `Ctrl+Shift+[` → `Command::SelectionToBottom`, with exact modifiers
  (ADR-T11-1). macroquad's `KeyCode` has no brace key: the shell maps
  only `KeyCode::LeftBracket` / `RightBracket`, so the core binds what it
  delivers for those physical keys, `BracketRight` / `BracketLeft` with
  `ctrl` and `shift` held. Whether every backend reports the bracket key
  code (and not an unknown key) while `Shift` is held is a manual check
  in the running app; the core needs no other binding. Key presses in text
  edit mode never reach the keymap (ADR-T29-2), so there the keys keep
  resizing text.
- **Block.** The selected shapes move as one block and keep their order
  among themselves. The unselected shapes keep theirs.
- **One step up.** The step target is the lowest unselected shape above
  the topmost selected shape whose bounds intersect the selection's
  bounds (the union of the selected shapes' bounds, the box the Select
  tool draws). The block lands right above it. One step down mirrors
  this: the highest unselected overlapping shape below the bottommost
  selected shape, block right below it. With no target nothing changes.
- **Top / bottom.** The block goes above (below) every unselected shape.
- **Transaction.** `document::tx_restack(doc, ids, below)` builds the
  move from existing edits: the selected shapes are removed top down,
  then inserted bottom up at `below + k`, where `below` is the number of
  unselected shapes left under the block. Its inverse (remove the
  inserts top down, insert at the old indices bottom up) restores the
  order exactly, ids included. When the order would not change it is
  empty, so [[History]] records nothing. The snapshot format is
  unchanged.
- **Guards.** The commands do nothing while a gesture is in progress,
  with an empty selection, or when the order would not change; the
  selection stays selected.

## Alternatives considered

- **A `Move` edit variant** — smaller transactions for big strokes, but
  a new edit kind for `Document::apply`, its run batching, the snapshot
  format and its fixtures. Remove + insert is already exact.
- **Per-shape overlap** (a target must overlap one selected shape) —
  more precise with a spread selection, but it disagrees with the box
  the user sees; the union box is what "the selection's bounds" means
  elsewhere.
- **Step from the lowest selected shape** — a spread selection would
  then gather around a shape between its members; stepping past the
  shape above the whole selection is the usual "bring forward".

## Consequences

- A restack holds two clones of each selected shape in the undo stack;
  the byte budget (ADR-T49-1) counts them.
- The document revision moves on a restack and the base revision
  follows (not an append), so the shell redraws the document layer.

## Rollback plan

Remove the four bindings and commands, `tx_restack` and the editor's
restack function, in a new ADR with `git revert` of the T100 commits.
