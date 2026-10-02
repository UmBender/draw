---
title: Layered redraws and measuring a frame
step: 24
requires: ["[[12 The app loop, idle redraw and UI]]", "[[14 Profiling and shipping a release build]]", "[[15 Anti-aliasing and multisampling]]"]
feature:
code: ["src/shell/app.rs", "src/core/editor.rs"]
tags: [tutorial]
---

# Layered redraws and measuring a frame

> **Learning path step 24.** Requires: steps 12, 14 and 15 · Next: —

## Why this matters here

After `Ctrl+A`, `Ctrl+D` ×9 a page holds about 5 000 shapes, and drawing
one circle started to feel sluggish. Before this step, every pointer move
re-tessellated and re-uploaded *all* shapes, although only the circle preview
had changed. So a frame cost O(document), when it should cost O(change).

## The concept

**Measure before optimising.** A frame timer is the cheapest profiler: wrap
the work in `Instant::now()` / `elapsed()` and print the result with the
input size. It only sees CPU time. The GPU runs asynchronously, so fill rate
and the MSAA resolve show up in `perf record` and in overall CPU usage, not
in the timer.

**Layers.** Split what is drawn by how often it changes:

```
 document layer (cached)        frame (cached)               window
┌──────────────────────┐  blit ┌──────────────────────┐ blit ┌────────┐
│ bg, dot grid, shapes │ ────▶ │ + preview, guides,   │ ───▶ │        │
│ (minus hidden ones)  │       │   selection, marquee,│      │        │
└──────────────────────┘       │   toolbar            │      └────────┘
   changes rarely              └──────────────────────┘
                                  changes on every move
```

**Cache keys.** A cache is valid only as long as its *inputs* are unchanged.
Write the inputs down as a value (here `LayerKey`) and compare. Anything you
forget in the key turns into a stale-picture bug, so the key lists *every*
input of `draw_document`: document, camera, size, hidden shapes, and whether
the dot grid is on.

**Revisions instead of diffs.** Comparing two 5 000-shape documents on
every move would cost what we are trying to save. A counter that grows on
every change is O(1) to compare. It does not have to be exact: an extra bump
only costs one unneeded full redraw, while a missed bump leaves a stale
picture. So when in doubt, bump.

## How draw implements it

- `Editor::document_revision` (`src/core/editor.rs`). Every document change
  goes through `History`. `Editor::track_document` wraps the calls that can
  commit (pointer down/up, cut/paste/duplicate/delete, undo/redo/clear) and
  compares the undo/redo lengths before and after. A commit on a full undo
  stack (500 entries) keeps both lengths, so there the call's `bool`
  decides. Pointer *moves* are never wrapped, because tools only commit on
  down or up.
- `LayerKey`, `plan_redraw` and `Redraw` (`src/shell/app.rs`) form a pure
  decision table: no cache or a changed key → `Full`; same key and dirty →
  `Overlay`; otherwise → `None`.
- `CachedFrame::present` creates both MSAA targets, renders the document
  layer on `Full` (`draw_document`), and on `Full` or `Overlay` re-renders
  the frame: `blit` of the document texture, then `draw_overlay`. Both
  layers use the same `Camera2D::from_display_rect`, so the `flip_y` blit
  into the frame works exactly like the final blit to the window.
- `DRAW_FRAME_TIMES=1` (`frame_timing_enabled`, `frame_log_line`) prints one
  line per re-render.

## Try it

1. `cargo build --release`, then
   `DRAW_FRAME_TIMES=1 ./target/release/draw`. Draw a few shapes, `Ctrl+A`,
   then `Ctrl+D` nine times.
2. Drag a circle: the lines say `overlay re-render` and stay flat. Pan with
   the middle button: they say `full re-render` and grow with the shape
   count.
3. Remove `underlay` from `LayerKey` and its tests, run the app, and toggle
   grid snap. The dots appear only after the next pan: that is a stale
   cache key.
4. Add a test in `core::editor` showing that `Command::CycleSmoothing` keeps
   the revision.

## Further reading

- [[Rendering performance options]] — what to try next (append-only
  updates, cheaper tessellation, retained buffers).
- [[ADR-T24-1 Document layer keyed by a document revision]].
- Brendan Gregg, *perf Examples* — reading `perf report` call graphs.
