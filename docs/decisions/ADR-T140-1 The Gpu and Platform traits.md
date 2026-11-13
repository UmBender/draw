---
id: ADR-T140-1
title: The Gpu and Platform traits
status: accepted
kind: decision
date: 2026-11-13
task: T140
builds_on: [ADR-T127-5, ADR-T131-1, ADR-T132-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T140-1 The Gpu and Platform traits

## Context

The shell talks to macroquad directly (`draw_line`, `draw_mesh`, `next_frame`,
a miniquad `EventHandler`, …). The 2.0 plan replaces macroquad by own
backends ([[ADR-T127-5 OpenGL ES 3.0 first, Vulkan optional]]), so the shell
must first be written against traits that both macroquad and the own
backends can interpret. No `dyn` ([[ADR-T131-1 No dyn outside an allow-list]]): `main`
picks the interpreter with a `match` and monomorphises.

## Decision

`draw-gfx::gpu` defines `Gpu`, `draw-sys::platform` defines `Platform` and
`RawEvent`.

- **`Gpu`** has four associated types (`Buffer`, `Texture`, `Target`,
  `Pipeline`) and creates resources through `&mut self`. It draws only
  through `submit(&CommandList<Self>)`: one recorded list per frame. A list
  is a sequence of `Command`s: `BeginPass { target, view, clear }`, `Draw {
  pipeline, texture, geometry }` and `EndPass`. Every parameter is in the
  command: the view (size in logical pixels, which maps to clip space), the
  render target, the clear colour and the blend mode (in the `Pipeline`
  description). The interpreter keeps **no state between lists**.
- All 2D primitives (line, rectangle, circle, triangle, glyph quad, blit)
  are tessellated on the CPU into `Vertex` triangles; there is one mesh type.
- **`Platform`** polls `RawEvent`s into a caller-owned `Vec` (no allocation
  per frame in steady state), reports `WindowMetrics`, owns the clipboard,
  asks for a wake-up (`schedule_update`) and yields to the compositor
  (`next_frame`, `impl Future`, no `dyn`).
- **`RawEvent`** (keys, buttons, pointer, tablet, size and scale, focus,
  seat) lives in `draw-sys` because `draw-sys` cannot depend on `draw-core`;
  the shell translates it to `machine::Input` in `input_map.rs`.
- Interpreters: a **recording** one (keeps every call in order, used by
  tests), a **null** one (does nothing) and, in the migration step, the
  macroquad one. They live in `src/shell/` (only `src` may depend on
  macroquad) and the recording and null ones are public in the lib.

### Review against OpenGL ES 3.0 and Vulkan (AC-3)

| `Gpu` method / command | OpenGL ES 3.0 | Vulkan | Hidden state? |
|---|---|---|---|
| `create_buffer(vertices, indices)` | `glGenBuffers`, `glBufferData` (`GL_ARRAY_BUFFER`, `GL_ELEMENT_ARRAY_BUFFER`) | `vkCreateBuffer` + memory + staging copy | none: usage fixed by the call |
| `create_texture(desc, rgba)` | `glGenTextures`, `glTexImage2D(GL_RGBA8)`, `glTexParameteri` filter | `vkCreateImage` (`R8G8B8A8_UNORM`), view, sampler from `desc.filter`, staging copy | none: filter is in `desc`, not bound state |
| `write_texture(texture, rect, rgba)` | `glTexSubImage2D` | staging copy + layout barrier | none |
| `create_target(desc)` | `glGenFramebuffers`, colour attachment, optional MSAA `glRenderbufferStorageMultisample` + resolve | `VkImage` with `COLOR_ATTACHMENT \| SAMPLED`, render pass, framebuffer | none: sample count in `desc` |
| `target_texture(target)` | the framebuffer's colour texture (after resolve) | the image view (after the pass end transition) | none |
| `pipeline(desc)` | a program plus `glBlendFunc` choice, kept by the interpreter and applied on every `Draw` | `vkCreateGraphicsPipelines` with the blend state baked in | none: blend is in `desc` and the command names its pipeline |
| `Command::BeginPass { target, view, clear }` | `glBindFramebuffer`, `glViewport`, `glClear` if `clear`; the view is the orthographic matrix uniform | `vkCmdBeginRenderPass` (`loadOp` = clear or load), dynamic viewport, push constant for the view | none: camera is a parameter, not a global |
| `Command::Draw { pipeline, texture, geometry }` | `glUseProgram`, `glBindTexture`, `glDrawElements` | `vkCmdBindPipeline`, descriptor set for the texture, `vkCmdDrawIndexed` | none: pipeline and texture are named per draw |
| `Command::EndPass` | MSAA resolve `glBlitFramebuffer`, unbind | `vkCmdEndRenderPass` | none |
| `submit(list)` | executes the list in order; `eglSwapBuffers` stays in `Platform` | records one command buffer, `vkQueueSubmit` | none: nothing survives the list |

No case needs state set outside the list: the former macroquad globals
(`set_camera`, the current render target, the default camera) become the
`view` and `target` of `BeginPass`. Vulkan's explicit image layout
transitions are the interpreter's, derived from pass order (a target that is
drawn to and then sampled in a later pass gets its barrier at `EndPass`).

## Alternatives considered

- **Immediate `draw_*` methods on `Gpu`** — mirrors macroquad, but hides the
  camera and the render target in state and cannot be recorded as one list.
- **Trait objects (`Box<dyn Gpu>`)** — forbidden by ADR-T131-1.
- **`RawEvent` in `draw-core`** — `draw-sys` must not depend on it
  (`tests/deps.rs`).

## Consequences

- Tests can run the shell against the recording interpreter with no window.
- The macroquad interpreter has to rebuild `Camera2D` and `RenderTarget`
  from the pass parameters on every submit (a few calls per pass).
- A `Draw` with `Geometry::Inline` copies vertices into the list; the
  interpreter reuses its vectors (`CommandList::clear` keeps capacity).

## Rollback plan

Delete `gpu.rs`, `platform.rs` and the interpreters; the shell code that
still calls macroquad is untouched until the migration step lands.
