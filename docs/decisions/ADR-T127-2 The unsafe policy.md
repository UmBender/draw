---
id: ADR-T127-2
title: The unsafe policy
status: accepted
kind: decision
date: 2026-11-12
task: T127
builds_on: [ADR-T127-1]
amends: [ADR-0014]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, 2.0, unsafe]
---

# ADR-T127-2 The unsafe policy

## Context

[[ADR-0014 Minimal dependencies and no unsafe]] sets
`unsafe_code = "forbid"` crate-wide. That holds for `src` (0 `unsafe`
lines), but all 544 `unsafe` lines of the tree are in the vendored miniquad
([[2.0 baseline]]). Owner decision, 2026-11-12: "**`unsafe` is allowed**,
in confined modules, with a full test set per primitive."

EGL needs the `wl_display` of libwayland-client and the X11 path needs the
connection of libxcb, so a backend of our own cannot avoid FFI.

## Decision

- `unsafe` is allowed only in `draw-sys` and in `draw-gfx/gpu/gl.rs`.
  `draw-types`, `draw-core` and `draw` keep `unsafe_code = "forbid"`;
  `draw-gfx` has `deny` with that one file allowed.
- Primitives are limited to: dynamic loading (`dlopen`), FFI calls, shared
  memory for buffers and passing file descriptors.
- `clippy::undocumented_unsafe_blocks` is denied; each block has a `SAFETY:`
  comment and each wrapper a `# Safety` section.
- A primitive is merged only with all of: a safe wrapper that makes misuse
  impossible by type; unit tests of its invariants and error paths;
  integration tests against Xvfb and a headless sway (`scripts/e2e.sh`);
  one regression test per bug (`regression_<ID>_<slug>`, never deleted);
  Miri for the pure parts, address and thread sanitizers in the nightly job,
  and fuzzing of every decoder of bytes that came from a server.
- `scripts/check.sh` checks the lint level of every crate.

This amends only the `unsafe_code = "forbid"` line of ADR-0014.

## Alternatives considered

- **Keep `forbid` everywhere** — impossible without a pure-Rust window and
  GL context, which cannot hand EGL a display handle.
- **A crate for FFI** — adds dependencies against
  [[ADR-T127-3 The dependency list]].

## Consequences

- The compiler enforces where `unsafe` may be written.
- Each primitive costs a test set larger than its code.

## Rollback plan

Move the primitive back behind an existing safe wrapper, or restore
`forbid` in `draw-sys` by returning to macroquad with a new ADR.
