---
title: Settings that follow what you last did
step: 37
requires: ["[[29 Typing on a canvas without losing the shortcuts]]", "[[31 Prototyping an interaction behind a switch]]"]
feature: "[[Text blocks]]"
code: ["src/core/tools/text.rs", "src/core/editor.rs"]
tags: [tutorial]
---

# Settings that follow what you last did

> **Learning path step 37.** Requires: step 29 (text blocks), step 31
> (text in shapes) · Next: —

## Why this matters here

In a contest you write many small notes of the same size. If every new
block starts small, you press `Ctrl+]` three times per note. A tool that
remembers what you just did removes that repetition — but a tool that
remembers *too much* surprises you: the next note suddenly comes out tiny
because the last thing you typed was inside a small grid cell.

## The concept

An **implicit setting** is one the user never sets directly; the program
infers it from an action. Three questions decide whether it helps:

1. **What counts as the signal?** Only a deliberate action. Here that is
   `Ctrl+[` / `Ctrl+]`; merely typing at some size is not a request to
   keep that size.
2. **In which units is it stored?** In the units the user perceives.
   Text is judged on screen, so the memory is in screen pixels, not world
   units — after zooming out a new note is still readable.
3. **How is it reset?** By the explicit control for the same thing. The
   stroke-width keys `[` / `]` used to set the text size; pressing them
   says "use the width rule again", so they clear the memory.

```text
 [ / ]  ──────────────► size_px = None ─┐
                                        ├─► new block: size_px ?? text_size_px(width)
 Ctrl+] … Esc (resized) ► size_px = px ─┘
 Ctrl+Z while typing   ─► unchanged
```

## How draw implements it

- `TextEdit::step_size` (`src/core/tools/text.rs`) sets a private
  `resized` flag when it changes the size; `TextEdit::resized` reads it.
- `text::State::size_px` is the memory and `State::new_size_px(width)`
  the rule: the memory, else `text_size_px(width)`. `on_pointer` converts
  it to world units with `camera.world_len` at the current zoom.
- `Editor::finish_text` (`src/core/editor.rs`) stores
  `edit.size * zoom` when the finished edit was resized. `cancel_text`
  never does, so `Ctrl+Z` while typing forgets the resize.
- `Editor::set_width`, behind `WidthUp` / `WidthDown`, clears the memory
  even when the width is already at the end of the ladder.
- `Editor::reopen_clicked_text` uses the same rule for an empty shape
  slot opened with Select; existing blocks and shape text keep their own
  size.

The decision and the rejected alternatives are in
[[ADR-T37-1 Remember the last text size]].

## Try it

1. Run `cargo test text_size_memory` — eleven editor tests describe the
   rule end to end.
2. Change `finish_text` to remember every finished block, not only
   resized ones. Which test fails, and why is that the case the ADR
   wanted to avoid?
3. Store `edit.size` (world units) instead of `edit.size * zoom`; watch
   `text_size_memory_keeps_on_screen_size_across_zoom` fail.

## Further reading

- [[ADR-0013 World-space widths and zoom limits]] — the same screen-pixel
  rule for stroke widths.
- Jef Raskin, *The Humane Interface*, on modes and habituation.
