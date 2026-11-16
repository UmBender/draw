//! The clipboard of the seat: `wl_data_device_manager`, `wl_data_device`,
//! offers (read) and a `wl_data_source` (write) for text and `image/png`
//! (T170, ADR-T170-1). The decisions are in [`crate::wayland::transfer`].

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, CString, c_char, c_ulong, c_void};
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::ptr::null_mut;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::wayland::client::Proxy;
use crate::wayland::connection::{Shared, shared_of};
use crate::wayland::generated::wayland::{
    wl_data_device, wl_data_device_manager, wl_data_offer, wl_data_source,
};
use crate::wayland::transfer::{
    MAX_TRANSFER, Payload, payload_bytes, pick_text_mime, read_limited, serves, source_mimes,
    write_payload,
};

/// The data device manager version bound.
pub(crate) const MANAGER_VERSION: u32 = 3;

/// How long a clipboard read waits for the other client in total.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

const O_CLOEXEC: i32 = 0o200_0000;
const POLLIN: i16 = 1;

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

// TEST: wayland_clipboard_text_reaches_wl_paste
unsafe extern "C" {
    fn pipe2(fds: *mut i32, flags: i32) -> i32;
    fn poll(fds: *mut PollFd, nfds: c_ulong, timeout: i32) -> i32;
}

/// A pipe: the read end and the write end.
fn pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut fds = [0_i32; 2];
    // TEST: wayland_clipboard_text_reaches_wl_paste
    // SAFETY: `fds` has room for the two descriptors `pipe2` writes.
    if unsafe { pipe2(fds.as_mut_ptr(), O_CLOEXEC) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // TEST: wayland_clipboard_text_reaches_wl_paste
    // SAFETY: `pipe2` succeeded: both descriptors are open and owned here.
    Ok(unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) })
}

/// A reader that gives up at a deadline instead of blocking forever on a
/// client that never writes.
struct Deadline {
    file: File,
    until: Instant,
}

impl Read for Deadline {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let left = self.until.saturating_duration_since(Instant::now());
        let ms = i32::try_from(left.as_millis()).unwrap_or(i32::MAX);
        let mut fd = PollFd {
            fd: self.file.as_raw_fd(),
            events: POLLIN,
            revents: 0,
        };
        // TEST: wayland_clipboard_reads_text_from_wl_copy
        // SAFETY: `fd` is one valid `pollfd`.
        let ready = unsafe { poll(&raw mut fd, 1, ms) };
        if ready == 0 {
            return Err(io::ErrorKind::TimedOut.into());
        }
        if ready < 0 {
            let err = io::Error::last_os_error();
            return if err.kind() == io::ErrorKind::Interrupted {
                self.read(buf)
            } else {
                Err(err)
            };
        }
        self.file.read(buf)
    }
}

/// The clipboard objects and state of the connection.
#[derive(Debug)]
pub(crate) struct ClipboardState {
    pub manager: Cell<*mut Proxy>,
    manager_version: Cell<u32>,
    device: Cell<*mut Proxy>,
    serial: Cell<u32>,
    /// The offers made so far with their MIME types, by proxy address.
    offers: RefCell<Vec<(usize, Vec<String>)>>,
    /// The offer of the current selection of another client.
    selection: Cell<*mut Proxy>,
    /// The source the process owns, with what it serves.
    source: Cell<*mut Proxy>,
    payload: RefCell<Option<Arc<Payload>>>,
}

impl Default for ClipboardState {
    fn default() -> Self {
        Self {
            manager: Cell::new(null_mut()),
            manager_version: Cell::new(0),
            device: Cell::new(null_mut()),
            serial: Cell::new(0),
            offers: RefCell::new(Vec::new()),
            selection: Cell::new(null_mut()),
            source: Cell::new(null_mut()),
            payload: RefCell::new(None),
        }
    }
}

impl ClipboardState {
    /// Keeps the serial of the latest input event: `set_selection` needs one.
    pub(crate) fn note_serial(&self, serial: u32) {
        self.serial.set(serial);
    }

    /// Records the manager and the version it was bound at.
    pub(crate) fn set_manager(&self, proxy: *mut Proxy, version: u32) {
        self.manager.set(proxy);
        self.manager_version.set(version);
    }

    /// Takes the data device of `seat`.
    pub(crate) fn init(&self, shared: &Shared, seat: *mut Proxy) {
        if self.manager.get().is_null() || seat.is_null() || !self.device.get().is_null() {
            return;
        }
        let data = std::ptr::from_ref(shared).cast_mut().cast::<c_void>();
        // TEST: wayland_clipboard_empty_is_none
        // SAFETY: `manager` and `seat` are live proxies of this connection.
        let device =
            unsafe { wl_data_device_manager::get_data_device(&shared.client, self.manager.get(), seat) };
        self.device.set(device);
        if !device.is_null() {
            // TEST: wayland_clipboard_empty_is_none
            // SAFETY: `device` is live; `data` outlives it.
            unsafe { wl_data_device::add_listener(&shared.client, device, &DEVICE_LISTENER, data) };
        }
    }

    fn mimes_of(&self, offer: *mut Proxy) -> Vec<String> {
        self.offers
            .borrow()
            .iter()
            .find(|(id, _)| *id == offer as usize)
            .map(|(_, mimes)| mimes.clone())
            .unwrap_or_default()
    }

    /// The clipboard's text: the process's own copy, or what the current
    /// offer gives for its best text type.
    pub(crate) fn text(&self, shared: &Shared) -> Option<String> {
        if let Some(own) = self.payload.borrow().as_deref() {
            return match own {
                Payload::Text(text) => Some(text.clone()),
                Payload::Png(_) => None,
            };
        }
        let offer = self.selection.get();
        if offer.is_null() {
            return None;
        }
        let mimes = self.mimes_of(offer);
        let mime = CString::new(pick_text_mime(&mimes)?).ok()?;
        let (read, write) = pipe().ok()?;
        // TEST: wayland_clipboard_reads_text_from_wl_copy
        // SAFETY: `offer` is a live `wl_data_offer`; `receive` passes a copy
        // of the descriptor to the compositor, the write end closes below.
        unsafe { wl_data_offer::receive(&shared.client, offer, mime.as_ptr(), write.as_raw_fd()) };
        // TEST: wayland_clipboard_reads_text_from_wl_copy
        // SAFETY: `display` is live while `shared` is.
        unsafe { (shared.client.display_flush)(shared.display) };
        drop(write);
        let reader = Deadline {
            file: File::from(read),
            until: Instant::now() + READ_TIMEOUT,
        };
        let bytes = read_limited(reader, MAX_TRANSFER).ok()?;
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Becomes the owner of the selection with `payload`.
    pub(crate) fn set(&self, shared: &Shared, payload: Payload) {
        let (manager, device) = (self.manager.get(), self.device.get());
        if manager.is_null() || device.is_null() {
            return;
        }
        let c = &shared.client;
        let data = std::ptr::from_ref(shared).cast_mut().cast::<c_void>();
        // TEST: wayland_clipboard_text_reaches_wl_paste
        // SAFETY: `manager` is a live `wl_data_device_manager`.
        let source = unsafe { wl_data_device_manager::create_data_source(c, manager) };
        if source.is_null() {
            return;
        }
        // TEST: wayland_clipboard_text_reaches_wl_paste
        // SAFETY: `source` is live; `data` outlives it.
        unsafe { wl_data_source::add_listener(c, source, &SOURCE_LISTENER, data) };
        for mime in source_mimes(&payload) {
            if let Ok(mime) = CString::new(*mime) {
                // TEST: wayland_clipboard_text_reaches_wl_paste
                // SAFETY: `source` is live; `mime` is NUL-terminated.
                unsafe { wl_data_source::offer(c, source, mime.as_ptr()) };
            }
        }
        // TEST: wayland_clipboard_text_reaches_wl_paste
        // SAFETY: `device` and `source` are live.
        unsafe { wl_data_device::set_selection(c, device, source, self.serial.get()) };
        let old = self.source.replace(source);
        *self.payload.borrow_mut() = Some(Arc::new(payload));
        if !old.is_null() {
            // TEST: wayland_clipboard_text_reaches_wl_paste
            // SAFETY: the old source is live and destroyed once.
            unsafe { wl_data_source::destroy(c, old) };
        }
        // TEST: wayland_clipboard_text_reaches_wl_paste
        // SAFETY: `display` is live while `shared` is.
        unsafe { (c.display_flush)(shared.display) };
    }

    /// Destroys the clipboard objects before the connection closes.
    pub(crate) fn destroy(&self, shared: &Shared) {
        let c = &shared.client;
        let source = self.source.replace(null_mut());
        if !source.is_null() {
            // TEST: wayland_clipboard_text_reaches_wl_paste
            // SAFETY: live, destroyed once.
            unsafe { wl_data_source::destroy(c, source) };
        }
        let offers: Vec<usize> = self.offers.borrow_mut().drain(..).map(|(id, _)| id).collect();
        for id in offers {
            // TEST: wayland_clipboard_empty_is_none
            // SAFETY: the offers are live proxies, destroyed once.
            unsafe { wl_data_offer::destroy(c, id as *mut Proxy) };
        }
        self.selection.set(null_mut());
        let device = self.device.replace(null_mut());
        if !device.is_null() {
            if self.manager_version.get() >= 2 {
                // TEST: wayland_clipboard_empty_is_none
                // SAFETY: live, released once.
                unsafe { wl_data_device::release(c, device) };
            } else {
                // TEST: wayland_clipboard_empty_is_none
                // SAFETY: live, destroyed once.
                unsafe { (c.proxy_destroy)(device) };
            }
        }
        let manager = self.manager.replace(null_mut());
        if !manager.is_null() {
            // TEST: wayland_clipboard_empty_is_none
            // SAFETY: live, destroyed once.
            unsafe { (c.proxy_destroy)(manager) };
        }
    }
}

extern "C" fn on_data_offer(data: *mut c_void, _device: *mut Proxy, offer: *mut Proxy) {
    // TEST: wayland_clipboard_reads_text_from_wl_copy
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    shared
        .input
        .clipboard
        .offers
        .borrow_mut()
        .push((offer as usize, Vec::new()));
    // TEST: wayland_clipboard_reads_text_from_wl_copy
    // SAFETY: `offer` is the live proxy this event created; `data` outlives it.
    unsafe { wl_data_offer::add_listener(&shared.client, offer, &OFFER_LISTENER, data) };
}

extern "C" fn on_offer_mime(data: *mut c_void, offer: *mut Proxy, mime: *const c_char) {
    if mime.is_null() {
        return;
    }
    // TEST: wayland_clipboard_reads_text_from_wl_copy
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    // TEST: wayland_clipboard_reads_text_from_wl_copy
    // SAFETY: `mime` is a NUL-terminated string valid for the call.
    let mime = unsafe { CStr::from_ptr(mime) }.to_string_lossy().into_owned();
    let mut offers = shared.input.clipboard.offers.borrow_mut();
    if let Some((_, mimes)) = offers.iter_mut().find(|(id, _)| *id == offer as usize) {
        mimes.push(mime);
    }
}

extern "C" fn on_selection(data: *mut c_void, _device: *mut Proxy, offer: *mut Proxy) {
    // TEST: wayland_clipboard_reads_text_from_wl_copy
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    let clip = &shared.input.clipboard;
    let old = clip.selection.replace(offer);
    if !old.is_null() && old != offer {
        clip.offers.borrow_mut().retain(|(id, _)| *id != old as usize);
        // TEST: wayland_clipboard_reads_text_from_wl_copy
        // SAFETY: `old` is a live offer, destroyed once.
        unsafe { wl_data_offer::destroy(&shared.client, old) };
    }
}

extern "C" fn on_send(data: *mut c_void, source: *mut Proxy, mime: *const c_char, fd: i32) {
    // TEST: wayland_clipboard_text_reaches_wl_paste
    // SAFETY: libwayland hands over ownership of the descriptor; it closes
    // when the `OwnedFd` drops, on every path.
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    if mime.is_null() {
        return;
    }
    // TEST: wayland_clipboard_text_reaches_wl_paste
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    let clip = &shared.input.clipboard;
    // TEST: wayland_clipboard_text_reaches_wl_paste
    // SAFETY: `mime` is a NUL-terminated string valid for the call.
    let mime = unsafe { CStr::from_ptr(mime) }.to_string_lossy();
    let payload = match clip.payload.borrow().as_ref() {
        Some(p) if clip.source.get() == source && serves(p, &mime) => Arc::clone(p),
        _ => return,
    };
    // The write may block on a slow reader: not on the loop's thread.
    let _ = std::thread::Builder::new()
        .name("clipboard-send".into())
        .spawn(move || {
            let _ = write_payload(File::from(fd), payload_bytes(&payload));
        });
}

extern "C" fn on_cancelled(data: *mut c_void, source: *mut Proxy) {
    // TEST: wayland_clipboard_text_reaches_wl_paste
    // SAFETY: libwayland passes the data pointer given to `add_listener`.
    let shared = unsafe { shared_of(data) };
    let clip = &shared.input.clipboard;
    if clip.source.get() == source {
        clip.source.set(null_mut());
        *clip.payload.borrow_mut() = None;
    }
    // TEST: wayland_clipboard_text_reaches_wl_paste
    // SAFETY: a cancelled source is live until destroyed, once, here.
    unsafe { wl_data_source::destroy(&shared.client, source) };
}

static DEVICE_LISTENER: wl_data_device::Listener = wl_data_device::Listener {
    data_offer: on_data_offer,
    selection: on_selection,
    ..wl_data_device::IGNORE
};

static OFFER_LISTENER: wl_data_offer::Listener = wl_data_offer::Listener {
    offer: on_offer_mime,
    ..wl_data_offer::IGNORE
};

static SOURCE_LISTENER: wl_data_source::Listener = wl_data_source::Listener {
    send: on_send,
    cancelled: on_cancelled,
    ..wl_data_source::IGNORE
};
