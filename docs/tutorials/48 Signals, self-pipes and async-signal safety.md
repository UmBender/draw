---
title: Signals, self-pipes and async-signal safety
step: 48
requires: ["[[12 The app loop, idle redraw and UI]]", "[[35 Reading a pen through a patched window library]]", "[[47 A flight recorder for rare bugs]]"]
feature: "[[Clean exit on signals]]"
code: ["vendor/miniquad/src/signal.rs", "vendor/miniquad/src/native/linux_wayland.rs", "vendor/miniquad/src/native/linux_x11.rs", "src/main.rs", "src/shell/app.rs"]
tags: [tutorial]
---

# Signals, self-pipes and async-signal safety

> **Learning path step 48.** Requires: step 12 (the app loop), step 35
> (the patched miniquad), step 47 (the flight recorder) · Next: —

## Why this matters here

draw sleeps in a blocking event loop ([[ADR-T12-1 Blocking event loop
with cached frame]]). When you press `Ctrl-C` in its terminal, the kernel
sends `SIGINT`. Without a handler, the default action ends the process on
the spot: no ordered teardown of the window and GL context, and no last
line in the session log. With a handler, two questions come up. What may
the handler do? And how does it wake a loop that is asleep in `poll`?

## The concept

**A handler interrupts anything.** It can run between any two
instructions of any thread that does not block the signal. That thread
may be inside `malloc` with the heap lock held, or halfway through a
`println!`. If the handler calls `malloc` too, it deadlocks on a lock
its own thread holds. POSIX therefore lists the few functions that are
*async-signal-safe*: `write`, `read`, `sigaction`, `raise`, `alarm`,
`_exit` and a few dozen more. Atomic loads and stores are safe too.
Allocating, formatting, locking a `Mutex` and Rust's `println!` are not.

**So the handler only records.** It stores the signal in an atomic and
returns. The real work happens later, in normal code.

**The self-pipe trick wakes the sleeper.** A thread asleep in `poll` does
not look at atomics. So the program creates a pipe at start-up and adds
its read end to the `poll` set. The handler writes one byte into the
pipe (`write` is async-signal-safe), and `poll` returns because the pipe
is readable. This works whichever thread got the signal.

```
Ctrl-C ─▶ kernel ─▶ on_quit(SIGINT)          (any thread)
                     ├─ RECEIVED = 2          (atomic)
                     └─ write(pipe, 1 byte)
                                 │
event loop: poll([display, timer, pipe]) ◀────┘ returns
            └─ received()? ─▶ quit_ordered = true ─▶ loop ends
main: log "signal: SIGINT", exit line ─▶ reraise(SIGINT)
```

**Die by the signal you got.** A program that catches `SIGINT` should,
after cleaning up, put the default action back and raise the signal
again. The parent then sees "killed by SIGINT" (`WIFSIGNALED`), not a
normal exit with code 130. Shells and `make` use that difference to stop
a loop on `Ctrl-C` instead of moving on to the next item.

**`SA_RESETHAND` for the impatient.** With this flag the kernel puts the
default action back as it delivers the first signal. If the cleanup
hangs, a second `Ctrl-C` kills at once.

## How draw implements it

- `signal::install_quit` (`vendor/miniquad/src/signal.rs`) makes the
  pipe with `pipe2(O_NONBLOCK | O_CLOEXEC)` and installs `on_quit` for
  `SIGINT` and `SIGTERM` with `SA_RESETHAND | SA_RESTART`. `on_quit`
  saves and restores `errno`, so the interrupted code never sees the
  handler's `write` result.
- The Wayland loop's `poll_new_event` polls a third descriptor,
  `signal::wake_fd()`. The X11 loops used to block inside `XNextEvent`,
  which retries on `EINTR` and never looks at our pipe. Now, when
  `XPending` says nothing is queued, they call `signal::wait_readable`
  on `XConnectionNumber` and the pipe first. After the wait, every loop
  calls `signal::order_quit_if_received`.
- `main` (`src/main.rs`) logs `app::signal_line` and the exit line with
  `app::exit_reason`, then calls `signal::reraise`.
- The unsafe code lives in the vendored miniquad, because draw itself
  forbids `unsafe` ([[ADR-0014 Minimal dependencies and no unsafe]],
  [[ADR-T48-1 Signal handling in the vendored miniquad]]).

**Fatal signals break the rule on purpose.** `on_fatal` calls draw's
hook, `app::on_fatal_signal`. The hook formats text and writes a dump,
which is exactly what a handler should not do. The process is dying
anyway, and the dump is worth the risk, so draw only limits the damage:

- `diag::try_with` uses `try_lock`, so a crash while the recorder was
  locked skips the dump instead of deadlocking on it.
- `alarm(2)` ends the process if the hook deadlocks elsewhere, for
  example in `malloc`.
- A 256 KiB alternate stack (`sigaltstack`, `SA_ONSTACK`) gives the hook
  room on the main thread, even after a stack overflow.
- Afterwards the handler puts the *previous* action back. For a fault
  raised by the kernel (`si_code > 0`) it simply returns: the faulting
  instruction runs again, and Rust's stack-overflow check or the
  default core dump takes over. A signal sent with `kill` would not
  repeat, so it is raised again with the default action.

## Try it

1. `cargo build --release && ./target/release/draw`, then `Ctrl-C` in
   the terminal. Run `echo $?` (130) and `tail -2
   ~/.local/state/draw/draw.log`.
2. `./target/release/draw & sleep 2; kill -SEGV $!`. Look for the
   `sigsegv` dump path on stderr and `coredumpctl list draw`.
3. Remove `libc::SA_RESETHAND` in `install_quit` and add a
   `std::thread::sleep` of a minute before `signal::reraise` in `main`.
   A second `Ctrl-C` no longer kills draw. Put both back.
4. In the Wayland `poll_new_event`, poll 2 descriptors instead of 3.
   Now `Ctrl-C` quits only on the next input event, such as a mouse
   move over the window.

## Further reading

- `man 7 signal-safety`: the list of async-signal-safe functions.
- `man 2 sigaction` (`SA_RESETHAND`, `SA_ONSTACK`, `SA_SIGINFO`) and
  `man 2 sigaltstack`.
- D. J. Bernstein's self-pipe trick, and Linux's `signalfd(2)`, which
  does the same job without a handler but only for threads that block
  the signal.
- "Proper handling of SIGINT/SIGQUIT", <https://www.cons.org/cracauer/sigint.html>.
