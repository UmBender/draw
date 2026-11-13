//! GLX as the fallback for X11 without EGL (ADR-T145-1): an OpenGL ES 3.0
//! context through `GLX_EXT_create_context_es2_profile`, on a pbuffer or on
//! a window.

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::ptr::NonNull;

use crate::egl::{ContextError, GlContext};

/// A GLX context and the surface and display connection it owns.
#[derive(Debug)]
pub struct GlxContext {
    _handle: NonNull<c_void>,
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
        let _ = (display_name, width, height);
        todo!("T145: implement GLX pbuffer context")
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
        let _ = (display, window);
        todo!("T145: implement GLX window context")
    }
}

impl GlContext for GlxContext {
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
