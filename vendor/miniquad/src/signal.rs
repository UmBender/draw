//! draw patch (ADR-T48-1): quit and fatal signal handling.
//!
//! `SIGINT` and `SIGTERM` are turned into an ordered quit. The handler
//! records the signal and writes one byte to a self-pipe, and the event
//! loops wait on that pipe next to the display. A second quit signal
//! gets the default action (`SA_RESETHAND`). Fatal signals run one hook,
//! then fall back to the previous action.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::OnceLock;

/// Size of the alternate signal stack the fatal handler runs on: room for
/// the hook's formatting and file writes (Rust's own is a few KiB).
const FATAL_STACK_BYTES: usize = 256 * 1024;

/// Seconds the fatal hook may take before `SIGALRM` ends the process.
const FATAL_HOOK_SECONDS: libc::c_uint = 2;

/// The first quit signal's number, 0 before one arrives.
static RECEIVED: AtomicI32 = AtomicI32::new(0);
/// The self-pipe's read end, -1 before [`install_quit`].
static READ_FD: AtomicI32 = AtomicI32::new(-1);
/// The self-pipe's write end, -1 before [`install_quit`].
static WRITE_FD: AtomicI32 = AtomicI32::new(-1);
/// The fatal hook, set once by [`install_fatal`].
static FATAL_HOOK: OnceLock<fn(FatalSignal)> = OnceLock::new();
/// The actions the fatal handler replaced, in [`FatalSignal::ALL`] order.
static PREVIOUS: OnceLock<[libc::sigaction; 5]> = OnceLock::new();
/// Whether the fatal hook has run.
static FATAL_RAN: AtomicBool = AtomicBool::new(false);

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
        match self {
            QuitSignal::Interrupt => libc::SIGINT,
            QuitSignal::Terminate => libc::SIGTERM,
        }
    }

    /// The `SIG*` name.
    pub fn name(self) -> &'static str {
        match self {
            QuitSignal::Interrupt => "SIGINT",
            QuitSignal::Terminate => "SIGTERM",
        }
    }

    /// The quit signal with Linux number `number`, if any.
    pub fn from_number(number: i32) -> Option<QuitSignal> {
        QuitSignal::ALL
            .iter()
            .copied()
            .find(|s| s.number() == number)
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
        match self {
            FatalSignal::Segv => libc::SIGSEGV,
            FatalSignal::Bus => libc::SIGBUS,
            FatalSignal::Ill => libc::SIGILL,
            FatalSignal::Fpe => libc::SIGFPE,
            FatalSignal::Abrt => libc::SIGABRT,
        }
    }

    /// The `SIG*` name.
    pub fn name(self) -> &'static str {
        match self {
            FatalSignal::Segv => "SIGSEGV",
            FatalSignal::Bus => "SIGBUS",
            FatalSignal::Ill => "SIGILL",
            FatalSignal::Fpe => "SIGFPE",
            FatalSignal::Abrt => "SIGABRT",
        }
    }

    /// The fatal signal with Linux number `number`, if any.
    pub fn from_number(number: i32) -> Option<FatalSignal> {
        FatalSignal::ALL
            .iter()
            .copied()
            .find(|s| s.number() == number)
    }

    /// Its position in [`FatalSignal::ALL`].
    fn index(self) -> usize {
        match self {
            FatalSignal::Segv => 0,
            FatalSignal::Bus => 1,
            FatalSignal::Ill => 2,
            FatalSignal::Fpe => 3,
            FatalSignal::Abrt => 4,
        }
    }
}

/// Installs the `SIGINT`/`SIGTERM` handler and the self-pipe. Returns
/// `false` if either could not be set up; draw then keeps the default
/// action. Calling it again is harmless.
pub fn install_quit() -> bool {
    if READ_FD.load(Ordering::SeqCst) < 0 {
        let mut fds: [libc::c_int; 2] = [-1, -1];
        // SAFETY: `fds` has room for the two descriptors pipe2 writes.
        let made = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK | libc::O_CLOEXEC) };
        if made != 0 {
            return false;
        }
        WRITE_FD.store(fds[1], Ordering::SeqCst);
        READ_FD.store(fds[0], Ordering::SeqCst);
    }
    let handler: extern "C" fn(libc::c_int) = on_quit;
    QuitSignal::ALL.iter().all(|signal| {
        // SAFETY: the handler only touches atomics and calls `write`, both
        // async-signal-safe; the action is fully initialised.
        unsafe {
            set_action(
                signal.number(),
                handler as libc::sighandler_t,
                libc::SA_RESETHAND | libc::SA_RESTART,
            )
        }
    })
}

/// The quit handler: records the first signal and wakes the event loop.
extern "C" fn on_quit(sig: libc::c_int) {
    // SAFETY: errno is thread-local; it is put back so the interrupted
    // code does not see the handler's `write` result.
    let errno = unsafe { *libc::__errno_location() };
    let _ = RECEIVED.compare_exchange(0, sig, Ordering::SeqCst, Ordering::SeqCst);
    let fd = WRITE_FD.load(Ordering::SeqCst);
    if fd >= 0 {
        let byte: u8 = 1;
        // SAFETY: one byte from a live local to an open, non-blocking pipe.
        // A full pipe already holds a wake-up, so a failed write is fine.
        unsafe { libc::write(fd, (&byte as *const u8).cast(), 1) };
    }
    // SAFETY: see above.
    unsafe { *libc::__errno_location() = errno };
}

/// Installs `handler` for `sig` with `flags` and an empty mask.
///
/// # Safety
///
/// `handler` must be `SIG_DFL`, `SIG_IGN` or an async-signal-safe
/// function of the shape `flags` implies.
unsafe fn set_action(sig: libc::c_int, handler: libc::sighandler_t, flags: libc::c_int) -> bool {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction = handler;
    action.sa_flags = flags;
    libc::sigemptyset(&mut action.sa_mask);
    libc::sigaction(sig, &action, std::ptr::null_mut()) == 0
}

/// Puts the default action back for `sig`, unblocks it and raises it.
///
/// # Safety
///
/// Async-signal-safe; ends the process unless `sig` is ignored by default.
unsafe fn raise_default(sig: libc::c_int) {
    set_action(sig, libc::SIG_DFL, 0);
    let mut set: libc::sigset_t = std::mem::zeroed();
    libc::sigemptyset(&mut set);
    libc::sigaddset(&mut set, sig);
    libc::pthread_sigmask(libc::SIG_UNBLOCK, &set, std::ptr::null_mut());
    libc::raise(sig);
}

/// The first quit signal received since [`install_quit`], if any.
pub fn received() -> Option<QuitSignal> {
    QuitSignal::from_number(RECEIVED.load(Ordering::SeqCst))
}

/// Ends the process by `signal` with its default action, so the parent
/// sees it killed by that signal.
pub fn reraise(signal: QuitSignal) -> ! {
    let sig = signal.number();
    // SAFETY: plain libc calls on a valid signal number.
    unsafe { raise_default(sig) };
    // Not reached unless the signal was blocked elsewhere: same status.
    std::process::exit(128 + sig)
}

/// Installs `hook` for the fatal signals. It runs once, on the first
/// fatal signal, before the previous action takes over. Returns `false`
/// if a handler could not be installed.
pub fn install_fatal(hook: fn(FatalSignal)) -> bool {
    if FATAL_HOOK.set(hook).is_err() {
        return true;
    }
    install_fatal_stack();
    // SAFETY: an all-zero sigaction is a valid value to be overwritten.
    let mut previous: [libc::sigaction; 5] = unsafe { std::mem::zeroed() };
    for &signal in FatalSignal::ALL.iter() {
        // SAFETY: reads the current action into a live slot.
        let read = unsafe {
            libc::sigaction(
                signal.number(),
                std::ptr::null(),
                &mut previous[signal.index()],
            )
        };
        if read != 0 {
            return false;
        }
    }
    let _ = PREVIOUS.set(previous);
    let handler: extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void) = on_fatal;
    FatalSignal::ALL.iter().all(|signal| {
        // SAFETY: `on_fatal` has the SA_SIGINFO shape; see its notes on
        // what it calls.
        unsafe {
            set_action(
                signal.number(),
                handler as libc::sighandler_t,
                libc::SA_SIGINFO | libc::SA_ONSTACK | libc::SA_RESETHAND,
            )
        }
    })
}

/// Gives the calling thread an alternate signal stack big enough for the
/// hook. The memory lives until the process ends.
fn install_fatal_stack() {
    let memory: &'static mut [u8] = Box::leak(vec![0u8; FATAL_STACK_BYTES].into_boxed_slice());
    let stack = libc::stack_t {
        ss_sp: memory.as_mut_ptr().cast(),
        ss_flags: 0,
        ss_size: memory.len(),
    };
    // SAFETY: `stack` describes leaked memory that is never freed. A
    // failure keeps the previous alternate stack.
    unsafe { libc::sigaltstack(&stack, std::ptr::null_mut()) };
}

/// The fatal handler: runs the hook once, bounded by `alarm`, then hands
/// the signal to the action it replaced.
///
/// The hook is not async-signal-safe (ADR-T48-1): it may allocate and
/// lock. `alarm` ends a hook that deadlocks.
extern "C" fn on_fatal(sig: libc::c_int, info: *mut libc::siginfo_t, _context: *mut libc::c_void) {
    let Some(signal) = FatalSignal::from_number(sig) else {
        return;
    };
    if !FATAL_RAN.swap(true, Ordering::SeqCst) {
        if let Some(hook) = FATAL_HOOK.get() {
            // SAFETY: async-signal-safe.
            unsafe { libc::alarm(FATAL_HOOK_SECONDS) };
            hook(signal);
            // SAFETY: async-signal-safe; cancels the alarm.
            unsafe { libc::alarm(0) };
        }
    }
    // SAFETY: `info` comes from the kernel for an SA_SIGINFO handler.
    let from_kernel = !info.is_null() && unsafe { (*info).si_code } > 0;
    let previous = PREVIOUS.get().map(|all| all[signal.index()]);
    // SAFETY: puts back an action that was installed before ours, or the
    // default; both are valid for `sig`.
    unsafe {
        match previous {
            Some(action) => {
                libc::sigaction(sig, &action, std::ptr::null_mut());
            }
            None => {
                set_action(sig, libc::SIG_DFL, 0);
            }
        }
        if !from_kernel {
            // Sent by kill/raise/abort: returning would carry on, so the
            // default action ends the process now.
            raise_default(sig);
        }
    }
    // From the kernel: returning re-runs the faulting instruction, and
    // the previous action (Rust's stack-overflow check) or the default
    // (a core dump) takes over.
}

/// The self-pipe's read end for the event loops, or -1 before
/// [`install_quit`] (`poll` ignores negative descriptors).
pub(crate) fn wake_fd() -> i32 {
    READ_FD.load(Ordering::SeqCst)
}

/// Blocks until `fd` is readable, a quit signal wakes the self-pipe, or a
/// signal interrupts the wait. For the X11 loops, which would otherwise
/// block inside `XNextEvent`.
pub(crate) fn wait_readable(fd: libc::c_int) {
    let mut fds = [
        libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: wake_fd(),
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    // SAFETY: `fds` is a live array of two pollfd; negative fds are ignored.
    unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
}

/// Orders the quit if a quit signal arrived. Called by the event loops
/// after each wait.
pub(crate) fn order_quit_if_received() {
    if received().is_some() {
        if let Ok(mut display) = crate::native_display().try_lock() {
            display.quit_ordered = true;
        }
    }
}
