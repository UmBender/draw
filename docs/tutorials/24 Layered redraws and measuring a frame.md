---
title: Layered redraws and measuring a frame
step: 24
requires: ["[[12 The app loop, idle redraw and UI]]", "[[14 Profiling and shipping a release build]]", "[[15 Anti-aliasing and multisampling]]"]
feature:
code: ["src/shell/app.rs", "src/shell/render.rs", "src/core/editor.rs"]
tags: [tutorial]
---

# Layered redraws and measuring a frame

> **Learning path step 24.** Requires: steps 12, 14 and 15 · Next: —

## Why this matters here

After drawing ten shapes and doubling them nine times (`Ctrl+A`, `Ctrl+D`),
a page holds about 5 000 shapes, and drawing one circle felt sluggish.
Before this step, every pointer move re-tessellated and re-uploaded *all*
shapes, although only the circle preview had changed. A frame cost
O(document) when it should cost O(change). This step is also a lesson in
*why you measure*: the first fix was right but not enough. Only the
owner's numbers showed where the rest of the time went.

## The concept

**Measure before optimising, and with more than one tool.**

- A frame timer is the cheapest profiler: wrap the work in `Instant::now()`
  / `elapsed()` and print the result together with the input size.
- The timer only sees CPU time. The GPU runs asynchronously, so a move can
  log 0.15 ms and still feel clunky because of GPU fill. A/B switches like
  `DRAW_MSAA=off` reveal that.
- `perf record` shows where the CPU time goes. You need symbols for that:
  build with `CARGO_PROFILE_RELEASE_STRIP=false` and
  `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only`.

**Layers.** Split what is drawn by how often it changes:

```
 document layer (cached, MSAA)           window, every woken frame
┌──────────────────────────┐   blit   ┌────────────────────────────┐
│ bg, dot grid, shapes     │ ───────▶ │ document layer             │
│ (minus hidden ones)      │          │ + preview, guides,         │
└──────────────────────────┘          │   selection, marquee,      │
  re-rendered on a key change,        │   toolbar (single-sample)  │
  appended to on a pure add           └────────────────────────────┘
```

**Cache keys.** A cache is valid only as long as its *inputs* are unchanged.
Write the inputs down as a value (here `LayerKey`) and compare. Anything you
forget in the key becomes a stale-picture bug. So the key lists *every*
input of `draw_document`: document revision and length, camera, size,
hidden shapes, and whether the dot grid is on.

**Revisions instead of diffs.** Comparing two 5 000-shape documents on
every move would cost what we are trying to save. A counter that grows on
every change is O(1) to compare. It does not have to be exact: an extra bump
only costs one unneeded full redraw, while a missed bump leaves a stale
picture. So when in doubt, bump. A second counter, the *base revision*,
moves only on changes that are not pure appends. While it stays at or below
the cached revision, the old layer is still a correct prefix, and drawing
the new shapes on top finishes it.

**Batching.** A GPU API call is cheap; a hundred thousand of them are not.
macroquad copies the vertices of every `draw_triangle` into its batch with
its own `memmove`. Collecting the triangles of many shapes into one vertex
and index buffer cuts the per-call overhead to one copy per few thousand
indices. Quads then share 4 vertices instead of 6, and fans share their
centre.

## How draw implements it

- `Editor::document_revision` and `document_base_revision`
  (`src/core/editor.rs`):
  - `Editor::track_document` wraps the calls that can commit (pointer
    down/up, cut, paste, duplicate, delete, undo, redo, clear). It compares
    the undo/redo lengths before and after; on a full undo stack the call's
    `bool` decides.
  - Pointer *moves* are never wrapped, because tools commit only on down
    or up.
  - Creation tools on pointer up (`creates_shapes`), plus paste and
    duplicate, are appends when the document grew. Everything else also
    raises the base revision.
- `LayerKey`, `plan_redraw` and `Redraw` (`src/shell/app.rs`) form a pure
  decision table:

  | Situation | Result |
  |-----------|--------|
  | No cache yet | `Full` |
  | Only appends, view unchanged | `Append { from }` |
  | Any other key change | `Full` |
  | Same key, input changed something | `Overlay` |
  | Nothing changed | `None` |

- `CachedFrame::present` creates the MSAA document target and calls
  `draw_document` on `Full` (clear + underlay + shapes) or on `Append` (the
  shapes from `from` on, no clear). macroquad starts render passes with
  `PassAction::Nothing`, so the target keeps its pixels. Then it `blit`s the
  layer to the window and calls `draw_overlay` on top.
- `Batch` (`src/shell/render.rs`) offers `triangle`, `rect`, `line`, `fan`
  and `strip`. It sends chunks of at most `BATCH_MAX_VERTICES` /
  `BATCH_MAX_INDICES` to a `MeshSink`: `GlSink` calls `draw_mesh`, and the
  tests use a recording sink. Labels call `batch.flush()` before drawing
  text, so they stay on top of their shape.
- `DRAW_FRAME_TIMES=1` (`frame_timing_enabled`, `frame_log_line`) prints one
  line per re-render: `full`, `append` or `overlay`.

## Try it

1. Run `cargo build --release`, then
   `DRAW_FRAME_TIMES=1 ./target/release/draw`. Draw ten shapes, then press
   `Ctrl+A`, `Ctrl+D` nine times.
2. Drag a circle: the lines say `overlay`. Release it: one `append` line,
   not `full`. Undo it: `full`. Pan: `full`, and the time grows with the
   shape count.
3. Remove `underlay` from `LayerKey` and its tests, run the app, and toggle
   grid snap. The dots appear only after the next pan: that is a stale
   cache key.
4. In `track_document`, treat every change as an append. Erase a shape: it
   stays on screen until you pan. That is why only commands known to insert
   on top count as appends.
5. Set `BATCH_MAX_INDICES` to 6 and compare the `full` times: that is
   roughly the old one-call-per-quad cost.

## Further reading

- [[Rendering performance options]] — what is done and what is still open.
- [[ADR-T24-1 Document layer keyed by a document revision]],
  [[ADR-T24-2 Overlay drawn straight to the window]],
  [[ADR-T24-3 Batched meshes and append-only document updates]].
- Brendan Gregg, *perf Examples* — reading `perf report` output.
