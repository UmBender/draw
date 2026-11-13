//! EGL contexts for OpenGL ES 3.0 (ADR-T145-1): headless (surfaceless, then
//! pbuffer) for tests and on a native window for the backends. Every failure
//! is a [`ContextError`] value.

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::fmt;
use std::ptr::NonNull;

use crate::loader::LoadError;

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

/// An EGL OpenGL ES 3.0 context and the surface it owns.
#[derive(Debug)]
pub struct EglContext {
    _handle: NonNull<c_void>,
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
        let _ = (library, width, height);
        todo!("T145: implement EGL headless context")
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
        let _ = (platform, display, window);
        todo!("T145: implement EGL window context")
    }
}

impl GlContext for EglContext {
    fn make_current(&self) -> Result<(), ContextError> {
        todo!("T145")
    }

    fn swap_buffers(&self) -> Result<(), ContextError> {
        todo!("T145")
    }

    fn proc_address(&self, name: &str) -> Option<*const c_void> {
        let _ = name;
        todo!("T145")
    }
}
