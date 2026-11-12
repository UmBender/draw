//! Units and plain value types of draw: no I/O and no `unsafe`
//! (ADR-T132-1). [`geom`] holds the vectors and boxes the other crates share;
//! [`units`], [`ids`], [`zoom`] and [`nonempty`] hold the types that cannot be
//! mixed up (ADR-T133-1).

#![forbid(unsafe_code)]

pub mod geom;
pub mod ids;
pub mod nonempty;
pub mod units;
pub mod zoom;
