---
id: ADR-T16-3
title: Helper settings and hooks
status: accepted
kind: decision
date: 2026-10-01
task: T16
builds_on: [ADR-T08-1, ADR-T16-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T16-3 Helper settings and hooks

## Context

Snapping (T17), the grid tool (T18) and numbering (T19) run in parallel and
each owns only its logic file plus a few call sites. Tools see editor state
through `ToolCtx`/`ToolView` ([[ADR-T08-1 Tool context and gesture overlay]]),
whose struct literals appear in many test fixtures; the renderer is called
from `shell::app`. Every shared change must happen once, in T16.

## Decision

- The editor keeps a `Helpers` value — `smart_snap`, `grid_snap`,
  `numbering` (all off by default), `next_number` (starts at
  `numbering::FIRST_NUMBER` = 1) and `grid_cols`/`grid_rows` (4 × 4) — as a
  field of `DrawStyle`. `DrawStyle` already means "settings for new shapes"
  and travels in `ToolCtx::style`/`ToolView::style`, so no tool fixture
  changes. `Editor::helpers()` reads it.
- Tools never write helper settings; only `Editor::apply` does. T19 advances
  and rolls back the counter in the editor (it owns that hook).
- `Overlay` gains `guides: Vec<[Vec2; 2]>`, world-space alignment guides
  (empty until T17).
- Logic entry points exist as identity stubs: `snap::snap_end`,
  `numbering::label_new`, `tools::grid::{on_pointer, preview, cancel}`.
- The renderer exposes two no-op hooks, `render::draw_underlay` (before
  shapes: dot grid) and `render::draw_guides` (after the overlay), already
  called by `shell::app`.

## Alternatives considered

- **A separate `helpers` field on `ToolCtx`/`ToolView`** — cleaner name, but
  touches every fixture in five tool modules for no behavioural gain.
- **Tools mutate the counter through `&mut`** — makes undo rollback of the
  counter a cross-module concern; keeping it in the editor keeps one owner.

## Consequences

- T17, T18 and T19 only fill stubs and their own files.
- `DrawStyle` is no longer just colour and width; its docs say so.
- Default-on helpers would need a change in `editor.rs` and a new ADR.

## Rollback plan

Move `Helpers` out of `DrawStyle` into its own context field (new ADR),
updating the fixtures; the stubs can be deleted with their tasks' reverts.
