---
id: T45
title: Doubled keys after a long idle on Wayland
status: done
wave: 30
branch: fix/wayland-idle-doubled-keys
depends_on: [T44]
adrs: []
feature: "[[Native Wayland (experimental)]]"
tutorial: ""
tags: [task]
---

# T45 Doubled keys after a long idle on Wayland

## Goal

With `DRAW_BACKEND=wayland`, a key press reaches the editor once even
after the window has been left open and unused for a long time. Today,
after such an idle period, each typed character can show up twice
(`aa`). This is the bug of [[T44 One keyboard per Wayland seat]] coming
back with two copies instead of three.

## Known facts (2026-11-02, owner's Hyprland 0.56.2)

- Owner's report: the app was left open and unused for a while. On
  returning, typed text came out twice per key. Earlier the same bug
  (T44) gave three copies. The owner suspects it depends on how long the
  app has been open or idle.
- **Check the build first.** In the owner's shell, `draw` is an alias for
  `flatpak run io.github.UmBender.draw`. The installed Flatpak (1.4.3,
  deployed 2026-10-28 01:06) is **older than the T44 fix** (merged
  2026-11-02 17:30). If the report came from that Flatpak, it is the
  T44 bug again, not a new one: rebuild and reinstall the Flatpak from
  `main` and try again before changing any code. Local binaries at the
  time of writing: `target/release/draw` built 17:26:13 (just before the
  T44 fix commit at 17:26:26, so it is not certain it has the fix) and
  `target/debug/draw` built 17:31:35 (after the merge, so it has it).
- Two copies instead of three fits the T44 explanation: the compositor
  re-sent `wl_seat.capabilities` once instead of twice. A long idle is a
  likely trigger for that re-send (screen blanking/DPMS, idle inhibitor
  or lock screen, USB autosuspend of the keyboard or tablet, suspend and
  resume). T44 could not find the trigger; an idle period is the first
  concrete lead.
- If the bug shows up with a build that has T44, the T44 explanation is
  incomplete. Places to look, all in
  `vendor/miniquad/src/native/linux_wayland.rs`:
  - a second `wl_seat` (the registry announcing a seat again after idle
    or resume binds it and adds the listener a second time);
  - `keyboard_handle_keymap` / `keyboard_handle_enter` running again
    after idle and registering something twice;
  - key repeat (`KeyboardContext::repeated_key`,
    `generate_key_repeat_events`): a stale `repeated_key` or repeat
    timer after a missed `leave`/release could add exactly one extra
    character per press;
  - the same key event reaching the editor through two paths (Wayland
    `Char` plus a repeat event with `repeat = true`).

## Proposal (settle in the spec step)

1. Find out which binary the owner ran. Reinstall the Flatpak from
   `main` (with T44) and try to reproduce: leave the window idle (screen
   blanking, lock, suspend) and type afterwards.
2. Run the T44 build with `DRAW_BACKEND=wayland WAYLAND_DEBUG=client`
   over a long idle period and save the log. Look for repeated
   `wl_seat.capabilities`, `wl_registry.global` for `wl_seat`,
   `get_keyboard`, `wl_keyboard.keymap`/`enter`, and whether one physical
   key gives one or two `wl_keyboard.key` events.
3. If it only happened with the old Flatpak: close the task as a
   duplicate of T44 and record AC-7 of T44 as checked by this run.
4. Otherwise, fix the duplicated path with the same pattern as T44: a
   pure helper in the vendored crate tested from draw, the FFI callback
   only applying its answer, marked `draw patch` and listed in
   `vendor/miniquad/PATCHED.md`, with an ADR or an amendment to
   ADR-T44-1.

## Spec

Closed as a duplicate of [[T44 One keyboard per Wayland seat]]: the
report came from a Flatpak built before the T44 fix. With a Flatpak
rebuilt from `main` (T44 included) the bug did not come back over a long
run, so there is nothing to fix (Proposal, case 3).

- **AC-1** — owner check with the rebuilt Flatpak and
  `DRAW_BACKEND=wayland`, left open for a long time: each typed key gives
  one character. Checked 2026-11-02 (see *Log*).

## Out of scope

- X11.
- Upstreaming to miniquad.

## Files owned

- `vendor/miniquad/src/native/linux_wayland.rs`, `vendor/miniquad/src/seat.rs`,
  `vendor/miniquad/PATCHED.md` (only if a code fix is needed)
- `src/shell/input_map.rs` (tests)
- `docs/features/Native Wayland (experimental).md`

## Subtasks (one commit each)

- [x] spec — `docs(T45): close as a duplicate of T44`
- [x] tests — skipped: no code change
- [x] models — skipped: no code change
- [x] behaviour — skipped: no code change
- [x] quality — skipped: no code change
- [x] docs — same commit as the spec; the feature note needs no change

## Learning path

No new tutorial expected.

## Log

- 2026-11-02 — note written from the owner's report; not investigated
  yet. The installed Flatpak predates the T44 fix, so check the build
  first.
- 2026-11-02 — checked the build. The installed Flatpak (1.4.3,
  committed 2026-10-28) predated the T44 fix, so the report came from a
  binary without it. Rebuilt with `scripts/package.sh` and installed with
  `flatpak update --user` (Flatpak commit `782c358…`, 2026-11-02 23:47
  UTC, from `main` at `4222ca3`). The owner ran it for a long time: no
  doubled letters. Closed as a duplicate of T44; T44's AC-7 (typing
  gives one character) is checked by this run. No code, test, ADR or
  feature-note change.
