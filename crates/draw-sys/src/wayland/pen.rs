//! The pure state of Wayland pen tools (T170, ADR-T170-1): what the
//! `zwp_tablet_tool_v2` events of one `frame` become as
//! [`RawEvent`]s. No compositor and no `unsafe`.

use crate::platform::{RawButton, RawEvent};

/// The largest pressure value of the protocol.
pub const PRESSURE_MAX: f32 = 65535.0;

/// The kind of a tablet tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    /// The pen tip.
    Pen,
    /// The eraser end.
    Eraser,
    /// Any other tool.
    Other,
}

/// The kind a `zwp_tablet_tool_v2.type` value stands for.
#[must_use]
pub fn tool_kind(wire: u32) -> ToolKind {
    let _ = wire;
    todo!()
}

/// A pressure from the wire as `0..=1`.
#[must_use]
pub fn pressure_from_wire(wire: u32) -> f32 {
    let _ = wire;
    todo!()
}

/// The button a barrel button's evdev code stands for.
#[must_use]
pub fn pen_button_from_evdev(code: u32) -> Option<RawButton> {
    let _ = code;
    todo!()
}

/// One tablet tool and the events since its last `frame`.
#[derive(Debug, Clone)]
pub struct PenTool {
    focused: bool,
    position: (f32, f32),
    pressure: f32,
    tip_down: bool,
    tilt: (f32, f32),
    kind: ToolKind,
    motion: Option<(f32, f32)>,
    tip: Option<bool>,
    buttons: Vec<(RawButton, bool)>,
    proximity_out: bool,
}

impl Default for PenTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PenTool {
    /// A tool out of proximity.
    #[must_use]
    pub fn new() -> Self {
        Self {
            focused: false,
            position: (0.0, 0.0),
            pressure: 0.0,
            tip_down: false,
            tilt: (0.0, 0.0),
            kind: ToolKind::Other,
            motion: None,
            tip: None,
            buttons: Vec::new(),
            proximity_out: false,
        }
    }

    /// `proximity_in`; `ours` is whether the surface is this window.
    pub fn proximity_in(&mut self, ours: bool) {
        let _ = ours;
        todo!()
    }

    /// `proximity_out`, applied at the next `frame`.
    pub fn proximity_out(&mut self) {
        todo!()
    }

    /// `down`.
    pub fn down(&mut self) {
        todo!()
    }

    /// `up`.
    pub fn up(&mut self) {
        todo!()
    }

    /// `motion` in `wl_fixed_t` surface coordinates at `scale`.
    pub fn motion(&mut self, x: i32, y: i32, scale: i32) {
        let _ = (x, y, scale);
        todo!()
    }

    /// `pressure` as on the wire.
    pub fn pressure(&mut self, wire: u32) {
        let _ = wire;
        todo!()
    }

    /// `tilt` in `wl_fixed_t` degrees.
    pub fn tilt(&mut self, x: i32, y: i32) {
        let _ = (x, y);
        todo!()
    }

    /// `button` with an evdev code and `wl_pointer`-like state (1 pressed).
    pub fn button(&mut self, code: u32, state: u32) {
        let _ = (code, state);
        todo!()
    }

    /// `type`: the kind from the wire value.
    pub fn set_kind(&mut self, wire: u32) {
        let _ = wire;
        todo!()
    }

    /// `frame`: the events of the frame, which is then empty again.
    pub fn frame(&mut self) -> Vec<RawEvent> {
        todo!()
    }

    /// The kind of the tool.
    #[must_use]
    pub fn kind(&self) -> ToolKind {
        self.kind
    }

    /// Whether the tool is in proximity of the window.
    #[must_use]
    pub fn in_proximity(&self) -> bool {
        self.focused
    }

    /// Whether the tip touches the tablet, as last sent.
    #[must_use]
    pub fn tip_down(&self) -> bool {
        self.tip_down
    }

    /// The last tilt in degrees, always finite.
    #[must_use]
    pub fn tilt_degrees(&self) -> (f32, f32) {
        self.tilt
    }
}

/// The tools of the seat, by proxy address.
#[derive(Debug, Default)]
pub struct Pens {
    tools: Vec<(usize, PenTool)>,
}

impl Pens {
    /// Adds a tool.
    pub fn add(&mut self, id: usize) {
        let _ = id;
        todo!()
    }

    /// Removes a tool.
    pub fn remove(&mut self, id: usize) {
        let _ = id;
        todo!()
    }

    /// The tool with `id`.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut PenTool> {
        let _ = id;
        todo!()
    }

    /// Whether pointer input is dropped (ADR-T40-1).
    #[must_use]
    pub fn shadowed(&self) -> bool {
        todo!()
    }

    /// How many tools are known.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Whether no tool is known.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}
