---
id: T37
title: Remember the text size
status: done
wave: 24
branch: task/T37-text-size-memory
depends_on: [T29, T31]
adrs: [ADR-T37-1]
feature: "[[Text blocks]]"
tutorial: "[[37 Settings that follow what you last did]]"
tags: [task]
---

# T37 Remember the text size

## Goal

A new text block starts at the size of the last block the user typed,
so after growing one block with `Ctrl+]` the next block is already that
big. Today every new block starts at `text_size_px(stroke width)`
(`src/core/tools/text.rs:357`) and has to be grown again each time.

## Known facts (2026-10-21)

- `TEXT_BASE_PX + TEXT_PX_PER_WIDTH * width_px` sets a new block's
  on-screen size; the world size is that divided by the zoom
  (`camera.world_len`).
- `Ctrl+[` / `Ctrl+]` step an open block through the text ladder at the
  current zoom (ADR-T29-3); that ADR rejected also changing the stroke
  width, noting "the next block still follows `[` / `]`" — this task
  changes that consequence, so it needs a new ADR amending ADR-T29-3.
- Text in a shape or cell (T31) is fitted into the host frame; its
  `size` is the largest size, not the shown one.

## Proposal (settle in the spec step)

- **What is remembered.** The editor keeps `text_px: Option<f32>`, the
  **on-screen** size (px at the zoom it was committed) of the last text
  block committed with a size change from `Ctrl+[` / `Ctrl+]`. Remembering
  on-screen pixels, not world units, keeps a new block readable after a
  zoom, like the stroke-width rule does.
- **When it is used.** A new free block, and a new text in an empty shape
  slot, start at the remembered size; `None` falls back to
  `text_size_px(width)` as now. Opening an existing block keeps its own
  size (unchanged).
- **When it is forgotten.** `[` / `]` outside edit mode (an explicit
  width change) clear it, so the stroke width still sets the size when
  the user asks for it. Undo of the resized block does not clear it.
- **Lifetime.** The session only; nothing is saved to disk.
- **Open question for the owner.** Should merely *typing* a block at the
  remembered size count (always "last block's size"), or only an explicit
  `Ctrl+[` / `Ctrl+]`? The proposal is the latter: a block opened in a
  small cell would otherwise shrink the next free block.

## Spec

Decided in [[ADR-T37-1 Remember the last text size]]; the owner's open
question is settled as proposed: only an explicit `Ctrl+[` / `Ctrl+]`
counts. The memory is `text::State::size_px` (on-screen px), set when a
resized edit is finished, read by `text::State::new_size_px(width)`.

- **AC-1 — Next block at the remembered size.** After a block is grown
  with `Ctrl+]` and finished, the next new block starts at that on-screen
  size; a later block typed without resizing does not change it; an edit
  thrown away with `Ctrl+Z` is not remembered; undoing the resized block
  keeps it.
  Tests: `text_size_memory_next_block_starts_at_last_resized_size`,
  `text_size_memory_unresized_block_keeps_it`,
  `text_size_memory_cancelled_edit_is_not_remembered`,
  `text_size_memory_survives_undo` (editor);
  `edit_step_size_marks_edit_resized`,
  `state_new_size_px_prefers_remembered_size` (text).
- **AC-2 — On-screen size across zoom.** Zooming between the two blocks
  keeps the on-screen size: the second block's world size is the
  remembered px divided by the new zoom.
  Test: `text_size_memory_keeps_on_screen_size_across_zoom`.
- **AC-3 — `[` / `]` reset.** `WidthUp` / `WidthDown` outside edit mode
  clear the memory; the next block follows `text_size_px(width)`.
  Tests: `text_size_memory_width_up_resets_to_width_rule`,
  `text_size_memory_width_key_at_ladder_end_still_resets`.
- **AC-4 — Existing text unchanged; empty slots use it.** Re-opening a
  block keeps its own size; an empty rectangle slot (Text tool or Select
  click) starts at the remembered size; existing shape text keeps its
  size.
  Tests: `text_size_memory_reopened_block_keeps_its_size`,
  `text_size_memory_empty_shape_slot_starts_at_it`,
  `text_size_memory_select_click_in_empty_slot_starts_at_it`,
  `text_size_memory_existing_shape_text_keeps_its_size`.
- **AC-5 — Invariants and docs.** `step_size` at random zooms keeps the
  size finite and positive and marks the edit resized exactly when a step
  succeeded (`edit_random_steps_keep_size_finite_and_flag_resized`);
  the existing fuzz sessions (`tests/fuzz.rs`) pass unchanged. README
  *Typing in a text block* describes the rule. The toolbar is not
  touched: it shows no text size and `src/shell/` is not owned.

## Out of scope

- Persisting settings between runs.
- A size picker in the toolbar.

## Files owned

- `src/core/tools/text.rs`, `src/core/editor.rs`
- `README.md` — section *Typing in a text block* only
- feature note `docs/features/Text blocks.md`, tutorial
  `docs/tutorials/37 Settings that follow what you last did.md`, new
  `ADR-T37-*`

## Subtasks (one commit each)

- [x] spec — `docs(T37): specify remembered text size`
- [x] tests — `test(T37): add failing tests for the remembered text size`
- [x] models — `feat(T37): keep the last text size in the editor`
- [x] behaviour — `feat(T37): start new blocks at the remembered size`
- [x] quality — `chore(T37): pass clippy, rustfmt and rustdoc`
- [x] docs — `docs(T37): update feature note and add tutorial`

## Learning path

Step 37 — requires step 29 (text blocks) and step 31 (text in shapes).

## Log

- 2026-10-21 — Created from the owner's request: "I want the font size
  be remembered from the last block, so I do not need to make the font
  bigger every time." Runs in wave 24 in parallel with T36 and T38.
- 2026-10-21 — Spec: the open question settled as proposed (only an
  explicit `Ctrl+[` / `Ctrl+]` counts); ADR-T37-1 amends ADR-T29-3. The
  memory lives in `text::State::size_px` rather than a new editor field
  so the Text tool reads it directly.
- 2026-10-21 — Implemented. Quality step produced no changes
  (`scripts/check.sh` green after the behaviour commit), so no `chore`
  commit. The toolbar was not changed: it shows no text size and
  `src/shell/` is not owned; README and the feature note describe the
  rule.
- 2026-10-21 — Integrated into `main`. Status → done.
