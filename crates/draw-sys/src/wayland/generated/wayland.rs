//! Generated from `protocols/wayland.xml` by the generator of
//! `tests/wayland_generator.rs` (ADR-T168-1). Do not edit: regenerate with
//! `cargo test -p draw-sys --test wayland_generator -- --ignored write_generated_protocol`.

#![allow(missing_docs, unsafe_code, unused_imports, clippy::all, clippy::pedantic)]

/// The `wl_display` interface.
pub mod wl_display {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 1;

    static REQUEST_TYPES_0: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_callback::INTERFACE]);
    static REQUEST_TYPES_1: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_registry::INTERFACE]);
    static REQUESTS: MessageTable<2> = MessageTable([Message { name: c"sync".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"get_registry".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<2> = MessageTable([Message { name: c"error".as_ptr(), signature: c"ous".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"delete_id".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_display".as_ptr(),
        version: 1,
        method_count: 2,
        methods: REQUESTS.0.as_ptr(),
        event_count: 2,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `error`.
        pub error: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy, u32, *const c_char), // TEST: wayland_window_configures_with_a_size
        /// Event `delete_id`.
        pub delete_id: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_error(_: *mut c_void, _: *mut Proxy, _: *mut Proxy, _: u32, _: *const c_char) {}
    extern "C" fn ignore_delete_id(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        error: ignore_error,
        delete_id: ignore_delete_id,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `wl_display` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `sync` of `wl_display`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_display` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn sync(client: &Client, proxy: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, &raw const crate::wayland::generated::wayland::wl_callback::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>()) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `get_registry` of `wl_display`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_display` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn get_registry(client: &Client, proxy: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, &raw const crate::wayland::generated::wayland::wl_registry::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>()) }
    }

}

/// The `wl_registry` interface.
pub mod wl_registry {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 1;

    static REQUEST_TYPES_0: TypeTable<4> = TypeTable([null(), null(), null(), null()]);
    static REQUESTS: MessageTable<1> = MessageTable([Message { name: c"bind".as_ptr(), signature: c"usun".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<2> = MessageTable([Message { name: c"global".as_ptr(), signature: c"usu".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"global_remove".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_registry".as_ptr(),
        version: 1,
        method_count: 1,
        methods: REQUESTS.0.as_ptr(),
        event_count: 2,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `global`.
        pub global: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, *const c_char, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `global_remove`.
        pub global_remove: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_global(_: *mut c_void, _: *mut Proxy, _: u32, _: *const c_char, _: u32) {}
    extern "C" fn ignore_global_remove(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        global: ignore_global,
        global_remove: ignore_global_remove,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `wl_registry` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `bind` of `wl_registry`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_registry` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn bind(client: &Client, proxy: *mut Proxy, name: u32, interface: *const Interface, version: u32) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, interface, version, 0, name, (*interface).name, version, null_mut::<Proxy>()) }
    }

}

/// The `wl_compositor` interface.
pub mod wl_compositor {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 7;

    static REQUEST_TYPES_0: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_surface::INTERFACE]);
    static REQUEST_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_2: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<3> = MessageTable([Message { name: c"create_surface".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"create_region".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }, Message { name: c"release".as_ptr(), signature: c"7".as_ptr(), types: REQUEST_TYPES_2.0.as_ptr() }]);

    static EVENTS: MessageTable<0> = MessageTable([]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_compositor".as_ptr(),
        version: 7,
        method_count: 3,
        methods: REQUESTS.0.as_ptr(),
        event_count: 0,
        events: EVENTS.0.as_ptr(),
    };

    // TEST: wayland_window_configures_with_a_size
    /// Request `create_surface` of `wl_compositor`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_compositor` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn create_surface(client: &Client, proxy: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>()) }
    }

    // `create_region` is not wrapped: it creates `wl_region`, which is not generated.

    // TEST: wayland_window_configures_with_a_size
    /// Request `release` of `wl_compositor`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_compositor` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn release(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 2, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `wl_surface` interface.
pub mod wl_surface {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 7;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_1: TypeTable<3> = TypeTable([null(), null(), null()]);
    static REQUEST_TYPES_2: TypeTable<4> = TypeTable([null(), null(), null(), null()]);
    static REQUEST_TYPES_3: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_callback::INTERFACE]);
    static REQUEST_TYPES_4: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_5: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_6: TypeTable<0> = TypeTable([]);
    static REQUEST_TYPES_7: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_8: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_9: TypeTable<4> = TypeTable([null(), null(), null(), null()]);
    static REQUEST_TYPES_10: TypeTable<2> = TypeTable([null(), null()]);
    static REQUEST_TYPES_11: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_callback::INTERFACE]);
    static REQUESTS: MessageTable<12> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"attach".as_ptr(), signature: c"?oii".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }, Message { name: c"damage".as_ptr(), signature: c"iiii".as_ptr(), types: REQUEST_TYPES_2.0.as_ptr() }, Message { name: c"frame".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_3.0.as_ptr() }, Message { name: c"set_opaque_region".as_ptr(), signature: c"?o".as_ptr(), types: REQUEST_TYPES_4.0.as_ptr() }, Message { name: c"set_input_region".as_ptr(), signature: c"?o".as_ptr(), types: REQUEST_TYPES_5.0.as_ptr() }, Message { name: c"commit".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_6.0.as_ptr() }, Message { name: c"set_buffer_transform".as_ptr(), signature: c"2i".as_ptr(), types: REQUEST_TYPES_7.0.as_ptr() }, Message { name: c"set_buffer_scale".as_ptr(), signature: c"3i".as_ptr(), types: REQUEST_TYPES_8.0.as_ptr() }, Message { name: c"damage_buffer".as_ptr(), signature: c"4iiii".as_ptr(), types: REQUEST_TYPES_9.0.as_ptr() }, Message { name: c"offset".as_ptr(), signature: c"5ii".as_ptr(), types: REQUEST_TYPES_10.0.as_ptr() }, Message { name: c"get_release".as_ptr(), signature: c"7n".as_ptr(), types: REQUEST_TYPES_11.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_output::INTERFACE]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_output::INTERFACE]);
    static EVENT_TYPES_2: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<4> = MessageTable([Message { name: c"enter".as_ptr(), signature: c"o".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"leave".as_ptr(), signature: c"o".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"preferred_buffer_scale".as_ptr(), signature: c"6i".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"preferred_buffer_transform".as_ptr(), signature: c"6u".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_surface".as_ptr(),
        version: 7,
        method_count: 12,
        methods: REQUESTS.0.as_ptr(),
        event_count: 4,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `enter`.
        pub enter: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `leave`.
        pub leave: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `preferred_buffer_scale`.
        pub preferred_buffer_scale: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `preferred_buffer_transform`.
        pub preferred_buffer_transform: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_enter(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_leave(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_preferred_buffer_scale(_: *mut c_void, _: *mut Proxy, _: i32) {}
    extern "C" fn ignore_preferred_buffer_transform(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        enter: ignore_enter,
        leave: ignore_leave,
        preferred_buffer_scale: ignore_preferred_buffer_scale,
        preferred_buffer_transform: ignore_preferred_buffer_transform,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `attach` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn attach(client: &Client, proxy: *mut Proxy, buffer: *mut Proxy, x: i32, y: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 0, buffer, x, y) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `damage` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn damage(client: &Client, proxy: *mut Proxy, x: i32, y: i32, width: i32, height: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 2, null(), (client.get_version)(proxy), 0, x, y, width, height) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `frame` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn frame(client: &Client, proxy: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 3, &raw const crate::wayland::generated::wayland::wl_callback::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>()) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_opaque_region` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_opaque_region(client: &Client, proxy: *mut Proxy, region: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 4, null(), (client.get_version)(proxy), 0, region) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_input_region` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_input_region(client: &Client, proxy: *mut Proxy, region: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 5, null(), (client.get_version)(proxy), 0, region) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `commit` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn commit(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 6, null(), (client.get_version)(proxy), 0) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_buffer_transform` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_buffer_transform(client: &Client, proxy: *mut Proxy, transform: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 7, null(), (client.get_version)(proxy), 0, transform) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_buffer_scale` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_buffer_scale(client: &Client, proxy: *mut Proxy, scale: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 8, null(), (client.get_version)(proxy), 0, scale) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `damage_buffer` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn damage_buffer(client: &Client, proxy: *mut Proxy, x: i32, y: i32, width: i32, height: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 9, null(), (client.get_version)(proxy), 0, x, y, width, height) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `offset` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn offset(client: &Client, proxy: *mut Proxy, x: i32, y: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 10, null(), (client.get_version)(proxy), 0, x, y) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `get_release` of `wl_surface`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_surface` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn get_release(client: &Client, proxy: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 11, &raw const crate::wayland::generated::wayland::wl_callback::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>()) }
    }

}

/// The `wl_callback` interface.
pub mod wl_callback {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 1;

    static REQUESTS: MessageTable<0> = MessageTable([]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<1> = MessageTable([Message { name: c"done".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_callback".as_ptr(),
        version: 1,
        method_count: 0,
        methods: REQUESTS.0.as_ptr(),
        event_count: 1,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `done`.
        pub done: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_done(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        done: ignore_done,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `wl_callback` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

}

/// The `wl_output` interface.
pub mod wl_output {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 4;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<1> = MessageTable([Message { name: c"release".as_ptr(), signature: c"3".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<8> = TypeTable([null(), null(), null(), null(), null(), null(), null(), null()]);
    static EVENT_TYPES_1: TypeTable<4> = TypeTable([null(), null(), null(), null()]);
    static EVENT_TYPES_2: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_4: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_5: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<6> = MessageTable([Message { name: c"geometry".as_ptr(), signature: c"iiiiissi".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"mode".as_ptr(), signature: c"uiii".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"done".as_ptr(), signature: c"2".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"scale".as_ptr(), signature: c"2i".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }, Message { name: c"name".as_ptr(), signature: c"4s".as_ptr(), types: EVENT_TYPES_4.0.as_ptr() }, Message { name: c"description".as_ptr(), signature: c"4s".as_ptr(), types: EVENT_TYPES_5.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_output".as_ptr(),
        version: 4,
        method_count: 1,
        methods: REQUESTS.0.as_ptr(),
        event_count: 6,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `geometry`.
        pub geometry: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32, i32, i32, i32, i32, *const c_char, *const c_char, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `mode`.
        pub mode: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, i32, i32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `done`.
        pub done: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `scale`.
        pub scale: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `name`.
        pub name: unsafe extern "C" fn(*mut c_void, *mut Proxy, *const c_char), // TEST: wayland_window_configures_with_a_size
        /// Event `description`.
        pub description: unsafe extern "C" fn(*mut c_void, *mut Proxy, *const c_char), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_geometry(_: *mut c_void, _: *mut Proxy, _: i32, _: i32, _: i32, _: i32, _: i32, _: *const c_char, _: *const c_char, _: i32) {}
    extern "C" fn ignore_mode(_: *mut c_void, _: *mut Proxy, _: u32, _: i32, _: i32, _: i32) {}
    extern "C" fn ignore_done(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_scale(_: *mut c_void, _: *mut Proxy, _: i32) {}
    extern "C" fn ignore_name(_: *mut c_void, _: *mut Proxy, _: *const c_char) {}
    extern "C" fn ignore_description(_: *mut c_void, _: *mut Proxy, _: *const c_char) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        geometry: ignore_geometry,
        mode: ignore_mode,
        done: ignore_done,
        scale: ignore_scale,
        name: ignore_name,
        description: ignore_description,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `wl_output` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `release` of `wl_output`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_output` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn release(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `wl_seat` interface.
pub mod wl_seat {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 11;

    static REQUEST_TYPES_0: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_pointer::INTERFACE]);
    static REQUEST_TYPES_1: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::wayland::wl_keyboard::INTERFACE]);
    static REQUEST_TYPES_2: TypeTable<1> = TypeTable([null()]);
    static REQUEST_TYPES_3: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<4> = MessageTable([Message { name: c"get_pointer".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"get_keyboard".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }, Message { name: c"get_touch".as_ptr(), signature: c"n".as_ptr(), types: REQUEST_TYPES_2.0.as_ptr() }, Message { name: c"release".as_ptr(), signature: c"5".as_ptr(), types: REQUEST_TYPES_3.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<2> = MessageTable([Message { name: c"capabilities".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"name".as_ptr(), signature: c"2s".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_seat".as_ptr(),
        version: 11,
        method_count: 4,
        methods: REQUESTS.0.as_ptr(),
        event_count: 2,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `capabilities`.
        pub capabilities: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `name`.
        pub name: unsafe extern "C" fn(*mut c_void, *mut Proxy, *const c_char), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_capabilities(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_name(_: *mut c_void, _: *mut Proxy, _: *const c_char) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        capabilities: ignore_capabilities,
        name: ignore_name,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `wl_seat` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `get_pointer` of `wl_seat`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_seat` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn get_pointer(client: &Client, proxy: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, &raw const crate::wayland::generated::wayland::wl_pointer::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>()) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `get_keyboard` of `wl_seat`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_seat` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn get_keyboard(client: &Client, proxy: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, &raw const crate::wayland::generated::wayland::wl_keyboard::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>()) }
    }

    // `get_touch` is not wrapped: it creates `wl_touch`, which is not generated.

    // TEST: wayland_window_configures_with_a_size
    /// Request `release` of `wl_seat`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_seat` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn release(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 3, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `wl_pointer` interface.
pub mod wl_pointer {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 11;

    static REQUEST_TYPES_0: TypeTable<4> = TypeTable([null(), &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE, null(), null()]);
    static REQUEST_TYPES_1: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<2> = MessageTable([Message { name: c"set_cursor".as_ptr(), signature: c"u?oii".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"release".as_ptr(), signature: c"3".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<4> = TypeTable([null(), &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE, null(), null()]);
    static EVENT_TYPES_1: TypeTable<2> = TypeTable([null(), &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE]);
    static EVENT_TYPES_2: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_3: TypeTable<4> = TypeTable([null(), null(), null(), null()]);
    static EVENT_TYPES_4: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_5: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_6: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_7: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_8: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_9: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_10: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_11: TypeTable<2> = TypeTable([null(), null()]);
    static EVENTS: MessageTable<12> = MessageTable([Message { name: c"enter".as_ptr(), signature: c"uoff".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"leave".as_ptr(), signature: c"uo".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"motion".as_ptr(), signature: c"uff".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"button".as_ptr(), signature: c"uuuu".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }, Message { name: c"axis".as_ptr(), signature: c"uuf".as_ptr(), types: EVENT_TYPES_4.0.as_ptr() }, Message { name: c"frame".as_ptr(), signature: c"5".as_ptr(), types: EVENT_TYPES_5.0.as_ptr() }, Message { name: c"axis_source".as_ptr(), signature: c"5u".as_ptr(), types: EVENT_TYPES_6.0.as_ptr() }, Message { name: c"axis_stop".as_ptr(), signature: c"5uu".as_ptr(), types: EVENT_TYPES_7.0.as_ptr() }, Message { name: c"axis_discrete".as_ptr(), signature: c"5ui".as_ptr(), types: EVENT_TYPES_8.0.as_ptr() }, Message { name: c"axis_value120".as_ptr(), signature: c"8ui".as_ptr(), types: EVENT_TYPES_9.0.as_ptr() }, Message { name: c"axis_relative_direction".as_ptr(), signature: c"9uu".as_ptr(), types: EVENT_TYPES_10.0.as_ptr() }, Message { name: c"warp".as_ptr(), signature: c"11ff".as_ptr(), types: EVENT_TYPES_11.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_pointer".as_ptr(),
        version: 11,
        method_count: 2,
        methods: REQUESTS.0.as_ptr(),
        event_count: 12,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `enter`.
        pub enter: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, *mut Proxy, i32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `leave`.
        pub leave: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `motion`.
        pub motion: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, i32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `button`.
        pub button: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `axis`.
        pub axis: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `frame`.
        pub frame: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `axis_source`.
        pub axis_source: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `axis_stop`.
        pub axis_stop: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `axis_discrete`.
        pub axis_discrete: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `axis_value120`.
        pub axis_value120: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `axis_relative_direction`.
        pub axis_relative_direction: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `warp`.
        pub warp: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32, i32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_enter(_: *mut c_void, _: *mut Proxy, _: u32, _: *mut Proxy, _: i32, _: i32) {}
    extern "C" fn ignore_leave(_: *mut c_void, _: *mut Proxy, _: u32, _: *mut Proxy) {}
    extern "C" fn ignore_motion(_: *mut c_void, _: *mut Proxy, _: u32, _: i32, _: i32) {}
    extern "C" fn ignore_button(_: *mut c_void, _: *mut Proxy, _: u32, _: u32, _: u32, _: u32) {}
    extern "C" fn ignore_axis(_: *mut c_void, _: *mut Proxy, _: u32, _: u32, _: i32) {}
    extern "C" fn ignore_frame(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_axis_source(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_axis_stop(_: *mut c_void, _: *mut Proxy, _: u32, _: u32) {}
    extern "C" fn ignore_axis_discrete(_: *mut c_void, _: *mut Proxy, _: u32, _: i32) {}
    extern "C" fn ignore_axis_value120(_: *mut c_void, _: *mut Proxy, _: u32, _: i32) {}
    extern "C" fn ignore_axis_relative_direction(_: *mut c_void, _: *mut Proxy, _: u32, _: u32) {}
    extern "C" fn ignore_warp(_: *mut c_void, _: *mut Proxy, _: i32, _: i32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        enter: ignore_enter,
        leave: ignore_leave,
        motion: ignore_motion,
        button: ignore_button,
        axis: ignore_axis,
        frame: ignore_frame,
        axis_source: ignore_axis_source,
        axis_stop: ignore_axis_stop,
        axis_discrete: ignore_axis_discrete,
        axis_value120: ignore_axis_value120,
        axis_relative_direction: ignore_axis_relative_direction,
        warp: ignore_warp,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `wl_pointer` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_cursor` of `wl_pointer`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_pointer` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_cursor(client: &Client, proxy: *mut Proxy, serial: u32, surface: *mut Proxy, hotspot_x: i32, hotspot_y: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 0, serial, surface, hotspot_x, hotspot_y) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `release` of `wl_pointer`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_pointer` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn release(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `wl_keyboard` interface.
pub mod wl_keyboard {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 11;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<1> = MessageTable([Message { name: c"release".as_ptr(), signature: c"3".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_1: TypeTable<3> = TypeTable([null(), &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE, null()]);
    static EVENT_TYPES_2: TypeTable<2> = TypeTable([null(), &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE]);
    static EVENT_TYPES_3: TypeTable<4> = TypeTable([null(), null(), null(), null()]);
    static EVENT_TYPES_4: TypeTable<5> = TypeTable([null(), null(), null(), null(), null()]);
    static EVENT_TYPES_5: TypeTable<2> = TypeTable([null(), null()]);
    static EVENTS: MessageTable<6> = MessageTable([Message { name: c"keymap".as_ptr(), signature: c"uhu".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"enter".as_ptr(), signature: c"uoa".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"leave".as_ptr(), signature: c"uo".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"key".as_ptr(), signature: c"uuuu".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }, Message { name: c"modifiers".as_ptr(), signature: c"uuuuu".as_ptr(), types: EVENT_TYPES_4.0.as_ptr() }, Message { name: c"repeat_info".as_ptr(), signature: c"4ii".as_ptr(), types: EVENT_TYPES_5.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"wl_keyboard".as_ptr(),
        version: 11,
        method_count: 1,
        methods: REQUESTS.0.as_ptr(),
        event_count: 6,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `keymap`.
        pub keymap: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, i32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `enter`.
        pub enter: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, *mut Proxy, *mut c_void), // TEST: wayland_window_configures_with_a_size
        /// Event `leave`.
        pub leave: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `key`.
        pub key: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `modifiers`.
        pub modifiers: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32, u32, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `repeat_info`.
        pub repeat_info: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32, i32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_keymap(_: *mut c_void, _: *mut Proxy, _: u32, _: i32, _: u32) {}
    extern "C" fn ignore_enter(_: *mut c_void, _: *mut Proxy, _: u32, _: *mut Proxy, _: *mut c_void) {}
    extern "C" fn ignore_leave(_: *mut c_void, _: *mut Proxy, _: u32, _: *mut Proxy) {}
    extern "C" fn ignore_key(_: *mut c_void, _: *mut Proxy, _: u32, _: u32, _: u32, _: u32) {}
    extern "C" fn ignore_modifiers(_: *mut c_void, _: *mut Proxy, _: u32, _: u32, _: u32, _: u32, _: u32) {}
    extern "C" fn ignore_repeat_info(_: *mut c_void, _: *mut Proxy, _: i32, _: i32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        keymap: ignore_keymap,
        enter: ignore_enter,
        leave: ignore_leave,
        key: ignore_key,
        modifiers: ignore_modifiers,
        repeat_info: ignore_repeat_info,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `wl_keyboard` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `release` of `wl_keyboard`.
    ///
    /// # Safety
    /// `proxy` is a live `wl_keyboard` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn release(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

}
