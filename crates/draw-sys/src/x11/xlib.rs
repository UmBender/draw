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

use super::event::{CloseAtoms, EVENT_BYTES};
use super::platform::GlSurface;
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
const STRUCTURE_NOTIFY_MASK: c_long = 1 << 17;

const GLX_RED_SIZE: c_int = 8;
const GLX_GREEN_SIZE: c_int = 9;
const GLX_BLUE_SIZE: c_int = 10;
const GLX_ALPHA_SIZE: c_int = 11;
const GLX_DRAWABLE_TYPE: c_int = 0x8010;
const GLX_RENDER_TYPE: c_int = 0x8011;
const GLX_RGBA_BIT: c_int = 1;
const GLX_WINDOW_BIT: c_int = 1;

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
    // Declared last: the library outlives the pointers above.
    _libx11: Library,
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
        Ok(Rc::new(Self {
            api,
            display,
            _gl_libraries: gl_libraries,
        }))
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
        let wanted = [
            GLX_DRAWABLE_TYPE,
            GLX_WINDOW_BIT,
            GLX_RENDER_TYPE,
            GLX_RGBA_BIT,
            GLX_RED_SIZE,
            8,
            GLX_GREEN_SIZE,
            8,
            GLX_BLUE_SIZE,
            8,
            GLX_ALPHA_SIZE,
            8,
            0,
        ];
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
                event_mask: EXPOSURE_MASK | STRUCTURE_NOTIFY_MASK,
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
    /// Whether an event is waiting.
    pub(super) fn has_pending(&self) -> bool {
        let conn = &self.handle.conn;
        // TEST: x11_platform_schedule_update_ends_the_wait
        // SAFETY: the display is live.
        unsafe { (conn.api.pending)(conn.display) > 0 }
    }

    /// The next waiting event as raw bytes, `None` if there is none.
    pub(super) fn next_event(&self) -> Option<[u8; EVENT_BYTES]> {
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
        Some(bytes)
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
