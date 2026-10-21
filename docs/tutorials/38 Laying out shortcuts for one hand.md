---
title: Laying out shortcuts for one hand
step: 38
requires: ["[[11 Keymaps and gesture macros]]", "[[12 The app loop, idle redraw and UI]]"]
feature: "[[Shortcuts and macros]]"
code: ["src/core/keymap.rs", "src/shell/toolbar.rs", "src/core/editor.rs"]
tags: [tutorial]
---

# Laying out shortcuts for one hand

> **Learning path step 38.** Requires: step 11 (keymaps and gesture
> macros), step 12 (the app loop and toolbar) · Next: —

## Why this matters here

While drawing, one hand holds the mouse or the pen and only the other
reaches the keyboard. A shortcut under the drawing hand costs a trip: put
the pen down, press `P`, pick the pen up again. Mnemonics (`P` for pen,
`L` for line) are easy to learn but say nothing about where the key is.
For a tool switched dozens of times per problem, position wins.

## The concept

**1. Frequency decides the key.** Rank the commands by how often they are
used *while the other hand is busy*, then hand out the keys the free hand
reaches without moving, best first: home row (`A S D F`), the row above
(`Q W E R T`), the row below (`Z X C V B`). The pen is the most used tool
and gets `D` (home row, middle finger); line gets `W`; smart snap, toggled
mid-drawing, gets `X`.

**2. Exact modifiers keep bare keys free.** `Ctrl+D` duplicates and
`Ctrl+X` cuts. Because a chord matches only its exact modifiers
([[ADR-T11-1 Exact modifier matching]]), bare `D` and `X` were free all
along: no shortcut had to move to make room.

**3. One source of truth, checked by tests.** A key printed in two places
(the keymap table and a toolbar label) will drift the first time one of
them changes. Instead of deriving the label at run time, a test compares
them: cheap at run time, loud when they disagree.

## How draw implements it

`core::keymap::BINDINGS` is the only table of bindings
([[ADR-T38-1 Left-hand keys for pen, line and smart snap]]):

```rust
(KeyChord::bare(Key::D), Command::SetTool(Tool::Pen), "Pen"),
(KeyChord::bare(Key::W), Command::SetTool(Tool::Line), "Line"),
// …
(KeyChord::bare(Key::X), Command::ToggleSmartSnap, "Smart snap on / off"),
```

The keymap tests keep an independent copy of the documented table
(`documented()`), so a binding changes only when both the code and the
docs' mirror change; `p_l_m_are_unbound` pins that the old keys are free.

The toolbar spells each tool's key in `tool_label` and the smart snap
toggle in `SMART_SNAP_LABEL`. `tool_labels_match_keymap` looks up the bare
chord bound to each command in `BINDINGS` and compares its `Key` name
(`format!("{:?}", key)` gives `"D"`) with the label.

While a text block is open no binding fires (ADR-T29-2), so `d`, `w`, `x`
type letters; `text_left_hand_keys_type_letters` sends the key press and
the character exactly as the shell does and checks both.

## Try it

1. Change the pen row back to `Key::P` and run `cargo test keymap`: the
   documented table and `p_l_m_are_unbound` fail. Then run
   `cargo test toolbar`: `tool_labels_match_keymap` fails too — the label
   still says `D`.
2. Bind `Key::D` with `Modifiers::NONE` twice and run the tests:
   `bindings_have_unique_chords` catches it.
3. Draw a graph with only the left hand on the keyboard: `N`, `C` nodes,
   `A` edges, `X` so they meet the outlines, `D` for notes by hand.

## Further reading

- [[ADR-T38-1 Left-hand keys for pen, line and smart snap]],
  [[ADR-T11-1 Exact modifier matching]]
- [[Keymap]], [[Contest cheat sheet]]
