---
id: T133
title: Typed units and ids
status: in-progress
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

Scope and the non-breaking approach: [[ADR-T133-1 Typed layer added beside the public API]].

- **AC-1** — `draw-types` defines `World`, `Screen` and `Px` units over `Vec2`, `Aabb` and `f32` (`Point<U>`, `Rect<U>`, `Len<U>`), with the conversions only through `Camera`.
  Tests: `units_point_round_trip_through_camera`, `units_rect_visible_world_matches_vec2_api`, `units_len_scales_with_zoom` (draw-core `camera.rs`); `units_point_arithmetic_keeps_unit`; compile-fail files `mix_world_and_screen_point.rs`, `add_screen_to_world_len.rs`, `cast_between_units.rs`, checked by `scripts/compile_fail.sh` (with a compile-pass control).
- **AC-2** — `Arena<T>`/`Id<T>` generational, `PageIx` (`0..PAGE_COUNT`), `ColorId` already validated; a dangling id cannot be created by the public API (`Id` has no public constructor).
  Tests: `arena_insert_after_remove_reuses_slot_with_new_generation`, `arena_stale_id_is_rejected`, `arena_ids_of_other_arena_slot_do_not_alias_after_reuse`, `page_ix_rejects_out_of_range`, `page_ix_round_trips_index`, proptest `arena_matches_model_over_insert_remove_sequences`; compile-fail `forge_id.rs`.
- **AC-3** — `Zoom` cannot be NaN or out of range, `NonEmpty<T>` rejects empty input; the snapshot reader parses stroke points through `NonEmpty`; `Camera` stores a `Zoom`.
  Tests: `zoom_new_rejects_nan`, `zoom_new_rejects_out_of_range`, `zoom_new_accepts_bounds`, `zoom_clamped_maps_non_finite_to_one`, `nonempty_from_vec_rejects_empty`, `nonempty_pop_keeps_one_element`, `nonempty_first_last_are_total`; the existing snapshot and file fuzz tests keep passing; `snapshot_stroke_without_points_is_rejected` (existing behaviour kept).
- **AC-4** — `draw-core` behaviour is unchanged: all snapshot fixtures load, the existing suite is green; `Camera` keeps its `Vec2` methods and gains typed ones. The e2e suite is not run in the unattended batch (no display), the shell is untouched.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-types/`, `crates/draw-core/src/**` (type migration)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T133): specify typed units and ids`
- [ ] tests — `test(T133): add failing tests for the types`
- [ ] models — `feat(T133): add the unit and id types`
- [ ] behaviour — `refactor(T133): migrate the core to the types`
- [ ] quality — `chore(T133): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
