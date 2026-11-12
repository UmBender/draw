//! Units and plain value types of draw: no I/O and no `unsafe`
//! (ADR-T132-1). [`geom`] holds the vectors and boxes the other crates share.

#![forbid(unsafe_code)]

pub mod geom;
