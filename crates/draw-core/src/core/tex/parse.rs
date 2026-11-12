//! Parser, printer and the node tree.

use super::{
    DELIMITERS, ENVIRONMENT_MAX_CELLS, MAX_DEPTH, accent, alphabet, environment, expansion, kern,
    symbol, xarrow,
};

/// A run of a block's text.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    /// Plain text, drawn as typed (may contain line breaks).
    Text(String),
    /// `\$` outside a formula: a literal dollar sign.
    Dollar,
    /// A `$…$` formula, without its dollars.
    Math(Vec<Node>),
    /// A `$$…$$` formula, without its dollars: laid out in display style
    /// on a line of its own (ADR-T105-1).
    Display(Vec<Node>),
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
    /// `\frac`, `\binom`, `\sqrt`, `\pmod`, a math alphabet
    /// ([`ALPHABETS`](super::ALPHABETS)) or an accent ([`ACCENTS`](super::ACCENTS)) with all its arguments.
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
    /// `\operatorname{…}` or `\operatorname*{…}`: a raw body drawn as an
    /// upright operator word (ADR-T64-1).
    OperatorName {
        /// Whether a `*` follows the name directly: the word takes limits.
        star: bool,
        /// Whitespace between the name (or star) and `{`.
        lead: String,
        /// The body between the braces (balanced braces allowed).
        body: String,
    },
    /// An infix command of [`INFIX`] that split its group (or the
    /// formula) in two, as TeX's `\over` does (ADR-T71-1).
    Infix {
        /// Name without the backslash.
        name: String,
        /// The nodes before the command.
        left: Vec<Node>,
        /// The nodes after the command.
        right: Vec<Node>,
    },
    /// A `\left … \right` pair with brackets grown to its body
    /// (ADR-T89-1).
    LeftRight {
        /// What follows `\left`.
        open: Delimiter,
        /// The nodes between the two commands.
        body: Vec<Node>,
        /// What follows `\right`.
        close: Delimiter,
    },
    /// A `\begin{…} … \end{…}` environment of [`ENVIRONMENTS`](super::ENVIRONMENTS)
    /// (ADR-T90-1).
    Environment {
        /// The name between the braces.
        name: String,
        /// Whitespace between `\begin` and `{`.
        lead: String,
        /// Whitespace between `\end` and `{`.
        end_lead: String,
        /// The rows, split on `\\`, of cells, split on `&`. There is at
        /// least one row of at least one cell.
        rows: Vec<Vec<Vec<Node>>>,
    },
    /// A spacing command of [`KERNS`](super::KERNS), as typed: space that is not an
    /// atom, so it never changes its neighbours' classes (ADR-T99-1).
    Kern(String),
    /// An extensible arrow of [`XARROWS`](super::XARROWS) with its labels (ADR-T103-1).
    XArrow {
        /// Name without the backslash.
        name: String,
        /// The optional `[…]` label set under the arrow.
        below: Option<Bracket>,
        /// The required label set above the arrow.
        above: Arg,
    },
    /// `\displaystyle` (`true`) or `\textstyle` (`false`): switches the
    /// style of the rest of its list; not an atom (ADR-T105-1).
    MathStyle(bool),
    /// Source outside the subset, drawn as typed in the warning colour.
    Literal(String),
}

/// The delimiter after `\left` or `\right`, as typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delimiter {
    /// Whitespace between the command and the delimiter.
    pub lead: String,
    /// Its source: a row of [`DELIMITERS`].
    pub source: String,
}

/// The infix commands that split their group (ADR-T71-1): `\over`,
/// `\atop`, `\choose`, `\brack`, `\brace`.
pub const INFIX: &[&str] = &["over", "atop", "choose", "brack", "brace"];

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

/// An optional `[…]` argument (ADR-T103-1).
#[derive(Debug, Clone, PartialEq)]
pub struct Bracket {
    /// Whitespace skipped before the `[`.
    pub lead: String,
    /// The nodes between the brackets.
    pub nodes: Vec<Node>,
}

/// What a stretched extensible arrow draws at the end its head does not
/// point to (ADR-T103-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tail {
    /// Nothing: the shaft just ends.
    Plain,
    /// A vertical bar, as `↦`'s.
    Bar,
    /// A hook above the shaft, as `↪`'s.
    Hook,
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
        } else if let Some(close) = (ch == '$')
            .then(|| closing_double_dollar(&chars, i))
            .flatten()
        {
            flush(&mut plain, &mut segments);
            segments.push(Segment::Display(Parser::new(&chars[i + 2..close]).parse()));
            i = close + 2;
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
pub(super) fn closing_dollar(chars: &[char], open: usize) -> Option<usize> {
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

/// The index of the first `$` of the `$$` closing the display formula
/// opened by a `$$` at `open`: the next unescaped `$` on the same line,
/// if another `$` follows it and something lies between. `None` when
/// `open` starts no `$$` or nothing closes it (ADR-T105-1).
pub(super) fn closing_double_dollar(chars: &[char], open: usize) -> Option<usize> {
    if chars.get(open + 1) != Some(&'$') {
        return None;
    }
    let mut j = open + 2;
    while let Some(&ch) = chars.get(j) {
        match ch {
            '\n' => return None,
            '$' => return (j > open + 2 && chars.get(j + 1) == Some(&'$')).then_some(j),
            '\\' if chars.get(j + 1).is_some_and(|&next| next != '\n') => j += 2,
            _ => j += 1,
        }
    }
    None
}

/// Recursive-descent parser of one formula.
pub(super) struct Parser<'a> {
    /// The formula, without its dollars.
    chars: &'a [char],
    /// Next character to read.
    pos: usize,
    /// For each `{`, the index of its matching `}`, if it has one within
    /// [`MAX_DEPTH`].
    closes: Vec<Option<usize>>,
    /// For each `\left`, the index of the `\` of its `\right`, and for
    /// each `\begin`, the index of the `\` of its `\end`, if it has one
    /// within [`MAX_DEPTH`] (ADR-T89-1, ADR-T90-1).
    rights: Vec<Option<usize>>,
}

impl<'a> Parser<'a> {
    pub(super) fn new(chars: &'a [char]) -> Self {
        let (closes, rights) = match_pairs(chars);
        Self {
            chars,
            pos: 0,
            closes,
            rights,
        }
    }

    /// The whole formula.
    pub(super) fn parse(mut self) -> Vec<Node> {
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
    /// The first infix command of the list splits it in two; later ones
    /// are literals (ADR-T71-1).
    fn list(&mut self, end: usize) -> Vec<Node> {
        let mut out: Vec<Node> = Vec::new();
        let mut split: Option<(String, Vec<Node>)> = None;
        while self.pos < end {
            let ch = self.chars[self.pos];
            match ch {
                '^' | '_' => self.script(ch, end, &mut out),
                '\\' if split.is_none()
                    && let Some(name) = self.infix(end) =>
                {
                    split = Some((name, std::mem::take(&mut out)));
                }
                _ => {
                    let node = self.atom(end);
                    out.push(node);
                }
            }
        }
        match split {
            Some((name, left)) => vec![Node::Infix {
                name,
                left,
                right: out,
            }],
            None => out,
        }
    }

    /// The name of an infix command at the cursor (a `\`), read past;
    /// `None` (cursor unchanged) for any other command.
    fn infix(&mut self, end: usize) -> Option<String> {
        let start = self.pos;
        let name = self.name(end);
        if INFIX.contains(&name.as_str()) {
            Some(name)
        } else {
            self.pos = start;
            None
        }
    }

    /// One node that is not a script, at the cursor (which is before
    /// `end`).
    fn atom(&mut self, end: usize) -> Node {
        let ch = self.chars[self.pos];
        if ch.is_whitespace() {
            return Node::Space(self.whitespace(end));
        }
        if ch == '{'
            && let Some(close) = self.group_close(end)
        {
            self.pos += 1;
            let nodes = self.list(close);
            self.pos = close + 1;
            return Node::Group(nodes);
        }
        if ch == '\\' {
            return self.command(end);
        }
        self.pos += 1;
        match ch {
            '{' | '}' => Node::Literal(ch.to_string()),
            '~' => Node::Kern(ch.to_string()),
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
                let source = format!("\\{name}");
                if symbol(&name).is_some() {
                    Some((false, vec![Node::Symbol(name)]))
                } else {
                    kern(&source).map(|_| (false, vec![Node::Kern(source)]))
                }
            }
            Some('~') => {
                self.pos += 1;
                Some((false, vec![Node::Kern("~".to_owned())]))
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
        let start = self.pos + 1;
        self.pos = name_end(self.chars, start, end);
        self.chars[start..self.pos].iter().collect()
    }

    /// A command at the cursor (a `\`), with its arguments, or literal.
    fn command(&mut self, end: usize) -> Node {
        let start = self.pos;
        let name = self.name(end);
        if symbol(&name).is_some() {
            return Node::Symbol(name);
        }
        let source = format!("\\{name}");
        if kern(&source).is_some() {
            return Node::Kern(source);
        }
        let after = self.pos;
        let node = match name.as_str() {
            "left" => self.left_right(start, end),
            "begin" => return self.environment(start, end),
            "end" => return self.stray_end(start, end),
            "displaystyle" => return Node::MathStyle(true),
            "textstyle" => return Node::MathStyle(false),
            "frac" | "dfrac" | "tfrac" | "binom" | "tbinom" | "dbinom" | "multiset" => {
                self.args(2, end).map(|args| Node::Command {
                    name: name.clone(),
                    args,
                })
            }
            n if matches!(n, "sqrt" | "pmod" | "underline")
                || alphabet(n).is_some()
                || accent(n).is_some() =>
            {
                self.args(1, end).map(|args| Node::Command {
                    name: name.clone(),
                    args,
                })
            }
            "text" => self.text_body(end),
            "operatorname" => return self.operator_name(end),
            n if xarrow(n).is_some() => self.xarrow(&name, end),
            n if expansion(n).is_some() => self.shortcut(&name, end),
            _ => None,
        };
        node.unwrap_or_else(|| {
            self.pos = after;
            Node::Literal(format!("\\{name}"))
        })
    }

    /// The pair of the `\left` at `start`, its name read; `None` when it
    /// has no matching `\right` before `end` (ADR-T89-1).
    fn left_right(&mut self, start: usize, end: usize) -> Option<Node> {
        let right = self.rights[start].filter(|&right| right < end)?;
        let open = self.delimiter(end)?;
        let body = self.list(right);
        self.pos = right;
        self.name(end);
        let close = self.delimiter(end)?;
        Some(Node::LeftRight { open, body, close })
    }

    /// The environment of the `\begin` at `start`, its name read; a literal
    /// when it is invalid (ADR-T90-1).
    fn environment(&mut self, start: usize, end: usize) -> Node {
        let Some((lead, name, past)) = read_environment_name(self.chars, self.pos, end) else {
            return Node::Literal("\\begin".to_owned());
        };
        let table = self.rights[start]
            .filter(|&close| close < end)
            .and_then(|close| Some((close, self.cell_spans(past, close)?)));
        let closing = table.as_ref().and_then(|(close, _)| {
            let after = name_end(self.chars, close + 1, end);
            read_environment_name(self.chars, after, end)
        });
        let (Some((_, spans)), Some((end_lead, _, after))) = (table, closing) else {
            self.pos = past;
            return Node::Literal(self.chars[start..past].iter().collect());
        };
        let mut rows = Vec::with_capacity(spans.len());
        for row in spans {
            let mut cells = Vec::with_capacity(row.len());
            for (from, to) in row {
                self.pos = from;
                cells.push(self.list(to));
            }
            rows.push(cells);
        }
        self.pos = after;
        Node::Environment {
            name,
            lead,
            end_lead,
            rows,
        }
    }

    /// An `\end` at `start` that closes no environment, its name read: a
    /// literal, with its `{name}` if it has one.
    fn stray_end(&mut self, start: usize, end: usize) -> Node {
        let Some((_, _, past)) = read_environment_name(self.chars, self.pos, end) else {
            return Node::Literal("\\end".to_owned());
        };
        self.pos = past;
        Node::Literal(self.chars[start..past].iter().collect())
    }

    /// The `(from, to)` spans of the cells of the environment body
    /// `start..end`, by row, without parsing them. A separator inside a
    /// group, a `\left` pair or a nested environment is not the body's.
    /// `None` past [`ENVIRONMENT_MAX_CELLS`] rows or columns (ADR-T90-1).
    fn cell_spans(&self, start: usize, end: usize) -> Option<Vec<Vec<(usize, usize)>>> {
        let limit = ENVIRONMENT_MAX_CELLS;
        let mut rows = vec![Vec::new()];
        let (mut cell, mut i) = (start, start);
        while i < end {
            match self.chars[i] {
                '\\' => {
                    if let Some(partner) = self.rights[i].filter(|&partner| partner < end) {
                        i = partner;
                    } else if i + 1 < end && self.chars[i + 1] == '\\' {
                        let row = rows.last_mut()?;
                        row.push((cell, i));
                        if row.len() > limit || rows.len() > limit {
                            return None;
                        }
                        rows.push(Vec::new());
                        i += 2;
                        cell = i;
                    } else {
                        i = name_end(self.chars, i + 1, end);
                    }
                }
                '&' => {
                    let row = rows.last_mut()?;
                    row.push((cell, i));
                    if row.len() >= limit {
                        return None;
                    }
                    i += 1;
                    cell = i;
                }
                '{' => i = self.closes[i].filter(|&close| close < end).unwrap_or(i) + 1,
                _ => i += 1,
            }
        }
        let row = rows.last_mut()?;
        row.push((cell, end));
        if row.len() > limit {
            return None;
        }
        // A trailing `\\` adds no row, so it does not count either.
        let blank =
            self.chars[cell..end].iter().all(|ch| ch.is_whitespace()) && rows.last()?.len() == 1;
        (rows.len() - usize::from(blank) <= limit).then_some(rows)
    }

    /// The delimiter at the cursor, read past.
    fn delimiter(&mut self, end: usize) -> Option<Delimiter> {
        let (delimiter, past) = read_delimiter(self.chars, self.pos, end)?;
        self.pos = past;
        Some(delimiter)
    }

    /// `\operatorname`, its name read: a `*` directly after it, then a
    /// raw `{…}` body; literal (with its star) without a body
    /// (ADR-T64-1).
    fn operator_name(&mut self, end: usize) -> Node {
        let star = self.pos < end && self.peek() == Some('*');
        self.pos += usize::from(star);
        let after = self.pos;
        if let Some((lead, body)) = self.raw_body(end) {
            return Node::OperatorName { star, lead, body };
        }
        self.pos = after;
        let star = if star { "*" } else { "" };
        Node::Literal(format!("\\operatorname{star}"))
    }

    /// The arguments of shortcut `name` of [`EXPANSIONS`], its name read:
    /// the required ones, then each optional one that is a `{group}`;
    /// `None` when a required one is missing (ADR-T69-1).
    fn shortcut(&mut self, name: &str, end: usize) -> Option<Node> {
        let def = expansion(name)?;
        let mut args = self.args(def.required, end)?;
        for _ in 1..def.templates.len() {
            let start = self.pos;
            self.whitespace(end);
            let group =
                self.pos < end && self.peek() == Some('{') && self.group_close(end).is_some();
            self.pos = start;
            if !group {
                break;
            }
            args.extend(self.arg(end));
        }
        Some(Node::Command {
            name: name.to_owned(),
            args,
        })
    }

    /// The labels of extensible arrow `name`, its name read: an optional
    /// `[…]`, then a required argument; `None` when either is broken
    /// (ADR-T103-1).
    fn xarrow(&mut self, name: &str, end: usize) -> Option<Node> {
        let below = if self.bracket_follows(end) {
            Some(self.bracket(end)?)
        } else {
            None
        };
        let above = self.arg(end)?;
        Some(Node::XArrow {
            name: name.to_owned(),
            below,
            above,
        })
    }

    /// Whether a `[` follows the cursor after whitespace before `end`;
    /// the cursor does not move.
    fn bracket_follows(&self, end: usize) -> bool {
        self.chars[self.pos.min(end)..end]
            .iter()
            .find(|ch| !ch.is_whitespace())
            == Some(&'[')
    }

    /// An optional `[…]` argument at the cursor: whitespace, `[`, nodes
    /// up to the first `]` outside braces before `end`; `None` (the
    /// cursor may then have moved) when there is no `[` or it has no `]`
    /// (ADR-T103-1).
    fn bracket(&mut self, end: usize) -> Option<Bracket> {
        let lead = self.whitespace(end);
        if self.pos >= end || self.peek() != Some('[') {
            return None;
        }
        let open = self.pos;
        let mut i = open + 1;
        let close = loop {
            match self.chars.get(i).filter(|_| i < end)? {
                ']' => break i,
                '{' => i = self.closes[i].filter(|&close| close < end).unwrap_or(i) + 1,
                '\\' => i = name_end(self.chars, i + 1, end),
                _ => i += 1,
            }
        };
        self.pos = open + 1;
        let nodes = self.list(close);
        self.pos = close + 1;
        Some(Bracket { lead, nodes })
    }

    /// `n` arguments, or `None` if one is missing.
    fn args(&mut self, n: usize, end: usize) -> Option<Vec<Arg>> {
        (0..n).map(|_| self.arg(end)).collect()
    }

    /// The `{…}` body of `\text`, raw.
    fn text_body(&mut self, end: usize) -> Option<Node> {
        self.raw_body(end)
            .map(|(lead, body)| Node::Text { lead, body })
    }

    /// Whitespace and a raw `{…}` body at the cursor; `None` when no
    /// closed group follows (the cursor may then have moved).
    fn raw_body(&mut self, end: usize) -> Option<(String, String)> {
        let lead = self.whitespace(end);
        let close = self.group_close(end)?;
        let body = self.chars[self.pos + 1..close].iter().collect();
        self.pos = close + 1;
        Some((lead, body))
    }
}

/// The end of the command name starting at `start`: past a run of ASCII
/// letters or one other character; `start` when `end` is reached.
fn name_end(chars: &[char], start: usize, end: usize) -> usize {
    match chars.get(start).filter(|_| start < end) {
        Some(ch) if ch.is_ascii_alphabetic() => {
            let mut i = start;
            while i < end && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            i
        }
        Some(_) => start + 1,
        None => start,
    }
}

/// The delimiter of [`DELIMITERS`] at `pos`, after optional whitespace, and
/// the index past it; `None` when there is no valid one before `end`.
fn read_delimiter(chars: &[char], pos: usize, end: usize) -> Option<(Delimiter, usize)> {
    let mut at = pos;
    while at < end && chars[at].is_whitespace() {
        at += 1;
    }
    let past = if at < end && chars[at] == '\\' {
        name_end(chars, at + 1, end)
    } else {
        (at + 1).min(end)
    };
    let source: String = chars[at..past].iter().collect();
    DELIMITERS.iter().any(|(s, _)| *s == source).then(|| {
        let lead = chars[pos..at].iter().collect();
        (Delimiter { lead, source }, past)
    })
}

/// The whitespace, the name and the index past the `}` of a `{name}` of
/// ASCII letters at `pos`, after optional whitespace; `None` when there is
/// none before `end`.
fn read_environment_name(
    chars: &[char],
    pos: usize,
    end: usize,
) -> Option<(String, String, usize)> {
    let mut brace = pos;
    while brace < end && chars[brace].is_whitespace() {
        brace += 1;
    }
    if brace >= end || chars[brace] != '{' {
        return None;
    }
    let mut close = brace + 1;
    while close < end && chars[close].is_ascii_alphabetic() {
        close += 1;
    }
    (close < end && chars[close] == '}').then(|| {
        (
            chars[pos..brace].iter().collect(),
            chars[brace + 1..close].iter().collect(),
            close + 1,
        )
    })
}

/// What an [`Opening`] opened.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Opener {
    /// A `{`.
    Brace,
    /// A `\left`.
    Left,
    /// A `\begin` of the environment with this name.
    Begin(&'static str),
}

/// A `{`, `\left` or `\begin` still waiting for its partner in
/// [`match_pairs`].
struct Opening {
    /// Its index.
    at: usize,
    /// Whether it is within [`MAX_DEPTH`].
    within: bool,
    /// What it is.
    kind: Opener,
}

/// For each `{` of `chars`, the index of its matching `}`, and for each
/// `\left` or `\begin`, the index of the `\` of its `\right` or `\end`.
/// Escaped braces (`\{`) are skipped; what nests deeper than [`MAX_DEPTH`]
/// matches nothing. A pair never crosses a group: a `}` drops the `\left`s
/// and `\begin`s still open inside it, and a `\right` or `\end` only closes
/// its own opener on top of the stack (ADR-T89-1, ADR-T90-1).
fn match_pairs(chars: &[char]) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
    let mut closes = vec![None; chars.len()];
    let mut rights = vec![None; chars.len()];
    let mut open: Vec<Opening> = Vec::new();
    let (mut depth, mut braces) = (0, 0);
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                let after = name_end(chars, i + 1, chars.len());
                let name = &chars[i + 1..after];
                let is = |word: &str| name.iter().copied().eq(word.chars());
                let known = (is("begin") || is("end"))
                    .then(|| read_environment_name(chars, after, chars.len()))
                    .flatten()
                    .and_then(|(_, name, past)| Some((environment(&name)?.name, past)));
                if let Some((name, past)) = known {
                    if is("begin") {
                        let within = depth < MAX_DEPTH;
                        depth += usize::from(within);
                        open.push(Opening {
                            at: i,
                            within,
                            kind: Opener::Begin(name),
                        });
                    } else if let Some(begin) = open.pop_if(|top| top.kind == Opener::Begin(name))
                        && begin.within
                    {
                        rights[begin.at] = Some(i);
                        depth -= 1;
                    }
                    i = past;
                    continue;
                }
                let past = (is("left") || is("right"))
                    .then(|| read_delimiter(chars, after, chars.len()))
                    .flatten()
                    .map(|(_, past)| past);
                if past.is_some() {
                    if is("left") {
                        let within = depth < MAX_DEPTH;
                        depth += usize::from(within);
                        open.push(Opening {
                            at: i,
                            within,
                            kind: Opener::Left,
                        });
                    } else if let Some(left) = open.pop_if(|top| top.kind == Opener::Left)
                        && left.within
                    {
                        rights[left.at] = Some(i);
                        depth -= 1;
                    }
                }
                i = past.unwrap_or(after);
                continue;
            }
            '{' => {
                let within = depth < MAX_DEPTH;
                depth += usize::from(within);
                braces += 1;
                open.push(Opening {
                    at: i,
                    within,
                    kind: Opener::Brace,
                });
            }
            '}' if braces > 0 => {
                braces -= 1;
                while let Some(top) = open.pop() {
                    depth -= usize::from(top.within);
                    if top.kind == Opener::Brace {
                        if top.within {
                            closes[top.at] = Some(i);
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    (closes, rights)
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
            Segment::Display(nodes) => {
                out.push_str("$$");
                print_nodes(nodes, &mut out);
                out.push_str("$$");
            }
        }
    }
    out
}

/// The source of a [`Node::MathStyle`]: `\displaystyle` or `\textstyle`.
const fn style_command(display: bool) -> &'static str {
    if display {
        "\\displaystyle"
    } else {
        "\\textstyle"
    }
}

fn print_nodes(nodes: &[Node], out: &mut String) {
    for node in nodes {
        print_node(node, out);
    }
}

fn print_node(node: &Node, out: &mut String) {
    match node {
        Node::Space(text) | Node::Kern(text) | Node::Literal(text) => out.push_str(text),
        Node::MathStyle(display) => out.push_str(style_command(*display)),
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
        Node::OperatorName { star, lead, body } => {
            out.push_str("\\operatorname");
            if *star {
                out.push('*');
            }
            out.push_str(lead);
            out.push('{');
            out.push_str(body);
            out.push('}');
        }
        Node::Infix { name, left, right } => {
            print_nodes(left, out);
            out.push('\\');
            out.push_str(name);
            print_nodes(right, out);
        }
        Node::LeftRight { open, body, close } => {
            out.push_str("\\left");
            out.push_str(&open.lead);
            out.push_str(&open.source);
            print_nodes(body, out);
            out.push_str("\\right");
            out.push_str(&close.lead);
            out.push_str(&close.source);
        }
        Node::Environment {
            name,
            lead,
            end_lead,
            rows,
        } => print_environment(name, lead, end_lead, rows, out),
        Node::XArrow { name, below, above } => {
            out.push('\\');
            out.push_str(name);
            if let Some(below) = below {
                out.push_str(&below.lead);
                out.push('[');
                print_nodes(&below.nodes, out);
                out.push(']');
            }
            print_arg(above, out);
        }
    }
}

/// The source of a [`Node::Environment`].
fn print_environment(
    name: &str,
    lead: &str,
    end_lead: &str,
    rows: &[Vec<Vec<Node>>],
    out: &mut String,
) {
    let command = |word: &str, lead: &str, out: &mut String| {
        out.push_str(word);
        out.push_str(lead);
        out.push('{');
        out.push_str(name);
        out.push('}');
    };
    command("\\begin", lead, out);
    for (r, row) in rows.iter().enumerate() {
        if r > 0 {
            out.push_str("\\\\");
        }
        for (c, cell) in row.iter().enumerate() {
            if c > 0 {
                out.push('&');
            }
            print_nodes(cell, out);
        }
    }
    command("\\end", end_lead, out);
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
