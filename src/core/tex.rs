//! A small TeX subset inside text blocks (ADR-T30-1).
//!
//! [`parse`] splits the text of a block into plain text and `$…$`
//! formulas and parses each formula into a lossless tree of [`Node`]s:
//! [`print()`] gives the source back. [`layout`] typesets the tree into
//! positioned glyphs and rules ([`Ink`]) with TeX's box model, reduced:
//! every box has a width, an ascent and a descent, in em of the block's
//! font size, y down. Both are total: anything outside the subset is a
//! [`Node::Literal`], drawn as its source with the warning flag.

use crate::core::geom::{Aabb, Vec2};
use crate::core::math_metrics;
use crate::core::text_layout::{ADVANCE, BASELINE, LINE_HEIGHT};

/// Ascent of a text-face glyph box per unit of scale: the text font's cap
/// height 0.735 with its overshoot.
pub const GLYPH_ASCENT: f32 = 0.75;

/// Descent of a text-face glyph box per unit of scale: the text font's
/// descender.
pub const GLYPH_DESCENT: f32 = 0.22;

/// Ascent of text-face brackets and bars per unit of scale.
pub const TALL_ASCENT: f32 = 0.85;

/// Descent of text-face brackets and bars per unit of scale.
pub const TALL_DESCENT: f32 = 0.17;

/// Height of the math axis (the middle of `−`) above the baseline: the
/// math font's `AxisHeight` (ADR-T33-1).
pub const AXIS: f32 = 0.25;

/// Thickness of fraction bars and root signs: `FractionRuleThickness`.
pub const RULE: f32 = 0.04;

/// Scale of scripts, limits and fraction parts relative to their parent:
/// `ScriptPercentScaleDown`.
pub const SCRIPT_SCALE: f32 = 0.7;

/// Smallest scale of nested scripts: `ScriptScriptPercentScaleDown`.
pub const MIN_SCALE: f32 = 0.5;

/// Smallest rise of a superscript's baseline: `SuperscriptShiftUp`.
pub const SUP_SHIFT: f32 = 0.363;

/// A superscript's baseline sits at most this far below the top of its
/// base: `SuperscriptBaselineDropMax`.
pub const SUP_DROP: f32 = 0.25;

/// Smallest height of a superscript's bottom above the baseline:
/// `SuperscriptBottomMin`.
pub const SUP_BOTTOM_MIN: f32 = 0.108;

/// Smallest drop of a subscript's baseline: `SubscriptShiftDown`.
pub const SUB_SHIFT: f32 = 0.247;

/// A subscript's baseline sits this far above the bottom of its base.
pub const SUB_DROP: f32 = 0.05;

/// Highest top of a subscript above the baseline: `SubscriptTopMax`.
pub const SUB_TOP_MAX: f32 = 0.344;

/// Smallest gap between a superscript and a subscript on one base:
/// `SubSuperscriptGapMin`.
pub const SCRIPT_GAP: f32 = 0.16;

/// Space after scripts: `SpaceAfterScript`.
pub const SCRIPT_SPACE: f32 = 0.056;

/// Gap between a big operator and its limits: `LowerLimitGapMin`.
pub const LIMIT_GAP: f32 = 0.167;

/// Scale of the big operators `∑` and `∏`.
pub const BIG_OP_SCALE: f32 = 1.3;

/// Smallest gap between a fraction bar and its numerator or denominator:
/// `FractionNumeratorGapMin`.
pub const FRAC_GAP: f32 = 0.04;

/// Smallest rise of a numerator's baseline: `FractionNumeratorShiftUp`.
pub const FRAC_NUM_SHIFT: f32 = 0.394;

/// Smallest drop of a denominator's baseline:
/// `FractionDenominatorShiftDown`.
pub const FRAC_DEN_SHIFT: f32 = 0.345;

/// Space left and right of a fraction's parts.
pub const FRAC_PAD: f32 = 0.1;

/// Gap between a root's content and its overbar.
pub const SQRT_GAP: f32 = 0.1;

/// Width of the root sign before the content.
pub const SQRT_SIGN: f32 = 0.55;

/// Largest scale of a binomial's parentheses.
pub const BINOM_MAX_SCALE: f32 = 3.0;

/// Space on each side of a relation: TeX's thick space, 5/18 em.
pub const REL_SPACE: f32 = 5.0 / 18.0;

/// Space on each side of a binary operator: TeX's medium space, 2/9 em.
pub const BIN_SPACE: f32 = 2.0 / 9.0;

/// Space between a function or big operator and an ordinary atom, and
/// after punctuation: TeX's thin space, 1/6 em.
pub const OP_SPACE: f32 = 1.0 / 6.0;

/// Room kept above and below a formula in its line.
pub const MATH_PAD: f32 = 0.03;

/// The math minus a `-` is drawn as.
pub const MINUS: char = '\u{2212}';

/// Deepest brace nesting parsed as groups; deeper braces are literal, so
/// parsing and layout never recurse without bound.
pub const MAX_DEPTH: usize = 32;

/// A run of a block's text.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    /// Plain text, drawn as typed (may contain line breaks).
    Text(String),
    /// `\$` outside a formula: a literal dollar sign.
    Dollar,
    /// A `$…$` formula, without its dollars.
    Math(Vec<Node>),
}

/// One node of a formula; every node keeps its source (ADR-T30-1).
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// Whitespace, ignored by the layout.
    Space(String),
    /// A character typed as is: letter, digit, operator, bracket.
    Char(char),
    /// A command of the symbol table, by name without the backslash.
    Symbol(String),
    /// A balanced `{…}` group.
    Group(Vec<Node>),
    /// Scripts on a base: one `^` and one `_` at most, in source order.
    Scripts {
        /// The atom before the first script; `None` when there is none
        /// right before it.
        base: Option<Box<Node>>,
        /// The first script.
        first: Script,
        /// The second script, of the other kind.
        second: Option<Script>,
    },
    /// `\frac`, `\binom`, `\sqrt` or `\pmod` with all its arguments.
    Command {
        /// Name without the backslash.
        name: String,
        /// The arguments, as many as the command takes.
        args: Vec<Arg>,
    },
    /// `\text{…}`: a raw body drawn as typed.
    Text {
        /// Whitespace between `\text` and `{`.
        lead: String,
        /// The body between the braces (balanced braces allowed).
        body: String,
    },
    /// Source outside the subset, drawn as typed in the warning colour.
    Literal(String),
}

/// A superscript or subscript.
#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    /// `^` or `_`.
    pub kind: ScriptKind,
    /// What follows it.
    pub arg: Arg,
}

/// The kind of a script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptKind {
    /// `^`.
    Sup,
    /// `_`.
    Sub,
}

/// The argument of a script or command.
#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    /// Whitespace skipped before the argument.
    pub lead: String,
    /// Whether it is a `{group}`; otherwise `nodes` is one token.
    pub braced: bool,
    /// The argument's nodes.
    pub nodes: Vec<Node>,
}

/// TeX atom class, which sets the space between neighbours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Ordinary: letters, digits, Greek, `∞`.
    Ord,
    /// A function name in upright type: `log`.
    Op,
    /// A word with limits: `min`, `max`, `lim`.
    Limits,
    /// A big operator with limits, drawn enlarged: `∑`, `∏`.
    BigOp,
    /// An integral, drawn enlarged with its scripts beside it: `∫`, `∮`
    /// (ADR-T36-1).
    Integral,
    /// A binary operator: `+`, `×`.
    Bin,
    /// A relation: `=`, `≤`.
    Rel,
    /// An opening bracket.
    Open,
    /// A closing bracket.
    Close,
    /// Punctuation: `,`, `;`.
    Punct,
}

/// One command of the symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolDef {
    /// Command name without the backslash.
    pub name: &'static str,
    /// What it draws: one character or an upright word.
    pub text: &'static str,
    /// Its atom class.
    pub class: Class,
}

/// Shorthand for a table row.
const fn def(name: &'static str, text: &'static str, class: Class) -> SymbolDef {
    SymbolDef { name, text, class }
}

/// Every symbol command of the subset (ADR-T30-1).
pub const SYMBOLS: &[SymbolDef] = &[
    def("alpha", "α", Class::Ord),
    def("beta", "β", Class::Ord),
    def("gamma", "γ", Class::Ord),
    def("delta", "δ", Class::Ord),
    def("epsilon", "ϵ", Class::Ord),
    def("varepsilon", "ε", Class::Ord),
    def("zeta", "ζ", Class::Ord),
    def("eta", "η", Class::Ord),
    def("theta", "θ", Class::Ord),
    def("vartheta", "ϑ", Class::Ord),
    def("iota", "ι", Class::Ord),
    def("kappa", "κ", Class::Ord),
    def("lambda", "λ", Class::Ord),
    def("mu", "μ", Class::Ord),
    def("nu", "ν", Class::Ord),
    def("xi", "ξ", Class::Ord),
    def("omicron", "ο", Class::Ord),
    def("pi", "π", Class::Ord),
    def("varpi", "ϖ", Class::Ord),
    def("rho", "ρ", Class::Ord),
    def("varrho", "ϱ", Class::Ord),
    def("sigma", "σ", Class::Ord),
    def("varsigma", "ς", Class::Ord),
    def("tau", "τ", Class::Ord),
    def("upsilon", "υ", Class::Ord),
    def("phi", "ϕ", Class::Ord),
    def("varphi", "φ", Class::Ord),
    def("chi", "χ", Class::Ord),
    def("psi", "ψ", Class::Ord),
    def("omega", "ω", Class::Ord),
    def("Gamma", "Γ", Class::Ord),
    def("Delta", "Δ", Class::Ord),
    def("Theta", "Θ", Class::Ord),
    def("Lambda", "Λ", Class::Ord),
    def("Xi", "Ξ", Class::Ord),
    def("Pi", "Π", Class::Ord),
    def("Sigma", "Σ", Class::Ord),
    def("Upsilon", "Υ", Class::Ord),
    def("Phi", "Φ", Class::Ord),
    def("Psi", "Ψ", Class::Ord),
    def("Omega", "Ω", Class::Ord),
    def("le", "≤", Class::Rel),
    def("leq", "≤", Class::Rel),
    def("ge", "≥", Class::Rel),
    def("geq", "≥", Class::Rel),
    def("ne", "≠", Class::Rel),
    def("neq", "≠", Class::Rel),
    def("approx", "≈", Class::Rel),
    def("equiv", "≡", Class::Rel),
    def("to", "→", Class::Rel),
    def("leftarrow", "←", Class::Rel),
    def("Rightarrow", "⇒", Class::Rel),
    def("iff", "⇔", Class::Rel),
    def("in", "∈", Class::Rel),
    def("notin", "∉", Class::Rel),
    def("subset", "⊂", Class::Rel),
    def("cdot", "·", Class::Bin),
    def("times", "×", Class::Bin),
    def("div", "÷", Class::Bin),
    def("pm", "±", Class::Bin),
    def("cup", "∪", Class::Bin),
    def("cap", "∩", Class::Bin),
    def("land", "∧", Class::Bin),
    def("lor", "∨", Class::Bin),
    def("oplus", "⊕", Class::Bin),
    def("bmod", "mod", Class::Bin),
    def("neg", "¬", Class::Ord),
    def("infty", "∞", Class::Ord),
    def("ldots", "…", Class::Ord),
    def("cdots", "⋯", Class::Ord),
    def("lfloor", "⌊", Class::Open),
    def("rfloor", "⌋", Class::Close),
    def("lceil", "⌈", Class::Open),
    def("rceil", "⌉", Class::Close),
    def("{", "{", Class::Open),
    def("}", "}", Class::Close),
    def("$", "$", Class::Ord),
    def("sum", "∑", Class::BigOp),
    def("prod", "∏", Class::BigOp),
    def("int", "∫", Class::Integral),
    def("iint", "∬", Class::Integral),
    def("iiint", "∭", Class::Integral),
    def("oint", "∮", Class::Integral),
    def("oiint", "∯", Class::Integral),
    def("min", "min", Class::Limits),
    def("max", "max", Class::Limits),
    def("lim", "lim", Class::Limits),
    // Physics symbols (T36).
    def("partial", "∂", Class::Ord),
    def("nabla", "∇", Class::Ord),
    def("hbar", "ℏ", Class::Ord),
    def("ell", "ℓ", Class::Ord),
    def("Re", "ℜ", Class::Ord),
    def("Im", "ℑ", Class::Ord),
    def("AA", "Å", Class::Ord),
    def("degree", "°", Class::Ord),
    def("forall", "∀", Class::Ord),
    def("exists", "∃", Class::Ord),
    def("emptyset", "∅", Class::Ord),
    def("prime", "′", Class::Ord),
    def("vdots", "⋮", Class::Ord),
    def("ddots", "⋱", Class::Ord),
    def("vert", "|", Class::Ord),
    def("Vert", "‖", Class::Ord),
    def("propto", "∝", Class::Rel),
    def("sim", "∼", Class::Rel),
    def("simeq", "≃", Class::Rel),
    def("cong", "≅", Class::Rel),
    def("ll", "≪", Class::Rel),
    def("gg", "≫", Class::Rel),
    def("lesssim", "≲", Class::Rel),
    def("gtrsim", "≳", Class::Rel),
    def("doteq", "≐", Class::Rel),
    def("coloneqq", "≔", Class::Rel),
    def("parallel", "∥", Class::Rel),
    def("perp", "⊥", Class::Rel),
    def("mid", "∣", Class::Rel),
    def("rightarrow", "→", Class::Rel),
    def("leftrightarrow", "↔", Class::Rel),
    def("Leftarrow", "⇐", Class::Rel),
    def("Leftrightarrow", "⇔", Class::Rel),
    def("uparrow", "↑", Class::Rel),
    def("downarrow", "↓", Class::Rel),
    def("mapsto", "↦", Class::Rel),
    def("rightleftharpoons", "⇌", Class::Rel),
    def("mp", "∓", Class::Bin),
    def("otimes", "⊗", Class::Bin),
    def("wedge", "∧", Class::Bin),
    def("vee", "∨", Class::Bin),
    def("circ", "∘", Class::Bin),
    def("ast", "∗", Class::Bin),
    def("star", "⋆", Class::Bin),
    def("bullet", "∙", Class::Bin),
    def("dagger", "†", Class::Bin),
    def("langle", "⟨", Class::Open),
    def("rangle", "⟩", Class::Close),
    // TeX's thin space `\,`: the six-per-em space (ADR-T36-1).
    def(",", "\u{2006}", Class::Ord),
    def("log", "log", Class::Op),
    def("ln", "ln", Class::Op),
    def("gcd", "gcd", Class::Op),
    def("lcm", "lcm", Class::Op),
];

/// The table entry of command `name` (without the backslash).
#[must_use]
pub fn symbol(name: &str) -> Option<&'static SymbolDef> {
    SYMBOLS.iter().find(|s| s.name == name)
}

/// Which embedded font draws a glyph (ADR-T33-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Face {
    /// The monospaced text font: plain text, literals, characters the
    /// math font lacks.
    #[default]
    Text,
    /// The math font: formulas.
    Math,
}

/// The math italic form of `ch`: Latin letters and lowercase Greek;
/// anything else as is (ADR-T33-1).
#[must_use]
pub fn italic(ch: char) -> char {
    let offset = |from: char, base: u32| char::from_u32(base + (u32::from(ch) - u32::from(from)));
    let mapped = match ch {
        // The italic h is the older Planck constant.
        'h' => Some('\u{210E}'),
        'a'..='z' => offset('a', 0x1D44E),
        'A'..='Z' => offset('A', 0x1D434),
        'α'..='ω' => offset('α', 0x1D6FC),
        'ϵ' => Some('\u{1D716}'),
        'ϑ' => Some('\u{1D717}'),
        'ϰ' => Some('\u{1D718}'),
        'ϕ' => Some('\u{1D719}'),
        'ϱ' => Some('\u{1D71A}'),
        'ϖ' => Some('\u{1D71B}'),
        _ => None,
    };
    mapped.unwrap_or(ch)
}

/// The script (`\mathcal`) form of `ch`: Latin capitals; anything else as
/// [`italic`] draws it (ADR-T33-1).
#[must_use]
pub fn calligraphic(ch: char) -> char {
    // Script capitals that Unicode had before the math block.
    let mapped = match ch {
        'B' => Some('\u{212C}'),
        'E' => Some('\u{2130}'),
        'F' => Some('\u{2131}'),
        'H' => Some('\u{210B}'),
        'I' => Some('\u{2110}'),
        'L' => Some('\u{2112}'),
        'M' => Some('\u{2133}'),
        'R' => Some('\u{211B}'),
        'A'..='Z' => char::from_u32(0x1D49C + (u32::from(ch) - u32::from('A'))),
        _ => None,
    };
    mapped.unwrap_or_else(|| italic(ch))
}

/// How the letters of a formula part are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    /// Math italic.
    Italic,
    /// `\mathcal`: script capitals.
    Calligraphic,
}

impl Style {
    /// `ch` typed in a formula part of this style.
    fn map(self, ch: char) -> char {
        match self {
            Self::Italic => italic(ch),
            Self::Calligraphic => calligraphic(ch),
        }
    }
}

/// Something to draw, in em from its box's origin, y down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ink {
    /// A glyph of one of the embedded fonts.
    Glyph {
        /// The character.
        ch: char,
        /// The font that draws it.
        face: Face,
        /// Left end of its baseline.
        pos: Vec2,
        /// Its size relative to the block's font size.
        scale: f32,
        /// Drawn in the warning colour: source outside the subset.
        warn: bool,
    },
    /// A straight stroke: fraction bar or part of a root sign.
    Rule {
        /// One end.
        a: Vec2,
        /// The other end.
        b: Vec2,
        /// Stroke width.
        width: f32,
    },
}

impl Ink {
    /// The box the ink covers: a text-face glyph's cell (its advance by
    /// its ascent and descent), a math-face glyph's advance and ink box
    /// together, a rule's segment grown by half its width.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        match *self {
            Self::Glyph {
                ch,
                face,
                pos,
                scale,
                ..
            } => {
                if let Some(m) = math_glyph_metrics(face, ch) {
                    return Aabb::from_corners(
                        Vec2::new(pos.x + m.x_min.min(0.0) * scale, pos.y - m.y_max * scale),
                        Vec2::new(
                            pos.x + m.advance.max(m.x_max) * scale,
                            pos.y - m.y_min * scale,
                        ),
                    );
                }
                let (ascent, descent) = glyph_extent(ch);
                Aabb::from_corners(
                    Vec2::new(pos.x, pos.y - ascent * scale),
                    Vec2::new(pos.x + ADVANCE * scale, pos.y + descent * scale),
                )
            }
            Self::Rule { a, b, width } => Aabb::from_corners(a, b).expand(width / 2.0),
        }
    }

    /// The ink moved by `by`.
    fn shifted(self, by: Vec2) -> Self {
        match self {
            Self::Glyph {
                ch,
                face,
                pos,
                scale,
                warn,
            } => Self::Glyph {
                ch,
                face,
                pos: pos + by,
                scale,
                warn,
            },
            Self::Rule { a, b, width } => Self::Rule {
                a: a + by,
                b: b + by,
                width,
            },
        }
    }
}

/// A laid-out formula: its ink around a baseline that starts at the
/// origin.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MathBox {
    /// Advance width.
    pub width: f32,
    /// Height above the baseline.
    pub ascent: f32,
    /// Depth below the baseline.
    pub descent: f32,
    /// Italic correction of its last glyph: room a superscript leaves.
    pub italic: f32,
    /// What to draw, relative to the left end of the baseline.
    pub ink: Vec<Ink>,
}

impl MathBox {
    /// One text-face glyph at `scale`: a monospaced cell.
    fn glyph(ch: char, scale: f32, warn: bool) -> Self {
        let (ascent, descent) = glyph_extent(ch);
        Self {
            width: ADVANCE * scale,
            ascent: ascent * scale,
            descent: descent * scale,
            italic: 0.0,
            ink: vec![Ink::Glyph {
                ch,
                face: Face::Text,
                pos: Vec2::ZERO,
                scale,
                warn,
            }],
        }
    }

    /// One math-face glyph at `scale` with its metrics; a text-face cell
    /// when the math font lacks `ch` (ADR-T33-1).
    fn math(ch: char, scale: f32) -> Self {
        let Some(m) = math_metrics::get(ch) else {
            return Self::glyph(ch, scale, false);
        };
        Self {
            width: m.advance * scale,
            ascent: m.y_max * scale,
            descent: -m.y_min * scale,
            italic: m.italic * scale,
            ink: vec![Ink::Glyph {
                ch,
                face: Face::Math,
                pos: Vec2::ZERO,
                scale,
                warn: false,
            }],
        }
    }

    /// Empty space `width` wide.
    fn gap(width: f32) -> Self {
        Self {
            width,
            ..Self::default()
        }
    }

    /// Adds the ink of `other` with its baseline origin at `at`.
    fn put(&mut self, other: &Self, at: Vec2) {
        self.ink.extend(other.ink.iter().map(|ink| ink.shifted(at)));
    }

    /// `boxes` side by side on one baseline; the italic correction is the
    /// last box's.
    fn row(boxes: impl IntoIterator<Item = Self>) -> Self {
        let mut out = Self::default();
        for b in boxes {
            out.put(&b, Vec2::new(out.width, 0.0));
            out.width += b.width;
            out.ascent = out.ascent.max(b.ascent);
            out.descent = out.descent.max(b.descent);
            out.italic = b.italic;
        }
        out
    }

    /// The characters of `text` side by side in the text face, `-` as
    /// typed.
    fn word(text: &str, scale: f32, warn: bool) -> Self {
        Self::row(text.chars().map(|ch| Self::glyph(ch, scale, warn)))
    }

    /// The characters of `text` side by side, upright in the math face.
    fn upright(text: &str, scale: f32) -> Self {
        Self::row(text.chars().map(|ch| Self::math(ch, scale)))
    }

    /// Leftmost and rightmost x of its ink; `(0, 0)` without ink.
    fn ink_span(&self) -> (f32, f32) {
        let mut span: Option<(f32, f32)> = None;
        for ink in &self.ink {
            let b = ink.bounds();
            span = Some(span.map_or((b.min.x, b.max.x), |(lo, hi)| {
                (lo.min(b.min.x), hi.max(b.max.x))
            }));
        }
        span.unwrap_or((0.0, 0.0))
    }
}

/// A laid-out block: its ink relative to its top-left corner.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    /// Width of the widest line.
    pub width: f32,
    /// Sum of the line heights (at least one line).
    pub height: f32,
    /// What to draw.
    pub ink: Vec<Ink>,
}

impl Layout {
    /// `(0, 0)` to `(width, height)`.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        Aabb::from_corners(Vec2::ZERO, Vec2::new(self.width, self.height))
    }
}

/// The math metrics of a glyph of `face`; `None` for the text face or a
/// character the math font lacks.
fn math_glyph_metrics(face: Face, ch: char) -> Option<math_metrics::GlyphMetrics> {
    match face {
        Face::Math => math_metrics::get(ch),
        Face::Text => None,
    }
}

/// Ascent and descent of text-face glyph `ch` per unit of scale.
#[must_use]
pub fn glyph_extent(ch: char) -> (f32, f32) {
    if "()[]{}|⌊⌋⌈⌉∑∏√".contains(ch) {
        (TALL_ASCENT, TALL_DESCENT)
    } else {
        (GLYPH_ASCENT, GLYPH_DESCENT)
    }
}

// ---- parsing ----------------------------------------------------------------

/// Splits a block's text into plain text, `\$` and formulas, and parses
/// every formula. Total: every string parses.
#[must_use]
pub fn parse(text: &str) -> Vec<Segment> {
    let chars: Vec<char> = text.chars().collect();
    let mut segments = Vec::new();
    let mut plain = String::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' && chars.get(i + 1) == Some(&'$') {
            flush(&mut plain, &mut segments);
            segments.push(Segment::Dollar);
            i += 2;
        } else if let Some(close) = (ch == '$').then(|| closing_dollar(&chars, i)).flatten() {
            flush(&mut plain, &mut segments);
            segments.push(Segment::Math(Parser::new(&chars[i + 1..close]).parse()));
            i = close + 1;
        } else {
            plain.push(ch);
            i += 1;
        }
    }
    flush(&mut plain, &mut segments);
    segments
}

/// Moves pending plain text into `segments`.
fn flush(plain: &mut String, segments: &mut Vec<Segment>) {
    if !plain.is_empty() {
        segments.push(Segment::Text(std::mem::take(plain)));
    }
}

/// The index of the `$` closing the formula opened at `open`: the first
/// unescaped `$` on the same line, not right after `open`.
fn closing_dollar(chars: &[char], open: usize) -> Option<usize> {
    let mut j = open + 1;
    while let Some(&ch) = chars.get(j) {
        match ch {
            '\n' => return None,
            '$' => return (j > open + 1).then_some(j),
            '\\' if chars.get(j + 1).is_some_and(|&next| next != '\n') => j += 2,
            _ => j += 1,
        }
    }
    None
}

/// Recursive-descent parser of one formula.
struct Parser<'a> {
    /// The formula, without its dollars.
    chars: &'a [char],
    /// Next character to read.
    pos: usize,
    /// For each `{`, the index of its matching `}`, if it has one within
    /// [`MAX_DEPTH`].
    closes: Vec<Option<usize>>,
}

impl<'a> Parser<'a> {
    fn new(chars: &'a [char]) -> Self {
        Self {
            chars,
            pos: 0,
            closes: match_braces(chars),
        }
    }

    /// The whole formula.
    fn parse(mut self) -> Vec<Node> {
        self.list(self.chars.len())
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// The `}` matching a `{` at the cursor, if it lies before `end`.
    fn group_close(&self, end: usize) -> Option<usize> {
        self.closes
            .get(self.pos)
            .copied()
            .flatten()
            .filter(|&close| close < end)
    }

    /// Nodes up to `end`.
    fn list(&mut self, end: usize) -> Vec<Node> {
        let mut out: Vec<Node> = Vec::new();
        while self.pos < end {
            let ch = self.chars[self.pos];
            match ch {
                '^' | '_' => self.script(ch, end, &mut out),
                _ => {
                    let node = self.atom(end);
                    out.push(node);
                }
            }
        }
        out
    }

    /// One node that is not a script, at the cursor (which is before
    /// `end`).
    fn atom(&mut self, end: usize) -> Node {
        let ch = self.chars[self.pos];
        if ch.is_whitespace() {
            return Node::Space(self.whitespace(end));
        }
        if ch == '{' {
            if let Some(close) = self.group_close(end) {
                self.pos += 1;
                let nodes = self.list(close);
                self.pos = close + 1;
                return Node::Group(nodes);
            }
        }
        if ch == '\\' {
            return self.command(end);
        }
        self.pos += 1;
        match ch {
            '{' | '}' => Node::Literal(ch.to_string()),
            _ => Node::Char(ch),
        }
    }

    /// The whitespace run at the cursor.
    fn whitespace(&mut self, end: usize) -> String {
        let start = self.pos;
        while self.pos < end && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// A `^` or `_` at the cursor: attaches to the node before it, or is
    /// literal.
    fn script(&mut self, ch: char, end: usize, out: &mut Vec<Node>) {
        let kind = if ch == '^' {
            ScriptKind::Sup
        } else {
            ScriptKind::Sub
        };
        let full = matches!(out.last(), Some(Node::Scripts { first, second, .. })
            if second.is_some() || first.kind == kind);
        let start = self.pos;
        self.pos += 1;
        let arg = if full { None } else { self.arg(end) };
        let Some(arg) = arg else {
            self.pos = start + 1;
            out.push(Node::Literal(ch.to_string()));
            return;
        };
        let script = Script { kind, arg };
        match out.pop() {
            Some(Node::Scripts {
                base,
                first,
                second: None,
            }) => out.push(Node::Scripts {
                base,
                first,
                second: Some(script),
            }),
            Some(Node::Space(space)) => {
                out.push(Node::Space(space));
                out.push(Node::Scripts {
                    base: None,
                    first: script,
                    second: None,
                });
            }
            base => out.push(Node::Scripts {
                base: base.map(Box::new),
                first: script,
                second: None,
            }),
        }
    }

    /// A script or command argument at the cursor: a `{group}`, a symbol
    /// command or one character. `None` (cursor unchanged) when there is
    /// none.
    fn arg(&mut self, end: usize) -> Option<Arg> {
        let start = self.pos;
        let lead = self.whitespace(end);
        let arg = match self.peek().filter(|_| self.pos < end) {
            Some('{') => self.group_close(end).map(|close| {
                self.pos += 1;
                let nodes = self.list(close);
                self.pos = close + 1;
                (true, nodes)
            }),
            Some('\\') => {
                let name = self.name(end);
                symbol(&name).map(|_| (false, vec![Node::Symbol(name)]))
            }
            Some('}' | '^' | '_') | None => None,
            Some(ch) => {
                self.pos += 1;
                Some((false, vec![Node::Char(ch)]))
            }
        };
        if arg.is_none() {
            self.pos = start;
        }
        arg.map(|(braced, nodes)| Arg {
            lead,
            braced,
            nodes,
        })
    }

    /// Reads the command name after the `\` at the cursor: a run of ASCII
    /// letters or one other character; empty at the end.
    fn name(&mut self, end: usize) -> String {
        self.pos += 1;
        let start = self.pos;
        match self.chars.get(self.pos).filter(|_| self.pos < end) {
            Some(ch) if ch.is_ascii_alphabetic() => {
                while self.pos < end && self.chars[self.pos].is_ascii_alphabetic() {
                    self.pos += 1;
                }
            }
            Some(_) => self.pos += 1,
            None => {}
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// A command at the cursor (a `\`), with its arguments, or literal.
    fn command(&mut self, end: usize) -> Node {
        let name = self.name(end);
        if symbol(&name).is_some() {
            return Node::Symbol(name);
        }
        let after = self.pos;
        let node = match name.as_str() {
            "frac" | "binom" => self.args(2, end).map(|args| Node::Command {
                name: name.clone(),
                args,
            }),
            "sqrt" | "pmod" | "mathcal" => self.args(1, end).map(|args| Node::Command {
                name: name.clone(),
                args,
            }),
            "text" => self.text_body(end),
            _ => None,
        };
        node.unwrap_or_else(|| {
            self.pos = after;
            Node::Literal(format!("\\{name}"))
        })
    }

    /// `n` arguments, or `None` if one is missing.
    fn args(&mut self, n: usize, end: usize) -> Option<Vec<Arg>> {
        (0..n).map(|_| self.arg(end)).collect()
    }

    /// The `{…}` body of `\text`, raw.
    fn text_body(&mut self, end: usize) -> Option<Node> {
        let lead = self.whitespace(end);
        let close = self.group_close(end)?;
        let body = self.chars[self.pos + 1..close].iter().collect();
        self.pos = close + 1;
        Some(Node::Text { lead, body })
    }
}

/// For each `{` of `chars`, the index of its matching `}`; escaped braces
/// (`\{`) are skipped and braces nested deeper than [`MAX_DEPTH`] match
/// nothing.
fn match_braces(chars: &[char]) -> Vec<Option<usize>> {
    let mut closes = vec![None; chars.len()];
    // Open braces: index and whether it is within the depth limit.
    let mut open: Vec<(usize, bool)> = Vec::new();
    let mut depth = 0;
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 1,
            '{' => {
                let within = depth < MAX_DEPTH;
                depth += usize::from(within);
                open.push((i, within));
            }
            '}' => {
                if let Some((start, within)) = open.pop() {
                    if within {
                        closes[start] = Some(i);
                        depth -= 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    closes
}

// ---- printing ---------------------------------------------------------------

/// The source of `segments`: `print(&parse(s)) == s` for every `s`.
#[must_use]
pub fn print(segments: &[Segment]) -> String {
    let mut out = String::new();
    for segment in segments {
        match segment {
            Segment::Text(text) => out.push_str(text),
            Segment::Dollar => out.push_str("\\$"),
            Segment::Math(nodes) => {
                out.push('$');
                print_nodes(nodes, &mut out);
                out.push('$');
            }
        }
    }
    out
}

fn print_nodes(nodes: &[Node], out: &mut String) {
    for node in nodes {
        print_node(node, out);
    }
}

fn print_node(node: &Node, out: &mut String) {
    match node {
        Node::Space(text) | Node::Literal(text) => out.push_str(text),
        Node::Char(ch) => out.push(*ch),
        Node::Symbol(name) => {
            out.push('\\');
            out.push_str(name);
        }
        Node::Group(nodes) => {
            out.push('{');
            print_nodes(nodes, out);
            out.push('}');
        }
        Node::Scripts {
            base,
            first,
            second,
        } => {
            if let Some(base) = base {
                print_node(base, out);
            }
            for script in std::iter::once(first).chain(second) {
                out.push(match script.kind {
                    ScriptKind::Sup => '^',
                    ScriptKind::Sub => '_',
                });
                print_arg(&script.arg, out);
            }
        }
        Node::Command { name, args } => {
            out.push('\\');
            out.push_str(name);
            for arg in args {
                print_arg(arg, out);
            }
        }
        Node::Text { lead, body } => {
            out.push_str("\\text");
            out.push_str(lead);
            out.push('{');
            out.push_str(body);
            out.push('}');
        }
    }
}

fn print_arg(arg: &Arg, out: &mut String) {
    out.push_str(&arg.lead);
    if arg.braced {
        out.push('{');
        print_nodes(&arg.nodes, out);
        out.push('}');
    } else {
        print_nodes(&arg.nodes, out);
    }
}

// ---- layout -----------------------------------------------------------------

/// Scale of a script of something at `scale`.
fn script_scale(scale: f32) -> f32 {
    (scale * SCRIPT_SCALE).max(MIN_SCALE).min(scale)
}

/// Lays out the nodes of one formula at `scale` (1 for the block's font
/// size).
#[must_use]
pub fn layout_math(nodes: &[Node], scale: f32) -> MathBox {
    layout_styled(nodes, scale, Style::Italic)
}

/// [`layout_math`] with letters drawn in `style`.
fn layout_styled(nodes: &[Node], scale: f32, style: Style) -> MathBox {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let mut atoms: Vec<(MathBox, Class)> = nodes
        .iter()
        .filter_map(|node| atom(node, scale, style))
        .collect();
    // A binary operator without an operand on both sides is ordinary.
    for i in 0..atoms.len() {
        if atoms[i].1 != Class::Bin {
            continue;
        }
        let before = i.checked_sub(1).map(|j| atoms[j].1);
        let after = atoms.get(i + 1).map(|a| a.1);
        let unary = matches!(
            before,
            None | Some(
                Class::Bin
                    | Class::Op
                    | Class::Limits
                    | Class::BigOp
                    | Class::Integral
                    | Class::Rel
                    | Class::Open
                    | Class::Punct
            )
        ) || matches!(after, None | Some(Class::Rel | Class::Close | Class::Punct));
        if unary {
            atoms[i].1 = Class::Ord;
        }
    }
    let mut boxes = Vec::with_capacity(atoms.len() * 2);
    let mut prev: Option<Class> = None;
    for (b, class) in atoms {
        if let Some(prev) = prev {
            let space = spacing(prev, class, scale);
            if space > 0.0 {
                boxes.push(MathBox::gap(space));
            }
        }
        prev = Some(class);
        boxes.push(b);
    }
    MathBox::row(boxes)
}

/// Space between neighbouring atoms of classes `a` and `b` at `scale`.
fn spacing(a: Class, b: Class, scale: f32) -> f32 {
    let op = |c| {
        matches!(
            c,
            Class::Op | Class::Limits | Class::BigOp | Class::Integral
        )
    };
    // TeX's thin space around operators, in scripts too.
    if (op(a) && (b == Class::Ord || op(b))) || (op(b) && matches!(a, Class::Ord | Class::Close)) {
        return OP_SPACE * scale;
    }
    // Punctuation, relation and operator space only at the block's own
    // size.
    if scale < 1.0 {
        return 0.0;
    }
    if a == Class::Punct {
        return OP_SPACE * scale;
    }
    if (a == Class::Rel) != (b == Class::Rel)
        && a != Class::Open
        && !matches!(b, Class::Close | Class::Punct)
    {
        return REL_SPACE * scale;
    }
    if a == Class::Bin || b == Class::Bin {
        return BIN_SPACE * scale;
    }
    0.0
}

/// Class of a character typed in a formula.
fn char_class(ch: char) -> Class {
    match ch {
        '=' | '<' | '>' | ':' => Class::Rel,
        '+' | '-' | '*' => Class::Bin,
        '(' | '[' => Class::Open,
        ')' | ']' => Class::Close,
        ',' | ';' => Class::Punct,
        _ => Class::Ord,
    }
}

/// The box and class of one node with letters in `style`; `None` for
/// whitespace.
fn atom(node: &Node, scale: f32, style: Style) -> Option<(MathBox, Class)> {
    let atom = match node {
        Node::Space(_) => return None,
        Node::Char(ch) => {
            let shown = if *ch == '-' { MINUS } else { style.map(*ch) };
            (MathBox::math(shown, scale), char_class(*ch))
        }
        Node::Symbol(name) => match symbol(name) {
            Some(def) => (symbol_box(def, scale), def.class),
            None => (MathBox::word(&format!("\\{name}"), scale, true), Class::Ord),
        },
        Node::Group(nodes) => (layout_styled(nodes, scale, style), Class::Ord),
        Node::Scripts {
            base,
            first,
            second,
        } => {
            let (base, class) = base
                .as_deref()
                .and_then(|b| atom(b, scale, style))
                .unwrap_or((MathBox::default(), Class::Ord));
            let ss = script_scale(scale);
            let (mut sup, mut sub) = (None, None);
            for script in std::iter::once(first).chain(second) {
                let b = layout_styled(&script.arg.nodes, ss, style);
                match script.kind {
                    ScriptKind::Sup => sup = Some(b),
                    ScriptKind::Sub => sub = Some(b),
                }
            }
            let b = match class {
                Class::BigOp | Class::Limits => limits(&base, sup, sub, scale),
                // The subscript tucks under the slant (ADR-T36-1).
                Class::Integral => {
                    let tuck = base.italic;
                    scripts(base, sup, sub, tuck, scale)
                }
                _ => scripts(base, sup, sub, 0.0, scale),
            };
            (b, class)
        }
        Node::Command { name, args } => {
            let arg = |i: usize| args.get(i).map_or(&[][..], |a: &Arg| &a.nodes[..]);
            let b = match name.as_str() {
                "frac" => fraction(arg(0), arg(1), scale, true, style),
                "binom" => binomial(arg(0), arg(1), scale, style),
                "sqrt" => root(arg(0), scale, style),
                "mathcal" => layout_styled(arg(0), scale, Style::Calligraphic),
                // TeX's \pmod: a quad, "(mod", a third of a quad.
                "pmod" => MathBox::row([
                    MathBox::gap(scale),
                    MathBox::upright("(mod", scale),
                    MathBox::gap(scale / 3.0),
                    layout_styled(arg(0), scale, style),
                    MathBox::math(')', scale),
                ]),
                _ => MathBox::word(&format!("\\{name}"), scale, true),
            };
            (b, Class::Ord)
        }
        Node::Text { body, .. } => (MathBox::upright(body, scale), Class::Ord),
        Node::Literal(text) => (MathBox::word(text, scale, true), Class::Ord),
    };
    Some(atom)
}

/// The box of a table symbol: a glyph (lowercase Greek in italic), an
/// enlarged big operator or integral, or an upright word.
fn symbol_box(def: &SymbolDef, scale: f32) -> MathBox {
    let mut chars = def.text.chars();
    match (chars.next(), chars.next(), def.class) {
        (Some(ch), None, Class::BigOp | Class::Integral) => big_operator(ch, scale),
        (Some(ch), None, _) => MathBox::math(italic(ch), scale),
        _ => MathBox::upright(def.text, scale),
    }
}

/// `∑`, `∏` or `∫`, enlarged and centred on the axis, with the enlarged
/// glyph's italic correction.
fn big_operator(ch: char, scale: f32) -> MathBox {
    let k = BIG_OP_SCALE * scale;
    let glyph = MathBox::math(ch, k);
    // The glyph's middle sits on the axis.
    let y = -AXIS * scale - (glyph.descent - glyph.ascent) / 2.0;
    let mut out = MathBox {
        width: glyph.width,
        ascent: glyph.ascent - y,
        descent: glyph.descent + y,
        italic: glyph.italic,
        ink: Vec::new(),
    };
    out.put(&glyph, Vec2::new(0.0, y));
    out
}

/// Superscript and subscript to the right of `base` (ADR-T33-1): the
/// superscript after the base's italic correction, the subscript `tuck`
/// left of the base's advance (ADR-T36-1).
fn scripts(
    base: MathBox,
    sup: Option<MathBox>,
    sub: Option<MathBox>,
    tuck: f32,
    scale: f32,
) -> MathBox {
    let mut up = (SUP_SHIFT * scale).max(base.ascent - SUP_DROP * scale);
    if let Some(sup) = &sup {
        up = up.max(sup.descent + SUP_BOTTOM_MIN * scale);
    }
    let mut down = (SUB_SHIFT * scale).max(base.descent - SUB_DROP * scale);
    if let Some(sub) = &sub {
        down = down.max(sub.ascent - SUB_TOP_MAX * scale);
    }
    if let (Some(sup), Some(sub)) = (&sup, &sub) {
        let gap = (down - sub.ascent) - (sup.descent - up);
        if gap < SCRIPT_GAP * scale {
            down += SCRIPT_GAP * scale - gap;
        }
    }
    let x = base.width;
    let mut out = MathBox {
        width: base.width,
        ascent: base.ascent,
        descent: base.descent,
        italic: 0.0,
        ink: base.ink,
    };
    if let Some(sup) = sup {
        let at = x + base.italic;
        out.put(&sup, Vec2::new(at, -up));
        out.width = out.width.max(at + sup.width);
        out.ascent = out.ascent.max(up + sup.ascent);
        out.descent = out.descent.max(sup.descent - up);
    }
    if let Some(sub) = sub {
        let at = x - tuck;
        out.put(&sub, Vec2::new(at, down));
        out.width = out.width.max(at + sub.width);
        out.ascent = out.ascent.max(sub.ascent - down);
        out.descent = out.descent.max(down + sub.descent);
    }
    if out.width > x {
        out.width += SCRIPT_SPACE * scale;
    }
    out
}

/// Limits centred above and below operator `op`.
fn limits(op: &MathBox, sup: Option<MathBox>, sub: Option<MathBox>, scale: f32) -> MathBox {
    let gap = LIMIT_GAP * scale;
    let width = [&sup, &sub]
        .into_iter()
        .flatten()
        .fold(op.width, |w, b| w.max(b.width));
    let mut out = MathBox {
        width,
        ascent: op.ascent,
        descent: op.descent,
        italic: 0.0,
        ink: Vec::new(),
    };
    out.put(op, Vec2::new((width - op.width) / 2.0, 0.0));
    if let Some(sup) = sup {
        let y = -op.ascent - gap - sup.descent;
        out.put(&sup, Vec2::new((width - sup.width) / 2.0, y));
        out.ascent = out.ascent.max(sup.ascent - y);
    }
    if let Some(sub) = sub {
        let y = op.descent + gap + sub.ascent;
        out.put(&sub, Vec2::new((width - sub.width) / 2.0, y));
        out.descent = out.descent.max(y + sub.descent);
    }
    out
}

/// A fraction of `num` over `den`, with a bar on the axis when `bar`.
fn fraction(num: &[Node], den: &[Node], scale: f32, bar: bool, style: Style) -> MathBox {
    let ss = script_scale(scale);
    let (n, d) = (layout_styled(num, ss, style), layout_styled(den, ss, style));
    let pad = FRAC_PAD * scale;
    let width = n.width.max(d.width) + 2.0 * pad;
    let axis = -AXIS * scale;
    let half = RULE * scale / 2.0;
    let n_base = (axis - half - FRAC_GAP * scale - n.descent).min(-FRAC_NUM_SHIFT * scale);
    let d_base = (axis + half + FRAC_GAP * scale + d.ascent).max(FRAC_DEN_SHIFT * scale);
    let mut out = MathBox {
        width,
        ascent: (n.ascent - n_base).max(half - axis),
        descent: (d_base + d.descent).max(axis + half),
        italic: 0.0,
        ink: Vec::new(),
    };
    out.put(&n, Vec2::new((width - n.width) / 2.0, n_base));
    out.put(&d, Vec2::new((width - d.width) / 2.0, d_base));
    if bar {
        out.ink.push(Ink::Rule {
            a: Vec2::new(pad / 2.0, axis),
            b: Vec2::new(width - pad / 2.0, axis),
            width: RULE * scale,
        });
    }
    out
}

/// `n` over `k` without a bar, in parentheses as tall as both.
fn binomial(n: &[Node], k: &[Node], scale: f32, style: Style) -> MathBox {
    let inner = fraction(n, k, scale, false, style);
    let height = inner.ascent + inner.descent;
    let unit = MathBox::math('(', 1.0);
    let tall = (unit.ascent + unit.descent).max(f32::EPSILON);
    let paren = (height / tall).clamp(scale, BINOM_MAX_SCALE * scale);
    let (open, close) = (MathBox::math('(', paren), MathBox::math(')', paren));
    // The parentheses' middle sits on the middle of the stack.
    let middle = (inner.descent - inner.ascent) / 2.0;
    let y = middle + (open.ascent - open.descent) / 2.0;
    let mut out = MathBox {
        width: open.width + inner.width + close.width,
        ascent: inner.ascent.max(open.ascent - y),
        descent: inner.descent.max(open.descent + y),
        italic: 0.0,
        ink: Vec::new(),
    };
    out.put(&open, Vec2::new(0.0, y));
    out.put(&inner, Vec2::new(open.width, 0.0));
    out.put(&close, Vec2::new(open.width + inner.width, y));
    out
}

/// A root sign of four rules over `content`.
fn root(content: &[Node], scale: f32, style: Style) -> MathBox {
    let c = layout_styled(content, scale, style);
    let top = -(c.ascent + SQRT_GAP * scale);
    let bottom = c.descent;
    let at = |t: f32| top + (bottom - top) * t;
    let x = SQRT_SIGN * scale;
    let points = [
        Vec2::new(0.05 * scale, at(0.6)),
        Vec2::new(0.15 * scale, at(0.5)),
        Vec2::new(0.3 * scale, bottom),
        Vec2::new(0.5 * scale, top),
        Vec2::new(x + c.width + 0.05 * scale, top),
    ];
    let width = RULE * scale;
    let mut out = MathBox {
        width: points[4].x + 0.05 * scale,
        ascent: -top + width / 2.0,
        descent: bottom + width / 2.0,
        italic: 0.0,
        ink: Vec::new(),
    };
    out.put(&c, Vec2::new(x, 0.0));
    out.ink.extend(points.windows(2).map(|w| Ink::Rule {
        a: w[0],
        b: w[1],
        width,
    }));
    out
}

/// One line of a block being laid out.
struct Line {
    ink: Vec<Ink>,
    width: f32,
    ascent: f32,
    descent: f32,
}

impl Line {
    fn new() -> Self {
        Self {
            ink: Vec::new(),
            width: 0.0,
            ascent: BASELINE,
            descent: LINE_HEIGHT - BASELINE,
        }
    }

    /// A plain character in the next cell.
    fn cell(&mut self, ch: char) {
        self.ink.push(Ink::Glyph {
            ch,
            face: Face::Text,
            pos: Vec2::new(self.width, 0.0),
            scale: 1.0,
            warn: false,
        });
        self.width += ADVANCE;
    }
}

/// Lays out a parsed block: lines split on `\n`, plain characters in
/// their monospaced cells, formulas typeset; a line grows to fit its
/// formulas.
#[must_use]
pub fn layout(segments: &[Segment]) -> Layout {
    let mut lines = Vec::new();
    let mut line = Line::new();
    for segment in segments {
        match segment {
            Segment::Text(text) => {
                for ch in text.chars() {
                    if ch == '\n' {
                        lines.push(std::mem::replace(&mut line, Line::new()));
                    } else {
                        line.cell(ch);
                    }
                }
            }
            Segment::Dollar => line.cell('$'),
            Segment::Math(nodes) => {
                let b = layout_math(nodes, 1.0);
                // Italic overhangs stay inside the line (ADR-T33-1).
                let (left, right) = b.ink_span();
                let x = line.width + (-(line.width + left)).max(0.0);
                line.ink
                    .extend(b.ink.iter().map(|ink| ink.shifted(Vec2::new(x, 0.0))));
                line.width = (x + b.width).max(x + right);
                line.ascent = line.ascent.max(b.ascent + MATH_PAD);
                line.descent = line.descent.max(b.descent + MATH_PAD);
            }
        }
    }
    lines.push(line);
    let mut out = Layout::default();
    for line in lines {
        let baseline = Vec2::new(0.0, out.height + line.ascent);
        out.ink
            .extend(line.ink.into_iter().map(|ink| ink.shifted(baseline)));
        out.width = out.width.max(line.width);
        out.height += line.ascent + line.descent;
    }
    out
}

#[cfg(test)]
mod tests {
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

    // ---- AC-1 parse --------------------------------------------------------

    #[test]
    fn tex_parse_plain_text_is_one_text_segment() {
        assert_eq!(parse("a b\nc"), vec![Segment::Text("a b\nc".to_owned())]);
        assert_eq!(parse(""), Vec::<Segment>::new());
    }

    #[test]
    fn tex_parse_dollar_pair_is_a_formula() {
        // Act
        let segments = parse("x $a$ y");

        // Assert
        assert_eq!(
            segments,
            vec![
                Segment::Text("x ".to_owned()),
                Segment::Math(vec![Node::Char('a')]),
                Segment::Text(" y".to_owned()),
            ]
        );
    }

    #[test]
    fn tex_parse_escaped_dollar_is_a_dollar_segment() {
        assert_eq!(
            parse("\\$5"),
            vec![Segment::Dollar, Segment::Text("5".to_owned())]
        );
        // Inside a formula `\$` does not close it.
        assert_eq!(math("\\$"), vec![sym("$")]);
    }

    #[test]
    fn tex_parse_unclosed_or_empty_dollar_is_text() {
        assert_eq!(
            parse("costs $5"),
            vec![Segment::Text("costs $5".to_owned())]
        );
        assert_eq!(parse("$$"), vec![Segment::Text("$$".to_owned())]);
        assert_eq!(
            parse("$$x$"),
            vec![
                Segment::Text("$".to_owned()),
                Segment::Math(vec![Node::Char('x')]),
            ]
        );
    }

    #[test]
    fn tex_parse_formula_does_not_cross_a_line_break() {
        assert_eq!(parse("$a\nb$"), vec![Segment::Text("$a\nb$".to_owned())]);
        assert_eq!(
            parse("$a\n$b$"),
            vec![
                Segment::Text("$a\n".to_owned()),
                Segment::Math(vec![Node::Char('b')]),
            ]
        );
    }

    #[test]
    fn tex_parse_scripts_bind_to_the_previous_atom() {
        // Act / Assert: both scripts on one base, in either order.
        assert_eq!(
            math("x_i^2"),
            vec![Node::Scripts {
                base: Some(Box::new(Node::Char('x'))),
                first: script(ScriptKind::Sub, tok(Node::Char('i'))),
                second: Some(script(ScriptKind::Sup, tok(Node::Char('2')))),
            }]
        );
        // Whitespace before `^` leaves it without a base.
        assert_eq!(
            math("x ^2"),
            vec![
                Node::Char('x'),
                Node::Space(" ".to_owned()),
                Node::Scripts {
                    base: None,
                    first: script(ScriptKind::Sup, tok(Node::Char('2'))),
                    second: None,
                },
            ]
        );
    }

    #[test]
    fn tex_parse_script_takes_one_token_or_a_group() {
        let sup = |arg| Node::Scripts {
            base: Some(Box::new(Node::Char('x'))),
            first: script(ScriptKind::Sup, arg),
            second: None,
        };

        assert_eq!(
            math("x^ab"),
            vec![sup(tok(Node::Char('a'))), Node::Char('b')]
        );
        assert_eq!(
            math("x^{ab}"),
            vec![sup(braced(vec![Node::Char('a'), Node::Char('b')]))]
        );
        assert_eq!(math("x^\\alpha"), vec![sup(tok(sym("alpha")))]);
    }

    #[test]
    fn tex_parse_commands_take_braced_or_single_token_arguments() {
        assert_eq!(
            math("\\frac{a}{b}"),
            vec![Node::Command {
                name: "frac".to_owned(),
                args: vec![braced(vec![Node::Char('a')]), braced(vec![Node::Char('b')]),],
            }]
        );
        assert_eq!(
            math("\\frac12"),
            vec![Node::Command {
                name: "frac".to_owned(),
                args: vec![tok(Node::Char('1')), tok(Node::Char('2'))],
            }]
        );
        assert_eq!(
            math("\\sqrt {x}"),
            vec![Node::Command {
                name: "sqrt".to_owned(),
                args: vec![Arg {
                    lead: " ".to_owned(),
                    braced: true,
                    nodes: vec![Node::Char('x')],
                }],
            }]
        );
    }

    #[test]
    fn tex_parse_mathcal_takes_one_argument() {
        assert_eq!(
            math("\\mathcal{A}"),
            vec![Node::Command {
                name: "mathcal".to_owned(),
                args: vec![braced(vec![Node::Char('A')])],
            }]
        );
        assert_eq!(
            math("\\mathcal L"),
            vec![Node::Command {
                name: "mathcal".to_owned(),
                args: vec![Arg {
                    lead: " ".to_owned(),
                    braced: false,
                    nodes: vec![Node::Char('L')],
                }],
            }]
        );
        assert_eq!(
            math("\\mathcal"),
            vec![Node::Literal("\\mathcal".to_owned())]
        );
    }

    #[test]
    fn tex_parse_accents_take_one_argument() {
        for def in ACCENTS {
            // Act
            let braced_arg = math(&format!("\\{}{{x}}", def.name));
            let token_arg = math(&format!("\\{} \\alpha", def.name));

            // Assert
            assert_eq!(
                braced_arg,
                vec![Node::Command {
                    name: def.name.to_owned(),
                    args: vec![braced(vec![Node::Char('x')])],
                }],
                "{}",
                def.name
            );
            assert_eq!(
                token_arg,
                vec![Node::Command {
                    name: def.name.to_owned(),
                    args: vec![Arg {
                        lead: " ".to_owned(),
                        braced: false,
                        nodes: vec![sym("alpha")],
                    }],
                }],
                "{}",
                def.name
            );
            assert!(symbol(def.name).is_none(), "{} is also a symbol", def.name);
            let rows = ACCENTS.iter().filter(|a| a.name == def.name).count();
            assert_eq!(rows, 1, "{} has one row", def.name);
            assert_eq!(accent(def.name), Some(def));
        }
        let names: Vec<&str> = ACCENTS.iter().map(|a| a.name).collect();
        assert_eq!(
            names,
            ["vec", "hat", "tilde", "dot", "ddot", "bar", "overline"]
        );
    }

    #[test]
    fn tex_parse_accent_without_argument_is_literal() {
        assert_eq!(math("\\vecx"), vec![Node::Literal("\\vecx".to_owned())]);
        assert_eq!(math("\\vec"), vec![Node::Literal("\\vec".to_owned())]);
        assert_eq!(
            math("\\hat^2"),
            vec![Node::Scripts {
                base: Some(Box::new(Node::Literal("\\hat".to_owned()))),
                first: script(ScriptKind::Sup, tok(Node::Char('2'))),
                second: None,
            }]
        );
    }

    #[test]
    fn tex_parse_symbols_come_from_the_table() {
        // Act
        let nodes = math("\\alpha\\le\\sum");

        // Assert
        assert_eq!(nodes, vec![sym("alpha"), sym("le"), sym("sum")]);
        let def = |name| symbol(name).map(|s| (s.text, s.class));
        assert_eq!(def("alpha"), Some(("α", Class::Ord)));
        assert_eq!(def("le"), Some(("≤", Class::Rel)));
        assert_eq!(def("times"), Some(("×", Class::Bin)));
        assert_eq!(def("log"), Some(("log", Class::Op)));
        assert_eq!(def("sum"), Some(("∑", Class::BigOp)));
        assert_eq!(def("min"), Some(("min", Class::Limits)));
        assert_eq!(def("lfloor"), Some(("⌊", Class::Open)));
        assert_eq!(def("foo"), None);
        let mut names: Vec<&str> = SYMBOLS.iter().map(|s| s.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), SYMBOLS.len(), "symbol names are unique");
    }

    #[test]
    fn tex_parse_integrals_are_symbols() {
        // Act
        let nodes = math("\\int\\iint\\iiint\\oint\\oiint");

        // Assert
        let names = ["int", "iint", "iiint", "oint", "oiint"];
        assert_eq!(nodes, names.map(sym).to_vec());
        let texts = names.map(|name| symbol(name).map(|s| (s.text, s.class)));
        assert_eq!(
            texts,
            ["∫", "∬", "∭", "∮", "∯"].map(|t| Some((t, Class::Integral)))
        );
        assert_eq!(math("\\intx"), vec![Node::Literal("\\intx".to_owned())]);
    }

    #[test]
    fn tex_parse_physics_symbols_have_their_text_and_class() {
        // Arrange: the *Physics symbols* table of T36, and `\,`.
        let table = [
            ("partial", "∂", Class::Ord),
            ("nabla", "∇", Class::Ord),
            ("hbar", "ℏ", Class::Ord),
            ("ell", "ℓ", Class::Ord),
            ("Re", "ℜ", Class::Ord),
            ("Im", "ℑ", Class::Ord),
            ("AA", "Å", Class::Ord),
            ("degree", "°", Class::Ord),
            ("forall", "∀", Class::Ord),
            ("exists", "∃", Class::Ord),
            ("emptyset", "∅", Class::Ord),
            ("prime", "′", Class::Ord),
            ("vdots", "⋮", Class::Ord),
            ("ddots", "⋱", Class::Ord),
            ("vert", "|", Class::Ord),
            ("Vert", "‖", Class::Ord),
            ("propto", "∝", Class::Rel),
            ("sim", "∼", Class::Rel),
            ("simeq", "≃", Class::Rel),
            ("cong", "≅", Class::Rel),
            ("ll", "≪", Class::Rel),
            ("gg", "≫", Class::Rel),
            ("lesssim", "≲", Class::Rel),
            ("gtrsim", "≳", Class::Rel),
            ("doteq", "≐", Class::Rel),
            ("coloneqq", "≔", Class::Rel),
            ("parallel", "∥", Class::Rel),
            ("perp", "⊥", Class::Rel),
            ("mid", "∣", Class::Rel),
            ("rightarrow", "→", Class::Rel),
            ("leftrightarrow", "↔", Class::Rel),
            ("Leftarrow", "⇐", Class::Rel),
            ("Leftrightarrow", "⇔", Class::Rel),
            ("uparrow", "↑", Class::Rel),
            ("downarrow", "↓", Class::Rel),
            ("mapsto", "↦", Class::Rel),
            ("rightleftharpoons", "⇌", Class::Rel),
            ("mp", "∓", Class::Bin),
            ("otimes", "⊗", Class::Bin),
            ("wedge", "∧", Class::Bin),
            ("vee", "∨", Class::Bin),
            ("circ", "∘", Class::Bin),
            ("ast", "∗", Class::Bin),
            ("star", "⋆", Class::Bin),
            ("bullet", "∙", Class::Bin),
            ("dagger", "†", Class::Bin),
            ("langle", "⟨", Class::Open),
            ("rangle", "⟩", Class::Close),
            (",", "\u{2006}", Class::Ord),
        ];

        for (name, text, class) in table {
            // Act
            let nodes = math(&format!("\\{name}"));

            // Assert
            assert_eq!(nodes, vec![sym(name)], "{name}");
            assert_eq!(
                symbol(name).map(|s| (s.text, s.class)),
                Some((text, class)),
                "{name}"
            );
        }
    }

    #[test]
    fn tex_parse_text_command_keeps_its_raw_body() {
        assert_eq!(
            math("\\text{a {b} c}"),
            vec![Node::Text {
                lead: String::new(),
                body: "a {b} c".to_owned(),
            }]
        );
    }

    #[test]
    fn tex_parse_unknown_command_is_literal() {
        assert_eq!(
            math("\\foo x"),
            vec![
                Node::Literal("\\foo".to_owned()),
                Node::Space(" ".to_owned()),
                Node::Char('x'),
            ]
        );
        // A control symbol outside the table; a `\` before the closing
        // `$` would escape it instead.
        assert_eq!(
            math("\\;x"),
            vec![Node::Literal("\\;".to_owned()), Node::Char('x')]
        );
    }

    #[test]
    fn tex_parse_unbalanced_braces_are_literal() {
        assert_eq!(
            math("{a"),
            vec![Node::Literal("{".to_owned()), Node::Char('a')]
        );
        assert_eq!(
            math("a}"),
            vec![Node::Char('a'), Node::Literal("}".to_owned())]
        );
        assert_eq!(math("{a}"), vec![Node::Group(vec![Node::Char('a')])]);
    }

    #[test]
    fn tex_parse_deep_nesting_is_literal_past_the_limit() {
        // Arrange: far deeper than any formula, deeper than the stack allows
        // for naive recursion.
        let src = format!("${}x{}$", "{".repeat(100_000), "}".repeat(100_000));

        // Act
        let segments = parse(&src);

        // Assert
        assert_eq!(print(&segments), src);
        let l = layout(&segments);
        assert!(l.width.is_finite() && l.height.is_finite());
    }

    #[test]
    fn tex_parse_missing_argument_is_literal() {
        assert_eq!(
            math("x^"),
            vec![Node::Char('x'), Node::Literal("^".to_owned())]
        );
        assert_eq!(
            math("\\frac{a}"),
            vec![
                Node::Literal("\\frac".to_owned()),
                Node::Group(vec![Node::Char('a')]),
            ]
        );
        assert_eq!(math("\\sqrt"), vec![Node::Literal("\\sqrt".to_owned())]);
        assert_eq!(
            math("\\text x"),
            vec![
                Node::Literal("\\text".to_owned()),
                Node::Space(" ".to_owned()),
                Node::Char('x'),
            ]
        );
    }

    #[test]
    fn tex_parse_double_superscript_is_literal() {
        assert_eq!(
            math("x^1^2"),
            vec![
                Node::Scripts {
                    base: Some(Box::new(Node::Char('x'))),
                    first: script(ScriptKind::Sup, tok(Node::Char('1'))),
                    second: None,
                },
                Node::Literal("^".to_owned()),
                Node::Char('2'),
            ]
        );
        assert_eq!(
            math("x_1^2_3"),
            vec![
                Node::Scripts {
                    base: Some(Box::new(Node::Char('x'))),
                    first: script(ScriptKind::Sub, tok(Node::Char('1'))),
                    second: Some(script(ScriptKind::Sup, tok(Node::Char('2')))),
                },
                Node::Literal("_".to_owned()),
                Node::Char('3'),
            ]
        );
    }

    /// Text built from formula-ish tokens and arbitrary characters.
    fn arb_source() -> impl Strategy<Value = String> {
        let token = prop_oneof![
            4 => prop::sample::select(vec![
                "$", "$", "\\$", "{", "}", "^", "_", "\\", " ", "\n", "x", "1", "-", "=",
                "\\frac", "\\sqrt", "\\binom", "\\text", "\\pmod", "\\alpha", "\\sum",
                "\\min", "\\log", "\\le", "\\{", "\\}", "\\foo", "(", ")", "|",
                "\\mathcal", "f", "j", "\\text{日}", "\\int", "\\oint", "\\,",
                "\\partial", "\\vec", "\\hat", "\\bar", "\\dot", "\\ddot", "\\tilde",
                "\\overline",
            ])
            .prop_map(str::to_owned),
            1 => any::<char>().prop_map(String::from),
        ];
        prop::collection::vec(token, 0..60).prop_map(|tokens| tokens.concat())
    }

    proptest! {
        #[test]
        fn tex_parse_never_panics_and_prints_back_the_source(src in arb_source()) {
            // Act
            let segments = parse(&src);

            // Assert
            prop_assert_eq!(print(&segments), src);
        }

        #[test]
        fn tex_layout_bounds_contain_every_ink(src in arb_source()) {
            // Act
            let l = layout(&parse(&src));

            // Assert
            let b = l.bounds().expand(1e-3);
            for ink in &l.ink {
                let ib = ink.bounds();
                prop_assert!(b.contains(ib.min) && b.contains(ib.max), "{ink:?} {b:?}");
            }
        }

        #[test]
        fn tex_layout_is_finite(src in arb_source()) {
            // Act
            let l = layout(&parse(&src));

            // Assert
            prop_assert!(l.width.is_finite() && l.width >= 0.0);
            prop_assert!(l.height.is_finite() && l.height > 0.0);
            for ink in &l.ink {
                let ib = ink.bounds();
                prop_assert!(ib.min.is_finite() && ib.max.is_finite(), "{ink:?}");
            }
        }
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

    #[test]
    fn tex_layout_char_uses_its_glyph_metrics() {
        // Act
        let b = boxed("x");

        // Assert
        let x = mm(X);
        assert!(approx_eq(b.width, x.advance, EPS));
        assert!(approx_eq(b.ascent, x.y_max, EPS));
        assert!(approx_eq(b.descent, -x.y_min, EPS));
        assert_eq!(glyphs(&b.ink), vec![(X, v(0.0, 0.0), 1.0, false)]);
        assert_eq!(faces(&b.ink), vec![(X, Face::Math)]);
    }

    #[test]
    fn tex_layout_superscript_is_raised_and_scaled() {
        // Act
        let b = boxed("x^2");

        // Assert: TeX's rule — the larger of the shift, the drop from the
        // base's top and the script's bottom limit.
        let s = SCRIPT_SCALE;
        let up = SUP_SHIFT
            .max(mm(X).y_max - SUP_DROP)
            .max(-mm('2').y_min * s + SUP_BOTTOM_MIN);
        let (pos, scale) = glyph(&b.ink, '2');
        let x = mm(X).advance + mm(X).italic;
        assert!(pos.approx_eq(v(x, -up), EPS), "{pos:?}");
        assert!(approx_eq(scale, s, EPS));
        assert!(approx_eq(b.width, x + adv("2") * s + SCRIPT_SPACE, EPS));
        // A script of a script stops at the smallest scale.
        let (_, scale) = glyph(&boxed("x^{y^z}").ink, Z);
        assert!(approx_eq(scale, MIN_SCALE, EPS));
        assert!(glyphs(&boxed("x^{y^z}").ink).iter().any(|g| g.0 == Y));
    }

    #[test]
    fn tex_layout_subscript_is_lowered_and_scaled() {
        // Act
        let b = boxed("x_2");

        // Assert
        let s = SCRIPT_SCALE;
        let down = SUB_SHIFT
            .max(-mm(X).y_min - SUB_DROP)
            .max(mm('2').y_max * s - SUB_TOP_MAX);
        let (pos, scale) = glyph(&b.ink, '2');
        assert!(pos.approx_eq(v(mm(X).advance, down), EPS), "{pos:?}");
        assert!(approx_eq(scale, s, EPS));
    }

    #[test]
    fn tex_layout_subscript_top_stays_low() {
        // Arrange: a subscript with a superscript of its own is tall.
        let sub = layout_math(&math("b^2"), SCRIPT_SCALE);
        assert!(sub.ascent - SUB_SHIFT > SUB_TOP_MAX, "the case is tall");

        // Act
        let b = boxed("x_{b^2}");

        // Assert: its top is no higher than SUB_TOP_MAX.
        let (pos, _) = glyph(&b.ink, B);
        assert!(approx_eq(pos.y - sub.ascent, -SUB_TOP_MAX, EPS), "{pos:?}");
    }

    #[test]
    fn tex_layout_superscript_clears_the_italic_correction() {
        // Arrange
        let f = mm(F);
        assert!(f.italic > 0.0, "italic f has a correction");

        // Act
        let sup = boxed("f^2");
        let sub = boxed("f_2");

        // Assert: superscripts start after it, subscripts tuck under.
        let (p, _) = glyph(&sup.ink, '2');
        let (q, _) = glyph(&sub.ink, '2');
        assert!(approx_eq(p.x, f.advance + f.italic, EPS), "{p:?}");
        assert!(approx_eq(q.x, f.advance, EPS), "{q:?}");
    }

    #[test]
    fn tex_layout_sub_and_sup_share_the_base() {
        for src in ["x_i^2", "x^2_i"] {
            // Act
            let b = boxed(src);

            // Assert: both start at the base's right edge, apart by at
            // least SCRIPT_GAP.
            let (sup, s) = glyph(&b.ink, '2');
            let (sub, _) = glyph(&b.ink, I);
            let x = mm(X).advance;
            assert!(
                approx_eq(sup.x, x + mm(X).italic, EPS) && approx_eq(sub.x, x, EPS),
                "{src}"
            );
            let sup_bottom = sup.y - mm('2').y_min * s;
            let sub_top = sub.y - mm(I).y_max * s;
            assert!(
                sub_top - sup_bottom >= SCRIPT_GAP - EPS,
                "{src}: {sup:?} {sub:?}"
            );
            let scripts = adv("2").max(mm(I).advance) * s;
            assert!(approx_eq(b.width, x + scripts + SCRIPT_SPACE, EPS));
        }
    }

    #[test]
    fn tex_layout_frac_centres_parts_around_a_bar_on_the_axis() {
        // Act
        let b = boxed("\\frac{a}{b}");

        // Assert: one bar on the axis, numerator above, denominator below.
        let bars = rules(&b.ink);
        assert_eq!(bars.len(), 1);
        let (p, q, width) = bars[0];
        assert!(approx_eq(p.y, -AXIS, EPS) && approx_eq(q.y, -AXIS, EPS));
        assert!(approx_eq(width, RULE, EPS));
        assert!(p.x >= 0.0 && q.x <= b.width && q.x - p.x >= mm(B).advance * SCRIPT_SCALE);
        let (num, ns) = glyph(&b.ink, A);
        let (den, ds) = glyph(&b.ink, B);
        assert!(approx_eq(ns, SCRIPT_SCALE, EPS) && approx_eq(ds, SCRIPT_SCALE, EPS));
        assert!(num.y - mm(A).y_min * ns <= -AXIS - RULE / 2.0 - FRAC_GAP + EPS);
        assert!(den.y - mm(B).y_max * ds >= -AXIS + RULE / 2.0 + FRAC_GAP - EPS);
        assert!(num.y <= -FRAC_NUM_SHIFT + EPS && den.y >= FRAC_DEN_SHIFT - EPS);
        let middle = b.width / 2.0;
        assert!(approx_eq(num.x + mm(A).advance * ns / 2.0, middle, EPS));
        assert!(approx_eq(den.x + mm(B).advance * ds / 2.0, middle, EPS));
        assert!(b.ascent > mm(A).y_max && b.descent > -mm(B).y_min);
    }

    #[test]
    fn tex_layout_sqrt_draws_a_root_sign_over_the_content() {
        // Act
        let b = boxed("\\sqrt{x}");

        // Assert: four strokes, the last a bar over the whole content.
        let strokes = rules(&b.ink);
        assert_eq!(strokes.len(), 4);
        let (x, scale) = glyph(&b.ink, X);
        assert!(approx_eq(scale, 1.0, EPS) && approx_eq(x.y, 0.0, EPS));
        let (p, q, _) = strokes[3];
        assert!(approx_eq(p.y, q.y, EPS), "overbar is horizontal");
        assert!(p.y < -mm(X).y_max, "overbar above the content");
        assert!(
            p.x <= x.x && q.x >= x.x + mm(X).advance,
            "overbar covers the content"
        );
        assert!(b.ascent >= -p.y && b.width >= q.x);
    }

    #[test]
    fn tex_layout_binom_has_parens_and_no_bar() {
        // Act
        let b = boxed("\\binom{n}{k}");

        // Assert
        assert!(rules(&b.ink).is_empty());
        let g = glyphs(&b.ink);
        let (first, last) = (g[0], g[g.len() - 1]);
        assert_eq!((first.0, last.0), ('(', ')'));
        assert!(first.2 > 1.0 && approx_eq(first.2, last.2, EPS), "{g:?}");
        assert!(approx_eq(first.1.x, 0.0, EPS));
        assert!(approx_eq(last.1.x + adv(")") * last.2, b.width, EPS));
        assert!(faces(&b.ink).iter().all(|f| f.1 == Face::Math));
        let (n, ns) = glyph(&b.ink, N);
        let (k, ks) = glyph(&b.ink, K);
        assert!(n.y < k.y);
        assert!(approx_eq(
            n.x + mm(N).advance * ns / 2.0,
            k.x + mm(K).advance * ks / 2.0,
            EPS
        ));
    }

    #[test]
    fn tex_layout_sum_limits_are_centred_above_and_below() {
        // Act
        let b = boxed("\\sum_{k=1}^{n}");

        // Assert
        let (sum, big) = glyph(&b.ink, '∑');
        assert!(approx_eq(big, BIG_OP_SCALE, EPS));
        let centre = sum.x + adv("∑") * big / 2.0;
        let (n, ns) = glyph(&b.ink, N);
        let (k, ks) = glyph(&b.ink, K);
        assert!(
            approx_eq(n.x + mm(N).advance * ns / 2.0, centre, EPS),
            "{n:?} {centre}"
        );
        // "k=1" at script scale, without relation space.
        let below = (mm(K).advance + adv("=1")) * ks;
        assert!(approx_eq(k.x + below / 2.0, centre, EPS), "{k:?} {centre}");
        let sum_top = sum.y - mm('∑').y_max * big;
        let sum_bottom = sum.y - mm('∑').y_min * big;
        assert!(n.y - mm(N).y_min * ns <= sum_top - LIMIT_GAP + EPS);
        assert!(k.y - mm(K).y_max * ks >= sum_bottom + LIMIT_GAP - EPS);
        // Words with limits too, in upright type.
        let m = boxed("\\min_k");
        let (k, ks) = glyph(&m.ink, K);
        assert!(
            approx_eq(k.x + mm(K).advance * ks / 2.0, adv("min") / 2.0, EPS),
            "{k:?}"
        );
        assert!(k.y > -mm('n').y_min);
    }

    #[test]
    fn tex_layout_integral_is_enlarged_like_sum() {
        // Act
        let int = boxed("\\int");
        let sum = boxed("\\sum");

        // Assert
        let (s, k) = glyph(&int.ink, '∫');
        let (_, sum_k) = glyph(&sum.ink, '∑');
        assert!(approx_eq(k, BIG_OP_SCALE, EPS));
        assert!(approx_eq(k, sum_k, EPS));
        // Its middle on the axis (y down).
        let middle = s.y - (mm('∫').y_max + mm('∫').y_min) * k / 2.0;
        assert!(approx_eq(middle, -AXIS, EPS), "{middle}");
        assert!(approx_eq(int.width, mm('∫').advance * k, EPS));
    }

    #[test]
    fn tex_layout_integral_scripts_sit_beside_the_sign() {
        // Act
        let int = boxed("\\int_a^b");

        // Assert
        let (sign, k) = glyph(&int.ink, '∫');
        let (lo, _) = glyph(&int.ink, A);
        let (hi, _) = glyph(&int.ink, B);
        let advance = mm('∫').advance * k;
        let italic = mm('∫').italic * k;
        assert!(italic > 0.0);
        assert!(approx_eq(hi.x, sign.x + advance + italic, EPS), "{hi:?}");
        assert!(approx_eq(lo.x, sign.x + advance - italic, EPS), "{lo:?}");
        assert!(lo.x > sign.x && hi.x > lo.x);
        assert!(hi.y < 0.0 && lo.y > 0.0, "{hi:?} {lo:?}");
        // Beside, not above and below: the scripts overlap the sign's
        // height.
        let top = sign.y - mm('∫').y_max * k;
        let bottom = sign.y - mm('∫').y_min * k;
        assert!(hi.y > top && lo.y < bottom, "{hi:?} {lo:?} {top} {bottom}");
        // The other integrals too.
        let oint = boxed("\\oint_C");
        let (sign, k) = glyph(&oint.ink, '∮');
        let (c, _) = glyph(&oint.ink, '\u{1D436}');
        let shift = (mm('∮').advance - mm('∮').italic) * k;
        assert!(approx_eq(c.x, sign.x + shift, EPS), "{c:?}");
    }

    #[test]
    fn tex_layout_integral_gets_operator_space() {
        // Act
        let w = boxed("\\int f").width;

        // Assert
        let int = mm('∫').advance * BIG_OP_SCALE;
        assert!(approx_eq(w, int + OP_SPACE + mm(F).advance, EPS), "{w}");
        // A minus after the sign is unary.
        let w = boxed("\\int -f").width;
        let minus = adv("\u{2212}");
        assert!(
            approx_eq(w, int + OP_SPACE + minus + mm(F).advance, EPS),
            "{w}"
        );
    }

    #[test]
    fn tex_layout_thin_space_is_a_sixth_of_an_em() {
        // Act
        let w = boxed("a\\,b").width;

        // Assert
        let thin = w - mm(A).advance - mm(B).advance;
        assert!(approx_eq(thin, mm('\u{2006}').advance, EPS), "{thin}");
        assert!(approx_eq(thin, 1.0 / 6.0, 1e-3), "{thin}");
    }

    #[test]
    fn tex_layout_relations_and_binary_operators_get_space() {
        let width = |src| boxed(src).width;
        let ab = mm(A).advance + mm(B).advance;

        assert!(approx_eq(
            width("a=b"),
            ab + adv("=") + 2.0 * REL_SPACE,
            EPS
        ));
        assert!(approx_eq(
            width("a+b"),
            ab + adv("+") + 2.0 * BIN_SPACE,
            EPS
        ));
        // A leading minus is unary; spaces in the source are ignored.
        assert!(approx_eq(width("-a"), adv("\u{2212}") + mm(A).advance, EPS));
        assert!(approx_eq(
            width("a + b"),
            ab + adv("+") + 2.0 * BIN_SPACE,
            EPS
        ));
        // A function before an ordinary atom, not before a bracket.
        let n = mm(N).advance;
        assert!(approx_eq(width("\\log n"), adv("log") + OP_SPACE + n, EPS));
        assert!(approx_eq(width("\\log(n)"), adv("log()") + n, EPS));
        // Before an operator after an atom, and after punctuation.
        assert!(approx_eq(
            width("n\\log n"),
            n + adv("log") + 2.0 * OP_SPACE + n,
            EPS
        ));
        assert!(approx_eq(width("a,b"), ab + adv(",") + OP_SPACE, EPS));
        // No relation space inside scripts.
        let script = (mm(I).advance + adv("=1")) * SCRIPT_SCALE;
        assert!(approx_eq(
            width("a_{i=1}"),
            mm(A).advance + script + SCRIPT_SPACE,
            EPS
        ));
    }

    #[test]
    fn tex_layout_minus_is_the_math_minus() {
        let chars: Vec<char> = glyphs(&boxed("a-b").ink).iter().map(|g| g.0).collect();

        assert_eq!(chars, vec![A, MINUS, B]);
    }

    #[test]
    fn tex_layout_literal_is_warned() {
        // Act
        let unknown = glyphs(&boxed("\\foo").ink);
        let brace = glyphs(&boxed("{a").ink);

        // Assert
        let chars: String = unknown.iter().map(|g| g.0).collect();
        assert_eq!(chars, "\\foo");
        assert!(unknown.iter().all(|g| g.3));
        assert_eq!(
            brace.iter().map(|g| (g.0, g.3)).collect::<Vec<_>>(),
            vec![('{', true), (A, false)]
        );
    }

    #[test]
    fn tex_layout_plain_text_matches_the_cell_layout() {
        // Act
        let l = layout(&parse("a b\ncd"));

        // Assert
        let cells: Vec<(char, Vec2)> = text_layout::glyphs(Vec2::ZERO, "a b\ncd", 1.0)
            .map(|g| (g.ch, g.pos))
            .collect();
        let got: Vec<(char, Vec2)> = glyphs(&l.ink).iter().map(|g| (g.0, g.1)).collect();
        assert_eq!(got.len(), cells.len());
        for (g, c) in got.iter().zip(&cells) {
            assert!(g.0 == c.0 && g.1.approx_eq(c.1, EPS), "{g:?} {c:?}");
        }
        assert!(approx_eq(l.width, 1.8, EPS) && approx_eq(l.height, 2.5, EPS));
        // `\$` is one `$` cell.
        let d = layout(&parse("\\$5"));
        let chars: Vec<(char, Vec2)> = glyphs(&d.ink).iter().map(|g| (g.0, g.1)).collect();
        assert_eq!(chars, vec![('$', v(0.0, 1.0)), ('5', v(0.6, 1.0))]);
        assert!(approx_eq(d.width, 1.2, EPS));
    }

    #[test]
    fn tex_layout_tall_formula_grows_its_line() {
        // Arrange
        let frac = boxed("\\frac{a}{b}");

        // Act
        let l = layout(&parse("$\\frac{a}{b}$\nx"));

        // Assert
        let first = frac.ascent.max(text_layout::BASELINE - MATH_PAD)
            + MATH_PAD
            + frac
                .descent
                .max(text_layout::LINE_HEIGHT - text_layout::BASELINE - MATH_PAD)
            + MATH_PAD;
        assert!(approx_eq(l.height, first + text_layout::LINE_HEIGHT, EPS));
        let (x, _) = glyph(&l.ink, 'x');
        assert!(
            x.approx_eq(v(0.0, first + text_layout::BASELINE), EPS),
            "{x:?}"
        );
        let (a, s) = glyph(&l.ink, A);
        assert!(a.y - mm(A).y_max * s >= -EPS, "inside the line");
        // A formula taller than a text line grows it.
        let tall = layout(&parse("$\\frac{\\sum_k^n}{b}$"));
        assert!(tall.height > text_layout::LINE_HEIGHT);
    }

    #[test]
    fn tex_layout_mixes_text_and_formula_on_one_line() {
        // Act
        let l = layout(&parse("f $x^2$ g"));

        // Assert: plain cells, the formula, then cells after it, one line.
        let (f, _) = glyph(&l.ink, 'f');
        let (x, _) = glyph(&l.ink, X);
        let (two, _) = glyph(&l.ink, '2');
        let (g, _) = glyph(&l.ink, 'g');
        let formula = boxed("x^2");
        let (two_in, _) = glyph(&formula.ink, '2');
        assert!(f.approx_eq(v(0.0, 1.0), EPS));
        assert!(x.approx_eq(v(1.2, 1.0), EPS));
        assert!(two.approx_eq(v(1.2, 1.0) + two_in, EPS), "{two:?}");
        assert!(g.approx_eq(v(1.2 + formula.width + 0.6, 1.0), EPS), "{g:?}");
        assert!(approx_eq(l.width, 1.8 + formula.width + 0.6, EPS));
        assert!(approx_eq(l.height, text_layout::LINE_HEIGHT, EPS));
    }

    #[test]
    fn tex_layout_line_holds_italic_overhangs() {
        // Arrange: italic j hangs left of its pen, italic f right of its
        // advance.
        let (j, f) = (mm(J), mm(F));
        assert!(j.x_min < 0.0 && f.x_max > f.advance);

        // Act
        let left = layout(&parse("$j$"));
        let right = layout(&parse("$f$ g"));
        let inside = layout(&parse("a$j$"));

        // Assert: a formula starting a line moves right by its overhang;
        // the line is as wide as its rightmost ink.
        let (pos, _) = glyph(&left.ink, J);
        assert!(approx_eq(pos.x, -j.x_min, EPS), "{pos:?}");
        let (g, _) = glyph(&right.ink, 'g');
        assert!(approx_eq(g.x, f.x_max + 0.6, EPS), "{g:?}");
        let (pos, _) = glyph(&inside.ink, J);
        assert!(approx_eq(pos.x, 0.6, EPS), "{pos:?}");
        for l in [&left, &right, &inside] {
            let b = l.bounds().expand(1e-4);
            assert!(l.ink.iter().all(|ink| {
                let ib = ink.bounds();
                b.contains(ib.min) && b.contains(ib.max)
            }));
        }
    }

    // ---- T39 accents (ADR-T39-1) -------------------------------------------

    /// The ink accent `def` adds: its glyph, or its rule.
    fn mark_ink(ink: &[Ink], def: &AccentDef) -> Vec<Ink> {
        ink.iter()
            .copied()
            .filter(|ink| match (*ink, def.mark) {
                (Ink::Glyph { ch, .. }, Mark::Glyph(mark, _)) => ch == mark,
                (Ink::Rule { .. }, Mark::Bar | Mark::Overline) => true,
                _ => false,
            })
            .collect()
    }

    /// The bounds of all of `ink`; panics without ink.
    fn ink_box(ink: &[Ink]) -> Aabb {
        match ink.iter().map(Ink::bounds).reduce(|a, b| a.union(&b)) {
            Some(b) => b,
            None => panic!("no ink"),
        }
    }

    /// `\name{arg}`.
    fn accented(def: &AccentDef, arg: &str) -> MathBox {
        boxed(&format!("\\{}{{{arg}}}", def.name))
    }

    #[test]
    fn tex_layout_accent_sits_above_its_argument() {
        for def in ACCENTS {
            for arg in ["x", "F", "2", "AB"] {
                // Arrange
                let plain = boxed(arg);

                // Act
                let b = accented(def, arg);

                // Assert: the mark's bottom is the gap above the argument.
                let mark = mark_ink(&b.ink, def);
                assert_eq!(mark.len(), 1, "\\{}{{{arg}}}: {:?}", def.name, b.ink);
                let m = ink_box(&mark);
                let top = -(plain.ascent + ACCENT_GAP);
                assert!(
                    approx_eq(m.max.y, top, EPS),
                    "\\{}{{{arg}}}: {m:?}",
                    def.name
                );
                assert!(
                    m.max.y < ink_box(&plain.ink).min.y,
                    "\\{}{{{arg}}}",
                    def.name
                );
            }
        }
    }

    #[test]
    fn tex_layout_accent_is_centred_on_an_upright_argument() {
        for def in ACCENTS.iter().filter(|d| d.mark != Mark::Overline) {
            for arg in ["2", "AB", "\\Gamma", "xy"] {
                // Arrange
                let centre = ink_box(&boxed(arg).ink).center().x;

                // Act
                let b = accented(def, arg);

                // Assert
                let m = ink_box(&mark_ink(&b.ink, def));
                assert!(
                    approx_eq(m.center().x, centre, EPS),
                    "\\{}{{{arg}}}: {} vs {centre}",
                    def.name,
                    m.center().x
                );
            }
        }
    }

    #[test]
    fn tex_layout_accent_over_an_italic_letter_is_skewed() {
        for def in ACCENTS.iter().filter(|d| d.mark != Mark::Overline) {
            for arg in ["x", "F", "\\alpha", " v "] {
                // Arrange
                let plain = boxed(arg);
                let centre = ink_box(&plain.ink).center().x;

                // Act
                let b = accented(def, arg);

                // Assert: the top of a slanted letter lies right of its
                // middle.
                let m = ink_box(&mark_ink(&b.ink, def));
                let want = centre + ACCENT_SKEW * plain.ascent;
                assert!(
                    approx_eq(m.center().x, want, EPS),
                    "\\{}{{{arg}}}: {} vs {want}",
                    def.name,
                    m.center().x
                );
            }
        }
    }

    #[test]
    fn tex_layout_bar_is_a_short_rule() {
        // Act
        let b = boxed("\\bar{AB}");

        // Assert: as long as the macron, whatever the argument.
        let strokes = rules(&b.ink);
        assert_eq!(strokes.len(), 1);
        let (p, q, width) = strokes[0];
        assert!(approx_eq(p.y, q.y, EPS), "horizontal");
        assert!(approx_eq((q.x - p.x).abs(), ACCENT_BAR, EPS), "{p:?} {q:?}");
        assert!(approx_eq(width, RULE, EPS));
    }

    #[test]
    fn tex_layout_overline_spans_its_argument() {
        for arg in ["AB", "x", "x^2"] {
            // Arrange
            let plain = boxed(arg);

            // Act
            let b = boxed(&format!("\\overline{{{arg}}}"));

            // Assert
            let strokes = rules(&b.ink);
            assert_eq!(strokes.len(), 1, "{arg}");
            let (p, q, width) = strokes[0];
            assert!(approx_eq(p.y, q.y, EPS), "{arg}: horizontal");
            assert!(approx_eq(p.x.min(q.x), 0.0, EPS), "{arg}: {p:?}");
            assert!(approx_eq(p.x.max(q.x), plain.width, EPS), "{arg}: {q:?}");
            assert!(approx_eq(width, RULE, EPS));
        }
    }

    #[test]
    fn tex_layout_accent_keeps_the_argument_width_and_descent() {
        for def in ACCENTS {
            for arg in ["y", "F", "AB", "x_i"] {
                // Arrange
                let plain = boxed(arg);

                // Act
                let b = accented(def, arg);

                // Assert: the ascent reaches the mark's top.
                let name = def.name;
                assert!(approx_eq(b.width, plain.width, EPS), "\\{name}{{{arg}}}");
                assert!(
                    approx_eq(b.descent, plain.descent, EPS),
                    "\\{name}{{{arg}}}"
                );
                assert!(approx_eq(b.italic, plain.italic, EPS), "\\{name}{{{arg}}}");
                let m = ink_box(&mark_ink(&b.ink, def));
                assert!(approx_eq(b.ascent, -m.min.y, EPS), "\\{name}{{{arg}}}");
            }
        }
    }

    #[test]
    fn tex_layout_accent_takes_scripts() {
        // Arrange
        let vec_f = boxed("\\vec{F}");
        let dot_x = boxed("\\dot{x}");

        // Act
        let sub = boxed("\\vec{F}_x");
        let sup = boxed("\\dot{x}^2");
        let inner = boxed("x^{\\hat a}");

        // Assert: scripts start after the accented box, the superscript
        // above its accent's reach.
        let (x, _) = glyph(&sub.ink, X);
        assert!(approx_eq(x.x, vec_f.width, EPS), "{x:?}");
        assert!(x.y > 0.0);
        let (two, _) = glyph(&sup.ink, '2');
        assert!(approx_eq(two.x, dot_x.width + dot_x.italic, EPS), "{two:?}");
        assert!(-two.y >= dot_x.ascent - SUP_DROP - EPS, "{two:?}");
        let (hat, scale) = glyph(&inner.ink, '^');
        assert!(approx_eq(scale, SCRIPT_SCALE, EPS), "{hat:?}");
    }

    // ---- AC-3 faces and mapping (ADR-T33-1) --------------------------------

    #[test]
    fn tex_math_letters_are_italic() {
        // Act
        let got = faces(&boxed("aAzZh").ink);

        // Assert
        assert_eq!(
            got,
            vec![
                (A, Face::Math),
                ('\u{1D434}', Face::Math),
                (Z, Face::Math),
                ('\u{1D44D}', Face::Math),
                ('\u{210E}', Face::Math),
            ]
        );
    }

    #[test]
    fn tex_lowercase_greek_is_italic_and_capitals_upright() {
        // Act
        let got = faces(
            &boxed("\\alpha\\omega\\varphi\\phi\\epsilon\\varepsilon\\varsigma\\Gamma\\Omega").ink,
        );

        // Assert
        let chars: Vec<char> = got.iter().map(|g| g.0).collect();
        assert_eq!(
            chars,
            vec![
                '\u{1D6FC}',
                '\u{1D714}',
                '\u{1D711}',
                '\u{1D719}',
                '\u{1D716}',
                '\u{1D700}',
                '\u{1D70D}',
                'Γ',
                'Ω',
            ]
        );
        assert!(got.iter().all(|g| g.1 == Face::Math));
    }

    #[test]
    fn tex_digits_operators_and_functions_are_upright_math() {
        // Act
        let got = faces(&boxed("1+\\log x\\le\\infty\\text{ok}").ink);

        // Assert
        let chars: String = got.iter().map(|g| g.0).collect();
        assert_eq!(chars, format!("1+log{X}≤∞ok"));
        assert!(got.iter().all(|g| g.1 == Face::Math));
        let pmod: String = faces(&boxed("\\pmod{n}").ink).iter().map(|g| g.0).collect();
        assert_eq!(pmod, format!("(mod{N})"));
    }

    #[test]
    fn tex_mathcal_maps_to_script_letters() {
        // Act
        let got = faces(&boxed("\\mathcal{ABE}x\\mathcal L\\mathcal{a1}").ink);

        // Assert: capitals turn script (holes from Letterlike Symbols),
        // anything else is drawn as outside `\mathcal`.
        assert_eq!(
            got.iter().map(|g| g.0).collect::<Vec<_>>(),
            vec!['\u{1D49C}', '\u{212C}', '\u{2130}', X, '\u{2112}', A, '1']
        );
        assert!(got.iter().all(|g| g.1 == Face::Math));
        assert!(math_metrics::get('\u{1D49C}').is_some());
    }

    #[test]
    fn tex_literals_and_plain_text_use_the_text_face() {
        // Act
        let l = layout(&parse("a$x\\foo$"));

        // Assert
        let got = faces(&l.ink);
        assert_eq!(got[0], ('a', Face::Text));
        assert_eq!(got[1], (X, Face::Math));
        assert!(got[2..].iter().all(|g| g.1 == Face::Text), "{got:?}");
    }

    #[test]
    fn tex_char_without_math_glyph_falls_back_to_a_text_cell() {
        // Arrange
        assert!(math_metrics::get('日').is_none());

        // Act
        let typed = boxed("日");
        let text = boxed("\\text{日}");

        // Assert: one 0.6 em cell of the text font.
        for b in [typed, text] {
            assert_eq!(faces(&b.ink), vec![('日', Face::Text)]);
            assert!(approx_eq(b.width, ADVANCE, EPS));
            assert!(approx_eq(b.ascent, GLYPH_ASCENT, EPS));
        }
    }
}
