---
id: ADR-T24-2
title: Overlay drawn straight to the window
status: accepted
kind: decision
date: 2026-10-02
task: T24
builds_on: [ADR-T24-1, ADR-T12-1]
amends: [ADR-T24-1, ADR-T15-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T24-2 Overlay drawn straight to the window

## Context

With [[ADR-T24-1 Document layer keyed by a document revision]], a pointer
move during a drag re-renders the *frame* target: the full-screen document
texture is blitted into a 4× multisampled target, the overlay is drawn on
top, the target is resolved, and the result is blitted to the window. On
the target (UHD 620) that move costs only 0.15 ms of CPU but still feels
clunky, and gets smoother with `DRAW_MSAA=off`. The cost is GPU fill: about
four full-screen sample writes, a resolve and one more blit per move.

## Decision

- Drop the cached frame target. Every woken frame blits the cached document
  layer to the window (one full-screen quad) and draws the overlay on top:
  preview shapes, guides, selection, marquee and toolbar.
- Only the document layer is multisampled
  ([[ADR-T15-1 MSAA on the cached frame]] now applies to it alone). The
  overlay is drawn on the window's single-sample framebuffer.
- Drawing the overlay on every woken frame, even when nothing changed,
  costs ≈ 0.15 ms of CPU. That is the price of no longer caching it.

## Alternatives considered

- **Single-sample frame target** — the same loss of anti-aliasing on the
  overlay, plus one more full-screen pass and another texture.
- **Window MSAA (`Conf::sample_count`)** — keeps the overlay smooth, but
  every frame resolves a full-screen multisampled framebuffer, which is the
  cost we are removing.
- **A small MSAA target just for the toolbar** — keeps toolbar icons
  smooth. Worth adding only if the aliased icons bother the owner.

## Consequences

- One full-screen single-sample blit per woken frame instead of about six
  sample-weighted passes. Half the GPU memory of ADR-T24-1 (one MSAA target).
- Preview shapes, guides and the toolbar's drawn icons (circle, arrow,
  swatches) are aliased. Committed shapes stay multisampled, and so does
  toolbar text, which is anti-aliased by its font texture.
- `Redraw::Overlay` and `Redraw::None` now differ only in the frame timer:
  the overlay is drawn either way.

## Rollback plan

`shell::app`: bring back the frame target from ADR-T24-1 (render the overlay
into it when dirty, blit it every frame). Record it as a rollback ADR
reverting this one.
