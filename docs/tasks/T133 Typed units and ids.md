---
id: T133
title: Typed units and ids
status: todo
wave: 66
branch: task/T133-typed-units-and-ids
depends_on: [T132]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T133 Typed units and ids

## Goal

Coordinates, ids and counts that cannot be mixed up are different types, so a whole class of conversion bugs stops compiling.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §3 and §4: `Point<World>` and `Point<Screen>`, `Id<Shape>`, `PageIx`, `Zoom`, `NonEmpty<T>`.
- Today screen, world and pixel values are all `Vec2`/`f32`.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `draw-types` defines `World`, `Screen` and `Px` units over `Vec2`, `Aabb` and `f32`, with the conversions only through `Camera`.
  Tests: conversion round trips; compile-fail tests (`trybuild`-style kept as `tests/compile_fail/*.rs` checked by a script, no new dependency) that mixing units does not compile.
- **AC-2** — Typed ids: `Id<Shape>` from a generational arena, `PageIx` (`0..PAGE_COUNT`), `ColorId` already validated; a dangling id cannot be created by the public API.
  Tests: arena reuse, stale id rejected, proptest over insert/remove sequences.
- **AC-3** — `Zoom` cannot be NaN or out of range, `NonEmpty<Point>` backs `Stroke`; snapshot parsing returns these types ('parse, don't validate').
  Tests: the snapshot and file fuzz tests keep passing; new cases for each rejected value.
- **AC-4** — `draw-core` is migrated to the types with no change of behaviour: all snapshot fixtures load, the e2e suite is green.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-types/`, `crates/draw-core/src/**` (type migration)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T133): specify typed units and ids`
- [ ] tests — `test(T133): add failing tests for the types`
- [ ] models — `feat(T133): add the unit and id types`
- [ ] behaviour — `refactor(T133): migrate the core to the types`
- [ ] quality — `chore(T133): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
