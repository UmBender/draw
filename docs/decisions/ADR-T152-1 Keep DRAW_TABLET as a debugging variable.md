---
id: ADR-T152-1
title: Keep DRAW_TABLET as a debugging variable
status: accepted
kind: decision
date: 2026-11-12
task: T152
builds_on: []
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T152-1 Keep DRAW_TABLET as a debugging variable

UNATTENDED CHOICE: taken by the batch run under the plan's recommended
default; the owner should review it.

## Context

[[T152 Command line and DRAW_* variables audit]] leaves `DRAW_TABLET`
(`auto`, `on`, `off`) to be decided there. Pen mode is picked
automatically from the tablet's reports; the variable only overrides a
wrong guess, which is a troubleshooting need, not a setting.

## Decision

`DRAW_TABLET` stays, documented and listed by `--help` as a debugging
variable like `DRAW_MSAA`. It is not a flag and not a preference.

## Alternatives considered

- **Remove it** — a mis-detected tablet would have no workaround.
- **Make it a flag** — the command line keeps only `--help`, `--version`
  and `--recover`.

## Consequences

The user documentation names it only in its Debugging section.

## Rollback plan

Delete `TABLET_ENV` in `src/shell/tablet.rs` and its row in the Debugging
section in a new ADR.
