//! An `xdg_toplevel` window whose protocol order is a type state: `Created`
//! (no buffer may be attached or committed, the first `configure` is
//! awaited), `Configured` (the first `configure` is acked) and `Mapped` (a
//! GL context draws into it). Attaching, committing and acking twice in the
//! wrong state do not compile (ADR-T168-1).

#![allow(unsafe_code)]

use std::ffi::{CString, c_void};
use std::rc::Rc;

use crate::egl::{ContextError, EglContext, GlContext, Platform};
use crate::wayland::client::Proxy;
use crate::wayland::connection::{Connection, Shared, TIMEOUT, WaylandError};
use crate::wayland::generated::wayland::{wl_callback, wl_compositor, wl_surface};
use crate::wayland::generated::xdg_shell::{xdg_surface, xdg_toplevel, xdg_wm_base};

/// Size used when the compositor lets the client choose.
const DEFAULT_SIZE: (i32, i32) = (640, 480);

/// State: the objects exist, nothing is acked yet.
#[derive(Debug)]
pub struct Created;

/// State: the first `configure` is acked.
#[derive(Debug)]
pub struct Configured;

/// State: a GL context draws into the window.
#[derive(Debug)]
pub struct Mapped {
    gl: GlPart,
}

/// A GL context and the `wl_egl_window` under it. The context is declared
/// first, so it is dropped before the native window.
#[derive(Debug)]
struct GlPart {
    context: EglContext,
    native: NativeWindow,
}

/// A `wl_egl_window`.
#[derive(Debug)]
struct NativeWindow {
    ptr: *mut c_void,
    shared: Rc<Shared>,
}

impl Drop for NativeWindow {
    fn drop(&mut self) {
        // TEST: wayland_egl_context_renders_and_swaps
        // SAFETY: `ptr` came from `wl_egl_window_create` and is destroyed
        // once, here, after the EGL surface on it.
        unsafe { (self.shared.client.egl_window_destroy)(self.ptr) };
    }
}

/// The protocol objects of a window, destroyed in reverse order.
#[derive(Debug)]
struct Objects {
    shared: Rc<Shared>,
    surface: *mut Proxy,
    xdg_surface: *mut Proxy,
    toplevel: *mut Proxy,
}

impl Drop for Objects {
    fn drop(&mut self) {
        let c = &self.shared.client;
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the three proxies are live and destroyed once, here, in
        // reverse order of creation (destructor requests).
        unsafe {
            xdg_toplevel::destroy(c, self.toplevel);
            xdg_surface::destroy(c, self.xdg_surface);
            wl_surface::destroy(c, self.surface);
        }
    }
}

/// A window in state `S`.
#[derive(Debug)]
pub struct Window<S> {
    // Declared first: the GL part is dropped before the protocol objects.
    state: S,
    objects: Objects,
}

extern "C" fn on_toplevel_configure(
    data: *mut c_void,
    _toplevel: *mut Proxy,
    width: i32,
    height: i32,
    _states: *mut c_void,
) {
    // TEST: wayland_window_configures_with_a_size
    // SAFETY: libwayland passes the data pointer given to `add_listener`,
    // the address of the live `Shared`.
    let shared = unsafe { &*data.cast::<Shared>() };
    shared.win.pending.set((width, height));
}

extern "C" fn on_toplevel_close(data: *mut c_void, _toplevel: *mut Proxy) {
    // TEST: wayland_close_is_reported
    // SAFETY: as in `on_toplevel_configure`.
    let shared = unsafe { &*data.cast::<Shared>() };
    shared.win.close.set(true);
}

extern "C" fn on_xdg_surface_configure(data: *mut c_void, _surface: *mut Proxy, serial: u32) {
    // TEST: wayland_window_configures_with_a_size
    // SAFETY: as in `on_toplevel_configure`.
    let shared = unsafe { &*data.cast::<Shared>() };
    shared.win.serial.set(Some(serial));
}

extern "C" fn on_surface_enter(data: *mut c_void, _surface: *mut Proxy, output: *mut Proxy) {
    // TEST: wayland_output_scale_is_an_integer
    // SAFETY: as in `on_toplevel_configure`.
    let shared = unsafe { &*data.cast::<Shared>() };
    shared.win.entered.borrow_mut().push(output as usize);
}

extern "C" fn on_surface_leave(data: *mut c_void, _surface: *mut Proxy, output: *mut Proxy) {
    // TEST: wayland_output_scale_is_an_integer
    // SAFETY: as in `on_toplevel_configure`.
    let shared = unsafe { &*data.cast::<Shared>() };
    shared
        .win
        .entered
        .borrow_mut()
        .retain(|o| *o != output as usize);
}

extern "C" fn on_frame_done(data: *mut c_void, callback: *mut Proxy, _time: u32) {
    // TEST: wayland_frame_callback_resolves_next_frame
    // SAFETY: as in `on_toplevel_configure`.
    let shared = unsafe { &*data.cast::<Shared>() };
    shared.win.frame_pending.set(false);
    // TEST: wayland_frame_callback_resolves_next_frame
    // SAFETY: a `wl_callback` is done after its one event and has no
    // destructor request: the proxy is destroyed here, once.
    unsafe { (shared.client.proxy_destroy)(callback) };
}

static XDG_SURFACE_LISTENER: xdg_surface::Listener = xdg_surface::Listener {
    configure: on_xdg_surface_configure,
};
static TOPLEVEL_LISTENER: xdg_toplevel::Listener = xdg_toplevel::Listener {
    configure: on_toplevel_configure,
    close: on_toplevel_close,
    ..xdg_toplevel::IGNORE
};
static SURFACE_LISTENER: wl_surface::Listener = wl_surface::Listener {
    enter: on_surface_enter,
    leave: on_surface_leave,
    ..wl_surface::IGNORE
};
static CALLBACK_LISTENER: wl_callback::Listener = wl_callback::Listener {
    done: on_frame_done,
};

/// A C string for a title: NUL bytes are dropped.
fn c_text(text: &str) -> CString {
    CString::new(text.replace('\0', "")).unwrap_or_default()
}

impl Connection {
    /// Creates a toplevel window with a title and an application id. The
    /// window is `Created`: wait for its first configure with
    /// [`Window::wait_configure`].
    ///
    /// # Errors
    /// [`WaylandError::NoObject`] if the compositor refuses an object.
    pub fn create_window(
        &self,
        title: &str,
        app_id: &str,
    ) -> Result<Window<Created>, WaylandError> {
        let shared = &self.shared;
        let c = &shared.client;
        let data = Rc::as_ptr(shared).cast_mut().cast::<c_void>();
        shared.win.serial.set(None);
        shared.win.close.set(false);
        shared.win.entered.borrow_mut().clear();
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the compositor proxy is live (bound in `connect`).
        let surface = unsafe { wl_compositor::create_surface(c, shared.compositor.get()) };
        if surface.is_null() {
            return Err(WaylandError::NoObject("wl_compositor.create_surface"));
        }
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: `wm_base` and `surface` are live proxies of this
        // connection.
        let xdg = unsafe { xdg_wm_base::get_xdg_surface(c, shared.wm_base.get(), surface) };
        if xdg.is_null() {
            // TEST: wayland_window_configures_with_a_size
            // SAFETY: `surface` is live and not used again.
            unsafe { wl_surface::destroy(c, surface) };
            return Err(WaylandError::NoObject("xdg_wm_base.get_xdg_surface"));
        }
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: `xdg` is live.
        let toplevel = unsafe { xdg_surface::get_toplevel(c, xdg) };
        if toplevel.is_null() {
            // TEST: wayland_window_configures_with_a_size
            // SAFETY: both are live and not used again.
            unsafe {
                xdg_surface::destroy(c, xdg);
                wl_surface::destroy(c, surface);
            }
            return Err(WaylandError::NoObject("xdg_surface.get_toplevel"));
        }
        let title = c_text(title);
        let app_id = c_text(app_id);
        // TEST: wayland_window_sets_title_and_app_id
        // SAFETY: the proxies are live, the strings live through the calls
        // and `data` outlives the proxies (they die with the `Objects`,
        // which hold the `Rc`).
        unsafe {
            xdg_surface::add_listener(c, xdg, &XDG_SURFACE_LISTENER, data);
            xdg_toplevel::add_listener(c, toplevel, &TOPLEVEL_LISTENER, data);
            wl_surface::add_listener(c, surface, &SURFACE_LISTENER, data);
            xdg_toplevel::set_title(c, toplevel, title.as_ptr());
            xdg_toplevel::set_app_id(c, toplevel, app_id.as_ptr());
        }
        Ok(Window {
            state: Created,
            objects: Objects {
                shared: Rc::clone(shared),
                surface,
                xdg_surface: xdg,
                toplevel,
            },
        })
    }
}

impl<S> Window<S> {
    /// The logical size, `(0, 0)` before the first configure.
    #[must_use]
    pub fn size(&self) -> (i32, i32) {
        self.objects.shared.win.size.get()
    }

    /// The integer buffer scale.
    #[must_use]
    pub fn scale(&self) -> i32 {
        self.objects.shared.scale()
    }

    /// Whether the compositor asked the window to close.
    #[must_use]
    pub fn close_requested(&self) -> bool {
        self.objects.shared.win.close.get()
    }

    /// Whether a frame callback is outstanding.
    #[must_use]
    pub fn frame_pending(&self) -> bool {
        self.objects.shared.win.frame_pending.get()
    }

    fn next_state<T>(self, state: T) -> Window<T> {
        Window {
            state,
            objects: self.objects,
        }
    }
}

impl Window<Created> {
    /// The first, bare commit that makes the compositor send `configure`.
    pub fn commit_initial(&self) {
        let o = &self.objects;
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the surface is live while `self` is.
        unsafe { wl_surface::commit(&o.shared.client, o.surface) };
    }

    /// Acks the first `configure` and takes its size. Consumes the window:
    /// acking a second time does not compile.
    #[must_use]
    pub fn ack_configure(self, serial: u32) -> Window<Configured> {
        let o = &self.objects;
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the xdg surface is live while `self` is.
        unsafe { xdg_surface::ack_configure(&o.shared.client, o.xdg_surface, serial) };
        o.shared.apply_pending_size(DEFAULT_SIZE);
        self.next_state(Configured)
    }

    /// Commits, waits for the first `configure` and acks it.
    ///
    /// # Errors
    /// [`WaylandError::Timeout`] or [`WaylandError::Protocol`].
    pub fn wait_configure(self) -> Result<Window<Configured>, WaylandError> {
        self.commit_initial();
        let shared = Rc::clone(&self.objects.shared);
        shared.wait_until(|| shared.win.serial.get().is_some(), TIMEOUT)?;
        let serial = shared.win.serial.take().unwrap_or(0);
        Ok(self.ack_configure(serial))
    }
}

impl Window<Configured> {
    /// Commits the surface.
    pub fn commit(&self) {
        self.objects.commit();
    }

    // TEST: wayland_egl_context_renders_and_swaps
    /// Attaches a `wl_buffer` (or none) at `x`, `y`.
    ///
    /// # Safety
    /// `buffer` is null or a live `wl_buffer` proxy of this connection.
    pub unsafe fn attach(&self, buffer: *mut Proxy, x: i32, y: i32) {
        // TEST: wayland_egl_context_renders_and_swaps
        // SAFETY: the caller's contract.
        unsafe { self.objects.attach(buffer, x, y) };
    }

    /// Opens an OpenGL ES 3.0 context on the window; it is made current.
    ///
    /// # Errors
    /// [`WaylandError::Context`] if EGL refuses, [`WaylandError::NoObject`]
    /// if there is no `wl_egl_window`.
    pub fn open_gl(self) -> Result<Window<Mapped>, WaylandError> {
        let shared = Rc::clone(&self.objects.shared);
        let scale = shared.scale();
        let (width, height) = shared.win.size.get();
        // TEST: wayland_egl_context_renders_and_swaps
        // SAFETY: the surface is live; the size is positive (configure).
        let ptr = unsafe {
            (shared.client.egl_window_create)(
                self.objects.surface,
                (width * scale).max(1),
                (height * scale).max(1),
            )
        };
        if ptr.is_null() {
            return Err(WaylandError::NoObject("wl_egl_window_create"));
        }
        let native = NativeWindow {
            ptr,
            shared: Rc::clone(&shared),
        };
        // TEST: wayland_egl_context_renders_and_swaps
        // SAFETY: `display` and `ptr` are a live `wl_display*` and
        // `wl_egl_window*`; the context is dropped before `native` (field
        // order of `GlPart`), and the display outlives both (`Rc`).
        let context =
            unsafe { EglContext::from_window(Platform::Wayland, shared.display.cast(), ptr) }
                .map_err(WaylandError::Context)?;
        context.make_current().map_err(WaylandError::Context)?;
        let o = &self.objects;
        // TEST: wayland_output_scale_is_an_integer
        // SAFETY: the surface is live.
        unsafe { wl_surface::set_buffer_scale(&shared.client, o.surface, scale) };
        shared.win.applied.set((width, height, scale));
        Ok(self.next_state(Mapped {
            gl: GlPart { context, native },
        }))
    }
}

impl Window<Mapped> {
    /// Commits the surface.
    pub fn commit(&self) {
        self.objects.commit();
    }

    // TEST: wayland_egl_context_renders_and_swaps
    /// Attaches a `wl_buffer` (or none) at `x`, `y`.
    ///
    /// # Safety
    /// `buffer` is null or a live `wl_buffer` proxy of this connection.
    pub unsafe fn attach(&self, buffer: *mut Proxy, x: i32, y: i32) {
        // TEST: wayland_egl_context_renders_and_swaps
        // SAFETY: the caller's contract.
        unsafe { self.objects.attach(buffer, x, y) };
    }

    /// The GL context.
    #[must_use]
    pub fn context(&self) -> &EglContext {
        &self.state.gl.context
    }

    /// Presents the back buffer.
    ///
    /// # Errors
    /// [`ContextError`] if EGL refuses.
    pub fn swap(&self) -> Result<(), ContextError> {
        self.state.gl.context.swap_buffers()
    }

    /// Asks for a frame callback unless one is outstanding; it fires after
    /// the next commit is shown.
    pub fn request_frame(&self) {
        let shared = &self.objects.shared;
        if shared.win.frame_pending.get() {
            return;
        }
        let c = &shared.client;
        let data = Rc::as_ptr(shared).cast_mut().cast::<c_void>();
        // TEST: wayland_schedule_update_requests_a_frame
        // SAFETY: the surface is live while `self` is.
        let callback = unsafe { wl_surface::frame(c, self.objects.surface) };
        if callback.is_null() {
            return;
        }
        // TEST: wayland_schedule_update_requests_a_frame
        // SAFETY: `callback` is live; `data` outlives it.
        unsafe { wl_callback::add_listener(c, callback, &CALLBACK_LISTENER, data) };
        shared.win.frame_pending.set(true);
    }

    /// Acks a pending `configure` and follows a change of size or scale:
    /// resizes the native window and sets the buffer scale.
    pub fn apply_configure(&self) {
        let shared = &self.objects.shared;
        let c = &shared.client;
        if let Some(serial) = shared.win.serial.take() {
            // TEST: wayland_resize_event_follows_the_compositor
            // SAFETY: the xdg surface is live while `self` is.
            unsafe { xdg_surface::ack_configure(c, self.objects.xdg_surface, serial) };
            shared.apply_pending_size(DEFAULT_SIZE);
        }
        let (width, height) = shared.win.size.get();
        let scale = shared.scale();
        let applied = shared.win.applied.get();
        if applied == (width, height, scale) {
            return;
        }
        // TEST: wayland_resize_event_follows_the_compositor
        // SAFETY: the native window is live while `self` is.
        unsafe {
            (c.egl_window_resize)(
                self.state.gl.native.ptr,
                (width * scale).max(1),
                (height * scale).max(1),
                0,
                0,
            );
        }
        if applied.2 != scale {
            // TEST: wayland_output_scale_is_an_integer
            // SAFETY: the surface is live while `self` is.
            unsafe { wl_surface::set_buffer_scale(c, self.objects.surface, scale) };
        }
        shared.win.applied.set((width, height, scale));
    }
}

impl Objects {
    fn commit(&self) {
        // TEST: wayland_egl_context_renders_and_swaps
        // SAFETY: the surface is live while `self` is.
        unsafe { wl_surface::commit(&self.shared.client, self.surface) };
    }

    /// # Safety
    /// `buffer` is null or a live `wl_buffer` proxy.
    unsafe fn attach(&self, buffer: *mut Proxy, x: i32, y: i32) {
        // TEST: wayland_egl_context_renders_and_swaps
        // SAFETY: the surface is live; the caller vouches for `buffer`.
        unsafe { wl_surface::attach(&self.shared.client, self.surface, buffer, x, y) };
    }
}

impl Shared {
    /// Takes the size of the last toplevel `configure`; a `0` side keeps
    /// the size in use, or `fallback` when there is none yet.
    pub(crate) fn apply_pending_size(&self, fallback: (i32, i32)) {
        let (pw, ph) = self.win.pending.get();
        let (cw, ch) = self.win.size.get();
        let width = if pw > 0 {
            pw
        } else if cw > 0 {
            cw
        } else {
            fallback.0
        };
        let height = if ph > 0 {
            ph
        } else if ch > 0 {
            ch
        } else {
            fallback.1
        };
        self.win.size.set((width, height));
    }
}
