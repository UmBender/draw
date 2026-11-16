//! [`WaylandPlatform`]: the window part of [`Platform`] over a Wayland
//! connection (T168). Keyboard, pointer and seats are T169, tablet and
//! clipboard T170: until then those parts return nothing.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::egl::EglContext;
use crate::platform::{Platform, RawEvent, WindowMetrics};
use crate::wayland::connection::{Connection, WaylandError};
use crate::wayland::transfer::Payload;
use crate::wayland::window::{Mapped, Window};

/// How long `next_frame` waits for a frame callback before it gives up
/// (a hidden window gets none).
const FRAME_WAIT: Duration = Duration::from_millis(500);

/// A Wayland window with a GL context, as a [`Platform`].
#[derive(Debug)]
pub struct WaylandPlatform {
    // Declared first: the window is dropped before the connection.
    window: Window<Mapped>,
    connection: Connection,
    reported: WindowMetrics,
}

impl WaylandPlatform {
    /// Connects, opens a window with a title and an application id, and
    /// shows it with a first present. `socket` is the compositor socket, or
    /// the environment's when `None`.
    ///
    /// # Errors
    /// [`WaylandError`] from the connection, the window or the context.
    pub fn connect(socket: Option<&Path>, title: &str, app_id: &str) -> Result<Self, WaylandError> {
        let connection = Connection::connect(socket)?;
        let window = connection
            .create_window(title, app_id)?
            .wait_configure()?
            .open_gl()?;
        window.swap().map_err(WaylandError::Context)?;
        connection.shared.roundtrip()?;
        window.apply_configure();
        let mut platform = Self {
            reported: metrics(&window),
            window,
            connection,
        };
        platform.pump();
        platform.reported = metrics(&platform.window);
        Ok(platform)
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
    }
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
        shared.input.clipboard.set(shared, Payload::Text(text.to_owned()));
    }

    fn set_clipboard_png(&mut self, png: &[u8]) {
        let shared = &self.connection.shared;
        shared.input.clipboard.set(shared, Payload::Png(png.to_vec()));
    }

    fn schedule_update(&mut self) {
        self.window.request_frame();
        let _ = self.connection.shared.dispatch(0);
    }

    async fn next_frame(&mut self) {
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
}
