//! [`X11Platform`]: the window part of `Platform` over Xlib (T175 AC-3).
//!
//! `poll` reports resizes, keys, pointer, wheel, focus and pen samples
//! (T176); selections come with T177 and until then the clipboard is empty.

use std::collections::HashMap;
use std::ffi::c_void;
use std::future::{Future, ready};
use std::time::{Duration, Instant};

use crate::egl::{ContextError, EglContext, GlContext};
use crate::glx::GlxContext;
use crate::platform::{Platform, RawEvent, WindowMetrics};

use super::dpi::{resolve_dpi, scale_from_dpi, window_metrics};
use super::event::{XEvent, decode, is_close};
use super::keys::{
    KeyTracker, button_from_x, char_from_keysym, group_of, key_from_keysym, level_for,
    mods_from_state, wheel_delta,
};
use super::pen::{PenTracker, PressureAxis};
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
    keys: KeyTracker,
    pen: PenTracker,
    axes: HashMap<i32, Option<PressureAxis>>,
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
        conn.select_raw_motion();
        Ok(Self {
            keys: KeyTracker::default(),
            pen: PenTracker::default(),
            axes: HashMap::new(),
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

    /// Whether pen pressure is available (`libXi` with `XInput2` 2.0).
    #[must_use]
    pub fn pen_supported(&self) -> bool {
        self.window.conn().has_xi()
    }

    fn key(&mut self, out: &mut Vec<RawEvent>, down: bool, keycode: u32, state: u32) {
        let conn = self.window.conn();
        let group = group_of(state);
        let base = conn.keysym(keycode, group, 0);
        let mods = mods_from_state(state);
        let repeat = if down {
            self.keys.press(keycode)
        } else {
            self.keys.release(keycode);
            false
        };
        out.push(RawEvent::Key {
            key: key_from_keysym(base, keycode),
            mods,
            down,
            repeat,
        });
        if down && !mods.ctrl && !mods.alt {
            let alphabetic = char_from_keysym(base).is_some_and(char::is_alphabetic);
            let level = level_for(state, alphabetic);
            let mut sym = conn.keysym(keycode, group, level);
            if sym == 0 {
                sym = base;
            }
            if let Some(ch) = char_from_keysym(sym) {
                out.push(RawEvent::Char { ch, mods, repeat });
            }
        }
    }

    fn button(&mut self, out: &mut Vec<RawEvent>, down: bool, button: u32, x: i32, y: i32) {
        if let Some((dx, dy)) = wheel_delta(button) {
            if down {
                out.push(RawEvent::Wheel { dx, dy });
            }
            return;
        }
        let (x, y) = (x as f32, y as f32);
        out.extend(self.pen.sample(x, y));
        out.push(RawEvent::Button {
            button: button_from_x(button),
            x,
            y,
            down,
        });
    }

    /// Puts `text` on the `PRIMARY` selection.
    pub fn set_primary_text(&mut self, text: &str) {
        let _ = text;
        todo!()
    }

    /// The text of the `PRIMARY` selection, `None` when empty.
    pub fn primary_text(&mut self) -> Option<String> {
        todo!()
    }

    /// The PNG on the clipboard, `None` when there is none.
    pub fn clipboard_png(&mut self) -> Option<Vec<u8>> {
        todo!()
    }

    /// The quit signal (`SIGINT`, `SIGTERM`) that closed the loop, if any.
    #[must_use]
    pub fn quit_signal(&self) -> Option<i32> {
        todo!()
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
        while let Some(fetched) = self.window.next_event() {
            let event = decode(&fetched.bytes);
            if is_close(event, atoms) {
                self.close = true;
                continue;
            }
            match event {
                XEvent::Configure { width, height }
                    if width > 0 && height > 0 && (width, height) != self.size =>
                {
                    self.size = (width, height);
                    out.push(RawEvent::Resize(window_metrics(width, height, self.scale)));
                }
                XEvent::Key {
                    down,
                    keycode,
                    state,
                } => self.key(out, down, keycode, state),
                XEvent::Button { down, button, x, y } => self.button(out, down, button, x, y),
                XEvent::Motion { x, y } => {
                    let (x, y) = (x as f32, y as f32);
                    out.extend(self.pen.sample(x, y));
                    out.push(RawEvent::PointerMove { x, y });
                }
                XEvent::Focus(focus) => {
                    if !focus {
                        self.keys.clear();
                    }
                    out.push(RawEvent::Focus(focus));
                }
                XEvent::Generic { .. } => {
                    if let Some(raw) = fetched.raw {
                        let conn = self.window.conn();
                        let axis = *self
                            .axes
                            .entry(raw.source)
                            .or_insert_with(|| conn.pressure_axis(raw.source));
                        self.pen.raw_event(raw.source, axis, &raw.mask, &raw.values);
                    }
                }
                XEvent::Configure { .. } | XEvent::ClientMessage { .. } | XEvent::Other(_) => {}
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
