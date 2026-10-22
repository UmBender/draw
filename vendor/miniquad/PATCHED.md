# Patched miniquad 0.4.11

This is miniquad 0.4.11 from crates.io (MIT OR Apache-2.0, licences
alongside), patched for pen tablet input in draw. Cargo uses it through
`[patch.crates-io]` in draw's `Cargo.toml`. See draw's
`docs/decisions/ADR-T35-1 Vendored miniquad with a pen tablet patch.md`.

Removed from the crate as published: `.cargo-ok` and `.github/`.
Every change is marked `draw patch (ADR-T35-1)` in the source.

## Changes

- `src/pen.rs` (new), `src/lib.rs` (`pub mod pen`): `PenSample` and
  `take_samples()`. Backends push a sample just before they hand a pen's
  pointer motion, press or release to the event handler. At most 4096
  samples are kept between two `take_samples()` calls.
  `pointer_shadowed_by_pen()` says whether a plain pointer event is
  dropped because a tablet tool is in proximity (ADR-T40-1).
- `src/native/linux_x11/xi_input.rs`: `XIQueryDevice` and
  `XIFreeDeviceInfo` with their structs, and `PenTracker`, which finds a
  source device's valuator labelled `Abs Pressure` and keeps the
  normalised pressure of the pen that sent the last `XI_RawMotion`.
  `read_cookie` feeds it.
- `src/native/linux_x11.rs`: `X11Display::pen`; core motion, press and
  release push a sample while the tracker holds a pressure.
- `src/native/linux_wayland/extensions/tablet.rs` (new),
  `extensions.rs` (`pub mod tablet`): tablet-unstable-v2 version 1,
  written out by hand with full argument types.
- `src/native/linux_wayland.rs`: binds `zwp_tablet_manager_v2`, takes the
  tablet seat of the `wl_seat`, and turns each tool frame into pointer
  events (tip → left, `BTN_STYLUS` → middle, `BTN_STYLUS2` → right) with
  samples; a tool in proximity gets the window's cursor.
  While any tool is in proximity, `wl_pointer` motion and button events
  are dropped: Hyprland copies the pen onto `wl_pointer`, and the copy
  would steal the tip-down's sample (ADR-T40-1).

- Upstream warnings, which Cargo shows for a path dependency:
  `set_or_replace_display` (`src/lib.rs`) is compiled only for Android,
  where it is used, and `respond_to_clipboard_request`
  (`src/native/linux_x11/clipboard.rs`) reads `MESSAGE` through
  `addr_of!` instead of a shared reference to a `static mut`.

## Upgrading

Re-apply these changes to the new release, or drop the patch if upstream
gained tablet input, and record the upgrade in a new ADR.
