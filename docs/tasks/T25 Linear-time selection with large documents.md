---
id: T25
title: Linear-time selection with large documents
status: ready
wave: 13
branch: task/T25-fast-lookup
depends_on: [T14]
adrs: ["[[ADR-T06-1 Self-checking edits and rollback atomicity]]", "[[ADR-T14-1 Performance budgets]]"]
feature:
tutorial: "[[25 Indexing a z-ordered store]]"
tags: [task]
---

# T25 Linear-time selection with large documents

## Goal

With a big selection, clicks, releases and the selection outline cost tens
of milliseconds. Every operation that walks the selection asks the document
for each id, and `Document::get` / `Document::index_of` find a shape by a
linear scan, so these passes are O(selection × document).

Found during [[T24 Smooth redraws with large documents]] (2026-10-02). The
session was 10 shapes, then select all + duplicate ×9, giving 5 120 shapes
with 2 560 selected. A release-mode probe through `Editor` measured:

| Event | CPU |
|-------|-----|
| `Editor::selection_bounds` (drawn on every overlay redraw) | 15.5 ms |
| Pointer up after a pen or ellipse drag (`prune_selection`) | ≈ 10 ms |
| Select-tool press | 14 ms |
| Releasing a move of the selection | 45 ms |
| Eraser move / release | 3 ms / 20 ms |

`perf` on the target during a pan session put ≈ 13 % of all samples in
`ShapeId`'s `==` (`document.rs`), called from these scans.

## Spec

Draft — refined in the spec step (each AC names its tests, ADRs added).

- **AC-1 — O(1) lookup.** `Document::get`, `index_of` and the id checks in
  `apply` use an id → position index instead of a scan; the public API is
  unchanged. Inserts and removes keep the index right, including rollback
  of a failed transaction ([[ADR-T06-1 Self-checking edits and rollback atomicity]]).
  *Tests:* index stays consistent after insert/remove/replace/undo/rollback
  (unit + proptest against a scan).
- **AC-2 — Linear transactions.** Applying a transaction that removes or
  replaces k of n shapes costs O(n + k), not O(n · k).
- **AC-3 — Eraser bookkeeping.** The eraser's `marked` / `cleared` sets
  give O(1) membership checks; behaviour and preview order unchanged.
- **AC-4 — Budget.** New `tests/perf.rs` benchmarks on the 10 000-shape
  document with half of it selected: selection bounds, prune, move commit
  and delete each < 1 ms (median), recorded in an ADR amending
  [[ADR-T14-1 Performance budgets]].
- **AC-5 — No regressions.** `scripts/check.sh`, the fuzz harness and the
  T14 perf tests stay green.

## Out of scope

- Rendering ([[T24 Smooth redraws with large documents]]).
- `src/core/editor.rs` and `src/shell/*` — owned by T24 in the same wave;
  the editor's passes become linear through the faster `Document::get`.
- A spatial index for hit-testing.

## Files owned

- `src/core/document.rs`
- `src/core/tools/select.rs`, `src/core/tools/eraser.rs` (lookups only)
- `tests/perf.rs` (new benchmarks)
- new ADRs `ADR-T25-*`, tutorial `docs/tutorials/25 Indexing a z-ordered store.md`

## Subtasks (one commit each)

- [ ] spec — `docs(T25): specify linear-time lookup acceptance criteria`
- [ ] tests — `test(T25): add failing tests for indexed lookup`
- [ ] models — `feat(T25): add the document position index`
- [ ] behaviour — `feat(T25): look shapes up by index`
- [ ] quality — `chore(T25): pass clippy and rustfmt`
- [ ] docs — `docs(T25): add tutorial and measurements`

## Learning path

Step 25 — requires step 6 (transaction log) and step 14 (profiling).

## Log

- 2026-10-02 — Created from the T24 measurements; not started.
