---
id: ADR-T127-1
title: The 2.0 architecture
status: proposed
kind: decision
date: 2026-11-12
task: T127
builds_on: [ADR-0003]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, 2.0]
---

# ADR-T127-1 The 2.0 architecture

## Context

[[2.0 Architecture proposal]] sets out the 2.0.0 refactor. Owner decisions of
2026-11-12, quoted from §1 of the proposal:

- "**The release depends on the end-to-end suite**, on X11 and on Wayland."
- "**Threads are welcome only with a measured gain.**"
- "**All the quality proposals are accepted.**"
- "**No dynamic dispatch, as far as possible**, and compile-time techniques
  (tagless final, GADT-like encodings, type states) to catch bugs early."
- "**The control engine is an automaton.**"
- "2.0.0 adds no features": the user sees the same app, faster and with
  fewer bugs.

The measured starting point is [[2.0 baseline]]: `editor.rs` has 10470 lines
and 383 tests, `tex.rs` 10972 and 281, `core` never imports `shell`, 19 files
mention macroquad, `src` has 11 `dyn` lines and no `unsafe`.

## Decision

1. **A cargo workspace** with `unsafe` enforced per crate:
   `draw-types` (units, typed ids; `forbid`), `draw-core` (document, tools,
   TeX, automaton; `forbid`), `draw-gfx` (retained renderer; `deny`, allowed
   only in `gpu/gl.rs`), `draw-sys` (platform; `unsafe` allowed under
   [[ADR-T127-2 The unsafe policy]]) and `draw` (wiring; `forbid`).
2. **The control engine is a statechart with a pure transition function**,
   `step : (Machine, Input) -> (Machine, Vec<Effect>)`: input and effects are
   closed enums, time is an input (`Tick`), modal overlays are states and not
   flags, transitions are `match`es with no wildcard arm, and a reference
   model checks the machine under `proptest`. The diagram is generated from
   the transition table and a test fails when the docs differ.
3. **Static dispatch.** Edges are traits with interpreters (`Platform`,
   `Gpu`); the program is monomorphised per backend, chosen once at start.
   A recording and a null interpreter serve the tests and the shell fuzz. A
   test fails on `dyn` or `Box<dyn` outside an allow-list kept in an ADR.
   Compile-time techniques are used where Rust supports them (typed
   requests, type states, indexed ids, units) and an exhaustive `enum` where
   a GADT would cost more than it saves.
4. **The release gate** runs `check.sh --fuzz` and `e2e.sh` on X11 and
   Wayland. Coverage has a ratchet in CI. `editor.rs` dissolves into the
   automaton's regions and `tex.rs` splits into parser, layout, metrics and
   glyph tables.
5. **Strangler order**: quality rails first, then types and automaton with
   the old `Editor` as an adapter, then the traits and the recording
   platform on macroquad, then the new backend, which replaces macroquad
   only when it passes the same e2e suite.

## Alternatives considered

- **Keep one crate with modules** — the rule "no `unsafe` here" would stay a
  habit instead of a compiler check.
- **`dyn Platform`** — simpler, but the owner asked to avoid dynamic
  dispatch and there are only two backends.
- **Rewrite in one step** — the app must keep working at each commit.

## Consequences

- Easier: behaviour is walkable by tests (model, reachable-state search);
  `editor.rs` and `tex.rs` shrink; wrong-unit and wrong-id bugs stop
  compiling.
- Harder: a large epic ([[T50 Version 2.0.0 refactor]], T127 to T152); two
  monomorphisations grow compile time and binary size (limit of
  [[ADR-T82-1 Shipped binary limit of 20 MiB]] applies).
- Watch: the frame-time table of the baseline is empty until measured on
  the i5-8250U.

## Rollback plan

Each step ships green behind the old code. Before the cutover of T149,
revert the task's commits and record a rollback ADR. After it, restore
macroquad from history with a new ADR; the core crates are unaffected.
