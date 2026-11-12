//! Symbol, accent, delimiter and alphabet tables.

use super::{CASES_STRETCH, Hash, Style, Tail, VEC_SCALE};

/// What `\left` and `\right` accept: each delimiter's source and the glyph
/// it draws; `.` draws none (ADR-T89-1).
pub const DELIMITERS: &[(&str, Option<char>)] = &[
    (".", None),
    ("(", Some('(')),
    (")", Some(')')),
    ("[", Some('[')),
    ("]", Some(']')),
    ("\\{", Some('{')),
    ("\\}", Some('}')),
    ("|", Some('|')),
    ("\\|", Some('‖')),
    ("\\langle", Some('⟨')),
    ("\\rangle", Some('⟩')),
    ("\\lfloor", Some('⌊')),
    ("\\rfloor", Some('⌋')),
    ("\\lceil", Some('⌈')),
    ("\\rceil", Some('⌉')),
];

/// How the cells of a column sit in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// In the middle.
    Centre,
    /// At the left edge.
    Left,
}

/// One environment of the subset (ADR-T90-1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvironmentDef {
    /// The name between the braces.
    pub name: &'static str,
    /// The opening and closing delimiter, as rows of [`DELIMITERS`];
    /// `None` for no brackets at all.
    pub brackets: Option<(&'static str, &'static str)>,
    /// How the cells sit in their columns.
    pub align: Align,
    /// Factor on the least height of a row.
    pub stretch: f32,
}

/// The environments of the subset (ADR-T90-1).
pub const ENVIRONMENTS: &[EnvironmentDef] = &[
    EnvironmentDef {
        name: "matrix",
        brackets: None,
        align: Align::Centre,
        stretch: 1.0,
    },
    EnvironmentDef {
        name: "pmatrix",
        brackets: Some(("(", ")")),
        align: Align::Centre,
        stretch: 1.0,
    },
    EnvironmentDef {
        name: "bmatrix",
        brackets: Some(("[", "]")),
        align: Align::Centre,
        stretch: 1.0,
    },
    EnvironmentDef {
        name: "vmatrix",
        brackets: Some(("|", "|")),
        align: Align::Centre,
        stretch: 1.0,
    },
    EnvironmentDef {
        name: "cases",
        brackets: Some(("\\{", ".")),
        align: Align::Left,
        stretch: CASES_STRETCH,
    },
];

/// The row of [`ENVIRONMENTS`] called `name`.
#[must_use]
pub fn environment(name: &str) -> Option<&'static EnvironmentDef> {
    ENVIRONMENTS.iter().find(|def| def.name == name)
}

/// One extensible arrow of the subset (ADR-T103-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XArrowDef {
    /// Name without the backslash.
    pub name: &'static str,
    /// The plain arrow glyph it draws at its natural length.
    pub glyph: char,
    /// The head glyph at the left end of a stretched arrow, if any.
    pub left: Option<char>,
    /// The head glyph at the right end of a stretched arrow, if any.
    pub right: Option<char>,
    /// Whether the shaft is two rules (`⇒`) instead of one (`→`).
    pub double: bool,
    /// What the end without a head draws.
    pub tail: Tail,
}

/// The extensible arrows of the subset (ADR-T103-1).
pub const XARROWS: &[XArrowDef] = &[
    xarrow_def("xrightarrow", '→', None, Some('→'), false, Tail::Plain),
    xarrow_def("xleftarrow", '←', Some('←'), None, false, Tail::Plain),
    xarrow_def(
        "xleftrightarrow",
        '↔',
        Some('←'),
        Some('→'),
        false,
        Tail::Plain,
    ),
    xarrow_def("xRightarrow", '⇒', None, Some('⇒'), true, Tail::Plain),
    xarrow_def("xLeftarrow", '⇐', Some('⇐'), None, true, Tail::Plain),
    xarrow_def(
        "xLeftrightarrow",
        '⇔',
        Some('⇐'),
        Some('⇒'),
        true,
        Tail::Plain,
    ),
    xarrow_def("xmapsto", '↦', None, Some('→'), false, Tail::Bar),
    xarrow_def("xhookrightarrow", '↪', None, Some('→'), false, Tail::Hook),
];

/// Shorthand for a row of [`XARROWS`].
const fn xarrow_def(
    name: &'static str,
    glyph: char,
    left: Option<char>,
    right: Option<char>,
    double: bool,
    tail: Tail,
) -> XArrowDef {
    XArrowDef {
        name,
        glyph,
        left,
        right,
        double,
        tail,
    }
}

/// The row of [`XARROWS`] called `name`.
#[must_use]
pub fn xarrow(name: &str) -> Option<&'static XArrowDef> {
    XARROWS.iter().find(|def| def.name == name)
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
    def("varkappa", "ϰ", Class::Ord),
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
    // amsmath italic capitals (T62): stored italic, so no alphabet
    // changes them.
    def("varGamma", "\u{1D6E4}", Class::Ord),
    def("varDelta", "\u{1D6E5}", Class::Ord),
    def("varTheta", "\u{1D6E9}", Class::Ord),
    def("varLambda", "\u{1D6EC}", Class::Ord),
    def("varXi", "\u{1D6EF}", Class::Ord),
    def("varPi", "\u{1D6F1}", Class::Ord),
    def("varSigma", "\u{1D6F4}", Class::Ord),
    def("varUpsilon", "\u{1D6F6}", Class::Ord),
    def("varPhi", "\u{1D6F7}", Class::Ord),
    def("varPsi", "\u{1D6F9}", Class::Ord),
    def("varOmega", "\u{1D6FA}", Class::Ord),
    // unicode-math capitals that look like Latin letters (T62).
    def("Alpha", "Α", Class::Ord),
    def("Beta", "Β", Class::Ord),
    def("Epsilon", "Ε", Class::Ord),
    def("Zeta", "Ζ", Class::Ord),
    def("Eta", "Η", Class::Ord),
    def("Iota", "Ι", Class::Ord),
    def("Kappa", "Κ", Class::Ord),
    def("Mu", "Μ", Class::Ord),
    def("Nu", "Ν", Class::Ord),
    def("Omicron", "Ο", Class::Ord),
    def("Rho", "Ρ", Class::Ord),
    def("Tau", "Τ", Class::Ord),
    def("Chi", "Χ", Class::Ord),
    // Hebrew letters of set theory (T61): upright, no style maps them.
    def("aleph", "ℵ", Class::Ord),
    def("beth", "ℶ", Class::Ord),
    def("gimel", "ℷ", Class::Ord),
    def("daleth", "ℸ", Class::Ord),
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
    // Big set and logic operators (T66); ⨀ ⨁ ⨂ ⨄ ⨆ come from the T55 subset.
    def("coprod", "∐", Class::BigOp),
    def("bigcup", "⋃", Class::BigOp),
    def("bigcap", "⋂", Class::BigOp),
    def("bigwedge", "⋀", Class::BigOp),
    def("bigvee", "⋁", Class::BigOp),
    def("bigoplus", "⨁", Class::BigOp),
    def("bigotimes", "⨂", Class::BigOp),
    def("bigodot", "⨀", Class::BigOp),
    def("biguplus", "⨄", Class::BigOp),
    def("bigsqcup", "⨆", Class::BigOp),
    def("int", "∫", Class::Integral),
    def("iint", "∬", Class::Integral),
    def("iiint", "∭", Class::Integral),
    def("oint", "∮", Class::Integral),
    def("oiint", "∯", Class::Integral),
    def("min", "min", Class::Limits),
    def("max", "max", Class::Limits),
    def("lim", "lim", Class::Limits),
    // Named operators with limits (T64); TeX's thin space between the
    // words of `\liminf` and `\limsup`.
    def("inf", "inf", Class::Limits),
    def("sup", "sup", Class::Limits),
    def("liminf", "lim\u{2006}inf", Class::Limits),
    def("limsup", "lim\u{2006}sup", Class::Limits),
    def("det", "det", Class::Limits),
    def("Pr", "Pr", Class::Limits),
    def("gcd", "gcd", Class::Limits),
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
    // Combinatorics (T71).
    def("nmid", "∤", Class::Rel),
    def("#", "#", Class::Ord),
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
    def("lcm", "lcm", Class::Op),
    // Operator words with scripts beside them (T64).
    def("exp", "exp", Class::Op),
    def("lg", "lg", Class::Op),
    def("deg", "deg", Class::Op),
    def("dim", "dim", Class::Op),
    def("ker", "ker", Class::Op),
    def("hom", "hom", Class::Op),
    def("arg", "arg", Class::Op),
    // Trigonometric and hyperbolic functions (T63).
    def("sin", "sin", Class::Op),
    def("cos", "cos", Class::Op),
    def("tan", "tan", Class::Op),
    def("cot", "cot", Class::Op),
    def("sec", "sec", Class::Op),
    def("csc", "csc", Class::Op),
    def("arcsin", "arcsin", Class::Op),
    def("arccos", "arccos", Class::Op),
    def("arctan", "arctan", Class::Op),
    def("sinh", "sinh", Class::Op),
    def("cosh", "cosh", Class::Op),
    def("tanh", "tanh", Class::Op),
    def("coth", "coth", Class::Op),
    // Not in LaTeX's base set, but common (T63).
    def("arccot", "arccot", Class::Op),
    def("arcsec", "arcsec", Class::Op),
    def("arccsc", "arccsc", Class::Op),
    def("sech", "sech", Class::Op),
    def("csch", "csch", Class::Op),
    // Set and logic symbols (T65); `\varnothing` and `\barwedge` use the
    // subset's U+2205 and U+22BC.
    def("setminus", "∖", Class::Bin),
    def("complement", "∁", Class::Ord),
    def("varnothing", "∅", Class::Ord),
    def("supset", "⊃", Class::Rel),
    def("subseteq", "⊆", Class::Rel),
    def("supseteq", "⊇", Class::Rel),
    def("subsetneq", "⊊", Class::Rel),
    def("supsetneq", "⊋", Class::Rel),
    def("nsubseteq", "⊈", Class::Rel),
    def("ni", "∋", Class::Rel),
    def("sqcup", "⊔", Class::Bin),
    def("sqcap", "⊓", Class::Bin),
    def("uplus", "⊎", Class::Bin),
    def("lnot", "¬", Class::Ord),
    def("veebar", "⊻", Class::Bin),
    def("barwedge", "⊼", Class::Bin),
    def("top", "⊤", Class::Ord),
    def("bot", "⊥", Class::Ord),
    def("nexists", "∄", Class::Ord),
    // Wide relations: padded in `symbol_box` (ADR-T65-1).
    def("implies", "⟹", Class::Rel),
    def("impliedby", "⟸", Class::Rel),
    def("vdash", "⊢", Class::Rel),
    def("models", "⊨", Class::Rel),
    def("lvert", "|", Class::Open),
    def("rvert", "|", Class::Close),
    def("lVert", "‖", Class::Open),
    def("rVert", "‖", Class::Close),
    // Algebra and group theory (T67); ⨿ comes from the T55 subset. The
    // small triangles are relations here, like `\lhd`.
    def("lhd", "⊲", Class::Rel),
    def("rhd", "⊳", Class::Rel),
    def("unlhd", "⊴", Class::Rel),
    def("unrhd", "⊵", Class::Rel),
    def("triangleleft", "⊲", Class::Rel),
    def("triangleright", "⊳", Class::Rel),
    def("trianglelefteq", "⊴", Class::Rel),
    def("trianglerighteq", "⊵", Class::Rel),
    def("ntriangleleft", "⋪", Class::Rel),
    def("ntriangleright", "⋫", Class::Rel),
    def("rtimes", "⋊", Class::Bin),
    def("ltimes", "⋉", Class::Bin),
    def("wr", "≀", Class::Bin),
    def("odot", "⊙", Class::Bin),
    def("ominus", "⊖", Class::Bin),
    def("oslash", "⊘", Class::Bin),
    def("boxplus", "⊞", Class::Bin),
    def("boxtimes", "⊠", Class::Bin),
    def("amalg", "⨿", Class::Bin),
    def("diamond", "⋄", Class::Bin),
    // `\DeclareMathOperator` names of algebra (T67), drawn like
    // `\operatorname{…}`; no `\char` (a TeX primitive).
    def("Hom", "Hom", Class::Op),
    def("End", "End", Class::Op),
    def("Aut", "Aut", Class::Op),
    def("Inn", "Inn", Class::Op),
    def("Gal", "Gal", Class::Op),
    def("Ker", "Ker", Class::Op),
    def("im", "im", Class::Op),
    def("coker", "coker", Class::Op),
    def("id", "id", Class::Op),
    def("sgn", "sgn", Class::Op),
    def("tr", "tr", Class::Op),
    def("rank", "rank", Class::Op),
    // Category theory (T68): arrows are relations; the long ones come
    // from the T55 subset. `\colon` is punctuation, as in LaTeX.
    def("colon", ":", Class::Punct),
    def("hookrightarrow", "↪", Class::Rel),
    def("hookleftarrow", "↩", Class::Rel),
    def("twoheadrightarrow", "↠", Class::Rel),
    def("twoheadleftarrow", "↞", Class::Rel),
    def("rightarrowtail", "↣", Class::Rel),
    def("leftarrowtail", "↢", Class::Rel),
    def("longrightarrow", "⟶", Class::Rel),
    def("longleftarrow", "⟵", Class::Rel),
    def("longleftrightarrow", "⟷", Class::Rel),
    def("Longrightarrow", "⟹", Class::Rel),
    def("Longleftarrow", "⟸", Class::Rel),
    def("Longleftrightarrow", "⟺", Class::Rel),
    def("longmapsto", "⟼", Class::Rel),
    def("rightrightarrows", "⇉", Class::Rel),
    def("leftleftarrows", "⇇", Class::Rel),
    def("leftrightarrows", "⇆", Class::Rel),
    def("rightleftarrows", "⇄", Class::Rel),
    def("nearrow", "↗", Class::Rel),
    def("searrow", "↘", Class::Rel),
    def("swarrow", "↙", Class::Rel),
    def("nwarrow", "↖", Class::Rel),
    def("Uparrow", "⇑", Class::Rel),
    def("Downarrow", "⇓", Class::Rel),
    def("Updownarrow", "⇕", Class::Rel),
    def("updownarrow", "↕", Class::Rel),
    def("dashv", "⊣", Class::Rel),
    def("op", "op", Class::Op),
    def("Ob", "Ob", Class::Op),
    def("Mor", "Mor", Class::Op),
    def("Box", "□", Class::Ord),
    def("square", "□", Class::Ord),
    def("triangle", "△", Class::Ord),
    def("angle", "∠", Class::Ord),
    def("ohm", "\u{2126}", Class::Ord),
    def("micro", "\u{B5}", Class::Ord),
    def("grad", "∇", Class::Ord),
    // Quantum notation (T70): `\dag` as `physics` uses it in scripts,
    // the trace beside `\tr`.
    def("dag", "†", Class::Ord),
    def("Tr", "Tr", Class::Op),
];

/// The table entry of command `name` (without the backslash).
#[must_use]
pub fn symbol(name: &str) -> Option<&'static SymbolDef> {
    SYMBOLS.iter().find(|s| s.name == name)
}

/// What an accent draws over its argument (ADR-T39-1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mark {
    /// A math-face glyph at this fraction of the argument's scale.
    Glyph(char, f32),
    /// A rule [`ACCENT_BAR`](super::ACCENT_BAR) long, centred over the argument: `\bar`.
    Bar,
    /// A rule from the argument's left edge to its advance: `\overline`.
    Overline,
}

/// One accent command: a one-argument [`Node::Command`](super::Node::Command).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentDef {
    /// Command name without the backslash.
    pub name: &'static str,
    /// What it draws.
    pub mark: Mark,
}

/// Every accent command of the subset (ADR-T39-1).
pub const ACCENTS: &[AccentDef] = &[
    AccentDef {
        name: "vec",
        mark: Mark::Glyph('\u{2192}', VEC_SCALE),
    },
    AccentDef {
        name: "hat",
        mark: Mark::Glyph('^', 1.0),
    },
    AccentDef {
        name: "tilde",
        mark: Mark::Glyph('~', 1.0),
    },
    AccentDef {
        name: "dot",
        mark: Mark::Glyph('\u{B7}', 1.0),
    },
    AccentDef {
        name: "ddot",
        mark: Mark::Glyph('\u{A8}', 1.0),
    },
    AccentDef {
        name: "dddot",
        mark: Mark::Glyph('\u{20DB}', 1.0),
    },
    AccentDef {
        name: "bar",
        mark: Mark::Bar,
    },
    AccentDef {
        name: "overline",
        mark: Mark::Overline,
    },
];

/// The accent command `name` (without the backslash).
#[must_use]
pub fn accent(name: &str) -> Option<&'static AccentDef> {
    ACCENTS.iter().find(|a| a.name == name)
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

/// A kind of character that a math alphabet maps (ADR-T54-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Latin capitals `A`–`Z`.
    Upper,
    /// Latin lowercase `a`–`z`.
    Lower,
    /// Digits `0`–`9`.
    Digit,
    /// Greek capitals `Α`–`Ω`.
    GreekUpper,
    /// Greek lowercase `α`–`ω` and the variants `ϵ ϑ ϰ ϕ ϱ ϖ`.
    GreekLower,
}

impl Kind {
    /// Every kind, in declaration order (the order of a style's slots).
    pub const ALL: [Self; 5] = [
        Self::Upper,
        Self::Lower,
        Self::Digit,
        Self::GreekUpper,
        Self::GreekLower,
    ];

    /// The kind of `ch` and its index in that kind's run of a Unicode
    /// math alphabet (`A` → 0, `ϵ` → 26); `None` for any other character.
    #[must_use]
    pub fn of(ch: char) -> Option<(Self, u32)> {
        let from = |first: char| u32::from(ch) - u32::from(first);
        let kind = match ch {
            'A'..='Z' => (Self::Upper, from('A')),
            'a'..='z' => (Self::Lower, from('a')),
            '0'..='9' => (Self::Digit, from('0')),
            // Unassigned: no final sigma among the capitals.
            '\u{3A2}' => return None,
            'Α'..='Ω' => (Self::GreekUpper, from('Α')),
            'α'..='ω' => (Self::GreekLower, from('α')),
            // After `ω` and `∂` in every math alphabet.
            'ϵ' => (Self::GreekLower, 26),
            'ϑ' => (Self::GreekLower, 27),
            'ϰ' => (Self::GreekLower, 28),
            'ϕ' => (Self::GreekLower, 29),
            'ϱ' => (Self::GreekLower, 30),
            'ϖ' => (Self::GreekLower, 31),
            _ => return None,
        };
        Some(kind)
    }

    /// The slot of this kind in [`Kind::ALL`].
    pub(super) const fn slot(self) -> usize {
        self as usize
    }
}

/// How a math alphabet draws one kind of character (ADR-T54-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Map {
    /// Not mapped: the enclosing alphabet decides.
    Inherit,
    /// The character as typed.
    Upright,
    /// The run of the Unicode math alphabet starting at this code point.
    Block(u32),
}

/// One math alphabet: a command and how it maps each kind of character
/// (ADR-T54-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlphabetDef {
    /// Command name without the backslash.
    pub name: &'static str,
    /// Latin capitals.
    pub upper: Map,
    /// Latin lowercase.
    pub lower: Map,
    /// Digits.
    pub digit: Map,
    /// Greek capitals.
    pub greek_upper: Map,
    /// Greek lowercase and its variants.
    pub greek_lower: Map,
    /// Characters drawn from elsewhere than the block (Letterlike
    /// Symbols), as `(typed, drawn)`; used where the row maps a `Block`.
    pub holes: &'static [(char, char)],
    /// Whether the letters this row maps lean right: an accent over one
    /// of them is skewed (ADR-T56-1). Digits are always upright.
    pub slant: bool,
    /// Whether this row is a bold weight (`\mathbf`, `\boldsymbol`,
    /// `\bm`, bold sans): `\mathsf` inside it is bold sans (ADR-T60-1).
    pub bold: bool,
}

impl AlphabetDef {
    /// How this alphabet maps `kind`.
    #[must_use]
    pub const fn get(&self, kind: Kind) -> Map {
        match kind {
            Kind::Upper => self.upper,
            Kind::Lower => self.lower,
            Kind::Digit => self.digit,
            Kind::GreekUpper => self.greek_upper,
            Kind::GreekLower => self.greek_lower,
        }
    }

    /// `ch` in this alphabet; `None` when the alphabet does not map its
    /// kind or it has none.
    #[must_use]
    pub fn map(&self, ch: char) -> Option<char> {
        let (kind, index) = Kind::of(ch)?;
        match self.get(kind) {
            Map::Inherit => None,
            Map::Upright => Some(ch),
            Map::Block(start) => {
                let hole = self.holes.iter().find(|&&(typed, _)| typed == ch);
                let block = || start.checked_add(index).and_then(char::from_u32);
                Some(hole.map(|&(_, drawn)| drawn).or_else(block).unwrap_or(ch))
            }
        }
    }
}

/// Start of the italic capitals `𝐴`.
const ITALIC_UPPER: Map = Map::Block(0x1D434);
/// Start of the italic lowercase `𝑎`.
const ITALIC_LOWER: Map = Map::Block(0x1D44E);
/// Start of the italic Greek capitals `𝛤`.
const ITALIC_GREEK_UPPER: Map = Map::Block(0x1D6E2);
/// Start of the italic Greek lowercase `𝛼`.
const ITALIC_GREEK_LOWER: Map = Map::Block(0x1D6FC);
/// The italic h is the older Planck constant.
const ITALIC_HOLES: &[(char, char)] = &[('h', '\u{210E}')];

/// Math italic, the style a formula starts in; not a command (empty
/// name). It maps every kind, so every fallback ends here (ADR-T54-1).
pub const MATH_ITALIC: AlphabetDef = AlphabetDef {
    name: "",
    upper: ITALIC_UPPER,
    lower: ITALIC_LOWER,
    digit: Map::Upright,
    greek_upper: Map::Upright,
    greek_lower: ITALIC_GREEK_LOWER,
    holes: ITALIC_HOLES,
    slant: true,
    bold: false,
};

/// `\mathcal`: script capitals, with the holes Unicode filled before the
/// math block.
const MATHCAL: AlphabetDef = AlphabetDef {
    name: "mathcal",
    upper: Map::Block(0x1D49C),
    lower: Map::Inherit,
    digit: Map::Inherit,
    greek_upper: Map::Inherit,
    greek_lower: Map::Inherit,
    holes: &[
        ('B', '\u{212C}'),
        ('E', '\u{2130}'),
        ('F', '\u{2131}'),
        ('H', '\u{210B}'),
        ('I', '\u{2110}'),
        ('L', '\u{2112}'),
        ('M', '\u{2133}'),
        ('R', '\u{211B}'),
    ],
    slant: true,
    bold: false,
};

/// Every math alphabet command (ADR-T54-1).
pub const ALPHABETS: &[AlphabetDef] = &[
    MATHCAL,
    // Upright roman; Greek as outside, as in LaTeX.
    AlphabetDef {
        name: "mathrm",
        upper: Map::Upright,
        lower: Map::Upright,
        digit: Map::Upright,
        greek_upper: Map::Inherit,
        greek_lower: Map::Inherit,
        holes: &[],
        slant: false,
        bold: false,
    },
    // Italic letters set as a word; digits as outside.
    AlphabetDef {
        name: "mathit",
        upper: ITALIC_UPPER,
        lower: ITALIC_LOWER,
        digit: Map::Inherit,
        greek_upper: ITALIC_GREEK_UPPER,
        greek_lower: ITALIC_GREEK_LOWER,
        holes: ITALIC_HOLES,
        slant: true,
        bold: false,
    },
    // The default style, with italic Greek capitals.
    AlphabetDef {
        name: "mathnormal",
        greek_upper: ITALIC_GREEK_UPPER,
        ..MATH_ITALIC
    },
    // Upright bold; lowercase Greek as outside, as in LaTeX (T56).
    AlphabetDef {
        name: "mathbf",
        upper: Map::Block(0x1D400),
        lower: Map::Block(0x1D41A),
        digit: BOLD_DIGIT,
        greek_upper: Map::Block(0x1D6A8),
        greek_lower: Map::Inherit,
        holes: &[],
        slant: false,
        bold: true,
    },
    BOLDSYMBOL,
    AlphabetDef {
        name: "bm",
        ..BOLDSYMBOL
    },
    // Blackboard bold; dsfont's \mathds is the same row (T57).
    MATHBB,
    AlphabetDef {
        name: "mathds",
        ..MATHBB
    },
    // Fraktur letters (T58).
    MATHFRAK,
    // Script capitals, as \mathcal (T59).
    MATHSCR,
    // Sans-serif and typewriter (T60).
    MATHSF,
    MATHTT,
];

/// Start of the bold digits `𝟎`.
const BOLD_DIGIT: Map = Map::Block(0x1D7CE);

/// `\boldsymbol` (and `\bm`): bold italic Latin and Greek, bold digits
/// (T56). Bold italic has no holes.
const BOLDSYMBOL: AlphabetDef = AlphabetDef {
    name: "boldsymbol",
    upper: Map::Block(0x1D468),
    lower: Map::Block(0x1D482),
    digit: BOLD_DIGIT,
    greek_upper: Map::Block(0x1D71C),
    greek_lower: Map::Block(0x1D736),
    holes: &[],
    slant: true,
    bold: true,
};

/// `\mathbb` (and `\mathds`): double-struck Latin and digits, upright;
/// Greek as outside. More than amssymb's capitals (ADR-T57-1).
const MATHBB: AlphabetDef = AlphabetDef {
    name: "mathbb",
    upper: Map::Block(0x1D538),
    lower: Map::Block(0x1D552),
    digit: Map::Block(0x1D7D8),
    greek_upper: Map::Inherit,
    greek_lower: Map::Inherit,
    holes: &[
        ('C', '\u{2102}'),
        ('H', '\u{210D}'),
        ('N', '\u{2115}'),
        ('P', '\u{2119}'),
        ('Q', '\u{211A}'),
        ('R', '\u{211D}'),
        ('Z', '\u{2124}'),
    ],
    slant: false,
    bold: false,
};

/// `\mathfrak`: fraktur Latin capitals and lowercase, upright; digits and
/// Greek as outside (Unicode has none). Five capitals are Letterlike
/// holes (T58).
const MATHFRAK: AlphabetDef = AlphabetDef {
    name: "mathfrak",
    upper: Map::Block(0x1D504),
    lower: Map::Block(0x1D51E),
    digit: Map::Inherit,
    greek_upper: Map::Inherit,
    greek_lower: Map::Inherit,
    holes: &[
        ('C', '\u{212D}'),
        ('H', '\u{210C}'),
        ('I', '\u{2111}'),
        ('R', '\u{211C}'),
        ('Z', '\u{2128}'),
    ],
    slant: false,
    bold: false,
};

/// `\mathscr`: the `\mathcal` row under its own name. Latin Modern Math
/// has one script style and no script lowercase, so lowercase is
/// inherited as in `\mathcal` (ADR-T59-1).
const MATHSCR: AlphabetDef = AlphabetDef {
    name: "mathscr",
    ..MATHCAL
};

/// `\mathsf`: sans-serif Latin and digits, upright; Greek as outside
/// (T60).
pub(super) const MATHSF: AlphabetDef = AlphabetDef {
    name: "mathsf",
    upper: Map::Block(0x1D5A0),
    lower: Map::Block(0x1D5BA),
    digit: Map::Block(0x1D7E2),
    greek_upper: Map::Inherit,
    greek_lower: Map::Inherit,
    holes: &[],
    slant: false,
    bold: false,
};

/// Bold sans-serif Latin and digits, upright; Greek as outside. Not a
/// command: `\mathsf` inside a bold alphabet, or a bold alphabet inside
/// `\mathsf`, draws it (ADR-T60-1).
pub(super) const MATHSF_BOLD: AlphabetDef = AlphabetDef {
    upper: Map::Block(0x1D5D4),
    lower: Map::Block(0x1D5EE),
    digit: Map::Block(0x1D7EC),
    bold: true,
    ..MATHSF
};

/// `\mathtt`: monospace Latin and digits, upright; Greek as outside.
/// Unicode has no bold monospace (T60).
const MATHTT: AlphabetDef = AlphabetDef {
    name: "mathtt",
    upper: Map::Block(0x1D670),
    lower: Map::Block(0x1D68A),
    digit: Map::Block(0x1D7F6),
    ..MATHSF
};

/// The math alphabet command `name` (without the backslash).
#[must_use]
pub fn alphabet(name: &str) -> Option<&'static AlphabetDef> {
    ALPHABETS.iter().find(|a| a.name == name)
}

/// The math italic form of `ch`: Latin letters and lowercase Greek;
/// anything else as is (ADR-T33-1, ADR-T54-1).
#[must_use]
pub fn italic(ch: char) -> char {
    Style::ITALIC.map(ch)
}

/// The script (`\mathcal`) form of `ch`: Latin capitals; anything else as
/// [`italic`] draws it (ADR-T33-1, ADR-T54-1).
#[must_use]
pub fn calligraphic(ch: char) -> char {
    Style::ITALIC.with(&MATHCAL).map(ch)
}
