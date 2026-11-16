---
id: ADR-T168-2
title: Terminate the EGL display of a Wayland connection
status: accepted
kind: decision
date: 2026-11-15
task: T168
builds_on: [ADR-T145-1, ADR-T168-1]
amends: [ADR-T145-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T168-2 Terminate the EGL display of a Wayland connection

## Context

`egl.rs` never called `eglTerminate`: EGL shares one display per platform and native handle, and terminating the shared headless display would pull it from under other contexts. On Wayland the display is made from a `wl_display*`. After `wl_display_disconnect`, a new connection can get a `wl_display` at the same address, and Mesa returns its cached `EGLDisplay` with proxies of the dead connection. With several tests in one process this failed about 1 run in 35 with `wl_display#1: error 0: invalid object 11`.

## Decision

An `EglContext` made by `from_window` for `Platform::Wayland` terminates its display on drop (and on the failure paths of its construction), after the surface and context are destroyed. The Wayland `Window` drops the GL part before the connection (`Shared`), so `eglTerminate` always runs before `wl_display_disconnect`. The shared headless display is still never terminated. X11 is unchanged.

## Consequences

- The display of a connection is private to that connection, so terminating it affects no other context.
- Regression test: `wayland_many_connections_in_sequence_and_threads`.
- Amends the "never terminated" rule of ADR-T145-1 for native-window Wayland displays only.
