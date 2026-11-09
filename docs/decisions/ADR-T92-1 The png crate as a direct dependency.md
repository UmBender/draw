---
id: ADR-T92-1
title: The png crate as a direct dependency
status: accepted
kind: decision
date: 2026-11-09
task: T92
builds_on: ["[[ADR-0014 Minimal dependencies and no unsafe]]", "[[ADR-T32-1 Own glyph atlas rasterized with fontdue]]"]
amends: ["[[ADR-0014 Minimal dependencies and no unsafe]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T92-1 The png crate as a direct dependency

## Context

`Ctrl+E` writes the view to a PNG file ([[T92 Export the view to PNG]]).
macroquad's `Image::export_png` panics when the write fails: on a full disk
or a read-only home, `panic = "abort"` would lose the drawing. The write has
to return an error instead.

[[ADR-0014 Minimal dependencies and no unsafe]] requires an ADR for any new
dependency. `png` is already compiled and linked: macroquad needs it through
`image`, and `Cargo.lock` has it at 0.17.16.

## Decision

- `png = "0.17"` becomes a direct runtime dependency, the version macroquad
  already uses, so no second copy is built.
- `shell::export` is its only user: it encodes RGBA pixels to bytes in
  memory and returns every failure as a value.
- Pixels are encoded with `Compression::Fast`. The canvas is mostly flat, so
  the file stays small and the one frame that encodes stays short.

## Alternatives considered

- **`Image::export_png`** — panics when the write fails.
- **A hand-written PNG writer with stored deflate blocks** — no dependency,
  but more code to test, and files many times larger.
- **The `image` crate directly** — a far larger API for one encoder, and
  `image` only reaches `png` for this format anyway.

## Consequences

- `png` must move together with macroquad: an upgrade that makes macroquad
  use another `png` would build two copies until this version follows.
- The binary does not grow by a new crate, only by the encoder's code that
  was not already linked.

## Rollback plan

Remove `shell::export`'s use of `png` and the line in `Cargo.toml`, in a
`revert(T92)` commit referencing a new ADR.
