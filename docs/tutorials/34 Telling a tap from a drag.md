---
title: Telling a tap from a drag
step: 34
requires: ["[[08 An editor as an input-driven state machine]]", "[[28 Choosing a window backend at startup]]"]
feature: "[[Pen tablets]]"
code: ["src/core/tap.rs", "src/core/editor.rs", "src/shell/tablet.rs", "src/shell/app.rs", "src/shell/toolbar.rs"]
tags: [tutorial]
---

# Telling a tap from a drag

> **Learning path step 34.** Requires: step 8 (editor as an input-driven
> state machine), step 28 (choosing a backend at start-up) · Next: step 35
> (pen pressure, planned).

## Why this matters here

Every tool in draw decides "click or drag?" by distance: Select treats a
release within 3 px of the press as a click, Rect ignores drags under 2 px.
That suits a mouse, which stays put while its button clicks. A pen does
not. The tip lands, skids and lifts, and on a small tablet stretched over a
big screen each millimetre of pen travel is 18–36 pixels. The owner's taps
moved 10–40 px. Every tap became a drag: Select moved the shape it meant to
pick, Rect drew a sliver.

## The concept

A **slop** (or *touch slop*, as Android calls it) is a dead zone around the
press point. Until the pointer leaves it, the gesture is still possibly a
tap:

```
press ──► HOLDING ──(move inside slop)──► HOLDING   (move held back)
             │
             ├──(release inside slop)──► IDLE       (release at press point)
             │
             └──(move outside slop)────► DRAGGING   (held move + this move)
                                            │
                                            └──(release)──► IDLE
```

Two choices make it work as a **filter** in front of every tool rather than
a rule inside each one:

1. **Rewrite, don't annotate.** A tap leaves the filter as press + release
   at the *same* point. Every tool already treats that as a click, so no
   tool needs to know about pens.
2. **Hold, then flush.** Moves inside the slop are not dropped for good.
   When the pointer leaves, the last held move is replayed before the
   current one, so a stroke still starts at the press point and loses only
   detail smaller than the slop.

The price is latency: nothing moves for the first 16 px of a stroke. A pen
crosses 16 px in well under a millimetre, so in practice nobody sees it.

## How draw implements it

- `TapFilter::filter` (`src/core/tap.rs`) is a `match` on
  `(state, event)` that returns at most two events (`[Option<InputEvent>;
  2]`): nothing while holding, one event normally, two when it flushes. A
  fixed-size array means no allocation per event; `Editor::handle`
  iterates it with `.into_iter().flatten()`.
- Only the **last** held move is kept. Keeping all of them would need a
  buffer and would add nothing the stroke smoother does not throw away.
- Non-finite positions pass through without touching the state. The
  editor drops them anyway, and they must not turn a hold into a drag.
- The filter is off with a slop of 0 — `Editor::new()` — so every existing
  test and the mouse-only path are unchanged. The fuzz harness runs its
  random sessions a second time through an editor with the slop on.

**Detection** (`src/shell/tablet.rs`). miniquad does not say which device
an event came from, so draw asks the kernel once at start-up.
`/proc/bus/input/devices` lists every input device with bitmaps of its
keys and axes, written as hex words, most significant first:

```
N: Name="HUION 420 Pen"
B: KEY=1c01 0 0 0 0 0
B: ABS=1000003
```

The key bitmap has six 64-bit words. Bit 320 (`BTN_TOOL_PEN`, 0x140) is
bit 0 of the sixth word from the right: the last `1` of `1c01`.
The other bits of `1c01` are `BTN_TOUCH`, `BTN_STYLUS` and `BTN_STYLUS2`.
`ABS=1000003` has `ABS_X`, `ABS_Y` and `ABS_PRESSURE`. `has_bit` reverses
the words and picks one. Its word size is `usize::BITS`, because the kernel
prints `long`s.

## Try it

1. Change `PEN_TAP_SLOP_PX` to 4 and run
   `cargo test pen_tap_with_select_selects_instead_of_marquee`. The 12 px
   slip in the test now leaves the slop and Select drags the rectangle.
2. Write a test feeding `TapFilter` a press, a move 20 px away, a move back
   to 2 px and a release. Check that the release is **not** moved to the
   press point: once a drag, always a drag.
3. Run `grep -A6 Pen /proc/bus/input/devices` with a tablet plugged in and
   decode the `KEY=` line by hand.

## Further reading

- Android `ViewConfiguration.getScaledTouchSlop` — the same idea for
  touchscreens.
- `linux/input-event-codes.h` — the key and axis numbers.
- Linux `Documentation/input/event-codes.rst` — what `BTN_TOOL_PEN` and
  `ABS_PRESSURE` mean.
