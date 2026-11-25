//! The Xlib calls of the window part, loaded with `dlopen` (ADR-T142-1).
//! The only file of the X11 backend with `unsafe`: a [`Connection`] owns the
//! `Display*`, a [`Window`] owns a window and its colormap, and their types
//! carry what X11 orders (created, then mapped).

#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_int, c_long, c_uint, c_ulong, c_void};
use std::marker::PhantomData;
use std::ptr::null;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::egl::{EglContext, Platform as EglPlatform};
use crate::glx::GlxContext;
use crate::loader::Library;

use super::event::{CloseAtoms, EVENT_BYTES, XEvent, decode};
use super::pen::PressureAxis;
use super::platform::GlSurface;
use super::selection::{MAX_INCOMING, Request, TargetKind};
use super::{GlBackend, X11Error};

// The event decoder reads the LP64 layout of Xlib's structs.
const _: () = assert!(size_of::<c_long>() == 8);
const _: () = assert!(size_of::<[c_long; 24]>() == EVENT_BYTES);

type Ptr = *mut c_void;

const CW_BORDER_PIXEL: c_ulong = 1 << 3;
const CW_EVENT_MASK: c_ulong = 1 << 11;
const CW_COLORMAP: c_ulong = 1 << 13;
const INPUT_OUTPUT: c_uint = 1;
const EXPOSURE_MASK: c_long = 1 << 15;
const VISIBILITY_CHANGE_MASK: c_long = 1 << 16;
const STRUCTURE_NOTIFY_MASK: c_long = 1 << 17;
const PROPERTY_CHANGE_MASK: c_long = 1 << 22;

const POLLIN: i16 = 0x0001;

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: i16,
    revents: i16,
}

// TEST: x11_platform_schedule_update_ends_the_wait
// SAFETY: standard POSIX `poll` from libc.
unsafe extern "C" {
    fn poll(fds: *mut PollFd, nfds: c_ulong, timeout: c_int) -> c_int;
}

/// A window property read back from the server.
#[derive(Debug)]
pub(super) struct Property {
    /// The type atom.
    pub(super) ty: u64,
    /// The raw bytes.
    pub(super) data: Vec<u8>,
}

/// The atoms the selections use.
#[derive(Debug, Clone, Copy)]
pub(super) struct SelAtoms {
    pub(super) clipboard: u64,
    pub(super) primary: u64,
    pub(super) targets: u64,
    pub(super) utf8: u64,
    pub(super) string: u64,
    pub(super) text: u64,
    pub(super) plain_utf8: u64,
    pub(super) png: u64,
    pub(super) incr: u64,
    pub(super) atom: u64,
    /// The property draw's own reads land in.
    pub(super) property: u64,
}

impl SelAtoms {
    /// The kind a target atom stands for.
    pub(super) fn kind(&self, atom: u64) -> TargetKind {
        match atom {
            a if a == self.targets => TargetKind::Targets,
            a if a == self.utf8 => TargetKind::Utf8,
            a if a == self.string => TargetKind::String,
            a if a == self.text => TargetKind::Text,
            a if a == self.plain_utf8 => TargetKind::PlainUtf8,
            a if a == self.png => TargetKind::Png,
            _ => TargetKind::Other,
        }
    }

    /// The atom of a kind (`0` for `Other`).
    pub(super) fn atom_of(&self, kind: TargetKind) -> u64 {
        match kind {
            TargetKind::Targets => self.targets,
            TargetKind::Utf8 | TargetKind::Text => self.utf8,
            TargetKind::String => self.string,
            TargetKind::PlainUtf8 => self.plain_utf8,
            TargetKind::Png => self.png,
            TargetKind::Other => 0,
        }
    }
}
/// Key press and release, button press and release, pointer motion, focus.
const INPUT_MASK: c_long = 1 | 1 << 1 | 1 << 2 | 1 << 3 | 1 << 6 | 1 << 21;

/// Set by the error handler: an X request failed.
static X_ERROR: AtomicBool = AtomicBool::new(false);

#[repr(C)]
struct SetWindowAttributes {
    background_pixmap: c_ulong,
    background_pixel: c_ulong,
    border_pixmap: c_ulong,
    border_pixel: c_ulong,
    bit_gravity: c_int,
    win_gravity: c_int,
    backing_store: c_int,
    backing_planes: c_ulong,
    backing_pixel: c_ulong,
    save_under: c_int,
    event_mask: c_long,
    do_not_propagate_mask: c_long,
    override_redirect: c_int,
    colormap: c_ulong,
    cursor: c_ulong,
}

#[repr(C)]
struct ClassHint {
    res_name: *mut c_char,
    res_class: *mut c_char,
}

#[repr(C)]
struct VisualInfo {
    visual: Ptr,
    visual_id: c_ulong,
    screen: c_int,
    depth: c_int,
    class: c_int,
    red_mask: c_ulong,
    green_mask: c_ulong,
    blue_mask: c_ulong,
    colormap_size: c_int,
    bits_per_rgb: c_int,
}

#[repr(C)]
struct ErrorEvent {
    kind: c_int,
    display: Ptr,
    resource_id: c_ulong,
    serial: c_ulong,
    error_code: u8,
    request_code: u8,
    minor_code: u8,
}

type ErrorHandler = extern "C" fn(Ptr, *mut ErrorEvent) -> c_int;

/// Records that a request failed instead of letting Xlib exit the process.
extern "C" fn on_error(_display: Ptr, _event: *mut ErrorEvent) -> c_int {
    X_ERROR.store(true, Ordering::SeqCst);
    0
}

/// The Xlib functions the window part calls.
#[derive(Debug)]
struct XlibApi {
    // TEST: x11_window_has_title_and_class
    // (the `unsafe` of these function types: calling them is the unsafe act)
    open_display: unsafe extern "C" fn(*const c_char) -> Ptr,
    close_display: unsafe extern "C" fn(Ptr) -> c_int,
    default_screen: unsafe extern "C" fn(Ptr) -> c_int,
    root_window: unsafe extern "C" fn(Ptr, c_int) -> c_ulong,
    default_visual: unsafe extern "C" fn(Ptr, c_int) -> Ptr,
    default_depth: unsafe extern "C" fn(Ptr, c_int) -> c_int,
    // TEST: x11_window_has_title_and_class
    create_colormap: unsafe extern "C" fn(Ptr, c_ulong, Ptr, c_int) -> c_ulong,
    free_colormap: unsafe extern "C" fn(Ptr, c_ulong) -> c_int,
    #[allow(clippy::type_complexity)]
    create_window: unsafe extern "C" fn(
        Ptr,
        c_ulong,
        c_int,
        c_int,
        c_uint,
        c_uint,
        c_uint,
        c_int,
        c_uint,
        Ptr,
        c_ulong,
        *mut SetWindowAttributes,
    ) -> c_ulong,
    // TEST: x11_window_has_title_and_class
    destroy_window: unsafe extern "C" fn(Ptr, c_ulong) -> c_int,
    map_window: unsafe extern "C" fn(Ptr, c_ulong) -> c_int,
    store_name: unsafe extern "C" fn(Ptr, c_ulong, *const c_char) -> c_int,
    set_class_hint: unsafe extern "C" fn(Ptr, c_ulong, *mut ClassHint) -> c_int,
    intern_atom: unsafe extern "C" fn(Ptr, *const c_char, c_int) -> c_ulong,
    set_wm_protocols: unsafe extern "C" fn(Ptr, c_ulong, *mut c_ulong, c_int) -> c_int,
    // TEST: x11_window_has_title_and_class
    pending: unsafe extern "C" fn(Ptr) -> c_int,
    next_event: unsafe extern "C" fn(Ptr, *mut c_long) -> c_int,
    flush: unsafe extern "C" fn(Ptr) -> c_int,
    sync: unsafe extern "C" fn(Ptr, c_int) -> c_int,
    resource_manager_string: unsafe extern "C" fn(Ptr) -> *const c_char,
    display_width: unsafe extern "C" fn(Ptr, c_int) -> c_int,
    // TEST: x11_window_has_title_and_class
    display_width_mm: unsafe extern "C" fn(Ptr, c_int) -> c_int,
    send_event: unsafe extern "C" fn(Ptr, c_ulong, c_int, c_long, *mut c_long) -> c_int,
    set_error_handler: unsafe extern "C" fn(ErrorHandler) -> *const c_void,
    free: unsafe extern "C" fn(Ptr) -> c_int,
    // TEST: x11_key_by_xdotool_reports_key_and_char
    keycode_to_keysym: unsafe extern "C" fn(Ptr, c_uint, c_uint, c_uint) -> c_ulong,
    set_detectable: unsafe extern "C" fn(Ptr, c_int, *mut c_int) -> c_int,
    get_event_data: unsafe extern "C" fn(Ptr, *mut c_long) -> c_int,
    free_event_data: unsafe extern "C" fn(Ptr, *mut c_long),
    query_extension:
        unsafe extern "C" fn(Ptr, *const c_char, *mut c_int, *mut c_int, *mut c_int) -> c_int,
    // TEST: x11_clipboard_text_is_served_to_xclip
    set_selection_owner: unsafe extern "C" fn(Ptr, c_ulong, c_ulong, c_ulong) -> c_int,
    get_selection_owner: unsafe extern "C" fn(Ptr, c_ulong) -> c_ulong,
    convert_selection:
        unsafe extern "C" fn(Ptr, c_ulong, c_ulong, c_ulong, c_ulong, c_ulong) -> c_int,
    change_property: unsafe extern "C" fn(
        Ptr,
        c_ulong,
        c_ulong,
        c_ulong,
        c_int,
        c_int,
        *const u8,
        c_int,
    ) -> c_int,
    #[allow(clippy::type_complexity)]
    // TEST: x11_reads_large_png_by_incr
    get_window_property: unsafe extern "C" fn(
        Ptr,
        c_ulong,
        c_ulong,
        c_long,
        c_long,
        c_int,
        c_ulong,
        *mut c_ulong,
        *mut c_int,
        *mut c_ulong,
        *mut c_ulong,
        *mut *mut u8,
    ) -> c_int,
    // TEST: x11_reads_text_set_by_xclip
    delete_property: unsafe extern "C" fn(Ptr, c_ulong, c_ulong) -> c_int,
    // TEST: x11_large_png_is_served_with_incr
    select_input: unsafe extern "C" fn(Ptr, c_ulong, c_long) -> c_int,
    // TEST: x11_platform_schedule_update_ends_the_wait
    connection_number: unsafe extern "C" fn(Ptr) -> c_int,
    // Declared last: the library outlives the pointers above.
    _libx11: Library,
}

/// `XIEventMask`.
#[repr(C)]
struct XiEventMask {
    deviceid: c_int,
    mask_len: c_int,
    mask: *mut u8,
}

/// `XIAnyClassInfo`.
#[repr(C)]
struct XiAnyClass {
    kind: c_int,
    sourceid: c_int,
}

/// `XIValuatorClassInfo`.
#[repr(C)]
struct XiValuatorClass {
    kind: c_int,
    sourceid: c_int,
    number: c_int,
    label: c_ulong,
    min: f64,
    max: f64,
    value: f64,
    resolution: c_int,
    mode: c_int,
}

/// `XIDeviceInfo`.
#[repr(C)]
struct XiDeviceInfo {
    deviceid: c_int,
    name: *mut c_char,
    use_: c_int,
    attachment: c_int,
    enabled: c_int,
    num_classes: c_int,
    classes: *mut *mut XiAnyClass,
}

/// The head of `XIRawEvent` up to its valuator state.
#[repr(C)]
struct XiRawHead {
    kind: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: Ptr,
    extension: c_int,
    evtype: c_int,
    time: c_ulong,
    deviceid: c_int,
    sourceid: c_int,
    detail: c_int,
    flags: c_int,
    mask_len: c_int,
    mask: *const u8,
    values: *const f64,
}

const XI_RAW_MOTION: i32 = 17;
const XI_VALUATOR_CLASS: c_int = 2;
const XI_ALL_MASTER_DEVICES: c_int = 1;

/// The `libXi` functions, present when `XInput2` 2.0 is.
#[derive(Debug)]
struct XiApi {
    // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
    query_version: unsafe extern "C" fn(Ptr, *mut c_int, *mut c_int) -> c_int,
    // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
    select_events: unsafe extern "C" fn(Ptr, c_ulong, *mut XiEventMask, c_int) -> c_int,
    // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
    query_device: unsafe extern "C" fn(Ptr, c_int, *mut c_int) -> *mut XiDeviceInfo,
    // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
    free_device_info: unsafe extern "C" fn(*mut XiDeviceInfo),
    opcode: c_int,
    _libxi: Library,
}

/// One `XI_RawMotion`, copied out of the display's event data.
#[derive(Debug)]
pub(super) struct XiRaw {
    /// The source device.
    pub(super) source: i32,
    /// The valuator mask.
    pub(super) mask: Vec<u8>,
    /// The packed valuator values.
    pub(super) values: Vec<f64>,
}

/// An event with the `XInput2` data it carried.
#[derive(Debug)]
pub(super) struct Fetched {
    /// The `XEvent` bytes.
    pub(super) bytes: [u8; EVENT_BYTES],
    /// The raw motion, for an `XI_RawMotion` event.
    pub(super) raw: Option<XiRaw>,
}

impl XlibApi {
    fn load() -> Result<Self, X11Error> {
        let x11 = Library::open("libX11.so.6")?;
        // TEST: x11_window_has_title_and_class
        // SAFETY: each field type is the C prototype of the Xlib function
        // named in its lookup; the pointers are copied out and kept next to
        // a second handle to the library, which is dropped last.
        let api = unsafe {
            Self {
                open_display: *x11.symbol("XOpenDisplay")?,
                close_display: *x11.symbol("XCloseDisplay")?,
                default_screen: *x11.symbol("XDefaultScreen")?,
                root_window: *x11.symbol("XRootWindow")?,
                default_visual: *x11.symbol("XDefaultVisual")?,
                default_depth: *x11.symbol("XDefaultDepth")?,
                create_colormap: *x11.symbol("XCreateColormap")?,
                free_colormap: *x11.symbol("XFreeColormap")?,
                create_window: *x11.symbol("XCreateWindow")?,
                destroy_window: *x11.symbol("XDestroyWindow")?,
                map_window: *x11.symbol("XMapWindow")?,
                store_name: *x11.symbol("XStoreName")?,
                set_class_hint: *x11.symbol("XSetClassHint")?,
                intern_atom: *x11.symbol("XInternAtom")?,
                set_wm_protocols: *x11.symbol("XSetWMProtocols")?,
                pending: *x11.symbol("XPending")?,
                next_event: *x11.symbol("XNextEvent")?,
                flush: *x11.symbol("XFlush")?,
                sync: *x11.symbol("XSync")?,
                resource_manager_string: *x11.symbol("XResourceManagerString")?,
                display_width: *x11.symbol("XDisplayWidth")?,
                display_width_mm: *x11.symbol("XDisplayWidthMM")?,
                send_event: *x11.symbol("XSendEvent")?,
                set_error_handler: *x11.symbol("XSetErrorHandler")?,
                free: *x11.symbol("XFree")?,
                keycode_to_keysym: *x11.symbol("XkbKeycodeToKeysym")?,
                set_detectable: *x11.symbol("XkbSetDetectableAutoRepeat")?,
                get_event_data: *x11.symbol("XGetEventData")?,
                free_event_data: *x11.symbol("XFreeEventData")?,
                query_extension: *x11.symbol("XQueryExtension")?,
                set_selection_owner: *x11.symbol("XSetSelectionOwner")?,
                get_selection_owner: *x11.symbol("XGetSelectionOwner")?,
                convert_selection: *x11.symbol("XConvertSelection")?,
                change_property: *x11.symbol("XChangeProperty")?,
                get_window_property: *x11.symbol("XGetWindowProperty")?,
                delete_property: *x11.symbol("XDeleteProperty")?,
                select_input: *x11.symbol("XSelectInput")?,
                connection_number: *x11.symbol("XConnectionNumber")?,
                _libx11: Library::open("libX11.so.6")?,
            }
        };
        Ok(api)
    }
}

/// An open X display, closed when the last owner drops.
#[derive(Debug)]
pub(super) struct Connection {
    api: XlibApi,
    display: Ptr,
    /// GL libraries kept loaded until after `XCloseDisplay`: they hook the
    /// display's close, and the GL contexts `dlclose` their own handles
    /// before the display goes (this field drops after `Drop::drop`).
    _gl_libraries: Vec<Library>,
    xi: Option<XiApi>,
}

impl XiApi {
    /// Loads `libXi` and negotiates `XInput2` 2.0; `None` if either fails.
    fn open(api: &XlibApi, display: Ptr) -> Option<Self> {
        let lib = Library::open("libXi.so.6").ok()?;
        let mut opcode: c_int = 0;
        let (mut event, mut error) = (0, 0);
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: `display` is live, the name is NUL terminated and the
        // out pointers are valid.
        let known = unsafe {
            (api.query_extension)(
                display,
                c"XInputExtension".as_ptr(),
                &raw mut opcode,
                &raw mut event,
                &raw mut error,
            )
        };
        if known == 0 {
            return None;
        }
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: the types are the libXi prototypes of the names.
        let (query_version, select_events, query_device, free_device_info) = unsafe {
            (
                *lib.symbol("XIQueryVersion").ok()?,
                *lib.symbol("XISelectEvents").ok()?,
                *lib.symbol("XIQueryDevice").ok()?,
                *lib.symbol("XIFreeDeviceInfo").ok()?,
            )
        };
        let xi = Self {
            query_version,
            select_events,
            query_device,
            free_device_info,
            opcode,
            _libxi: lib,
        };
        let (mut major, mut minor) = (2, 0);
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: `display` is live and the version pointers are valid.
        let status = unsafe { (xi.query_version)(display, &raw mut major, &raw mut minor) };
        (status == 0).then_some(xi)
    }
}

impl Connection {
    /// Opens `name` (`None`: `$DISPLAY`) and installs the error handler.
    pub(super) fn open(name: Option<&str>) -> Result<Rc<Self>, X11Error> {
        let name = name
            .map(|n| CString::new(n).map_err(|_| X11Error::NoDisplay))
            .transpose()?;
        let api = XlibApi::load()?;
        // A missing GL library is reported later by the context itself.
        let gl_libraries: Vec<Library> = ["libGL.so.1", "libEGL.so.1"]
            .into_iter()
            .filter_map(|name| Library::open(name).ok())
            .collect();
        // TEST: x11_missing_display_is_an_error_value
        // SAFETY: the name is null (`$DISPLAY`) or a valid NUL-terminated
        // string that lives through the call.
        let display = unsafe { (api.open_display)(name.as_ref().map_or(null(), |n| n.as_ptr())) };
        if display.is_null() {
            return Err(X11Error::NoDisplay);
        }
        // TEST: x11_window_has_title_and_class
        // SAFETY: `on_error` has the handler type and lives forever.
        unsafe { (api.set_error_handler)(on_error) };
        let mut supported: c_int = 0;
        // TEST: x11_held_key_reports_repeat_until_release
        // SAFETY: `display` is live and `supported` is a valid out pointer.
        unsafe { (api.set_detectable)(display, 1, &raw mut supported) };
        let xi = XiApi::open(&api, display);
        Ok(Rc::new(Self {
            api,
            display,
            _gl_libraries: gl_libraries,
            xi,
        }))
    }

    /// Fetches the data of a generic `event` and frees it again; the raw
    /// motion if it is an `XInput2` one.
    fn fetch_generic(&self, event: &mut [c_long; 24]) -> Option<XiRaw> {
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: `event` is the generic event `XNextEvent` just filled.
        let got = unsafe { (self.api.get_event_data)(self.display, event.as_mut_ptr()) };
        if got == 0 {
            return None;
        }
        let mut raw = None;
        if let Some(xi) = &self.xi {
            // The cookie: extension at byte 32, evtype at 36, data at 48.
            let [ext, evtype] = [event[4] as i32, (event[4] >> 32) as i32];
            let data = event[6] as *const XiRawHead;
            if ext == xi.opcode && evtype == XI_RAW_MOTION && !data.is_null() {
                // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
                // SAFETY: Xlib points `data` at an `XIRawEvent` that lives
                // until `XFreeEventData`; its mask holds `mask_len` bytes and
                // its values one `double` per set mask bit.
                raw = unsafe {
                    let head = &*data;
                    let len = usize::try_from(head.mask_len).unwrap_or(0);
                    (!head.mask.is_null() && !head.values.is_null()).then(|| {
                        let mask = std::slice::from_raw_parts(head.mask, len).to_vec();
                        let count: usize = mask.iter().map(|b| b.count_ones() as usize).sum();
                        let values = (0..count)
                            .map(|i| std::ptr::read_unaligned(head.values.add(i)))
                            .collect();
                        XiRaw {
                            source: head.sourceid,
                            mask,
                            values,
                        }
                    })
                };
            }
        }
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: the data was fetched above and is freed once.
        unsafe { (self.api.free_event_data)(self.display, event.as_mut_ptr()) };
        raw
    }

    /// Whether `XInput2` raw events are available.
    pub(super) fn has_xi(&self) -> bool {
        self.xi.is_some()
    }

    /// The root window of the default screen.
    pub(super) fn root(&self) -> u64 {
        // TEST: x11_window_has_title_and_class
        // SAFETY: `display` is live.
        unsafe { (self.api.root_window)(self.display, (self.api.default_screen)(self.display)) }
    }

    /// Asks for `XI_RawMotion` of the master devices on the root window.
    pub(super) fn select_raw_motion(&self) {
        let Some(xi) = &self.xi else { return };
        let mut mask = [0u8; 3];
        mask[(XI_RAW_MOTION / 8) as usize] = 1 << (XI_RAW_MOTION % 8);
        let mut selection = XiEventMask {
            deviceid: XI_ALL_MASTER_DEVICES,
            mask_len: 3,
            mask: mask.as_mut_ptr(),
        };
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: `display` is live; the selection and its mask live
        // through the call, which copies them.
        unsafe {
            (xi.select_events)(self.display, self.root(), &raw mut selection, 1);
            (self.api.flush)(self.display);
        }
    }

    /// The keysym of `keycode` at `group` and `level`, `0` if none.
    pub(super) fn keysym(&self, keycode: u32, group: u32, level: u32) -> u32 {
        // TEST: x11_key_by_xdotool_reports_key_and_char
        // SAFETY: `display` is live; Xlib returns `NoSymbol` (0) for any
        // keycode, group or level it does not know.
        let sym = unsafe { (self.api.keycode_to_keysym)(self.display, keycode, group, level) };
        u32::try_from(sym).unwrap_or(0)
    }

    /// The `Abs Pressure` axis of `device`, `None` if it is not a pen.
    pub(super) fn pressure_axis(&self, device: i32) -> Option<PressureAxis> {
        let xi = self.xi.as_ref()?;
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: `display` is live and the name is NUL terminated.
        let label = unsafe { (self.api.intern_atom)(self.display, c"Abs Pressure".as_ptr(), 1) };
        if label == 0 {
            return None;
        }
        let mut count: c_int = 0;
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: `display` is live and `count` is a valid out pointer.
        let info = unsafe { (xi.query_device)(self.display, device, &raw mut count) };
        if info.is_null() {
            return None;
        }
        let mut found = None;
        for d in 0..usize::try_from(count).unwrap_or(0) {
            // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
            // SAFETY: `info` holds `count` device infos, each with
            // `num_classes` class pointers; a class of the valuator kind is
            // an `XIValuatorClassInfo`. They stay valid until the free below.
            unsafe {
                let device_info = &*info.add(d);
                for c in 0..usize::try_from(device_info.num_classes).unwrap_or(0) {
                    let class = *device_info.classes.add(c);
                    if class.is_null() || (*class).kind != XI_VALUATOR_CLASS {
                        continue;
                    }
                    // Libxi lays the class structs out in one malloc'd
                    // buffer with natural alignment.
                    #[allow(clippy::cast_ptr_alignment)]
                    let valuator = &*class.cast::<XiValuatorClass>();
                    if valuator.label == label {
                        found = Some(PressureAxis {
                            number: valuator.number,
                            min: valuator.min,
                            max: valuator.max,
                        });
                    }
                }
            }
        }
        // TEST: x11_xi2_raw_motion_from_xtest_is_not_a_pen
        // SAFETY: `info` came from `XIQueryDevice` and is freed once.
        unsafe { (xi.free_device_info)(info) };
        found
    }

    /// The raw `Display*`, valid while `self` lives.
    pub(super) fn raw_display(&self) -> *mut c_void {
        self.display
    }

    /// The X resource string, the width in pixels and in millimetres of the
    /// default screen.
    pub(super) fn screen_info(&self) -> (Option<String>, i32, i32) {
        // TEST: x11_window_has_title_and_class
        // SAFETY: `display` is live; the returned string, if any, is a
        // NUL-terminated string owned by Xlib that is copied at once.
        unsafe {
            let screen = (self.api.default_screen)(self.display);
            let resources = (self.api.resource_manager_string)(self.display);
            let text = (!resources.is_null())
                .then(|| CStr::from_ptr(resources).to_string_lossy().into_owned());
            (
                text,
                (self.api.display_width)(self.display, screen),
                (self.api.display_width_mm)(self.display, screen),
            )
        }
    }

    fn atom(&self, name: &CStr) -> c_ulong {
        // TEST: x11_window_has_title_and_class
        // SAFETY: `display` is live and `name` is NUL terminated.
        unsafe { (self.api.intern_atom)(self.display, name.as_ptr(), 0) }
    }

    /// Sends `window` the `WM_DELETE_WINDOW` message a window manager sends
    /// when the user closes it.
    pub(super) fn send_delete_window(&self, window: u64) -> Result<(), X11Error> {
        let protocols = self.atom(c"WM_PROTOCOLS");
        let delete = self.atom(c"WM_DELETE_WINDOW");
        let mut bytes = [0u8; EVENT_BYTES];
        bytes[0..4].copy_from_slice(&33i32.to_ne_bytes());
        bytes[32..40].copy_from_slice(&window.to_ne_bytes());
        bytes[40..48].copy_from_slice(&protocols.to_ne_bytes());
        bytes[48..52].copy_from_slice(&32i32.to_ne_bytes());
        bytes[56..64].copy_from_slice(&delete.to_ne_bytes());
        let mut event = [0 as c_long; 24];
        for (long, chunk) in event.iter_mut().zip(bytes.as_chunks::<8>().0) {
            *long = c_long::from_ne_bytes(*chunk);
        }
        // TEST: x11_wm_close_sets_close_requested
        // SAFETY: `display` is live and `event` is a full-size
        // `XClientMessageEvent` buffer that lives through the call.
        let sent = unsafe {
            let status = (self.api.send_event)(self.display, window, 0, 0, event.as_mut_ptr());
            (self.api.flush)(self.display);
            status
        };
        if sent == 0 || self.failed() {
            return Err(X11Error::Call("XSendEvent"));
        }
        Ok(())
    }

    /// The atoms the selections use.
    pub(super) fn sel_atoms(&self) -> SelAtoms {
        SelAtoms {
            clipboard: self.atom(c"CLIPBOARD"),
            primary: self.atom(c"PRIMARY"),
            targets: self.atom(c"TARGETS"),
            utf8: self.atom(c"UTF8_STRING"),
            string: self.atom(c"STRING"),
            text: self.atom(c"TEXT"),
            plain_utf8: self.atom(c"text/plain;charset=utf-8"),
            png: self.atom(c"image/png"),
            incr: self.atom(c"INCR"),
            atom: self.atom(c"ATOM"),
            property: self.atom(c"_SCRATCH_SELECTION"),
        }
    }

    /// Takes `selection` for `window` at the current time; whether the
    /// server made `window` the owner.
    pub(super) fn own_selection(&self, window: u64, selection: u64) -> bool {
        // TEST: x11_clipboard_text_is_served_to_xclip
        // SAFETY: `display` is live; the atoms and the window are plain ids.
        unsafe {
            (self.api.set_selection_owner)(self.display, selection, window, 0);
            (self.api.get_selection_owner)(self.display, selection) == window
        }
    }

    /// Asks the owner of `selection` for `target`, to be written to
    /// `property` of `window`.
    pub(super) fn convert(&self, selection: u64, target: u64, property: u64, window: u64) {
        // TEST: x11_reads_text_set_by_xclip
        // SAFETY: `display` is live; the arguments are plain ids.
        unsafe {
            (self.api.convert_selection)(self.display, selection, target, property, window, 0);
            (self.api.flush)(self.display);
        }
    }

    /// Writes `data` to `property` of `window` as `ty` with 8-bit items.
    pub(super) fn write_bytes(&self, window: u64, property: u64, ty: u64, data: &[u8]) {
        let len = c_int::try_from(data.len()).unwrap_or(c_int::MAX);
        // TEST: x11_png_is_served_as_image_png
        // SAFETY: `display` is live; `data` holds at least `len` bytes and
        // is only read during the call.
        unsafe {
            (self.api.change_property)(
                self.display,
                window,
                property,
                ty,
                8,
                0,
                data.as_ptr(),
                len,
            );
            (self.api.flush)(self.display);
        }
    }

    /// Writes `values` to `property` of `window` as `ty` with 32-bit items.
    pub(super) fn write_longs(&self, window: u64, property: u64, ty: u64, values: &[u64]) {
        let longs: Vec<c_long> = values.iter().map(|v| v.cast_signed()).collect();
        let len = c_int::try_from(longs.len()).unwrap_or(0);
        // TEST: x11_targets_list_text_targets
        // SAFETY: `display` is live; for format 32 Xlib reads `len` C
        // `long`s from the pointer, which `longs` provides.
        unsafe {
            (self.api.change_property)(
                self.display,
                window,
                property,
                ty,
                32,
                0,
                longs.as_ptr().cast::<u8>(),
                len,
            );
            (self.api.flush)(self.display);
        }
    }

    /// Reads (and with `delete`, removes) `property` of `window`; `None` if
    /// it does not exist or holds more than [`MAX_INCOMING`] bytes.
    pub(super) fn read_property(
        &self,
        window: u64,
        property: u64,
        delete: bool,
    ) -> Option<Property> {
        let (mut ty, mut format, mut items, mut after) = (0 as c_ulong, 0 as c_int, 0, 0);
        let mut data: *mut u8 = std::ptr::null_mut();
        let words = c_long::try_from(MAX_INCOMING / 4).unwrap_or(c_long::MAX);
        // TEST: x11_reads_large_png_by_incr
        // SAFETY: `display` is live; every out pointer is valid for the call.
        let status = unsafe {
            (self.api.get_window_property)(
                self.display,
                window,
                property,
                0,
                words,
                c_int::from(delete),
                0,
                &raw mut ty,
                &raw mut format,
                &raw mut items,
                &raw mut after,
                &raw mut data,
            )
        };
        if status != 0 || data.is_null() {
            return None;
        }
        let unit = match format {
            8 => 1,
            16 => 2,
            32 => size_of::<c_long>(),
            _ => 0,
        };
        let len = usize::try_from(items).unwrap_or(0).saturating_mul(unit);
        // TEST: x11_reads_large_png_by_incr
        // SAFETY: Xlib returned `data` with `items` items of `unit` bytes
        // (plus a NUL); the bytes are copied and the buffer freed once.
        let bytes = unsafe {
            let bytes = std::slice::from_raw_parts(data, len).to_vec();
            (self.api.free)(data.cast());
            bytes
        };
        (ty != 0 && after == 0).then_some(Property { ty, data: bytes })
    }

    /// Sends the `SelectionNotify` that answers `request` (`property` `0`
    /// refuses).
    pub(super) fn send_selection_notify(&self, request: &Request, property: u64) {
        let mut bytes = [0u8; EVENT_BYTES];
        bytes[0..4].copy_from_slice(&31i32.to_ne_bytes());
        bytes[32..40].copy_from_slice(&request.requestor.to_ne_bytes());
        bytes[40..48].copy_from_slice(&request.selection.to_ne_bytes());
        bytes[48..56].copy_from_slice(&request.target.to_ne_bytes());
        bytes[56..64].copy_from_slice(&property.to_ne_bytes());
        bytes[64..72].copy_from_slice(&request.time.to_ne_bytes());
        let mut event = [0 as c_long; 24];
        for (long, chunk) in event.iter_mut().zip(bytes.as_chunks::<8>().0) {
            *long = c_long::from_ne_bytes(*chunk);
        }
        // TEST: x11_clipboard_text_is_served_to_xclip
        // SAFETY: `display` is live and `event` is a full-size
        // `XSelectionEvent` buffer that lives through the call.
        unsafe {
            (self.api.send_event)(self.display, request.requestor, 0, 0, event.as_mut_ptr());
            (self.api.flush)(self.display);
        }
    }

    /// Asks for the property events of a foreign `window` (for `INCR`).
    pub(super) fn watch_property(&self, window: u64) {
        // TEST: x11_large_png_is_served_with_incr
        // SAFETY: `display` is live; `window` is an id (a dead window is an
        // X error the handler records).
        unsafe {
            (self.api.select_input)(self.display, window, PROPERTY_CHANGE_MASK);
            (self.api.flush)(self.display);
        }
    }

    /// Removes `property` of `window`.
    pub(super) fn delete_property(&self, window: u64, property: u64) {
        // TEST: x11_reads_text_set_by_xclip
        // SAFETY: `display` is live; the arguments are plain ids.
        unsafe {
            (self.api.delete_property)(self.display, window, property);
        }
    }

    /// Waits for the requests so far and reports whether one failed.
    fn failed(&self) -> bool {
        // TEST: x11_window_has_title_and_class
        // SAFETY: `display` is live.
        unsafe { (self.api.sync)(self.display, 0) };
        X_ERROR.swap(false, Ordering::SeqCst)
    }

    /// The visual and depth of the first GLX config `GlxContext::from_window`
    /// will choose (same attributes, same order), or the default ones.
    fn glx_visual(&self, screen: c_int) -> Result<(Ptr, c_int), X11Error> {
        // TEST: x11_glx_context_clears_and_swaps
        type Choose = unsafe extern "C" fn(Ptr, c_int, *const c_int, *mut c_int) -> *mut Ptr;
        type VisualOf = unsafe extern "C" fn(Ptr, Ptr) -> *mut VisualInfo;
        let gl = Library::open("libGL.so.1")?;
        let wanted = crate::glx::WINDOW_CONFIG;
        let mut count: c_int = 0;
        // TEST: x11_glx_context_clears_and_swaps
        // SAFETY: the types are the GLX 1.3 prototypes of the names.
        let (choose, visual_of) = unsafe {
            (
                *gl.symbol::<Choose>("glXChooseFBConfig")?,
                *gl.symbol::<VisualOf>("glXGetVisualFromFBConfig")?,
            )
        };
        // TEST: x11_glx_context_clears_and_swaps
        // SAFETY: `display` is live, `wanted` is zero terminated and `count`
        // is a valid out pointer; the config array and the visual info are
        // read once and freed with `XFree`.
        unsafe {
            let configs = choose(self.display, screen, wanted.as_ptr(), &raw mut count);
            if configs.is_null() || count < 1 {
                return Err(X11Error::Call("glXChooseFBConfig"));
            }
            let info = visual_of(self.display, *configs);
            (self.api.free)(configs.cast());
            if info.is_null() {
                return Err(X11Error::Call("glXGetVisualFromFBConfig"));
            }
            let found = ((*info).visual, (*info).depth);
            (self.api.free)(info.cast());
            Ok(found)
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // TEST: x11_window_has_title_and_class
        // SAFETY: the display was opened by `open` and is closed once.
        let _ = unsafe { (self.api.close_display)(self.display) };
    }
}

/// The window exists but is not shown.
#[derive(Debug)]
pub(super) struct Created;

/// The window is mapped: events can be read.
#[derive(Debug)]
pub(super) struct Mapped;

/// Which visual a window gets.
#[derive(Debug, Clone, Copy)]
pub(super) enum VisualChoice {
    /// The visual of the GLX config the GL context will pick.
    Glx,
    /// The screen's default visual.
    Default,
}

/// The X resources of a window, released when dropped.
#[derive(Debug)]
struct Handle {
    conn: Rc<Connection>,
    id: c_ulong,
    colormap: c_ulong,
}

impl Drop for Handle {
    fn drop(&mut self) {
        // TEST: x11_window_has_title_and_class
        // SAFETY: the window and colormap were made on this live display and
        // are released once.
        unsafe {
            (self.conn.api.destroy_window)(self.conn.display, self.id);
            (self.conn.api.free_colormap)(self.conn.display, self.colormap);
            (self.conn.api.flush)(self.conn.display);
        }
    }
}

/// A window in the state `S`.
///
/// Only a [`Mapped`] window reads events: `next_event` is not defined for
/// a [`Created`] one.
#[derive(Debug)]
pub(super) struct Window<S> {
    handle: Handle,
    atoms: CloseAtoms,
    state: PhantomData<S>,
}

impl Window<Created> {
    /// Creates (unmapped) a window with title, class and the delete
    /// protocol.
    pub(super) fn create(
        conn: &Rc<Connection>,
        title: &str,
        class: &str,
        size: (u32, u32),
        visual: VisualChoice,
    ) -> Result<Self, X11Error> {
        let api = &conn.api;
        let display = conn.display;
        let title = c_text(title);
        let class = c_text(class);
        // TEST: x11_window_has_title_and_class
        // SAFETY: `display` is live.
        let (screen, root) = unsafe {
            let screen = (api.default_screen)(display);
            (screen, (api.root_window)(display, screen))
        };
        let (visual, depth) = match visual {
            VisualChoice::Glx => conn.glx_visual(screen)?,
            // TEST: x11_egl_fallback_swaps_on_the_window
            // SAFETY: `display` is live and `screen` is its default screen.
            VisualChoice::Default => unsafe {
                (
                    (api.default_visual)(display, screen),
                    (api.default_depth)(display, screen),
                )
            },
        };
        // TEST: x11_window_has_title_and_class
        // SAFETY: `display`, `root` and `visual` are live and match; the
        // attribute struct and the strings live through the calls.
        let (id, colormap) = unsafe {
            let colormap = (api.create_colormap)(display, root, visual, 0);
            let mut attributes = SetWindowAttributes {
                background_pixmap: 0,
                background_pixel: 0,
                border_pixmap: 0,
                border_pixel: 0,
                bit_gravity: 0,
                win_gravity: 0,
                backing_store: 0,
                backing_planes: 0,
                backing_pixel: 0,
                save_under: 0,
                event_mask: EXPOSURE_MASK
                    | VISIBILITY_CHANGE_MASK
                    | STRUCTURE_NOTIFY_MASK
                    | INPUT_MASK
                    | PROPERTY_CHANGE_MASK,
                do_not_propagate_mask: 0,
                override_redirect: 0,
                colormap,
                cursor: 0,
            };
            let id = (api.create_window)(
                display,
                root,
                0,
                0,
                size.0.max(1),
                size.1.max(1),
                0,
                depth,
                INPUT_OUTPUT,
                visual,
                CW_BORDER_PIXEL | CW_EVENT_MASK | CW_COLORMAP,
                &raw mut attributes,
            );
            (id, colormap)
        };
        let handle = Handle {
            conn: Rc::clone(conn),
            id,
            colormap,
        };
        if id == 0 || conn.failed() {
            return Err(X11Error::Call("XCreateWindow"));
        }
        let protocols = conn.atom(c"WM_PROTOCOLS");
        let mut delete = conn.atom(c"WM_DELETE_WINDOW");
        let mut hint = ClassHint {
            res_name: class.as_ptr().cast_mut(),
            res_class: class.as_ptr().cast_mut(),
        };
        // TEST: x11_window_has_title_and_class
        // SAFETY: the window is live; the strings, the hint and the atom
        // array live through the calls and are not written by Xlib.
        unsafe {
            (api.store_name)(display, id, title.as_ptr());
            (api.set_class_hint)(display, id, &raw mut hint);
            (api.set_wm_protocols)(display, id, &raw mut delete, 1);
        }
        Ok(Self {
            handle,
            atoms: CloseAtoms {
                protocols,
                delete_window: delete,
            },
            state: PhantomData,
        })
    }

    /// Maps the window.
    pub(super) fn map(self) -> Window<Mapped> {
        let conn = &self.handle.conn;
        // TEST: x11_window_has_title_and_class
        // SAFETY: the window is live on this display.
        unsafe {
            (conn.api.map_window)(conn.display, self.handle.id);
            (conn.api.flush)(conn.display);
        }
        Window {
            handle: self.handle,
            atoms: self.atoms,
            state: PhantomData,
        }
    }
}

impl<S> Window<S> {
    /// The window's XID.
    pub(super) fn id(&self) -> u64 {
        self.handle.id
    }

    /// The atoms of a close request.
    pub(super) fn close_atoms(&self) -> CloseAtoms {
        self.atoms
    }
}

impl Window<Mapped> {
    /// The display connection of the window.
    pub(super) fn conn(&self) -> &Connection {
        &self.handle.conn
    }

    /// Whether an event is waiting.
    pub(super) fn has_pending(&self) -> bool {
        let conn = &self.handle.conn;
        // TEST: x11_platform_schedule_update_ends_the_wait
        // SAFETY: the display is live.
        unsafe { (conn.api.pending)(conn.display) > 0 }
    }

    /// Flushes queued X requests and waits in `poll` until an X event
    /// arrives, `wake_fd` becomes readable, or `timeout_ms` elapses (`-1`
    /// waits indefinitely).
    pub(super) fn wait_events(&self, timeout_ms: i32, wake_fd: Option<i32>) {
        let conn = &self.handle.conn;
        // TEST: x11_platform_schedule_update_ends_the_wait
        // SAFETY: the display is live.
        unsafe { (conn.api.flush)(conn.display) };
        if self.has_pending() {
            return;
        }
        // TEST: x11_platform_schedule_update_ends_the_wait
        // SAFETY: the display is live.
        let x_fd = unsafe { (conn.api.connection_number)(conn.display) };
        let mut fds = [
            PollFd {
                fd: x_fd,
                events: POLLIN,
                revents: 0,
            },
            PollFd {
                fd: wake_fd.unwrap_or(-1),
                events: POLLIN,
                revents: 0,
            },
        ];
        let nfds = if wake_fd.is_some() { 2 } else { 1 };
        // TEST: x11_platform_schedule_update_ends_the_wait
        // SAFETY: `fds[..nfds]` is valid and lives through the call.
        let _ = unsafe { poll(fds.as_mut_ptr(), nfds, timeout_ms) };
    }

    /// The next waiting event as raw bytes, `None` if there is none.
    pub(super) fn next_event(&self) -> Option<Fetched> {
        if !self.has_pending() {
            return None;
        }
        let conn = &self.handle.conn;
        let mut event = [0 as c_long; 24];
        // TEST: x11_platform_poll_reports_the_resize
        // SAFETY: `XEvent` is a union of 24 `long`s at most; the buffer has
        // that size and alignment, and an event is pending.
        unsafe { (conn.api.next_event)(conn.display, event.as_mut_ptr()) };
        let mut bytes = [0u8; EVENT_BYTES];
        for (chunk, long) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(event) {
            *chunk = long.to_ne_bytes();
        }
        let raw = if matches!(decode(&bytes), XEvent::Generic { .. }) {
            conn.fetch_generic(&mut event)
        } else {
            None
        };
        Some(Fetched { bytes, raw })
    }
}

/// The GL context of `window_id`, a window of `conn`.
///
/// The caller keeps `conn` alive (through the window's `Rc`) until the
/// context is dropped: `X11Platform` declares the context before the window.
pub(super) fn open_context(
    conn: &Connection,
    window_id: u64,
    backend: GlBackend,
) -> Result<GlSurface, X11Error> {
    let display = conn.raw_display();
    Ok(match backend {
        // TEST: x11_glx_context_clears_and_swaps
        // SAFETY: `display` is open and outlives the context (see above) and
        // `window_id` is a window of it.
        GlBackend::Glx => GlSurface::Glx(unsafe { GlxContext::from_window(display, window_id)? }),
        GlBackend::Egl => {
            let mut native = window_id;
            // TEST: x11_egl_fallback_swaps_on_the_window
            // SAFETY: as above; EGL reads the XID through the pointer during
            // the call only.
            GlSurface::Egl(unsafe {
                EglContext::from_window(
                    EglPlatform::X11,
                    display,
                    (&raw mut native).cast::<c_void>(),
                )?
            })
        }
    })
}

/// A C string from `text`, dropping interior NUL bytes.
fn c_text(text: &str) -> CString {
    let clean: Vec<u8> = text.bytes().filter(|b| *b != 0).collect();
    CString::new(clean).unwrap_or_default()
}
