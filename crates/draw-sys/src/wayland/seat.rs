//! The seat of the connection: `wl_seat`, `wl_keyboard` and `wl_pointer`
//! as [`RawEvent`]s (T169, ADR-T169-1). The listeners fill a queue that
//! `WaylandPlatform::poll` drains; the decisions are in [`input`].

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::ffi::{c_char, c_void};
use std::fs::File;
use std::io::ErrorKind;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::fs::FileExt;
use std::ptr::null_mut;
use std::time::Instant;

use crate::platform::{
    BackendEvent, BackendRecord, Device, DeviceChange, RawEvent, has_release_request,
};
use crate::wayland::client::Proxy;
use crate::wayland::clipboard::ClipboardState;
use crate::wayland::connection::{Shared, shared_of};
use crate::wayland::generated::wayland::{wl_keyboard, wl_pointer, wl_seat};
use crate::wayland::input::{
    AxisFrame, SeatDevices, fixed_to_f32, keymap_usable, pointer_position, raw_button_from_evdev,
    raw_key_from_keysym, repeats_to_send, wheel_of_frame,
};
use crate::wayland::tablet::TabletState;
use crate::wayland::xkb::{KeyState, Keymap, Xkb};

/// The largest keymap the backend reads, in bytes.
const MAX_KEYMAP: usize = 16 * 1024 * 1024;

/// A key held down that repeats.
#[derive(Debug, Clone, Copy)]
struct Held {
    code: u32,
    since: Instant,
    emitted: u32,
}

/// What the seat devices leave behind.
#[derive(Debug)]
pub(crate) struct InputState {
    pub seat: Cell<*mut Proxy>,
    seat_version: Cell<u32>,
    devices: RefCell<SeatDevices>,
    keyboard: Cell<*mut Proxy>,
    pointer: Cell<*mut Proxy>,
    events: RefCell<Vec<RawEvent>>,
    xkb: RefCell<Option<Xkb>>,
    keymap: RefCell<Option<Keymap>>,
    state: RefCell<Option<KeyState>>,
    rate: Cell<i32>,
    delay: Cell<i32>,
    held: Cell<Option<Held>>,
    pos: Cell<(f32, f32)>,
    axis: Cell<AxisFrame>,
    pub tablet: TabletState,
    pub clipboard: ClipboardState,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            seat: Cell::new(null_mut()),
            seat_version: Cell::new(0),
            devices: RefCell::new(SeatDevices::default()),
            keyboard: Cell::new(null_mut()),
            pointer: Cell::new(null_mut()),
            events: RefCell::new(Vec::new()),
            xkb: RefCell::new(None),
            keymap: RefCell::new(None),
            state: RefCell::new(None),
            rate: Cell::new(0),
            delay: Cell::new(0),
            held: Cell::new(None),
            pos: Cell::new((0.0, 0.0)),
            axis: Cell::new(AxisFrame::default()),
            tablet: TabletState::default(),
            clipboard: ClipboardState::default(),
        }
    }
}

impl InputState {
    pub(crate) fn push(&self, event: RawEvent) {
        self.events.borrow_mut().push(event);
    }

    pub(crate) fn record(&self, event: BackendEvent) {
        self.push(RawEvent::Backend(BackendRecord {
            at: Instant::now(),
            event,
        }));
    }

    /// Moves the queued events into `out`.
    pub(crate) fn drain(&self, out: &mut Vec<RawEvent>) {
        out.append(&mut self.events.borrow_mut());
    }

    /// Queues the repeats of the held key that are due at `now`.
    pub(crate) fn poll_repeats(&self, now: Instant) {
        let Some(mut held) = self.held.get() else {
            return;
        };
        let held_for = now.saturating_duration_since(held.since);
        let due = repeats_to_send(self.rate.get(), self.delay.get(), held_for, held.emitted);
        if due == 0 {
            return;
        }
        for _ in 0..due {
            self.key_down(held.code, true);
        }
        // Repeats beyond the cap are dropped, not replayed (T45): count
        // everything that was due as sent.
        let total = crate::wayland::input::repeats_due(
            self.rate.get(),
            self.delay.get(),
            held_for,
            held.emitted,
        );
        held.emitted = held.emitted.saturating_add(total);
        self.held.set(Some(held));
    }

    /// Queues a key press, and the characters it types.
    fn key_down(&self, code: u32, repeat: bool) {
        let state = self.state.borrow();
        let Some(state) = state.as_ref() else {
            return;
        };
        let mods = state.mods();
        let key = raw_key_from_keysym(state.keysym(code));
        self.push(RawEvent::Key {
            key,
            mods,
            down: true,
            repeat,
        });
        for ch in state.utf8(code).chars().filter(|c| !c.is_control()) {
            self.push(RawEvent::Char { ch, mods, repeat });
        }
    }

    fn key_up(&self, code: u32) {
        let state = self.state.borrow();
        let Some(state) = state.as_ref() else {
            return;
        };
        self.push(RawEvent::Key {
            key: raw_key_from_keysym(state.keysym(code)),
            mods: state.mods(),
            down: false,
            repeat: false,
        });
    }

    /// Reads and installs a keymap; `false` if it is unusable and the last
    /// good one stays (ADR-T79-1).
    fn install_keymap(&self, fd: OwnedFd, format: u32, size: u32) -> bool {
        if !keymap_usable(format, size) {
            return false;
        }
        let file = File::from(fd);
        let want = usize::try_from(size).unwrap_or(MAX_KEYMAP).min(MAX_KEYMAP);
        let mut text = vec![0_u8; want];
        let mut got = 0;
        while got < want {
            match file.read_at(&mut text[got..], got as u64) {
                Ok(0) => break,
                Ok(n) => got += n,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => return false,
            }
        }
        text.truncate(got);
        if self.xkb.borrow().is_none() {
            *self.xkb.borrow_mut() = Xkb::load().ok();
        }
        let xkb = self.xkb.borrow();
        let Some(keymap) = xkb.as_ref().and_then(|x| x.keymap_from_text(&text)) else {
            return false;
        };
        let Some(state) = keymap.state() else {
            return false;
        };
        // Built before the old pair is released.
        *self.keymap.borrow_mut() = Some(keymap);
        *self.state.borrow_mut() = Some(state);
        true
    }
}

extern "C" fn on_capabilities(data: *mut c_void, seat: *mut Proxy, caps: u32) {
    // TEST: wayland_seat_has_one_keyboard_and_one_pointer
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    let input = &shared.input;
    let c = &shared.client;
    input.record(BackendEvent::Capabilities {
        keyboard: caps & 2 != 0,
        pointer: caps & 1 != 0,
        touch: caps & 4 != 0,
    });
    let changes = input.devices.borrow_mut().on_capabilities(caps);
    for (device, change) in changes {
        let release = has_release_request(input.seat_version.get());
        match (device, change) {
            (Device::Keyboard, DeviceChange::Create) => {
                // TEST: wayland_seat_has_one_keyboard_and_one_pointer
                // SAFETY: `seat` is the live `wl_seat` this event came from.
                let proxy = unsafe { wl_seat::get_keyboard(c, seat) };
                input.keyboard.set(proxy);
                if !proxy.is_null() {
                    // TEST: wayland_seat_has_one_keyboard_and_one_pointer
                    // SAFETY: `proxy` is the live keyboard just made; `data`
                    // outlives it (`Shared::drop` destroys it).
                    unsafe { wl_keyboard::add_listener(c, proxy, &KEYBOARD_LISTENER, data) };
                }
            }
            (Device::Pointer, DeviceChange::Create) => {
                // TEST: wayland_seat_has_one_keyboard_and_one_pointer
                // SAFETY: `seat` is the live `wl_seat` this event came from.
                let proxy = unsafe { wl_seat::get_pointer(c, seat) };
                input.pointer.set(proxy);
                if !proxy.is_null() {
                    // TEST: wayland_seat_has_one_keyboard_and_one_pointer
                    // SAFETY: as for the keyboard.
                    unsafe { wl_pointer::add_listener(c, proxy, &POINTER_LISTENER, data) };
                }
            }
            (Device::Keyboard, DeviceChange::Release) => {
                release_keyboard(shared, release);
            }
            (Device::Pointer, DeviceChange::Release) => {
                let proxy = input.pointer.replace(null_mut());
                destroy(shared, proxy, release, true);
            }
            _ => continue,
        }
        input.record(match change {
            DeviceChange::Create => BackendEvent::DeviceCreated(device),
            DeviceChange::Release | DeviceChange::Keep => BackendEvent::DeviceReleased(device),
        });
    }
}

/// Releases (version 3 and later) or destroys a seat device proxy.
fn destroy(shared: &Shared, proxy: *mut Proxy, release: bool, pointer: bool) {
    if proxy.is_null() {
        return;
    }
    let c = &shared.client;
    if release && pointer {
        // TEST: wayland_seat_has_one_keyboard_and_one_pointer
        // SAFETY: `proxy` is a live `wl_pointer`, released once (a destructor
        // request).
        unsafe { wl_pointer::release(c, proxy) };
    } else if release {
        // TEST: wayland_seat_has_one_keyboard_and_one_pointer
        // SAFETY: `proxy` is a live `wl_keyboard`, released once.
        unsafe { wl_keyboard::release(c, proxy) };
    } else {
        // TEST: wayland_seat_has_one_keyboard_and_one_pointer
        // SAFETY: `proxy` is live and destroyed once.
        unsafe { (c.proxy_destroy)(proxy) };
    }
}

fn release_keyboard(shared: &Shared, release: bool) {
    let input = &shared.input;
    let proxy = input.keyboard.replace(null_mut());
    destroy(shared, proxy, release, false);
    input.held.set(None);
    input.push(RawEvent::Focus(false));
}

extern "C" fn on_seat_name(_data: *mut c_void, _seat: *mut Proxy, _name: *const c_char) {}

extern "C" fn on_keymap(data: *mut c_void, _kb: *mut Proxy, format: u32, fd: i32, size: u32) {
    // TEST: wayland_keyboard_typed_text_arrives
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    let input = &shared.input;
    if fd < 0 {
        input.record(BackendEvent::KeymapIgnored { format, size });
        return;
    }
    // TEST: wayland_keyboard_typed_text_arrives
    // SAFETY: libwayland hands over ownership of the keymap descriptor; it
    // is closed when the `OwnedFd` drops, on every path.
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    if input.install_keymap(fd, format, size) {
        input.record(BackendEvent::Keymap);
    } else {
        input.record(BackendEvent::KeymapIgnored { format, size });
    }
}

extern "C" fn on_kb_enter(
    data: *mut c_void,
    _kb: *mut Proxy,
    serial: u32,
    _surface: *mut Proxy,
    _keys: *mut c_void,
) {
    // TEST: wayland_keyboard_focus_is_reported
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    input.clipboard.note_serial(serial);
    input.record(BackendEvent::KeyboardEnter);
    input.push(RawEvent::Focus(true));
}

extern "C" fn on_kb_leave(data: *mut c_void, _kb: *mut Proxy, _serial: u32, _surface: *mut Proxy) {
    // TEST: wayland_keyboard_focus_is_reported
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    input.held.set(None);
    input.record(BackendEvent::KeyboardLeave);
    input.push(RawEvent::Focus(false));
}

extern "C" fn on_key(
    data: *mut c_void,
    _kb: *mut Proxy,
    serial: u32,
    _time: u32,
    code: u32,
    state: u32,
) {
    // TEST: wayland_keyboard_typed_text_arrives
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    input.clipboard.note_serial(serial);
    let mods = input.state.borrow().as_ref().map(KeyState::mods);
    input.record(BackendEvent::Key {
        code,
        state,
        ctrl: mods.is_some_and(|m| m.ctrl),
        alt: mods.is_some_and(|m| m.alt),
    });
    match state {
        1 => {
            input.key_down(code, false);
            let repeats = input
                .keymap
                .borrow()
                .as_ref()
                .is_some_and(|k| k.key_repeats(code));
            input.held.set(repeats.then(|| Held {
                code,
                since: Instant::now(),
                emitted: 0,
            }));
        }
        0 => {
            input.key_up(code);
            if input.held.get().is_some_and(|h| h.code == code) {
                input.held.set(None);
            }
        }
        _ => {}
    }
}

extern "C" fn on_modifiers(
    data: *mut c_void,
    _kb: *mut Proxy,
    _serial: u32,
    depressed: u32,
    latched: u32,
    locked: u32,
    group: u32,
) {
    // TEST: wayland_keyboard_modifiers_arrive
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    if let Some(state) = input.state.borrow_mut().as_mut() {
        state.update_mask(depressed, latched, locked, group);
    }
}

extern "C" fn on_repeat_info(data: *mut c_void, _kb: *mut Proxy, rate: i32, delay: i32) {
    // TEST: wayland_keyboard_typed_text_arrives
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    input.rate.set(rate);
    input.delay.set(delay);
    input.record(BackendEvent::RepeatInfo { rate, delay });
}

/// Queues a pointer move to `(x, y)` fixed-point surface coordinates.
fn pointer_to(shared: &Shared, x: i32, y: i32) {
    // A tool in proximity sends the same movement itself (ADR-T40-1).
    if shared.input.tablet.shadowed() {
        return;
    }
    let pos = pointer_position(x, y, shared.scale());
    shared.input.pos.set(pos);
    shared
        .input
        .push(RawEvent::PointerMove { x: pos.0, y: pos.1 });
}

extern "C" fn on_ptr_enter(
    data: *mut c_void,
    _ptr: *mut Proxy,
    serial: u32,
    _surface: *mut Proxy,
    x: i32,
    y: i32,
) {
    // TEST: wayland_pointer_motion_arrives
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    shared.input.clipboard.note_serial(serial);
    pointer_to(shared, x, y);
}

extern "C" fn on_ptr_motion(data: *mut c_void, _ptr: *mut Proxy, _time: u32, x: i32, y: i32) {
    // TEST: wayland_pointer_motion_arrives
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    pointer_to(unsafe { shared_of(data) }, x, y);
}

extern "C" fn on_ptr_button(
    data: *mut c_void,
    _ptr: *mut Proxy,
    serial: u32,
    _time: u32,
    button: u32,
    state: u32,
) {
    // TEST: wayland_pointer_buttons_arrive
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    input.clipboard.note_serial(serial);
    if input.tablet.shadowed() {
        return;
    }
    let (x, y) = input.pos.get();
    input.push(RawEvent::Button {
        button: raw_button_from_evdev(button),
        x,
        y,
        down: state == 1,
    });
}

extern "C" fn on_ptr_axis(data: *mut c_void, _ptr: *mut Proxy, _time: u32, axis: u32, value: i32) {
    // TEST: wayland_pointer_wheel_arrives
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    let mut frame = input.axis.get();
    match axis {
        0 => frame.dy += fixed_to_f32(value),
        1 => frame.dx += fixed_to_f32(value),
        _ => {}
    }
    input.axis.set(frame);
}

extern "C" fn on_ptr_value120(data: *mut c_void, _ptr: *mut Proxy, axis: u32, value: i32) {
    // TEST: wayland_pointer_wheel_arrives
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    let mut frame = input.axis.get();
    match axis {
        0 => frame.v120_y = Some(frame.v120_y.unwrap_or(0).saturating_add(value)),
        1 => frame.v120_x = Some(frame.v120_x.unwrap_or(0).saturating_add(value)),
        _ => {}
    }
    input.axis.set(frame);
}

extern "C" fn on_ptr_frame(data: *mut c_void, _ptr: *mut Proxy) {
    // TEST: wayland_pointer_wheel_arrives
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let input = &unsafe { shared_of(data) }.input;
    let frame = input.axis.replace(AxisFrame::default());
    if let Some((dx, dy)) = wheel_of_frame(&frame) {
        input.push(RawEvent::Wheel { dx, dy });
    }
}

/// The listener of the seat.
pub(crate) static SEAT_LISTENER: wl_seat::Listener = wl_seat::Listener {
    capabilities: on_capabilities,
    name: on_seat_name,
};

static KEYBOARD_LISTENER: wl_keyboard::Listener = wl_keyboard::Listener {
    keymap: on_keymap,
    enter: on_kb_enter,
    leave: on_kb_leave,
    key: on_key,
    modifiers: on_modifiers,
    repeat_info: on_repeat_info,
};

static POINTER_LISTENER: wl_pointer::Listener = wl_pointer::Listener {
    enter: on_ptr_enter,
    motion: on_ptr_motion,
    button: on_ptr_button,
    axis: on_ptr_axis,
    frame: on_ptr_frame,
    axis_value120: on_ptr_value120,
    ..wl_pointer::IGNORE
};

impl InputState {
    /// Records the version the seat was bound at.
    pub(crate) fn set_seat(&self, proxy: *mut Proxy, version: u32) {
        self.seat.set(proxy);
        self.seat_version.set(version);
    }

    /// Releases the devices and the seat before the connection closes.
    pub(crate) fn destroy_devices(&self, shared: &Shared) {
        let release = has_release_request(self.seat_version.get());
        self.tablet.destroy(shared);
        self.clipboard.destroy(shared);
        destroy(shared, self.keyboard.replace(null_mut()), release, false);
        destroy(shared, self.pointer.replace(null_mut()), release, true);
        let seat = self.seat.replace(null_mut());
        if !seat.is_null() {
            if release && self.seat_version.get() >= 5 {
                // TEST: wayland_seat_has_one_keyboard_and_one_pointer
                // SAFETY: `seat` is a live `wl_seat`, released once.
                unsafe { wl_seat::release(&shared.client, seat) };
            } else {
                // TEST: wayland_seat_has_one_keyboard_and_one_pointer
                // SAFETY: `seat` is live and destroyed once.
                unsafe { (shared.client.proxy_destroy)(seat) };
            }
        }
    }
}
