//! GLX as the fallback for X11 without EGL (ADR-T145-1): an OpenGL ES 3.0
//! context through `GLX_EXT_create_context_es2_profile`, on a pbuffer or on
//! a window.

#![allow(unsafe_code)]

use std::ffi::{CString, c_char, c_int, c_ulong, c_void};
use std::ptr::{null, null_mut};

use crate::egl::{ContextError, GlContext, check_size};
use crate::loader::{Library, LoadError};

type Ptr = *mut c_void;

const GLX_RED_SIZE: c_int = 8;
const GLX_GREEN_SIZE: c_int = 9;
const GLX_BLUE_SIZE: c_int = 10;
const GLX_ALPHA_SIZE: c_int = 11;
const GLX_DRAWABLE_TYPE: c_int = 0x8010;
const GLX_RENDER_TYPE: c_int = 0x8011;
const GLX_RGBA_BIT: c_int = 1;
const GLX_WINDOW_BIT: c_int = 1;
const GLX_PBUFFER_BIT: c_int = 4;
const GLX_PBUFFER_WIDTH: c_int = 0x8041;
const GLX_PBUFFER_HEIGHT: c_int = 0x8040;
const GLX_CONTEXT_MAJOR_VERSION_ARB: c_int = 0x2091;
const GLX_CONTEXT_MINOR_VERSION_ARB: c_int = 0x2092;
const GLX_CONTEXT_PROFILE_MASK_ARB: c_int = 0x9126;
const GLX_CONTEXT_ES2_PROFILE_BIT_EXT: c_int = 0x4;

type CreateContextAttribs = unsafe extern "C" fn(Ptr, Ptr, Ptr, c_int, *const c_int) -> Ptr;

/// The Xlib and GLX functions draw uses, with the libraries that own them.
#[derive(Debug)]
struct GlxApi {
    x_open_display: unsafe extern "C" fn(*const c_char) -> Ptr,
    x_close_display: unsafe extern "C" fn(Ptr) -> c_int,
    x_default_screen: unsafe extern "C" fn(Ptr) -> c_int,
    x_free: unsafe extern "C" fn(Ptr) -> c_int,
    choose_fb_config: unsafe extern "C" fn(Ptr, c_int, *const c_int, *mut c_int) -> *mut Ptr,
    create_pbuffer: unsafe extern "C" fn(Ptr, Ptr, *const c_int) -> c_ulong,
    create_window: unsafe extern "C" fn(Ptr, Ptr, c_ulong, *const c_int) -> c_ulong,
    destroy_pbuffer: unsafe extern "C" fn(Ptr, c_ulong),
    destroy_window: unsafe extern "C" fn(Ptr, c_ulong),
    destroy_context: unsafe extern "C" fn(Ptr, Ptr),
    make_context_current: unsafe extern "C" fn(Ptr, c_ulong, c_ulong, Ptr) -> c_int,
    swap_buffers: unsafe extern "C" fn(Ptr, c_ulong),
    get_proc_address: unsafe extern "C" fn(*const u8) -> *const c_void,
    // Declared last: the libraries outlive the pointers above.
    _libgl: Library,
    _libx11: Library,
}

impl GlxApi {
    fn load() -> Result<Self, LoadError> {
        let x11 = Library::open("libX11.so.6")?;
        let gl = Library::open("libGL.so.1")?;
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: each field type is the C prototype of the Xlib or GLX 1.4
        // function named in its lookup; the pointers are copied out and kept
        // next to the libraries, which are dropped last.
        let api = unsafe {
            Self {
                x_open_display: *x11.symbol("XOpenDisplay")?,
                x_close_display: *x11.symbol("XCloseDisplay")?,
                x_default_screen: *x11.symbol("XDefaultScreen")?,
                x_free: *x11.symbol("XFree")?,
                choose_fb_config: *gl.symbol("glXChooseFBConfig")?,
                create_pbuffer: *gl.symbol("glXCreatePbuffer")?,
                create_window: *gl.symbol("glXCreateWindow")?,
                destroy_pbuffer: *gl.symbol("glXDestroyPbuffer")?,
                destroy_window: *gl.symbol("glXDestroyWindow")?,
                destroy_context: *gl.symbol("glXDestroyContext")?,
                make_context_current: *gl.symbol("glXMakeContextCurrent")?,
                swap_buffers: *gl.symbol("glXSwapBuffers")?,
                get_proc_address: *gl.symbol("glXGetProcAddressARB")?,
                _libgl: Library::open("libGL.so.1")?,
                _libx11: Library::open("libX11.so.6")?,
            }
        };
        Ok(api)
    }
}

/// What the context draws to.
#[derive(Debug, Clone, Copy)]
enum Surface {
    Pbuffer(c_ulong),
    Window(c_ulong),
}

impl Surface {
    fn id(self) -> c_ulong {
        match self {
            Self::Pbuffer(id) | Self::Window(id) => id,
        }
    }
}

/// A GLX context and the surface and display connection it owns.
#[derive(Debug)]
pub struct GlxContext {
    api: GlxApi,
    display: Ptr,
    context: Ptr,
    surface: Surface,
    /// Whether `Drop` closes `display` (a pbuffer context opened it).
    owns_display: bool,
}

impl GlxContext {
    /// A context on a `width × height` pbuffer of the X display
    /// `display_name` (`None`: `$DISPLAY`).
    ///
    /// # Errors
    /// [`ContextError::BadSize`] for a zero or huge size,
    /// [`ContextError::NoDisplay`] if the display cannot be opened, and
    /// [`ContextError`] for any other failure.
    pub fn pbuffer(
        display_name: Option<&str>,
        width: u32,
        height: u32,
    ) -> Result<Self, ContextError> {
        check_size(width, height)?;
        let name = display_name
            .map(|n| CString::new(n).map_err(|_| ContextError::NoDisplay))
            .transpose()?;
        let api = GlxApi::load()?;
        // TEST: glx_bad_display_is_an_error_value
        // SAFETY: the name is null (`$DISPLAY`) or a valid NUL-terminated
        // string that lives through the call.
        let display = unsafe { (api.x_open_display)(name.as_ref().map_or(null(), |n| n.as_ptr())) };
        if display.is_null() {
            return Err(ContextError::NoDisplay);
        }
        let attributes = [
            GLX_PBUFFER_WIDTH,
            width as c_int,
            GLX_PBUFFER_HEIGHT,
            height as c_int,
            0,
        ];
        Self::build(api, display, GLX_PBUFFER_BIT, true, |api, config| {
            // TEST: glx_pbuffer_context_makes_current_under_xvfb
            // SAFETY: `display` and `config` are live and the attribute list
            // is zero terminated.
            let id = unsafe { (api.create_pbuffer)(display, config, attributes.as_ptr()) };
            (id != 0)
                .then_some(Surface::Pbuffer(id))
                .ok_or(ContextError::Call {
                    call: "glXCreatePbuffer",
                    code: 0,
                })
        })
    }

    /// A context on an existing window.
    ///
    /// # Errors
    /// [`ContextError`] if GLX refuses the config, window or context.
    ///
    /// # Safety
    /// `display` must be a live Xlib `Display*` that outlives the context
    /// and `window` a window of it.
    // TEST: glx_pbuffer_context_makes_current_under_xvfb
    pub unsafe fn from_window(display: *mut c_void, window: u64) -> Result<Self, ContextError> {
        if display.is_null() {
            return Err(ContextError::NoDisplay);
        }
        let api = GlxApi::load()?;
        Self::build(api, display, GLX_WINDOW_BIT, false, |api, config| {
            // TEST: glx_pbuffer_context_makes_current_under_xvfb
            // SAFETY: the caller of `from_window` vouches for `display` and
            // `window`; `config` is live.
            let id = unsafe { (api.create_window)(display, config, window, null()) };
            (id != 0)
                .then_some(Surface::Window(id))
                .ok_or(ContextError::Call {
                    call: "glXCreateWindow",
                    code: 0,
                })
        })
    }

    /// Chooses a config of `drawable` type, makes the surface and an
    /// OpenGL ES 3.0 context, and makes it current. On failure the display
    /// is closed if `owns_display`.
    fn build<F>(
        api: GlxApi,
        display: Ptr,
        drawable: c_int,
        owns_display: bool,
        make_surface: F,
    ) -> Result<Self, ContextError>
    where
        F: FnOnce(&GlxApi, Ptr) -> Result<Surface, ContextError>,
    {
        let result = Self::build_inner(&api, display, drawable, make_surface);
        match result {
            Ok((context, surface)) => {
                let made = Self {
                    api,
                    display,
                    context,
                    surface,
                    owns_display,
                };
                made.make_current()?;
                Ok(made)
            }
            Err(error) => {
                if owns_display {
                    // TEST: glx_bad_display_is_an_error_value
                    // SAFETY: the display was opened by us and is closed once.
                    let _ = unsafe { (api.x_close_display)(display) };
                }
                Err(error)
            }
        }
    }

    fn build_inner<F>(
        api: &GlxApi,
        display: Ptr,
        drawable: c_int,
        make_surface: F,
    ) -> Result<(Ptr, Surface), ContextError>
    where
        F: FnOnce(&GlxApi, Ptr) -> Result<Surface, ContextError>,
    {
        let wanted = [
            GLX_DRAWABLE_TYPE,
            drawable,
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
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: `display` is live, `wanted` is zero terminated and `count`
        // is a valid out pointer.
        let configs = unsafe {
            let screen = (api.x_default_screen)(display);
            (api.choose_fb_config)(display, screen, wanted.as_ptr(), &raw mut count)
        };
        if configs.is_null() || count < 1 {
            return Err(ContextError::Unsupported("an RGBA8 GLX config"));
        }
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: `configs` points to `count >= 1` configs; the first is
        // read and the array is freed with `XFree` once.
        let config = unsafe {
            let first = *configs;
            (api.x_free)(configs.cast());
            first
        };
        let context = Self::es_context(api, display, config)?;
        match make_surface(api, config) {
            Ok(surface) => Ok((context, surface)),
            Err(error) => {
                // TEST: glx_pbuffer_context_makes_current_under_xvfb
                // SAFETY: `context` was just created on `display`.
                unsafe { (api.destroy_context)(display, context) };
                Err(error)
            }
        }
    }

    /// An OpenGL ES 3.0 context through the `es2_profile` extension.
    fn es_context(api: &GlxApi, display: Ptr, config: Ptr) -> Result<Ptr, ContextError> {
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: the name is NUL-terminated and static.
        let create =
            unsafe { (api.get_proc_address)(c"glXCreateContextAttribsARB".as_ptr().cast()) };
        if create.is_null() {
            return Err(ContextError::Unsupported("GLX_ARB_create_context"));
        }
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: `glXCreateContextAttribsARB` has the type
        // `CreateContextAttribs` by the extension specification.
        let create = unsafe { std::mem::transmute::<*const c_void, CreateContextAttribs>(create) };
        let attributes = [
            GLX_CONTEXT_MAJOR_VERSION_ARB,
            3,
            GLX_CONTEXT_MINOR_VERSION_ARB,
            0,
            GLX_CONTEXT_PROFILE_MASK_ARB,
            GLX_CONTEXT_ES2_PROFILE_BIT_EXT,
            0,
        ];
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: `display` and `config` are live, nothing is shared, direct
        // rendering is asked for (`1`) and the list is zero terminated.
        let context = unsafe { create(display, config, null_mut(), 1, attributes.as_ptr()) };
        if context.is_null() {
            return Err(ContextError::Unsupported("an OpenGL ES 3.0 GLX context"));
        }
        Ok(context)
    }
}

impl GlContext for GlxContext {
    fn make_current(&self) -> Result<(), ContextError> {
        let id = self.surface.id();
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: the display, surface and context belong to `self`.
        let ok = unsafe { (self.api.make_context_current)(self.display, id, id, self.context) };
        if ok == 0 {
            return Err(ContextError::Call {
                call: "glXMakeContextCurrent",
                code: 0,
            });
        }
        Ok(())
    }

    fn swap_buffers(&self) -> Result<(), ContextError> {
        if let Surface::Window(id) = self.surface {
            // TEST: glx_pbuffer_context_makes_current_under_xvfb
            // SAFETY: the display and window surface belong to `self`.
            unsafe { (self.api.swap_buffers)(self.display, id) };
        }
        Ok(())
    }

    fn proc_address(&self, name: &str) -> Option<*const c_void> {
        let c_name = CString::new(name).ok()?;
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: `c_name` is a valid NUL-terminated string for the call.
        let address = unsafe { (self.api.get_proc_address)(c_name.as_ptr().cast()) };
        (!address.is_null()).then_some(address)
    }
}

impl Drop for GlxContext {
    fn drop(&mut self) {
        // TEST: glx_pbuffer_context_makes_current_under_xvfb
        // SAFETY: the handles belong to `self` and are released once: unbind,
        // surface, context, then the display if it was opened here.
        unsafe {
            let _ = (self.api.make_context_current)(self.display, 0, 0, null_mut());
            match self.surface {
                Surface::Pbuffer(id) => (self.api.destroy_pbuffer)(self.display, id),
                Surface::Window(id) => (self.api.destroy_window)(self.display, id),
            }
            (self.api.destroy_context)(self.display, self.context);
            if self.owns_display {
                let _ = (self.api.x_close_display)(self.display);
            }
        }
    }
}
