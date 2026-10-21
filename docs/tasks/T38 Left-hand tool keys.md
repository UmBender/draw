---
id: T38
title: Left-hand tool keys
status: todo
wave: 24
branch: task/T38-left-hand-keys
depends_on: [T11, T12]
adrs: []
feature: "[[Shortcuts and macros]]"
tutorial: "[[38 Laying out shortcuts for one hand]]"
tags: [task]
---

# T38 Left-hand tool keys

## Goal

With the right hand on the mouse or pen, every frequent tool is reachable
by the left hand without leaving the home position. Today Pen (`P`) and
Line (`L`) sit under the right hand, so switching to them means putting the
pen down or crossing the left hand over the keyboard.

## Known facts (2026-10-21)

- `core::keymap::BINDINGS` is one `const` table; several chords may map to
  the same command (`Ctrl+Shift+Z` and `Ctrl+Y` both redo), and modifiers
  match exactly (ADR-T11-1).
- Left-hand letters (QWERTY / ABNT2) already bound bare: `A` arrow, `R`
  rectangle, `C` ellipse, `E` eraser, `B` bucket, `V` select, `G` grid,
  `T` text, `S` anti-tremor, `F` fit view; `1`–`6` colours, `Tab`, `Esc`,
  `Space` (pan).
- Left-hand letters **free** bare: `Q`, `W`, `D`, `X`, `Z` (`Ctrl+D`,
  `Ctrl+X`, `Ctrl+Z` stay as they are — a bare key does not clash with a
  `Ctrl` chord).
- Still on the right hand: `P` pen, `L` line, `H` hand, `M` smart snap,
  `N` / `Shift+N` numbering, `[` / `]` width, `0` reset view, arrows.
- The toolbar prints each tool's key (`src/shell/toolbar.rs:335`).

## Proposal (owner decides in the spec step)

Recommended mapping, by how often the tool is used while drawing:

| Command | Now | Proposed | Why |
|---------|-----|----------|-----|
| Pen | `P` | **`D`** (draw) | Home row, middle finger: the most used tool on the strongest key |
| Line | `L` | **`W`** | One row up from `S`, next to `E`/`R`; lines alternate with arrows (`A`) and rectangles (`R`) |
| Hand | `H` | `Space` + drag already pans; **`Q`** as an optional extra | Rarely needed as a sticky tool |
| Smart snap on/off | `M` | **`X`** (optional) | Bottom row, left; toggled mid-drawing |
| Numbering on/off | `N` | **`Z`** (optional; `Shift+Z` restarts at 1) | Bottom-left; used while placing graph nodes with `C` |

- **Keep the old keys as aliases.** `P`, `L`, `H`, `M`, `N` keep working
  (extra rows in `BINDINGS`), so muscle memory and the docs of older
  releases stay valid; the toolbar shows the new left-hand key.
- **`[` / `]` (stroke width)** stay; a left-hand pair (e.g. `Shift+1` /
  `Shift+2`) is possible but layouts disagree on shifted digits — leave it
  out unless the owner wants it.

Questions for the owner:

1. Pen `D`, Line `W` — accept, or prefer another pair (e.g. Pen `Q`,
   Line `W`; Pen `D`, Line `X`)?
2. Take the optional `Q` / `X` / `Z` moves for hand, smart snap and
   numbering, or only pen and line?
3. Keep `P` / `L` (and the others) as aliases, or remove them so each
   command has one key?

## Decision (owner, 2026-10-21)

- Pen → `D`, Line → `W`, Smart snap on/off → `X`.
- Hand (`H`) and numbering (`N` / `Shift+N`) stay where they are.
- The old keys `P`, `L`, `M` are replaced, not kept as aliases ("change
  the pen, line and smart snap for the proposed letters").

## Spec

To be written in the spec step. Expected:

- **AC-1** — The new bare keys resolve to their commands; the aliases
  (if kept) still resolve; no chord appears twice in `BINDINGS`
  (existing uniqueness test).
- **AC-2** — Text edit mode is unchanged: the new keys type their
  letters while a block is open.
- **AC-3** — The toolbar labels show the new keys.
- **AC-4** — README *Keyboard reference*, *Recipes*, the cheat sheet,
  `docs/architecture/Keymap.md` and the feature notes list the new keys.

## Out of scope

- User-configurable keymaps.
- Layout detection (keys are matched as the shell reports them).

## Files owned

- `src/core/keymap.rs`, `src/shell/toolbar.rs`
- `README.md` — sections *Keyboard reference* and *Recipes* only
- `docs/architecture/Keymap.md`, `docs/features/Shortcuts and macros.md`,
  `docs/features/Contest cheat sheet.md`, `docs/features/Drawing tools.md`,
  `docs/features/Shapes.md`, `docs/features/Anti-tremor strokes.md`
- tutorial `docs/tutorials/38 Laying out shortcuts for one hand.md`, new
  `ADR-T38-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T38): specify left-hand tool keys`
- [ ] tests — `test(T38): add failing tests for left-hand bindings`
- [ ] models — skipped if no new types are needed (record in the log)
- [ ] behaviour — `feat(T38): bind tools to left-hand keys`
- [ ] quality — `chore(T38): pass clippy, rustfmt and rustdoc`
- [ ] docs — `docs(T38): update shortcuts, cheat sheet and add tutorial`

## Learning path

Step 38 — requires step 11 (keymaps) and step 12 (toolbar).

## Log

- 2026-10-21 — Owner chose pen `D`, line `W`, smart snap `X`; the
  optional hand and numbering moves were not taken.
- 2026-10-21 — Created from the owner's request: `P` and `L` are on the
  other side of the keyboard from the hand left free while drawing with a
  mouse or pen; propose left-hand replacements. Runs in wave 24 in
  parallel with T36 and T37.
