---
id: T162
title: RawEvent and the input collector
status: done
wave: 68
branch: task/T162-rawevent-and-the-input-collector
depends_on: [T140, T161]
adrs: [ADR-T162-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T162 RawEvent and the input collector

## Goal

Slice 1 of [[T156 Platform migration of input and the loop]]: `RawEvent` carries everything the shell reads from miniquad's input, and `Collector`, `HeldMods`, `KeyWatch`, `map_key` and `diag` consume `RawEvent`/`RawKey`/`RawMods` instead of miniquad's `KeyCode`/`KeyMods`/`BackendRecord`. A thin translation from miniquad's events to `RawEvent` keeps the app working.

## Known facts (2026-11-13)

- Split out of [[T156 Platform migration of input and the loop]] after the batch blocked it (too large for one session; findings in the T156 Log). The traits are in [[ADR-T140-1 The Gpu and Platform traits]]; the filter and readback calls in [[ADR-T159-1 Texture filter and readback in the Gpu trait]].
- `vendor/miniquad` is not changed. No pixel change allowed.
- `input_map.rs` is 1,736 lines, about 1,200 of tests driven by miniquad events.
- Owner decision (batch plan): the diag records travel as `RawEvent::Backend(BackendRecord)` with the record types copied into `draw-sys` (one ordered stream, lossless), in an ADR amending ADR-T140-1. The pen pairing (`pair_pen`) must stay lossless too; the shape is the session's choice, recorded.

## Spec

Decisions are in [[ADR-T162-1 RawEvent carries the backend records and the pen samples]].

- **AC-1** — `RawEvent` has a `Backend(BackendRecord)` variant; `RawEvent::Tablet` carries the pen samples the pairing needs; `draw-sys` holds `Device`, `BackendEvent`, `BackendRecord`, `DeviceChange` and the pure helpers `device_change`, `has_release_request` and `pointer_shadowed_by_pen`.
  Tests (`crates/draw-sys/src/platform.rs`): `device_change_announced_without_device_creates`, `device_change_announced_with_device_keeps`, `device_change_withdrawn_with_device_releases`, `device_change_withdrawn_without_device_keeps`, `has_release_request_needs_version_three`, `pointer_shadowed_by_pen_with_a_tool_in_proximity_is_true`, `pointer_shadowed_by_pen_without_tools_in_proximity_is_false`, `raw_event_backend_keeps_the_record`.
  Collector (`input_map.rs`): `collector_tablet_event_pairs_pressure_with_pointer_event`, `collector_backend_event_is_recorded_at_its_time`, `collector_backend_focus_event_forgets_held_keys`.
- **AC-2** — `Collector`, `HeldMods`, `KeyWatch`, `map_key` and `diag::event::key_tag` take `RawEvent`/`RawKey`/`RawMods`; no miniquad type in `input_map.rs` or `diag/` outside `src/shell/platform_mq.rs`.
  Tests (`platform_mq.rs`): `raw_key_letters_and_digits_map_to_their_variants`, `raw_key_unlisted_key_keeps_its_code`, `raw_key_keypad_enter_is_enter`, `raw_mods_drop_logo`, `raw_button_unknown_is_other`, `mq_input_replays_events_in_order`.
- **AC-3** — The migrated `input_map` and `diag` tests drive `RawEvent` (through the test helpers in `platform_mq`) and keep their assertions word for word; the same tests as before still pass.
- **AC-4** — `e2e.sh --backend all` green.

## Out of scope

- `app::run`, the clipboard, the window and the loop ([[T163 macroquad Platform interpreter and the loop]]); the `MqGpu` thread-local and `main` ([[T164 Generic run over Platform and Gpu]]).

## Files owned

- `crates/draw-sys/src/platform.rs` and `crates/draw-sys/tests/`
- `src/shell/input_map.rs`, `src/shell/diag/`, `src/shell/tablet.rs`
- a new translation module from miniquad events to `RawEvent` in `src/shell/` (e.g. `platform_mq.rs`), `src/shell/mod.rs`, `src/shell/recording.rs`
- call sites in other `src/shell/` files and in `tests/`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T162): specify the slice`
- [x] tests — `test(T162): add failing tests`
- [x] models — `feat(T162): add the types`
- [x] behaviour — `refactor(T162): migrate the slice`
- [x] quality — `chore(T162): pass clippy and rustfmt`

## Log

- 2026-11-13 — done, `scripts/check.sh` green, `e2e.sh --backend all` green (30 cases). Decisions in [[ADR-T162-1 RawEvent carries the backend records and the pen samples]]. Deviations: the migrated tests drive `RawEvent` through `platform_mq::keys` (`KeyCode` constants and a `Drive` trait with miniquad's method names), so their text is unchanged except `MouseButton::Unknown` → `Other` and `logo` fields; `map_mods_drops_logo` moved to `platform_mq::raw_mods_drop_logo` (`map_mods_keeps_shift_ctrl_and_alt` stays). `Collector` still receives pen samples through `set_pen_samples` from `app.rs`; `RawEvent::Tablet` is handled too, for T163. `FrameRecorder` still drains backend records with `diag::take_backend`; `RawEvent::Backend` is handled by the collector for T163. Dump names of letters, digits and unlisted keys changed (`Letter('Z')`, `Other(n)`). The quality step changed only formatting.

- 2026-11-13 — split from T156 by the owner, following the batch's recommendation.
