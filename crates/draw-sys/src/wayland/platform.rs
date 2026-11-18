//! [`WaylandPlatform`]: the window part of [`Platform`] over a Wayland
//! connection (T168). Keyboard, pointer and seats are T169, tablet and
//! clipboard T170: until then those parts return nothing.

use std::cell::Cell;
use std::ffi::c_void;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::egl::{ContextError, EglContext, GlContext};
use crate::platform::{Platform, RawEvent, WindowMetrics};
use crate::wayland::connection::{Connection, WaylandError};
use crate::wayland::transfer::Payload;
use crate::wayland::window::{Mapped, Window};

/// How long `next_frame` waits for a frame callback before it gives up
/// (a hidden window gets none).
const FRAME_WAIT: Duration = Duration::from_millis(500);

/// Key-repeat polling step in `next_idle` while a key is held down.
const REPEAT_WAIT_MS: i32 = 10;

/// A cheap clone of the GL context of a [`WaylandPlatform`], for a `Gpu` to
/// own (ADR-T171-1). The platform keeps swapping the buffers.
#[derive(Debug, Clone)]
pub struct SharedGl {
    context: Rc<EglContext>,
    size: Rc<Cell<(u32, u32)>>,
}

impl SharedGl {
    /// The current size of the window in physical pixels.
    #[must_use]
    pub fn window_size(&self) -> (u32, u32) {
        self.size.get()
    }
}

impl GlContext for SharedGl {
    fn make_current(&self) -> Result<(), ContextError> {
        self.context.make_current()
    }

    fn swap_buffers(&self) -> Result<(), ContextError> {
        self.context.swap_buffers()
    }

    fn proc_address(&self, name: &str) -> Option<*const c_void> {
        self.context.proc_address(name)
    }
}

/// A Wayland window with a GL context, as a [`Platform`].
#[derive(Debug)]
pub struct WaylandPlatform {
    // Declared first: the window is dropped before the connection.
    window: Window<Mapped>,
    connection: Connection,
    reported: WindowMetrics,
    size: Rc<Cell<(u32, u32)>>,
    update_scheduled: bool,
}

impl WaylandPlatform {
    /// Connects, opens a window with a title and an application id, and
    /// shows it with a first present. `socket` is the compositor socket, or
    /// the environment's when `None`.
    ///
    /// # Errors
    /// [`WaylandError`] from the connection, the window or the context.
    pub fn connect(socket: Option<&Path>, title: &str, app_id: &str) -> Result<Self, WaylandError> {
        Self::connect_with(socket, title, app_id, 4)
    }

    /// Connects like [`WaylandPlatform::connect`], requesting `samples`
    /// MSAA on the window surface (`0` or `1` disables multisampling).
    ///
    /// # Errors
    /// [`WaylandError`] from the connection, the window or the context.
    pub fn connect_with(
        socket: Option<&Path>,
        title: &str,
        app_id: &str,
        samples: i32,
    ) -> Result<Self, WaylandError> {
        let connection = Connection::connect(socket)?;
        let window = connection
            .create_window(title, app_id)?
            .wait_configure()?
            .open_gl_samples(samples)?;
        window.swap().map_err(WaylandError::Context)?;
        connection.shared.roundtrip()?;
        window.apply_configure();
        let reported = metrics(&window);
        let mut platform = Self {
            size: Rc::new(Cell::new(pixels(reported))),
            reported,
            window,
            connection,
            update_scheduled: false,
        };
        platform.pump();
        platform.reported = metrics(&platform.window);
        platform.size.set(pixels(platform.reported));
        Ok(platform)
    }

    /// A handle on the GL context for a `Gpu` to own. The platform keeps
    /// swapping the buffers in `next_frame`; the Gpu must not.
    ///
    /// Every handle must be dropped before the platform: the GL window
    /// goes before the native window (ADR-T171-1).
    #[must_use]
    pub fn share_gl(&self) -> SharedGl {
        SharedGl {
            context: self.window.context_handle(),
            size: Rc::clone(&self.size),
        }
    }

    /// Whether the compositor asked the window to close.
    #[must_use]
    pub fn close_requested(&self) -> bool {
        self.window.close_requested()
    }

    /// Whether the compositor's tablet manager is bound.
    #[must_use]
    pub fn has_tablet_manager(&self) -> bool {
        !self.connection.shared.input.tablet.manager.get().is_null()
    }

    /// Whether a frame callback is outstanding.
    #[must_use]
    pub fn frame_pending(&self) -> bool {
        self.window.frame_pending()
    }

    /// The GL context of the window.
    #[must_use]
    pub fn context(&self) -> &EglContext {
        self.window.context()
    }

    /// Handles what the compositor has sent, without waiting.
    fn pump(&mut self) {
        // A broken connection also shows as `close_requested` staying false
        // and no more events; T171 decides how `main` reports it.
        let _ = self.connection.dispatch(Duration::ZERO);
        self.window.apply_configure();
        self.size.set(pixels(metrics(&self.window)));
    }
}

/// The size of the window in physical pixels (logical size × scale).
fn pixels(metrics: WindowMetrics) -> (u32, u32) {
    let px = |v: f32| (v * metrics.scale).round().max(1.0) as u32;
    (px(metrics.width), px(metrics.height))
}

fn metrics(window: &Window<Mapped>) -> WindowMetrics {
    let (width, height) = window.size();
    WindowMetrics {
        width: width as f32,
        height: height as f32,
        scale: window.scale() as f32,
    }
}

impl Platform for WaylandPlatform {
    fn poll(&mut self, out: &mut Vec<RawEvent>) {
        self.pump();
        let input = &self.connection.shared.input;
        input.poll_repeats(Instant::now());
        input.drain(out);
        let now = metrics(&self.window);
        if now != self.reported {
            self.reported = now;
            out.push(RawEvent::Resize(now));
        }
    }

    fn window(&mut self) -> WindowMetrics {
        self.pump();
        metrics(&self.window)
    }

    fn clipboard_text(&mut self) -> Option<String> {
        self.pump();
        let shared = &self.connection.shared;
        shared.input.clipboard.text(shared)
    }

    fn set_clipboard_text(&mut self, text: &str) {
        let shared = &self.connection.shared;
        shared
            .input
            .clipboard
            .set(shared, Payload::Text(text.to_owned()));
    }

    fn set_clipboard_png(&mut self, png: &[u8]) {
        let shared = &self.connection.shared;
        shared
            .input
            .clipboard
            .set(shared, Payload::Png(png.to_vec()));
    }

    fn schedule_update(&mut self) {
        self.update_scheduled = true;
        self.window.request_frame();
        let _ = self.connection.shared.dispatch(0);
    }

    fn skips_idle_swap(&self) -> bool {
        true
    }

    fn paused(&self) -> bool {
        self.window.is_suspended()
    }

    async fn next_frame(&mut self) {
        self.update_scheduled = false;
        self.window.request_frame();
        // A failed present leaves the old frame on screen; the next frame
        // tries again.
        let _ = self.window.swap();
        let window = &self.window;
        let _ = self
            .connection
            .shared
            .wait_until(|| !window.frame_pending(), FRAME_WAIT);
        self.pump();
    }

    fn next_idle(&mut self) -> impl std::future::Future<Output = ()> {
        if std::mem::take(&mut self.update_scheduled)
            || self.window.close_requested()
            || self.connection.shared.input.has_events()
            || crate::signal::peek().is_some()
        {
            return std::future::ready(());
        }
        let was_paused = self.paused();
        loop {
            let timeout_ms = if self.connection.shared.input.has_held_key() {
                REPEAT_WAIT_MS
            } else {
                -1
            };
            let wake_fd = crate::signal::wake_fd();
            if self
                .connection
                .shared
                .dispatch_with_wake(timeout_ms, wake_fd)
                .is_err()
            {
                break;
            }
            self.window.apply_configure();
            self.size.set(pixels(metrics(&self.window)));
            let input = &self.connection.shared.input;
            if input.has_events()
                || input.has_held_key()
                || self.window.close_requested()
                || self.paused() != was_paused
                || metrics(&self.window) != self.reported
                || crate::signal::peek().is_some()
            {
                break;
            }
        }
        std::future::ready(())
    }
}
