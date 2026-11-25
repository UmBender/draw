//! Pen pressure from `XInput2` raw events, as pure functions (T176 AC-3).

use crate::platform::RawEvent;

/// `XInput2` version negotiated with `XIQueryVersion` (T196).
///
/// `XInput2` 2.1 delivers `XI_RawMotion` even while a button press holds an
/// implicit passive pointer grab; 2.0 suppresses raw motion during grabs.
pub const XI_VERSION: (i32, i32) = (2, 1);

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

/// What an `XInput2` device probe found for pen pressure (T196 AC-2).
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceProbe {
    /// The `XInput2` device id.
    pub id: i32,
    /// The device name reported by `XIQueryDevice`.
    pub name: String,
    /// Number of valuator classes on the device.
    pub valuators: usize,
    /// The `Abs Pressure` axis, if the device has one.
    pub pressure: Option<PressureAxis>,
}

/// Resolves the source device id of an `XIRawEvent`.
///
/// Prefers `source_id` (the physical slave device); falls back to
/// `device_id` when `source_id` is unset (`<= 1`). Returns `None` if
/// neither names a concrete device (`0` is `XIAllDevices`, `1` is
/// `XIAllMasterDevices`).
#[must_use]
pub const fn raw_source(device_id: i32, source_id: i32) -> Option<i32> {
    if source_id > 1 {
        Some(source_id)
    } else if device_id > 1 {
        Some(device_id)
    } else {
        None
    }
}

/// Builds a [`DeviceProbe`] from a device's `(number, label, min, max)`
/// valuators and the interned `Abs Pressure` atom.
#[must_use]
pub fn probe_valuators(
    id: i32,
    name: String,
    pressure_atom: u64,
    axes: &[(i32, u64, f64, f64)],
) -> DeviceProbe {
    let pressure = (pressure_atom != 0)
        .then(|| {
            axes.iter()
                .rfind(|&&(_, label, _, _)| label == pressure_atom)
                .map(|&(number, _, min, max)| PressureAxis { number, min, max })
        })
        .flatten();
    DeviceProbe {
        id,
        name,
        valuators: axes.len(),
        pressure,
    }
}

/// Formats a `--log debug` line for `probe`, or `None` for devices
/// without valuators (T196 AC-2).
#[must_use]
pub fn probe_log_line(probe: &DeviceProbe) -> Option<String> {
    if let Some(axis) = probe.pressure {
        Some(format!(
            "x11 pen: device {} {:?} pressure axis {} range {}..{}",
            probe.id, probe.name, axis.number, axis.min, axis.max
        ))
    } else if probe.valuators > 0 {
        Some(format!(
            "x11 pen: device {} {:?} has {} valuators, no Abs Pressure",
            probe.id, probe.name, probe.valuators
        ))
    } else {
        None
    }
}

/// The value of valuator `number` in a raw event, `None` if it did not
/// change. Values are packed in the order of the set bits of `mask`.
#[must_use]
pub fn valuator(mask: &[u8], values: &[f64], number: i32) -> Option<f64> {
    let number = usize::try_from(number).ok()?;
    let is_set = |bit: usize| {
        mask.get(bit / 8)
            .is_some_and(|byte| byte & (1 << (bit % 8)) != 0)
    };
    if !is_set(number) {
        return None;
    }
    let index = (0..number).filter(|&bit| is_set(bit)).count();
    values.get(index).copied()
}

/// `value` on `axis` in `0..=1`; `0` for a non-finite value or an empty range.
#[must_use]
pub fn normalise(value: f64, axis: PressureAxis) -> f32 {
    let range = axis.max - axis.min;
    let unit = (value - axis.min) / range;
    if range > 0.0 && unit.is_finite() {
        #[allow(clippy::cast_possible_truncation)]
        let narrow = unit.clamp(0.0, 1.0) as f32;
        narrow
    } else {
        0.0
    }
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
    pub fn raw_event(
        &mut self,
        device: i32,
        axis: Option<PressureAxis>,
        mask: &[u8],
        values: &[f64],
    ) {
        let Some(axis) = axis else {
            self.current = None;
            return;
        };
        let previous = match self.current {
            Some((pen, pressure)) if pen == device => pressure,
            _ => 0.0,
        };
        let pressure =
            valuator(mask, values, axis.number).map_or(previous, |value| normalise(value, axis));
        self.current = Some((device, pressure));
    }

    /// The pressure of the pen that sent the last raw event, if it was one.
    #[must_use]
    pub fn pressure(&self) -> Option<f32> {
        self.current.map(|(_, pressure)| pressure)
    }

    /// The pen sample for a core event at `(x, y)`, `None` without a pen.
    #[must_use]
    pub fn sample(&self, x: f32, y: f32) -> Option<RawEvent> {
        self.pressure()
            .map(|pressure| RawEvent::Tablet { x, y, pressure })
    }
}
