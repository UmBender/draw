---
title: Keymaps and gesture macros
step: 11
requires: ["[[08 An editor as an input-driven state machine]]"]
feature: "[[Shortcuts and macros]]"
code: ["src/core/keymap.rs", "src/core/command.rs", "src/core/input.rs"]
tags: [tutorial]
---

# Keymaps and gesture macros

> **Learning path step 11.** Requires: [[08 An editor as an input-driven state machine]]

## Why this matters here

draw is keyboard-first: during a contest you press `R`, drag, press `A`,
drag, without hunting for buttons. The keymap is where a physical chord
becomes an action. If it lived in a big `match` scattered across the shell,
the toolbar tooltips, the docs and the actual behaviour would drift apart.

## The concept

**Bindings are data.** A keymap is a function from a *chord* (key plus held
modifiers) to an optional command:

```
(Key::C, {})       ──▶ SetTool(Ellipse)
(Key::C, {ctrl})   ──▶ Copy
(Key::C, {shift})  ──▶ None
```

Writing it as a table instead of code gives three things at once:

1. **One source of truth** — the same rows can be looked up, listed in a
   toolbar, and checked against the documentation.
2. **Testable invariants** — "no chord bound twice", "every row has a
   description" are one-line loops.
3. **Total behaviour** — there are only 43 keys × 8 modifier sets = 344
   chords, so a test can try *all* of them.

The remaining design choice is how modifiers match. draw uses **exact
matching**: the held set must equal the chord's set. Then `Ctrl+Z` and
`Ctrl+Shift+Z` can never shadow each other, whatever order the rows are in
([[ADR-T11-1 Exact modifier matching]]).

*Gesture macros* (`Shift` to constrain, right-drag to erase, `Alt`-drag to
duplicate) are a different kind of shortcut: they modify a drag rather than
fire once. They live in the editor and tools, which see modifiers on every
pointer event, not in the keymap.

## How draw implements it

- `KeyChord { key, mods }` in `src/core/keymap.rs`, with `const`
  constructors `bare`, `ctrl` and `ctrl_shift` so the table can be built at
  compile time.
- `BINDINGS: &[(KeyChord, Command, &str)]` lists every row of [[Keymap]].
  Colours are produced by a private `const fn color(digit)` that calls
  `ColorId::from_key_digit`; an invalid digit would be a *compile* error,
  because the `panic!` runs during constant evaluation.
- `resolve` builds a `KeyChord` and does a linear `find` over `BINDINGS`.
  ~35 rows is far cheaper than a frame; a `HashMap` would add start-up work
  and nothing measurable.
- The tests keep their own copy of the documented table (`documented()`)
  instead of reading `BINDINGS`, so a typo in the table cannot also hide in
  the test. `resolve_matches_documented_table_for_all_chords` compares the
  two over every chord; the proptest `unbound_keys_resolve_to_none` samples
  the same space as a fuzz check.
- `Space` is not in the table: `Editor::handle` intercepts it as the pan
  modifier before asking the keymap.

## Try it

1. Bind `G` to `Command::ToggleToolbar` by adding a row to `BINDINGS`.
   Which test fails, and why is that the right outcome (hint: the docs were
   not updated)?
2. Change `resolve` to ignore `shift` for bare keys. Run the tests: which
   assertion in `modifiers_disambiguate` catches it?

## Further reading

- [[Keymap]] — the documented bindings.
- [[08 An editor as an input-driven state machine]] — where `resolve` is
  called.
