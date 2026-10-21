---
id: T38
title: Left-hand tool keys
status: done
wave: 24
branch: task/T38-left-hand-keys
depends_on: [T11, T12]
adrs: ["[[ADR-T38-1 Left-hand keys for pen, line and smart snap]]"]
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

[[ADR-T38-1 Left-hand keys for pen, line and smart snap]] records the new
keys and why the old ones are freed instead of kept as aliases.

Acceptance criteria and the tests that prove them:

- **AC-1 — New keys.** Bare `D` selects the pen, bare `W` the line tool,
  bare `X` toggles smart snap; `Ctrl+D` (duplicate) and `Ctrl+X` (cut)
  are unchanged; `Shift`/`Ctrl`/`Alt` + `D`, `W`, `X` resolve to nothing
  else (exact modifiers, ADR-T11-1).
  Tests (`src/core/keymap.rs`): `left_hand_keys_select_pen_line_and_smart_snap`,
  `left_hand_keys_keep_their_ctrl_chords`.
- **AC-2 — Old keys freed.** `P`, `L` and `M` resolve to nothing with any
  modifiers.
  Test (`src/core/keymap.rs`): `p_l_m_are_unbound`.
- **AC-3 — Documented table.** The independent table in the keymap tests
  (mirror of `docs/architecture/Keymap.md`) lists the new keys; every
  chord resolves as documented and chords stay unique.
  Tests (`src/core/keymap.rs`, updated): `every_documented_binding`,
  `helper_bindings`, `shift_chords_are_exact`,
  `resolve_matches_documented_table_for_all_chords`,
  `bindings_have_unique_chords`.
- **AC-4 — Text edit mode.** While a block is open `d`, `w`, `x` type their
  letters; after `Esc` they are shortcuts again.
  Tests (`src/core/editor.rs`): `text_left_hand_keys_type_letters`;
  `text_after_escape_keys_act_as_shortcuts_again` (now presses `D`).
- **AC-5 — Toolbar labels.** Each tool button shows the key its tool is
  bound to in `BINDINGS`, and the smart snap toggle shows `X`.
  Tests (`src/shell/toolbar.rs`): `tool_labels_match_keymap`,
  `smart_snap_label_matches_keymap`.
- **AC-6 — Docs.** README *Quick start*, *Keyboard reference* and *Recipes*,
  `docs/architecture/Keymap.md`, the cheat sheet and the feature notes
  that name `P`, `L` or `M` show `D`, `W`, `X`.

## Out of scope

- User-configurable keymaps.
- Layout detection (keys are matched as the shell reports them).

## Files owned

- `src/core/keymap.rs`, `src/shell/toolbar.rs`
- `src/core/editor.rs` — tests only (one existing test presses `P`)
- `README.md` — sections *Quick start*, *Keyboard reference* and *Recipes* only
- `docs/architecture/Keymap.md`, `docs/features/Shortcuts and macros.md`,
  `docs/features/Contest cheat sheet.md`, `docs/features/Drawing tools.md`,
  `docs/features/Shapes.md`, `docs/features/Anti-tremor strokes.md`,
  `docs/features/Text blocks.md`, `docs/features/Snapping.md`
- tutorial `docs/tutorials/38 Laying out shortcuts for one hand.md`, new
  `ADR-T38-*`

## Subtasks (one commit each)

- [x] spec — `docs(T38): specify left-hand tool keys`
- [x] tests — `test(T38): add failing tests for left-hand bindings`
- [x] models — `feat(T38): add the smart snap toggle label constant`
- [x] behaviour — `feat(T38): bind tools to left-hand keys`
- [x] quality — `chore(T38): pass clippy, rustfmt and rustdoc`
- [x] docs — `docs(T38): update shortcuts, cheat sheet and add tutorial`

## Learning path

Step 38 — requires step 11 (keymaps) and step 12 (toolbar).

## Log

- 2026-10-21 — Spec step: `src/core/editor.rs` (one test pressed `P`),
  `docs/features/Text blocks.md` and `docs/features/Snapping.md` (name
  `P` / `M`) added to the files owned. Tutorials of earlier steps keep the
  keys of their time.
- 2026-10-21 — Owner chose pen `D`, line `W`, smart snap `X`; the
  optional hand and numbering moves were not taken.
- 2026-10-21 — Created from the owner's request: `P` and `L` are on the
  other side of the keyboard from the hand left free while drawing with a
  mouse or pen; propose left-hand replacements. Runs in wave 24 in
  parallel with T36 and T37.
- 2026-10-21 — Tests step: red for the right reason (`SMART_SNAP_LABEL`
  missing). Models step: only the toolbar's smart snap label became a
  named constant so `smart_snap_label_matches_keymap` can see it.
- 2026-10-21 — Behaviour: three rows of `BINDINGS` and the toolbar labels
  changed; all 958 lib tests and the fuzz and perf suites green. Quality:
  clippy asked for let-else in a test helper. `scripts/check.sh` green.
- 2026-10-21 — Docs: README (*Quick start*, *Keyboard reference*,
  *Recipes*), cheat sheet, `Keymap.md` and the feature notes show `D`, `W`,
  `X`. For the integrator: `CHANGELOG.md` *Changed* entry (pen `P`→`D`,
  line `L`→`W`, smart snap `M`→`X`) and step 35's "Next" link in the
  Learning Path. The in-app check on the target is **pending the owner**.
  Status → review.
- 2026-10-21 — Integrated into `main`. Status → done.
