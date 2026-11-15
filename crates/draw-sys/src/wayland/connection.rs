//! The connection to the compositor: the registry, the bound globals
//! (`wl_compositor`, `xdg_wm_base`, the `wl_output`s), ping and pong, and
//! the dispatch loop. One window per connection (T168).

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, CString, c_char, c_ulong, c_void};
use std::fmt;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr::null_mut;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::egl::ContextError;
use crate::loader::LoadError;
use crate::wayland::client::{Client, Proxy};
use crate::wayland::generated::wayland::{wl_compositor, wl_output, wl_registry};
use crate::wayland::generated::xdg_shell::xdg_wm_base;

/// How long a blocking wait for the compositor lasts.
pub(crate) const TIMEOUT: Duration = Duration::from_secs(5);

/// Why a Wayland call failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaylandError {
    /// `libwayland-client` or `libwayland-egl` could not be loaded.
    Load(LoadError),
    /// The compositor socket could not be opened.
    Connect(String),
    /// The compositor lacks a global the backend needs.
    MissingGlobal(&'static str),
    /// A request returned no object.
    NoObject(&'static str),
    /// The connection broke or the compositor sent a protocol error.
    Protocol(String),
    /// The compositor did not answer in time.
    Timeout,
    /// The GL context failed.
    Context(ContextError),
}

impl fmt::Display for WaylandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(e) => write!(f, "cannot load the Wayland libraries: {e}"),
            Self::Connect(what) => write!(f, "cannot connect to the compositor: {what}"),
            Self::MissingGlobal(name) => write!(f, "the compositor has no {name}"),
            Self::NoObject(request) => write!(f, "{request} returned no object"),
            Self::Protocol(what) => write!(f, "Wayland protocol failure: {what}"),
            Self::Timeout => write!(f, "the compositor did not answer in time"),
            Self::Context(e) => write!(f, "GL context: {e}"),
        }
    }
}

impl std::error::Error for WaylandError {}

/// One announced global.
#[derive(Debug)]
struct Global {
    name: u32,
    interface: String,
    version: u32,
}

/// One bound `wl_output` and its integer scale.
#[derive(Debug)]
struct Output {
    proxy: *mut Proxy,
    scale: i32,
}

/// What the window's events leave behind, read by the window and the
/// platform.
#[derive(Debug, Default)]
pub(crate) struct WinState {
    /// The serial of a configure that is not acked yet.
    pub serial: Cell<Option<u32>>,
    /// The size the toplevel's last `configure` asked for, `0` for "any".
    pub pending: Cell<(i32, i32)>,
    /// The logical size in use.
    pub size: Cell<(i32, i32)>,
    /// The compositor asked the window to close.
    pub close: Cell<bool>,
    /// A frame callback is outstanding.
    pub frame_pending: Cell<bool>,
    /// The outputs the surface is on, as proxy addresses.
    pub entered: RefCell<Vec<usize>>,
    /// Width, height and scale the GL window was last sized for.
    pub applied: Cell<(i32, i32, i32)>,
}

/// The state shared by the connection, the window and the listeners. The
/// listeners receive its address as their data pointer.
#[derive(Debug)]
pub(crate) struct Shared {
    pub client: Client,
    pub display: *mut Proxy,
    registry: Cell<*mut Proxy>,
    globals: RefCell<Vec<Global>>,
    pub compositor: Cell<*mut Proxy>,
    pub wm_base: Cell<*mut Proxy>,
    outputs: RefCell<Vec<Output>>,
    pub win: WinState,
}

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

const POLLIN: i16 = 1;

// TEST: wayland_connect_binds_the_globals
unsafe extern "C" {
    fn poll(fds: *mut PollFd, nfds: c_ulong, timeout: i32) -> i32;
}

/// The data pointer of a listener, as the shared state.
///
/// # Safety
/// `data` is the address of the [`Shared`] the proxy was created for; it
/// outlives every proxy because `Shared::drop` destroys them.
// TEST: wayland_connect_binds_the_globals
unsafe fn shared_of<'a>(data: *mut c_void) -> &'a Shared {
    // TEST: wayland_connect_binds_the_globals
    // SAFETY: the caller's contract.
    unsafe { &*data.cast::<Shared>() }
}

extern "C" fn on_global(
    data: *mut c_void,
    _registry: *mut Proxy,
    name: u32,
    interface: *const c_char,
    version: u32,
) {
    // TEST: wayland_connect_binds_the_globals
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    // TEST: wayland_connect_binds_the_globals
    // SAFETY: `interface` is a NUL-terminated string valid for the call.
    let interface = unsafe { CStr::from_ptr(interface) }
        .to_string_lossy()
        .into_owned();
    shared.globals.borrow_mut().push(Global {
        name,
        interface,
        version,
    });
}

extern "C" fn on_ping(data: *mut c_void, wm_base: *mut Proxy, serial: u32) {
    // TEST: wayland_connect_binds_the_globals
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    // TEST: wayland_connect_binds_the_globals
    // SAFETY: `wm_base` is the live `xdg_wm_base` this event came from.
    unsafe { xdg_wm_base::pong(&shared.client, wm_base, serial) };
}

extern "C" fn on_output_scale(data: *mut c_void, output: *mut Proxy, scale: i32) {
    // TEST: wayland_output_scale_is_an_integer
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    for known in shared.outputs.borrow_mut().iter_mut() {
        if known.proxy == output {
            known.scale = scale.max(1);
        }
    }
}

static REGISTRY_LISTENER: wl_registry::Listener = wl_registry::Listener {
    global: on_global,
    ..wl_registry::IGNORE
};
static WM_BASE_LISTENER: xdg_wm_base::Listener = xdg_wm_base::Listener { ping: on_ping };
static OUTPUT_LISTENER: wl_output::Listener = wl_output::Listener {
    scale: on_output_scale,
    ..wl_output::IGNORE
};

impl Shared {
    /// The error text after a failed libwayland call.
    fn failure(&self, what: &str) -> WaylandError {
        // TEST: wayland_connect_binds_the_globals
        // SAFETY: `display` is live while `self` is.
        let code = unsafe { (self.client.display_get_error)(self.display) };
        WaylandError::Protocol(format!("{what} (error {code})"))
    }

    /// Handles what is already queued.
    fn dispatch_pending(&self) -> Result<(), WaylandError> {
        // TEST: wayland_connect_binds_the_globals
        // SAFETY: `display` is live while `self` is.
        let done = unsafe { (self.client.display_dispatch_pending)(self.display) };
        if done < 0 {
            return Err(self.failure("dispatch"));
        }
        Ok(())
    }

    /// Reads what the compositor sent (waiting up to `timeout_ms`) and
    /// handles it.
    pub(crate) fn dispatch(&self, timeout_ms: i32) -> Result<(), WaylandError> {
        let c = &self.client;
        loop {
            // TEST: wayland_connect_binds_the_globals
            // SAFETY: `display` is live while `self` is.
            if unsafe { (c.display_prepare_read)(self.display) } == 0 {
                break;
            }
            self.dispatch_pending()?;
        }
        // A full socket buffer (EAGAIN) is retried by the next call.
        // TEST: wayland_connect_binds_the_globals
        // SAFETY: `display` is live while `self` is.
        let _ = unsafe { (c.display_flush)(self.display) };
        let mut fd = PollFd {
            // TEST: wayland_connect_binds_the_globals
            // SAFETY: `display` is live while `self` is.
            fd: unsafe { (c.display_get_fd)(self.display) },
            events: POLLIN,
            revents: 0,
        };
        // TEST: wayland_connect_binds_the_globals
        // SAFETY: one valid `pollfd` that lives through the call.
        let ready = unsafe { poll(&raw mut fd, 1, timeout_ms) };
        if ready > 0 && fd.revents & POLLIN != 0 {
            // TEST: wayland_connect_binds_the_globals
            // SAFETY: `prepare_read` succeeded above and is paired once.
            if unsafe { (c.display_read_events)(self.display) } < 0 {
                return Err(self.failure("read"));
            }
        } else {
            // TEST: wayland_connect_binds_the_globals
            // SAFETY: `prepare_read` succeeded above and is paired once.
            unsafe { (c.display_cancel_read)(self.display) };
        }
        self.dispatch_pending()
    }

    /// Dispatches until `done` holds, or fails after [`TIMEOUT`].
    pub(crate) fn wait_until(
        &self,
        done: impl Fn() -> bool,
        timeout: Duration,
    ) -> Result<(), WaylandError> {
        let deadline = Instant::now() + timeout;
        loop {
            if done() {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(WaylandError::Timeout);
            }
            self.dispatch(20)?;
        }
    }

    /// Waits until the compositor has handled everything sent so far.
    pub(crate) fn roundtrip(&self) -> Result<(), WaylandError> {
        // TEST: wayland_connect_binds_the_globals
        // SAFETY: `display` is live while `self` is.
        if unsafe { (self.client.display_roundtrip)(self.display) } < 0 {
            return Err(self.failure("roundtrip"));
        }
        Ok(())
    }

    /// The integer buffer scale of the window: the largest scale among the
    /// outputs it is on, or among all outputs before it is on any.
    pub(crate) fn scale(&self) -> i32 {
        let outputs = self.outputs.borrow();
        let entered = self.win.entered.borrow();
        let on_window = outputs
            .iter()
            .filter(|o| entered.contains(&(o.proxy as usize)))
            .map(|o| o.scale)
            .max();
        on_window
            .or_else(|| outputs.iter().map(|o| o.scale).max())
            .unwrap_or(1)
            .max(1)
    }

    /// Binds the globals the backend uses.
    fn bind_globals(&self) -> Result<(), WaylandError> {
        let globals = self.globals.borrow();
        let data = std::ptr::from_ref(self).cast_mut().cast::<c_void>();
        for global in globals.iter() {
            let c = &self.client;
            let registry = self.registry.get();
            let (interface, ours): (*const _, u32) = match global.interface.as_str() {
                "wl_compositor" => (&raw const wl_compositor::INTERFACE, wl_compositor::VERSION),
                "xdg_wm_base" => (
                    &raw const crate::wayland::generated::xdg_shell::xdg_wm_base::INTERFACE,
                    xdg_wm_base::VERSION,
                ),
                "wl_output" => (&raw const wl_output::INTERFACE, wl_output::VERSION),
                _ => continue,
            };
            let version = global.version.min(ours);
            // TEST: wayland_connect_binds_the_globals
            // SAFETY: `registry` is live; `interface` is a generated table
            // and `name` and `version` come from the registry.
            let proxy = unsafe { wl_registry::bind(c, registry, global.name, interface, version) };
            if proxy.is_null() {
                return Err(WaylandError::NoObject("wl_registry.bind"));
            }
            match global.interface.as_str() {
                "wl_compositor" => self.compositor.set(proxy),
                "xdg_wm_base" => {
                    self.wm_base.set(proxy);
                    // TEST: wayland_connect_binds_the_globals
                    // SAFETY: `proxy` is the live `xdg_wm_base` just bound;
                    // `data` outlives it.
                    unsafe { xdg_wm_base::add_listener(c, proxy, &WM_BASE_LISTENER, data) };
                }
                _ => {
                    self.outputs.borrow_mut().push(Output { proxy, scale: 1 });
                    // TEST: wayland_output_scale_is_an_integer
                    // SAFETY: `proxy` is the live `wl_output` just bound;
                    // `data` outlives it.
                    unsafe { wl_output::add_listener(c, proxy, &OUTPUT_LISTENER, data) };
                }
            }
        }
        Ok(())
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        let c = &self.client;
        for output in self.outputs.borrow().iter() {
            // TEST: wayland_connect_binds_the_globals
            // SAFETY: the output proxies are live and destroyed once, here.
            unsafe { (c.proxy_destroy)(output.proxy) };
        }
        if !self.wm_base.get().is_null() {
            // TEST: wayland_connect_binds_the_globals
            // SAFETY: live, destroyed once, here (a destructor request).
            unsafe { xdg_wm_base::destroy(c, self.wm_base.get()) };
        }
        for proxy in [self.compositor.get(), self.registry.get()] {
            if !proxy.is_null() {
                // TEST: wayland_connect_binds_the_globals
                // SAFETY: live, destroyed once, here.
                unsafe { (c.proxy_destroy)(proxy) };
            }
        }
        // TEST: wayland_connect_binds_the_globals
        // SAFETY: `display` came from `wl_display_connect` and is closed
        // once, after every proxy of it is gone.
        unsafe { (c.display_disconnect)(self.display) };
    }
}

/// A connection to a Wayland compositor with its globals bound.
#[derive(Debug, Clone)]
pub struct Connection {
    pub(crate) shared: Rc<Shared>,
}

impl Connection {
    /// Connects to the compositor at `socket`, or to the one the
    /// environment names (`WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`) when `None`.
    ///
    /// # Errors
    /// [`WaylandError`] if the libraries or the socket are missing, or the
    /// compositor lacks `wl_compositor` or `xdg_wm_base`.
    pub fn connect(socket: Option<&Path>) -> Result<Self, WaylandError> {
        let client = Client::load().map_err(WaylandError::Load)?;
        let name = match socket {
            Some(path) => Some(
                CString::new(path.as_os_str().as_bytes())
                    .map_err(|_| WaylandError::Connect("socket path has a NUL byte".to_owned()))?,
            ),
            None => None,
        };
        // TEST: wayland_missing_server_is_an_error_value
        // SAFETY: `name` is null or a NUL-terminated string that lives
        // through the call.
        let display = unsafe {
            (client.display_connect)(name.as_ref().map_or(std::ptr::null(), |n| n.as_ptr()))
        };
        if display.is_null() {
            let what = socket.map_or_else(
                || "the default socket".to_owned(),
                |p| p.display().to_string(),
            );
            return Err(WaylandError::Connect(what));
        }
        let shared = Rc::new(Shared {
            client,
            display,
            registry: Cell::new(null_mut()),
            globals: RefCell::new(Vec::new()),
            compositor: Cell::new(null_mut()),
            wm_base: Cell::new(null_mut()),
            outputs: RefCell::new(Vec::new()),
            win: WinState::default(),
        });
        let data = Rc::as_ptr(&shared).cast_mut().cast::<c_void>();
        // TEST: wayland_connect_binds_the_globals
        // SAFETY: `display` is the live display of `shared`.
        let registry = unsafe {
            crate::wayland::generated::wayland::wl_display::get_registry(&shared.client, display)
        };
        if registry.is_null() {
            return Err(WaylandError::NoObject("wl_display.get_registry"));
        }
        shared.registry.set(registry);
        // TEST: wayland_connect_binds_the_globals
        // SAFETY: `registry` is live; `data` is the address of `shared`,
        // which outlives the proxy (`Shared::drop` destroys it).
        unsafe { wl_registry::add_listener(&shared.client, registry, &REGISTRY_LISTENER, data) };
        shared.roundtrip()?;
        shared.bind_globals()?;
        shared.roundtrip()?;
        if shared.compositor.get().is_null() {
            return Err(WaylandError::MissingGlobal("wl_compositor"));
        }
        if shared.wm_base.get().is_null() {
            return Err(WaylandError::MissingGlobal("xdg_wm_base"));
        }
        Ok(Self { shared })
    }

    /// Whether the compositor announced a global with this interface name.
    #[must_use]
    pub fn has_global(&self, interface: &str) -> bool {
        self.shared
            .globals
            .borrow()
            .iter()
            .any(|g| g.interface == interface)
    }

    /// Reads and handles the compositor's events, waiting at most
    /// `timeout` for the first.
    ///
    /// # Errors
    /// [`WaylandError::Protocol`] if the connection broke.
    pub fn dispatch(&self, timeout: Duration) -> Result<(), WaylandError> {
        let ms = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
        self.shared.dispatch(ms)
    }
}
