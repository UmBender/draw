// tablet-unstable-v2.xml, version 1 (draw patch, ADR-T35-1).
//
// Written out by hand: `wayland_interface!` leaves event types null, but
// libwayland needs the interface of every `new_id` event argument to create
// its proxy, and reads the type of every object argument. Pads, groups,
// rings and strips are declared only so that their events can be read; the
// patch adds no listeners to them.

#![allow(non_upper_case_globals)]

use super::super::libwayland_client::{wl_fixed_t, wl_interface, wl_message, wl_surface};
use core::ffi::{c_int, c_uint};
use std::ptr::null;

const NONE: *const wl_interface = null();

macro_rules! message {
    ($name:literal, $signature:literal) => {
        wl_message {
            name: concat!($name, "\0").as_ptr() as _,
            signature: concat!($signature, "\0").as_ptr() as _,
            types: std::ptr::null_mut(),
        }
    };
    ($name:literal, $signature:literal, $types:ident) => {
        wl_message {
            name: concat!($name, "\0").as_ptr() as _,
            signature: concat!($signature, "\0").as_ptr() as _,
            types: unsafe { &$types as *const _ as *mut *const wl_interface },
        }
    };
}

macro_rules! interface {
    ($name:ident, $str:literal, $requests:ident: $request_count:literal, $events:ident: $event_count:literal) => {
        pub static mut $name: wl_interface = wl_interface {
            name: concat!($str, "\0").as_ptr() as _,
            version: 1,
            method_count: $request_count,
            methods: unsafe { &$requests as *const _ as *const wl_message },
            event_count: $event_count,
            events: unsafe { &$events as *const _ as *const wl_message },
        };
    };
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct zwp_tablet_manager_v2 {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct zwp_tablet_seat_v2 {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct zwp_tablet_tool_v2 {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct zwp_tablet_v2 {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct zwp_tablet_pad_v2 {
    _unused: [u8; 0],
}

/// `zwp_tablet_manager_v2.get_tablet_seat`.
pub const TABLET_MANAGER_GET_TABLET_SEAT: u32 = 0;
/// `zwp_tablet_tool_v2.set_cursor`.
pub const TABLET_TOOL_SET_CURSOR: u32 = 0;
/// `zwp_tablet_tool_v2.destroy`.
pub const TABLET_TOOL_DESTROY: u32 = 1;
/// The largest `zwp_tablet_tool_v2.pressure` value.
pub const TOOL_PRESSURE_MAX: f32 = 65535.0;

// ---- zwp_tablet_manager_v2 ----

static mut manager_get_tablet_seat_types: [*const wl_interface; 2] =
    [unsafe { &zwp_tablet_seat_v2_interface as *const _ }, NONE];
static mut manager_requests: [wl_message; 2] = [
    message!("get_tablet_seat", "no", manager_get_tablet_seat_types),
    message!("destroy", ""),
];
static mut manager_events: [wl_message; 0] = [];
interface!(
    zwp_tablet_manager_v2_interface,
    "zwp_tablet_manager_v2",
    manager_requests: 2,
    manager_events: 0
);

// ---- zwp_tablet_seat_v2 ----

static mut seat_tablet_added_types: [*const wl_interface; 1] =
    [unsafe { &zwp_tablet_v2_interface as *const _ }];
static mut seat_tool_added_types: [*const wl_interface; 1] =
    [unsafe { &zwp_tablet_tool_v2_interface as *const _ }];
static mut seat_pad_added_types: [*const wl_interface; 1] =
    [unsafe { &zwp_tablet_pad_v2_interface as *const _ }];
static mut seat_requests: [wl_message; 1] = [message!("destroy", "")];
static mut seat_events: [wl_message; 3] = [
    message!("tablet_added", "n", seat_tablet_added_types),
    message!("tool_added", "n", seat_tool_added_types),
    message!("pad_added", "n", seat_pad_added_types),
];
interface!(
    zwp_tablet_seat_v2_interface,
    "zwp_tablet_seat_v2",
    seat_requests: 1,
    seat_events: 3
);

// ---- zwp_tablet_tool_v2 ----

static mut tool_set_cursor_types: [*const wl_interface; 4] = [NONE, NONE, NONE, NONE];
static mut tool_proximity_in_types: [*const wl_interface; 3] = [NONE, NONE, NONE];
static mut tool_requests: [wl_message; 2] = [
    message!("set_cursor", "u?oii", tool_set_cursor_types),
    message!("destroy", ""),
];
static mut tool_events: [wl_message; 19] = [
    message!("type", "u"),
    message!("hardware_serial", "uu"),
    message!("hardware_id_wacom", "uu"),
    message!("capability", "u"),
    message!("done", ""),
    message!("removed", ""),
    message!("proximity_in", "uoo", tool_proximity_in_types),
    message!("proximity_out", ""),
    message!("down", "u"),
    message!("up", ""),
    message!("motion", "ff"),
    message!("pressure", "u"),
    message!("distance", "u"),
    message!("tilt", "ff"),
    message!("rotation", "f"),
    message!("slider", "i"),
    message!("wheel", "fi"),
    message!("button", "uuu"),
    message!("frame", "u"),
];
interface!(
    zwp_tablet_tool_v2_interface,
    "zwp_tablet_tool_v2",
    tool_requests: 2,
    tool_events: 19
);

// ---- zwp_tablet_v2 ----

static mut tablet_requests: [wl_message; 1] = [message!("destroy", "")];
static mut tablet_events: [wl_message; 5] = [
    message!("name", "s"),
    message!("id", "uu"),
    message!("path", "s"),
    message!("done", ""),
    message!("removed", ""),
];
interface!(
    zwp_tablet_v2_interface,
    "zwp_tablet_v2",
    tablet_requests: 1,
    tablet_events: 5
);

// ---- zwp_tablet_pad_v2 and its groups, rings and strips ----

static mut pad_group_types: [*const wl_interface; 1] =
    [unsafe { &zwp_tablet_pad_group_v2_interface as *const _ }];
static mut pad_enter_types: [*const wl_interface; 3] = [NONE, NONE, NONE];
static mut pad_leave_types: [*const wl_interface; 2] = [NONE, NONE];
static mut pad_requests: [wl_message; 2] = [
    message!("set_feedback", "usu"),
    message!("destroy", ""),
];
static mut pad_events: [wl_message; 8] = [
    message!("group", "n", pad_group_types),
    message!("path", "s"),
    message!("buttons", "u"),
    message!("done", ""),
    message!("button", "uuu"),
    message!("enter", "uoo", pad_enter_types),
    message!("leave", "uo", pad_leave_types),
    message!("removed", ""),
];
interface!(
    zwp_tablet_pad_v2_interface,
    "zwp_tablet_pad_v2",
    pad_requests: 2,
    pad_events: 8
);

static mut group_ring_types: [*const wl_interface; 1] =
    [unsafe { &zwp_tablet_pad_ring_v2_interface as *const _ }];
static mut group_strip_types: [*const wl_interface; 1] =
    [unsafe { &zwp_tablet_pad_strip_v2_interface as *const _ }];
static mut group_requests: [wl_message; 1] = [message!("destroy", "")];
static mut group_events: [wl_message; 6] = [
    message!("buttons", "a"),
    message!("ring", "n", group_ring_types),
    message!("strip", "n", group_strip_types),
    message!("modes", "u"),
    message!("done", ""),
    message!("mode_switch", "uuu"),
];
interface!(
    zwp_tablet_pad_group_v2_interface,
    "zwp_tablet_pad_group_v2",
    group_requests: 1,
    group_events: 6
);

static mut ring_requests: [wl_message; 2] = [
    message!("set_feedback", "su"),
    message!("destroy", ""),
];
static mut ring_events: [wl_message; 4] = [
    message!("source", "u"),
    message!("angle", "f"),
    message!("stop", ""),
    message!("frame", "u"),
];
interface!(
    zwp_tablet_pad_ring_v2_interface,
    "zwp_tablet_pad_ring_v2",
    ring_requests: 2,
    ring_events: 4
);

static mut strip_requests: [wl_message; 2] = [
    message!("set_feedback", "su"),
    message!("destroy", ""),
];
static mut strip_events: [wl_message; 4] = [
    message!("source", "u"),
    message!("position", "u"),
    message!("stop", ""),
    message!("frame", "u"),
];
interface!(
    zwp_tablet_pad_strip_v2_interface,
    "zwp_tablet_pad_strip_v2",
    strip_requests: 2,
    strip_events: 4
);

crate::wl_listener!(
    zwp_tablet_seat_v2_listener,
    zwp_tablet_seat_v2,
    zwp_tablet_seat_v2_dummy,
    fn tablet_added(id: *mut zwp_tablet_v2),
    fn tool_added(id: *mut zwp_tablet_tool_v2),
    fn pad_added(id: *mut zwp_tablet_pad_v2),
);

crate::wl_listener!(
    zwp_tablet_tool_v2_listener,
    zwp_tablet_tool_v2,
    zwp_tablet_tool_v2_dummy,
    fn type_(tool_type: c_uint),
    fn hardware_serial(hi: c_uint, lo: c_uint),
    fn hardware_id_wacom(hi: c_uint, lo: c_uint),
    fn capability(capability: c_uint),
    fn done(),
    fn removed(),
    fn proximity_in(
        serial: c_uint,
        tablet: *mut zwp_tablet_v2,
        surface: *mut wl_surface,
    ),
    fn proximity_out(),
    fn down(serial: c_uint),
    fn up(),
    fn motion(x: wl_fixed_t, y: wl_fixed_t),
    fn pressure(pressure: c_uint),
    fn distance(distance: c_uint),
    fn tilt(tilt_x: wl_fixed_t, tilt_y: wl_fixed_t),
    fn rotation(degrees: wl_fixed_t),
    fn slider(position: c_int),
    fn wheel(degrees: wl_fixed_t, clicks: c_int),
    fn button(serial: c_uint, button: c_uint, state: c_uint),
    fn frame(time: c_uint),
);
