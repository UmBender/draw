//! EGL contexts for OpenGL ES 3.0 (ADR-T145-1): headless (surfaceless, then
//! pbuffer) for tests and on a native window for the backends. Every failure
//! is a [`ContextError`] value.
//!
//! The display is never terminated: EGL shares one display per platform in
//! the process, and `eglTerminate` would pull it from under another context.

#![allow(unsafe_code)]

use std::ffi::{CString, c_char, c_void};
use std::fmt;
use std::ptr::{null, null_mut};

use crate::loader::{Library, LoadError};

/// Why a context could not be created or used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextError {
    /// A needed library or symbol is missing.
    Load(LoadError),
    /// The width or height is zero or too large.
    BadSize,
    /// The display could not be opened or has no EGL/GLX.
    NoDisplay,
    /// The driver lacks something needed, for example OpenGL ES 3.0.
    Unsupported(&'static str),
    /// An EGL or GLX call failed.
    Call {
        /// Name of the call.
        call: &'static str,
        /// The error code it reported (`eglGetError`, or `0` for GLX).
        code: i32,
    },
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(e) => write!(f, "{e}"),
            Self::BadSize => f.write_str("context size is zero or too large"),
            Self::NoDisplay => f.write_str("cannot open the display"),
            Self::Unsupported(what) => write!(f, "not supported: {what}"),
            Self::Call { call, code } => write!(f, "{call} failed with code {code:#x}"),
        }
    }
}

impl std::error::Error for ContextError {}

impl From<LoadError> for ContextError {
    fn from(error: LoadError) -> Self {
        Self::Load(error)
    }
}

/// A GL context that can be made current and swapped; implemented by
/// [`EglContext`] and by the GLX one. It owns its surface.
pub trait GlContext {
    /// Makes the context current on the calling thread.
    ///
    /// # Errors
    /// [`ContextError::Call`] if the driver refuses.
    fn make_current(&self) -> Result<(), ContextError>;

    /// Presents the back buffer (a no-op for a headless context).
    ///
    /// # Errors
    /// [`ContextError::Call`] if the driver refuses.
    fn swap_buffers(&self) -> Result<(), ContextError>;

    /// The address of the GL function `name`, `None` if the driver has none.
    fn proc_address(&self, name: &str) -> Option<*const c_void>;
}

/// The native platform of a window context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// `EGL_KHR_platform_wayland`: the display is a `wl_display*`.
    Wayland,
    /// `EGL_KHR_platform_x11`: the display is an Xlib `Display*`.
    X11,
}

/// Largest side of a pbuffer.
pub(crate) const MAX_SIDE: u32 = 16384;

/// Checks a pbuffer size.
pub(crate) fn check_size(width: u32, height: u32) -> Result<(), ContextError> {
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        Err(ContextError::BadSize)
    } else {
        Ok(())
    }
}

type Ptr = *mut c_void;
type Int = i32;

const EGL_NONE: Int = 0x3038;
const EGL_DONT_CARE: Int = -1;
const EGL_SURFACE_TYPE: Int = 0x3033;
const EGL_RENDERABLE_TYPE: Int = 0x3040;
const EGL_PBUFFER_BIT: Int = 0x1;
const EGL_WINDOW_BIT: Int = 0x4;
const EGL_OPENGL_ES3_BIT: Int = 0x40;
const EGL_RED_SIZE: Int = 0x3022;
const EGL_GREEN_SIZE: Int = 0x3023;
const EGL_BLUE_SIZE: Int = 0x3024;
const EGL_ALPHA_SIZE: Int = 0x3021;
const EGL_WIDTH: Int = 0x3057;
const EGL_HEIGHT: Int = 0x3056;
const EGL_CONTEXT_MAJOR_VERSION: Int = 0x3098;
const EGL_CONTEXT_MINOR_VERSION: Int = 0x30FB;
const EGL_OPENGL_ES_API: u32 = 0x30A0;
const EGL_PLATFORM_WAYLAND_KHR: u32 = 0x31D8;
const EGL_PLATFORM_X11_KHR: u32 = 0x31D5;
const EGL_PLATFORM_SURFACELESS_MESA: u32 = 0x31DD;

/// The EGL functions draw uses, with the library that owns them.
#[derive(Debug)]
struct EglApi {
    get_proc_address: unsafe extern "C" fn(*const c_char) -> *const c_void,
    get_error: unsafe extern "C" fn() -> Int,
    get_platform_display: unsafe extern "C" fn(u32, Ptr, *const isize) -> Ptr,
    get_display: unsafe extern "C" fn(Ptr) -> Ptr,
    initialize: unsafe extern "C" fn(Ptr, *mut Int, *mut Int) -> u32,
    bind_api: unsafe extern "C" fn(u32) -> u32,
    choose_config: unsafe extern "C" fn(Ptr, *const Int, *mut Ptr, Int, *mut Int) -> u32,
    create_context: unsafe extern "C" fn(Ptr, Ptr, Ptr, *const Int) -> Ptr,
    destroy_context: unsafe extern "C" fn(Ptr, Ptr) -> u32,
    create_pbuffer_surface: unsafe extern "C" fn(Ptr, Ptr, *const Int) -> Ptr,
    create_platform_window_surface: unsafe extern "C" fn(Ptr, Ptr, Ptr, *const isize) -> Ptr,
    destroy_surface: unsafe extern "C" fn(Ptr, Ptr) -> u32,
    make_current: unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32,
    swap_buffers: unsafe extern "C" fn(Ptr, Ptr) -> u32,
    // Declared last: the library must outlive the pointers above, and
    // fields drop in order.
    _library: Library,
}

impl EglApi {
    fn load(library: &str) -> Result<Self, ContextError> {
        let lib = Library::open(library)?;
        // TEST: egl_headless_context_makes_current
        // SAFETY: each field type is the C prototype of the EGL 1.5
        // function named in its lookup; the pointers are copied out of the
        // symbols and kept next to the library, which is dropped last.
        let api = unsafe {
            Self {
                get_proc_address: *lib.symbol("eglGetProcAddress")?,
                get_error: *lib.symbol("eglGetError")?,
                get_platform_display: *lib.symbol("eglGetPlatformDisplay")?,
                get_display: *lib.symbol("eglGetDisplay")?,
                initialize: *lib.symbol("eglInitialize")?,
                bind_api: *lib.symbol("eglBindAPI")?,
                choose_config: *lib.symbol("eglChooseConfig")?,
                create_context: *lib.symbol("eglCreateContext")?,
                destroy_context: *lib.symbol("eglDestroyContext")?,
                create_pbuffer_surface: *lib.symbol("eglCreatePbufferSurface")?,
                create_platform_window_surface: *lib.symbol("eglCreatePlatformWindowSurface")?,
                destroy_surface: *lib.symbol("eglDestroySurface")?,
                make_current: *lib.symbol("eglMakeCurrent")?,
                swap_buffers: *lib.symbol("eglSwapBuffers")?,
                _library: Library::open(library)?,
            }
        };
        drop(lib);
        Ok(api)
    }

    /// The error of a failed call.
    fn failed(&self, call: &'static str) -> ContextError {
        // TEST: egl_headless_context_makes_current
        // SAFETY: `eglGetError` takes no arguments and is always callable.
        let code = unsafe { (self.get_error)() };
        ContextError::Call { call, code }
    }

    /// An initialised display, or `None`.
    fn display(&self, platform: u32, native: Ptr) -> Result<Ptr, ContextError> {
        let display = if platform == 0 {
            // TEST: egl_headless_context_makes_current
            // SAFETY: a null native display asks for the default one.
            unsafe { (self.get_display)(native) }
        } else {
            // TEST: egl_headless_context_makes_current
            // SAFETY: `native` is null (default) or a handle the caller of
            // `from_window` vouches for; no attributes are passed.
            unsafe { (self.get_platform_display)(platform, native, null()) }
        };
        if display.is_null() {
            return Err(ContextError::NoDisplay);
        }
        let (mut major, mut minor) = (0, 0);
        // TEST: egl_headless_context_makes_current
        // SAFETY: `display` came from EGL and the two out pointers are
        // valid for the call.
        let ok = unsafe { (self.initialize)(display, &raw mut major, &raw mut minor) };
        if ok == 0 {
            return Err(self.failed("eglInitialize"));
        }
        Ok(display)
    }

    /// Picks an RGBA8 OpenGL ES 3 config for `surface_type`.
    fn config(&self, display: Ptr, surface_type: Int) -> Result<Ptr, ContextError> {
        let attributes = [
            EGL_SURFACE_TYPE,
            surface_type,
            EGL_RENDERABLE_TYPE,
            EGL_OPENGL_ES3_BIT,
            EGL_RED_SIZE,
            8,
            EGL_GREEN_SIZE,
            8,
            EGL_BLUE_SIZE,
            8,
            EGL_ALPHA_SIZE,
            8,
            EGL_NONE,
        ];
        let mut config: Ptr = null_mut();
        let mut count: Int = 0;
        // TEST: egl_headless_context_makes_current
        // SAFETY: `attributes` is `EGL_NONE` terminated, `config` has room
        // for the one config asked for, and `count` is a valid out pointer.
        let ok = unsafe {
            (self.choose_config)(
                display,
                attributes.as_ptr(),
                &raw mut config,
                1,
                &raw mut count,
            )
        };
        if ok == 0 {
            return Err(self.failed("eglChooseConfig"));
        }
        if count < 1 || config.is_null() {
            return Err(ContextError::Unsupported("an RGBA8 OpenGL ES 3 config"));
        }
        Ok(config)
    }

    /// An OpenGL ES 3.0 context for `config`.
    fn context(&self, display: Ptr, config: Ptr) -> Result<Ptr, ContextError> {
        // TEST: egl_headless_context_makes_current
        // SAFETY: `EGL_OPENGL_ES_API` is a valid API.
        if unsafe { (self.bind_api)(EGL_OPENGL_ES_API) } == 0 {
            return Err(self.failed("eglBindAPI"));
        }
        let attributes = [
            EGL_CONTEXT_MAJOR_VERSION,
            3,
            EGL_CONTEXT_MINOR_VERSION,
            0,
            EGL_NONE,
        ];
        // TEST: egl_headless_context_makes_current
        // SAFETY: `display` and `config` came from EGL, no sharing, and the
        // attribute list is `EGL_NONE` terminated.
        let context =
            unsafe { (self.create_context)(display, config, null_mut(), attributes.as_ptr()) };
        if context.is_null() {
            return Err(self.failed("eglCreateContext"));
        }
        Ok(context)
    }
}

/// An EGL OpenGL ES 3.0 context and the surface it owns.
#[derive(Debug)]
pub struct EglContext {
    api: EglApi,
    display: Ptr,
    context: Ptr,
    /// Null for a surfaceless context.
    surface: Ptr,
}

impl EglContext {
    /// A headless context: the Mesa surfaceless platform first, then the
    /// default platform with a `width × height` pbuffer.
    ///
    /// # Errors
    /// [`ContextError`] for a bad size, a missing `libEGL.so.1`, or a driver
    /// without OpenGL ES 3.0.
    pub fn headless(width: u32, height: u32) -> Result<Self, ContextError> {
        Self::headless_with_library("libEGL.so.1", width, height)
    }

    /// Like [`EglContext::headless`] with the EGL library named `library`.
    ///
    /// # Errors
    /// As [`EglContext::headless`]; [`ContextError::Load`] if `library`
    /// cannot be loaded.
    pub fn headless_with_library(
        library: &str,
        width: u32,
        height: u32,
    ) -> Result<Self, ContextError> {
        check_size(width, height)?;
        let surfaceless = Self::build(
            library,
            EGL_PLATFORM_SURFACELESS_MESA,
            null_mut(),
            |_, _| Ok(null_mut()),
        );
        if let Ok(context) = surfaceless {
            return Ok(context);
        }
        Self::build(library, 0, null_mut(), |api, (display, config)| {
            let size = [EGL_WIDTH, width as Int, EGL_HEIGHT, height as Int, EGL_NONE];
            // TEST: egl_headless_context_makes_current
            // SAFETY: `display` and `config` came from EGL and `size` is
            // `EGL_NONE` terminated.
            let surface = unsafe { (api.create_pbuffer_surface)(display, config, size.as_ptr()) };
            if surface.is_null() {
                return Err(api.failed("eglCreatePbufferSurface"));
            }
            Ok(surface)
        })
    }

    /// A context on a native window.
    ///
    /// # Errors
    /// [`ContextError`] if EGL refuses the display, config or surface.
    ///
    /// # Safety
    /// `display` and `window` must be live native handles of `platform`
    /// (a `wl_display*` and `wl_egl_window*`, or an Xlib `Display*` and a
    /// pointer to the window's XID) that outlive the context.
    // TEST: egl_headless_context_makes_current
    pub unsafe fn from_window(
        platform: Platform,
        display: *mut c_void,
        window: *mut c_void,
    ) -> Result<Self, ContextError> {
        let id = match platform {
            Platform::Wayland => EGL_PLATFORM_WAYLAND_KHR,
            Platform::X11 => EGL_PLATFORM_X11_KHR,
        };
        Self::build_with(
            "libEGL.so.1",
            id,
            display,
            EGL_WINDOW_BIT,
            |api, (display, config)| {
                // TEST: egl_headless_context_makes_current
                // SAFETY: the caller of `from_window` vouches for `window`
                // being the native window of `display`'s platform.
                let surface = unsafe {
                    (api.create_platform_window_surface)(display, config, window, null())
                };
                if surface.is_null() {
                    return Err(api.failed("eglCreatePlatformWindowSurface"));
                }
                Ok(surface)
            },
        )
    }

    /// Builds a context on `platform`; `make_surface` gives its surface
    /// (null for none). Pbuffer type is requested when a surface is.
    fn build<F>(
        library: &str,
        platform: u32,
        native: Ptr,
        make_surface: F,
    ) -> Result<Self, ContextError>
    where
        F: FnOnce(&EglApi, (Ptr, Ptr)) -> Result<Ptr, ContextError>,
    {
        let surface_type = if platform == EGL_PLATFORM_SURFACELESS_MESA {
            EGL_DONT_CARE
        } else {
            EGL_PBUFFER_BIT
        };
        Self::build_with(library, platform, native, surface_type, make_surface)
    }

    fn build_with<F>(
        library: &str,
        platform: u32,
        native: Ptr,
        surface_type: Int,
        make_surface: F,
    ) -> Result<Self, ContextError>
    where
        F: FnOnce(&EglApi, (Ptr, Ptr)) -> Result<Ptr, ContextError>,
    {
        let api = EglApi::load(library)?;
        let display = api.display(platform, native)?;
        let config = api.config(display, surface_type)?;
        let context = api.context(display, config)?;
        let surface = match make_surface(&api, (display, config)) {
            Ok(surface) => surface,
            Err(error) => {
                // TEST: egl_headless_context_makes_current
                // SAFETY: `context` was just created on `display`.
                let _ = unsafe { (api.destroy_context)(display, context) };
                return Err(error);
            }
        };
        let made = Self {
            api,
            display,
            context,
            surface,
        };
        // A context that cannot be made current is unusable: `Drop` then
        // releases it and the caller may try another platform.
        made.make_current()?;
        Ok(made)
    }
}

impl GlContext for EglContext {
    fn make_current(&self) -> Result<(), ContextError> {
        // TEST: egl_headless_context_makes_current
        // SAFETY: all three handles belong to `self` and are live; a null
        // surface is allowed for a surfaceless context.
        let ok = unsafe {
            (self.api.make_current)(self.display, self.surface, self.surface, self.context)
        };
        if ok == 0 {
            return Err(self.api.failed("eglMakeCurrent"));
        }
        Ok(())
    }

    fn swap_buffers(&self) -> Result<(), ContextError> {
        if self.surface.is_null() {
            return Ok(());
        }
        // TEST: egl_headless_context_makes_current
        // SAFETY: the display and surface belong to `self` and are live.
        let ok = unsafe { (self.api.swap_buffers)(self.display, self.surface) };
        if ok == 0 {
            return Err(self.api.failed("eglSwapBuffers"));
        }
        Ok(())
    }

    fn proc_address(&self, name: &str) -> Option<*const c_void> {
        let c_name = CString::new(name).ok()?;
        // TEST: egl_context_resolves_gl_entry_points
        // SAFETY: `c_name` is a valid NUL-terminated string for the call.
        let address = unsafe { (self.api.get_proc_address)(c_name.as_ptr()) };
        (!address.is_null()).then_some(address)
    }
}

impl Drop for EglContext {
    fn drop(&mut self) {
        // TEST: egl_headless_context_makes_current
        // SAFETY: the handles belong to `self` and are released once, in this
        // order: unbind, surface, context. Failures are not actionable.
        unsafe {
            let _ = (self.api.make_current)(self.display, null_mut(), null_mut(), null_mut());
            if !self.surface.is_null() {
                let _ = (self.api.destroy_surface)(self.display, self.surface);
            }
            let _ = (self.api.destroy_context)(self.display, self.context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_size_rejects_zero_and_huge() {
        assert_eq!(check_size(0, 1), Err(ContextError::BadSize));
        assert_eq!(check_size(1, 0), Err(ContextError::BadSize));
        assert_eq!(check_size(MAX_SIDE + 1, 1), Err(ContextError::BadSize));
        assert_eq!(check_size(MAX_SIDE, MAX_SIDE), Ok(()));
    }

    #[test]
    fn error_text_names_what_failed() {
        assert!(ContextError::NoDisplay.to_string().contains("display"));
        assert!(
            ContextError::Unsupported("x")
                .to_string()
                .contains("not supported")
        );
    }
}
