//! Typesetting the node tree into boxes and ink.

use super::{
    ACCENT_BAR, ACCENT_GAP, ACCENT_SKEW, ADVANCE, ARRAY_COLUMN_GAP, ARRAY_ROW_ASCENT,
    ARRAY_ROW_DESCENT, AXIS, Aabb, AccentDef, Align, AlphabetDef, Arg, BASELINE, BIG_OP_SCALE,
    BIN_SPACE, BINOM_MAX_SCALE, Class, DELIM_FACTOR, DELIMITERS, DISPLAY_OP_SCALE, Delimiter,
    EnvironmentDef, FRAC_DEN_SHIFT, FRAC_GAP, FRAC_NUM_SHIFT, FRAC_PAD, Face, Kind,
    LEFT_RIGHT_MAX_SCALE, LIMIT_GAP, LINE_HEIGHT, MATH_ITALIC, MATH_PAD, MATHSF, MATHSF_BOLD,
    MIN_SCALE, MINUS, Map, Mark, NULL_DELIM_SPACE, Node, OP_SPACE, REL_SPACE, RULE, SCRIPT_GAP,
    SCRIPT_SCALE, SCRIPT_SPACE, SQRT_GAP, SQRT_SIGN, SUB_DROP, SUB_SHIFT, SUB_TOP_MAX,
    SUP_BOTTOM_MIN, SUP_DROP, SUP_SHIFT, ScriptKind, Segment, SymbolDef, Tail, Vec2,
    WIDE_REL_SPACE, WIDE_RELATIONS, XARROW_GAP, XARROW_HOOK_SEGMENTS, XARROW_PAD, XArrowDef,
    accent, alphabet, environment, expand, glyph_extent, kern, math_glyph_metrics, math_metrics,
    symbol, xarrow,
};

/// How the characters of a formula part are drawn: for each [`Kind`], the
/// alphabet that decides it (ADR-T54-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Style {
    /// One alphabet per slot of [`Kind::ALL`].
    rows: [&'static AlphabetDef; Kind::ALL.len()],
    /// Display style rather than text style (ADR-T105-1).
    display: bool,
}

impl Style {
    /// The style a `$…$` formula starts in.
    pub(super) const ITALIC: Self = Self {
        rows: [&MATH_ITALIC; Kind::ALL.len()],
        display: false,
    };

    /// This style in display (`true`) or text (`false`) style, the
    /// alphabets kept.
    const fn displayed(mut self, display: bool) -> Self {
        self.display = display;
        self
    }

    /// This style inside `alphabet`: the kinds it maps are its own, the
    /// others stay as they are. Sans-serif and a bold weight, nested
    /// either way round, give bold sans (ADR-T60-1).
    pub(super) fn with(mut self, alphabet: &'static AlphabetDef) -> Self {
        for kind in Kind::ALL {
            if alphabet.get(kind) != Map::Inherit {
                let row = &mut self.rows[kind.slot()];
                let sans = |a: &AlphabetDef| a.name == MATHSF.name;
                *row = if (sans(alphabet) && row.bold) || (alphabet.bold && sans(row)) {
                    &MATHSF_BOLD
                } else {
                    alphabet
                };
            }
        }
        self
    }

    /// Whether `ch` typed in a formula part of this style is drawn as a
    /// different, slanted glyph: an accent over it is skewed (ADR-T56-1).
    fn slanted(self, ch: char) -> bool {
        // Every math alphabet's digits are upright.
        Kind::of(ch).is_some_and(|(kind, _)| {
            let row = self.rows[kind.slot()];
            kind != Kind::Digit && row.slant && row.map(ch).is_some_and(|drawn| drawn != ch)
        })
    }

    /// `ch` typed in a formula part of this style.
    pub(super) fn map(self, ch: char) -> char {
        Kind::of(ch)
            .and_then(|(kind, _)| self.rows[kind.slot()].map(ch))
            .unwrap_or(ch)
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
// ---- layout -----------------------------------------------------------------

/// Scale of a script of something at `scale`.
pub(super) fn script_scale(scale: f32) -> f32 {
    (scale * SCRIPT_SCALE).max(MIN_SCALE).min(scale)
}

/// Lays out the nodes of one formula at `scale` (1 for the block's font
/// size).
#[must_use]
pub fn layout_math(nodes: &[Node], scale: f32) -> MathBox {
    layout_styled(nodes, scale, Style::ITALIC)
}

/// Lays out the nodes of one `$$…$$` formula at `scale` in display style
/// (ADR-T105-1).
#[must_use]
pub fn layout_display(nodes: &[Node], scale: f32) -> MathBox {
    layout_styled(nodes, scale, Style::ITALIC.displayed(true))
}

/// [`layout_math`] with letters drawn in `style`.
fn layout_styled(nodes: &[Node], scale: f32, style: Style) -> MathBox {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let mut atoms: Vec<(MathBox, Class)> = Vec::with_capacity(nodes.len());
    // The kerns typed before each atom, then those after the last one
    // (ADR-T99-1): one more entry than atoms.
    let mut kerns: Vec<f32> = vec![0.0];
    push_atoms(nodes, scale, style, &mut atoms, &mut kerns);
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
    let mut boxes = Vec::with_capacity(atoms.len() * 3 + 1);
    // Width laid out so far: a negative kern never takes the cursor left
    // of the row's start (ADR-T99-1).
    let mut cursor = 0.0_f32;
    let mut prev: Option<Class> = None;
    for ((b, class), kern) in atoms.into_iter().zip(kerns.iter().copied()) {
        if let Some(prev) = prev {
            let space = spacing(prev, class, scale);
            if space > 0.0 {
                push_box(&mut boxes, &mut cursor, MathBox::gap(space));
            }
        }
        push_kern(&mut boxes, &mut cursor, kern);
        prev = Some(class);
        push_box(&mut boxes, &mut cursor, b);
    }
    push_kern(
        &mut boxes,
        &mut cursor,
        kerns.last().copied().unwrap_or(0.0),
    );
    MathBox::row(boxes)
}

/// Pushes `b` onto a row being built and moves its `cursor` past it.
fn push_box(boxes: &mut Vec<MathBox>, cursor: &mut f32, b: MathBox) {
    *cursor += b.width;
    boxes.push(b);
}

/// Pushes a kern `width` wide onto a row being built, cut so the cursor
/// never goes left of the row's start (ADR-T99-1).
fn push_kern(boxes: &mut Vec<MathBox>, cursor: &mut f32, width: f32) {
    let width = width.max(-*cursor);
    if width.is_finite() && width != 0.0 {
        push_box(boxes, cursor, MathBox::gap(width));
    }
}

/// The atoms of `nodes` onto `out`, with each shortcut's long form
/// spliced in among its neighbours (ADR-T69-1). Kerns add to the last
/// entry of `kerns`, and each atom opens a new one (ADR-T99-1). A style
/// switch is no atom: it changes the style of the nodes after it
/// (ADR-T105-1).
fn push_atoms(
    nodes: &[Node],
    scale: f32,
    mut style: Style,
    out: &mut Vec<(MathBox, Class)>,
    kerns: &mut Vec<f32>,
) {
    for node in nodes {
        if let Node::MathStyle(display) = node {
            style = style.displayed(*display);
            continue;
        }
        if let Some(width) = kern_width(node, scale) {
            if let Some(last) = kerns.last_mut() {
                *last += width;
            }
            continue;
        }
        match expand(node) {
            Some(long) => push_atoms(&long, scale, style, out, kerns),
            None => {
                if let Some(a) = atom(node, scale, style) {
                    out.push(a);
                    kerns.push(0.0);
                }
            }
        }
    }
}

/// The width of a [`Node::Kern`] at `scale`; `None` for other nodes.
fn kern_width(node: &Node, scale: f32) -> Option<f32> {
    match node {
        Node::Kern(source) => Some(kern(source).unwrap_or(0.0) * scale),
        Node::Space(_)
        | Node::Char(_)
        | Node::Symbol(_)
        | Node::Group(_)
        | Node::Scripts { .. }
        | Node::Command { .. }
        | Node::Text { .. }
        | Node::OperatorName { .. }
        | Node::Infix { .. }
        | Node::LeftRight { .. }
        | Node::Environment { .. }
        | Node::XArrow { .. }
        | Node::MathStyle(_)
        | Node::Literal(_) => None,
    }
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
    // A shortcut on its own (a script's base) is one ordinary atom.
    if let Some(long) = expand(node) {
        return Some((layout_styled(&long, scale, style), Class::Ord));
    }
    let atom = match node {
        Node::Space(_) => return None,
        // A kern on its own (a script's base) is an empty box; never
        // negative, so nothing hangs left of it (ADR-T99-1).
        Node::Kern(_) => (
            MathBox::gap(kern_width(node, scale).unwrap_or(0.0).max(0.0)),
            Class::Ord,
        ),
        Node::Char(ch) => {
            let shown = if *ch == '-' { MINUS } else { style.map(*ch) };
            (MathBox::math(shown, scale), char_class(*ch))
        }
        Node::Symbol(name) => match symbol(name) {
            Some(def) => (symbol_box(def, scale, style), def.class),
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
                // Scripts are never in display style (ADR-T105-1).
                let b = layout_styled(&script.arg.nodes, ss, style.displayed(false));
                match script.kind {
                    ScriptKind::Sup => sup = Some(b),
                    ScriptKind::Sub => sub = Some(b),
                }
            }
            (attach_scripts(base, class, sup, sub, scale), class)
        }
        Node::Command { name, args } => {
            let arg = |i: usize| args.get(i).map_or(&[][..], |a: &Arg| &a.nodes[..]);
            let b = match name.as_str() {
                "frac" => fraction(arg(0), arg(1), scale, true, style),
                // `d` forces display parts, `t` text parts (ADR-T105-1).
                "dfrac" => fraction(arg(0), arg(1), scale, true, style.displayed(true)),
                "tfrac" => fraction(arg(0), arg(1), scale, true, style.displayed(false)),
                "binom" => binomial(arg(0), arg(1), scale, style),
                "dbinom" => binomial(arg(0), arg(1), scale, style.displayed(true)),
                "tbinom" => binomial(arg(0), arg(1), scale, style.displayed(false)),
                "multiset" => delimited(arg(0), arg(1), "((", "))", scale, style),
                "sqrt" => root(arg(0), scale, style),
                "underline" => underlined(arg(0), scale, style),
                // TeX's \pmod: a quad, "(mod", a third of a quad.
                "pmod" => MathBox::row([
                    MathBox::gap(scale),
                    MathBox::upright("(mod", scale),
                    MathBox::gap(scale / 3.0),
                    layout_styled(arg(0), scale, style),
                    MathBox::math(')', scale),
                ]),
                _ => match (alphabet(name), accent(name)) {
                    (Some(a), _) => layout_styled(arg(0), scale, style.with(a)),
                    (None, Some(def)) => accented(def, arg(0), scale, style),
                    (None, None) => MathBox::word(&format!("\\{name}"), scale, true),
                },
            };
            (b, Class::Ord)
        }
        Node::Text { body, .. } => (MathBox::upright(body, scale), Class::Ord),
        Node::OperatorName { star, body, .. } => {
            let class = if *star { Class::Limits } else { Class::Op };
            (MathBox::upright(body, scale), class)
        }
        Node::Infix { name, left, right } => (infix(name, left, right, scale, style), Class::Ord),
        Node::LeftRight { open, body, close } => {
            (left_right(open, body, close, scale, style), Class::Ord)
        }
        Node::Environment { name, rows, .. } => {
            (environment_box(name, rows, scale, style), Class::Ord)
        }
        Node::XArrow { name, below, above } => {
            let below = below.as_ref().map_or(&[][..], |b| &b.nodes[..]);
            let b = match xarrow(name) {
                Some(def) => extensible_arrow(def, &above.nodes, below, scale, style),
                None => MathBox::word(&format!("\\{name}"), scale, true),
            };
            (b, Class::Rel)
        }
        Node::Literal(text) => (MathBox::word(text, scale, true), Class::Ord),
        // A style switch on its own (a script's base) is an empty box.
        Node::MathStyle(_) => (MathBox::default(), Class::Ord),
    };
    Some(atom)
}

/// Extensible arrow `def` with label `above` over it and `below` under
/// it, at `scale` (ADR-T103-1).
fn extensible_arrow(
    def: &XArrowDef,
    above: &[Node],
    below: &[Node],
    scale: f32,
    style: Style,
) -> MathBox {
    let plain = MathBox::math(def.glyph, scale);
    let ss = script_scale(scale);
    let style = style.displayed(false);
    let (up, down) = (
        layout_styled(above, ss, style),
        layout_styled(below, ss, style),
    );
    if up.ink.is_empty() && down.ink.is_empty() && up.width <= 0.0 && down.width <= 0.0 {
        return plain;
    }
    let label = up.width.max(down.width);
    let width = plain.width.max(label + 2.0 * XARROW_PAD * scale);
    let mut out = if width > plain.width {
        stretched_arrow(def, width, scale)
    } else {
        plain
    };
    let gap = XARROW_GAP * scale;
    let (top, bottom) = (out.ascent, out.descent);
    if !up.ink.is_empty() {
        let y = -top - gap - up.descent;
        out.put(&up, Vec2::new((width - up.width) / 2.0, y));
        out.ascent = out.ascent.max(up.ascent - y);
    }
    if !down.ink.is_empty() {
        let y = bottom + gap + down.ascent;
        out.put(&down, Vec2::new((width - down.width) / 2.0, y));
        out.descent = out.descent.max(y + down.descent);
    }
    out.italic = 0.0;
    out
}

/// Extensible arrow `def` stretched to `width` at `scale`: its head
/// glyphs at their natural size, a shaft of rules overlapping them and
/// its tail (ADR-T103-1).
fn stretched_arrow(def: &XArrowDef, width: f32, scale: f32) -> MathBox {
    let plain = MathBox::math(def.glyph, scale);
    let (x_min, right_bearing) =
        math_metrics::get(def.glyph).map_or((0.0, 0.0), |m| (m.x_min, m.advance - m.x_max));
    let t = RULE * scale;
    let mut out = MathBox {
        width,
        ascent: plain.ascent,
        descent: plain.descent,
        italic: 0.0,
        ink: Vec::new(),
    };
    // The shaft runs from `start` to `end`, through the middle of each
    // head so it overlaps the glyph's own shaft.
    let start = if let Some(ch) = def.left {
        let head = MathBox::math(ch, scale);
        out.put(&head, Vec2::ZERO);
        head.width / 2.0
    } else {
        let edge = x_min * scale;
        match def.tail {
            Tail::Plain => edge,
            Tail::Bar => {
                let x = edge + t / 2.0;
                out.ink.push(Ink::Rule {
                    a: Vec2::new(x, plain.descent),
                    b: Vec2::new(x, -plain.ascent),
                    width: t,
                });
                x
            }
            Tail::Hook => hook(&mut out, edge + t / 2.0, plain.ascent - t / 2.0, scale),
        }
    };
    let end = if let Some(ch) = def.right {
        let head = MathBox::math(ch, scale);
        out.put(&head, Vec2::new(width - head.width, 0.0));
        width - head.width / 2.0
    } else {
        width - right_bearing * scale
    };
    let bars = if def.double {
        let (lo, hi) = math_metrics::get('=').map_or((AXIS - 0.097, AXIS + 0.097), |eq| {
            (eq.y_min + RULE / 2.0, eq.y_max - RULE / 2.0)
        });
        vec![lo, hi]
    } else {
        vec![AXIS]
    };
    for bar in bars {
        let y = -bar * scale;
        out.ink.push(Ink::Rule {
            a: Vec2::new(start, y),
            b: Vec2::new(end, y),
            width: t,
        });
    }
    // Rules are counted with half their width all round, as their bounds.
    let (mut ascent, mut descent) = (out.ascent, out.descent);
    for ink in &out.ink {
        let b = ink.bounds();
        ascent = ascent.max(-b.min.y);
        descent = descent.max(b.max.y);
    }
    out.ascent = ascent;
    out.descent = descent;
    out
}

/// Pushes onto `out` the hook of a stretched `\xhookrightarrow`: a half
/// circle of [`XARROW_HOOK_SEGMENTS`] rules above the shaft, its leftmost
/// point at `left` and its top at `top` above the baseline. Returns the x
/// where the shaft starts, under the hook's end (ADR-T103-1).
fn hook(out: &mut MathBox, left: f32, top: f32, scale: f32) -> f32 {
    let axis = AXIS * scale;
    let r = ((top - axis) / 2.0).max(0.0);
    let (cx, cy) = (left + r, axis + r);
    // y up from the bottom of the circle (on the shaft) round its left
    // side to its top.
    let point = |k: usize| {
        let phi = std::f32::consts::PI * (1.5 - k as f32 / XARROW_HOOK_SEGMENTS as f32);
        Vec2::new(cx + r * phi.cos(), -(cy + r * phi.sin()))
    };
    for k in 0..XARROW_HOOK_SEGMENTS {
        out.ink.push(Ink::Rule {
            a: point(k),
            b: point(k + 1),
            width: RULE * scale,
        });
    }
    cx
}

/// The box of a table symbol: a glyph (a letter in `style`), an enlarged
/// big operator or integral, or an upright word.
fn symbol_box(def: &SymbolDef, scale: f32, style: Style) -> MathBox {
    let b = match (letter(def), def.class) {
        (Some(ch), Class::BigOp | Class::Integral) => big_operator(ch, scale, style.display),
        (Some(ch), Class::Ord) => MathBox::math(style.map(ch), scale),
        (Some(ch), _) => MathBox::math(ch, scale),
        (None, _) => MathBox::upright(def.text, scale),
    };
    if WIDE_RELATIONS.contains(&def.name) {
        // amsmath's `\;` each side, at every size (ADR-T65-1).
        let pad = WIDE_REL_SPACE * scale;
        MathBox::row([MathBox::gap(pad), b, MathBox::gap(pad)])
    } else {
        b
    }
}

/// The one character a table symbol draws; `None` for a word.
fn letter(def: &SymbolDef) -> Option<char> {
    let mut chars = def.text.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) => Some(ch),
        _ => None,
    }
}

/// A big operator (`∑`, `⋃`, …) or integral, enlarged and centred on the
/// axis, with the enlarged glyph's italic correction; larger in
/// `display` style (ADR-T105-1).
fn big_operator(ch: char, scale: f32, display: bool) -> MathBox {
    let k = if display {
        DISPLAY_OP_SCALE
    } else {
        BIG_OP_SCALE
    } * scale;
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

/// `base` of `class` with its superscript and subscript: limits above and
/// below for big operators, beside it for every other class.
fn attach_scripts(
    base: MathBox,
    class: Class,
    sup: Option<MathBox>,
    sub: Option<MathBox>,
    scale: f32,
) -> MathBox {
    match class {
        Class::BigOp | Class::Limits => limits(&base, sup, sub, scale),
        // The advance already holds the italic correction (ADR-T41-1); the
        // subscript tucks under the slant (ADR-T36-1).
        Class::Integral
        | Class::Ord
        | Class::Op
        | Class::Bin
        | Class::Rel
        | Class::Open
        | Class::Close
        | Class::Punct => {
            let (kern, tuck) = if matches!(class, Class::Integral) {
                (0.0, base.italic)
            } else {
                (base.italic, 0.0)
            };
            scripts(base, sup, sub, kern, tuck, scale)
        }
    }
}

/// Superscript and subscript to the right of `base` (ADR-T33-1): the
/// superscript `kern` right of the base's advance (ADR-T41-1), the
/// subscript `tuck` left of it (ADR-T36-1).
fn scripts(
    base: MathBox,
    sup: Option<MathBox>,
    sub: Option<MathBox>,
    kern: f32,
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
        let at = x + kern;
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

/// A fraction of `num` over `den`, with a bar on the axis when `bar`. Its
/// parts are at script scale, or at full size in text style when `style`
/// is display style (ADR-T105-1).
fn fraction(num: &[Node], den: &[Node], scale: f32, bar: bool, style: Style) -> MathBox {
    let (ps, style) = if style.display {
        (scale, style.displayed(false))
    } else {
        (script_scale(scale), style)
    };
    let (n, d) = (layout_styled(num, ps, style), layout_styled(den, ps, style));
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
    delimited(n, k, "(", ")", scale, style)
}

/// `n` over `k` without a bar, between the brackets of `open` and
/// `close` (side by side), all as tall as the stack.
fn delimited(n: &[Node], k: &[Node], open: &str, close: &str, scale: f32, style: Style) -> MathBox {
    let inner = fraction(n, k, scale, false, style);
    let height = inner.ascent + inner.descent;
    let first = open.chars().next().unwrap_or('(');
    let unit = MathBox::math(first, 1.0);
    let tall = (unit.ascent + unit.descent).max(f32::EPSILON);
    let size = (height / tall).clamp(scale, BINOM_MAX_SCALE * scale);
    let brackets = |s: &str| {
        s.chars()
            .map(|ch| MathBox::math(ch, size))
            .collect::<Vec<_>>()
    };
    let (opens, closes) = (brackets(open), brackets(close));
    // The brackets' middle sits on the middle of the stack.
    let middle = (inner.descent - inner.ascent) / 2.0;
    let mut out = MathBox {
        width: 0.0,
        ascent: inner.ascent,
        descent: inner.descent,
        italic: 0.0,
        ink: Vec::new(),
    };
    let mut place = |b: &MathBox, centred: bool| {
        let y = if centred {
            middle + (b.ascent - b.descent) / 2.0
        } else {
            0.0
        };
        out.put(b, Vec2::new(out.width, y));
        out.width += b.width;
        out.ascent = out.ascent.max(b.ascent - y);
        out.descent = out.descent.max(b.descent + y);
    };
    for b in &opens {
        place(b, true);
    }
    place(&inner, false);
    for b in &closes {
        place(b, true);
    }
    out
}

/// `body` between brackets grown to cover it about the axis (ADR-T89-1).
fn left_right(
    open: &Delimiter,
    body: &[Node],
    close: &Delimiter,
    scale: f32,
    style: Style,
) -> MathBox {
    fenced(
        &open.source,
        layout_styled(body, scale, style),
        &close.source,
        scale,
    )
}

/// `inner` between the delimiters `open` and `close`, grown to cover it
/// about the axis (ADR-T89-1).
fn fenced(open: &str, inner: MathBox, close: &str, scale: f32) -> MathBox {
    let axis = AXIS * scale;
    let span = 2.0 * (inner.ascent - axis).max(inner.descent + axis) * DELIM_FACTOR;
    let (left, right) = (
        grown_bracket(open, span, scale),
        grown_bracket(close, span, scale),
    );
    MathBox::row([left, inner, right])
}

/// The glyph of delimiter `source` scaled to `span`, centred on the axis,
/// at least at `scale`; a small gap for `.`.
fn grown_bracket(source: &str, span: f32, scale: f32) -> MathBox {
    let glyph = DELIMITERS
        .iter()
        .find(|(known, _)| *known == source)
        .and_then(|(_, glyph)| *glyph);
    let Some(ch) = glyph else {
        return MathBox::gap(NULL_DELIM_SPACE * scale);
    };
    let unit = MathBox::math(ch, 1.0);
    let tall = (unit.ascent + unit.descent).max(f32::EPSILON);
    let size = (span / tall).clamp(scale, LEFT_RIGHT_MAX_SCALE * scale);
    let bracket = MathBox::math(ch, size);
    let y = -AXIS * scale - (bracket.descent - bracket.ascent) / 2.0;
    let mut out = MathBox {
        width: bracket.width,
        ascent: bracket.ascent - y,
        descent: bracket.descent + y,
        italic: 0.0,
        ink: Vec::new(),
    };
    out.put(&bracket, Vec2::new(0.0, y));
    out
}

/// The box of environment `name` (ADR-T90-1).
fn environment_box(name: &str, rows: &[Vec<Vec<Node>>], scale: f32, style: Style) -> MathBox {
    let Some(def) = environment(name) else {
        return MathBox::word(&format!("\\begin{{{name}}}"), scale, true);
    };
    let inner = table(shown_rows(rows), def, scale, style);
    match def.brackets {
        Some((open, close)) => fenced(open, inner, close, scale),
        None => inner,
    }
}

/// `rows` without a last row that is blank, as after a trailing `\\`.
fn shown_rows(rows: &[Vec<Vec<Node>>]) -> &[Vec<Vec<Node>>] {
    let blank = rows.last().is_some_and(
        |row| matches!(row.as_slice(), [cell] if cell.iter().all(|n| matches!(n, Node::Space(_)))),
    );
    &rows[..rows.len() - usize::from(blank)]
}

/// The cells of `rows` in columns, centred on the axis (ADR-T90-1), in
/// text style (ADR-T105-1).
fn table(rows: &[Vec<Vec<Node>>], def: &EnvironmentDef, scale: f32, style: Style) -> MathBox {
    let style = style.displayed(false);
    let boxes: Vec<Vec<MathBox>> = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| layout_styled(cell, scale, style))
                .collect()
        })
        .collect();
    let columns = boxes.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0.0_f32; columns];
    for row in &boxes {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = width.max(cell.width);
        }
    }
    let gap = ARRAY_COLUMN_GAP * scale;
    let mut lefts = Vec::with_capacity(columns);
    let mut right = 0.0;
    for width in &widths {
        lefts.push(right);
        right += width + gap;
    }
    let least = (
        ARRAY_ROW_ASCENT * def.stretch * scale,
        ARRAY_ROW_DESCENT * def.stretch * scale,
    );
    let extents: Vec<(f32, f32)> = boxes
        .iter()
        .map(|row| {
            row.iter().fold(least, |(ascent, descent), cell| {
                (ascent.max(cell.ascent), descent.max(cell.descent))
            })
        })
        .collect();
    let half = extents.iter().map(|(a, d)| a + d).sum::<f32>() / 2.0;
    let axis = AXIS * scale;
    let mut out = MathBox {
        width: (right - gap).max(0.0),
        ascent: (half + axis).max(0.0),
        descent: (half - axis).max(0.0),
        italic: 0.0,
        ink: Vec::new(),
    };
    let mut top = -axis - half;
    for (row, (ascent, descent)) in boxes.iter().zip(extents) {
        let baseline = top + ascent;
        for ((cell, left), width) in row.iter().zip(&lefts).zip(&widths) {
            let indent = match def.align {
                Align::Centre => (width - cell.width) / 2.0,
                Align::Left => 0.0,
            };
            out.put(cell, Vec2::new(left + indent, baseline));
        }
        top = baseline + descent;
    }
    out
}

/// The infix command `name` (one of [`INFIX`]) between `left` and
/// `right` (ADR-T71-1).
fn infix(name: &str, left: &[Node], right: &[Node], scale: f32, style: Style) -> MathBox {
    match name {
        "over" => fraction(left, right, scale, true, style),
        "atop" => fraction(left, right, scale, false, style),
        "brack" => delimited(left, right, "[", "]", scale, style),
        "brace" => delimited(left, right, "{", "}", scale, style),
        _ => binomial(left, right, scale, style),
    }
}

/// `\underline`: a rule under `content`, as wide as it, a gap below its
/// descent (the mirror of `\overline`, ADR-T39-1). The box keeps the
/// content's width, ascent and italic correction.
fn underlined(content: &[Node], scale: f32, style: Style) -> MathBox {
    let c = layout_styled(content, scale, style);
    let width = RULE * scale;
    let y = c.descent + ACCENT_GAP * scale + width / 2.0;
    let mut out = MathBox {
        width: c.width,
        ascent: c.ascent,
        descent: y + width / 2.0,
        italic: c.italic,
        ink: c.ink,
    };
    out.ink.push(Ink::Rule {
        a: Vec2::new(0.0, y),
        b: Vec2::new(c.width, y),
        width,
    });
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

/// The mark of accent `def` over `content` (ADR-T39-1): a gap above its
/// ascent, centred on its ink and skewed right over one italic letter.
/// The box keeps the content's width, descent and italic correction.
fn accented(def: &AccentDef, content: &[Node], scale: f32, style: Style) -> MathBox {
    let c = layout_styled(content, scale, style);
    let (lo, hi) = c.ink_span();
    let mut centre = f32::midpoint(lo, hi);
    if slanted(content, style) {
        centre += ACCENT_SKEW * c.ascent;
    }
    let bottom = -(c.ascent + ACCENT_GAP * scale);
    let mut out = MathBox {
        width: c.width,
        ascent: c.ascent,
        descent: c.descent,
        italic: c.italic,
        ink: c.ink,
    };
    let rule = |a: f32, b: f32| {
        let width = RULE * scale;
        let y = bottom - width / 2.0;
        Ink::Rule {
            a: Vec2::new(a, y),
            b: Vec2::new(b, y),
            width,
        }
    };
    match def.mark {
        Mark::Glyph(ch, k) => {
            let mark = MathBox::math(ch, k * scale);
            let (lo, hi) = mark.ink_span();
            let y = bottom - mark.descent;
            out.put(&mark, Vec2::new(centre - f32::midpoint(lo, hi), y));
            out.ascent = out.ascent.max(mark.ascent - y);
        }
        Mark::Bar => {
            let half = ACCENT_BAR * scale / 2.0;
            out.ink.push(rule(centre - half, centre + half));
            out.ascent = out.ascent.max(RULE * scale - bottom);
        }
        Mark::Overline => {
            out.ink.push(rule(0.0, c.width));
            out.ascent = out.ascent.max(RULE * scale - bottom);
        }
    }
    out
}

/// Whether `nodes` are one letter that `style` draws slanted: the
/// letters an accent is skewed over.
fn slanted(nodes: &[Node], style: Style) -> bool {
    let mut atoms = nodes.iter().filter(|n| !matches!(n, Node::Space(_)));
    match (atoms.next(), atoms.next()) {
        (Some(Node::Char(ch)), None) => style.slanted(*ch),
        (Some(Node::Symbol(name)), None) => symbol(name).is_some_and(|def| {
            def.class == Class::Ord && letter(def).is_some_and(|ch| style.slanted(ch))
        }),
        _ => false,
    }
}

/// One line of a block being laid out.
struct Line {
    ink: Vec<Ink>,
    width: f32,
    ascent: f32,
    descent: f32,
    /// For the line of a display formula: the formula's width and the
    /// left and right x of its ink, laid out from x = 0 and centred once
    /// the block's width is known (ADR-T105-1).
    display: Option<(f32, f32, f32)>,
}

impl Line {
    fn new() -> Self {
        Self {
            ink: Vec::new(),
            width: 0.0,
            ascent: BASELINE,
            descent: LINE_HEIGHT - BASELINE,
            display: None,
        }
    }

    /// Whether nothing has been put on the line.
    fn is_empty(&self) -> bool {
        self.ink.is_empty() && self.width <= 0.0 && self.display.is_none()
    }

    /// Formula `b` after what the line holds, its italic overhangs
    /// inside the line (ADR-T33-1).
    fn formula(&mut self, b: &MathBox) {
        let (left, right) = b.ink_span();
        let x = self.width + (-(self.width + left)).max(0.0);
        self.ink
            .extend(b.ink.iter().map(|ink| ink.shifted(Vec2::new(x, 0.0))));
        self.width = (x + b.width).max(x + right);
        self.ascent = self.ascent.max(b.ascent + MATH_PAD);
        self.descent = self.descent.max(b.descent + MATH_PAD);
    }

    /// How far right to move the line's ink in a block `width` wide: a
    /// display formula is centred, its ink kept inside the block.
    fn indent(&self, width: f32) -> f32 {
        match self.display {
            Some((own, left, right)) => ((width - own) / 2.0)
                .max((-left).max(0.0))
                .min(width - right),
            None => 0.0,
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
/// formulas. A display formula takes a line of its own, centred on the
/// block's width (ADR-T105-1).
#[must_use]
pub fn layout(segments: &[Segment]) -> Layout {
    let mut lines = Vec::new();
    let mut line = Line::new();
    // What follows a display formula starts a line of its own.
    let after_display = |lines: &mut Vec<Line>, line: &mut Line| {
        if line.display.is_some() {
            lines.push(std::mem::replace(line, Line::new()));
        }
    };
    for segment in segments {
        match segment {
            Segment::Text(text) => {
                for ch in text.chars() {
                    if ch == '\n' {
                        lines.push(std::mem::replace(&mut line, Line::new()));
                    } else {
                        after_display(&mut lines, &mut line);
                        line.cell(ch);
                    }
                }
            }
            Segment::Dollar => {
                after_display(&mut lines, &mut line);
                line.cell('$');
            }
            Segment::Math(nodes) => {
                after_display(&mut lines, &mut line);
                line.formula(&layout_math(nodes, 1.0));
            }
            Segment::Display(nodes) => {
                if !line.is_empty() {
                    lines.push(std::mem::replace(&mut line, Line::new()));
                }
                // Laid out from x = 0; `Line::indent` places it.
                let b = layout_display(nodes, 1.0);
                let (left, right) = b.ink_span();
                line.width = b.width.max(right) - left.min(0.0);
                line.ascent = line.ascent.max(b.ascent + MATH_PAD);
                line.descent = line.descent.max(b.descent + MATH_PAD);
                line.display = Some((b.width, left, right));
                line.ink = b.ink;
            }
        }
    }
    lines.push(line);
    let width = lines.iter().fold(0.0_f32, |w, line| w.max(line.width));
    let mut out = Layout::default();
    for line in lines {
        let baseline = Vec2::new(line.indent(width), out.height + line.ascent);
        out.ink
            .extend(line.ink.into_iter().map(|ink| ink.shifted(baseline)));
        out.width = out.width.max(line.width);
        out.height += line.ascent + line.descent;
    }
    out
}
