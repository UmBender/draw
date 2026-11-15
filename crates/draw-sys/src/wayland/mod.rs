//! The Wayland backend (T143, slice T168): connection, window and protocol
//! states over `libwayland-client`, loaded with `dlopen`. The protocol code
//! in [`generated`] is generated from `protocols/*.xml` (ADR-T168-1).

pub mod client;
pub mod connection;
pub mod ffi;
pub mod generated;
pub mod platform;
pub mod window;

pub use connection::{Connection, WaylandError};
pub use platform::WaylandPlatform;
pub use window::{Configured, Created, Mapped, Window};
