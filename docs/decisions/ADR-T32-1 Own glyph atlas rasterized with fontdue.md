---
id: ADR-T32-1
title: Own glyph atlas rasterized with fontdue
status: accepted
kind: decision
date: 2026-10-14
task: T32
builds_on: [ADR-0014, ADR-T24-3, ADR-T29-1]
amends: [ADR-0014]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T32-1 Own glyph atlas rasterized with fontdue

## Context

[[ADR-T29-1 Text shape with monospaced layout and an embedded font]] draws
every character of a text block with one `draw_text_ex` call. Measured on
the target (i5-8250U, 512 blocks, 9 728 characters, see the *Log* of
[[T32 Fast text rendering]]):

- **steady** full re-render 10–13 ms, of which 7–9 ms is text
  (≈ 0.8 µs per character: two mutex locks, two hash lookups and one
  `draw_texture_ex` quad per call);
- **first render at a new raster size** 80–160 ms, almost all of it text.
  macroquad's atlas marks itself dirty for each newly cached glyph and the
  next `draw_text_ex` re-uploads the **whole** atlas texture, so a new
  size costs one full-texture upload per new glyph. The owner's 68 ms
  sample in [[T29 Text blocks]] was one of these.

macroquad does not expose its atlas, its UVs or its glyph metrics, so text
cannot join the shape meshes of
[[ADR-T24-3 Batched meshes and append-only document updates]] without an
atlas of our own.

## Decision

- **Direct `fontdue` dependency**, `fontdue = "0.9"`, which resolves to
  the 0.9.4 macroquad already pulls in: no second copy, `Cargo.lock` only
  gains the edge from `draw` to it, binary growth ≈ 0. This amends
  [[ADR-0014 Minimal dependencies and no unsafe]] (runtime dependencies:
  macroquad, and fontdue pinned to macroquad's version). When macroquad
  moves to another fontdue, this requirement moves with it in the same
  commit.
- **`shell::glyphs`** owns text rasterization:
  - `GlyphAtlas` (CPU only, no GL): the embedded font parsed by fontdue
    with default settings — the same rasterizer and settings macroquad
    uses, so bitmaps are identical — a map `(char, raster px) → entry`
    and one shelf packer per page. Entries hold the page, texel rectangle
    and fontdue metrics (bearing, bitmap size). Rasterized bitmaps become
    pending uploads (`[255, 255, 255, coverage]` texels on a transparent
    black page, like macroquad's atlas); the CPU keeps no page copies.
  - Pages are `PAGE_SIDE` = 1024 texels square, RGBA8 (4 MiB each), at
    most `MAX_PAGES` = 4 (16 MiB of GPU memory). Glyphs are packed with a
    `GAP` of 2 texels on every side, as macroquad does, so linear
    filtering never bleeds between glyphs.
  - Raster size in physical pixels is `ceil(raster × dpi)` as in
    macroquad, clamped to `MAX_RASTER_PX` = 512 so every glyph fits a
    page; a glyph's quad is its bitmap scaled by
    `screen px / raster px` (logical pixels per texel).
  - Before packing, the glyph's size comes from fontdue's metrics (no
    rasterization). When no page has room and `MAX_PAGES` are in use, the
    atlas records an **overflow** and the glyph is not drawn. At the start
    of the next frame an overflowed atlas is **cleared** (entries and
    packers reset, textures kept); the shell then re-renders the document
    layer in full once, so missing glyphs reappear. A frame that
    overflows right after a clear does not retry, so the loop never spins.
    At most 12 raster sizes × the glyphs on screen are ever live.
  - `GlyphTextures` (GL): one `Texture2D` per page, linear filter like
    macroquad's font atlas, created zeroed when the page appears; pending
    uploads go up with `Texture2D::update_part`, once per text block that
    rasterized something — never the whole page per glyph.
  - `glyph_quad` (pure): baseline point, entry and scale → screen quad and
    UVs.
- The atlas and textures live in two main-thread `thread_local!` cells so
  the batch's sink can read textures while text is being tessellated.
- If the embedded font cannot be parsed (a unit test proves it parses),
  text blocks and labels are not drawn and the error is logged once. The
  toolbar keeps macroquad's default font (out of scope).

## Alternatives considered

- **Cache each text block as a small render target** — one texture and
  one draw per block, still one `draw_text_ex` per character when a block
  is (re)rendered at a new zoom; hundreds of render targets to budget;
  does not fix the first-at-new-size cost.
- **One `draw_text_ex` call per line** — removes some per-call overhead
  but keeps macroquad's whole-atlas re-upload and one quad per character
  through `draw_texture_ex`; text still cannot share the shape meshes.
- **Pre-populate macroquad's cache** (`Font::populate_font_cache`) for all
  sizes at start-up — 12 sizes × the subset is slow to build and large,
  and each cached glyph still re-uploads the atlas once.
- **SDF fonts / custom shaders** — out of scope (GPU-side techniques).

## Consequences

- Text rendering no longer depends on macroquad's text module (only the
  toolbar does).
- Rasterization cost is paid once per `(char, size)` and uploads touch
  only the new glyph's texels.
- Memory: at most 16 MiB of textures plus the entry map; no CPU page
  copies.
- One more crate edge to keep in step with macroquad.

## Rollback plan

Revert the T32 behaviour commit: `render.rs` goes back to `draw_text_ex`
per character with the macroquad `Font` from `set_text_font`, and
`shell::glyphs` plus the `fontdue` line in `Cargo.toml` are removed, in a
`revert(T32)` commit with a rollback ADR reverting this one and
[[ADR-T32-2 Shapes and glyphs share one textured batch]].
