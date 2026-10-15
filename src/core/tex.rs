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
use crate::core::text_layout::{ADVANCE, BASELINE, LINE_HEIGHT};

/// Ascent of a glyph box per unit of scale: the font's cap height 0.735
/// with its overshoot.
pub const GLYPH_ASCENT: f32 = 0.75;

/// Descent of a glyph box per unit of scale: the font's descender.
pub const GLYPH_DESCENT: f32 = 0.22;

/// Ascent of brackets, bars and big operators per unit of scale.
pub const TALL_ASCENT: f32 = 0.85;

/// Descent of brackets, bars and big operators per unit of scale.
pub const TALL_DESCENT: f32 = 0.17;

/// Height of the math axis (the middle of `−`) above the baseline.
pub const AXIS: f32 = 0.34;

/// Thickness of fraction bars and root signs (that of `−`).
pub const RULE: f32 = 0.07;

/// Scale of scripts, limits and fraction parts relative to their parent.
pub const SCRIPT_SCALE: f32 = 0.7;

/// Smallest scale of nested scripts.
pub const MIN_SCALE: f32 = 0.5;

/// Smallest rise of a superscript's baseline.
pub const SUP_SHIFT: f32 = 0.4;

/// A superscript's baseline sits this far below the top of its base.
pub const SUP_DROP: f32 = 0.35;

/// Smallest drop of a subscript's baseline.
pub const SUB_SHIFT: f32 = 0.2;

/// A subscript's baseline sits this far above the bottom of its base.
pub const SUB_DROP: f32 = 0.05;

/// Smallest gap between a superscript and a subscript on one base.
pub const SCRIPT_GAP: f32 = 0.1;

/// Gap between a big operator and its limits.
pub const LIMIT_GAP: f32 = 0.1;

/// Scale of the big operators `∑` and `∏`.
pub const BIG_OP_SCALE: f32 = 1.3;

/// Gap between a fraction bar and its numerator or denominator.
pub const FRAC_GAP: f32 = 0.08;

/// Space left and right of a fraction's parts.
pub const FRAC_PAD: f32 = 0.1;

/// Gap between a root's content and its overbar.
pub const SQRT_GAP: f32 = 0.1;

/// Width of the root sign before the content.
pub const SQRT_SIGN: f32 = 0.55;

/// Largest scale of a binomial's parentheses.
pub const BINOM_MAX_SCALE: f32 = 3.0;

/// Space on each side of a relation.
pub const REL_SPACE: f32 = 0.15;

/// Space on each side of a binary operator.
pub const BIN_SPACE: f32 = 0.1;

/// Space after a function or big operator before an ordinary atom.
pub const OP_SPACE: f32 = 0.1;

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
    def("min", "min", Class::Limits),
    def("max", "max", Class::Limits),
    def("lim", "lim", Class::Limits),
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

/// Something to draw, in em from its box's origin, y down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ink {
    /// A glyph of the embedded font.
    Glyph {
        /// The character.
        ch: char,
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
    /// The box the ink covers: a glyph's cell (its advance by its ascent
    /// and descent), a rule's segment grown by half its width.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        match *self {
            Self::Glyph { ch, pos, scale, .. } => {
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
                pos,
                scale,
                warn,
            } => Self::Glyph {
                ch,
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
    /// What to draw, relative to the left end of the baseline.
    pub ink: Vec<Ink>,
}

impl MathBox {
    /// One glyph at `scale`.
    fn glyph(ch: char, scale: f32, warn: bool) -> Self {
        let (ascent, descent) = glyph_extent(ch);
        Self {
            width: ADVANCE * scale,
            ascent: ascent * scale,
            descent: descent * scale,
            ink: vec![Ink::Glyph {
                ch,
                pos: Vec2::ZERO,
                scale,
                warn,
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

    /// `boxes` side by side on one baseline.
    fn row(boxes: impl IntoIterator<Item = Self>) -> Self {
        let mut out = Self::default();
        for b in boxes {
            out.put(&b, Vec2::new(out.width, 0.0));
            out.width += b.width;
            out.ascent = out.ascent.max(b.ascent);
            out.descent = out.descent.max(b.descent);
        }
        out
    }

    /// The characters of `text` side by side, `-` as typed.
    fn word(text: &str, scale: f32, warn: bool) -> Self {
        Self::row(text.chars().map(|ch| Self::glyph(ch, scale, warn)))
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

/// Ascent and descent of glyph `ch` per unit of scale.
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
            "sqrt" | "pmod" => self.args(1, end).map(|args| Node::Command {
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
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let mut atoms: Vec<(MathBox, Class)> =
        nodes.iter().filter_map(|node| atom(node, scale)).collect();
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
    let op = |c| matches!(c, Class::Op | Class::Limits | Class::BigOp);
    if op(a) && (b == Class::Ord || op(b)) {
        return OP_SPACE * scale;
    }
    // Relation and operator space only at the block's own size.
    if scale < 1.0 {
        return 0.0;
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

/// The box and class of one node; `None` for whitespace.
fn atom(node: &Node, scale: f32) -> Option<(MathBox, Class)> {
    let atom = match node {
        Node::Space(_) => return None,
        Node::Char(ch) => {
            let shown = if *ch == '-' { MINUS } else { *ch };
            (MathBox::glyph(shown, scale, false), char_class(*ch))
        }
        Node::Symbol(name) => match symbol(name) {
            Some(def) => (symbol_box(def, scale), def.class),
            None => (MathBox::word(&format!("\\{name}"), scale, true), Class::Ord),
        },
        Node::Group(nodes) => (layout_math(nodes, scale), Class::Ord),
        Node::Scripts {
            base,
            first,
            second,
        } => {
            let (base, class) = base
                .as_deref()
                .and_then(|b| atom(b, scale))
                .unwrap_or((MathBox::default(), Class::Ord));
            let ss = script_scale(scale);
            let (mut sup, mut sub) = (None, None);
            for script in std::iter::once(first).chain(second) {
                let b = layout_math(&script.arg.nodes, ss);
                match script.kind {
                    ScriptKind::Sup => sup = Some(b),
                    ScriptKind::Sub => sub = Some(b),
                }
            }
            let b = if matches!(class, Class::BigOp | Class::Limits) {
                limits(&base, sup, sub, scale)
            } else {
                scripts(base, sup, sub, scale)
            };
            (b, class)
        }
        Node::Command { name, args } => {
            let arg = |i: usize| args.get(i).map_or(&[][..], |a: &Arg| &a.nodes[..]);
            let b = match name.as_str() {
                "frac" => fraction(arg(0), arg(1), scale, true),
                "binom" => binomial(arg(0), arg(1), scale),
                "sqrt" => root(arg(0), scale),
                "pmod" => MathBox::row([
                    MathBox::gap(ADVANCE * scale),
                    MathBox::word("(mod", scale, false),
                    MathBox::gap(ADVANCE * scale),
                    layout_math(arg(0), scale),
                    MathBox::glyph(')', scale, false),
                ]),
                _ => MathBox::word(&format!("\\{name}"), scale, true),
            };
            (b, Class::Ord)
        }
        Node::Text { body, .. } => (MathBox::word(body, scale, false), Class::Ord),
        Node::Literal(text) => (MathBox::word(text, scale, true), Class::Ord),
    };
    Some(atom)
}

/// The box of a table symbol: a glyph, an enlarged big operator or a word.
fn symbol_box(def: &SymbolDef, scale: f32) -> MathBox {
    let mut chars = def.text.chars();
    match (chars.next(), chars.next(), def.class) {
        (Some(ch), None, Class::BigOp) => big_operator(ch, scale),
        (Some(ch), None, _) => MathBox::glyph(ch, scale, false),
        _ => MathBox::word(def.text, scale, false),
    }
}

/// `∑` or `∏`, enlarged and centred on the axis.
fn big_operator(ch: char, scale: f32) -> MathBox {
    let k = BIG_OP_SCALE * scale;
    let (ascent, descent) = glyph_extent(ch);
    // The glyph's middle sits on the axis.
    let y = -AXIS * scale - (descent - ascent) * k / 2.0;
    MathBox {
        width: ADVANCE * k,
        ascent: ascent * k - y,
        descent: descent * k + y,
        ink: vec![Ink::Glyph {
            ch,
            pos: Vec2::new(0.0, y),
            scale: k,
            warn: false,
        }],
    }
}

/// Superscript and subscript to the right of `base`.
fn scripts(base: MathBox, sup: Option<MathBox>, sub: Option<MathBox>, scale: f32) -> MathBox {
    let up = (SUP_SHIFT * scale).max(base.ascent - SUP_DROP * scale);
    let mut down = (SUB_SHIFT * scale).max(base.descent - SUB_DROP * scale);
    if let (Some(sup), Some(sub)) = (&sup, &sub) {
        let gap = (down - sub.ascent) - (sup.descent - up);
        if gap < SCRIPT_GAP * scale {
            down += SCRIPT_GAP * scale - gap;
        }
    }
    let mut out = MathBox {
        width: base.width,
        ascent: base.ascent,
        descent: base.descent,
        ink: base.ink,
    };
    let x = out.width;
    if let Some(sup) = sup {
        out.put(&sup, Vec2::new(x, -up));
        out.width = out.width.max(x + sup.width);
        out.ascent = out.ascent.max(up + sup.ascent);
        out.descent = out.descent.max(sup.descent - up);
    }
    if let Some(sub) = sub {
        out.put(&sub, Vec2::new(x, down));
        out.width = out.width.max(x + sub.width);
        out.ascent = out.ascent.max(sub.ascent - down);
        out.descent = out.descent.max(down + sub.descent);
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
fn fraction(num: &[Node], den: &[Node], scale: f32, bar: bool) -> MathBox {
    let ss = script_scale(scale);
    let (n, d) = (layout_math(num, ss), layout_math(den, ss));
    let pad = FRAC_PAD * scale;
    let width = n.width.max(d.width) + 2.0 * pad;
    let axis = -AXIS * scale;
    let half = RULE * scale / 2.0;
    let n_base = axis - half - FRAC_GAP * scale - n.descent;
    let d_base = axis + half + FRAC_GAP * scale + d.ascent;
    let mut out = MathBox {
        width,
        ascent: (n.ascent - n_base).max(half - axis),
        descent: (d_base + d.descent).max(axis + half),
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
fn binomial(n: &[Node], k: &[Node], scale: f32) -> MathBox {
    let inner = fraction(n, k, scale, false);
    let height = inner.ascent + inner.descent;
    let paren = (height / (TALL_ASCENT + TALL_DESCENT)).clamp(scale, BINOM_MAX_SCALE * scale);
    // The parentheses' middle sits on the middle of the stack.
    let middle = (inner.descent - inner.ascent) / 2.0;
    let y = middle - (TALL_DESCENT - TALL_ASCENT) * paren / 2.0;
    let side = ADVANCE * paren;
    let mut out = MathBox {
        width: inner.width + 2.0 * side,
        ascent: inner.ascent.max(TALL_ASCENT * paren - y),
        descent: inner.descent.max(TALL_DESCENT * paren + y),
        ink: Vec::new(),
    };
    out.put(&MathBox::glyph('(', paren, false), Vec2::new(0.0, y));
    out.put(&inner, Vec2::new(side, 0.0));
    out.put(
        &MathBox::glyph(')', paren, false),
        Vec2::new(side + inner.width, y),
    );
    out
}

/// A root sign of four rules over `content`.
fn root(content: &[Node], scale: f32) -> MathBox {
    let c = layout_math(content, scale);
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
                let at = Vec2::new(line.width, 0.0);
                line.ink.extend(b.ink.iter().map(|ink| ink.shifted(at)));
                line.width += b.width;
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
    use crate::core::geom::{Vec2, approx_eq};
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
                } => Some((ch, pos, scale, warn)),
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
            math("\\,x"),
            vec![Node::Literal("\\,".to_owned()), Node::Char('x')]
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

    // ---- AC-2 layout -------------------------------------------------------

    #[test]
    fn tex_layout_char_is_one_cell() {
        // Act
        let b = boxed("x");

        // Assert
        assert!(approx_eq(b.width, 0.6, EPS));
        assert!(approx_eq(b.ascent, GLYPH_ASCENT, EPS));
        assert!(approx_eq(b.descent, GLYPH_DESCENT, EPS));
        assert_eq!(glyphs(&b.ink), vec![('x', v(0.0, 0.0), 1.0, false)]);
    }

    #[test]
    fn tex_layout_superscript_is_raised_and_scaled() {
        // Act
        let b = boxed("x^2");

        // Assert
        let (pos, scale) = glyph(&b.ink, '2');
        assert!(pos.approx_eq(v(0.6, -0.4), EPS), "{pos:?}");
        assert!(approx_eq(scale, SCRIPT_SCALE, EPS));
        assert!(approx_eq(b.width, 0.6 + 0.6 * SCRIPT_SCALE, EPS));
        // A script of a script stops at the smallest scale.
        let (_, scale) = glyph(&boxed("x^{y^z}").ink, 'z');
        assert!(approx_eq(scale, MIN_SCALE, EPS));
    }

    #[test]
    fn tex_layout_subscript_is_lowered_and_scaled() {
        // Act
        let b = boxed("x_2");

        // Assert
        let (pos, scale) = glyph(&b.ink, '2');
        assert!(pos.approx_eq(v(0.6, 0.2), EPS), "{pos:?}");
        assert!(approx_eq(scale, SCRIPT_SCALE, EPS));
    }

    #[test]
    fn tex_layout_sub_and_sup_share_the_base() {
        for src in ["x_i^2", "x^2_i"] {
            // Act
            let b = boxed(src);

            // Assert: both start at the base's right edge, apart by 0.1 em.
            let (sup, s) = glyph(&b.ink, '2');
            let (sub, _) = glyph(&b.ink, 'i');
            assert!(
                approx_eq(sup.x, 0.6, EPS) && approx_eq(sub.x, 0.6, EPS),
                "{src}"
            );
            let sup_bottom = sup.y + GLYPH_DESCENT * s;
            let sub_top = sub.y - GLYPH_ASCENT * s;
            assert!(sub_top - sup_bottom >= 0.1 - EPS, "{src}: {sup:?} {sub:?}");
            assert!(approx_eq(b.width, 0.6 + 0.6 * SCRIPT_SCALE, EPS));
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
        assert!(p.x >= 0.0 && q.x <= b.width && q.x - p.x >= 0.6 * SCRIPT_SCALE);
        let (num, ns) = glyph(&b.ink, 'a');
        let (den, ds) = glyph(&b.ink, 'b');
        assert!(approx_eq(ns, SCRIPT_SCALE, EPS) && approx_eq(ds, SCRIPT_SCALE, EPS));
        assert!(num.y + GLYPH_DESCENT * ns < -AXIS - RULE / 2.0);
        assert!(den.y - GLYPH_ASCENT * ds > -AXIS + RULE / 2.0);
        let middle = b.width / 2.0;
        assert!(approx_eq(num.x + 0.3 * ns, middle, EPS));
        assert!(approx_eq(den.x + 0.3 * ds, middle, EPS));
        assert!(b.ascent > GLYPH_ASCENT && b.descent > GLYPH_DESCENT);
    }

    #[test]
    fn tex_layout_sqrt_draws_a_root_sign_over_the_content() {
        // Act
        let b = boxed("\\sqrt{x}");

        // Assert: four strokes, the last a bar over the whole content.
        let strokes = rules(&b.ink);
        assert_eq!(strokes.len(), 4);
        let (x, scale) = glyph(&b.ink, 'x');
        assert!(approx_eq(scale, 1.0, EPS) && approx_eq(x.y, 0.0, EPS));
        let (p, q, _) = strokes[3];
        assert!(approx_eq(p.y, q.y, EPS), "overbar is horizontal");
        assert!(p.y < -GLYPH_ASCENT, "overbar above the content");
        assert!(p.x <= x.x && q.x >= x.x + 0.6, "overbar covers the content");
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
        assert!(approx_eq(last.1.x + 0.6 * last.2, b.width, EPS));
        let (n, _) = glyph(&b.ink, 'n');
        let (k, _) = glyph(&b.ink, 'k');
        assert!(n.y < k.y && approx_eq(n.x, k.x, EPS));
    }

    #[test]
    fn tex_layout_sum_limits_are_centred_above_and_below() {
        // Act
        let b = boxed("\\sum_{k=1}^{n}");

        // Assert
        let (sum, big) = glyph(&b.ink, '∑');
        assert!(approx_eq(big, BIG_OP_SCALE, EPS));
        let centre = sum.x + 0.3 * big;
        let (n, ns) = glyph(&b.ink, 'n');
        let (k, ks) = glyph(&b.ink, 'k');
        assert!(approx_eq(n.x + 0.3 * ns, centre, EPS), "{n:?} {centre}");
        // "k=1" is three cells at script scale, without relation space.
        assert!(approx_eq(k.x + 0.9 * ks, centre, EPS), "{k:?} {centre}");
        assert!(n.y + GLYPH_DESCENT * ns <= sum.y - TALL_ASCENT * big + EPS);
        assert!(k.y - GLYPH_ASCENT * ks >= sum.y + TALL_DESCENT * big - EPS);
        // Words with limits too.
        let m = boxed("\\min_k");
        let (k, ks) = glyph(&m.ink, 'k');
        assert!(approx_eq(k.x + 0.3 * ks, 0.9, EPS), "{k:?}");
        assert!(k.y > GLYPH_DESCENT);
    }

    #[test]
    fn tex_layout_relations_and_binary_operators_get_space() {
        let width = |src| boxed(src).width;

        assert!(approx_eq(width("a=b"), 1.8 + 2.0 * REL_SPACE, EPS));
        assert!(approx_eq(width("a+b"), 1.8 + 2.0 * BIN_SPACE, EPS));
        // A leading minus is unary; spaces in the source are ignored.
        assert!(approx_eq(width("-a"), 1.2, EPS));
        assert!(approx_eq(width("a + b"), 1.8 + 2.0 * BIN_SPACE, EPS));
        // A function before an ordinary atom, not before a bracket.
        assert!(approx_eq(width("\\log n"), 2.4 + OP_SPACE, EPS));
        assert!(approx_eq(width("\\log(n)"), 3.6, EPS));
        // No relation space inside scripts.
        assert!(approx_eq(width("a_{i=1}"), 0.6 + 1.8 * SCRIPT_SCALE, EPS));
    }

    #[test]
    fn tex_layout_minus_is_the_math_minus() {
        let chars: Vec<char> = glyphs(&boxed("a-b").ink).iter().map(|g| g.0).collect();

        assert_eq!(chars, vec!['a', MINUS, 'b']);
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
            vec![('{', true), ('a', false)]
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
        let first = frac.ascent + MATH_PAD + frac.descent + MATH_PAD;
        assert!(first > text_layout::LINE_HEIGHT);
        assert!(approx_eq(l.height, first + text_layout::LINE_HEIGHT, EPS));
        let (x, _) = glyph(&l.ink, 'x');
        assert!(
            x.approx_eq(v(0.0, first + text_layout::BASELINE), EPS),
            "{x:?}"
        );
        let (a, _) = glyph(&l.ink, 'a');
        assert!(a.y - GLYPH_ASCENT * SCRIPT_SCALE >= -EPS, "inside the line");
    }

    #[test]
    fn tex_layout_mixes_text_and_formula_on_one_line() {
        // Act
        let l = layout(&parse("f $x^2$ g"));

        // Assert: plain cells, the formula, then cells after it, one line.
        let (f, _) = glyph(&l.ink, 'f');
        let (x, _) = glyph(&l.ink, 'x');
        let (two, _) = glyph(&l.ink, '2');
        let (g, _) = glyph(&l.ink, 'g');
        let formula = 0.6 + 0.6 * SCRIPT_SCALE;
        assert!(f.approx_eq(v(0.0, 1.0), EPS));
        assert!(x.approx_eq(v(1.2, 1.0), EPS));
        assert!(two.approx_eq(v(1.8, 0.6), EPS));
        assert!(g.approx_eq(v(1.2 + formula + 0.6, 1.0), EPS), "{g:?}");
        assert!(approx_eq(l.width, 1.8 + formula + 0.6, EPS));
        assert!(approx_eq(l.height, text_layout::LINE_HEIGHT, EPS));
    }
}
