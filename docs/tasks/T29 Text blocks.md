---
id: T29
title: Text blocks
status: ready
wave: 17
branch: task/T29-text
depends_on: [T14]
adrs: []
feature: "[[Text blocks]]"
tutorial: "[[29 Typing on a canvas without losing the shortcuts]]"
tags: [task]
---

# T29 Text blocks

## Goal

A **Text** tool (`T`) places free-standing text on the canvas: click,
type, leave. Text blocks are ordinary shapes — they move, copy, erase,
undo and snap like the others — and stay sharp at every zoom. Typing
must never fire a tool shortcut, and leaving text mode must be one key.
Text inside rectangles and grid cells is
[[T31 Text in rectangles and grid cells]]; formulas are
[[T30 Basic TeX formulas]].

## Design points (settle in the spec step)

- **Edit mode.** While a block is being edited every printable key is
  text; only `Esc` (commit and leave), `Enter` (new line;
  `Ctrl+Enter` commits?), `Backspace`, `Delete`, arrows, `Home`/`End`
  and `Ctrl+Z` act. Clicking outside commits. An empty block is
  discarded. Editing an existing block: Text tool click on it, or
  double-click with Select.
- **Undo.** Creating or editing a block is **one** history entry when
  the edit ends, not one per key; `Ctrl+Z` during an edit undoes typing
  inside the block first (or cancels the edit — pick one).
- **Model.** New `Shape::Text { origin, text, size, style }` in
  `core::shape` (ADR-0004 amended again, like ADR-T16-1). Size in world
  units, picked from the current width (`[` / `]`) so it zooms with the
  canvas.
- **Layout without macroquad.** `core` must stay free of macroquad, but
  bounds, hit-testing, caret placement and snapping need text metrics.
  Proposal: a **monospaced** embedded font, so layout is
  `chars × advance × size` and fully testable in `core`. ADR-T29-1.
- **Font.** macroquad's default font is ASCII-only. Embed one OFL
  monospaced font (`include_bytes!`, rendered through macroquad's own
  TTF loader, no new dependency) covering Latin, Greek and the math
  symbols T30 needs. Binary size grows by the font; ADR-T29-1 records
  the font, its license and the new size, and the current number labels
  may move to it for one consistent look.
- **Typed characters.** `core::input` gains a text event fed from
  miniquad's `char_event` (keyboard layout and dead keys handled by the
  system); `shell::input_map` collects it. IME composition is out of
  scope.

## Spec

To be refined in the spec step; every AC names its tests there.

- **AC-1 — Place and type.** With the Text tool, a click starts a block
  at the pointer with the caret; characters appear as typed; `Esc` or a
  click outside commits; an empty block leaves no shape and no history
  entry.
- **AC-2 — Shortcuts are off while typing.** Every key of the keymap
  produces its character in edit mode (e.g. `P`, `1`, `[`, `Space`,
  `Tab`), checked against the whole `BINDINGS` table; after `Esc` the
  same keys act as shortcuts again.
- **AC-3 — Editing keys.** Caret movement, `Backspace`, `Delete`,
  `Home`/`End`, `Enter` for new lines, on multi-byte characters (no
  panics on UTF-8 boundaries; proptest over random edits).
- **AC-4 — Re-edit.** Clicking an existing block with the Text tool (or
  double-clicking with Select) re-opens it with the caret at the click.
- **AC-5 — Undo.** One history entry per finished edit; undo/redo
  restore the previous text exactly.
- **AC-6 — A shape like the others.** Bounds, hit-test, select, move,
  `Alt`-drag duplicate, copy/cut/paste, eraser, colour, snapping to and
  from its bounds, fit-to-content (`F`). The T13 fuzz harness generates
  text shapes and edit-mode input, and its invariants hold.
- **AC-7 — Rendering.** Sharp at 0.05× – 20× zoom (rasterize at a size
  near the on-screen size, like the label raster sizes), culled off
  screen, drawn in the document layer; the caret blinks only while
  editing (or does not blink, so the idle loop stays asleep — decide in
  the ADR).
- **AC-8 — Budgets.** 500 text blocks of 20 characters each stay within
  [[ADR-T24-5 Redraw budgets]] and [[ADR-T26-2 Gesture frame budget]];
  new perf test in `tests/perf.rs`. Binary stays under 2 MB.
- **AC-9 — Docs.** README tool table and quick start, CHANGELOG entry at
  integration.

## Out of scope

- Text inside rectangles and grid cells ([[T31 Text in rectangles and grid cells]]).
- Formulas ([[T30 Basic TeX formulas]]).
- Pasting system clipboard text, rich text, fonts or font choice, word
  wrap, alignment options, IME input.

## Files owned

- `src/core/shape.rs`, `src/core/input.rs`, `src/core/keymap.rs`,
  `src/core/editor.rs`, `src/core/command.rs`, `src/core/tools/text.rs`
  (new), `src/core/tools/mod.rs`, `src/core/text_layout.rs` (new),
  `src/core/snap.rs`, `src/core/mod.rs`
- `src/shell/input_map.rs`, `src/shell/render.rs`, `src/shell/toolbar.rs`,
  `src/shell/app.rs` (font loading)
- `assets/fonts/*` (new, with its license file)
- `tests/fuzz.rs`, `tests/perf.rs`, `README.md`
- new ADRs `ADR-T29-*`, feature note `docs/features/Text blocks.md`,
  tutorial `docs/tutorials/29 Typing on a canvas without losing the shortcuts.md`

## Subtasks (one commit each)

- [ ] spec — `docs(T29): …`
- [ ] tests — `test(T29): …`
- [ ] models — `feat(T29): …`
- [ ] behaviour — `feat(T29): …`
- [ ] quality — `chore(T29): …`
- [ ] docs — `docs(T29): …`

## Learning path

Step 29 — requires step 5/16 (shape model), step 8 (editor and input),
step 11 (keymap) and step 24 (layered redraw).

## Log

- 2026-10-14 — Created from the owner's proposal for text blocks and
  formulas. Shares `README.md` and `src/shell/app.rs` with T27/T28, so it
  runs in a later wave; it does not need them.
