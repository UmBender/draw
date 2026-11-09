---
id: T79
title: Wayland keyboard without a keymap
status: done
wave: 35
branch: fix/wayland-no-keymap
depends_on: [T75]
adrs: ["[[ADR-T79-1 Ignore an unusable Wayland keymap]]"]
feature: "[[Native Wayland]]"
tutorial: ""
tags: [task]
---

# T79 Wayland keyboard without a keymap

## Goal

A `wl_keyboard.keymap` event with no usable keymap no longer takes draw
down. draw keeps running, keeps the last keymap it had, and leaks no file
descriptor.

## Known facts (2026-11-08)

- **Symptom:** `keyboard_handle_keymap`
  (`vendor/miniquad/src/native/linux_wayland.rs`) maps the event's `fd`
  and does `assert!(map_shm != libc::MAP_FAILED)`. A `no_keymap` event has
  size 0, `mmap` fails, and the assert panics inside a Wayland callback.
- **How it was found:** [[T75 End-to-end test suite]], see
  [[ADR-T75-1 Headless sway session for end-to-end tests]]. Each `wtype`
  is its own virtual keyboard. When the active one exits, wlroots clears
  the seat's keyboard and sway adopts the next new device before its
  keymap arrives, so the client gets `no_keymap` with size 0.
- **The suite avoids it today,** so no test covers it. A real seat can
  still hit it: anything that creates and destroys virtual keyboards
  (`wtype`, on-screen keyboards, remote-desktop tools).
- **The handler ignores `format`.** The same mapping and parsing runs for
  any format, and only `XKB_V1` is a keymap it can read.
  `xkb_keymap_new_from_string` can also return null on a bad keymap, and
  the handler stores that without checking.
- Patches to the vendored crate are marked in the source and listed in
  `vendor/miniquad/PATCHED.md` ([[ADR-T35-1 Vendored miniquad with a pen tablet patch]]).

## Proposal (settle in the spec step)

If the format is not `XKB_V1`, the size is 0 or `mmap` fails: close the
`fd`, record a backend diagnostic event, keep the current keymap and
state, and return. Apply the same to a null result from
`xkb_keymap_new_from_string`. Keys pressed while no keymap has ever
arrived must not crash either; check that path in the spec step.

## Spec

Settled in [[ADR-T79-1 Ignore an unusable Wayland keymap]]. A keymap is
unusable if the format is not `XKB_V1`, the size is 0, `mmap` fails, or
libxkbcommon returns null for the keymap or the state.

- **AC-1** — draw survives a `no_keymap` event in a real session, and
  typing works once a keymap arrives. *Test:* e2e case
  `tests/e2e/regression-T79-no-keymap/`, with a new `end-keyboard`
  directive that ends the seat's active keyboard and adds a new one with
  no keymap, which sway adopts (harness:
  `parse_input_end_keyboard_is_one_step`). The case uses snapshots only
  and passes either way on a sway that sends no `no_keymap`.
- **AC-2** — A failed keymap keeps the previous one: keys typed afterwards
  still translate correctly. *Test:* the same case's final snapshot shows
  the text typed after the swap. A compositor sends no key between
  `no_keymap` and the next keymap, so a test cannot see the old pair in
  use; that the new pair is built before the old is released is a review
  item in the *Log*.
- **AC-3** — The `fd` is closed and the mapping released on every path.
  *Test:* review in the *Log*. Counting `/proc/<pid>/fd` around a swap was
  rejected: the renderer's own descriptors vary, so it would be flaky.
- **AC-4** — An ignored keymap is recorded with its format and size and
  logged at `info`, not as an anomaly. *Tests:*
  `backend_event_maps_every_kind`, `every_record_line_fits_96_bytes`,
  `keymap_ignored_line_names_format_and_size`,
  `keymap_ignored_is_logged_at_info`.
- **AC-5** — Keys, modifiers, leave and key repeat before any keymap has
  arrived do not touch a null keymap or state. *Test:* review in the
  *Log*: every compositor sends a keymap first, so no session reaches this
  path.

The workflow's *Models* step is skipped: the only new type is one enum
variant, which has no use without the behaviour.

## Out of scope

- Upstreaming to miniquad.
- The X11 backend.
- Changing how the e2e harness drives `wtype`, except what the regression
  case needs.

## Files owned

- `vendor/miniquad/src/native/linux_wayland.rs`
- `vendor/miniquad/PATCHED.md`
- `tests/e2e/regression-T79-no-keymap/` (new), `tests/e2e.rs` (only what
  the case needs)
- `docs/decisions/` (an ADR for the vendored change)
- `vendor/miniquad/src/diag.rs`, `src/shell/diag/` (only the new event;
  added by the owner in the batch plan of 2026-11-08)

## Subtasks (one commit each)

- [x] spec — `docs(T79): specify handling of a missing keymap`
- [x] regression — `test(T79): add regression test for a missing keymap`
- [x] behaviour — `fix(T79): survive a Wayland keyboard without a keymap`
- [x] quality — no commit: `scripts/check.sh` was green with no change
- [x] docs — `docs(T79): record the fix and close the task`

## Learning path

None planned.

## Log

- 2026-11-08 — created from the bug logged in T75 and ADR-T75-1.
- 2026-11-08 — reproduced on sway 1.11 before the fix: `keymap(0, fd, 0)`
  and then `assertion failed: map_shm != libc::MAP_FAILED`, SIGABRT at
  the first `end-keyboard`. After the fix `scripts/e2e.sh` passes all 11
  cases and no golden changed; `scripts/check.sh` is green.
- 2026-11-08 — UNATTENDED CHOICE: the case does not race `wtype` as the
  bug report did. Killing the active `wtype` and starting another did not
  send `no_keymap`: the capability flapped and the new keyboard's keys
  were lost. `end-keyboard` instead keeps an idle spare `wtype` (so the
  seat keeps its keyboard capability and draw its `wl_keyboard`), ends the
  active one, and creates a virtual keyboard that never uploads a keymap,
  through the wire helpers the virtual pointer uses. That is deterministic
  and needs no fixed wait. A first version without the spare also
  reproduced the crash, but after the fix it tripped the `keyboard
  re-created` anomaly, so it is not what was committed.
- 2026-11-08 — the regression step is two commits: the second replaces
  the flapping sequence above and was also confirmed red on the unfixed
  code (same assertion).
- 2026-11-08 — review, AC-2: `keyboard_handle_keymap` unrefs the old
  keymap and state only after both new ones are non-null, and returns
  before that otherwise.
- 2026-11-08 — review, AC-3: the only `mmap` is followed by `munmap`
  in the same branch, and the one `close(fd)` comes before any return.
- 2026-11-08 — review, AC-5: `keyboard_handle_key`,
  `keyboard_handle_modifiers` and `generate_key_repeat_events` return
  early on null; `keyboard_handle_leave` skips only the modifier reset.
- 2026-11-08 — the owner should drop the leftover stash found by its
  message `batch-T79-2026-11-08 fix held while confirming red`. It is the
  fix as committed in `fix(T79)`, set aside to confirm the red run; the
  batch permissions allow neither `pop` nor `drop`, so the fix was
  re-applied with edits and compared equal to the stash.
- 2026-11-08 — process note: one investigation command included
  `wtype -h`, against the plan's rule that `wtype` runs only inside
  `scripts/e2e.sh`. The permissions refused it and it did not run.
- 2026-11-08 — for the integrator: *Native Wayland* (a keymap with no
  usable content is ignored) and *End-to-end tests* (the `end-keyboard`
  directive) are outside *Files owned* and were not updated.
- 2026-11-09 — integrated, with both feature notes updated. Typing in
  draw on the desktop and one nightly run wait on the owner.
