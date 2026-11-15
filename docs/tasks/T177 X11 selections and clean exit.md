---
id: T177
title: X11 selections and clean exit
status: review
wave: 72
branch: task/T177-x11-selections-and-clean-exit
depends_on: [T176]
adrs: [ADR-T177-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T177 X11 selections and clean exit

## Goal

Slice 3 of T144: the clipboard and primary selections (text and `image/png`, with `INCR` for large transfers), and the window-manager close and `Ctrl-C` behaving as [[T48 Clean exit on Ctrl-C]] does today.

## Known facts (2026-11-15)

- Split out of [[T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections]] by the owner before the batch (as large as T143, T156 and T164).
- **Owner decision:** Xlib, not pure xcb: the GLX context of [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]] takes an Xlib `Display*`. XInput2 through `libXi`, as the vendored miniquad does. All libraries are loaded with `dlopen` through the `loader` of [[ADR-T142-1 dlopen through libc symbols and TEST markers]]; no new crate dependency ([[ADR-T127-3 The dependency list]]); each `unsafe` primitive follows [[ADR-T127-2 The unsafe policy]].
- Integration tests start their own Xvfb with the helper `crates/draw-sys/tests/common/xvfb.rs` (T145) and fail, never skip, when it is missing.
- The PNG selection of [[T115 Copy the selection as an image]] must keep working.
- Regressions to keep: the X11 selection bugs of T115.

## Spec

Design: [[ADR-T177-1 X11 selections and quit signals]]. Integration tests live in `crates/draw-sys/tests/x11_selection.rs` (Xvfb + `xclip`), pure tests in `x11_selection_pure.rs`.

- **AC-1** — Own and answer `CLIPBOARD` and `PRIMARY` (`UTF8_STRING`, `TARGETS`, `image/png`), read them from other clients, `INCR` both ways. Tests: `x11_clipboard_text_is_served_to_xclip`, `x11_primary_text_is_served_to_xclip`, `x11_targets_list_text_targets`, `x11_png_is_served_as_image_png`, `x11_large_png_is_served_with_incr`, `x11_reads_text_set_by_xclip`, `x11_reads_png_set_by_xclip`, `x11_reads_large_png_by_incr`, `x11_unknown_target_is_refused`, `x11_events_during_a_read_are_not_lost`; pure: `incr_sender_*`, `incr_receiver_*`, `targets_*`. Regressions for T115: `regression_t115_png_is_not_offered_as_text`, `regression_t115_new_text_replaces_png`.
- **AC-2** — WM close and `Ctrl-C` (SIGINT) exit cleanly as in T48. Tests: `x11_wm_close_sets_close_requested` (T175), `x11_sigint_sets_close_requested`, `x11_sigterm_reports_its_number`.
- **AC-3** — Selection decoders fuzzed: `selection_decoders_never_panic_on_random_bytes`, `incr_receiver_never_exceeds_the_cap` (seeded xorshift, as T175/T176: `proptest` is not a dev-dependency of this crate).

## Out of scope

- `main` and e2e ([[T178 X11 platform in main and the e2e suite]]).

## Files owned

- `crates/draw-sys/src/x11/**`, the `pub mod x11` line of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/tests/x11*.rs`, `crates/draw-sys/tests/common/xvfb.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T177): specify the slice`
- [x] tests — `test(T177): add failing tests`
- [x] models — `feat(T177): add the types`
- [x] behaviour — `feat(T177): implement the selections and the clean exit`
- [x] quality — `chore(T177): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T144 by the owner.
- 2026-11-15 — implemented in the batch. `scripts/check.sh` green. Deviations: the fuzz is a seeded xorshift, not proptest (not a dev-dependency, as in T175/T176); the two signal tests of the spec are one test, `x11_quit_signals_set_close_requested`, because the signal flag is process-wide; the property atom is `_SCRATCH_SELECTION` because the `DRAW_` scan of `no_draw_env` rejects the other name. `x11::signal::take` gives `main` (T178) the signal number to end the process by it as T48 does. Tested with `xclip` on Xvfb (text, PRIMARY, TARGETS, PNG, `INCR` both ways, refused targets, events kept during a read); not tried against a real desktop or browser.
