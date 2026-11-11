---
id: ADR-T115-1
title: Selection image on the clipboard
status: accepted
kind: decision
date: 2026-11-11
task: T115
builds_on: ["ADR-T35-1", "ADR-T92-1"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T115-1 Selection image on the clipboard

## Context

`Ctrl+Shift+C` must put the selected shapes on the system clipboard as a
PNG. miniquad's clipboard carries text only.

## Decision

- The vendored miniquad gets `Clipboard::set_png` (default: nothing) and
  `miniquad::window::clipboard_set_png(&[u8])`. Wayland offers one
  `wl_data_source` MIME type, `image/png`; X11 answers `TARGETS` and the
  `image/png` target of the `CLIPBOARD` selection. No external `wl-copy`
  or `xclip`, and no text fallback.
- Wayland writes the bytes to the receiver's pipe on a short thread, so a
  slow reader never stalls the frame. X11 answers with one property
  (no INCR); a PNG of a diagram is far below the server's request limit.
- Setting text or an image replaces the other.
- The image is the selection's box plus a 16 px margin, on the canvas
  background, at the current zoom but at least 1:1, at most
  4096 × 4096 px; a larger one is scaled down and the log says so.

## Alternatives considered

- **Shell out to `wl-copy` / `xclip`** — extra runtime dependencies, owner
  decided against.
- **INCR on X11** — much more code for sizes a diagram never reaches.

## Consequences

The patch is small and sits beside the text clipboard. It is not covered
by automated tests: it is checked by hand on Hyprland and X11.

## Rollback plan

Revert the T115 commits; `PATCHED.md` loses its entry with them.
