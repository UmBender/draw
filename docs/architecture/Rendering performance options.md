---
tags: [architecture]
task: "[[T15 Anti-aliasing]]"
status: notes
---

# Rendering performance options

Ideas for making redraws cheaper, collected after [[T15 Anti-aliasing]].
Nothing here is decided: each option that is picked up needs its own task and
ADR. Measure first (see *How to measure*), then pick the cheapest option that
fixes the measured problem.

## Where we are

Measured by the user on the target (i5-8th gen iGPU, Wayland), 2026-10-01,
while spamming circles quickly:

| Build | CPU while drawing | Idle |
|-------|-------------------|------|
| `DRAW_MSAA=off` | ≈ 8 % | ≈ 0 % |
| 4× MSAA (default) | ≈ 20 % | ≈ 0 % |

Acceptable for now. Idle stays at zero thanks to the blocking loop
([[ADR-T12-1 Blocking event loop with cached frame]]).

## Why drawing costs what it costs

Every state change re-renders the **whole scene** into the cached frame
(`shell::app::CachedFrame::present` → `draw_scene`):

1. Every visible shape is re-tessellated on the CPU (`shell::render`: polyline
   joints, ellipse rings with up to `MAX_SEGMENTS` = 256 segments, arrow heads).
2. macroquad's immediate-mode batcher (`QuadGl`) collects those triangles and
   re-uploads them to the GPU every time.
3. With MSAA the rasteriser writes 4 samples per pixel, then resolves.

While dragging a new circle, each pointer move is one full re-render, so the
cost grows with *number of shapes × pointer events*, not with what actually
changed (one preview shape). MSAA did not create that pattern; it multiplied
its fill cost.

## Options without `unsafe`

Ordered by expected payoff for the effort.

### 1. Two layers: committed document vs live overlay (biggest win)

Keep **two** cached targets: the *document layer* (all committed shapes) and
the frame. A pointer move during a gesture only changes the overlay
(preview, marquee, selection), so:

- document changed (commit, undo, pan, zoom, resize) → re-render document layer;
- only overlay changed → blit document layer + draw the 1–2 overlay shapes.

Drawing a circle then costs O(1 preview shape) per move instead of O(all
shapes). Needs the editor to report *what* changed (`Change::{Document,
View, Overlay}`) instead of a single `bool` from `Editor::handle` — a small
core API change (ADR, owner of `editor.rs`). The tools that hide shapes while
dragging (move, eraser via `Overlay::hidden`) must force a document-layer
re-render when `hidden` changes.

### 2. Append-only document updates

The most common change is "one shape added". Draw just that shape onto the
existing document layer (no clear) instead of re-rendering everything; fall
back to a full re-render for removals, edits, undo and camera changes. Needs a
document revision counter or "shapes appended since revision N" query in
core. Combines well with option 1.

### 3. Lower or adaptive MSAA

- `DRAW_MSAA=2` roughly halves the multisample fill cost; often visually
  close to 4× for line art. Try it before anything else — zero code.
- Render the *overlay* without MSAA during a gesture and only the document
  layer with MSAA (needs option 1); the preview is on screen for a moment.

### 4. Cheaper tessellation

- Lower the segment count curve in `render::circle_segments` (error bound of
  ~0.5 px instead of the current target), or cap at 128.
- Draw ellipse **fills** and outlines as one triangle fan + ring sharing
  vertices instead of separate `draw_triangle` calls.
- Use `macroquad::models::Mesh` + `draw_mesh` (safe) to submit a shape's
  vertices in one call instead of per-triangle calls; fewer function calls and
  index reuse.

### 5. Per-shape mesh cache

Cache each shape's tessellation keyed by `(ShapeId, zoom bucket)`; a
pan just changes the camera transform (draw meshes in world space with a
`Camera2D`), so only zoom changes re-tessellate. Widths are world units
([[ADR-0013 World-space widths and zoom limits]]), so world-space meshes are
valid; the screen-space tessellation of [[ADR-T07-1 Screen-space tessellation in the renderer]]
would need amending for thin-line minimum widths. Memory: a few KB per shape.

### 6. Throttle re-renders to the display refresh

Pointer devices can report at 500–1000 Hz. The collector already coalesces
all events since the last frame into one re-render, but with the blocking
loop each wake can trigger a frame. Setting `swap_interval: Some(1)` (vsync)
in the window conf, or skipping a re-render if the last one was < ~8 ms ago
while still processing events, caps work at the refresh rate.

### 7. Dirty-rectangle redraw

Re-render only the screen region that changed (union of old and new bounds of
changed shapes), using the camera to clip and `Camera2D::viewport` to restrict
drawing. Big win for large documents with small edits, but correctness is
fiddly (strokes overlapping the dirty rect, anti-aliased edges bleeding one
pixel). Consider only after 1 and 2.

### 8. Build flags

`-C target-cpu=native` in a local `.cargo/config.toml` (not committed for
portability) lets the compiler vectorise tessellation loops. Small, free.

## Options that need `unsafe`

`unsafe_code = "forbid"` ([[ADR-0014 Minimal dependencies and no unsafe]])
cannot be overridden locally. Any of these needs an ADR **amending** ADR-0014
to change the lint to `deny` and allow `unsafe` in exactly one shell module
(e.g. `shell::gpu`), with each block documented with a `// SAFETY:` comment.
The core stays `unsafe`-free.

### U1. Detect MSAA support (`get_internal_gl`)

`unsafe fn get_internal_gl()` exposes miniquad's
`info().features.resolve_attachments`, replacing the `DRAW_MSAA` escape hatch
with automatic fallback on GL 2. Correctness, not speed. The unsafety is only
"call from the main thread while no other macroquad borrow is alive", which is
easy to uphold in `run()`.

### U2. Retained GPU buffers via the miniquad backend

Through `get_internal_gl().quad_context` (a `&mut dyn RenderingBackend`),
create **static vertex/index buffers** per shape (or per chunk of shapes) and a
tiny pipeline with a camera uniform. Panning and zooming only update the
uniform: no re-tessellation and no re-upload; a frame becomes a handful of
draw calls. This is the "real" fix for large documents (10 000+ shapes) and
pairs with option 5. Cost: a custom shader, buffer lifetime management, and
flushing macroquad's batch (`quad_gl.flush()`) before and after our calls.

### U3. Scissored and partial resolves

Raw GL access (`glScissor`, `glBlitFramebuffer` on a sub-rectangle) lets
dirty-rect redraws (option 7) and MSAA resolves touch only the changed
region. Highest complexity; only worth it if profiling shows the resolve
itself dominates.

## Bigger moves (new dependencies)

Each needs an ADR against [[ADR-0014 Minimal dependencies and no unsafe]] and
[[ADR-0002 Rust and macroquad]].

- **A vector renderer** (`femtovg`, `vello`, `tiny-skia` on the CPU): proper
  analytic anti-aliasing, path caching, no MSAA cost. `vello` needs `wgpu`
  (heavy); `tiny-skia` is pure CPU — fine for small documents, upload one
  texture per frame.
- **`wgpu` directly**: full control over buffers and MSAA, much larger
  binary and compile time.

## How to measure

Before choosing, measure on the target:

- Add a debug-only frame timer around `render_into` (log ms per re-render and
  the shape count) — no new dependency.
- `perf record -g ./target/release/draw` then `perf report`: is the time in
  `shell::render` (tessellation), `QuadGl` (batching/upload) or the driver
  (fill/resolve)?
- Compare `DRAW_MSAA=off|2|4|8` with the same scripted session; T14's perf
  tests (`tests/perf.rs`) can grow a headless tessellation benchmark.

## Suggested order

1. Try `DRAW_MSAA=2` (free).
2. Measure (frame timer + `perf`).
3. Option 1 (two layers) — likely removes most of the 20 % while drawing.
4. Option 2 (append-only) and 4 (cheaper tessellation) if still needed.
5. Only for very large documents: U2 retained GPU buffers, behind an ADR.
