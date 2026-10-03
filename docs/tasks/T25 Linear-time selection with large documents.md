---
id: T25
title: Linear-time selection with large documents
status: done
wave: 13
branch: task/T25-fast-lookup
depends_on: [T14]
adrs: ["[[ADR-T06-1 Self-checking edits and rollback atomicity]]", "[[ADR-T14-1 Performance budgets]]", "[[ADR-T25-1 Lazy position index and batched edit runs]]", "[[ADR-T25-2 Id hasher and selection-scale budgets]]"]
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

Design: [[ADR-T25-1 Lazy position index and batched edit runs]]. Unit tests
live in `#[cfg(test)] mod tests` of the named module; perf tests in
`tests/perf.rs` (ignored, release, [[ADR-T14-1 Performance budgets]]).

- **AC-1 — O(1) lookup.** `Document::get` and `index_of` use a lazily built
  id → position index; the public API is unchanged. Structural edits
  (insert, remove) invalidate it, a replace keeps it, and a failed
  transaction leaves the document and its lookups exactly as before.
  *Tests* (`core::document`):
  `index_of_after_middle_remove_is_shifted`,
  `index_of_after_middle_insert_is_shifted`,
  `get_after_replace_returns_new_shape`,
  `failed_transaction_keeps_lookups`,
  `lookups_match_a_scan_after_random_transactions` (proptest).
- **AC-2 — Linear transactions.** A run of `Remove` edits with strictly
  decreasing indices (what `tx_remove`/`tx_clear` build), and a run of
  `Insert` edits with strictly increasing indices (their inverses, i.e.
  undo), is applied in one O(n + k) pass. The result and the error are
  the same as applying the edits one by one. *Tests*
  (`core::document`):
  `apply_matches_edit_by_edit_application` (proptest over random
  transactions, including invalid ones),
  `apply_remove_run_with_mismatch_changes_nothing`,
  `apply_insert_run_with_duplicate_id_changes_nothing`.
- **AC-3 — Eraser bookkeeping.** The eraser's marked and cleared shapes get
  O(1) membership checks; marking order, preview and committed edits are
  unchanged. *Tests* (`core::tools::eraser`): the existing eraser tests,
  plus `long_drag_marks_each_shape_once`.
- **AC-4 — Budget.** On the 10 000-shape benchmark document with every
  other shape selected (5 000), each of these has a median < 1 ms
  (`tests/perf.rs`), recorded in an ADR amending
  [[ADR-T14-1 Performance budgets]]:
  - `perf_lookup_5k_selected_of_10k_is_under_1ms` — `get` for every
    selected id (selection bounds, prune);
  - `perf_move_commit_5k_of_10k_is_under_1ms` — applying the `Replace`
    transaction of a move;
  - `perf_delete_5k_of_10k_is_under_1ms` — applying `tx_remove`;
  - `perf_undo_delete_5k_of_10k_is_under_1ms` — applying its inverse.
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

- [x] spec — `docs(T25): specify linear-time lookup acceptance criteria`
- [x] tests — `test(T25): add failing tests for indexed lookup`
- [x] models — `feat(T25): add the document position index`
- [x] behaviour — `feat(T25): look shapes up by index`
- [x] quality — `chore(T25): pass clippy and rustfmt`, then
  `refactor(T25): hash shape ids with a multiplicative hasher`
- [x] docs — `docs(T25): add tutorial and measurements`

## Learning path

Step 25 — requires step 6 (transaction log) and step 14 (profiling).

## Log

- 2026-10-02 — Created from the T24 measurements; not started.
- 2026-10-02 — Spec refined; ADR-T25-1 added. `src/core/tools/select.rs`
  needs no change: it already uses a set for the moved shapes, and its
  `prune` becomes linear through `Document::get`.
- 2026-10-02 — Tests: the AC-1–AC-3 correctness tests pass on the old code
  too (they pin behaviour; the oracle `model_apply` was checked against
  it). The red part is AC-4: 27–40 ms medians against 1 ms (busy machine).
- 2026-10-02 — Behaviour: lazy position index, batched remove and insert
  runs, linear roll-back through the same runs, and eraser sets. On an idle
  target: lookup 0.29 ms, delete 0.48–1.37 ms, move commit 0.92–1.38 ms,
  undo delete 0.94–1.20 ms — flaky at 1 ms. SipHash was half the cost →
  `IdHasher` (ADR-T25-2). After: lookup 0.09–0.12 ms, delete 0.26 ms, move
  commit 0.46 ms, undo 0.39 ms, three runs green. T14 budgets unchanged
  (culling 0.23 ms, hit-test 0.39 ms, eraser scan 0.30 ms, snap 0.97 ms).
- 2026-10-02 — Deviation: `median_time` in `tests/perf.rs` now drops the
  op's output after stopping the clock (otherwise freeing a 10 000-shape
  document is timed). Beyond "new benchmarks" in *Files owned*; existing
  budgets are unaffected (their outputs are scalars). Recorded in
  ADR-T25-2.
- 2026-10-02 — `scripts/check.sh` green (558 lib tests);
  `PROPTEST_CASES=20000` fuzz green; document properties green at 5 000
  cases. Perf runs need a quiet machine: a parallel C++ build (load ≈ 10)
  doubled every timing, T14's included.
