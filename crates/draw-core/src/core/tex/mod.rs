//! A small TeX subset inside text blocks (ADR-T30-1).
//!
//! [`parse`] splits the text of a block into plain text and `$…$`
//! formulas and parses each formula into a lossless tree of [`Node`]s:
//! [`print()`] gives the source back. [`layout`] typesets the tree into
//! positioned glyphs and rules ([`Ink`]) with TeX's box model, reduced:
//! every box has a width, an ascent and a descent, in em of the block's
//! font size, y down. Both are total: anything outside the subset is a
//! [`Node::Literal`], drawn as its source with the warning flag.
//!
//! The code lives in five modules, all re-exported here so the public
//! API is one flat namespace (T139):
//!
//! - `metrics`: layout constants, kerns and glyph extents;
//! - `parse`: the node tree, the parser and the printer;
//! - `glyphs`: symbol, accent, delimiter and alphabet tables;
//! - `macros`: table-driven expansions and user macros;
//! - `layout`: typesetting nodes into boxes and ink.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::core::geom::{Aabb, Vec2};
use crate::core::math_metrics;
use crate::core::text_layout::{ADVANCE, BASELINE, LINE_HEIGHT};

mod glyphs;
mod layout;
mod macros;
mod metrics;
mod parse;

pub use glyphs::*;
pub use layout::*;
pub use macros::*;
pub use metrics::*;
pub use parse::*;

#[cfg(test)]
mod tests;
