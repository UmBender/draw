//! Quit and fatal signal handling (ADR-T48-1, T149, T177): `SIGINT` and
//! `SIGTERM` are recorded into a flag and self-pipe so event loops wake; fatal
//! signals run one diagnostics hook on an alternate stack before handing back
//! to the previous action.

#![allow(unsafe_code)]

use std::ffi::{c_int, c_uint, c_void};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

const SIGINT: c_int = 2;
const SIGILL: c_int = 4;
const SIGABRT: c_int = 6;
const SIGBUS: c_int = 7;
const SIGFPE: c_int = 8;
const SIGSEGV: c_int = 11;
const SIGTERM: c_int = 15;

const SIG_DFL: usize = 0;
const SIG_UNBLOCK: c_int = 1;
const SA_SIGINFO: c_int = 0x4;
const SA_ONSTACK: c_int = 0x0800_0000;
const SA_RESTART: c_int = 0x1000_0000;
const SA_RESETHAND: c_int = 0x8000_0000u32.cast_signed();

const O_NONBLOCK: c_int = 0o4000;
const O_CLOEXEC: c_int = 0o2_000_000;

/// Size of the alternate signal stack for the fatal hook.
const FATAL_STACK_BYTES: usize = 256 * 1024;
/// Seconds the fatal hook may take before `SIGALRM` ends the process.
const FATAL_HOOK_SECONDS: c_uint = 2;

/// First quit signal received, `0` for none.
static RECEIVED: AtomicI32 = AtomicI32::new(0);
/// Self-pipe read and write ends for waking `poll` on quit signals.
static WAKE_READ: AtomicI32 = AtomicI32::new(-1);
static WAKE_WRITE: AtomicI32 = AtomicI32::new(-1);
/// Fatal hook set once by [`install_fatal`].
static FATAL_HOOK: OnceLock<fn(FatalSignal)> = OnceLock::new();
/// Previous actions replaced by [`install_fatal`], in [`FatalSignal::ALL`] order.
static PREVIOUS: OnceLock<[SigAction; 5]> = OnceLock::new();
/// Whether the fatal hook has run.
static FATAL_RAN: AtomicBool = AtomicBool::new(false);

#[repr(C)]
#[derive(Clone, Copy)]
struct SigSet {
    val: [u64; 16],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SigAction {
    handler: usize,
    mask: SigSet,
    flags: c_int,
    restorer: usize,
}

#[repr(C)]
struct StackT {
    sp: *mut c_void,
    flags: c_int,
    size: usize,
}

#[repr(C)]
struct SigInfo {
    signo: c_int,
    errno: c_int,
    code: c_int,
    _rest: [u8; 116],
}

// TEST: install_quit_records_a_sent_sigterm
// SAFETY: standard POSIX/glibc signal and pipe calls with C ABI.
unsafe extern "C" {
    fn sigemptyset(set: *mut SigSet) -> c_int;
    fn sigaddset(set: *mut SigSet, signum: c_int) -> c_int;
    fn sigaction(signum: c_int, act: *const SigAction, oldact: *mut SigAction) -> c_int;
    fn sigaltstack(ss: *const StackT, old_ss: *mut StackT) -> c_int;
    fn pthread_sigmask(how: c_int, set: *const SigSet, oldset: *mut SigSet) -> c_int;
    fn raise(sig: c_int) -> c_int;
    fn alarm(seconds: c_uint) -> c_uint;
    fn __errno_location() -> *mut c_int;
    fn pipe2(fds: *mut c_int, flags: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
}

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
    pub const ALL: [Self; 2] = [Self::Interrupt, Self::Terminate];

    /// The Linux signal number.
    #[must_use]
    pub const fn number(self) -> i32 {
        match self {
            Self::Interrupt => SIGINT,
            Self::Terminate => SIGTERM,
        }
    }

    /// The `SIG*` name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Interrupt => "SIGINT",
            Self::Terminate => "SIGTERM",
        }
    }

    /// The quit signal with Linux number `number`, if any.
    #[must_use]
    pub fn from_number(number: i32) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.number() == number)
    }
}

impl FatalSignal {
    /// Every fatal signal.
    pub const ALL: [Self; 5] = [Self::Segv, Self::Bus, Self::Ill, Self::Fpe, Self::Abrt];

    /// The Linux signal number.
    #[must_use]
    pub const fn number(self) -> i32 {
        match self {
            Self::Segv => SIGSEGV,
            Self::Bus => SIGBUS,
            Self::Ill => SIGILL,
            Self::Fpe => SIGFPE,
            Self::Abrt => SIGABRT,
        }
    }

    /// The `SIG*` name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Segv => "SIGSEGV",
            Self::Bus => "SIGBUS",
            Self::Ill => "SIGILL",
            Self::Fpe => "SIGFPE",
            Self::Abrt => "SIGABRT",
        }
    }

    /// The fatal signal with Linux number `number`, if any.
    #[must_use]
    pub fn from_number(number: i32) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.number() == number)
    }

    /// Its position in [`FatalSignal::ALL`].
    const fn index(self) -> usize {
        match self {
            Self::Segv => 0,
            Self::Bus => 1,
            Self::Ill => 2,
            Self::Fpe => 3,
            Self::Abrt => 4,
        }
    }
}

/// Only stores into an atomic and writes 1 byte to the non-blocking
/// self-pipe, preserving `errno`: async-signal-safe.
extern "C" fn on_quit(signum: c_int) {
    // TEST: x11_quit_signals_set_close_requested
    // SAFETY: `__errno_location` returns the calling thread's `errno`.
    let errno = unsafe { *__errno_location() };
    let _ = RECEIVED.compare_exchange(0, signum, Ordering::SeqCst, Ordering::SeqCst);
    let fd = WAKE_WRITE.load(Ordering::Acquire);
    if fd >= 0 {
        let byte = 1u8;
        // TEST: x11_quit_signals_set_close_requested
        // SAFETY: `fd` is a valid non-blocking pipe write end and `byte` is 1 byte on stack.
        let _ = unsafe { write(fd, (&raw const byte).cast(), 1) };
    }
    // TEST: x11_quit_signals_set_close_requested
    // SAFETY: restores the saved `errno` of the calling thread.
    unsafe { *__errno_location() = errno };
}

/// Installs `handler` for `sig` with `flags` and an empty mask.
///
/// # Safety
/// `handler` must be `SIG_DFL` or an async-signal-safe function matching
/// `flags`.
// TEST: fatal_signal_hook_is_installed
unsafe fn set_action(sig: c_int, handler: usize, flags: c_int) -> bool {
    let mut action = SigAction {
        handler,
        mask: SigSet { val: [0; 16] },
        flags,
        restorer: 0,
    };
    // TEST: fatal_signal_hook_is_installed
    // SAFETY: caller guarantees `handler` and `flags` are valid for `sig`.
    unsafe {
        sigemptyset(&raw mut action.mask);
        sigaction(sig, &raw const action, std::ptr::null_mut()) == 0
    }
}

/// Puts the default action back for `sig`, unblocks it and raises it.
///
/// # Safety
/// Async-signal-safe; terminates the process unless `sig` is ignored.
// TEST: fatal_signal_hook_is_installed
unsafe fn raise_default(sig: c_int) {
    let mut set = SigSet { val: [0; 16] };
    // TEST: fatal_signal_hook_is_installed
    // SAFETY: `SIG_DFL` and `sig` are valid; `set` is initialized by `sigemptyset`.
    unsafe {
        set_action(sig, SIG_DFL, 0);
        sigemptyset(&raw mut set);
        sigaddset(&raw mut set, sig);
        pthread_sigmask(SIG_UNBLOCK, &raw const set, std::ptr::null_mut());
        raise(sig);
    }
}

/// Installs the `SIGINT` and `SIGTERM` handlers and self-pipe; `false` if
/// either could not be set up.
#[must_use]
pub fn install_quit() -> bool {
    if WAKE_READ.load(Ordering::Acquire) < 0 {
        let mut fds = [-1 as c_int; 2];
        // TEST: x11_quit_signals_set_close_requested
        // SAFETY: `fds` is a valid 2-int buffer.
        if unsafe { pipe2(fds.as_mut_ptr(), O_NONBLOCK | O_CLOEXEC) } != 0 {
            return false;
        }
        WAKE_WRITE.store(fds[1], Ordering::Release);
        WAKE_READ.store(fds[0], Ordering::Release);
    }
    let handler: extern "C" fn(c_int) = on_quit;
    let flags = SA_RESETHAND | SA_RESTART;
    QuitSignal::ALL.into_iter().all(|sig| {
        // TEST: x11_quit_signals_set_close_requested
        // SAFETY: `on_quit` has the handler signature and uses only async-signal-safe ops.
        unsafe { set_action(sig.number(), handler as usize, flags) }
    })
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

/// The first quit signal received since [`install_quit`], if any.
#[must_use]
pub fn received() -> Option<QuitSignal> {
    QuitSignal::from_number(RECEIVED.load(Ordering::SeqCst))
}

/// Ends the process by `signal` with its default action, so the parent sees
/// it killed by that signal.
pub fn reraise(signal: QuitSignal) -> ! {
    let sig = signal.number();
    // TEST: x11_quit_signals_set_close_requested
    // SAFETY: valid signal number; resets to `SIG_DFL` and raises `sig`.
    unsafe { raise_default(sig) };
    std::process::exit(128 + sig)
}

fn install_fatal_stack() {
    let memory: &'static mut [u8] = Box::leak(vec![0u8; FATAL_STACK_BYTES].into_boxed_slice());
    let stack = StackT {
        sp: memory.as_mut_ptr().cast(),
        flags: 0,
        size: memory.len(),
    };
    // TEST: fatal_signal_hook_is_installed
    // SAFETY: `stack` describes leaked memory that is never freed.
    unsafe { sigaltstack(&raw const stack, std::ptr::null_mut()) };
}

/// Runs the fatal hook once on `sig`, bounded by `alarm`, then restores the
/// previous action.
extern "C" fn on_fatal(sig: c_int, info: *const SigInfo, _context: *mut c_void) {
    let Some(signal) = FatalSignal::from_number(sig) else {
        return;
    };
    if !FATAL_RAN.swap(true, Ordering::SeqCst)
        && let Some(hook) = FATAL_HOOK.get()
    {
        // TEST: fatal_signal_hook_is_installed
        // SAFETY: `alarm` is async-signal-safe.
        unsafe { alarm(FATAL_HOOK_SECONDS) };
        hook(signal);
        // TEST: fatal_signal_hook_is_installed
        // SAFETY: cancels the alarm.
        unsafe { alarm(0) };
    }
    // TEST: fatal_signal_hook_is_installed
    // SAFETY: `info` is valid or null as delivered by the kernel for `SA_SIGINFO`.
    let from_kernel = !info.is_null() && unsafe { (*info).code } > 0;
    let previous = PREVIOUS.get().map(|all| all[signal.index()]);
    // TEST: fatal_signal_hook_is_installed
    // SAFETY: puts back the prior action or `SIG_DFL` and re-raises if software-sent.
    unsafe {
        match previous {
            Some(action) => {
                sigaction(sig, &raw const action, std::ptr::null_mut());
            }
            None => {
                set_action(sig, SIG_DFL, 0);
            }
        }
        if !from_kernel {
            raise_default(sig);
        }
    }
}

/// Installs `hook` for the fatal signals. It runs once, on the first fatal
/// signal, before the previous action takes over.
#[must_use]
pub fn install_fatal(hook: fn(FatalSignal)) -> bool {
    if FATAL_HOOK.set(hook).is_err() {
        return true;
    }
    install_fatal_stack();
    let empty = SigAction {
        handler: 0,
        mask: SigSet { val: [0; 16] },
        flags: 0,
        restorer: 0,
    };
    let mut previous = [empty; 5];
    for signal in FatalSignal::ALL {
        // TEST: fatal_signal_hook_is_installed
        // SAFETY: reads the current action for a valid signal into `previous`.
        let ok = unsafe {
            sigaction(
                signal.number(),
                std::ptr::null(),
                &raw mut previous[signal.index()],
            ) == 0
        };
        if !ok {
            return false;
        }
    }
    let _ = PREVIOUS.set(previous);
    let handler: extern "C" fn(c_int, *const SigInfo, *mut c_void) = on_fatal;
    let flags = SA_SIGINFO | SA_ONSTACK | SA_RESETHAND;
    FatalSignal::ALL.into_iter().all(|signal| {
        // TEST: fatal_signal_hook_is_installed
        // SAFETY: `on_fatal` matches the `SA_SIGINFO` signature.
        unsafe { set_action(signal.number(), handler as usize, flags) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fatal_signal_hook_is_installed() {
        assert_eq!(std::mem::size_of::<SigAction>(), 152);
        assert_eq!(std::mem::size_of::<StackT>(), 24);
        assert_eq!(std::mem::size_of::<SigInfo>(), 128);
        assert!(install_fatal(|_| {}));
    }
}
