---
id: ADR-0002
title: Rust and macroquad
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0001]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0002 Rust and macroquad

## Context

Target machine: Linux on Wayland, Intel i5 8th gen with integrated graphics
(UHD 620, OpenGL 4.x capable). Requirements: low CPU use, quick startup, few
dependencies, nothing that breaks mid-contest.

## Decision

Rust (edition 2024, stable toolchain) with **macroquad** for windowing, input
and GPU-accelerated 2D drawing. macroquad is the only runtime dependency.

## Alternatives considered

- **winit + softbuffer + tiny-skia** (CPU raster) — very predictable but more
  code (event loop, buffer management) and redraw cost grows with screen size
  on a weak CPU.
- **egui/eframe** — great widgets but much larger dependency tree and an
  immediate-mode UI that repaints more than needed.
- **wgpu directly** — far too much code for a scratchpad.

## Consequences

- Shapes are tessellated and drawn by the GPU; the CPU stays mostly idle.
- miniquad (under macroquad) picks X11 (via XWayland) or native Wayland. Both
  must be verified in [[T12 App shell and toolbar]].
- macroquad's own types must not leak into `core` (see
  [[ADR-0003 Headless core and thin shell]]).

## Rollback plan

Only `shell/` depends on macroquad, so replacing it means rewriting `shell/`
alone, recorded as an ADR that supersedes this one.
