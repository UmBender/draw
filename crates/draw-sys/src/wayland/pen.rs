//! The pure state of Wayland pen tools (T170, ADR-T170-1): what the
//! `zwp_tablet_tool_v2` events of one `frame` become as
//! [`RawEvent`]s. No compositor and no `unsafe`.

use crate::platform::{RawButton, RawEvent, pointer_shadowed_by_pen};
use crate::wayland::input::{fixed_to_f32, pointer_position};

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
    match wire {
        0x140 => ToolKind::Pen,
        0x141 => ToolKind::Eraser,
        _ => ToolKind::Other,
    }
}

/// A pressure from the wire as `0..=1`.
#[must_use]
pub fn pressure_from_wire(wire: u32) -> f32 {
    (wire as f32 / PRESSURE_MAX).clamp(0.0, 1.0)
}

/// The button a barrel button's evdev code stands for (`BTN_STYLUS` and
/// `BTN_STYLUS2`).
#[must_use]
pub fn pen_button_from_evdev(code: u32) -> Option<RawButton> {
    match code {
        0x14b => Some(RawButton::Middle),
        0x14c => Some(RawButton::Right),
        _ => None,
    }
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
        self.focused = ours;
        self.proximity_out = false;
    }

    /// `proximity_out`, applied at the next `frame`.
    pub fn proximity_out(&mut self) {
        self.proximity_out = true;
    }

    /// `down`.
    pub fn down(&mut self) {
        self.tip = Some(true);
    }

    /// `up`.
    pub fn up(&mut self) {
        self.tip = Some(false);
    }

    /// `motion` in `wl_fixed_t` surface coordinates at `scale`.
    pub fn motion(&mut self, x: i32, y: i32, scale: i32) {
        self.motion = Some(pointer_position(x, y, scale));
    }

    /// `pressure` as on the wire.
    pub fn pressure(&mut self, wire: u32) {
        self.pressure = pressure_from_wire(wire);
    }

    /// `tilt` in `wl_fixed_t` degrees.
    pub fn tilt(&mut self, x: i32, y: i32) {
        self.tilt = (fixed_to_f32(x), fixed_to_f32(y));
    }

    /// `button` with an evdev code and `wl_pointer`-like state (1 pressed).
    pub fn button(&mut self, code: u32, state: u32) {
        if let Some(button) = pen_button_from_evdev(code) {
            self.buttons.push((button, state == 1));
        }
    }

    /// `type`: the kind from the wire value.
    pub fn set_kind(&mut self, wire: u32) {
        self.kind = tool_kind(wire);
    }

    /// `frame`: the events of the frame, which is then empty again. A frame
    /// of a tool that is not in proximity of the window sends nothing.
    pub fn frame(&mut self) -> Vec<RawEvent> {
        let motion = self.motion.take();
        let tip = self.tip.take();
        let buttons = std::mem::take(&mut self.buttons);
        let leaving = std::mem::take(&mut self.proximity_out);
        let mut out = Vec::new();
        if !self.focused {
            return out;
        }
        let pressure = self.pressure;
        let mut send = |event: RawEvent, (x, y): (f32, f32)| {
            out.push(RawEvent::Tablet { x, y, pressure });
            out.push(event);
        };
        if let Some((x, y)) = motion {
            self.position = (x, y);
            send(RawEvent::PointerMove { x, y }, (x, y));
        }
        let (x, y) = self.position;
        let mut button = |button: RawButton, down: bool| {
            send(RawEvent::Button { button, x, y, down }, (x, y));
        };
        if let Some(down) = tip.filter(|down| *down != self.tip_down) {
            self.tip_down = down;
            button(RawButton::Left, down);
        }
        for (b, down) in buttons {
            button(b, down);
        }
        if leaving {
            if self.tip_down {
                self.tip_down = false;
                button(RawButton::Left, false);
            }
            self.focused = false;
        }
        out
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
        if self.tools.iter().all(|(known, _)| *known != id) {
            self.tools.push((id, PenTool::new()));
        }
    }

    /// Removes a tool.
    pub fn remove(&mut self, id: usize) {
        self.tools.retain(|(known, _)| *known != id);
    }

    /// The tool with `id`.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut PenTool> {
        self.tools
            .iter_mut()
            .find(|(known, _)| *known == id)
            .map(|(_, tool)| tool)
    }

    /// The ids of the known tools.
    #[must_use]
    pub fn ids(&self) -> Vec<usize> {
        self.tools.iter().map(|(id, _)| *id).collect()
    }

    /// Whether pointer input is dropped (ADR-T40-1).
    #[must_use]
    pub fn shadowed(&self) -> bool {
        pointer_shadowed_by_pen(self.tools.iter().map(|(_, tool)| tool.focused))
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
