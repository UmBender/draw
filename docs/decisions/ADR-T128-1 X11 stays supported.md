---
id: ADR-T128-1
title: X11 stays supported
status: accepted
kind: rollback
date: 2026-11-12
task: T128
builds_on: [ADR-T28-1, ADR-T44-1]
amends: []
supersedes: []
reverts: [ADR-T46-1]
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T128-1 X11 stays supported

## Context

[[ADR-T46-1 Native Wayland by default and X11 deprecated]] made native
Wayland the default and declared X11 deprecated, to go away in 2.0.0. The
owner decided on 2026-11-12 that users on X11 must keep working. This
reverses goal 1 of [[T50 Version 2.0.0 refactor]].

## Decision

This reverts the deprecation part of ADR-T46-1 only:

- X11 (plain X11 and XWayland) is a supported backend: tested, documented
  and not scheduled for removal.
- The Wayland-first default stays: native Wayland on Wayland sessions,
  X11 elsewhere or as the fallback.
- `DRAW_BACKEND` and the Flatpak `x11` socket stay.
- No deprecation line is printed on X11 starts.

## Alternatives considered

- **Keep the deprecation and delay removal** — still tells X11 users they
  are on borrowed time.
- **Make X11 the default again** — loses the native Wayland gains of
  ADR-T46-1 for no benefit to X11 users.

## Consequences

Both backends need end-to-end coverage ([[T129 Release gate with end-to-end on X11 and Wayland]]).
The own backend of 2.0 must keep an X11 path.

## Rollback plan

A new ADR that deprecates X11 again, plus `git revert` of the T128 commits.
