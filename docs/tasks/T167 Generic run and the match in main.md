---
id: T167
title: Generic run and the match in main
status: todo
wave: 68
branch: task/T167-generic-run-and-the-match-in-main
depends_on: [T166]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T167 Generic run and the match in main

## Goal

Slice 3 of [[T164 Generic run over Platform and Gpu]]: `fn run<P: Platform, G: Gpu>()`, `main` picks the interpreters with a `match`, and a source scan keeps macroquad inside its interpreters except for a written allowlist that [[T149 Cutover removing miniquad and macroquad]] empties. Ends with the tablet check by hand (T156 AC-4).

## Known facts (2026-11-13)

- Split out of [[T164 Generic run over Platform and Gpu]] after the batch blocked it (findings in the T164 Log).
- After T166 the loop owns the `RenderCtx` and no thread-local holds the `Gpu`, but `run_on` still names `MqGpu`.
- Owner decision (batch plan): the scan has an allowlist instead of migrating everything here. Outside `platform_mq.rs` and `gpu_mq.rs` the shell still uses `macroquad::color::Color` (~80 uses: `toolbar.rs`, `sheet.rs`, `command_palette.rs`, `minimap.rs`, `ui_shapes.rs`, `render.rs`), `Font`/`load_ttf_font_from_bytes`/`measure_text` in `ui_text.rs`, `Window::from_config`, `Conf` and the signals in `main.rs`/`app.rs`, and `LinuxBackend` in `backend.rs`. These stay, listed in the test, each naming T149; the list may only shrink. Comments and docs do not count.
- AC-4 is the owner's: the batch stops at `review` with AC-1 to AC-3 done.
- `vendor/miniquad` is not changed. No pixel change allowed.

## Spec

Draft: the spec step refines each criterion into named tests.

- **AC-1** — `app::run` is generic over `P: Platform` and `G: Gpu`; `main` monomorphises it through a `match` (one arm, macroquad, until T143/T144). Tests: `main_runs_generic_run_through_a_match` (source scan) and a few frames of the app over `RecordingPlatform` and `RecordingGpu`.
- **AC-2** — No miniquad or macroquad path outside `platform_mq.rs` and `gpu_mq.rs` except the allowlist above. Test: `shell_sources_outside_the_mq_interpreters_name_no_macroquad` (the T161 scan widened), which also fails when an allowlist entry is no longer needed.
- **AC-3** — `e2e.sh --backend all` green; no pixel change.
- **AC-4** — Hand check with a tablet (pen pressure, tilt, eraser, seat changes), recorded in the Log by the owner (T156 AC-4).

## Out of scope

- Replacing `Color`, the menu font measure, the window config, the signals and `LinuxBackend` ([[T149 Cutover removing miniquad and macroquad]]); new backends ([[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]], [[T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections]]).

## Files owned

- `src/main.rs`, `src/shell/app.rs`, `src/shell/mod.rs`, `src/shell/recording.rs`, `src/shell/render_ctx.rs` (if T165 made it)
- the source scan tests (in `src/shell/` or `tests/`) and the other `src/shell/` call sites
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T167): specify the slice`
- [ ] tests — `test(T167): add failing tests`
- [ ] models — `feat(T167): add the types`
- [ ] behaviour — `refactor(T167): migrate the slice`
- [ ] quality — `chore(T167): pass clippy and rustfmt`
- [ ] tablet by hand — owner, recorded in the Log

## Log

- 2026-11-14 — split from T164 by the owner, following the batch's recommendation.
