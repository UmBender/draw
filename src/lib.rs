//! draw — a minimal, fast, keyboard-first scratch canvas for competitive programming.
//!
//! [`core`] holds all editing logic and has no dependency on the window or GPU;
//! [`shell`] is the thin macroquad layer that feeds it input and draws its state.
//! [`theme`] holds every colour draw shows; edit it and rebuild to restyle.

#![forbid(unsafe_code)]
// ADR-T131-1: test code may end a match with `other => panic!(…)`.
#![cfg_attr(test, allow(clippy::wildcard_enum_match_arm))]

// The pure logic and the theme live in `draw-core` (ADR-T132-1); the paths
// `draw::core` and `draw::theme` are unchanged.
pub use draw_core::core;
pub use draw_core::theme;
pub mod shell;
