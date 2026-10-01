---
id: T15
title: Anti-aliasing
status: review
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

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality (no changes needed) · [x] docs

## Learning path

Step 15 — requires step 12.

## Log

- 2026-10-01 — Spec: miniquad's MSAA support flag
  (`features.resolve_attachments`) is only reachable via `unsafe fn
  get_internal_gl`, which ADR-0014 forbids. The original AC-2 ("fallback from
  the reported capability") became a `DRAW_MSAA` override parsed by a pure
  function; recorded in [[ADR-T15-1 MSAA on the cached frame]].
- AC-3 tests cover the existing `physical_size` helper (it had no tests).
- Smoke run on this machine (KDE Wayland, Mesa): the release binary starts and
  keeps running with `DRAW_MSAA` unset, `off` and `8`; with 4× the window
  renders correctly (orientation, toolbar). Empty canvas, so edge quality
  could not be judged automatically.
- **AC-4 checked by the user on the target**: edges are smooth; idle CPU ≈ 0 %.
  Spamming circles costs ≈ 20 % CPU with 4× MSAA vs ≈ 8 % with
  `DRAW_MSAA=off` — accepted for now. Improvement options (with and without
  `unsafe`) are collected in [[Rendering performance options]].
- Files owned: `docs/architecture/Rendering performance options.md` was added
  at the user's request (new note, outside *Files owned*; nothing else edits
  it).
- Quality: `scripts/check.sh` was green right after the behaviour commit, so
  no quality commit.
- For the integrator: add [[ADR-T15-1 MSAA on the cached frame]] to the
  Decision Log (builds on ADR-0006, ADR-T12-1, ADR-0014), [[Anti-aliasing]]
  to the Feature Index and step 15 to the Learning Path.
