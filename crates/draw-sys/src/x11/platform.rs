//! [`X11Platform`]: the window part of `Platform` over Xlib (T175 AC-3).

use std::future::Future;

use crate::egl::{ContextError, GlContext};
use crate::platform::{Platform, RawEvent, WindowMetrics};

use super::{GlBackend, X11Error};

/// The GL context of an X11 window, GLX or EGL.
#[derive(Debug)]
pub struct GlSurface {}

impl GlContext for GlSurface {
    fn make_current(&self) -> Result<(), ContextError> {
        todo!()
    }

    fn swap_buffers(&self) -> Result<(), ContextError> {
        todo!()
    }

    fn proc_address(&self, name: &str) -> Option<*const std::ffi::c_void> {
        todo!("{name}")
    }
}

/// An X11 window with its GL context.
#[derive(Debug)]
pub struct X11Platform {}

impl X11Platform {
    /// Opens `display` (`None`: `$DISPLAY`) and a mapped window of
    /// `width × height` physical pixels with `title` and WM class `class`.
    ///
    /// # Errors
    /// [`X11Error`] if the display, a library or the context fails.
    pub fn open(
        display: Option<&str>,
        title: &str,
        class: &str,
        width: u32,
        height: u32,
        backend: GlBackend,
    ) -> Result<Self, X11Error> {
        todo!("{display:?} {title} {class} {width} {height} {backend:?}")
    }

    /// The window's XID.
    #[must_use]
    pub fn window_id(&self) -> u64 {
        todo!()
    }

    /// The GL context of the window.
    #[must_use]
    pub fn gl(&self) -> &GlSurface {
        todo!()
    }

    /// Whether the window manager asked to close the window.
    #[must_use]
    pub fn close_requested(&self) -> bool {
        todo!()
    }
}

impl Platform for X11Platform {
    fn poll(&mut self, out: &mut Vec<RawEvent>) {
        todo!("{out:?}")
    }

    fn window(&mut self) -> WindowMetrics {
        todo!()
    }

    fn clipboard_text(&mut self) -> Option<String> {
        todo!()
    }

    fn set_clipboard_text(&mut self, text: &str) {
        todo!("{text}")
    }

    fn set_clipboard_png(&mut self, png: &[u8]) {
        todo!("{png:?}")
    }

    fn schedule_update(&mut self) {
        todo!()
    }

    fn next_frame(&mut self) -> impl Future<Output = ()> {
        async { todo!() }
    }
}
