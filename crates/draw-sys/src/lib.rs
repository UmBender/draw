//! The platform layer of draw: windows, input and the GL context
//! (ADR-T132-1). It is empty until the backend tasks of the 2.0 plan fill
//! it. Files here may allow `unsafe` (ADR-T127-2), each primitive wrapped,
//! documented and tested.

#![deny(unsafe_code)]
