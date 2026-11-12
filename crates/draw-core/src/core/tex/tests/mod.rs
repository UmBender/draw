//! Tests of the TeX subset.

use super::*;
use crate::core::geom::{Aabb, Vec2, approx_eq};
use crate::core::math_metrics::{self, GlyphMetrics};
use crate::core::text_layout;
use proptest::prelude::*;

const EPS: f32 = 1e-4;

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// The nodes of formula `src` (written without its `$`s).
fn math(src: &str) -> Vec<Node> {
    match parse(&format!("${src}$")).as_slice() {
        [Segment::Math(nodes)] => nodes.clone(),
        other => panic!("one formula expected for {src:?}: {other:?}"),
    }
}

fn tok(node: Node) -> Arg {
    Arg {
        lead: String::new(),
        braced: false,
        nodes: vec![node],
    }
}

fn braced(nodes: Vec<Node>) -> Arg {
    Arg {
        lead: String::new(),
        braced: true,
        nodes,
    }
}

fn sym(name: &str) -> Node {
    Node::Symbol(name.to_owned())
}

fn script(kind: ScriptKind, arg: Arg) -> Script {
    Script { kind, arg }
}

/// `(char, position, scale, warn)` of every glyph, in order.
fn glyphs(ink: &[Ink]) -> Vec<(char, Vec2, f32, bool)> {
    ink.iter()
        .filter_map(|ink| match *ink {
            Ink::Glyph {
                ch,
                pos,
                scale,
                warn,
                ..
            } => Some((ch, pos, scale, warn)),
            Ink::Rule { .. } => None,
        })
        .collect()
}

/// `(char, face)` of every glyph, in order.
fn faces(ink: &[Ink]) -> Vec<(char, Face)> {
    ink.iter()
        .filter_map(|ink| match *ink {
            Ink::Glyph { ch, face, .. } => Some((ch, face)),
            Ink::Rule { .. } => None,
        })
        .collect()
}

/// `(a, b, width)` of every rule, in order.
fn rules(ink: &[Ink]) -> Vec<(Vec2, Vec2, f32)> {
    ink.iter()
        .filter_map(|ink| match *ink {
            Ink::Rule { a, b, width } => Some((a, b, width)),
            Ink::Glyph { .. } => None,
        })
        .collect()
}

/// The glyph `ch` of `ink`; panics if it is missing.
fn glyph(ink: &[Ink], ch: char) -> (Vec2, f32) {
    match glyphs(ink).into_iter().find(|g| g.0 == ch) {
        Some((_, pos, scale, _)) => (pos, scale),
        None => panic!("no glyph {ch:?} in {ink:?}"),
    }
}

fn boxed(src: &str) -> MathBox {
    layout_math(&math(src), 1.0)
}
// ---- AC-2 layout (metrics of ADR-T33-1) -------------------------------

/// The math-font metrics of `ch`; panics if the font lacks it.
fn mm(ch: char) -> GlyphMetrics {
    match math_metrics::get(ch) {
        Some(m) => m,
        None => panic!("no math metrics for {ch:?}"),
    }
}

/// Advance of `chars` side by side in the math face at scale 1.
fn adv(chars: &str) -> f32 {
    chars.chars().map(|ch| mm(ch).advance).sum()
}

/// Math italic `x`, `a`, `b`, … (ADR-T33-1).
const X: char = '\u{1D465}';
const A: char = '\u{1D44E}';
const B: char = '\u{1D44F}';
const F: char = '\u{1D453}';
const I: char = '\u{1D456}';
const J: char = '\u{1D457}';
const K: char = '\u{1D458}';
const N: char = '\u{1D45B}';
const Y: char = '\u{1D466}';
const Z: char = '\u{1D467}';
mod alphabets;
mod basics;
mod macros;
mod split;
mod structures;
