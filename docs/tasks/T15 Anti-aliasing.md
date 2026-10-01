---
id: T15
title: Anti-aliasing
status: in-progress
wave: 7
branch: task/T15-antialiasing
depends_on: [T12]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-T12-1 Blocking event loop with cached frame]]", "[[ADR-T15-1 MSAA on the cached frame]]"]
feature: "[[Anti-aliasing]]"
tutorial: "[[15 Anti-aliasing and multisampling]]"
tags: [task]
---

# T15 Anti-aliasing

## Goal

Strokes, circles and diagonal lines are drawn with smooth edges instead of
stair-stepped pixels, without breaking the idle-friendly loop or the cached
frame (ADR-T12-1).

## Spec

Decision: [[ADR-T15-1 MSAA on the cached frame]]. All tests are unit tests in
`shell::app::tests` on pure helpers (no window needed).

- **AC-1** — The cached scene render target is created with
  `frame_target_params(samples)`: `sample_count = samples`, no depth buffer.
  macroquad resolves it into a plain texture, so the blit is unchanged.
  *Tests:* `frame_target_params_uses_sample_count`,
  `frame_target_params_has_no_depth`.
- **AC-2** — `msaa_samples(setting: Option<&str>) -> i32` reads the
  `DRAW_MSAA` override: unset → `DEFAULT_MSAA_SAMPLES` (4); `"2"`, `"4"`,
  `"8"` → that count; `"1"`, `"0"`, `"off"` (any case, surrounding
  whitespace ignored) → 1 (multisampling off); anything else → default.
  *Tests:* `msaa_samples_unset_is_default`, `msaa_samples_valid_counts_are_used`,
  `msaa_samples_off_values_disable`, `msaa_samples_invalid_is_default`,
  `msaa_samples_is_always_supported_count` (proptest over any string).
- **AC-3** — The blit keeps the 1:1 physical-pixel mapping: the resolved
  texture uses `FRAME_FILTER = FilterMode::Nearest` and is sized by
  `physical_size` (DPI-scaled, at least 1×1, NaN-safe).
  *Tests:* `frame_filter_is_nearest`, `physical_size_scales_by_dpi`,
  `physical_size_is_at_least_one_pixel`, `physical_size_bad_dpi_uses_one`.
- **AC-4** — Manual check on the target (Wayland, i5-8th gen iGPU): edges are
  smooth at zoom 0.05, 1 and 20; idle CPU stays ≈ 0 %; drawing latency not
  noticeably worse; `DRAW_MSAA=off` shows the old aliased edges. Results in
  the log.

## Out of scope

Text anti-aliasing (handled by the font rasterizer), per-shape AA toggles.

## Files owned

`src/shell/app.rs`, new `ADR-T15-*`.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 15 — requires step 12.

## Log
