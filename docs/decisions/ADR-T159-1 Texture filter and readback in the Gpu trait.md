---
id: ADR-T159-1
title: Texture filter and readback in the Gpu trait
status: accepted
kind: decision
date: 2026-11-13
task: T159
builds_on: [ADR-T140-1]
amends: [ADR-T140-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T159-1 Texture filter and readback in the Gpu trait

## Context

[[ADR-T140-1 The Gpu and Platform traits]] fixes a texture's filter at
creation and has no way to read pixels back. The frame cache switches the
document layer between nearest and linear sampling per frame (a layer
scaled during a gesture is linear, one that lands 1:1 is nearest), gives the
minimap's overview a linear filter, and reads the layer back for the PNG
export and the selection image.

## Decision

- `TargetDesc` gets a `filter: Filter` field: the filter of the texture the
  target resolves to.
- `Gpu::set_filter(&mut self, texture, Filter) -> Result<(), GpuError>`
  changes the filter of an existing texture (also the texture of a target).
  It is a method, not a field of `Draw`, so the draws of T158 stay as they
  are.
- `Gpu::read_texture(&mut self, texture) -> Result<Vec<u8>, GpuError>`
  returns the RGBA8 texels in the order the backend stores them (rows from
  the bottom for a target texture on OpenGL); `width * height * 4` bytes.
  It is a trait method, not an `MqGpu`-only one, so every interpreter
  supports export.
- `BeginPass { target: None }` draws to the window with its own camera
  again: a pass never inherits the caller's camera.
- Draws into a target are collected per scope and submitted as **one**
  pass (one resolve of the multisampled target), not one pass per batch
  chunk.

## Consequences

- `RecordingGpu` records `SetFilter` and `ReadTexture` calls; `NullGpu`
  returns zeroed pixels of the texture's size.
- Unknown handles give `GpuError::UnknownHandle` in every interpreter.
