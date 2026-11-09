---
id: ADR-T91-2
title: Scaled text stays within the largest text size
status: accepted
kind: decision
date: 2026-11-09
task: T91
builds_on: ["[[ADR-T74-1 Line-based editor snapshot format]]"]
amends: ["[[ADR-T91-1 Resize handles and scale rules]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T91-2 Scaled text stays within the largest text size

## Context

[[ADR-T91-1 Resize handles and scale rules]] lets a resize grow text
until a coordinate or the text's box stops being finite. The editor
already has a smaller limit: `snapshot::MAX_TEXT_SIZE`, the largest size
it can make, past which a snapshot refuses to load the text and typing
could grow a block's bounds past `f32`. The fuzzer found a resize that
went over it (`snapshot_round_trip_continues_equal`).

## Decision

A resize caps the size of text at `MAX_TEXT_SIZE`, for text blocks,
text in shapes and text in grid cells. The shape's points and box keep
scaling; only the font size stops growing. A size that would stop being
positive stays as it was.

With the size capped, a text block's box cannot overflow while its
origin is finite, so the check on the box from ADR-T91-1 stays only for
grids, whose index band can.

## Alternatives considered

- **Rejecting the whole resize at the limit** — the preview would vanish
  while dragging.
- **Limiting the factor of the whole selection** — one large text would
  stop every other shape from growing.
- **Raising `MAX_TEXT_SIZE`** — the limit protects typing and the
  snapshot format.

## Consequences

- Text at the limit stops growing while its shape keeps growing, so a
  very large resize changes the text's proportion to its box.
- The limit is reached only at sizes several times larger than the
  largest on-screen text.

## Rollback plan

Add a rollback ADR and remove the `min(MAX_TEXT_SIZE)` in
`Shape::scaled_about`; the fuzz test will then show why it was there.
