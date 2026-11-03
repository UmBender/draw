---
id: T49
title: Long-running stability
status: done
wave: 32
branch: fix/long-running-stability
depends_on: [T47]
adrs: ["[[ADR-T49-1 Undo history capped by bytes]]"]
feature: ""
tutorial: ""
tags: [task]
---

# T49 Long-running stability

## Goal

draw can stay open for a whole contest day (hours, with long idle
periods, screen lock and suspend) without input going wrong or crashing.
Each bug found this way gets its own fix and test; this task finds them
and adds a way to keep finding them.

## Known facts (2026-11-02)

- Owner's reports: letters typed twice or three times after the app has
  been open for a while ([[T44 One keyboard per Wayland seat]],
  [[T45 Doubled keys after a long idle on Wayland]]), and crashes with
  no obvious cause after prolonged use. No crash reports or backtraces
  saved yet.
- The panic hook writes `$XDG_STATE_HOME/draw/crash.log`; check it
  first. Segfaults leave only a core dump (`coredumpctl list draw`).
- Candidates for slow-building problems: compositor events after idle,
  suspend/resume or device re-plug (seat, keyboard, tablet), GL context
  loss after resume, unbounded growth (history, glyph atlas, caches,
  position index), timers and key repeat across focus changes.

## Proposal (settle in the spec step)

1. Prerequisite: [[T47 Always-on diagnostics log]], so every occurrence
   during normal use leaves a log.
2. Collect what exists: `crash.log`, `coredumpctl`, and owner sessions
   with the T47 log.
3. Soak run: a scripted long session (synthetic input through the core
   `Editor`, plus a manual idle/lock/suspend checklist on the target)
   that watches memory, frame times and the T47 anomaly detectors.
4. A bounded-growth test in core: after N thousand edits/undos the
   memory held by history, caches and indices stays within a budget.
5. One sub-entry in *Log* per bug found; a bug that needs a design
   change becomes its own task.

## Spec

### Evidence (2026-11-03, draw 1.4.3 at `28cdc53`)

- `draw.log` (T47) covers 2026-11-03 02:03–03:12 UTC: short sessions,
  0 anomalies. Its only `fatal signal` lines (SIGSEGV 03:11:07, SIGABRT
  03:12:00) both land at exactly +2.0 s and match the scripted `kill -SEGV`
  / `kill -ABRT` checks of T48 AC-5, so they are not real crashes. No
  `crash.log`.
- `coredumpctl list draw`: the two T48 checks above, plus 2026-10-14/15
  cores (pre-1.1.0, files gone, already analysed in T48).
- So no real long-session crash has been captured yet. The T47 log stays on
  and keeps collecting (AC-6).

### Soak probe and the bug it found

A headless soak through `Editor` (every creation tool, a text block with a
formula, zoom, select-all, duplicate, delete, undo/redo; net-zero document)
ran 200 000 cycles in release with resident memory flat at 3.6 MiB. The
typeset cache, the glyph atlas (`MAX_PAGES`), the T47 ring and the shell
input queues are all bounded by construction.

**Bug B1 — history is bounded by entries, not bytes.** `History` keeps 500
transactions however large they are, and a selection move stores every
moved shape twice (`Edit::Replace { before, after }`). Moving, duplicating
or deleting a 1 000-stroke selection repeatedly grew resident memory from
4 MiB to a 140 MiB plateau; the ≈ 5 000-stroke scenes used for
frame-time checks ([[T24 Smooth redraws with large documents]]) scale that into
gigabytes. The kernel's OOM killer would then end draw with
`SIGKILL`: no panic, no core dump, no T47 line — a crash "with no obvious
cause after prolonged use". Fix: [[ADR-T49-1 Undo history capped by bytes]].

**Latent bug B2 — revision tracking by stack lengths.**
`Editor::track_document` detects a document change by comparing the undo
and redo stack lengths (plus a "full stack" special case). Once a commit
may drop several old entries, the lengths can stay equal across a real
change and the document layer would not be redrawn. Fix: `History` counts
its changes and the editor compares that count.

### Acceptance criteria

- **AC-1 Size estimate.** `Shape::heap_bytes` counts the heap held by a
  shape (stroke points and widths, text, grid fills and cell texts, shape
  text) by capacity; `Transaction::heap_bytes` is the transaction's own
  edit buffer plus every shape it holds (`Replace` holds two).
  `shape::tests::shape_heap_bytes_counts_points_widths_and_text`,
  `shape_heap_bytes_of_plain_shapes_is_zero`,
  `document::tests::transaction_heap_bytes_sums_edit_buffer_and_shapes`.
- **AC-2 Byte budget.** `History` keeps the bytes held by both stacks
  (`History::bytes`) within `HISTORY_BYTES` (64 MiB) by dropping the
  oldest undo entries, besides the 500-entry limit; the newest entry is
  always kept, even alone over budget. `History::with_limits(entries,
  bytes)` sets both. Bytes follow commit, undo, redo and clear exactly.
  `history::tests::history_over_byte_budget_drops_oldest`,
  `history_keeps_newest_entry_over_budget`,
  `history_bytes_follow_commit_undo_redo_and_clear`,
  `history_bytes_stay_exact_and_within_budget` (proptest).
- **AC-3 Change count.** `History::changes` grows by one on every commit,
  undo and redo that changes the document, and not on an empty commit,
  a failed commit, or an undo/redo with nothing to do.
  `history::tests::history_changes_count_every_document_change`.
- **AC-4 Revision under the budget (B2).** A commit that drops older
  entries while keeping the stack lengths still bumps
  `Editor::document_revision`.
  `editor::tests::editor_commit_dropping_old_entries_bumps_revision`.
- **AC-5 Soak (tests/soak.rs).**
  - A mixed session of 2 000 net-zero cycles never panics, ends with an
    empty document, and keeps `Editor::history_bytes` within
    `HISTORY_BYTES` and the undo depth within `HISTORY_LIMIT`.
    `soak_mixed_session_stays_bounded`.
  - 600 drags of a selected 1 000-stroke document keep
    `Editor::history_bytes` within `HISTORY_BYTES`, and the latest move
    can still be undone. `soak_large_selection_moves_keep_history_within_budget`.
  - Ignored, release, Linux, one thread: 200 000 cycles grow resident
    memory by less than 1 MiB after a 10 000-cycle warm-up.
    `soak_long_session_keeps_resident_memory_flat`
    (`cargo test --release --test soak -- --ignored --nocapture --test-threads=1`).
- **AC-6 Target checklist (manual, owner).** On the target laptop with
  the release build, each step, then `Ctrl+Shift+L` and a look at
  `draw.log`: (1) idle 30 min, type a word; (2) lock the screen
  5 min, unlock, type; (3) suspend 5 min, resume, type and draw;
  (4) unplug and re-plug the keyboard and the tablet, type and draw;
  (5) leave draw open a whole contest day. Pass: no doubled letters, no
  anomaly lines, no crash. Results and any new bug go in *Log*; a bug that
  needs a design change becomes its own task.

## Out of scope

- The Ctrl-C segfault ([[T48 Clean exit on Ctrl-C]]).
- The broad refactor ([[T50 Version 2.0.0 refactor]]), though findings
  feed it.

## Files owned

- `src/core/history.rs`, `src/core/document.rs`, `src/core/shape.rs`,
  `src/core/editor.rs` — B1, B2.
- `tests/soak.rs` (new) — AC-5.
- `docs/features/Undo and redo.md` — the new cap.
- `docs/decisions/ADR-T49-1 Undo history capped by bytes.md` (new).

## Subtasks (one commit each)

- [x] spec — `docs(T49): …`
- [x] tests — `test(T49): …`
- [x] models — `feat(T49): …`
- [x] behaviour — `fix(T49): …`
- [x] quality — `chore(T49): …`
- [x] docs — `docs(T49): …`

## Learning path

None expected.

## Log

- 2026-11-02 — note written from the owner's reports.
- 2026-11-03 — evidence collected (see *Spec*): no real long-session crash
  on record; the only fatal-signal dumps are T48's scripted checks.
- 2026-11-03 — soak probe through `Editor`: 200 000 mixed cycles flat at
  3.6 MiB resident; caches, glyph atlas, T47 ring and input queues bounded.
- 2026-11-03 — **B1** found and fixed: undo history bounded by entries,
  not bytes; repeated moves of a 1 000-stroke selection grew to 140 MiB
  (gigabytes for 5 000 strokes, an OOM kill with no trace).
  [[ADR-T49-1 Undo history capped by bytes]]: 64 MiB budget over both
  stacks, newest entry always kept.
- 2026-11-03 — **B2** fixed with B1: the editor tracked document changes by
  undo/redo stack lengths, which a commit dropping several entries would
  fool; it now follows `History::changes`.
- 2026-11-03 — `tests/soak.rs`: mixed session and large-selection moves in
  the default run (the latter reaches and holds the budget); the ignored
  release run kept resident memory at 3 564 KiB from cycle 10 000 to
  200 000. `scripts/check.sh` green.
- 2026-11-03 — no feature or tutorial note: the task has none (`feature`
  and `tutorial` empty, *Learning path* none); the existing
  [[Undo and redo]] feature note gained the byte budget.
- **Open — AC-6** (owner, on the target): idle, lock, suspend/resume,
  re-plug and a contest-day session with the release build; record the
  results and any new bug here. The doubled-letter reports (T44, T45) have
  no new occurrence in the T47 log yet.
- 2026-11-03 — integrated on `main` (merge `00e6450`), no conflicts;
  AC-6 (owner checks on the target) still open.
