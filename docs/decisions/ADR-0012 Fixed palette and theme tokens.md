---
id: ADR-0012
title: Fixed palette and theme tokens
status: provisional
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0004]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0012 Fixed palette and theme tokens

## Context

The user wants only a few colours and will supply a UI theme. Code must not be
blocked waiting for it.

## Decision

- Shapes store a **palette index** (`ColorId(u8)`, `0..6`), not an RGBA value,
  so a theme change recolours existing shapes and the model stays tiny.
- `core::palette` defines the 6 drawing colours and the UI tokens (`bg`,
  `surface`, `border`, `text`, `accent`) as `const` RGBA values in one place.
- Values are **provisional** (see [[Theme]]) until the user's theme arrives;
  then a new ADR amends this one with the real values. No other module
  hard-codes colours.

## Alternatives considered

- **Free colour picker** — explicitly unwanted.
- **Storing RGBA per shape** — larger shapes and no global recolour.

## Consequences

- Swapping the theme touches exactly one file.

## Rollback plan

Theme replacement = amending ADR + edit of `core::palette` and [[Theme]].
