---
id: T47
title: Always-on diagnostics log
status: done
wave: 31
branch: task/T47-diagnostics-log
depends_on: []
adrs: ["[[ADR-T47-1 Flight recorder and capped session log]]"]
feature: "[[Diagnostics log]]"
tutorial: "[[47 A flight recorder for rare bugs]]"
tags: [task]
---

# T47 Always-on diagnostics log

## Goal

draw always keeps a cheap record of what it has been doing, without any
command-line flag or environment variable, so that a rare bug (letters
typed twice or three times, a crash after hours open, a segfault on
exit) can be reported from the log of the session where it happened,
instead of having to make it happen again.

## Known facts (2026-11-02)

- Today the only persistent record is the panic report of
  [[ADR-T27-3 Crash report from a panic hook and split debug symbols]]:
  `$XDG_STATE_HOME/draw/crash.log`, written by the panic hook. It does
  not cover segfaults or signals, and records nothing before the crash.
- Other output is ad-hoc `eprintln!` (`src/main.rs`, `src/shell/app.rs`)
  and `DRAW_FRAME_TIMES`; nothing reaches a file.
- The bugs it must help with live below draw, in the vendored miniquad
  Wayland backend (seat, keyboard, repeat, tablet events), so the
  backend must be able to log too.
- [[ADR-0014 Minimal dependencies and no unsafe]]: prefer a small
  hand-written logger over `tracing`/`log` + appenders, unless the ADR
  for this task argues otherwise.
- Must cost almost nothing per frame and per input event (i5 8th gen
  iGPU target, [[ADR-T24-5 Redraw budgets]]).

## Proposal (settle in the spec step)

1. **Flight recorder.** A fixed-size in-memory ring of recent events
   (e.g. the last few thousand: backend and seat events, key down/char
   with timestamps, tool changes, undo/redo, frame-time outliers, idle
   and focus changes). Written to disk when something goes wrong
   (panic, fatal signal, clean exit with an anomaly) and on demand.
2. **Session log on disk.** A small, size-capped, rotated file in
   `$XDG_STATE_HOME/draw/` (e.g. `draw.log`, `draw.log.1`) with
   low-volume lines: start-up (version, build id, backend, compositor,
   seat capabilities, devices), warnings, anomalies, exit reason. Never
   document contents (privacy): key events log keycodes and counts, not
   typed text, or the ADR decides.
3. **Anomaly detectors** that write a line by themselves, e.g. the same
   key `Char` delivered twice for one press, a second `wl_seat`/keyboard,
   a frame over budget, a long gap with no events (idle).
4. **On-demand dump**: a shortcut or `SIGUSR1` writes the ring to a
   timestamped file and shows where, so the owner can save it right
   after noticing a bug.
5. **Verbosity**: `DRAW_LOG=debug|trace` raises detail, but the default
   level is useful on its own.
6. Hook the crash report of ADR-T27-3 and the signal handling of
   [[T48 Clean exit on Ctrl-C]] into the same files.
7. A short doc: where the logs are and what to attach to a bug report.

## Spec

Design: [[ADR-T47-1 Flight recorder and capped session log]]. Code:
`src/shell/diag/` (`mod.rs` API and global, `ring.rs`, `event.rs`,
`detect.rs`, `file.rs`), hooks in `src/shell/{app,input_map,crash}.rs`
and `src/main.rs`, and `vendor/miniquad/src/diag.rs` with its `push`
calls in the Wayland backend. Unit tests sit in `#[cfg(test)]` modules
of those files (`diag::*::tests`).

- **AC-1 Ring.** Holds the newest 4 096 records, oldest first, and
  never reallocates.
  `ring::tests::ring_keeps_newest_records_in_order`,
  `ring_push_never_grows_capacity`, proptest `ring_matches_vecdeque_model`.
- **AC-2 Small records.** A record is at most 32 bytes, so the ring is
  at most 128 KiB. `event::tests::record_is_at_most_32_bytes`.
- **AC-3 Privacy.** Plain text keys and chars never carry their
  identity in records, dump lines or log lines. Named keys and
  `Ctrl`/`Alt` shortcuts keep theirs.
  `event::tests::key_tag_plain_letter_is_text`,
  `key_tag_shift_letter_is_text`, `key_tag_ctrl_or_alt_letter_is_named`,
  `key_tag_named_key_keeps_code`, `backend_key_text_codes_are_hidden`,
  `backend_key_with_ctrl_keeps_code`, `char_class_hides_character`;
  `mod::tests::dump_of_typed_text_names_no_letter`.
- **AC-4 Levels.** `DRAW_LOG` is parsed case-insensitively. Unset means
  `info`. An unknown value means `info` plus a warning.
  `mod::tests::level_unset_is_info`, `level_names_parse_any_case`,
  `level_unknown_is_info_with_warning`,
  `level_below_threshold_writes_nothing`.
- **AC-5 Capped session log.** Lines are
  `<UTC ISO 8601 ms> <uptime> <LEVEL> <message>`. The file rotates to
  `draw.log.1` before a line would push it past 512 KiB, including at
  start-up. It never exceeds the cap.
  `file::tests::log_line_has_time_uptime_level_and_message`,
  `session_log_rotates_before_cap`,
  `session_log_rotates_oversized_file_at_open`,
  `session_log_appends_to_small_file_at_open`, proptest
  `session_log_never_exceeds_cap`, `utc_formats_known_instants`.
- **AC-6 Rate limit.** Each anomaly kind and the slow-frame line write
  at most 10 lines per session, then one "suppressed" line.
  `detect::tests::limiter_allows_ten_then_one_notice`,
  `mod::tests::anomaly_lines_are_rate_limited`.
- **AC-7 Detectors.** *Held key pressed again*, *several chars per
  press*, *repeat burst*, *keyboard re-created* and *backend queue
  overflow* each become an anomaly. A gap of more than 5 minutes
  without input writes an idle line.
  `detect::tests::key_watch_held_key_pressed_again_is_anomaly`,
  `key_watch_repeat_is_not_anomaly`,
  `key_watch_release_then_press_is_not_anomaly`,
  `key_watch_two_chars_per_press_is_anomaly`,
  `key_watch_repeat_chars_are_not_anomaly`,
  `key_watch_focus_change_forgets_held_keys`,
  `idle_watch_reports_gap_over_five_minutes`,
  `idle_watch_short_gap_is_quiet`,
  `backend_watch_repeat_burst_is_anomaly`,
  `backend_watch_second_keyboard_is_anomaly`,
  `backend_watch_first_keyboard_is_quiet`;
  `mod::tests::backend_overflow_is_anomaly`;
  `input_map::tests::collector_records_held_key_pressed_again`,
  `collector_records_text_keys_without_identity`.
- **AC-8 Dumps.** A dump has a header, one line per record (oldest
  first, at most 96 bytes each) and the anomaly counts. Its name is
  `dump-<UTC stamp>-<reason>.log`. At most 5 dump files are kept, and
  a session writes at most 3 automatic dumps.
  `mod::tests::dump_writes_header_records_and_counts`,
  `first_anomaly_of_a_kind_dumps_once`, `auto_dumps_are_capped_per_session`,
  `file::tests::dump_name_has_utc_stamp_and_reason`,
  `prune_dumps_keeps_newest_five`,
  `event::tests::every_record_line_fits_96_bytes`.
- **AC-9 On-demand dump.** `Ctrl+Shift+L` asks for a dump in every
  mode, and no editor command uses that chord.
  `app::tests::dump_chord_is_ctrl_shift_l`,
  `app::tests::dump_chord_is_not_an_editor_command`.
- **AC-10 Crash report.** The panic hook still writes `crash.log` (the
  existing tests stay). It also writes an `ERROR` line to `draw.log`
  and a `panic` dump. While the recorder is locked it skips both
  without blocking. The state directory is shared with `crash.log`.
  `crash::tests::state_dir_*` (renamed from `crash_log_path_*`),
  `mod::tests::panic_logs_error_and_dumps`,
  `mod::tests::try_with_locked_recorder_returns_none`.
- **AC-11 Start and exit lines.** The start line names the version,
  build id, pid, backend, session type, desktop, tablet mode and level.
  The exit line names the reason, uptime and anomaly counts.
  `mod::tests::start_line_names_version_build_and_settings`,
  `mod::tests::exit_line_names_reason_uptime_and_anomalies`.
- **AC-12 Backend events.** Every miniquad `BackendEvent` maps to a
  ring record, and the queue's dropped count reaches the overflow
  detector. `event::tests::backend_event_maps_every_kind`. The queue
  bound is tested in the vendored crate,
  `vendor/miniquad/src/diag.rs` `tests::queue_keeps_newest_and_counts_dropped`.
  `scripts/check.sh` does not run it, and Cargo cannot test a patched
  path dependency from draw's workspace. So it is run by hand on a copy
  of `vendor/miniquad` (`cargo test --lib diag` there).
- **AC-13 Cost.** Recording at the default level touches no file. In a
  release build a record costs under 1 µs on average.
  `mod::tests::recording_at_info_writes_no_file`, and
  `mod::tests::record_cost_is_under_a_microsecond`, which is ignored
  and checked only in release builds, as in `tests/perf.rs`. Manual
  check: frame times with `DRAW_FRAME_TIMES=1` on the scene of
  [[ADR-T24-5 Redraw budgets]], before and after, recorded in *Log*.

Not in this task (moved to [[T48 Clean exit on Ctrl-C]], which owns
the `unsafe` signal handling): `SIGUSR1` dumps and dumps on
`SIGSEGV`/`SIGABRT`. T48 calls `diag::dump`.

## Out of scope

- Remote telemetry or sending anything over the network.
- Fixing the bugs it helps find (T45, T48, T49).

## Files owned

- `src/shell/` (new logging module), `src/main.rs`, `src/shell/crash.rs`
- `vendor/miniquad/` (logging hooks, `PATCHED.md`)
- new ADR `ADR-T47-*`, a feature note *Diagnostics log*

## Subtasks (one commit each)

- [x] spec — `docs(T47): …`
- [x] tests — `test(T47): …`
- [x] models — `feat(T47): …`
- [x] behaviour — `feat(T47): …`
- [x] quality — `chore(T47): …`
- [x] docs — `docs(T47): …`

## Learning path

Possibly a tutorial on a lock-free ring buffer / flight recorder.

## Log

- 2026-11-02 — note written at the owner's request, first step of the
  road to 2.0.0 ([[T50 Version 2.0.0 refactor]]).
- 2026-11-02 — spec and [[ADR-T47-1 Flight recorder and capped session log]].
  `SIGUSR1` and fatal-signal dumps moved to [[T48 Clean exit on Ctrl-C]],
  which owns the `unsafe` signal code. T48 calls `diag::dump`.
- 2026-11-02 — implemented. The miniquad queue test
  (`queue_keeps_newest_and_counts_dropped`) passed on a copy of
  `vendor/miniquad`. `record_cost_is_under_a_microsecond` in release:
  **21 ns per record**. Smoke run on Hyprland (native Wayland, Huion
  tablet): `draw.log` got the start-up line with the build id, the seat
  capabilities, pointer and keyboard creation and the repeat info
  (25/s after 600 ms). `DRAW_LOG=trace` copied every record, with keys
  as `text`.
- Deviations from the spec: the char-class test now checks that two
  characters of one class leave the same record, instead of searching
  for the character in the line (`'a'` is in "char"). `AC-12`'s
  hand-run command was corrected. A notice from the backend choice is
  logged at `info` and its warning at `warn`.
- Not yet measured: the ADR-T24-5 redraw budgets with
  `DRAW_FRAME_TIMES=1` on the 5 120-stroke scene, before and after
  T47. The record cost is 21 ns and frames take one lock each, so no
  change is expected. The owner checks it on the target before the
  release.
- Integration note: T46 (`task/T46-wayland-default`, not yet on `main`)
  also touches `src/shell/app.rs` and `src/main.rs`. Expect small
  conflicts around `window_conf` and the backend notice lines.
- 2026-11-02 — integrated after T46. The one conflict, in
  `window_conf`, was resolved by logging T46's `Choice::notice` at
  `info`. Smoke run on `main`: the default is now `backend wayland`, with
  no notice, and the seat lines are logged as before.
