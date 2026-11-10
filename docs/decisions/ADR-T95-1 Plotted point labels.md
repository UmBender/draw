---
id: ADR-T95-1
title: Plotted point labels
status: accepted
kind: decision
date: 2026-11-09
task: T95
builds_on: [ADR-T93-1]
amends: [ADR-T93-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T95-1 Plotted point labels

## Context

A coordinate plane drawn with `Ctrl+P` ([[ADR-T93-1 Coordinate plane key and plotting rules]]) shows where points are but not which is which, and geometry problems name them. The grammar accepted exactly two numbers per line.

## Decision

- **Grammar.** A non-blank line is `x y`, optionally followed by a label: everything after the second number, without its outer whitespace. It may contain spaces. This replaces "exactly two numbers" in ADR-T93-1; the other refusals are unchanged.
- **Drawing.** A label is a text block at the block's text size, in its colour. The bottom-left of its box is a small gap up and to the right of the dot. Labels come after the points in the z-order.
- **Frame.** Labels take no part in the step or the ticks, so the same points give the same axes with or without them. The top-left of everything drawn, labels included, is the block's top-left.

## Alternatives considered

- **One-word labels** — a name like `top left` would be refused for no reason.
- **Growing the frame around labels** — the same points would get different axes.
- **Avoiding overlaps between labels** — needs a layout pass, and the labels can be dragged after the plot is made.

## Consequences

A third number on a line was an error and is now a label. Two close points can have overlapping labels.

## Rollback plan

Revert the T95 commits and record it in a rollback ADR. Text blocks already drawn stay as they are.
