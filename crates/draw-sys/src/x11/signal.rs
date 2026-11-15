//! Quit signals as a flag (T177, ADR-T177-1): `SIGINT` and `SIGTERM` are
//! recorded by an async-signal-safe handler and read by the platform's poll.

/// Installs the `SIGINT` and `SIGTERM` handlers; `false` if the system
/// refused.
#[must_use]
pub fn install_quit() -> bool {
    todo!()
}

/// Takes the number of the last quit signal received, if any.
#[must_use]
pub fn take() -> Option<i32> {
    todo!()
}
