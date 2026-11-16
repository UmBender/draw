---
id: ADR-T168-1
title: Generated Wayland protocol code and surface type states
status: accepted
kind: decision
date: 2026-11-15
task: T168
builds_on: [ADR-T142-1, ADR-T127-2, ADR-T127-3]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T168-1 Generated Wayland protocol code and surface type states

## Context

`draw-sys` talks to a Wayland compositor through `libwayland-client` loaded with `dlopen`, with no new crate dependency. The protocol needs `wl_interface` tables and typed request and event wrappers for each interface. The owner decided the code is generated from the protocol XML and checked in.

## Decision

- The generator lives in `crates/draw-sys/tests/wayland_generator.rs` (a minimal XML reader for the protocol subset, plus the emitter). A test regenerates the code in memory and compares it with `src/wayland/generated/*.rs`; an `#[ignore]` test (`write_generated_protocol`) writes the files. The build never reads the XML.
- Generated per interface: the `wl_interface` static (with `wl_message` tables), the `#[repr(C)]` listener struct (one `unsafe extern "C" fn` per event, in XML order), `add_listener`, and one `unsafe fn` per request over `wl_proxy_marshal_flags`. Interfaces outside the generated set (for example `wl_buffer`, `xdg_positioner`) are `*mut Proxy` arguments with a null interface entry; requests that would create such objects get no wrapper.
- The functions of `libwayland-client` are loaded once into a `Client` function table through the T142 `loader`; the interface statics are our own.
- The window is a type state `Window<S>` with `S` one of `Created`, `Configured`, `Mapped`. Only `Created` has the initial bare commit and `ack_configure` (consuming `self`, so acking twice does not compile); `commit` and `attach` exist from `Configured` on, via a sealed `Attached` marker. Compile-fail tests in `crates/draw-sys/tests/compile_fail/` run in `scripts/compile_fail.sh`.
- `RawEvent` has no close event and `platform.rs` is not owned by this task: the close request is `WaylandPlatform::close_requested()`, and T171 decides how `main` reads it.

## Alternatives considered

- **`wayland-scanner` or `wayland-client` crates** — a new dependency (ADR-T127-3).
- **Hand-written wrappers** — the nine interfaces are error prone by hand and T169 to T170 add more.
- **A `build.rs` generator** — the owner chose checked-in code so the build stays free of build scripts.

## Consequences

Protocol additions are a regeneration plus a diff review. The generated file is large but never edited by hand.

## Rollback plan

Revert the T168 commits; no other crate depends on `draw_sys::wayland` yet.
