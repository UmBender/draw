---
id: T170
title: Wayland tablet and clipboard
status: review
wave: 72
branch: task/T170-wayland-tablet-and-clipboard
depends_on: [T169]
adrs: [ADR-T170-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T170 Wayland tablet and clipboard

## Goal

Slice 3 of [[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]]: `zwp_tablet_v2` pen input (pressure, tilt, eraser, proximity) and the clipboard (text and `image/png`) through `wl_data_device`.

## Known facts (2026-11-15)

- Split out of T143 by the owner. Builds on [[T169 Wayland keyboard, pointer and seat]].
- Patches to port (`vendor/miniquad/PATCHED.md`): tablet pressure (T35), the pointer dropped while a tool is in proximity ([[ADR-T40-1 Tablet stream wins over copied wl_pointer input]], `pointer_shadowed_by_pen`), barrel-button mapping (`fix/pen-barrel-takeover`), tip bounce (`fix/pen-tip-bounce`).
- `tablet-v2.xml` (now in `wayland-protocols/stable/tablet/`, same `zwp_tablet_*_v2` interfaces; copied into `crates/draw-sys/protocols/` with T168) is added to the generator's inputs ([[T168 Wayland connection, window and protocol states]]).
- Headless sway has no virtual tablet: tablet tests decode recorded event sequences; the owner checks a real tablet by hand at review.
- Clipboard transfers pass file descriptors (pipes): an `unsafe` primitive under [[ADR-T127-2 The unsafe policy]]. `wl-copy`/`wl-paste` drive the integration tests.

## Spec

Decisions: [[ADR-T170-1 Wayland pen frames and clipboard transfers]].
Pure code in `wayland/pen.rs` and `wayland/transfer.rs`; tests in
`tests/wayland_tablet.rs` and `tests/wayland_clipboard.rs`.

- **AC-1** — Tablet: tools, proximity, tip, pressure, tilt, eraser, barrel buttons, frames to pen samples; pointer shadowed while a tool is in proximity. Tests on recorded sequences: `pen_frame_motion_sends_sample_then_move`, `pen_frame_tip_change_sends_left_button_once`, `pen_frame_out_of_proximity_drops_events`, `pen_proximity_out_lifts_a_tip_that_is_down`, `pen_pressure_is_normalised_and_clamped`, `pen_barrel_buttons_map_to_middle_and_right`, `pen_tilt_is_kept_and_finite`, `pen_tool_kind_reads_the_wire_type`, `pens_shadow_the_pointer_while_a_tool_is_in_proximity`, `wayland_tablet_manager_is_bound_without_tablets`; regressions `regression_T35_pressure_reaches_the_sample`, `regression_T40_pointer_shadowed_in_proximity`, `regression_pen_barrel_takeover`, `regression_pen_tip_bounce`. These are ports of vendored patches, so they have no snapshot files.
- **AC-2** — Clipboard: read and write text, write `image/png` (T115), large transfers. Pure: `text_mime_prefers_utf8_over_legacy_names`, `text_mime_none_without_a_text_type`, `png_offer_is_detected`, `source_mimes_follow_the_payload`, `read_limited_stops_at_the_limit`, `write_payload_ignores_a_closed_reader`. Integration with `wl-copy`/`wl-paste`: `wayland_clipboard_reads_text_from_wl_copy`, `wayland_clipboard_text_reaches_wl_paste`, `wayland_clipboard_png_reaches_wl_paste`, `wayland_clipboard_large_text_round_trips`, `wayland_clipboard_empty_is_none`.
- **AC-3** — Decoders fuzzed with the xorshift generator of T169 (no new dependency): `fuzz_pen_decoders_never_panic`, `fuzz_transfer_decoders_never_panic`.
- **AC-4** — Hand check with a tablet (owner, at review), recorded in the Log.

## Out of scope

- `main` and e2e ([[T171 Wayland platform in main and the e2e suite]]); primary selection.

## Files owned

- `crates/draw-sys/src/wayland/**`, `crates/draw-sys/protocols/**`, `crates/draw-sys/tests/wayland*.rs`, `crates/draw-sys/tests/common/sway.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T170): specify the slice`
- [x] tests — `test(T170): add failing tests for the tablet and clipboard`
- [x] models — `feat(T170): add the tablet and transfer types`
- [x] behaviour — `feat(T170): implement the tablet and clipboard`
- [x] quality — `chore(T170): pass clippy and rustfmt`
- [ ] tablet by hand — owner, recorded in the Log

## Log

- 2026-11-15 — split from T143 by the owner.
- 2026-11-16 — implemented in a batch run on top of T169. `scripts/check.sh` green. Clipboard text and `image/png` verified against `wl-copy`/`wl-paste` in a private sway (4 MiB both ways). The pen is covered by recorded-sequence tests only.
- 2026-11-16 — UNATTENDED CHOICE: edited `crates/draw-sys/src/wayland/{seat,connection,platform,mod}.rs` (wiring, serial tracking, pointer shadowing, destroy order) and `tests/wayland.rs` (the T168 test `wayland_platform_input_parts_return_nothing` asserted an empty clipboard; renamed and trimmed because the clipboard works now). Both are inside `crates/draw-sys`.
- 2026-11-16 — UNATTENDED CHOICE: the tablet manager is bound at version 1 (version 2 only adds pad dials, which are not generated); tablets and pads are destroyed as soon as they are announced, only tools are used. Generator now also emits `tablet_v2` and the `wl_data_*` interfaces.
- 2026-11-16 — the pen regressions (T35, T40, barrel, tip bounce) are ports of vendored patches: unit tests on the pure `PenTool`, no snapshot files.
- LEFTOVER: tilt and tool kind (eraser) are kept in `PenTool` but `RawEvent::Tablet` has no fields for them; proposed task: widen the event when the editor uses them.
- LEFTOVER: cursor shape while a pen is in proximity (`wp_cursor_shape_v1`, as the vendored code did); proposed task: with T171.
- LEFTOVER: primary selection and drag and drop stay out of scope.
- AC-4 (real tablet by hand) is the owner's: plug a tablet, draw with pressure, use both barrel buttons, check the mouse does not draw while the pen hovers.
