//! `libwayland-client` and `libwayland-egl`, loaded with `dlopen` through
//! the [`loader`](crate::loader) (ADR-T142-1): the function table the
//! generated code and the connection call through.

#![allow(unsafe_code)]

use std::ffi::{c_char, c_void};

use crate::loader::{Library, LoadError};
use crate::wayland::ffi::Interface;

/// An opaque `wl_proxy` (a `wl_display` is one too).
#[repr(C)]
#[derive(Debug)]
pub struct Proxy {
    _private: [u8; 0],
}

/// The functions of the two libraries. The libraries are kept loaded as
/// long as the table lives; the function pointers are copies that are valid
/// until then.
#[derive(Debug)]
pub struct Client {
    /// `wl_proxy_marshal_flags`.
    pub marshal_flags:
        unsafe extern "C" fn(*mut Proxy, u32, *const Interface, u32, u32, ...) -> *mut Proxy, // TEST: wayland_connect_binds_the_globals
    /// `wl_proxy_add_listener`.
    pub add_listener: unsafe extern "C" fn(*mut Proxy, *const c_void, *mut c_void) -> i32, // TEST: wayland_connect_binds_the_globals
    /// `wl_proxy_get_version`.
    pub get_version: unsafe extern "C" fn(*mut Proxy) -> u32, // TEST: wayland_connect_binds_the_globals
    /// `wl_proxy_destroy`.
    pub proxy_destroy: unsafe extern "C" fn(*mut Proxy), // TEST: wayland_connect_binds_the_globals
    /// `wl_display_connect`.
    pub display_connect: unsafe extern "C" fn(*const c_char) -> *mut Proxy, // TEST: wayland_connect_binds_the_globals
    /// `wl_display_disconnect`.
    pub display_disconnect: unsafe extern "C" fn(*mut Proxy), // TEST: wayland_connect_binds_the_globals
    /// `wl_display_get_fd`.
    pub display_get_fd: unsafe extern "C" fn(*mut Proxy) -> i32, // TEST: wayland_connect_binds_the_globals
    /// `wl_display_dispatch_pending`.
    pub display_dispatch_pending: unsafe extern "C" fn(*mut Proxy) -> i32, // TEST: wayland_connect_binds_the_globals
    /// `wl_display_flush`.
    pub display_flush: unsafe extern "C" fn(*mut Proxy) -> i32, // TEST: wayland_connect_binds_the_globals
    /// `wl_display_roundtrip`.
    pub display_roundtrip: unsafe extern "C" fn(*mut Proxy) -> i32, // TEST: wayland_connect_binds_the_globals
    /// `wl_display_prepare_read`.
    pub display_prepare_read: unsafe extern "C" fn(*mut Proxy) -> i32, // TEST: wayland_connect_binds_the_globals
    /// `wl_display_read_events`.
    pub display_read_events: unsafe extern "C" fn(*mut Proxy) -> i32, // TEST: wayland_connect_binds_the_globals
    /// `wl_display_cancel_read`.
    pub display_cancel_read: unsafe extern "C" fn(*mut Proxy), // TEST: wayland_connect_binds_the_globals
    /// `wl_display_get_error`.
    pub display_get_error: unsafe extern "C" fn(*mut Proxy) -> i32, // TEST: wayland_connect_binds_the_globals
    /// `wl_egl_window_create`.
    pub egl_window_create: unsafe extern "C" fn(*mut Proxy, i32, i32) -> *mut c_void, // TEST: wayland_egl_context_renders_and_swaps
    /// `wl_egl_window_resize`.
    pub egl_window_resize: unsafe extern "C" fn(*mut c_void, i32, i32, i32, i32), // TEST: wayland_resize_event_follows_the_compositor
    /// `wl_egl_window_destroy`.
    pub egl_window_destroy: unsafe extern "C" fn(*mut c_void), // TEST: wayland_egl_context_renders_and_swaps
    _egl_lib: Library,
    _core_lib: Library,
}

/// Loads `name` from `library`, typed as the field it initialises.
fn symbol<F: Copy>(library: &Library, name: &str) -> Result<F, LoadError> {
    // TEST: wayland_connect_binds_the_globals
    // SAFETY: every call site assigns the result to a field whose type is
    // the C signature of the named libwayland function.
    unsafe { library.symbol::<F>(name) }.map(|s| *s)
}

impl Client {
    /// Loads `libwayland-client.so.0` and `libwayland-egl.so.1`.
    ///
    /// # Errors
    /// [`LoadError`] if a library or a function is missing.
    pub fn load() -> Result<Self, LoadError> {
        let client = Library::open("libwayland-client.so.0")?;
        let egl = Library::open("libwayland-egl.so.1")?;
        Ok(Self {
            marshal_flags: symbol(&client, "wl_proxy_marshal_flags")?,
            add_listener: symbol(&client, "wl_proxy_add_listener")?,
            get_version: symbol(&client, "wl_proxy_get_version")?,
            proxy_destroy: symbol(&client, "wl_proxy_destroy")?,
            display_connect: symbol(&client, "wl_display_connect")?,
            display_disconnect: symbol(&client, "wl_display_disconnect")?,
            display_get_fd: symbol(&client, "wl_display_get_fd")?,
            display_dispatch_pending: symbol(&client, "wl_display_dispatch_pending")?,
            display_flush: symbol(&client, "wl_display_flush")?,
            display_roundtrip: symbol(&client, "wl_display_roundtrip")?,
            display_prepare_read: symbol(&client, "wl_display_prepare_read")?,
            display_read_events: symbol(&client, "wl_display_read_events")?,
            display_cancel_read: symbol(&client, "wl_display_cancel_read")?,
            display_get_error: symbol(&client, "wl_display_get_error")?,
            egl_window_create: symbol(&egl, "wl_egl_window_create")?,
            egl_window_resize: symbol(&egl, "wl_egl_window_resize")?,
            egl_window_destroy: symbol(&egl, "wl_egl_window_destroy")?,
            _egl_lib: egl,
            _core_lib: client,
        })
    }
}
