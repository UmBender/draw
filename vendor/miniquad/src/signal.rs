//! draw patch (ADR-T48-1): quit and fatal signal handling.
//!
//! `SIGINT` and `SIGTERM` are turned into an ordered quit. The handler
//! records the signal and writes one byte to a self-pipe, and the event
//! loops wait on that pipe next to the display. A second quit signal
//! gets the default action (`SA_RESETHAND`). Fatal signals run one hook,
//! then fall back to the previous action.

/// A signal that asks draw to quit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuitSignal {
    /// `SIGINT`, sent by `Ctrl-C` in the terminal.
    Interrupt,
    /// `SIGTERM`, sent by `kill` and session managers.
    Terminate,
}

/// A signal that ends the process after its hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FatalSignal {
    /// `SIGSEGV`: invalid memory access.
    Segv,
    /// `SIGBUS`: bus error.
    Bus,
    /// `SIGILL`: illegal instruction.
    Ill,
    /// `SIGFPE`: arithmetic exception.
    Fpe,
    /// `SIGABRT`: `abort()`.
    Abrt,
}

impl QuitSignal {
    /// Every quit signal.
    pub const ALL: [QuitSignal; 2] = [QuitSignal::Interrupt, QuitSignal::Terminate];

    /// The Linux signal number.
    pub fn number(self) -> i32 {
        todo!()
    }

    /// The `SIG*` name.
    pub fn name(self) -> &'static str {
        todo!()
    }

    /// The quit signal with Linux number `number`, if any.
    pub fn from_number(number: i32) -> Option<QuitSignal> {
        let _ = number;
        todo!()
    }
}

impl FatalSignal {
    /// Every fatal signal.
    pub const ALL: [FatalSignal; 5] = [
        FatalSignal::Segv,
        FatalSignal::Bus,
        FatalSignal::Ill,
        FatalSignal::Fpe,
        FatalSignal::Abrt,
    ];

    /// The Linux signal number.
    pub fn number(self) -> i32 {
        todo!()
    }

    /// The `SIG*` name.
    pub fn name(self) -> &'static str {
        todo!()
    }

    /// The fatal signal with Linux number `number`, if any.
    pub fn from_number(number: i32) -> Option<FatalSignal> {
        let _ = number;
        todo!()
    }
}

/// Installs the `SIGINT`/`SIGTERM` handler and the self-pipe. Returns
/// `false` if either could not be set up; draw then keeps the default
/// action. Calling it again is harmless.
pub fn install_quit() -> bool {
    todo!()
}

/// The first quit signal received since [`install_quit`], if any.
pub fn received() -> Option<QuitSignal> {
    todo!()
}

/// Ends the process by `signal` with its default action, so the parent
/// sees it killed by that signal.
pub fn reraise(signal: QuitSignal) -> ! {
    let _ = signal;
    todo!()
}

/// Installs `hook` for the fatal signals. It runs once, on the first
/// fatal signal, before the previous action takes over. Returns `false`
/// if a handler could not be installed.
pub fn install_fatal(hook: fn(FatalSignal)) -> bool {
    let _ = hook;
    todo!()
}

/// The self-pipe's read end for the event loops, or -1 before
/// [`install_quit`] (`poll` ignores negative descriptors).
pub(crate) fn wake_fd() -> i32 {
    todo!()
}
