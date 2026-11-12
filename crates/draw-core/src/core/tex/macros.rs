//! Table-driven expansions and user macros.

use super::{
    Arg, DefaultHasher, Hash, HashMap, Hasher, INFIX, Node, Parser, accent, alphabet,
    closing_dollar, closing_double_dollar, environment, kern, symbol, xarrow,
};

/// One shortcut command that draws as its long form (ADR-T69-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpansionDef {
    /// Command name without the backslash.
    pub name: &'static str,
    /// Arguments always taken, of any form (a group, a symbol, one
    /// character).
    pub required: usize,
    /// The long form by the number of optional arguments given: each
    /// optional argument is a `{group}` taken only when one follows.
    /// Subset source with slots `#1` … `#9` for the arguments.
    pub templates: &'static [&'static str],
}

/// Every shortcut command of the subset (ADR-T69-1): the `physics`
/// package's most used commands.
pub const EXPANSIONS: &[ExpansionDef] = &[
    ExpansionDef {
        name: "dd",
        required: 0,
        templates: &["\\,\\mathrm{d}", "\\,\\mathrm{d}#1"],
    },
    ExpansionDef {
        name: "dv",
        required: 1,
        templates: &[
            "\\frac{\\mathrm{d}}{\\mathrm{d}#1}",
            "\\frac{\\mathrm{d}#1}{\\mathrm{d}#2}",
        ],
    },
    ExpansionDef {
        name: "pdv",
        required: 1,
        templates: &[
            "\\frac{\\partial}{\\partial #1}",
            "\\frac{\\partial #1}{\\partial #2}",
        ],
    },
    ExpansionDef {
        name: "abs",
        required: 1,
        templates: &["\\lvert #1\\rvert"],
    },
    ExpansionDef {
        name: "norm",
        required: 1,
        templates: &["\\lVert #1\\rVert"],
    },
    ExpansionDef {
        name: "order",
        required: 1,
        templates: &["\\mathcal{O}(#1)"],
    },
    ExpansionDef {
        name: "vb",
        required: 1,
        templates: &["\\mathbf{#1}"],
    },
    ExpansionDef {
        name: "vu",
        required: 1,
        templates: &["\\hat{\\mathbf{#1}}"],
    },
    ExpansionDef {
        name: "curl",
        required: 0,
        templates: &["\\nabla\\times"],
    },
    // Quantum notation (T70): Dirac brackets with ordinary inner bars,
    // as `physics` draws them with `\middle\vert`, and commutators.
    ExpansionDef {
        name: "bra",
        required: 1,
        templates: &["\\langle #1\\rvert"],
    },
    ExpansionDef {
        name: "ket",
        required: 1,
        templates: &["\\lvert #1\\rangle"],
    },
    ExpansionDef {
        name: "braket",
        required: 1,
        templates: &[
            "\\langle #1\\vert #1\\rangle",
            "\\langle #1\\vert #2\\rangle",
        ],
    },
    ExpansionDef {
        name: "ketbra",
        required: 1,
        templates: &[
            "\\lvert #1\\rangle\\langle #1\\rvert",
            "\\lvert #1\\rangle\\langle #2\\rvert",
        ],
    },
    ExpansionDef {
        name: "expval",
        required: 1,
        templates: &[
            "\\langle #1\\rangle",
            "\\langle #2\\vert #1\\vert #2\\rangle",
        ],
    },
    ExpansionDef {
        name: "mel",
        required: 3,
        templates: &["\\langle #1\\vert #2\\vert #3\\rangle"],
    },
    ExpansionDef {
        name: "comm",
        required: 2,
        templates: &["[#1,#2]"],
    },
    ExpansionDef {
        name: "acomm",
        required: 2,
        templates: &["\\{#1,#2\\}"],
    },
];

/// The shortcut command `name` (without the backslash).
#[must_use]
pub fn expansion(name: &str) -> Option<&'static ExpansionDef> {
    EXPANSIONS.iter().find(|e| e.name == name)
}

/// The long form of `node` when it is a shortcut command of
/// [`EXPANSIONS`] with an argument count it takes: its template parsed,
/// each slot `#n` replaced by the nodes of argument `n` (ADR-T69-1).
/// `None` for any other node.
#[must_use]
pub fn expand(node: &Node) -> Option<Vec<Node>> {
    let Node::Command { name, args } = node else {
        return None;
    };
    let def = expansion(name)?;
    let optional = args.len().checked_sub(def.required)?;
    let template: Vec<char> = def.templates.get(optional)?.chars().collect();
    Some(fill_slots(Parser::new(&template).parse(), args))
}

/// `nodes` of a template with each slot `#n` (in groups and command
/// arguments too) replaced by the nodes of `args[n - 1]`; a slot past
/// the arguments is dropped.
fn fill_slots(nodes: Vec<Node>, args: &[Arg]) -> Vec<Node> {
    let mut out = Vec::with_capacity(nodes.len());
    let mut nodes = nodes.into_iter().peekable();
    while let Some(node) = nodes.next() {
        match node {
            Node::Char('#')
                if let Some(Node::Char(digit)) = nodes.peek()
                    && let Some(n) = digit.to_digit(10) =>
            {
                nodes.next();
                let arg = usize::try_from(n)
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                    .and_then(|i| args.get(i));
                if let Some(arg) = arg {
                    out.extend(arg.nodes.iter().cloned());
                }
            }
            Node::Group(inner) => out.push(Node::Group(fill_slots(inner, args))),
            Node::Command { name, args: inner } => out.push(Node::Command {
                name,
                args: inner
                    .into_iter()
                    .map(|arg| Arg {
                        nodes: fill_slots(arg.nodes, args),
                        ..arg
                    })
                    .collect(),
            }),
            other @ (Node::Space(_)
            | Node::Char(_)
            | Node::Symbol(_)
            | Node::Scripts { .. }
            | Node::Text { .. }
            | Node::OperatorName { .. }
            | Node::Infix { .. }
            | Node::LeftRight { .. }
            | Node::Environment { .. }
            | Node::Kern(_)
            | Node::XArrow { .. }
            | Node::MathStyle(_)
            | Node::Literal(_)) => out.push(other),
        }
    }
    out
}

// ---- user macros (ADR-T112-1) -------------------------------------------------

/// Deepest nesting of macro bodies [`expand_macros`] follows.
pub const MACRO_DEPTH: usize = 32;

/// Most expansion steps, and most characters of output beyond the input,
/// that [`expand_macros`] spends on one formula.
pub const MACRO_SIZE: usize = 10_000;

/// One `\newcommand`: how many arguments it takes and its body, with the
/// slots `#1` … `#9` as typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroDef {
    /// Number of arguments, 0 to 9.
    pub params: usize,
    /// The body between the braces.
    pub body: String,
}

/// The macros defined by the text blocks of a drawing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroTable {
    /// Definitions by name without the backslash.
    defs: HashMap<String, MacroDef>,
    /// Names of definitions skipped because the command is built in.
    ignored: Vec<String>,
}

impl MacroTable {
    /// The table of `texts`, given bottom of the stacking order first: a
    /// later definition of a name replaces an earlier one. Only formulas
    /// define macros.
    #[must_use]
    pub fn from_texts<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut table = Self::default();
        for text in texts {
            if !text.contains("newcommand") {
                continue;
            }
            let chars: Vec<char> = text.chars().collect();
            for (start, end, _) in formula_spans(&chars) {
                table.define_in(&chars[start..end]);
            }
        }
        table
    }

    /// Registers every definition in the formula `chars`.
    fn define_in(&mut self, chars: &[char]) {
        let mut i = 0;
        while i < chars.len() {
            if chars[i] != '\\' {
                i += 1;
            } else if let Some(def) = definition(chars, i) {
                if is_builtin(&def.name) {
                    if !self.ignored.contains(&def.name) {
                        self.ignored.push(def.name);
                    }
                } else {
                    self.defs.insert(
                        def.name,
                        MacroDef {
                            params: def.params,
                            body: def.body,
                        },
                    );
                }
                i = def.end;
            } else {
                i = command_name(chars, i).map_or(i + 2, |(_, end)| end);
            }
        }
    }

    /// The macro `name` (without the backslash).
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&MacroDef> {
        self.defs.get(name)
    }

    /// Whether no macro is defined.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// Names whose definition was skipped because the command is built in,
    /// in the order met.
    #[must_use]
    pub fn ignored(&self) -> &[String] {
        &self.ignored
    }

    /// A number that changes when the definition of `name` changes; 0
    /// when `name` is not defined.
    #[must_use]
    pub fn hash_of(&self, name: &str) -> u64 {
        self.defs.get(name).map_or(0, |def| {
            let mut hasher = DefaultHasher::new();
            def.params.hash(&mut hasher);
            def.body.hash(&mut hasher);
            hasher.finish() | 1
        })
    }
}

/// A block text after macro expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expanded {
    /// The text to parse: definitions removed, macros replaced.
    pub source: String,
    /// The user-defined or still undefined names the formulas mention,
    /// directly or through macro bodies, each once; built-in commands are
    /// left out.
    pub names: Vec<String>,
}

/// Rewrites the formulas of `text` with the macros of `table`: definitions
/// are removed, uses replaced by their bodies. A formula that exceeds
/// [`MACRO_DEPTH`] or [`MACRO_SIZE`] is kept as typed. Text outside
/// formulas is untouched.
#[must_use]
pub fn expand_macros(text: &str, table: &MacroTable) -> Expanded {
    let chars: Vec<char> = text.chars().collect();
    let spans = formula_spans(&chars);
    let mut source = String::new();
    let mut names = Vec::new();
    let mut last = 0;
    for (start, end, delim) in spans {
        source.extend(&chars[last..start - delim]);
        let mut expander = Expander {
            table,
            steps: 0,
            stripped: false,
            names: Vec::new(),
        };
        let mut body = String::new();
        if expander.expand(&chars[start..end], 0, &mut body).is_ok() {
            // A formula that held only definitions draws nothing, not even
            // its dollars (an empty `$$` would read as text).
            if !(expander.stripped && body.trim().is_empty()) {
                source.extend(&chars[start - delim..start]);
                source.push_str(&body);
                source.extend(&chars[end..end + delim]);
            }
        } else {
            source.extend(&chars[start - delim..end + delim]);
        }
        for name in expander.names {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        last = end + delim;
    }
    source.extend(&chars[last..]);
    Expanded { source, names }
}

/// The formulas of a block's characters, as `(content start, content end,
/// delimiter length)`: the splitting [`parse`] does.
fn formula_spans(chars: &[char]) -> Vec<(usize, usize, usize)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' && chars.get(i + 1) == Some(&'$') {
            i += 2;
        } else if let Some(close) = (ch == '$')
            .then(|| closing_double_dollar(chars, i))
            .flatten()
        {
            spans.push((i + 2, close, 2));
            i = close + 2;
        } else if let Some(close) = (ch == '$').then(|| closing_dollar(chars, i)).flatten() {
            spans.push((i + 1, close, 1));
            i = close + 1;
        } else {
            i += 1;
        }
    }
    spans
}

/// Commands that are neither symbols, accents, shortcuts, environments,
/// arrows, alphabets nor spacing but still have a meaning of their own.
const STRUCTURAL: &[&str] = &[
    "frac",
    "dfrac",
    "tfrac",
    "binom",
    "dbinom",
    "tbinom",
    "multiset",
    "sqrt",
    "pmod",
    "underline",
    "text",
    "operatorname",
    "left",
    "right",
    "begin",
    "end",
    "displaystyle",
    "textstyle",
    "newcommand",
    "renewcommand",
];

/// Whether `\name` means something already, so a macro cannot take it.
fn is_builtin(name: &str) -> bool {
    symbol(name).is_some()
        || accent(name).is_some()
        || expansion(name).is_some()
        || environment(name).is_some()
        || xarrow(name).is_some()
        || alphabet(name).is_some()
        || kern(&format!("\\{name}")).is_some()
        || INFIX.contains(&name)
        || STRUCTURAL.contains(&name)
}

/// The letters-only command name after the `\` at `at`, and the index
/// after it.
fn command_name(chars: &[char], at: usize) -> Option<(String, usize)> {
    if chars.get(at) != Some(&'\\') {
        return None;
    }
    let mut end = at + 1;
    while chars.get(end).is_some_and(char::is_ascii_alphabetic) {
        end += 1;
    }
    (end > at + 1).then(|| (chars[at + 1..end].iter().collect(), end))
}

/// The index of the `}` closing the `{` at `open`, skipping escaped
/// characters.
fn group_end(chars: &[char], open: usize) -> Option<usize> {
    let mut depth = 0_usize;
    let mut i = open;
    while let Some(&ch) = chars.get(i) {
        match ch {
            '\\' => i += 1,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// The index of the first non-space character at or after `i`.
fn skip_spaces(chars: &[char], mut i: usize) -> usize {
    while chars.get(i) == Some(&' ') {
        i += 1;
    }
    i
}

/// A `\newcommand` or `\renewcommand` read from source.
struct Definition {
    /// Name without the backslash.
    name: String,
    /// Declared argument count.
    params: usize,
    /// Body between the braces.
    body: String,
    /// Index after the closing brace of the body.
    end: usize,
}

/// The definition starting at the `\` at `at`: `\newcommand{\name}[n]{body}`
/// or `\renewcommand`, with `{\name}` or `\name` and `[n]` optional.
fn definition(chars: &[char], at: usize) -> Option<Definition> {
    let (command, end) = command_name(chars, at)?;
    if command != "newcommand" && command != "renewcommand" {
        return None;
    }
    let mut i = skip_spaces(chars, end);
    let braced = chars.get(i) == Some(&'{');
    if braced {
        i = skip_spaces(chars, i + 1);
    }
    let (name, after) = command_name(chars, i)?;
    i = after;
    if braced {
        i = skip_spaces(chars, i);
        if chars.get(i) != Some(&'}') {
            return None;
        }
        i += 1;
    }
    i = skip_spaces(chars, i);
    let mut params = 0;
    if chars.get(i) == Some(&'[') {
        let digit = chars.get(i + 1)?.to_digit(10)?;
        if chars.get(i + 2) != Some(&']') {
            return None;
        }
        params = usize::try_from(digit).ok()?;
        i = skip_spaces(chars, i + 3);
    }
    if chars.get(i) != Some(&'{') {
        return None;
    }
    let close = group_end(chars, i)?;
    Some(Definition {
        name,
        params,
        body: chars[i + 1..close].iter().collect(),
        end: close + 1,
    })
}

/// The arguments of a macro of `count` arguments starting at `from`: each
/// a `{group}` (braces dropped), a command or one character. `None` when
/// the text runs out first.
fn macro_args(chars: &[char], from: usize, count: usize) -> Option<(Vec<String>, usize)> {
    let mut args = Vec::with_capacity(count);
    let mut i = from;
    for _ in 0..count {
        i = skip_spaces(chars, i);
        let first = *chars.get(i)?;
        let (arg, next) = if first == '{' {
            let close = group_end(chars, i)?;
            (chars[i + 1..close].iter().collect(), close + 1)
        } else if let Some((_, end)) = command_name(chars, i) {
            (chars[i..end].iter().collect(), end)
        } else if first == '\\' {
            let end = (i + 2).min(chars.len());
            (chars[i..end].iter().collect(), end)
        } else {
            (first.to_string(), i + 1)
        };
        args.push(arg);
        i = next;
    }
    Some((args, i))
}

/// `body` with each slot `#n` replaced by `args[n - 1]`; slots past the
/// arguments stay as typed.
fn fill_macro_slots(body: &str, args: &[String]) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        let slot = (ch == '#')
            .then(|| chars.peek().and_then(|d| d.to_digit(10)))
            .flatten()
            .and_then(|n| usize::try_from(n).ok())
            .and_then(|n| n.checked_sub(1))
            .and_then(|i| args.get(i));
        if let Some(arg) = slot {
            chars.next();
            out.push_str(arg);
        } else {
            out.push(ch);
        }
    }
    out
}

/// Expansion of one formula's source.
struct Expander<'a> {
    /// The macros to apply.
    table: &'a MacroTable,
    /// Macro uses expanded so far.
    steps: usize,
    /// Whether a definition was removed.
    stripped: bool,
    /// User-defined or undefined names met, once each.
    names: Vec<String>,
}

impl Expander<'_> {
    /// Appends `chars` to `out` with definitions removed and macros
    /// expanded; `Err` when a limit is passed.
    fn expand(&mut self, chars: &[char], depth: usize, out: &mut String) -> Result<(), ()> {
        if depth > MACRO_DEPTH {
            return Err(());
        }
        let mut glued = false;
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];
            if ch != '\\' {
                // A letter right after an expanded command word would
                // extend that word.
                if glued && ch.is_ascii_alphabetic() && ends_with_command_word(out) {
                    out.push(' ');
                }
                glued = false;
                out.push(ch);
                i += 1;
                continue;
            }
            glued = false;
            let Some((name, end)) = command_name(chars, i) else {
                out.push('\\');
                if let Some(&next) = chars.get(i + 1) {
                    out.push(next);
                }
                i += 2;
                continue;
            };
            if let Some(def) = definition(chars, i) {
                self.stripped = true;
                i = def.end;
                continue;
            }
            if !is_builtin(&name) && !self.names.contains(&name) {
                self.names.push(name.clone());
            }
            let applied = self
                .table
                .get(&name)
                .and_then(|def| macro_args(chars, end, def.params).map(|args| (def, args)));
            let Some((def, (args, next))) = applied else {
                out.extend(&chars[i..end]);
                i = end;
                continue;
            };
            self.steps += 1;
            if self.steps > MACRO_SIZE {
                return Err(());
            }
            let body: Vec<char> = fill_macro_slots(&def.body, &args).chars().collect();
            if body.first().is_some_and(char::is_ascii_alphabetic) && ends_with_command_word(out) {
                out.push(' ');
            }
            self.expand(&body, depth + 1, out)?;
            if out.len() > MACRO_SIZE {
                return Err(());
            }
            glued = true;
            i = next;
        }
        Ok(())
    }
}

/// Whether `out` ends in a command name (`\alpha`), which a following
/// letter would lengthen.
fn ends_with_command_word(out: &str) -> bool {
    let word = out.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    word.len() < out.len() && word.ends_with('\\') && !word.ends_with("\\\\")
}
