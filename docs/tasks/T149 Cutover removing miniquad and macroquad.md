---
id: T149
title: Cutover removing miniquad and macroquad
status: done
wave: 73
branch: task/T149-cutover-remove-miniquad-and-macroquad
depends_on: [T143, T144, T146, T147, T148, T152, T153, T154, T155, T156, T157, T158, T159, T160, T161, T162, T163, T164, T165, T166, T167, T168, T169, T170, T171, T172, T173, T175, T176, T177, T178]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T149 Cutover removing miniquad and macroquad

## Goal

The vendored miniquad and macroquad are deleted; the app runs only on the new backend.

## Known facts (2026-11-12)

- A strangler: the new platform replaces the old only when it passes the same e2e suite on both backends.
- The source scan of [[T167 Generic run and the match in main]] allows a few macroquad uses outside its interpreters (`Color`, the menu font measure, the window config, the signals, `LinuxBackend`); this task empties that allowlist.

## Spec

Follows [[ADR-T127-4 An own backend replaces miniquad and macroquad]], [[ADR-T127-3 The dependency list]], and [[ADR-T48-1 Signal handling in the vendored miniquad]]:

- **AC-1 (No `miniquad` or `macroquad` left)** — `vendor/miniquad` is deleted, `macroquad` and the `[patch.crates-io]` `miniquad` entry are removed from `Cargo.toml` and `Cargo.lock`, `src/shell/platform_mq.rs` and `src/shell/gpu_mq.rs` are deleted, `--platform` is removed from `src/shell/options.rs`, `src/main.rs`, `scripts/e2e.sh`, and `tests/e2e.rs`, and `libdecor` (only used by `miniquad`) is removed from `packaging/runtime-libs`, `Cargo.toml`, and `flake.nix`. Named guards:
  - `tests/deps.rs` removes `"macroquad"` from `runtime_allowed("draw")`.
  - `src/shell/ui_text.rs::shell_sources_outside_the_mq_interpreters_name_no_macroquad` (and `macroquad_allowlist_has_no_stale_entry`) enforces an empty allowlist (zero `macroquad::` or `miniquad::` references anywhere in `src/`).
  - `tests/packaging.rs` replaces `cargo_patches_miniquad_with_the_vendored_copy` with asserting neither `Cargo.toml` nor `Cargo.lock` contains `miniquad` or `macroquad`.
  - `crates/draw-sys` owns quit & fatal signal handling (`FatalSignal`, `QuitSignal`, `install_fatal`, `install_quit`, `received`, `reraise`) replacing `vendor/miniquad/src/signal.rs` with zero new crates.
  - `src/shell/` replaces `macroquad::color::Color`, `macroquad::math::Vec2`, and `LinuxBackend` with `draw-gfx`/`draw-core` types (`Rgba` / `Color` / `Vec2` / `Backend`), removing macroquad font exclusions in `tests/shell_fuzz.rs`.
  - `tests/e2e/text-formula/expected.ppm` (and `expected.x11.ppm` if X11 differs in toolbar) is updated to `GlGpu`'s output because `macroquad` left `alpha_blend: None` (`glBlendFunc`), punching an alpha hole in the cached layer over antialiased text (`1.08 px` `'l'` stem vs `fontdue`'s `1.49 px`), whereas `GlGpu` uses `glBlendFuncSeparate` (`alpha_blend_matches_straight_alpha_over`, `1.47 px` stem).
- **AC-2 (`scripts/release.sh --dry-run` / full gates)** — `scripts/check.sh --fuzz`, `scripts/e2e.sh --backend all` (Wayland + X11), and packaging/runtime-libs checks pass; `scripts/release.sh --dry-run <version>` exits 0.
- **AC-3 (Measurements)** — Frame times and idle CPU on the native backend are recorded in `## Log`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `Cargo.toml`, `Cargo.lock`, `vendor/`
- `src/main.rs`, `src/lib.rs`, `src/shell/**`
- `crates/draw-sys/**` (signal module)
- `tests/{deps,e2e,options,packaging,perf,shell_fuzz,text_parity}.rs`, `tests/e2e/text-formula/**`
- `scripts/e2e.sh`, `scripts/audit/**`
- `packaging/runtime-libs`, `flake.nix`, `docs/**` (mentions)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T149): specify the cutover`
- [x] tests — `test(T149): drop macroquad and miniquad from guards`
- [x] behaviour — `refactor(T149): remove miniquad and macroquad`
- [x] docs — `docs(T149): update the notes and the measurements`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-17 — cutover completed and verified:
  - Deleted `vendor/miniquad`, `src/shell/gpu_mq.rs`, `src/shell/platform_mq.rs`, `macroquad` dependency, and `--platform`; moved quit + fatal signals into `crates/draw-sys/src/signal.rs`; updated `text-formula` screenshot goldens to `GlGpu` (`glBlendFuncSeparate` preserving `fontdue` coverage `1.47 px` vs `1.49 px`, fixing `macroquad`'s `glBlendFunc` alpha-hole darkening).
  - `./scripts/check.sh --fuzz` passed (all unit/integration/guard tests, `compile-fail`, `docs`, and 48/48 `proptest` fuzz cases at 20 000 cases each).
  - `./scripts/e2e.sh --backend all` passed on both Wayland (16/16 + `e2e_idle_cpu_is_near_zero`, 29.13s) and X11 (16/16 + `e2e_idle_cpu_is_near_zero`, 26.49s).
  - `./scripts/release.sh --dry-run 1.6.0` exited 0; release binary `target/release/draw` is `2.3M`.
  - Release microbenchmarks (`tests/perf.rs`): culling `296.687µs`, hit-test miss `394.458µs`, snap drag `970.309µs`, tessellate 10k glyphs `549.79µs`, autosave capture `1.924ms`.
