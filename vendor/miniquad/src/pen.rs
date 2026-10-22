//! Pen tablet samples (draw patch, ADR-T35-1).
//!
//! `EventHandler` has no pressure, so the backends that read a pen push a
//! [`PenSample`] here just before they hand the same pointer motion, press
//! or release to the event handler. The application drains them once per
//! frame with [`take_samples`] and pairs them with its pointer events by
//! position.

use std::sync::Mutex;

/// One pen pointer event's position and pressure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PenSample {
    /// Position in physical pixels, as passed to the event handler.
    pub x: f32,
    /// Position in physical pixels, as passed to the event handler.
    pub y: f32,
    /// Tip pressure in `[0, 1]`.
    pub pressure: f32,
}

/// Samples pushed since the last [`take_samples`].
static SAMPLES: Mutex<Vec<PenSample>> = Mutex::new(Vec::new());

/// Most samples kept between two [`take_samples`]; older ones are dropped.
const MAX_SAMPLES: usize = 4096;

/// Removes and returns the samples pushed so far, oldest first.
pub fn take_samples() -> Vec<PenSample> {
    let mut samples = SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
    std::mem::take(&mut *samples)
}

/// Queues a sample for the pointer event the backend is about to send.
#[allow(dead_code)] // only the Linux backends push samples
pub(crate) fn push(x: f32, y: f32, pressure: f32) {
    let mut samples = SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
    if samples.len() >= MAX_SAMPLES {
        samples.drain(..MAX_SAMPLES / 2);
    }
    samples.push(PenSample { x, y, pressure });
}

/// Whether the backend drops a plain pointer motion or button event because
/// a tablet tool is in proximity of the window (draw patch, ADR-T35-1;
/// ADR-T40-1).
///
/// Some compositors (Hyprland) copy the pen onto `wl_pointer` as well as
/// sending it through the tablet protocol. The copy has no [`PenSample`],
/// so it would steal the sample of the pen event at the same position.
/// `in_proximity` holds, for every known tool, whether it is in proximity.
pub fn pointer_shadowed_by_pen(in_proximity: impl IntoIterator<Item = bool>) -> bool {
    let _ = in_proximity;
    todo!()
}
