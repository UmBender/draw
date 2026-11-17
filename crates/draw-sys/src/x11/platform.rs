//! [`X11Platform`]: the window part of `Platform` over Xlib (T175 AC-3).
//!
//! `poll` reports resizes, keys, pointer, wheel, focus and pen samples
//! (T176); selections come with T177 and until then the clipboard is empty.

use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::ffi::c_void;
use std::future::{Future, ready};
use std::rc::Rc;
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
use super::selection::{
    Answer, INCR_THRESHOLD, IncrReceiver, IncrSender, IncrStep, Payload, Request, SelEvent,
    decode_selection,
};
use super::signal;
use super::xlib::{
    Connection, Created, Fetched, Mapped, SelAtoms, VisualChoice, Window, open_context,
};
use super::{GlBackend, X11Error};

/// How long a read of another client's selection waits for each step.
const READ_TIMEOUT: Duration = Duration::from_secs(1);
/// The most `INCR` sends in flight at once.
const MAX_OUTGOING: usize = 8;

/// A selection transfer draw is sending with `INCR`.
#[derive(Debug)]
struct Outgoing {
    requestor: u64,
    property: u64,
    ty: u64,
    sender: IncrSender,
}

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

/// A cheap clone of the GL context of an [`X11Platform`], for a `Gpu` to
/// own (ADR-T178-1). The platform keeps swapping the buffers.
#[derive(Debug, Clone)]
pub struct SharedGl {
    surface: Rc<GlSurface>,
    size: Rc<Cell<(u32, u32)>>,
}

impl SharedGl {
    /// The current size of the window in pixels.
    #[must_use]
    pub fn window_size(&self) -> (u32, u32) {
        self.size.get()
    }
}

impl GlContext for SharedGl {
    fn make_current(&self) -> Result<(), ContextError> {
        self.surface.make_current()
    }

    fn swap_buffers(&self) -> Result<(), ContextError> {
        self.surface.swap_buffers()
    }

    fn proc_address(&self, name: &str) -> Option<*const c_void> {
        self.surface.proc_address(name)
    }
}

/// An X11 window with its GL context.
///
/// Field order is drop order: the context goes before the window, the
/// window before the display connection it holds.
/// Whether the window is currently hidden by unmap or full obscuration.
#[derive(Debug, Clone, Copy)]
struct Visibility {
    mapped: bool,
    obscured: bool,
}

impl Visibility {
    fn paused(self) -> bool {
        !self.mapped || self.obscured
    }
}

/// An X11 window with its GL context.
///
/// Field order is drop order: the context goes before the window, the
/// window before the display connection it holds.
#[derive(Debug)]
pub struct X11Platform {
    gl: Rc<GlSurface>,
    window: Window<Mapped>,
    size: Rc<Cell<(u32, u32)>>,
    scale: f32,
    close: bool,
    visibility: Visibility,
    update_scheduled: bool,
    keys: KeyTracker,
    pen: PenTracker,
    axes: HashMap<i32, Option<PressureAxis>>,
    atoms: SelAtoms,
    owned: HashMap<u64, Payload>,
    outgoing: Vec<Outgoing>,
    stash: VecDeque<Fetched>,
    signal: Option<i32>,
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
        let atoms = conn.sel_atoms();
        Ok(Self {
            atoms,
            owned: HashMap::new(),
            outgoing: Vec::new(),
            stash: VecDeque::new(),
            signal: None,
            keys: KeyTracker::default(),
            pen: PenTracker::default(),
            axes: HashMap::new(),
            gl: Rc::new(gl),
            window,
            size: Rc::new(Cell::new((width.max(1), height.max(1)))),
            scale: scale_from_dpi(dpi),
            close: false,
            visibility: Visibility {
                mapped: true,
                obscured: false,
            },
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

    /// A handle on the GL context for a `Gpu` to own. The platform keeps
    /// swapping the buffers in `next_frame`; the Gpu must not.
    ///
    /// Every handle must be dropped before the platform: the GL window
    /// goes before the X window (ADR-T178-1).
    #[must_use]
    pub fn share_gl(&self) -> SharedGl {
        SharedGl {
            surface: Rc::clone(&self.gl),
            size: Rc::clone(&self.size),
        }
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
        self.own(self.atoms.primary, Payload::Text(text.to_owned()));
    }

    /// The text of the `PRIMARY` selection, `None` when empty.
    pub fn primary_text(&mut self) -> Option<String> {
        self.read_text(self.atoms.primary)
    }

    /// The PNG on the clipboard, `None` when there is none.
    pub fn clipboard_png(&mut self) -> Option<Vec<u8>> {
        let selection = self.atoms.clipboard;
        if let Some(payload) = self.owned.get(&selection) {
            return match payload {
                Payload::Png(png) => Some(png.clone()),
                Payload::Text(_) => None,
            };
        }
        self.fetch(selection, self.atoms.png)
    }

    /// The quit signal (`SIGINT`, `SIGTERM`) that closed the loop, if any.
    #[must_use]
    pub fn quit_signal(&self) -> Option<i32> {
        self.signal
    }

    /// Takes `selection` and serves `payload` from it.
    fn own(&mut self, selection: u64, payload: Payload) {
        if self
            .window
            .conn()
            .own_selection(self.window.id(), selection)
        {
            self.owned.insert(selection, payload);
        } else {
            self.owned.remove(&selection);
        }
    }

    fn read_text(&mut self, selection: u64) -> Option<String> {
        if let Some(payload) = self.owned.get(&selection) {
            return match payload {
                Payload::Text(text) => Some(text.clone()),
                Payload::Png(_) => None,
            };
        }
        let bytes = self.fetch(selection, self.atoms.utf8)?;
        String::from_utf8(bytes).ok()
    }

    /// The next event: a stashed one first.
    fn next_fetched(&mut self) -> Option<Fetched> {
        self.stash.pop_front().or_else(|| self.window.next_event())
    }

    /// Asks the owner of `selection` for `target` and waits up to
    /// [`READ_TIMEOUT`] per step. Events that are not the answer are
    /// stashed for the next `poll`.
    fn fetch(&mut self, selection: u64, target: u64) -> Option<Vec<u8>> {
        let (window, property) = (self.window.id(), self.atoms.property);
        let conn = self.window.conn();
        conn.delete_property(window, property);
        conn.convert(selection, target, property, window);
        let mut deadline = Instant::now() + READ_TIMEOUT;
        let mut incr: Option<IncrReceiver> = None;
        let mut kept = VecDeque::new();
        let result = loop {
            let Some(fetched) = self.window.next_event() else {
                if Instant::now() >= deadline {
                    break None;
                }
                std::thread::sleep(Duration::from_millis(1));
                continue;
            };
            match decode_selection(&fetched.bytes) {
                Some(SelEvent::Notify {
                    selection: s,
                    property: p,
                    ..
                }) if incr.is_none() && s == selection => {
                    if p == 0 {
                        break None;
                    }
                    let Some(prop) = self.window.conn().read_property(window, p, true) else {
                        break None;
                    };
                    if prop.ty == self.atoms.incr {
                        incr = Some(IncrReceiver::new());
                        deadline = Instant::now() + READ_TIMEOUT;
                    } else {
                        break Some(prop.data);
                    }
                }
                Some(SelEvent::Property {
                    window: w,
                    atom,
                    deleted: false,
                }) if w == window && atom == property && incr.is_some() => {
                    let Some(prop) = self.window.conn().read_property(window, property, true)
                    else {
                        continue;
                    };
                    deadline = Instant::now() + READ_TIMEOUT;
                    let step = incr.as_mut().map(|r| r.push(&prop.data));
                    match step {
                        Some(IncrStep::More) => {}
                        Some(IncrStep::Done) => break incr.take().map(IncrReceiver::finish),
                        Some(IncrStep::Overflow) | None => break None,
                    }
                }
                _ => kept.push_back(fetched),
            }
        };
        kept.append(&mut self.stash);
        self.stash = kept;
        result
    }

    /// Answers one selection event.
    fn selection_event(&mut self, event: SelEvent) {
        match event {
            SelEvent::Request(request) => self.serve(&request),
            SelEvent::Clear { selection } => {
                self.owned.remove(&selection);
            }
            SelEvent::Property {
                window,
                atom,
                deleted: true,
            } => self.next_incr_chunk(window, atom),
            SelEvent::Property { .. } | SelEvent::Notify { .. } => {}
        }
    }

    fn serve(&mut self, request: &Request) {
        let conn = self.window.conn();
        let property = if request.property == 0 {
            request.target
        } else {
            request.property
        };
        let kind = self.atoms.kind(request.target);
        let answer = self
            .owned
            .get(&request.selection)
            .map_or(Answer::Refuse, |payload| payload.answer(kind));
        let granted = match answer {
            Answer::Targets(kinds) => {
                let list: Vec<u64> = kinds.into_iter().map(|k| self.atoms.atom_of(k)).collect();
                conn.write_longs(request.requestor, property, self.atoms.atom, &list);
                property
            }
            Answer::Data(bytes, kind) if bytes.len() > INCR_THRESHOLD => {
                conn.watch_property(request.requestor);
                conn.write_longs(
                    request.requestor,
                    property,
                    self.atoms.incr,
                    &[bytes.len() as u64],
                );
                if self.outgoing.len() >= MAX_OUTGOING {
                    self.outgoing.remove(0);
                }
                self.outgoing.push(Outgoing {
                    requestor: request.requestor,
                    property,
                    ty: self.atoms.atom_of(kind),
                    sender: IncrSender::new(bytes),
                });
                property
            }
            Answer::Data(bytes, kind) => {
                conn.write_bytes(
                    request.requestor,
                    property,
                    self.atoms.atom_of(kind),
                    &bytes,
                );
                property
            }
            Answer::Refuse => 0,
        };
        conn.send_selection_notify(request, granted);
    }

    /// The requestor deleted `property`: write the next `INCR` chunk.
    fn next_incr_chunk(&mut self, window: u64, property: u64) {
        let Some(at) = self
            .outgoing
            .iter()
            .position(|o| o.requestor == window && o.property == property)
        else {
            return;
        };
        let entry = &mut self.outgoing[at];
        let (ty, requestor) = (entry.ty, entry.requestor);
        match entry.sender.next_chunk() {
            Some(chunk) => {
                let last = chunk.is_empty();
                self.window
                    .conn()
                    .write_bytes(requestor, property, ty, chunk);
                if last {
                    self.outgoing.remove(at);
                }
            }
            None => {
                self.outgoing.remove(at);
            }
        }
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
        if let Some(number) = signal::take() {
            self.signal = Some(number);
            self.close = true;
        }
        while let Some(fetched) = self.next_fetched() {
            if let Some(selection) = decode_selection(&fetched.bytes) {
                self.selection_event(selection);
                continue;
            }
            let event = decode(&fetched.bytes);
            if is_close(event, atoms) {
                self.close = true;
                continue;
            }
            match event {
                XEvent::Configure { width, height }
                    if width > 0 && height > 0 && (width, height) != self.size.get() =>
                {
                    self.size.set((width, height));
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
                XEvent::Mapped(mapped) => {
                    self.visibility.mapped = mapped;
                }
                XEvent::Visibility { obscured } => {
                    self.visibility.obscured = obscured;
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
        let (width, height) = self.size.get();
        window_metrics(width, height, self.scale)
    }

    fn clipboard_text(&mut self) -> Option<String> {
        self.read_text(self.atoms.clipboard)
    }

    fn set_clipboard_text(&mut self, text: &str) {
        self.own(self.atoms.clipboard, Payload::Text(text.to_owned()));
    }

    fn set_clipboard_png(&mut self, png: &[u8]) {
        self.own(self.atoms.clipboard, Payload::Png(png.to_vec()));
    }

    fn schedule_update(&mut self) {
        self.update_scheduled = true;
    }

    fn skips_idle_swap(&self) -> bool {
        true
    }

    fn paused(&self) -> bool {
        self.visibility.paused()
    }

    fn next_frame(&mut self) -> impl Future<Output = ()> {
        // A failed swap (a lost window) is seen by the next `poll`.
        let _ = self.gl.swap_buffers();
        if !self.update_scheduled
            && self.stash.is_empty()
            && !self.window.has_pending()
            && signal::peek().is_none()
        {
            let ms = i32::try_from(FRAME_WAIT.as_millis()).unwrap_or(16);
            self.window.wait_events(ms, signal::wake_fd());
        }
        self.update_scheduled = false;
        ready(())
    }

    fn next_idle(&mut self) -> impl Future<Output = ()> {
        if !std::mem::take(&mut self.update_scheduled)
            && self.stash.is_empty()
            && self.outgoing.is_empty()
            && !self.window.has_pending()
            && signal::peek().is_none()
        {
            self.window.wait_events(-1, signal::wake_fd());
        }
        ready(())
    }
}
