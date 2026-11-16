//! The Wayland backend (T143, slice T168): connection, window and protocol
//! states over `libwayland-client`, loaded with `dlopen`. The protocol code
//! in [`generated`] is generated from `protocols/*.xml` (ADR-T168-1).

pub mod client;
pub(crate) mod clipboard;
pub mod connection;
pub mod ffi;
#[rustfmt::skip]
pub mod generated;
pub mod input;
pub mod pen;
pub mod platform;
pub(crate) mod seat;
pub(crate) mod tablet;
pub mod transfer;
pub mod window;
pub mod xkb;

pub use connection::{Connection, WaylandError};
pub use platform::WaylandPlatform;
pub use window::{Configured, Created, Mapped, Window};
