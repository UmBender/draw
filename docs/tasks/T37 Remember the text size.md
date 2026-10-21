---
id: T37
title: Remember the text size
status: todo
wave: 24
branch: task/T37-text-size-memory
depends_on: [T29, T31]
adrs: []
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

To be written in the spec step. Expected:

- **AC-1** — After a block is grown with `Ctrl+]` and committed, the next
  new block starts at that on-screen size.
- **AC-2** — Zooming between the two blocks keeps the on-screen size.
- **AC-3** — `[` / `]` outside edit mode reset to the width rule.
- **AC-4** — Opening an existing block, and fitted text in shapes, keep
  their current behaviour.
- **AC-5** — Fuzz invariants hold; the toolbar/README describe it.

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

- [ ] spec — `docs(T37): specify remembered text size`
- [ ] tests — `test(T37): add failing tests for the remembered text size`
- [ ] models — `feat(T37): keep the last text size in the editor`
- [ ] behaviour — `feat(T37): start new blocks at the remembered size`
- [ ] quality — `chore(T37): pass clippy, rustfmt and rustdoc`
- [ ] docs — `docs(T37): update feature note and add tutorial`

## Learning path

Step 37 — requires step 29 (text blocks) and step 31 (text in shapes).

## Log

- 2026-10-21 — Created from the owner's request: "I want the font size
  be remembered from the last block, so I do not need to make the font
  bigger every time." Runs in wave 24 in parallel with T36 and T38.
