//! Quit signals as a flag (T177, ADR-T177-1): `SIGINT` and `SIGTERM` are
//! recorded by an async-signal-safe handler and read by the platform's poll.

#![allow(unsafe_code)]

use std::ffi::{c_int, c_void};
use std::sync::atomic::{AtomicI32, Ordering};

const SIGINT: c_int = 2;
const SIGTERM: c_int = 15;
const SIG_ERR: usize = usize::MAX;
const O_NONBLOCK: c_int = 0o4000;
const O_CLOEXEC: c_int = 0o2_000_000;

/// The last quit signal received, `0` for none.
static RECEIVED: AtomicI32 = AtomicI32::new(0);
/// Self-pipe read and write ends for waking `poll` on quit signals.
static WAKE_READ: AtomicI32 = AtomicI32::new(-1);
static WAKE_WRITE: AtomicI32 = AtomicI32::new(-1);

// TEST: x11_quit_signals_set_close_requested
// SAFETY: `signal`, `pipe2`, `read`, `write` are standard POSIX/glibc calls.
unsafe extern "C" {
    fn signal(signum: c_int, handler: extern "C" fn(c_int)) -> usize;
    fn pipe2(fds: *mut c_int, flags: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
}

/// Only stores into an atomic and writes 1 byte to the non-blocking
/// self-pipe: async-signal-safe.
extern "C" fn on_quit(signum: c_int) {
    RECEIVED.store(signum, Ordering::SeqCst);
    let fd = WAKE_WRITE.load(Ordering::Relaxed);
    if fd >= 0 {
        let byte = 1u8;
        // TEST: x11_quit_signals_set_close_requested
        // SAFETY: `fd` is a valid non-blocking pipe write end and `byte` is 1 byte on stack.
        let _ = unsafe { write(fd, (&raw const byte).cast(), 1) };
    }
}

/// Installs the `SIGINT` and `SIGTERM` handlers; `false` if the system
/// refused.
#[must_use]
pub fn install_quit() -> bool {
    if WAKE_READ.load(Ordering::Acquire) < 0 {
        let mut fds = [-1 as c_int; 2];
        // TEST: x11_quit_signals_set_close_requested
        // SAFETY: `fds` is a valid 2-int buffer.
        if unsafe { pipe2(fds.as_mut_ptr(), O_NONBLOCK | O_CLOEXEC) } == 0 {
            WAKE_READ.store(fds[0], Ordering::Release);
            WAKE_WRITE.store(fds[1], Ordering::Release);
        }
    }
    // TEST: x11_quit_signals_set_close_requested
    // SAFETY: `on_quit` has the handler type and does only async-signal-safe ops.
    unsafe { signal(SIGINT, on_quit) != SIG_ERR && signal(SIGTERM, on_quit) != SIG_ERR }
}

/// Read end of the quit-signal self-pipe, if installed, for `poll`.
#[must_use]
pub fn wake_fd() -> Option<c_int> {
    let fd = WAKE_READ.load(Ordering::Acquire);
    (fd >= 0).then_some(fd)
}

/// The number of the last quit signal received, without taking it.
#[must_use]
pub fn peek() -> Option<i32> {
    match RECEIVED.load(Ordering::SeqCst) {
        0 => None,
        number => Some(number),
    }
}

/// Takes the number of the last quit signal received, if any, draining the
/// wake pipe.
#[must_use]
pub fn take() -> Option<i32> {
    let fd = WAKE_READ.load(Ordering::Acquire);
    if fd >= 0 {
        let mut buf = [0u8; 64];
        // TEST: x11_quit_signals_set_close_requested
        // SAFETY: `fd` is a valid non-blocking pipe read end and `buf` is valid.
        while unsafe { read(fd, buf.as_mut_ptr().cast(), buf.len()) } > 0 {}
    }
    match RECEIVED.swap(0, Ordering::SeqCst) {
        0 => None,
        number => Some(number),
    }
}
