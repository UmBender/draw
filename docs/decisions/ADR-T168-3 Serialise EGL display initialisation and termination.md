---
id: ADR-T168-3
title: Serialise EGL display initialisation and termination
status: accepted
kind: decision
date: 2026-11-16
task: T168
builds_on: [ADR-T168-2]
amends: [ADR-T168-2]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T168-3 Serialise EGL display initialisation and termination

## Context

After ADR-T168-2 the test process still crashed with `SIGSEGV` about 1 run in 20. A core dump showed one thread in `eglInitialize` (Mesa `dri2_initialize_wayland`, zink, `vkGetInstanceProcAddr` jumping to an unmapped address) while another thread was in `__cxa_finalize`, a `dlclose` inside `eglTerminate`, unloading the Vulkan driver. Mesa's display set-up and tear-down are not safe to run at once in one process.

## Decision

`egl.rs` has one process-wide `static Mutex<()>` (`display_lock`). It is held around every `eglInitialize` (window and headless displays) and around the whole drop of an `EglContext` (unbind, destroy surface and context, `eglTerminate`). A poisoned lock is still taken, since it guards no data.

## Consequences

- The app has one connection, so the lock costs nothing at run time.
- Drop and initialisation of contexts in different threads wait for each other; rendering is not affected.
- Regression test: `wayland_many_connections_in_sequence_and_threads`, run in a loop (see the task Log).
