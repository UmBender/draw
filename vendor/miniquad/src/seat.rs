//! Seat devices on Wayland (draw patch, ADR-T35-1; ADR-T44-1).
//!
//! A compositor may send `wl_seat.capabilities` again whenever the seat's
//! devices change. The backend keeps at most one keyboard, pointer and
//! touch device: [`device_change`] says, for each of them, what one
//! `capabilities` event does to it.

/// What a `wl_seat.capabilities` event does to one seat device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceChange {
    /// Ask the seat for the device and add its listener.
    Create,
    /// Release the device: its capability is gone.
    Release,
    /// Leave the device as it is.
    Keep,
}

/// What to do with a seat device that `exists` (or not) when its
/// capability is `announced` (or not) by a `capabilities` event.
///
/// A capability announced again keeps the device it already has, so a
/// re-sent event adds no second listener.
pub fn device_change(exists: bool, announced: bool) -> DeviceChange {
    match (exists, announced) {
        (false, true) => DeviceChange::Create,
        (true, false) => DeviceChange::Release,
        _ => DeviceChange::Keep,
    }
}

/// Whether seat devices of a `wl_seat` bound at `version` have the
/// `release` request (version 3 and later); older ones are only
/// destroyed.
pub fn has_release_request(version: u32) -> bool {
    version >= 3
}
