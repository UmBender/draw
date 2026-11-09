---
id: T87
title: Graph or tree from an edge list
status: todo
wave: 53
branch: task/T87-graph-from-edges
depends_on: [T84]
adrs: [ADR-T87-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T87 Graph or tree from an edge list

## Goal

A selected text block of edges becomes a drawn graph or tree in one key. (idea C2 of [[Feature proposal]])

## Known facts (2026-11-09)

- Labels are `Option<u32>`, so node names must be numbers.
- Arrows snap to outlines ([[T20 Outline snapping]]) but the result here is computed directly.
- `Ctrl+L` is unbound; `Ctrl+Shift+L` is the diagnostics dump.

## Spec

The spec step names the test for each criterion.

- **AC-1** — With one text block selected, `Ctrl+L` replaces it with numbered circles and edges, one undo step.
- **AC-2** — It accepts `1-2 2-3`, `1>2` for directed edges, and lines of `u v` or `u v w`; a weight is drawn as text at the edge's middle.
- **AC-3** — If the edges form a tree it is drawn in layers from the first node named; anything else goes on a circle.
- **AC-4** — Edges end on the circles' outlines, and all results are ordinary shapes every tool can edit.
- **AC-5** — More than 200 nodes, or text that does not parse: nothing changes and the reason goes to the diagnostics log.
- **AC-6** — Self-loops and duplicate edges do not panic; a self-loop is skipped.

## Out of scope

- Segment tree templates.
- Force-directed layout.
- A leading `n m` line is read as an edge; there is no way to tell them apart.

## Files owned

- new `src/core/tools/layout.rs`, `src/core/tools/mod.rs`, `src/core/shape.rs`
- `src/core/numbering.rs`
- `docs/features/Numbered nodes.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T87-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T87): specify graph or tree from an edge list`
- [ ] tests — `test(T87): add failing tests`
- [ ] models — `feat(T87): add types`
- [ ] behaviour — `feat(T87): implement`
- [ ] quality — `chore(T87): pass clippy and rustfmt`
- [ ] docs — `docs(T87): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
