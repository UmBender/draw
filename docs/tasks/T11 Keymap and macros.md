---
id: T11
title: Keymap and macros
status: todo
wave: 5
branch: task/T11-keymap
depends_on: [T08]
adrs: []
feature: "[[Shortcuts and macros]]"
tutorial: "[[11 Keymaps and gesture macros]]"
tags: [task]
---

# T11 Keymap and macros

## Goal

Every binding in [[Keymap]] resolves to the right `Command`.

## Spec

- **AC-1** — `keymap::resolve(key: Key, mods: Modifiers) -> Option<Command>` implements the *Tools*,
  *Style*, *Edit* and *View* tables of [[Keymap]] exactly. *Test:* table-driven
  `keymap::tests::every_documented_binding` (one row per documented binding).
- **AC-2** — Bindings with modifiers don't fire the bare-key command (e.g. `Ctrl+C`
  ≠ `C` ellipse; `Ctrl+Backspace` ≠ `Backspace`). *Test:* `modifiers_disambiguate`.
- **AC-3** — Unbound keys return `None`. *Test:* proptest `unbound_keys_resolve_to_none`.
- **AC-4** — The table is a `const` slice so the toolbar and [[Keymap]] docs can list it
  (`BINDINGS: &[(KeyChord, Command, &str)]`). *Test:* `bindings_have_descriptions`.

## Out of scope

User-configurable keymaps. Gesture macros (`Shift` constrain, right-drag, `Alt`-drag)
are implemented by T08–T10; this task only documents them in the feature note.

## Files owned

`src/core/keymap.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 11 — requires step 8.

## Log
