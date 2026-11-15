//! The C layout of the protocol tables of `libwayland-client`
//! (`wl_interface`, `wl_message`) that the generated code fills.

#![allow(unsafe_code)]

use std::ffi::c_char;

/// `struct wl_interface`.
#[repr(C)]
#[derive(Debug)]
pub struct Interface {
    /// The interface name.
    pub name: *const c_char,
    /// The highest version known.
    pub version: i32,
    /// Number of requests.
    pub method_count: i32,
    /// The requests.
    pub methods: *const Message,
    /// Number of events.
    pub event_count: i32,
    /// The events.
    pub events: *const Message,
}

/// `struct wl_message`.
#[repr(C)]
#[derive(Debug)]
pub struct Message {
    /// The request or event name.
    pub name: *const c_char,
    /// The wire signature.
    pub signature: *const c_char,
    /// The interface of each object argument, null where there is none.
    pub types: *const *const Interface,
}

/// The `types` array of one message.
#[repr(transparent)]
#[derive(Debug)]
pub struct TypeTable<const N: usize>(pub [*const Interface; N]);

/// The messages of one side of an interface.
#[repr(transparent)]
#[derive(Debug)]
pub struct MessageTable<const N: usize>(pub [Message; N]);

// TEST: wayland_connect_binds_the_globals
// SAFETY: the tables are immutable statics of pointers to other statics and
// to string literals; libwayland only reads them.
unsafe impl Sync for Interface {}

// TEST: wayland_connect_binds_the_globals
// SAFETY: as for `Interface`: immutable, read only.
unsafe impl Sync for Message {}

// TEST: wayland_connect_binds_the_globals
// SAFETY: as for `Interface`: immutable, read only.
unsafe impl<const N: usize> Sync for TypeTable<N> {}

// TEST: wayland_connect_binds_the_globals
// SAFETY: as for `Interface`: immutable, read only.
unsafe impl<const N: usize> Sync for MessageTable<N> {}
