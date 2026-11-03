---
title: A flight recorder for rare bugs
step: 47
requires: ["[[12 The app loop, idle redraw and UI]]", "[[35 Reading a pen through a patched window library]]"]
feature: "[[Diagnostics log]]"
code: ["src/shell/diag/mod.rs", "src/shell/diag/ring.rs", "src/shell/diag/event.rs", "src/shell/diag/detect.rs", "src/shell/diag/file.rs", "src/shell/input_map.rs", "src/shell/app.rs", "src/shell/crash.rs", "vendor/miniquad/src/diag.rs"]
tags: [tutorial]
---

# A flight recorder for rare bugs

> **Learning path step 47.** Requires: step 12 (the app loop), step 35
> (a patched window library) · Next: —

## Why this matters here

The bugs left in draw happen once in a few hours: a letter typed twice
after the laptop slept, a crash after a long contest. By the time anyone
looks, the moment is gone, and asking a user to "run with logging on"
means waiting for the bug to happen again. The answer airplanes use is a
*flight recorder*. It always records, it only keeps the last few
minutes, and its content is saved when something goes wrong.

## The concept

**1. A ring buffer.** Keep the last N events in an array of N slots and
a `head` index. While the array is not full, push at the end. After
that, overwrite the slot at `head` (the oldest) and move `head` on:

```
capacity 4, pushed 1..=6:   [5, 6, 3, 4]   head = 2
oldest first:               items[head..] ++ items[..head] = 3 4 5 6
```

The array is allocated once, so recording costs a store and an index
update. There is no allocation, no formatting and no system call.

**2. Cheap records, expensive text.** Turning an event into text costs
far more than storing it. So each record is a small `Copy` value
(a timestamp plus an enum, at most 32 bytes). Text is made only when a
dump is written, which is rare.

**3. Bounded files.** Anything written always on must have a size
limit. The session log rotates: before a line would push the file over
the cap, the file becomes `draw.log.1` and a new one starts. Two files
of 512 KiB each is the worst case, however long draw runs. Anomaly lines
are rate-limited too, so one broken detector cannot push the start-up
line out of the log.

**4. Detectors instead of raw data.** A doubled letter would be obvious
if the recorder kept the letters, but then every dump would hold
whatever the user typed. So the detectors look at the real keys while
they run and only keep a set of held keys. They then emit an *anomaly*
record ("held key pressed again"). The ring stores text keys as plain
`text`.

## How draw implements it

- `diag::ring::Ring::push` is the ring above. `iter` chains
  `items[head..]` and `items[..head]`. The test
  `ring_push_never_grows_capacity` checks that 20 000 pushes leave the
  buffer's capacity unchanged.
- `diag::event::Record` and `Event` are the records.
  `record_is_at_most_32_bytes` pins the size. `key_tag` and
  `backend_key` apply the privacy rule to macroquad key codes and to
  evdev scancodes. `every_record_line_fits_96_bytes` checks the
  `Display` text of every variant.
- `diag::detect::KeyWatch` sees each `key_down`, `char` and `key_up`
  the `Collector` replays. `BackendWatch` reads the backend's records:
  a repeat timer that fired more than twice in one wake means the timer
  ran on through a suspend.
- `shell::app::FrameRecorder` collects one frame's records in a reused
  `Vec` and calls `diag::record_all`, which takes the global `Mutex`
  once per frame. The panic hook in `shell::crash` calls
  `diag::on_panic`, which uses `try_lock`. If the panic happened while
  the lock was held, waiting would deadlock, so it gives up instead.
- `vendor/miniquad/src/diag.rs` is a bounded queue like `pen.rs` from
  step 35. The Wayland callbacks push events, and draw swaps buffers
  with `take_events_into` every frame, so neither side allocates once
  warmed up.
- `diag::file::SessionLog::write_line` rotates before writing. The
  proptest `session_log_never_exceeds_cap` writes random line lengths
  and checks both files after every write.

## Try it

1. Run draw with `XDG_STATE_HOME=/tmp/st DRAW_LOG=trace` and open
   `/tmp/st/draw/draw.log` while you type and draw. Every record shows
   up, and the letters appear only as `text`.
2. Press `Ctrl+Shift+L` and read the dump. Find the `frame full` lines
   and compare them with the 16 ms budget of ADR-T24-5.
3. Change `REPEAT_BURST` in `detect.rs` to 0 and hold a key: every
   repeat now counts as a burst, so a dump is written. Run
   `cargo test detect` to see which test pins the real value.
4. Run `cargo test --release --lib record_cost -- --ignored --nocapture`
   to see what one record costs (about 20 ns on the target).

## Further reading

- Brendan Gregg, *Systems Performance*, on always-on tracing with fixed
  overhead.
- The Linux kernel's `ftrace` ring buffer, the same idea at kernel
  scale.
- Howard Hinnant, *chrono-Compatible Low-Level Date Algorithms*
  (`civil_from_days`, used by `diag::file::utc`).
