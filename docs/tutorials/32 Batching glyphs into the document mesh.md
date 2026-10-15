---
title: Batching glyphs into the document mesh
step: 32
requires: ["[[24 Layered redraws and measuring a frame]]", "[[29 Typing on a canvas without losing the shortcuts]]"]
feature: "[[Text blocks]]"
code: ["src/shell/glyphs.rs", "src/shell/render.rs", "src/shell/app.rs", "tests/perf.rs"]
tags: [tutorial]
---

# Batching glyphs into the document mesh

> **Learning path step 32.** Requires: step 24 (layered redraw and batched
> meshes) and step 29 (text blocks) · Next: —

## Why this matters here

Step 24 made 5 000 strokes cheap by sending them to the GPU as a few big
meshes instead of one draw call per triangle. Text blocks (step 29) went
the other way: one `draw_text_ex` call per character. With 512 blocks
(~10 000 characters) a steady full re-render took 10–13 ms, 70 % of it in
text, and the first render after zooming into a new raster size took
80–160 ms: a visible hitch every time the view settled.

The lesson is the one from step 24, applied to text: **measure first**,
then remove per-item overhead by batching. Measuring changed the plan.
The suspect was the per-call overhead, but most of the hitch came from
macroquad's glyph cache re-uploading its *whole* atlas texture after each
new glyph.

## The concept

A **glyph atlas** is one big texture holding many small glyph bitmaps.
Text becomes a list of textured quads, one per character, each pointing at
its bitmap with UV coordinates. Three ideas make it fast:

1. **Rasterize once.** A glyph is rasterized once per `(character, pixel
   size)` and remembered. A *shelf packer* places bitmaps left to right
   in rows ("shelves"), each row as tall as its tallest bitmap:

   ```text
   ┌────────────────────────────┐
   │■ a b c d e f g h i j k l m │  ← shelf 1 (white block ■ first)
   │n o p q r s t u v w x y z   │  ← shelf 2
   │A B C …                     │
   └────────────────────────────┘
   ```

2. **Upload only what changed.** A new glyph sends only its own texels to
   the GPU (`Texture2D::update_part`), not the whole page.
3. **One mesh for shapes *and* text.** The default shader computes
   `texture sample × vertex colour`. If every page has a small opaque
   white block, a shape whose vertices all sample that white texel looks
   exactly as if it were untextured. Shapes and glyphs can then share
   the same chunk and the same draw call, and z-order is just the order
   things were added in.

Memory is bounded: pages are 1024² RGBA (4 MiB), at most 4. When they are
full the atlas is cleared at the start of the next frame and the layer is
redrawn once.

## How draw implements it

- `src/shell/glyphs.rs`:
  - `ShelfPacker::pack` is the shelf packer with a 2-texel `GAP`, so
    linear filtering never bleeds between bitmaps.
  - `GlyphAtlas` is CPU-only and runs in unit tests without a window.
    `prepare(ch, px)` looks up fontdue's metrics, packs the bitmap, and
    only then rasterizes it into a queued `Upload`. A glyph with no room
    sets `overflowed` and is never rasterized. Its keys use a tiny
    multiply-rotate `KeyHasher`, because std's SipHash was the largest
    cost of tessellation.
  - `GlyphTextures::sync` creates page textures and applies the uploads.
  - `glyph_quad` turns a baseline point, an entry and a scale into a
    screen rectangle and UVs: `left = x + xmin·s`,
    `top = y − (ymin + h)·s`.
- `src/shell/render.rs`:
  - `Batch` remembers which page its chunk is bound to. `Batch::glyph`
    flushes only when a glyph comes from another page, and shape vertices
    use `WHITE_UV`.
  - `prepare_text_block` and `push_text_block` (text blocks), and
    `prepare_text_centered` and `push_text_centered` (number labels and
    grid indices), are pure functions over an atlas and a batch.
    `with_glyphs` wraps them for the GL path: prepare, sync the
    textures, then push.
- `src/shell/app.rs`: `DRAW_FRAME_TIMES=1` lines now end with
  `(text 2.4 ms, 26 rasterized)`. When the atlas overflowed in a frame,
  `rerender_after_overflow` re-renders the layer once.

## Try it

- Run `DRAW_FRAME_TIMES=1 cargo run --release` and build 512 blocks with
  the doubling recipe (8 blocks, then `Ctrl+A`, `Ctrl+D` six times). Zoom
  in until text grows: the first re-render at a new size reports a few
  dozen `rasterized`, the next ones `0`.
- Set `MAX_PAGES` to 1 and `PAGE_SIDE` to 256, then zoom to 20×: watch
  the atlas overflow, clear and redraw instead of growing.
- Run `cargo test --release --test perf -- --ignored tessellate`, then
  switch `GlyphAtlas::entries` back to a plain `HashMap` and compare the
  timings.

## Further reading

- [[ADR-T32-1 Own glyph atlas rasterized with fontdue]],
  [[ADR-T32-2 Shapes and glyphs share one textured batch]]
- [[ADR-T24-3 Batched meshes and append-only document updates]]
- Jukka Jylänki, *A Thousand Ways to Pack the Bin* (shelf packing).
