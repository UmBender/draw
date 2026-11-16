//! Generated from `protocols/xdg-shell.xml` by the generator of
//! `tests/wayland_generator.rs` (ADR-T168-1). Do not edit: regenerate with
//! `cargo test -p draw-sys --test wayland_generator -- --ignored write_generated_protocol`.

#![allow(missing_docs, unsafe_code, unused_imports, clippy::all, clippy::pedantic)]

/// The `xdg_wm_base` interface.
pub mod xdg_wm_base {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 7;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_2: TypeTable<2> = TypeTable([&raw const crate::wayland::generated::xdg_shell::xdg_surface::INTERFACE, &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE]);
    static REQUEST_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static REQUESTS: MessageTable<4> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"create_positioner".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }, Message { name: c"get_xdg_surface".as_ptr(), signature: c"no".as_ptr(), types: REQUEST_TYPES_2.0.as_ptr() }, Message { name: c"pong".as_ptr(), signature: c"u".as_ptr(), types: REQUEST_TYPES_3.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<1> = MessageTable([Message { name: c"ping".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"xdg_wm_base".as_ptr(),
        version: 7,
        method_count: 4,
        methods: REQUESTS.0.as_ptr(),
        event_count: 1,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `ping`.
        pub ping: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_ping(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        ping: ignore_ping,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_wm_base` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `xdg_wm_base`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_wm_base` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

    // `create_positioner` is not wrapped: it creates `xdg_positioner`, which is not generated.

    // TEST: wayland_window_configures_with_a_size
    /// Request `get_xdg_surface` of `xdg_wm_base`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_wm_base` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn get_xdg_surface(client: &Client, proxy: *mut Proxy, surface: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 2, &raw const crate::wayland::generated::xdg_shell::xdg_surface::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>(), surface) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `pong` of `xdg_wm_base`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_wm_base` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn pong(client: &Client, proxy: *mut Proxy, serial: u32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 3, null(), (client.get_version)(proxy), 0, serial) };
    }

}

/// The `xdg_surface` interface.
pub mod xdg_surface {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 7;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_1: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::xdg_shell::xdg_toplevel::INTERFACE]);
    static REQUEST_TYPES_2: TypeTable<3> = TypeTable([null(), &raw const crate::wayland::generated::xdg_shell::xdg_surface::INTERFACE, null()]);
    static REQUEST_TYPES_3: TypeTable<4> = TypeTable([null(), null(), null(), null()]);
    static REQUEST_TYPES_4: TypeTable<1> = TypeTable([null()]);
    static REQUESTS: MessageTable<5> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"get_toplevel".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }, Message { name: c"get_popup".as_ptr(), signature: c"n?oo".as_ptr(), types: REQUEST_TYPES_2.0.as_ptr() }, Message { name: c"set_window_geometry".as_ptr(), signature: c"iiii".as_ptr(), types: REQUEST_TYPES_3.0.as_ptr() }, Message { name: c"ack_configure".as_ptr(), signature: c"u".as_ptr(), types: REQUEST_TYPES_4.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<1> = MessageTable([Message { name: c"configure".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"xdg_surface".as_ptr(),
        version: 7,
        method_count: 5,
        methods: REQUESTS.0.as_ptr(),
        event_count: 1,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `configure`.
        pub configure: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_configure(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        configure: ignore_configure,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_surface` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `xdg_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `get_toplevel` of `xdg_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn get_toplevel(client: &Client, proxy: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, &raw const crate::wayland::generated::xdg_shell::xdg_toplevel::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>()) }
    }

    // `get_popup` is not wrapped: it creates `xdg_popup`, which is not generated.

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_window_geometry` of `xdg_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_window_geometry(client: &Client, proxy: *mut Proxy, x: i32, y: i32, width: i32, height: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 3, null(), (client.get_version)(proxy), 0, x, y, width, height) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `ack_configure` of `xdg_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn ack_configure(client: &Client, proxy: *mut Proxy, serial: u32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 4, null(), (client.get_version)(proxy), 0, serial) };
    }

}

/// The `xdg_toplevel` interface.
pub mod xdg_toplevel {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 7;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_1: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::xdg_shell::xdg_toplevel::INTERFACE]);
    static REQUEST_TYPES_2: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_4: TypeTable<4> = TypeTable([&raw const crate::wayland::generated::wayland::wl_seat::INTERFACE, null(), null(), null()]);
    static REQUEST_TYPES_5: TypeTable<2> = TypeTable([&raw const crate::wayland::generated::wayland::wl_seat::INTERFACE, null()]);
    static REQUEST_TYPES_6: TypeTable<3> = TypeTable([&raw const crate::wayland::generated::wayland::wl_seat::INTERFACE, null(), null()]);
    static REQUEST_TYPES_7: TypeTable<2> = TypeTable([null(), null()]);
    static REQUEST_TYPES_8: TypeTable<2> = TypeTable([null(), null()]);
    static REQUEST_TYPES_9: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_10: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_11: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_output::INTERFACE]);
    static REQUEST_TYPES_12: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_13: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<14> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"set_parent".as_ptr(), signature: c"?o".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }, Message { name: c"set_title".as_ptr(), signature: c"s".as_ptr(), types: REQUEST_TYPES_2.0.as_ptr() }, Message { name: c"set_app_id".as_ptr(), signature: c"s".as_ptr(), types: REQUEST_TYPES_3.0.as_ptr() }, Message { name: c"show_window_menu".as_ptr(), signature: c"ouii".as_ptr(), types: REQUEST_TYPES_4.0.as_ptr() }, Message { name: c"move".as_ptr(), signature: c"ou".as_ptr(), types: REQUEST_TYPES_5.0.as_ptr() }, Message { name: c"resize".as_ptr(), signature: c"ouu".as_ptr(), types: REQUEST_TYPES_6.0.as_ptr() }, Message { name: c"set_max_size".as_ptr(), signature: c"ii".as_ptr(), types: REQUEST_TYPES_7.0.as_ptr() }, Message { name: c"set_min_size".as_ptr(), signature: c"ii".as_ptr(), types: REQUEST_TYPES_8.0.as_ptr() }, Message { name: c"set_maximized".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_9.0.as_ptr() }, Message { name: c"unset_maximized".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_10.0.as_ptr() }, Message { name: c"set_fullscreen".as_ptr(), signature: c"?o".as_ptr(), types: REQUEST_TYPES_11.0.as_ptr() }, Message { name: c"unset_fullscreen".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_12.0.as_ptr() }, Message { name: c"set_minimized".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_13.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_1: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_2: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<4> = MessageTable([Message { name: c"configure".as_ptr(), signature: c"iia".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"close".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"configure_bounds".as_ptr(), signature: c"4ii".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"wm_capabilities".as_ptr(), signature: c"5a".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"xdg_toplevel".as_ptr(),
        version: 7,
        method_count: 14,
        methods: REQUESTS.0.as_ptr(),
        event_count: 4,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `configure`.
        pub configure: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32, i32, *mut c_void), // TEST: wayland_window_configures_with_a_size
        /// Event `close`.
        pub close: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `configure_bounds`.
        pub configure_bounds: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `wm_capabilities`.
        pub wm_capabilities: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut c_void), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_configure(_: *mut c_void, _: *mut Proxy, _: i32, _: i32, _: *mut c_void) {}
    extern "C" fn ignore_close(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_configure_bounds(_: *mut c_void, _: *mut Proxy, _: i32, _: i32) {}
    extern "C" fn ignore_wm_capabilities(_: *mut c_void, _: *mut Proxy, _: *mut c_void) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        configure: ignore_configure,
        close: ignore_close,
        configure_bounds: ignore_configure_bounds,
        wm_capabilities: ignore_wm_capabilities,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_parent` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_parent(client: &Client, proxy: *mut Proxy, parent: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 0, parent) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_title` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_title(client: &Client, proxy: *mut Proxy, title: *const c_char) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 2, null(), (client.get_version)(proxy), 0, title) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_app_id` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_app_id(client: &Client, proxy: *mut Proxy, app_id: *const c_char) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 3, null(), (client.get_version)(proxy), 0, app_id) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `show_window_menu` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn show_window_menu(client: &Client, proxy: *mut Proxy, seat: *mut Proxy, serial: u32, x: i32, y: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 4, null(), (client.get_version)(proxy), 0, seat, serial, x, y) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `move` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn r#move(client: &Client, proxy: *mut Proxy, seat: *mut Proxy, serial: u32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 5, null(), (client.get_version)(proxy), 0, seat, serial) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `resize` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn resize(client: &Client, proxy: *mut Proxy, seat: *mut Proxy, serial: u32, edges: u32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 6, null(), (client.get_version)(proxy), 0, seat, serial, edges) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_max_size` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_max_size(client: &Client, proxy: *mut Proxy, width: i32, height: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 7, null(), (client.get_version)(proxy), 0, width, height) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_min_size` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_min_size(client: &Client, proxy: *mut Proxy, width: i32, height: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 8, null(), (client.get_version)(proxy), 0, width, height) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_maximized` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_maximized(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 9, null(), (client.get_version)(proxy), 0) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `unset_maximized` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn unset_maximized(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 10, null(), (client.get_version)(proxy), 0) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_fullscreen` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_fullscreen(client: &Client, proxy: *mut Proxy, output: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 11, null(), (client.get_version)(proxy), 0, output) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `unset_fullscreen` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn unset_fullscreen(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 12, null(), (client.get_version)(proxy), 0) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_minimized` of `xdg_toplevel`.
    ///
    /// # Safety
    /// `proxy` is a live `xdg_toplevel` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_minimized(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 13, null(), (client.get_version)(proxy), 0) };
    }

}
