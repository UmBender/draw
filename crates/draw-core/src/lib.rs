//! Pure editing logic of draw: no window and no GPU (ADR-T132-1).
//!
//! [`core`] keeps its module paths from before the workspace split, and
//! [`theme`] holds every colour draw shows; edit it and rebuild to restyle.

#![forbid(unsafe_code)]
// ADR-T131-1: test code may end a match with `other => panic!(…)`.
#![cfg_attr(test, allow(clippy::wildcard_enum_match_arm))]

pub mod core;
pub mod machine;
pub mod theme;
