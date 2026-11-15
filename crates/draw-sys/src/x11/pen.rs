//! Pen pressure from XInput2 raw events, as pure functions (T176 AC-3).

use crate::platform::RawEvent;

/// The `Abs Pressure` valuator of a pen device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureAxis {
    /// The valuator number.
    pub number: i32,
    /// The smallest value the device reports.
    pub min: f64,
    /// The largest value the device reports.
    pub max: f64,
}

/// The value of valuator `number` in a raw event, `None` if it did not
/// change. Values are packed in the order of the set bits of `mask`.
#[must_use]
pub fn valuator(mask: &[u8], values: &[f64], number: i32) -> Option<f64> {
    todo!("T176 behaviour: {mask:?} {values:?} {number}")
}

/// `value` on `axis` in `0..=1`; `0` for a non-finite value or an empty range.
#[must_use]
pub fn normalise(value: f64, axis: PressureAxis) -> f32 {
    todo!("T176 behaviour: {value} {axis:?}")
}

/// Follows the pressure of the pen that sent the last raw motion.
///
/// A device with a pressure axis is a pen: its normalised pressure is kept
/// until the next raw event. A raw event only carries the valuators that
/// changed, so a missing pressure keeps the last one; a raw event from a
/// device without the axis clears it.
#[derive(Debug, Default)]
pub struct PenTracker {
    current: Option<(i32, f32)>,
}

impl PenTracker {
    /// Feeds one raw event of source `device`, whose pressure `axis` is
    /// `None` for a device that is not a pen.
    pub fn raw_event(&mut self, device: i32, axis: Option<PressureAxis>, mask: &[u8], values: &[f64]) {
        todo!("T176 behaviour: {device} {axis:?} {mask:?} {values:?} {:?}", self.current)
    }

    /// The pressure of the pen that sent the last raw event, if it was one.
    #[must_use]
    pub fn pressure(&self) -> Option<f32> {
        todo!("T176 behaviour: {:?}", self.current)
    }

    /// The pen sample for a core event at `(x, y)`, `None` without a pen.
    #[must_use]
    pub fn sample(&self, x: f32, y: f32) -> Option<RawEvent> {
        todo!("T176 behaviour: {x} {y}")
    }
}
