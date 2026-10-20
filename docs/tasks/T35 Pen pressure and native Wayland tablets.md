---
id: T35
title: Pen pressure and native Wayland tablets
status: review
wave: 23
branch: task/T35-pen-pressure
depends_on: [T34]
adrs: ["[[ADR-T35-1 Vendored miniquad with a pen tablet patch]]", "[[ADR-T35-2 Pressure as per-point width factors on pen strokes]]"]
feature: "[[Pen tablets]]"
tutorial: "[[35 Reading a pen through a patched window library]]"
tags: [task]
---

# T35 Pen pressure and native Wayland tablets

## Goal

Pen strokes get thicker the harder the pen presses, and the pen draws on the
native Wayland backend too. miniquad 0.4 reports no tablet events at all, so
this needs a patched miniquad. That overrides
[[ADR-0014 Minimal dependencies and no unsafe]] for one vendored crate and
needs its own ADR.

## Known facts (2026-10-20, see [[T34 Pen tablet input]])

- miniquad 0.4.11 X11: Xinput2 is loaded only for `XI_RawMotion`;
  tablet valuators (pressure is axis 2 on the Xwayland tablet device) are
  never selected.
- miniquad 0.4.11 Wayland: no `zwp_tablet_manager_v2`; Hyprland then sends
  pen motion as pointer motion but no tip or button presses.
- The kernel reports `ABS_PRESSURE` and both stylus buttons for the
  HUION 420 pen; `/dev/input/event*` is not readable by the user, so evdev
  is not an option.

## Open questions (owner decides in the spec step)

1. Vendor and patch miniquad (`[patch.crates-io]`), or move to another
   window library?
2. Pressure → width: which curve, and does it apply only to the pen tool or
   to shapes too?
3. Stroke model: per-point widths in `Shape::Stroke` (document format,
   tessellation, budgets ADR-T24-4).

## Owner decisions (2026-10-20)

1. Vendor and patch miniquad 0.4.11 (`[patch.crates-io]`); macroquad
   stays the crates.io release.
2. A half press draws the chosen width: `factor = clamp(2p, 0.25, 2.0)`.
   Pen tool only; shapes, eraser and mouse strokes ignore pressure.
3. Per-point width factors in `Shape::Stroke` (`None` without pressure).

## Spec

Decisions: [[ADR-T35-1 Vendored miniquad with a pen tablet patch]],
[[ADR-T35-2 Pressure as per-point width factors on pen strokes]].

- **AC-1 — Curve.** `core::pressure::width_factor(p)` is
  `clamp(2p, MIN_WIDTH_FACTOR, MAX_WIDTH_FACTOR)` with `0.25` and `2.0`;
  non-finite pressure gives `1`.
  Tests (`src/core/pressure.rs`): `width_factor_half_press_is_one`,
  `width_factor_clamps_light_and_hard_presses`,
  `width_factor_non_finite_is_one`,
  `width_factor_is_monotonic_and_bounded` (proptest).
- **AC-2 — Input.** `InputEvent::Pressure { pressure }` sets the pressure
  of the following pointer events. The tap filter passes it unchanged
  without changing its state. The editor keeps it clamped to `[0, 1]`
  (non-finite → `None`), and setting it needs no redraw.
  Tests: `tap_filter_passes_pressure_without_changing_state`
  (`src/core/tap.rs`); `pressure_event_sets_the_pen_pressure`,
  `pressure_event_clamps_and_drops_non_finite_values`,
  `pressure_event_needs_no_redraw` (`src/core/editor.rs`).
- **AC-3 — Smoothing.** `Smoother::push_width(raw, width)` keeps a width
  factor per kept point once the first kept point had one; a missing
  factor repeats the last, the EMA smooths factors with the same α, and
  `widths()` / `finish_with_widths()` stay in step with the points.
  `simplify_rdp_widths` also keeps a point whose factor is more than
  `WIDTH_EPSILON` from the chord's interpolated factor.
  Tests (`src/core/smoothing.rs`):
  `smoother_without_widths_reports_none`,
  `smoother_missing_width_repeats_the_last`,
  `smoother_widths_follow_the_ema`,
  `smoother_widths_stay_in_step_with_points` (proptest),
  `rdp_widths_keeps_a_width_change_on_a_straight_line`,
  `rdp_widths_with_constant_widths_matches_rdp` (proptest).
- **AC-4 — Shape.** `Shape::Stroke { points, widths, style }`. A stroke
  is finite only if `widths` is `None` or has one finite positive factor
  per point; bounds and hit-testing use the largest factor
  (`Shape::max_width_factor`).
  Tests (`src/core/shape.rs`): `stroke_max_width_factor_without_widths_is_one`,
  `stroke_bounds_grow_with_the_widest_point`,
  `stroke_hit_reaches_a_thick_point`,
  `stroke_with_bad_widths_is_not_finite`.
- **AC-5 — Pen tool.** A press with pressure gives a stroke with one
  factor per point (`width_factor` of each point's pressure); without
  pressure, `widths` is `None`; points without pressure after a pressed
  start repeat the last factor; a tap dot takes the press's factor. Other
  tools ignore pressure.
  Tests: `pen_with_pressure_stores_a_width_per_point`,
  `pen_without_pressure_stores_no_widths`,
  `pen_light_and_hard_presses_give_thin_and_thick_points`,
  `pen_tap_dot_takes_the_press_factor`,
  `pen_points_without_pressure_keep_the_last_factor`
  (`src/core/tools/pen.rs`); `pressure_reaches_the_pen_tool`,
  `pressure_leaves_rect_width_alone` (`src/core/editor.rs`).
- **AC-6 — Rendering.** `Batch::polyline_widths` with a constant width
  gives exactly the mesh of `Batch::polyline`; with varying widths each
  strip pair is offset by its own half-width; each point's screen width is
  `screen_width(style.width × factor, zoom)`.
  Tests (`src/shell/render.rs`):
  `polyline_widths_with_constant_width_matches_polyline` (proptest),
  `polyline_widths_offsets_each_point_by_its_half_width`,
  `stroke_screen_widths_scale_by_factor_and_clamp`.
- **AC-7 — Pairing.** `Collector::set_pen_samples` queues pen samples;
  a pointer event takes the pressure of the oldest queued sample at its
  physical position (within 0.01 px) and drops older ones; unmatched
  events have no pressure; `InputEvent::Pressure` comes right before a
  pointer event whose pressure differs from the last one emitted.
  Tests (`src/shell/input_map.rs`):
  `collector_pairs_pen_samples_with_pointer_events`,
  `collector_emits_pressure_only_when_it_changes`,
  `collector_mouse_after_pen_clears_pressure`,
  `collector_drops_samples_older_than_the_match`.
- **AC-8 — Patched miniquad.** `vendor/miniquad` is miniquad 0.4.11
  plus the patch in `vendor/miniquad/PATCHED.md`: `miniquad::pen`
  (`PenSample`, `take_samples`), X11 pressure from the `Abs Pressure`
  raw valuator, Wayland `zwp_tablet_v2` (tip → left, `BTN_STYLUS` →
  middle, `BTN_STYLUS2` → right, motion, pressure). The app drains the
  samples into the collector every frame.
  Tests: `pen_samples_are_empty_without_a_window` (`src/shell/app.rs`);
  `cargo_patches_miniquad_with_the_vendored_copy` (`tests/packaging.rs`).
- **AC-9 — Fuzz.** Sessions with pressure events keep every invariant
  (each stroke's widths match its points and lie in
  `[MIN_WIDTH_FACTOR, MAX_WIDTH_FACTOR]`), and undo/redo stays symmetric.
  Tests (`tests/fuzz.rs`): `pressure_sessions_keep_stroke_widths_in_step`,
  `pressure_undo_all_then_redo_all_is_symmetric`.
- **AC-10 — Docs.** README *Pen tablets*: pressure, the curve, native
  Wayland support and its barrel buttons.
- **AC-11 — Owner check (manual).** On the target with the HUION 420, on
  both backends: light and hard presses give thin and thick strokes; mouse
  strokes look as before; on `DRAW_BACKEND=wayland` the pen draws, the
  barrel buttons pan and erase.

## Out of scope

- Tilt and rotation.
- Pressure for shapes, eraser size or selection.
- Pad (express key) buttons.

## Files owned

- `Cargo.toml`, `Cargo.lock`, `vendor/miniquad/` (new)
- `src/core/pressure.rs` (new), `src/core/mod.rs`, `src/core/input.rs`,
  `src/core/tap.rs`, `src/core/editor.rs`, `src/core/smoothing.rs`,
  `src/core/shape.rs`, `src/core/snap.rs` (test constructor),
  `src/core/tools/mod.rs`, `src/core/tools/pen.rs`,
  `src/core/tools/grid.rs`, `src/core/tools/shape_tool.rs`,
  `src/core/clipboard.rs` (test constructors)
- `src/shell/input_map.rs`, `src/shell/app.rs`, `src/shell/render.rs`
- `tests/fuzz.rs`, `tests/perf.rs`, `tests/packaging.rs`, `README.md`
- `docs/architecture/Architecture.md`
- new ADRs `ADR-T35-*`, feature note `docs/features/Pen tablets.md`,
  tutorial `docs/tutorials/35 Reading a pen through a patched window library.md`

## Subtasks (one commit each)

- [x] spec — `docs(T35): specify pen pressure acceptance criteria`
- [x] tests — `test(T35): add failing tests for pen pressure and stroke widths`
- [x] models — `feat(T35): add pressure, stroke width and pen sample types`
- [x] behaviour — `feat(T35): read pen pressure and draw strokes with per-point widths`
- [x] quality — `chore(T35): pass clippy and rustfmt`
- [x] docs — `docs(T35): add feature note and tutorial`

## Learning path

Step 35 — requires step 34 (telling a tap from a drag) and step 24
(layered redraws, the stroke strip).

## Log

- 2026-10-20 — Created with T34: the owner chose pen ergonomics for 1.2.0
  and pressure later.
- 2026-10-20 — Open: only one barrel button reaches Xwayland apps (as
  middle); the other sends nothing. `/proc/bus/input/devices` advertises
  both (`KEY=1c01`: `BTN_STYLUS`, `BTN_STYLUS2`). Next check:
  `sudo evtest /dev/input/event13`, pressing each button. If
  `BTN_STYLUS2` appears, it is lost in Hyprland/Xwayland (Hyprland only
  remaps the eraser button, `input:tablettool:eraser_button_*`), and a
  native tablet backend here would see it. If nothing appears, it needs a
  driver-side remap (udev hwdb / hid-uclogic). Hyprland has no option to
  turn tablet tip presses into pointer clicks for clients without
  `zwp_tablet_v2`.
- 2026-10-20 — Owner decisions taken (above); spec written.
- 2026-10-20 — Spec, tests, models, behaviour, quality committed on
  `task/T35-pen-pressure`. `src/core/clipboard.rs` was added to *Files
  owned*: its test helper builds a `Pointer`. Deviations from the spec:
  `arb_pressure` in `tests/fuzz.rs` decodes one byte instead of a
  `prop_oneof!`, and the pressure session cycles 1–15 pressures: nested
  unions overflowed the 2 MB test-thread stack in debug builds.
  `simplify_rdp` now shares `rdp_keep` with `simplify_rdp_widths`: a span
  is split on geometry first, then on width, so constant widths give
  exactly the old result (proptest).
- 2026-10-20 — Smoke runs on the target: both backends start; with
  `WAYLAND_DEBUG=client` the native backend binds `zwp_tablet_manager_v2`,
  gets the tablet seat and the HUION 420 tablet, pad and pad group with
  no protocol error. The pen tool object only appears once the pen is in
  proximity, so the pen path is part of AC-11.
- 2026-10-20 — `scripts/check.sh` green. The vendored crate is not a
  workspace member, so fmt and clippy skip it; its two upstream warnings
  (`set_or_replace_display` unused, `static_mut_refs`) show in builds.
  `docs/architecture/Architecture.md` updated (`core::pressure`,
  `vendor/miniquad`, pairing in the data flow). ADRs accepted. AC-11
  (owner check with the pen, both backends) pending. Status → review.
- 2026-10-20 — Owner check: pressure works. The owner reported the two
  upstream miniquad warnings in the build (Cargo does not cap lints for a
  path dependency). Fixed in the vendored copy (`PATCHED.md`): the
  Android-only `set_or_replace_display` is compiled only for Android, and
  the X11 clipboard reads `MESSAGE` through `addr_of!`.
