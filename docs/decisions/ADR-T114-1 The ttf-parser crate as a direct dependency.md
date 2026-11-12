---
id: ADR-T114-1
title: The ttf-parser crate as a direct dependency
status: accepted
kind: decision
date: 2026-11-11
task: T114
builds_on: ["[[ADR-0014 Minimal dependencies and no unsafe]]", "[[ADR-T92-1 The png crate as a direct dependency]]"]
amends: ["[[ADR-0014 Minimal dependencies and no unsafe]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T114-1 The ttf-parser crate as a direct dependency

## Context

`Ctrl+Shift+E` writes the drawing as an SVG whose text is glyph outlines
([[T114 Export to SVG]]). fontdue rasterizes and exposes metrics, but not
the outlines of a glyph. [[ADR-0014 Minimal dependencies and no unsafe]]
requires an ADR for any new dependency. `ttf-parser` is already compiled and
linked: fontdue depends on it, and `Cargo.lock` has it at 0.25.1.

## Decision

- `ttf-parser = "0.25"` becomes a direct runtime dependency, the version
  fontdue already uses, so no second copy is built.
- `shell::glyphs` is its only user: it reads the outline of a character of
  the embedded text and math fonts and returns it as an SVG path in font
  units.
- The owner approved this dependency for the batch run (plan, T114).

## Alternatives considered

- **Tracing the fontdue bitmaps** — blurry vector output, far more code.
- **A hand-written glyf/CFF parser** — the math font is OpenType/CFF; a
  parser is a large, risky amount of code for no gain.

## Consequences

- `ttf-parser` must move together with fontdue.
- No new crate is built; only the outline code that fontdue did not link.

## Rollback plan

Remove the outline reader in `shell::glyphs` and the line in `Cargo.toml`,
in a `revert(T114)` commit referencing a new ADR.
