---
id: ADR-T108-1
title: The draw file format and its promise
status: accepted
kind: decision
date: 2026-11-11
task: T108
builds_on: ["[[ADR-T74-1 Editor snapshots]]", "[[ADR-T116-1 Pages]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T108-1 The draw file format and its promise

## Context

`Ctrl+S` and `draw file.draw` (T108) make a file the user keeps. The
snapshot format is not that file: it holds the undo history, tool state
and settings, and it breaks between versions.

## Decision

- A **document file** is a text file in the snapshot syntax (same
  writer/reader, `src/core/file.rs`). First line `draw-file 1`. Then
  `[pages]` with `current` and `count`, and one `[page]` per non-empty
  page: `index`, `camera`, `number` and the `[document]` section (shapes,
  text, locks). No history, selection, clipboard, tool or settings.
- A page equal to a fresh one is not written; the current page is written
  when it is not fresh. Writing is canonical: loading and saving again
  gives the same bytes.
- **Compatibility promise:** every later build reads every file version
  from 1 up. A version above the build's, a damaged or a truncated file
  is refused with a message; the canvas starts empty. When the shape model
  changes, the reader gains a branch for the old version; the version-1
  fixture `tests/files/v1.draw` must keep loading. The 2.0.0 refactor
  (T50) must keep this.
- `Ctrl+S` writes the command-line file, else `~/draw-<date>-<time>.draw`
  (reused by later presses). The text is built on the frame, the write
  (temporary file in the same directory, then rename) runs on a thread.
  A refused file is never the save target: the next `Ctrl+S` writes a new
  default file instead, so a damaged file is never overwritten.
- `Ctrl+S` is new; plain `Ctrl+S` does not clash with `Ctrl+Shift+S`
  (snapshot). A running text block is committed first, like any command.

## Alternatives considered

- **Reuse the snapshot file** — carries history and settings and has no
  compatibility.
- **JSON or a new dependency** — needs an ADR for a crate and a second
  syntax to maintain.

## Consequences

The shape model is bound to old files. Autosave is unchanged and
separate.

## Rollback plan

Revert T108; the file format promise ends with a new ADR that reverts
this one.
