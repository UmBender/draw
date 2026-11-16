//! Generated from `protocols/tablet-v2.xml` by the generator of
//! `tests/wayland_generator.rs` (ADR-T168-1). Do not edit: regenerate with
//! `cargo test -p draw-sys --test wayland_generator -- --ignored write_generated_protocol`.

#![allow(missing_docs, unsafe_code, unused_imports, clippy::all, clippy::pedantic)]

/// The `zwp_tablet_manager_v2` interface.
pub mod zwp_tablet_manager_v2 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<2> = TypeTable([&raw const crate::wayland::generated::tablet_v2::zwp_tablet_seat_v2::INTERFACE, &raw const crate::wayland::generated::wayland::wl_seat::INTERFACE]);
    static REQUEST_TYPES_1: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<2> = MessageTable([Message { name: c"get_tablet_seat".as_ptr(), signature: c"no".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }]);

    static EVENTS: MessageTable<0> = MessageTable([]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zwp_tablet_manager_v2".as_ptr(),
        version: 2,
        method_count: 2,
        methods: REQUESTS.0.as_ptr(),
        event_count: 0,
        events: EVENTS.0.as_ptr(),
    };

    // TEST: wayland_window_configures_with_a_size
    /// Request `get_tablet_seat` of `zwp_tablet_manager_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_manager_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn get_tablet_seat(client: &Client, proxy: *mut Proxy, seat: *mut Proxy) -> *mut Proxy {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, &raw const crate::wayland::generated::tablet_v2::zwp_tablet_seat_v2::INTERFACE, (client.get_version)(proxy), 0, null_mut::<Proxy>(), seat) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zwp_tablet_manager_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_manager_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `zwp_tablet_seat_v2` interface.
pub mod zwp_tablet_seat_v2 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<1> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::tablet_v2::zwp_tablet_v2::INTERFACE]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::tablet_v2::zwp_tablet_tool_v2::INTERFACE]);
    static EVENT_TYPES_2: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::tablet_v2::zwp_tablet_pad_v2::INTERFACE]);
    static EVENTS: MessageTable<3> = MessageTable([Message { name: c"tablet_added".as_ptr(), signature: c"n".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"tool_added".as_ptr(), signature: c"n".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"pad_added".as_ptr(), signature: c"n".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zwp_tablet_seat_v2".as_ptr(),
        version: 2,
        method_count: 1,
        methods: REQUESTS.0.as_ptr(),
        event_count: 3,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `tablet_added`.
        pub tablet_added: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `tool_added`.
        pub tool_added: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `pad_added`.
        pub pad_added: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_tablet_added(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_tool_added(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_pad_added(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        tablet_added: ignore_tablet_added,
        tool_added: ignore_tool_added,
        pad_added: ignore_pad_added,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_seat_v2` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zwp_tablet_seat_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_seat_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `zwp_tablet_tool_v2` interface.
pub mod zwp_tablet_tool_v2 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<4> = TypeTable([null(), &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE, null(), null()]);
    static REQUEST_TYPES_1: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<2> = MessageTable([Message { name: c"set_cursor".as_ptr(), signature: c"u?oii".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_1: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_2: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_4: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_5: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_6: TypeTable<3> = TypeTable([null(), &raw const crate::wayland::generated::tablet_v2::zwp_tablet_v2::INTERFACE, &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE]);
    static EVENT_TYPES_7: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_8: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_9: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_10: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_11: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_12: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_13: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_14: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_15: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_16: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_17: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_18: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<19> = MessageTable([Message { name: c"type".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"hardware_serial".as_ptr(), signature: c"uu".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"hardware_id_wacom".as_ptr(), signature: c"uu".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"capability".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }, Message { name: c"done".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_4.0.as_ptr() }, Message { name: c"removed".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_5.0.as_ptr() }, Message { name: c"proximity_in".as_ptr(), signature: c"uoo".as_ptr(), types: EVENT_TYPES_6.0.as_ptr() }, Message { name: c"proximity_out".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_7.0.as_ptr() }, Message { name: c"down".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_8.0.as_ptr() }, Message { name: c"up".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_9.0.as_ptr() }, Message { name: c"motion".as_ptr(), signature: c"ff".as_ptr(), types: EVENT_TYPES_10.0.as_ptr() }, Message { name: c"pressure".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_11.0.as_ptr() }, Message { name: c"distance".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_12.0.as_ptr() }, Message { name: c"tilt".as_ptr(), signature: c"ff".as_ptr(), types: EVENT_TYPES_13.0.as_ptr() }, Message { name: c"rotation".as_ptr(), signature: c"f".as_ptr(), types: EVENT_TYPES_14.0.as_ptr() }, Message { name: c"slider".as_ptr(), signature: c"i".as_ptr(), types: EVENT_TYPES_15.0.as_ptr() }, Message { name: c"wheel".as_ptr(), signature: c"fi".as_ptr(), types: EVENT_TYPES_16.0.as_ptr() }, Message { name: c"button".as_ptr(), signature: c"uuu".as_ptr(), types: EVENT_TYPES_17.0.as_ptr() }, Message { name: c"frame".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_18.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zwp_tablet_tool_v2".as_ptr(),
        version: 2,
        method_count: 2,
        methods: REQUESTS.0.as_ptr(),
        event_count: 19,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `type`.
        pub r#type: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `hardware_serial`.
        pub hardware_serial: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `hardware_id_wacom`.
        pub hardware_id_wacom: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `capability`.
        pub capability: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `done`.
        pub done: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `removed`.
        pub removed: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `proximity_in`.
        pub proximity_in: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `proximity_out`.
        pub proximity_out: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `down`.
        pub down: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `up`.
        pub up: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `motion`.
        pub motion: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `pressure`.
        pub pressure: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `distance`.
        pub distance: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `tilt`.
        pub tilt: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `rotation`.
        pub rotation: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `slider`.
        pub slider: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `wheel`.
        pub wheel: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `button`.
        pub button: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `frame`.
        pub frame: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_type(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_hardware_serial(_: *mut c_void, _: *mut Proxy, _: u32, _: u32) {}
    extern "C" fn ignore_hardware_id_wacom(_: *mut c_void, _: *mut Proxy, _: u32, _: u32) {}
    extern "C" fn ignore_capability(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_done(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_removed(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_proximity_in(_: *mut c_void, _: *mut Proxy, _: u32, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_proximity_out(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_down(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_up(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_motion(_: *mut c_void, _: *mut Proxy, _: i32, _: i32) {}
    extern "C" fn ignore_pressure(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_distance(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_tilt(_: *mut c_void, _: *mut Proxy, _: i32, _: i32) {}
    extern "C" fn ignore_rotation(_: *mut c_void, _: *mut Proxy, _: i32) {}
    extern "C" fn ignore_slider(_: *mut c_void, _: *mut Proxy, _: i32) {}
    extern "C" fn ignore_wheel(_: *mut c_void, _: *mut Proxy, _: i32, _: i32) {}
    extern "C" fn ignore_button(_: *mut c_void, _: *mut Proxy, _: u32, _: u32, _: u32) {}
    extern "C" fn ignore_frame(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        r#type: ignore_type,
        hardware_serial: ignore_hardware_serial,
        hardware_id_wacom: ignore_hardware_id_wacom,
        capability: ignore_capability,
        done: ignore_done,
        removed: ignore_removed,
        proximity_in: ignore_proximity_in,
        proximity_out: ignore_proximity_out,
        down: ignore_down,
        up: ignore_up,
        motion: ignore_motion,
        pressure: ignore_pressure,
        distance: ignore_distance,
        tilt: ignore_tilt,
        rotation: ignore_rotation,
        slider: ignore_slider,
        wheel: ignore_wheel,
        button: ignore_button,
        frame: ignore_frame,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_tool_v2` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_cursor` of `zwp_tablet_tool_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_tool_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_cursor(client: &Client, proxy: *mut Proxy, serial: u32, surface: *mut Proxy, hotspot_x: i32, hotspot_y: i32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 0, serial, surface, hotspot_x, hotspot_y) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zwp_tablet_tool_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_tool_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `zwp_tablet_v2` interface.
pub mod zwp_tablet_v2 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<1> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_1: TypeTable<2> = TypeTable([null(), null()]);
    static EVENT_TYPES_2: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_3: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_4: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_5: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<6> = MessageTable([Message { name: c"name".as_ptr(), signature: c"s".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"id".as_ptr(), signature: c"uu".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"path".as_ptr(), signature: c"s".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"done".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }, Message { name: c"removed".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_4.0.as_ptr() }, Message { name: c"bustype".as_ptr(), signature: c"2u".as_ptr(), types: EVENT_TYPES_5.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zwp_tablet_v2".as_ptr(),
        version: 2,
        method_count: 1,
        methods: REQUESTS.0.as_ptr(),
        event_count: 6,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `name`.
        pub name: unsafe extern "C" fn(*mut c_void, *mut Proxy, *const c_char), // TEST: wayland_window_configures_with_a_size
        /// Event `id`.
        pub id: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `path`.
        pub path: unsafe extern "C" fn(*mut c_void, *mut Proxy, *const c_char), // TEST: wayland_window_configures_with_a_size
        /// Event `done`.
        pub done: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `removed`.
        pub removed: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `bustype`.
        pub bustype: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_name(_: *mut c_void, _: *mut Proxy, _: *const c_char) {}
    extern "C" fn ignore_id(_: *mut c_void, _: *mut Proxy, _: u32, _: u32) {}
    extern "C" fn ignore_path(_: *mut c_void, _: *mut Proxy, _: *const c_char) {}
    extern "C" fn ignore_done(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_removed(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_bustype(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        name: ignore_name,
        id: ignore_id,
        path: ignore_path,
        done: ignore_done,
        removed: ignore_removed,
        bustype: ignore_bustype,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_v2` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zwp_tablet_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `zwp_tablet_pad_ring_v2` interface.
pub mod zwp_tablet_pad_ring_v2 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<2> = TypeTable([null(), null()]);
    static REQUEST_TYPES_1: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<2> = MessageTable([Message { name: c"set_feedback".as_ptr(), signature: c"su".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_2: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<4> = MessageTable([Message { name: c"source".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"angle".as_ptr(), signature: c"f".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"stop".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"frame".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zwp_tablet_pad_ring_v2".as_ptr(),
        version: 2,
        method_count: 2,
        methods: REQUESTS.0.as_ptr(),
        event_count: 4,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `source`.
        pub source: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `angle`.
        pub angle: unsafe extern "C" fn(*mut c_void, *mut Proxy, i32), // TEST: wayland_window_configures_with_a_size
        /// Event `stop`.
        pub stop: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `frame`.
        pub frame: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_source(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_angle(_: *mut c_void, _: *mut Proxy, _: i32) {}
    extern "C" fn ignore_stop(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_frame(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        source: ignore_source,
        angle: ignore_angle,
        stop: ignore_stop,
        frame: ignore_frame,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_ring_v2` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_feedback` of `zwp_tablet_pad_ring_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_ring_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_feedback(client: &Client, proxy: *mut Proxy, description: *const c_char, serial: u32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 0, description, serial) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zwp_tablet_pad_ring_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_ring_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `zwp_tablet_pad_strip_v2` interface.
pub mod zwp_tablet_pad_strip_v2 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<2> = TypeTable([null(), null()]);
    static REQUEST_TYPES_1: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<2> = MessageTable([Message { name: c"set_feedback".as_ptr(), signature: c"su".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_2: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<4> = MessageTable([Message { name: c"source".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"position".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"stop".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"frame".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zwp_tablet_pad_strip_v2".as_ptr(),
        version: 2,
        method_count: 2,
        methods: REQUESTS.0.as_ptr(),
        event_count: 4,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `source`.
        pub source: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `position`.
        pub position: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `stop`.
        pub stop: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `frame`.
        pub frame: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_source(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_position(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_stop(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_frame(_: *mut c_void, _: *mut Proxy, _: u32) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        source: ignore_source,
        position: ignore_position,
        stop: ignore_stop,
        frame: ignore_frame,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_strip_v2` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_feedback` of `zwp_tablet_pad_strip_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_strip_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_feedback(client: &Client, proxy: *mut Proxy, description: *const c_char, serial: u32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 0, description, serial) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zwp_tablet_pad_strip_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_strip_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `zwp_tablet_pad_group_v2` interface.
pub mod zwp_tablet_pad_group_v2 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<1> = MessageTable([Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::tablet_v2::zwp_tablet_pad_ring_v2::INTERFACE]);
    static EVENT_TYPES_2: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::tablet_v2::zwp_tablet_pad_strip_v2::INTERFACE]);
    static EVENT_TYPES_3: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_4: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_5: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_6: TypeTable<1> = TypeTable([null()]);
    static EVENTS: MessageTable<7> = MessageTable([Message { name: c"buttons".as_ptr(), signature: c"a".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"ring".as_ptr(), signature: c"n".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"strip".as_ptr(), signature: c"n".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"modes".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }, Message { name: c"done".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_4.0.as_ptr() }, Message { name: c"mode_switch".as_ptr(), signature: c"uuu".as_ptr(), types: EVENT_TYPES_5.0.as_ptr() }, Message { name: c"dial".as_ptr(), signature: c"2n".as_ptr(), types: EVENT_TYPES_6.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zwp_tablet_pad_group_v2".as_ptr(),
        version: 2,
        method_count: 1,
        methods: REQUESTS.0.as_ptr(),
        event_count: 7,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `buttons`.
        pub buttons: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut c_void), // TEST: wayland_window_configures_with_a_size
        /// Event `ring`.
        pub ring: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `strip`.
        pub strip: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `modes`.
        pub modes: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `done`.
        pub done: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `mode_switch`.
        pub mode_switch: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `dial`.
        pub dial: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_buttons(_: *mut c_void, _: *mut Proxy, _: *mut c_void) {}
    extern "C" fn ignore_ring(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_strip(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_modes(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_done(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_mode_switch(_: *mut c_void, _: *mut Proxy, _: u32, _: u32, _: u32) {}
    extern "C" fn ignore_dial(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        buttons: ignore_buttons,
        ring: ignore_ring,
        strip: ignore_strip,
        modes: ignore_modes,
        done: ignore_done,
        mode_switch: ignore_mode_switch,
        dial: ignore_dial,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_group_v2` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zwp_tablet_pad_group_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_group_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 1) };
    }

}

/// The `zwp_tablet_pad_v2` interface.
pub mod zwp_tablet_pad_v2 {
    use crate::wayland::client::{Client, Proxy};
    use crate::wayland::ffi::{Interface, Message, MessageTable, TypeTable};
    use std::ffi::{c_char, c_void};
    use std::ptr::{null, null_mut};

    /// Highest version this code knows.
    pub const VERSION: u32 = 2;

    static REQUEST_TYPES_0: TypeTable<3> = TypeTable([null(), null(), null()]);
    static REQUEST_TYPES_1: TypeTable<0> = TypeTable([]);
    static REQUESTS: MessageTable<2> = MessageTable([Message { name: c"set_feedback".as_ptr(), signature: c"usu".as_ptr(), types: REQUEST_TYPES_0.0.as_ptr() }, Message { name: c"destroy".as_ptr(), signature: c"".as_ptr(), types: REQUEST_TYPES_1.0.as_ptr() }]);

    static EVENT_TYPES_0: TypeTable<1> = TypeTable([&raw const crate::wayland::generated::tablet_v2::zwp_tablet_pad_group_v2::INTERFACE]);
    static EVENT_TYPES_1: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_2: TypeTable<1> = TypeTable([null()]);
    static EVENT_TYPES_3: TypeTable<0> = TypeTable([]);
    static EVENT_TYPES_4: TypeTable<3> = TypeTable([null(), null(), null()]);
    static EVENT_TYPES_5: TypeTable<3> = TypeTable([null(), &raw const crate::wayland::generated::tablet_v2::zwp_tablet_v2::INTERFACE, &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE]);
    static EVENT_TYPES_6: TypeTable<2> = TypeTable([null(), &raw const crate::wayland::generated::wayland::wl_surface::INTERFACE]);
    static EVENT_TYPES_7: TypeTable<0> = TypeTable([]);
    static EVENTS: MessageTable<8> = MessageTable([Message { name: c"group".as_ptr(), signature: c"n".as_ptr(), types: EVENT_TYPES_0.0.as_ptr() }, Message { name: c"path".as_ptr(), signature: c"s".as_ptr(), types: EVENT_TYPES_1.0.as_ptr() }, Message { name: c"buttons".as_ptr(), signature: c"u".as_ptr(), types: EVENT_TYPES_2.0.as_ptr() }, Message { name: c"done".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_3.0.as_ptr() }, Message { name: c"button".as_ptr(), signature: c"uuu".as_ptr(), types: EVENT_TYPES_4.0.as_ptr() }, Message { name: c"enter".as_ptr(), signature: c"uoo".as_ptr(), types: EVENT_TYPES_5.0.as_ptr() }, Message { name: c"leave".as_ptr(), signature: c"uo".as_ptr(), types: EVENT_TYPES_6.0.as_ptr() }, Message { name: c"removed".as_ptr(), signature: c"".as_ptr(), types: EVENT_TYPES_7.0.as_ptr() }]);

    /// The `wl_interface` table.
    pub static INTERFACE: Interface = Interface {
        name: c"zwp_tablet_pad_v2".as_ptr(),
        version: 2,
        method_count: 2,
        methods: REQUESTS.0.as_ptr(),
        event_count: 8,
        events: EVENTS.0.as_ptr(),
    };

    /// The event handlers, in protocol order.
    #[repr(C)]
    pub struct Listener {
        /// Event `group`.
        pub group: unsafe extern "C" fn(*mut c_void, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `path`.
        pub path: unsafe extern "C" fn(*mut c_void, *mut Proxy, *const c_char), // TEST: wayland_window_configures_with_a_size
        /// Event `buttons`.
        pub buttons: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `done`.
        pub done: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `button`.
        pub button: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, u32, u32), // TEST: wayland_window_configures_with_a_size
        /// Event `enter`.
        pub enter: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, *mut Proxy, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `leave`.
        pub leave: unsafe extern "C" fn(*mut c_void, *mut Proxy, u32, *mut Proxy), // TEST: wayland_window_configures_with_a_size
        /// Event `removed`.
        pub removed: unsafe extern "C" fn(*mut c_void, *mut Proxy), // TEST: wayland_window_configures_with_a_size
    }

    extern "C" fn ignore_group(_: *mut c_void, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_path(_: *mut c_void, _: *mut Proxy, _: *const c_char) {}
    extern "C" fn ignore_buttons(_: *mut c_void, _: *mut Proxy, _: u32) {}
    extern "C" fn ignore_done(_: *mut c_void, _: *mut Proxy) {}
    extern "C" fn ignore_button(_: *mut c_void, _: *mut Proxy, _: u32, _: u32, _: u32) {}
    extern "C" fn ignore_enter(_: *mut c_void, _: *mut Proxy, _: u32, _: *mut Proxy, _: *mut Proxy) {}
    extern "C" fn ignore_leave(_: *mut c_void, _: *mut Proxy, _: u32, _: *mut Proxy) {}
    extern "C" fn ignore_removed(_: *mut c_void, _: *mut Proxy) {}

    /// Handlers that do nothing, to update with `..IGNORE`.
    pub const IGNORE: Listener = Listener {
        group: ignore_group,
        path: ignore_path,
        buttons: ignore_buttons,
        done: ignore_done,
        button: ignore_button,
        enter: ignore_enter,
        leave: ignore_leave,
        removed: ignore_removed,
    };

    // TEST: wayland_window_configures_with_a_size
    /// Installs `listener` on `proxy`; `0` on success.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_v2` proxy of `client`, and `data` stays valid
    /// (whatever the handlers do with it) until the proxy is destroyed.
    pub unsafe fn add_listener(client: &Client, proxy: *mut Proxy, listener: &'static Listener, data: *mut c_void) -> i32 {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for `proxy` and `data`; the listener is a
        // `repr(C)` table of function pointers in event order.
        unsafe { (client.add_listener)(proxy, std::ptr::from_ref(listener).cast(), data) }
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `set_feedback` of `zwp_tablet_pad_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn set_feedback(client: &Client, proxy: *mut Proxy, button: u32, description: *const c_char, serial: u32) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 0, null(), (client.get_version)(proxy), 0, button, description, serial) };
    }

    // TEST: wayland_window_configures_with_a_size
    /// Request `destroy` of `zwp_tablet_pad_v2`.
    ///
    /// # Safety
    /// `proxy` is a live `zwp_tablet_pad_v2` proxy of `client`; object arguments are live
    /// proxies of the same connection (null where the protocol allows it).
    pub unsafe fn destroy(client: &Client, proxy: *mut Proxy) {
        // TEST: wayland_window_configures_with_a_size
        // SAFETY: the caller vouches for the proxies; the opcode, signature and
        // argument types follow the protocol XML this file was generated from.
        unsafe { (client.marshal_flags)(proxy, 1, null(), (client.get_version)(proxy), 1) };
    }

}
