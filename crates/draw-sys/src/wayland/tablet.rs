//! The tablet side of the seat: `zwp_tablet_manager_v2`,
//! `zwp_tablet_seat_v2` and the tools as [`RawEvent`]s (T170, ADR-T170-1).
//! The listeners record into [`Pens`]; the decisions are in [`crate::wayland::pen`].

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::ptr::null_mut;

use crate::platform::BackendEvent;
use crate::wayland::client::Proxy;
use crate::wayland::connection::{Shared, shared_of};
use crate::wayland::generated::tablet_v2::{
    zwp_tablet_manager_v2, zwp_tablet_pad_v2, zwp_tablet_seat_v2, zwp_tablet_tool_v2, zwp_tablet_v2,
};
use crate::wayland::pen::Pens;

/// The manager version bound: version 2 only adds pad dials, which the
/// generated code does not know.
pub(crate) const MANAGER_VERSION: u32 = 1;

/// The tablet objects of the connection.
#[derive(Debug)]
pub(crate) struct TabletState {
    pub manager: Cell<*mut Proxy>,
    seat: Cell<*mut Proxy>,
    pens: RefCell<Pens>,
}

impl Default for TabletState {
    fn default() -> Self {
        Self {
            manager: Cell::new(null_mut()),
            seat: Cell::new(null_mut()),
            pens: RefCell::new(Pens::default()),
        }
    }
}

impl TabletState {
    /// Whether a tool in proximity makes `wl_pointer` input a copy to drop
    /// (ADR-T40-1).
    pub(crate) fn shadowed(&self) -> bool {
        self.pens.borrow().shadowed()
    }

    /// Takes the tablet seat of `seat`, if the compositor has tablets.
    pub(crate) fn init(&self, shared: &Shared, seat: *mut Proxy) {
        if self.manager.get().is_null() || seat.is_null() || !self.seat.get().is_null() {
            return;
        }
        let data = std::ptr::from_ref(shared).cast_mut().cast::<c_void>();
        // TEST: wayland_tablet_manager_is_bound_without_tablets
        // SAFETY: `manager` and `seat` are live proxies of this connection.
        let proxy = unsafe {
            zwp_tablet_manager_v2::get_tablet_seat(&shared.client, self.manager.get(), seat)
        };
        self.seat.set(proxy);
        if !proxy.is_null() {
            // TEST: wayland_tablet_manager_is_bound_without_tablets
            // SAFETY: `proxy` is the live tablet seat just made; `data`
            // outlives it (`Shared::drop` destroys it).
            unsafe {
                zwp_tablet_seat_v2::add_listener(&shared.client, proxy, &SEAT_LISTENER, data)
            };
        }
    }

    /// Destroys the tablet objects before the connection closes.
    pub(crate) fn destroy(&self, shared: &Shared) {
        let c = &shared.client;
        let ids: Vec<usize> = self.pens.borrow().ids();
        for id in ids {
            // TEST: wayland_tablet_manager_is_bound_without_tablets
            // SAFETY: the ids are live tool proxies, destroyed once here.
            unsafe { zwp_tablet_tool_v2::destroy(c, id as *mut Proxy) };
        }
        let seat = self.seat.replace(null_mut());
        if !seat.is_null() {
            // TEST: wayland_tablet_manager_is_bound_without_tablets
            // SAFETY: live, destroyed once.
            unsafe { zwp_tablet_seat_v2::destroy(c, seat) };
        }
        let manager = self.manager.replace(null_mut());
        if !manager.is_null() {
            // TEST: wayland_tablet_manager_is_bound_without_tablets
            // SAFETY: live, destroyed once.
            unsafe { zwp_tablet_manager_v2::destroy(c, manager) };
        }
    }
}

extern "C" fn on_tablet_added(data: *mut c_void, _seat: *mut Proxy, tablet: *mut Proxy) {
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    // The tablets themselves are not used: only their tools.
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: `tablet` is the live proxy this event created.
    unsafe { zwp_tablet_v2::destroy(&shared.client, tablet) };
}

extern "C" fn on_pad_added(data: *mut c_void, _seat: *mut Proxy, pad: *mut Proxy) {
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: `pad` is the live proxy this event created.
    unsafe { zwp_tablet_pad_v2::destroy(&shared.client, pad) };
}

extern "C" fn on_tool_added(data: *mut c_void, _seat: *mut Proxy, tool: *mut Proxy) {
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    let tablet = &shared.input.tablet;
    tablet.pens.borrow_mut().add(tool as usize);
    shared.input.record(BackendEvent::ToolAdded);
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: `tool` is the live proxy this event created; `data` outlives it.
    unsafe { zwp_tablet_tool_v2::add_listener(&shared.client, tool, &TOOL_LISTENER, data) };
}

/// Runs `f` on the tool `tool` if it is one of ours.
fn with_tool(
    data: *mut c_void,
    tool: *mut Proxy,
    f: impl FnOnce(&Shared, &mut crate::wayland::pen::PenTool),
) {
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    let mut pens = shared.input.tablet.pens.borrow_mut();
    if let Some(state) = pens.get_mut(tool as usize) {
        f(shared, state);
    }
}

extern "C" fn on_type(data: *mut c_void, tool: *mut Proxy, kind: u32) {
    with_tool(data, tool, |_, t| t.set_kind(kind));
}

extern "C" fn on_removed(data: *mut c_void, tool: *mut Proxy) {
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    shared.input.tablet.pens.borrow_mut().remove(tool as usize);
    shared.input.record(BackendEvent::ToolRemoved);
    // TEST: wayland_tablet_manager_is_bound_without_tablets
    // SAFETY: `tool` is live and destroyed once, after `removed`.
    unsafe { zwp_tablet_tool_v2::destroy(&shared.client, tool) };
}

extern "C" fn on_proximity_in(
    data: *mut c_void,
    tool: *mut Proxy,
    serial: u32,
    _tablet: *mut Proxy,
    surface: *mut Proxy,
) {
    with_tool(data, tool, |shared, t| {
        // One window: any surface is ours.
        t.proximity_in(!surface.is_null());
        shared.input.clipboard.note_serial(serial);
        shared.input.record(BackendEvent::ProximityIn);
    });
}

extern "C" fn on_proximity_out(data: *mut c_void, tool: *mut Proxy) {
    with_tool(data, tool, |shared, t| {
        t.proximity_out();
        shared.input.record(BackendEvent::ProximityOut);
    });
}

extern "C" fn on_down(data: *mut c_void, tool: *mut Proxy, serial: u32) {
    with_tool(data, tool, |shared, t| {
        t.down();
        shared.input.clipboard.note_serial(serial);
    });
}

extern "C" fn on_up(data: *mut c_void, tool: *mut Proxy) {
    with_tool(data, tool, |_, t| t.up());
}

extern "C" fn on_motion(data: *mut c_void, tool: *mut Proxy, x: i32, y: i32) {
    with_tool(data, tool, |shared, t| t.motion(x, y, shared.scale()));
}

extern "C" fn on_pressure(data: *mut c_void, tool: *mut Proxy, pressure: u32) {
    with_tool(data, tool, |_, t| t.pressure(pressure));
}

extern "C" fn on_tilt(data: *mut c_void, tool: *mut Proxy, x: i32, y: i32) {
    with_tool(data, tool, |_, t| t.tilt(x, y));
}

extern "C" fn on_button(data: *mut c_void, tool: *mut Proxy, serial: u32, button: u32, state: u32) {
    with_tool(data, tool, |shared, t| {
        t.button(button, state);
        shared.input.clipboard.note_serial(serial);
    });
}

extern "C" fn on_frame(data: *mut c_void, tool: *mut Proxy, _time: u32) {
    with_tool(data, tool, |shared, t| {
        for event in t.frame() {
            shared.input.push(event);
        }
    });
}

static SEAT_LISTENER: zwp_tablet_seat_v2::Listener = zwp_tablet_seat_v2::Listener {
    tablet_added: on_tablet_added,
    tool_added: on_tool_added,
    pad_added: on_pad_added,
};

static TOOL_LISTENER: zwp_tablet_tool_v2::Listener = zwp_tablet_tool_v2::Listener {
    r#type: on_type,
    removed: on_removed,
    proximity_in: on_proximity_in,
    proximity_out: on_proximity_out,
    down: on_down,
    up: on_up,
    motion: on_motion,
    pressure: on_pressure,
    tilt: on_tilt,
    button: on_button,
    frame: on_frame,
    ..zwp_tablet_tool_v2::IGNORE
};
