---
id: ADR-T75-1
title: Headless sway session for end-to-end tests
status: accepted
kind: decision
date: 2026-11-08
task: T75
builds_on: ["[[ADR-T74-1 Line-based editor snapshot format]]", "[[ADR-T77-1 Test gates run in capped low-priority scopes]]", "[[ADR-T46-1 Native Wayland by default and X11 deprecated]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T75-1 Headless sway session for end-to-end tests

## Context

The shell (window, backend, `input_map`, renderer, toolbar, clean exit)
only runs in a real display session. The suite needs one that behaves the
same on the owner's laptop and on a CI runner, takes real OS input and can
never touch the owner's desktop ([[T75 End-to-end test suite]]).

A first run under headless sway found three facts that shape the design:

- A headless seat has no input devices, so `swaymsg seat - cursor …`
  succeeds but no client ever gets a `wl_pointer`.
- Each `wtype` is its own virtual keyboard. When the active one exits,
  wlroots clears the seat's keyboard and sway adopts the next new device
  before its keymap arrives. The client gets `no_keymap` with size 0, and
  `vendor/miniquad` asserts on the failed `mmap`.
- Without a device on the seat, the blocking loop has nothing to wake it.

## Decision

**Session.** `scripts/e2e.sh` starts `sway` with `WLR_BACKENDS=headless`,
pixman, `LIBGL_ALWAYS_SOFTWARE=1` (Mesa llvmpipe) and the config in
`tests/e2e/sway.conf`: one 800×600 output at scale 1, one borderless tiled
window, no bar and no bindings.

**Isolation is a hard rule.** The script runs sway on a private
`XDG_RUNTIME_DIR`, sets `WAYLAND_DISPLAY` to the absolute path of that
socket, clears `DISPLAY`, and aborts before any tool can run if the sockets
are not the ones it started. `tests/e2e.rs` repeats the check, and every
child gets an explicit environment with `env_clear`.

**Input.** Keys go through `wtype`, the pointer through
`swaymsg seat - cursor`, and close through `swaymsg … kill`, so
`shell::input_map` sees real compositor events.

- The harness holds one `zwlr_virtual_pointer_v1` open, written directly
  on the Wayland socket with the standard library. It sends no events; it
  only gives the seat a pointer capability so `swaymsg` can drive it.
- A first `wtype` with no commands keeps a keyboard on the seat from
  before the app starts.
- Every later `wtype` ends with a long sleep and is killed only after the
  next one has typed, so the active keyboard is never destroyed.

**Waiting.** After each input step the harness sends the sync chord
`Ctrl+Shift+9`. Wayland keeps a client's events in order, so when the app
reports that chord, every earlier event has been handled. Each pointer step
is synced alone, because the backend reports a press at the latest pointer
position of the batch.

**Hooks**, in `shell::diag::e2e`, are inert unless their variable is set:

- `DRAW_E2E_DUMP=<file>` receives the editor snapshot at every sync, by
  atomic rename. The dump after the last sync is the final state, so no
  hook is needed in the exit path.
- `DRAW_E2E_IDLE=<file>` receives the sync count once the frame is
  settled and presented: no camera settle or atlas re-render pending, and
  the buffer swapped.
- The sync chord is only intercepted while one of the two is set.

**Goldens.** Every case compares the final snapshot. Screenshot goldens
(`grim -t ppm`, no image decoder) exist for 5 cases only, at 800×600, with
a tolerance per case.

**Resources.** The whole run, build included, sits in one capped
low-priority scope with limits named at the top of the script, as in
[[ADR-T77-1 Test gates run in capped low-priority scopes]]. The scope starts
before `XDG_RUNTIME_DIR` is replaced, because `systemd-run --user` needs
the real one.

## Alternatives considered

- **Xvfb and xdotool** — only exercises the deprecated X11 backend.
- **`wlrctl`, `ydotool`** — extra packages; `ydotool` goes through
  `uinput`, which would reach the owner's real desktop.
- **Replacing `wtype` with our own virtual keyboard** — the socket code
  is already there, but it gives up the owner's choice of tool while the
  keep-alive order avoids the problem.
- **Fixed sleeps** — slow and flaky on a loaded machine.
- **Dump in `main` after the window closes** — `src/main.rs` is outside
  the task, and the last sync already holds the final state.

## Consequences

- Every input step costs one `wtype` process and one frame, which is
  fine for short scripts.
- `no_keymap` on a real seat would still crash `vendor/miniquad`; the
  suite avoids it and the fix belongs in its own task.
- A case cannot end on an event after its last sync, which matches how
  cases are written.

## Rollback plan

Delete `tests/e2e.rs`, `tests/e2e/`, `scripts/e2e.sh`, `shell::diag::e2e`,
its two calls in `shell::app` and the nightly job, in a `revert(T75)`
commit under a new ADR with `kind: rollback`.
