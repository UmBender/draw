//! A small TeX subset inside text blocks (ADR-T30-1).
//!
//! [`parse`] splits the text of a block into plain text and `$…$`
//! formulas and parses each formula into a lossless tree of [`Node`]s:
//! [`print`] gives the source back. [`layout`] typesets the tree into
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
        todo!()
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
    todo!("{ch}")
}

/// Splits a block's text into plain text, `\$` and formulas, and parses
/// every formula. Total: every string parses.
#[must_use]
pub fn parse(text: &str) -> Vec<Segment> {
    todo!("{text}")
}

/// The source of `segments`: `print(&parse(s)) == s` for every `s`.
#[must_use]
pub fn print(segments: &[Segment]) -> String {
    todo!("{segments:?}")
}

/// Lays out the nodes of one formula at `scale` (1 for the block's font
/// size).
#[must_use]
pub fn layout_math(nodes: &[Node], scale: f32) -> MathBox {
    todo!("{nodes:?} {scale}")
}

/// Lays out a parsed block: lines split on `\n`, plain characters in
/// their monospaced cells, formulas typeset; a line grows to fit its
/// formulas.
#[must_use]
pub fn layout(segments: &[Segment]) -> Layout {
    todo!("{segments:?}")
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
        assert_eq!(
            math("a\\"),
            vec![Node::Char('a'), Node::Literal("\\".to_owned())]
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
        let m = boxed("\\min_i");
        let (i, is) = glyph(&m.ink, 'i');
        assert!(approx_eq(i.x + 0.3 * is, 0.9, EPS), "{i:?}");
        assert!(i.y > GLYPH_DESCENT);
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
