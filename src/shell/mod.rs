//! The macroquad layer: window, input translation, rendering and toolbar.
//!
//! The only part of the crate that depends on macroquad.

pub mod app;
pub mod autosave;
pub mod backend;
pub mod command_palette;
pub mod crash;
pub mod diag;
pub mod export;
pub mod file;
pub mod glyphs;
pub mod input_map;
pub mod minimap;
pub mod options;
pub mod preflight;
pub mod recording;
pub mod render;
pub mod sheet;
pub mod tablet;
pub mod toolbar;
pub mod ui_shapes;
pub mod ui_text;
