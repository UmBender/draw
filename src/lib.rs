//! draw — a minimal, fast, keyboard-first scratch canvas for competitive programming.
//!
//! [`core`] holds all editing logic and has no dependency on the window or GPU;
//! [`shell`] is the thin macroquad layer that feeds it input and draws its state.

pub mod core;
pub mod shell;
