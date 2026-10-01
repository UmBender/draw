---
id: T11
title: Keymap and macros
status: in-progress
wave: 5
branch: task/T11-keymap
depends_on: [T08]
adrs: [ADR-T11-1]
feature: "[[Shortcuts and macros]]"
tutorial: "[[11 Keymaps and gesture macros]]"
tags: [task]
---

# T11 Keymap and macros

## Goal

Every binding in [[Keymap]] resolves to the right `Command`.

## Spec

A chord is a `KeyChord { key, mods }`. A binding fires only when the pressed
modifiers equal the chord's modifiers **exactly**
([[ADR-T11-1 Exact modifier matching]]). `Space` is not in the table: the
editor consumes it as the pan modifier before the keymap is asked (T08).

- **AC-1** — `keymap::resolve(key: Key, mods: Modifiers) -> Option<Command>` implements the *Tools*,
  *Style*, *Edit* and *View* tables of [[Keymap]] exactly. *Test:* table-driven
  `keymap::tests::every_documented_binding` (one row per documented binding,
  written independently of `BINDINGS`).
- **AC-2** — Bindings with modifiers don't fire the bare-key command (e.g. `Ctrl+C`
  ≠ `C` ellipse; `Ctrl+Backspace` ≠ `Backspace`), and extra modifiers on a bare
  key resolve to nothing (`Shift+R`, `Alt+1`). *Test:* `modifiers_disambiguate`.
- **AC-3** — Unbound keys return `None`: for every key and modifier
  combination not in the documented table, `resolve` is `None`. *Test:* proptest
  `unbound_keys_resolve_to_none`, plus exhaustive
  `resolve_matches_documented_table_for_all_chords`.
- **AC-4** — The table is a `const` slice so the toolbar and [[Keymap]] docs can list it
  (`BINDINGS: &[(KeyChord, Command, &str)]`); every description is non-empty and
  no chord appears twice. `resolve` is defined as a lookup in `BINDINGS`.
  *Tests:* `bindings_have_descriptions`, `bindings_have_unique_chords`,
  `resolve_agrees_with_bindings`.

## Out of scope

User-configurable keymaps. Gesture macros (`Shift` constrain, right-drag, `Alt`-drag)
are implemented by T08–T10; this task only documents them in the feature note.

## Files owned

`src/core/keymap.rs`

## Subtasks (one commit each)

- [x] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 11 — requires step 8.

## Log
