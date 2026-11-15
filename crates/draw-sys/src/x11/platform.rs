//! [`X11Platform`]: the window part of `Platform` over Xlib (T175 AC-3).
//!
//! Keyboard, pointer and pen events come with T176, selections with T177:
//! until then `poll` reports only resizes and the clipboard is empty.

use std::ffi::c_void;
use std::future::{Future, ready};
use std::time::{Duration, Instant};

use crate::egl::{ContextError, EglContext, GlContext};
use crate::glx::GlxContext;
use crate::platform::{Platform, RawEvent, WindowMetrics};

use super::dpi::{resolve_dpi, scale_from_dpi, window_metrics};
use super::event::{XEvent, decode, is_close};
use super::xlib::{Connection, Created, Mapped, VisualChoice, Window, open_context};
use super::{GlBackend, X11Error};

/// The longest `next_frame` waits for events when nothing asked for a
/// frame (idle at zero is T147).
const FRAME_WAIT: Duration = Duration::from_millis(16);

/// The GL context of an X11 window, GLX or EGL.
#[derive(Debug)]
pub enum GlSurface {
    /// GLX with the ES 3.0 profile.
    Glx(GlxContext),
    /// EGL on X11.
    Egl(EglContext),
}

impl GlContext for GlSurface {
    fn make_current(&self) -> Result<(), ContextError> {
        match self {
            Self::Glx(c) => c.make_current(),
            Self::Egl(c) => c.make_current(),
        }
    }

    fn swap_buffers(&self) -> Result<(), ContextError> {
        match self {
            Self::Glx(c) => c.swap_buffers(),
            Self::Egl(c) => c.swap_buffers(),
        }
    }

    fn proc_address(&self, name: &str) -> Option<*const c_void> {
        match self {
            Self::Glx(c) => c.proc_address(name),
            Self::Egl(c) => c.proc_address(name),
        }
    }
}

/// An X11 window with its GL context.
///
/// Field order is drop order: the context goes before the window, the
/// window before the display connection it holds.
#[derive(Debug)]
pub struct X11Platform {
    gl: GlSurface,
    window: Window<Mapped>,
    size: (u32, u32),
    scale: f32,
    close: bool,
    update_scheduled: bool,
}

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
        let conn = Connection::open(display)?;
        let (resources, pixels, millimetres) = conn.screen_info();
        let dpi = resolve_dpi(resources.as_deref(), pixels, millimetres);
        let choice = match backend {
            GlBackend::Glx => VisualChoice::Glx,
            GlBackend::Egl => VisualChoice::Default,
        };
        let created: Window<Created> =
            Window::create(&conn, title, class, (width, height), choice)?;
        let gl = open_context(&conn, created.id(), backend)?;
        let window = created.map();
        Ok(Self {
            gl,
            window,
            size: (width.max(1), height.max(1)),
            scale: scale_from_dpi(dpi),
            close: false,
            update_scheduled: false,
        })
    }

    /// The window's XID.
    #[must_use]
    pub fn window_id(&self) -> u64 {
        self.window.id()
    }

    /// The GL context of the window.
    #[must_use]
    pub fn gl(&self) -> &GlSurface {
        &self.gl
    }

    /// Whether the window manager asked to close the window.
    #[must_use]
    pub fn close_requested(&self) -> bool {
        self.close
    }
}

impl Platform for X11Platform {
    fn poll(&mut self, out: &mut Vec<RawEvent>) {
        let atoms = self.window.close_atoms();
        while let Some(raw) = self.window.next_event() {
            let event = decode(&raw);
            if is_close(event, atoms) {
                self.close = true;
            } else if let XEvent::Configure { width, height } = event
                && width > 0
                && height > 0
                && (width, height) != self.size
            {
                self.size = (width, height);
                out.push(RawEvent::Resize(window_metrics(width, height, self.scale)));
            }
        }
    }

    fn window(&mut self) -> WindowMetrics {
        window_metrics(self.size.0, self.size.1, self.scale)
    }

    fn clipboard_text(&mut self) -> Option<String> {
        None
    }

    fn set_clipboard_text(&mut self, _text: &str) {}

    fn set_clipboard_png(&mut self, _png: &[u8]) {}

    fn schedule_update(&mut self) {
        self.update_scheduled = true;
    }

    fn next_frame(&mut self) -> impl Future<Output = ()> {
        // A failed swap (a lost window) is seen by the next `poll`.
        let _ = self.gl.swap_buffers();
        let deadline = Instant::now() + FRAME_WAIT;
        while !self.update_scheduled && !self.window.has_pending() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        self.update_scheduled = false;
        ready(())
    }
}
