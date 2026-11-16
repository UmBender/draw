//! Quit signals as a flag (T177, ADR-T177-1): `SIGINT` and `SIGTERM` are
//! recorded by an async-signal-safe handler and read by the platform's poll.

#![allow(unsafe_code)]

use std::ffi::c_int;
use std::sync::atomic::{AtomicI32, Ordering};

const SIGINT: c_int = 2;
const SIGTERM: c_int = 15;
const SIG_ERR: usize = usize::MAX;

/// The last quit signal received, `0` for none.
static RECEIVED: AtomicI32 = AtomicI32::new(0);

// TEST: x11_quit_signals_set_close_requested
// SAFETY: `signal` is the glibc function with this prototype; the handler
// type is the C one.
unsafe extern "C" {
    fn signal(signum: c_int, handler: extern "C" fn(c_int)) -> usize;
}

/// Only stores into an atomic: async-signal-safe.
extern "C" fn on_quit(signum: c_int) {
    RECEIVED.store(signum, Ordering::SeqCst);
}

/// Installs the `SIGINT` and `SIGTERM` handlers; `false` if the system
/// refused.
#[must_use]
pub fn install_quit() -> bool {
    // TEST: x11_quit_signals_set_close_requested
    // SAFETY: `on_quit` has the handler type and does only an atomic store.
    unsafe { signal(SIGINT, on_quit) != SIG_ERR && signal(SIGTERM, on_quit) != SIG_ERR }
}

/// The number of the last quit signal received, without taking it.
#[must_use]
pub fn peek() -> Option<i32> {
    match RECEIVED.load(Ordering::SeqCst) {
        0 => None,
        number => Some(number),
    }
}

/// Takes the number of the last quit signal received, if any.
#[must_use]
pub fn take() -> Option<i32> {
    match RECEIVED.swap(0, Ordering::SeqCst) {
        0 => None,
        number => Some(number),
    }
}
