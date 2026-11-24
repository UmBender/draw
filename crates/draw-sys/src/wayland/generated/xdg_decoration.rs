//! Generated from `protocols/xdg-decoration-unstable-v1.xml` by the generator of
//! `tests/wayland_generator.rs` (ADR-T168-1). Do not edit: regenerate with
//! `cargo test -p draw-sys --test wayland_generator -- --ignored write_generated_protocol`.

#![allow(missing_docs, unsafe_code, unused_imports, clippy::all, clippy::pedantic)]

/// The `zxdg_decoration_manager_v1` interface.
pub mod zxdg_decoration_manager_v1 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_1: TypeTable<2> = TypeTable([&raw const crate::wayland::generated::xdg_decoration::zxdg_toplevel_decoration_v1::INTERFACE, &raw const crate::wayland::generated::xdg_shell::xdg_toplevel::INTERFACE]);
    static REQUESTS: MessageTable<2> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"get_toplevel_decoration".as_ptr(), signature: c"no".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }]);

    static EVENTS: MessageTable<0> = MessageTable([]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zxdg_decoration_manager_v1".as_ptr(),
        version: 2,
        method_count: 2,
        methods: REQUESTS.0.as_ptr(),
        event_count: 0,
        events: EVENTS.0.as_ptr(),
    };

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zxdg_decoration_manager_v1`.
    ///
    /// # Safety
    /// `proxy` is a live `zxdg_decoration_manager_v1` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `get_toplevel_decoration` of `zxdg_decoration_manager_v1`.
    ///
    /// # Safety
    /// `proxy` is a live `zxdg_decoration_manager_v1` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn get_toplevel_decoration(client: &Client, proxy: *mut Proxy, toplevel: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, &raw const crate::wayland::generated::xdg_decoration::zxdg_toplevel_decoration_v1::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>(), toplevel) }
    }

}

/// The `zxdg_toplevel_decoration_v1` interface.
pub mod zxdg_toplevel_decoration_v1 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_2: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<3> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"set_mode".as_ptr(), signature: c"u".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }, Message { name: c"unset_mode".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_2.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<1> = MessageTable([Message { name: c"configure".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zxdg_toplevel_decoration_v1".as_ptr(),
        version: 2,
        method_count: 3,
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
    /// `proxy` is a live `zxdg_toplevel_decoration_v1` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zxdg_toplevel_decoration_v1`.
    ///
    /// # Safety
    /// `proxy` is a live `zxdg_toplevel_decoration_v1` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_mode` of `zxdg_toplevel_decoration_v1`.
    ///
    /// # Safety
    /// `proxy` is a live `zxdg_toplevel_decoration_v1` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_mode(client: &Client, proxy: *mut Proxy, mode: u32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 0, mode) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `unset_mode` of `zxdg_toplevel_decoration_v1`.
    ///
    /// # Safety
    /// `proxy` is a live `zxdg_toplevel_decoration_v1` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn unset_mode(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 2, null(), (client.get_version)(proxy), 0) };
    }

}
