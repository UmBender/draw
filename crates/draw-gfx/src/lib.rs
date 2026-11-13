//! The retained renderer of draw over an abstract GPU (ADR-T132-1). The
//! [`gpu`] trait is in; the renderer tasks of the 2.0 plan fill the rest.
//! Only `gpu/gl.rs` may allow `unsafe` (ADR-T127-2).

#![deny(unsafe_code)]

pub mod gpu;
