//! Editor snapshots (ADR-T74-1): the whole headless [`Editor`] state as a
//! small, canonical, line-based text, and back.
//!
//! ```text
//! draw-snapshot 3
//! [editor]
//! tool pen
//! …
//! [document]
//! next-id 3
//! shape 1 line 0 3.0 10.0 10.0 50.0 40.0
//! …
//! locked 1
//! …
//! ```
//!
//! Version 2 added the optional `locked` record and the `lock` and
//! `unlock` edits (ADR-T107-1). Version 3 added the final `[pages]`
//! section and a `[page]` section per stored page (ADR-T116-1); version 2
//! still loads, as one page.
//!
//! [`Editor::snapshot`] captures a state, [`Snapshot::to_text`] writes it,
//! [`Snapshot::parse`] reads and validates text, and
//! [`Editor::from_snapshot`] turns a snapshot back into an editor. The
//! text is canonical: one state has exactly one text. Private tool states
//! write and read their own records through [`Line`] and [`Tokens`].

use std::fmt::{self, Write as _};
use std::str::FromStr;

use crate::core::command::Tool;
use crate::core::document::{Document, Edit, IdSet, ShapeId, Transaction};
use crate::core::editor::Editor;
use crate::core::geom::{Aabb, Vec2};
use crate::core::input::{Modifiers, PointerButton};
use crate::core::palette::ColorId;
use crate::core::shape::{CellFill, CellText, GRID_MAX_CELLS, Shape, ShapeText, Style, TextSlot};
use crate::core::smoothing::SmoothingLevel;

/// First token of a snapshot's first line.
pub const HEADER: &str = "draw-snapshot";

/// Version of the format this build writes and reads. Bumped, with an ADR,
/// whenever the text changes (ADR-T74-1).
pub const VERSION: u32 = 3;

/// Oldest version this build still reads: version 2 has no `[pages]`
/// section, so it loads as a single page (ADR-T116-1).
pub const MIN_VERSION: u32 = 2;

/// Largest on-screen text size the editor sets, in pixels: the thickest
/// width's rung, with 1 % for rounding.
pub const MAX_TEXT_SIZE_PX: f32 = (crate::core::tools::text::TEXT_BASE_PX
    + crate::core::tools::text::TEXT_PX_PER_WIDTH * crate::core::editor::MAX_WIDTH_PX)
    * 1.01;

/// Largest text size in world units the editor can make: the largest
/// on-screen size at the smallest zoom. Larger loaded sizes could grow a
/// block's bounds past `f32` while typing.
pub const MAX_TEXT_SIZE: f32 = MAX_TEXT_SIZE_PX / crate::core::camera::ZOOM_MIN;

/// Whether `size` is a text size the editor can make.
#[must_use]
pub fn text_size_ok(size: f32) -> bool {
    size > 0.0 && size <= MAX_TEXT_SIZE
}

/// The sections, in the order they appear.
pub const SECTIONS: [&str; 7] = [
    "editor",
    "document",
    "history",
    "clipboard",
    "tools",
    "pages",
    "page",
];

/// Why a text is not a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotError {
    /// 1-based line of the problem.
    pub line: usize,
    /// What is wrong, for people.
    pub message: String,
}

impl SnapshotError {
    /// An error at `line` (1-based).
    #[must_use]
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for SnapshotError {}

/// A validated editor state: what [`Editor::snapshot`] captures and
/// [`Snapshot::parse`] accepts. Loading it cannot fail.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot(pub(crate) Editor);

impl Snapshot {
    /// The canonical text of this state.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut w = Writer::default();
        w.out.push_str(HEADER);
        let _ = writeln!(w.out, " {VERSION}");
        self.0.write_snapshot(&mut w);
        w.out
    }

    /// Reads a snapshot from `text`.
    ///
    /// # Errors
    ///
    /// A [`SnapshotError`] naming the first bad line: wrong header or
    /// version, unknown section or key, a malformed or non-finite number,
    /// or a state that breaks an editor invariant.
    pub fn parse(text: &str) -> Result<Self, SnapshotError> {
        let mut r = Reader::new(text);
        let mut header = r.record(HEADER)?;
        let version: u32 = header.number("version")?;
        if !(MIN_VERSION..=VERSION).contains(&version) {
            return Err(header.error(format!(
                "unsupported snapshot version {version} (this build reads version {VERSION})"
            )));
        }
        header.end()?;
        let editor = Editor::read_snapshot(&mut r, version)?;
        r.finish()?;
        Ok(Self(editor))
    }
}

/// Builds snapshot text section by section.
#[derive(Debug, Default)]
pub struct Writer {
    pub(crate) out: String,
}

impl Writer {
    /// Starts section `[name]`.
    pub fn section(&mut self, name: &str) {
        let _ = writeln!(self.out, "[{name}]");
    }

    /// Writes one record: `key`, then the tokens `body` appends.
    pub fn record(&mut self, key: &str, body: impl FnOnce(&mut Line<'_>)) {
        self.out.push_str(key);
        body(&mut Line { out: &mut self.out });
        self.out.push('\n');
    }
}

/// The tokens of one record being written.
#[derive(Debug)]
pub struct Line<'a> {
    out: &'a mut String,
}

impl Line<'_> {
    /// Appends a bare word (a number, a name, `-`).
    pub fn word(&mut self, word: impl fmt::Display) -> &mut Self {
        let _ = write!(self.out, " {word}");
        self
    }

    /// Appends a float with round-trip precision.
    pub fn f32(&mut self, value: f32) -> &mut Self {
        self.word(format_args!("{value:?}"))
    }

    /// Appends `1` or `0`.
    pub fn bool(&mut self, value: bool) -> &mut Self {
        self.word(if value { "1" } else { "0" })
    }

    /// Appends `-`, the absent value.
    pub fn none(&mut self) -> &mut Self {
        self.word("-")
    }

    /// Appends a quoted, escaped string.
    pub fn string(&mut self, text: &str) -> &mut Self {
        self.out.push(' ');
        quote(self.out, text);
        self
    }

    /// Appends a value's tokens.
    pub fn put<T: Snap>(&mut self, value: &T) -> &mut Self {
        value.write(self);
        self
    }

    /// Appends a value's tokens, or `-` when absent.
    pub fn opt<T: Snap>(&mut self, value: Option<&T>) -> &mut Self {
        match value {
            Some(value) => self.put(value),
            None => self.none(),
        }
    }
}

/// `text` quoted: `\\`, `\"`, `\n`, `\r`, `\t` and `\u{hex}` for other
/// control characters, so a string never spans lines.
fn quote(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", u32::from(ch));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

/// Reads a quoted string's body (after the opening quote); returns it and
/// the rest of the line after the closing quote.
fn unquote(body: &str) -> Result<(String, &str), String> {
    let mut text = String::new();
    let mut chars = body.char_indices();
    while let Some((at, ch)) = chars.next() {
        match ch {
            '"' => return Ok((text, &body[at + 1..])),
            '\\' => {
                let escaped = match chars.next().map(|(_, c)| c) {
                    Some('\\') => '\\',
                    Some('"') => '"',
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('u') => {
                        if chars.next().map(|(_, c)| c) != Some('{') {
                            return Err("expected `{` after `\\u`".to_owned());
                        }
                        let mut hex = String::new();
                        loop {
                            match chars.next().map(|(_, c)| c) {
                                Some('}') => break,
                                Some(c) if c.is_ascii_hexdigit() && hex.len() < 6 => hex.push(c),
                                _ => return Err("bad `\\u{…}` escape".to_owned()),
                            }
                        }
                        u32::from_str_radix(&hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .ok_or_else(|| format!("`\\u{{{hex}}}` is not a character"))?
                    }
                    Some(other) => return Err(format!("unknown escape `\\{other}`")),
                    None => return Err("unterminated string".to_owned()),
                };
                text.push(escaped);
            }
            ch => text.push(ch),
        }
    }
    Err("unterminated string".to_owned())
}

/// One token of a record.
#[derive(Debug, Clone, PartialEq)]
enum Token<'a> {
    /// A bare word: a key, number, name or `-`.
    Word(&'a str),
    /// A quoted string, unescaped.
    Str(String),
}

/// The tokens of `line`, separated by single spaces.
fn tokenize(line: &str) -> Result<Vec<Token<'_>>, String> {
    let mut items = Vec::new();
    let mut rest = line;
    loop {
        if let Some(body) = rest.strip_prefix('"') {
            let (text, after) = unquote(body)?;
            items.push(Token::Str(text));
            rest = after;
        } else {
            let end = rest.find(' ').unwrap_or(rest.len());
            let word = &rest[..end];
            if word.is_empty() {
                return Err("empty token (extra space or empty line)".to_owned());
            }
            if word.contains('"') {
                return Err(format!("stray quote in `{word}`"));
            }
            items.push(Token::Word(word));
            rest = &rest[end..];
        }
        if rest.is_empty() {
            return Ok(items);
        }
        rest = rest
            .strip_prefix(' ')
            .ok_or_else(|| "expected a space after a string".to_owned())?;
        if rest.is_empty() {
            return Err("trailing space".to_owned());
        }
    }
}

/// Reads snapshot text line by line.
#[derive(Debug)]
pub struct Reader<'a> {
    lines: Vec<&'a str>,
    next: usize,
    /// Text in shapes is allowed (the `shape-text` switch is on).
    shape_text: bool,
}

impl<'a> Reader<'a> {
    /// A reader over `text`; a final newline ends the last line.
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        let mut lines: Vec<&str> = text.split('\n').collect();
        if text.ends_with('\n') {
            lines.pop();
        }
        Self {
            lines,
            next: 0,
            shape_text: false,
        }
    }

    /// Allows or forbids text in the shapes read from now on.
    pub fn set_shape_text(&mut self, on: bool) {
        self.shape_text = on;
    }

    /// 1-based number of the next line.
    #[must_use]
    pub fn line(&self) -> usize {
        self.next + 1
    }

    fn peek(&self) -> Option<&'a str> {
        self.lines.get(self.next).copied()
    }

    fn eof(&self) -> SnapshotError {
        SnapshotError::new(self.line(), "unexpected end of snapshot")
    }

    /// The key of the next line, unless it is a section header or missing.
    #[must_use]
    pub fn peek_key(&self) -> Option<&'a str> {
        let line = self.peek()?;
        if line.starts_with('[') {
            return None;
        }
        Some(line.split(' ').next().unwrap_or(line))
    }

    fn section_error(&self, header: &str) -> SnapshotError {
        let name = header
            .strip_prefix('[')
            .and_then(|h| h.strip_suffix(']'))
            .unwrap_or(header);
        if SECTIONS.contains(&name) {
            SnapshotError::new(self.line(), format!("section `[{name}]` out of order"))
        } else {
            SnapshotError::new(self.line(), format!("unknown section `{header}`"))
        }
    }

    /// Enters section `[name]`.
    ///
    /// # Errors
    ///
    /// When the next line is another section, a record or missing.
    pub fn section(&mut self, name: &str) -> Result<(), SnapshotError> {
        let line = self.peek().ok_or_else(|| self.eof())?;
        if line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) == Some(name) {
            self.next += 1;
            return Ok(());
        }
        if line.starts_with('[') {
            return Err(self.section_error(line));
        }
        let key = line.split(' ').next().unwrap_or(line);
        Err(SnapshotError::new(
            self.line(),
            format!("unknown key `{key}` (expected section `[{name}]`)"),
        ))
    }

    /// The next record, whatever its key.
    ///
    /// # Errors
    ///
    /// When the next line is missing, a section header or not tokens.
    pub fn any_record(&mut self) -> Result<(&'a str, Tokens<'a>), SnapshotError> {
        let line = self.peek().ok_or_else(|| self.eof())?;
        if line.starts_with('[') {
            return Err(self.section_error(line));
        }
        let number = self.line();
        let mut items = tokenize(line).map_err(|err| SnapshotError::new(number, err))?;
        let key = match items.first() {
            Some(Token::Word(key)) => *key,
            _ => return Err(SnapshotError::new(number, "a record starts with a key")),
        };
        items.remove(0);
        self.next += 1;
        Ok((
            key,
            Tokens {
                line: number,
                items,
                next: 0,
                shape_text: self.shape_text,
            },
        ))
    }

    /// The next record, which must have `key`.
    ///
    /// # Errors
    ///
    /// When the next line is missing, malformed or has another key.
    pub fn record(&mut self, key: &str) -> Result<Tokens<'a>, SnapshotError> {
        let number = self.line();
        let (found, tokens) = self.any_record()?;
        if found == key {
            Ok(tokens)
        } else {
            self.next -= 1;
            Err(SnapshotError::new(
                number,
                format!("unknown key `{found}` (expected `{key}`)"),
            ))
        }
    }

    /// Checks that nothing follows the last section.
    ///
    /// # Errors
    ///
    /// When lines are left.
    pub fn finish(&self) -> Result<(), SnapshotError> {
        match self.peek() {
            None => Ok(()),
            Some(line) if line.starts_with('[') => Err(self.section_error(line)),
            Some(line) => Err(SnapshotError::new(
                self.line(),
                format!(
                    "unknown key `{}` after the last record",
                    line.split(' ').next().unwrap_or(line)
                ),
            )),
        }
    }
}

/// The tokens of one record being read.
#[derive(Debug)]
pub struct Tokens<'a> {
    line: usize,
    items: Vec<Token<'a>>,
    next: usize,
    shape_text: bool,
}

impl<'a> Tokens<'a> {
    /// The record's 1-based line.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// An error on this record's line.
    #[must_use]
    pub fn error(&self, message: impl Into<String>) -> SnapshotError {
        SnapshotError::new(self.line, message)
    }

    fn take(&mut self, what: &str) -> Result<Token<'a>, SnapshotError> {
        let token = self
            .items
            .get(self.next)
            .cloned()
            .ok_or_else(|| self.error(format!("missing {what}")))?;
        self.next += 1;
        Ok(token)
    }

    /// The next bare word.
    ///
    /// # Errors
    ///
    /// When it is missing or a string.
    pub fn word(&mut self, what: &str) -> Result<&'a str, SnapshotError> {
        match self.take(what)? {
            Token::Word(word) => Ok(word),
            Token::Str(_) => Err(self.error(format!("expected {what}, found a string"))),
        }
    }

    /// The next quoted string.
    ///
    /// # Errors
    ///
    /// When it is missing or a bare word.
    pub fn string(&mut self, what: &str) -> Result<String, SnapshotError> {
        match self.take(what)? {
            Token::Str(text) => Ok(text),
            Token::Word(word) => {
                Err(self.error(format!("expected {what} (a string), found `{word}`")))
            }
        }
    }

    /// The next word parsed as a number.
    ///
    /// # Errors
    ///
    /// When it is missing or not a number of type `T`.
    pub fn number<T: FromStr>(&mut self, what: &str) -> Result<T, SnapshotError> {
        let word = self.word(what)?;
        word.parse()
            .map_err(|_| self.error(format!("bad {what} `{word}`")))
    }

    /// The next word as a finite float.
    ///
    /// # Errors
    ///
    /// When it is missing, not a number or not finite.
    pub fn f32(&mut self, what: &str) -> Result<f32, SnapshotError> {
        let value: f32 = self.number(what)?;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(self.error(format!("{what} is not finite")))
        }
    }

    /// The next word as any float, `inf` and `NaN` included: for live
    /// drag positions, which may leave the finite range before a commit
    /// drops them.
    ///
    /// # Errors
    ///
    /// When it is missing or not a number.
    pub fn loose_f32(&mut self, what: &str) -> Result<f32, SnapshotError> {
        self.number(what)
    }

    /// The next two words as a point that may be non-finite (see
    /// [`Tokens::loose_f32`]).
    ///
    /// # Errors
    ///
    /// When they are missing or not numbers.
    pub fn loose_vec2(&mut self) -> Result<Vec2, SnapshotError> {
        Ok(Vec2::new(self.loose_f32("x")?, self.loose_f32("y")?))
    }

    /// A list of segments whose points may be non-finite.
    ///
    /// # Errors
    ///
    /// When the tokens are missing or not numbers.
    pub fn loose_segments(&mut self) -> Result<Vec<[Vec2; 2]>, SnapshotError> {
        let len: usize = self.number("list length")?;
        let mut items = Vec::new();
        for _ in 0..len {
            items.push([self.loose_vec2()?, self.loose_vec2()?]);
        }
        Ok(items)
    }

    /// The next word as `0` or `1`.
    ///
    /// # Errors
    ///
    /// When it is anything else.
    pub fn bool(&mut self, what: &str) -> Result<bool, SnapshotError> {
        match self.word(what)? {
            "0" => Ok(false),
            "1" => Ok(true),
            other => Err(self.error(format!("bad {what} `{other}` (0 or 1)"))),
        }
    }

    /// Consumes `-` if it is next; whether it was.
    pub fn none(&mut self) -> bool {
        let absent = self.items.get(self.next) == Some(&Token::Word("-"));
        if absent {
            self.next += 1;
        }
        absent
    }

    /// The next value.
    ///
    /// # Errors
    ///
    /// When its tokens are missing or bad.
    pub fn get<T: Snap>(&mut self) -> Result<T, SnapshotError> {
        T::read(self)
    }

    /// The next value, or `None` for `-`.
    ///
    /// # Errors
    ///
    /// When its tokens are bad.
    pub fn opt<T: Snap>(&mut self) -> Result<Option<T>, SnapshotError> {
        if self.none() {
            Ok(None)
        } else {
            self.get().map(Some)
        }
    }

    /// Whether text in shapes is allowed in this record.
    #[must_use]
    pub fn shape_text(&self) -> bool {
        self.shape_text
    }

    /// Whether every token was read.
    #[must_use]
    pub fn at_end(&self) -> bool {
        self.next >= self.items.len()
    }

    /// Checks that every token was read.
    ///
    /// # Errors
    ///
    /// When tokens are left.
    pub fn end(&self) -> Result<(), SnapshotError> {
        if self.next < self.items.len() {
            Err(self.error(format!(
                "{} unexpected token(s) at the end of the record",
                self.items.len() - self.next
            )))
        } else {
            Ok(())
        }
    }
}

/// A value with a token form inside a record.
pub trait Snap: Sized {
    /// Appends the value's tokens.
    fn write(&self, line: &mut Line<'_>);

    /// Reads the value's tokens.
    ///
    /// # Errors
    ///
    /// When the tokens are missing, malformed or out of range.
    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError>;
}

impl Snap for f32 {
    fn write(&self, line: &mut Line<'_>) {
        line.f32(*self);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        tokens.f32("number")
    }
}

impl Snap for u32 {
    fn write(&self, line: &mut Line<'_>) {
        line.word(self);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        tokens.number("count")
    }
}

impl Snap for bool {
    fn write(&self, line: &mut Line<'_>) {
        line.bool(*self);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        tokens.bool("flag")
    }
}

/// A list: its length, then its items.
impl<T: Snap> Snap for Vec<T> {
    fn write(&self, line: &mut Line<'_>) {
        line.word(self.len());
        for item in self {
            line.put(item);
        }
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let len: usize = tokens.number("list length")?;
        // No `with_capacity`: a bad length runs out of tokens instead.
        let mut items = Vec::new();
        for _ in 0..len {
            items.push(tokens.get()?);
        }
        Ok(items)
    }
}

impl Snap for Vec2 {
    fn write(&self, line: &mut Line<'_>) {
        line.f32(self.x).f32(self.y);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        Ok(Self::new(tokens.f32("x")?, tokens.f32("y")?))
    }
}

/// A guide or any segment: two points.
impl Snap for [Vec2; 2] {
    fn write(&self, line: &mut Line<'_>) {
        line.put(&self[0]).put(&self[1]);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        Ok([tokens.get()?, tokens.get()?])
    }
}

impl Snap for Aabb {
    fn write(&self, line: &mut Line<'_>) {
        line.put(&self.min).put(&self.max);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let (min, max): (Vec2, Vec2) = (tokens.get()?, tokens.get()?);
        if min.x <= max.x && min.y <= max.y {
            Ok(Self { min, max })
        } else {
            Err(tokens.error("box with min above max"))
        }
    }
}

impl Snap for ShapeId {
    fn write(&self, line: &mut Line<'_>) {
        line.word(self.get());
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        tokens.number("shape id").map(ShapeId)
    }
}

impl Snap for ColorId {
    fn write(&self, line: &mut Line<'_>) {
        line.word(self.index());
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let index: u8 = tokens.number("colour")?;
        Self::new(index).ok_or_else(|| tokens.error(format!("unknown colour {index}")))
    }
}

impl Snap for Style {
    fn write(&self, line: &mut Line<'_>) {
        line.put(&self.color).f32(self.width);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let color = tokens.get()?;
        let width = tokens.f32("width")?;
        if width < 0.0 {
            return Err(tokens.error("negative width"));
        }
        Ok(Self { color, width })
    }
}

/// Reads one of `names`, returning its index.
fn choice(tokens: &mut Tokens<'_>, what: &str, names: &[&str]) -> Result<usize, SnapshotError> {
    let word = tokens.word(what)?;
    names
        .iter()
        .position(|name| *name == word)
        .ok_or_else(|| tokens.error(format!("unknown {what} `{word}`")))
}

const TOOLS: [(Tool, &str); 11] = [
    (Tool::Pen, "pen"),
    (Tool::Line, "line"),
    (Tool::Arrow, "arrow"),
    (Tool::Rect, "rect"),
    (Tool::Ellipse, "ellipse"),
    (Tool::Grid, "grid"),
    (Tool::Text, "text"),
    (Tool::Eraser, "eraser"),
    (Tool::Bucket, "bucket"),
    (Tool::Select, "select"),
    (Tool::Hand, "hand"),
];

impl Snap for Tool {
    fn write(&self, line: &mut Line<'_>) {
        let name = TOOLS
            .iter()
            .find(|(t, _)| t == self)
            .map_or("pen", |(_, n)| n);
        line.word(name);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let names = TOOLS.map(|(_, name)| name);
        choice(tokens, "tool", &names).map(|i| TOOLS[i].0)
    }
}

const BUTTONS: [(PointerButton, &str); 3] = [
    (PointerButton::Left, "left"),
    (PointerButton::Middle, "middle"),
    (PointerButton::Right, "right"),
];

impl Snap for PointerButton {
    fn write(&self, line: &mut Line<'_>) {
        let name = BUTTONS
            .iter()
            .find(|(b, _)| b == self)
            .map_or("left", |(_, n)| n);
        line.word(name);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let names = BUTTONS.map(|(_, name)| name);
        choice(tokens, "button", &names).map(|i| BUTTONS[i].0)
    }
}

/// Three letters: `s`, `c`, `a` for held Shift, Ctrl, Alt, `-` for not.
impl Snap for Modifiers {
    fn write(&self, line: &mut Line<'_>) {
        let flag = |on: bool, ch: char| if on { ch } else { '-' };
        let word: String = [
            flag(self.shift, 's'),
            flag(self.ctrl, 'c'),
            flag(self.alt, 'a'),
        ]
        .iter()
        .collect();
        line.word(word);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let word = tokens.word("modifiers")?;
        let bytes = word.as_bytes();
        let flag = |i: usize, ch: u8| match bytes.get(i) {
            Some(b) if *b == ch => Some(true),
            Some(b'-') => Some(false),
            _ => None,
        };
        match (bytes.len(), flag(0, b's'), flag(1, b'c'), flag(2, b'a')) {
            (3, Some(shift), Some(ctrl), Some(alt)) => Ok(Self { shift, ctrl, alt }),
            _ => Err(tokens.error(format!("bad modifiers `{word}` (like `s-a`)"))),
        }
    }
}

const SMOOTHING: [(SmoothingLevel, &str); 4] = [
    (SmoothingLevel::Off, "off"),
    (SmoothingLevel::Low, "low"),
    (SmoothingLevel::Medium, "medium"),
    (SmoothingLevel::High, "high"),
];

impl Snap for SmoothingLevel {
    fn write(&self, line: &mut Line<'_>) {
        let name = SMOOTHING
            .iter()
            .find(|(l, _)| l == self)
            .map_or("low", |(_, n)| n);
        line.word(name);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let names = SMOOTHING.map(|(_, name)| name);
        choice(tokens, "smoothing", &names).map(|i| SMOOTHING[i].0)
    }
}

/// `shape` or `cell <col> <row>`.
impl Snap for TextSlot {
    fn write(&self, line: &mut Line<'_>) {
        match self {
            Self::Shape => line.word("shape"),
            Self::Cell { col, row } => line.word("cell").word(col).word(row),
        };
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        match choice(tokens, "text slot", &["shape", "cell"])? {
            0 => Ok(Self::Shape),
            _ => Ok(Self::Cell {
                col: tokens.number("column")?,
                row: tokens.number("row")?,
            }),
        }
    }
}

/// `<size> "<text>"`, non-blank with a positive size.
impl Snap for ShapeText {
    fn write(&self, line: &mut Line<'_>) {
        line.f32(self.size).string(&self.text);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let size = tokens.f32("text size")?;
        let text = tokens.string("text")?;
        if !text_size_ok(size) {
            return Err(tokens.error(format!("text size {size} out of range")));
        }
        Self::new(text, size).ok_or_else(|| tokens.error("blank text or non-positive size"))
    }
}

impl Snap for CellFill {
    fn write(&self, line: &mut Line<'_>) {
        line.word(self.col).word(self.row).put(&self.color);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        Ok(Self {
            col: tokens.number("column")?,
            row: tokens.number("row")?,
            color: tokens.get()?,
        })
    }
}

impl Snap for CellText {
    fn write(&self, line: &mut Line<'_>) {
        line.word(self.col).word(self.row).put(&self.content);
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        Ok(Self {
            col: tokens.number("column")?,
            row: tokens.number("row")?,
            content: tokens.get()?,
        })
    }
}

/// Whether `cells` lie inside `cols × rows`, sorted by `(row, col)` and
/// unique.
fn cells_ok(cells: impl Iterator<Item = (u32, u32)>, cols: u32, rows: u32) -> bool {
    let mut last: Option<(u32, u32)> = None;
    for (col, row) in cells {
        if col >= cols || row >= rows || last.is_some_and(|l| l >= (row, col)) {
            return false;
        }
        last = Some((row, col));
    }
    true
}

impl Snap for Shape {
    fn write(&self, line: &mut Line<'_>) {
        match self {
            Self::Stroke {
                points,
                widths,
                style,
            } => {
                line.word("stroke")
                    .put(style)
                    .put(points)
                    .opt(widths.as_ref());
            }
            Self::Line { a, b, style } => {
                line.word("line").put(style).put(a).put(b);
            }
            Self::Arrow { a, b, style } => {
                line.word("arrow").put(style).put(a).put(b);
            }
            Self::Rect {
                a,
                b,
                style,
                fill,
                label,
                text,
            }
            | Self::Ellipse {
                a,
                b,
                style,
                fill,
                label,
                text,
            } => {
                let kind = if matches!(self, Self::Rect { .. }) {
                    "rect"
                } else {
                    "ellipse"
                };
                line.word(kind)
                    .put(style)
                    .put(a)
                    .put(b)
                    .opt(fill.as_ref())
                    .opt(label.as_ref())
                    .opt(text.as_ref());
            }
            Self::Grid {
                a,
                b,
                cols,
                rows,
                style,
                axes,
                fills,
                texts,
            } => {
                line.word("grid")
                    .put(style)
                    .put(a)
                    .put(b)
                    .word(cols)
                    .word(rows)
                    .bool(*axes)
                    .put(fills)
                    .put(texts);
            }
            Self::Text {
                origin,
                text,
                size,
                style,
            } => {
                line.word("text")
                    .put(style)
                    .put(origin)
                    .f32(*size)
                    .string(text);
            }
        }
    }

    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError> {
        let kinds = ["stroke", "line", "arrow", "rect", "ellipse", "grid", "text"];
        let kind = choice(tokens, "shape kind", &kinds)?;
        let style: Style = tokens.get()?;
        let shape = match kind {
            0 => read_stroke(tokens, style)?,
            1 | 2 => {
                let (a, b) = (tokens.get()?, tokens.get()?);
                if kind == 1 {
                    Self::Line { a, b, style }
                } else {
                    Self::Arrow { a, b, style }
                }
            }
            3 | 4 => {
                let (a, b) = (tokens.get()?, tokens.get()?);
                let fill = tokens.opt()?;
                let label: Option<u32> = tokens.opt()?;
                let text: Option<ShapeText> = tokens.opt()?;
                if kind == 3 {
                    Self::Rect {
                        a,
                        b,
                        style,
                        fill,
                        label,
                        text,
                    }
                } else {
                    Self::Ellipse {
                        a,
                        b,
                        style,
                        fill,
                        label,
                        text,
                    }
                }
            }
            5 => read_grid(tokens, style)?,
            _ => {
                let origin = tokens.get()?;
                let size = tokens.f32("text size")?;
                let text = tokens.string("text")?;
                if !text_size_ok(size) || text.trim().is_empty() {
                    return Err(tokens.error("blank text or text size out of range"));
                }
                Self::Text {
                    origin,
                    text,
                    size,
                    style,
                }
            }
        };
        let has_text = shape.slot_text(TextSlot::Shape).is_some() || !shape.cell_texts().is_empty();
        if has_text && !tokens.shape_text() {
            return Err(tokens.error("text in a shape while the shape-text switch is off"));
        }
        if !shape.is_finite() {
            return Err(tokens.error("shape is not finite"));
        }
        let bounds = shape.bounds();
        if matches!(shape, Self::Text { .. }) && !(bounds.min.is_finite() && bounds.max.is_finite())
        {
            return Err(tokens.error("text block too large: its bounds are not finite"));
        }
        Ok(shape)
    }
}

/// The rest of a `stroke` shape after its style.
fn read_stroke(tokens: &mut Tokens<'_>, style: Style) -> Result<Shape, SnapshotError> {
    let points: Vec<Vec2> = tokens.get()?;
    let widths: Option<Vec<f32>> = tokens.opt()?;
    if points.is_empty() {
        return Err(tokens.error("stroke without points"));
    }
    if let Some(widths) = &widths {
        if widths.len() != points.len() {
            return Err(tokens.error("stroke widths out of step with its points"));
        }
        if widths.iter().any(|w| *w <= 0.0) {
            return Err(tokens.error("stroke width factor not positive"));
        }
    }
    Ok(Shape::Stroke {
        points,
        widths,
        style,
    })
}

/// The rest of a `grid` shape after its style.
fn read_grid(tokens: &mut Tokens<'_>, style: Style) -> Result<Shape, SnapshotError> {
    let (a, b) = (tokens.get()?, tokens.get()?);
    let cols: u32 = tokens.number("columns")?;
    let rows: u32 = tokens.number("rows")?;
    let axes = tokens.bool("axes")?;
    let fills: Vec<CellFill> = tokens.get()?;
    let texts: Vec<CellText> = tokens.get()?;
    let cells = 1..=GRID_MAX_CELLS;
    if !cells.contains(&cols) || !cells.contains(&rows) {
        return Err(tokens.error(format!("grid of {cols} × {rows} cells")));
    }
    if !cells_ok(fills.iter().map(|f| (f.col, f.row)), cols, rows)
        || !cells_ok(texts.iter().map(|t| (t.col, t.row)), cols, rows)
    {
        return Err(tokens.error("grid cells outside the grid, unsorted or repeated"));
    }
    Ok(Shape::Grid {
        a,
        b,
        cols,
        rows,
        style,
        axes,
        fills,
        texts,
    })
}

/// Writes `edit` as an `insert`, `remove`, `replace`, `lock` or `unlock`
/// record.
fn write_edit(w: &mut Writer, edit: &Edit) {
    match edit {
        Edit::Insert { index, id, shape } => w.record("insert", |l| {
            l.word(index).put(id).put(shape);
        }),
        Edit::Remove { index, id, shape } => w.record("remove", |l| {
            l.word(index).put(id).put(shape);
        }),
        Edit::Replace { id, before, after } => w.record("replace", |l| {
            l.put(id).put(before).put(after);
        }),
        Edit::Lock { id, locked } => w.record(if *locked { "lock" } else { "unlock" }, |l| {
            l.put(id);
        }),
    }
}

/// Reads one edit record.
fn read_edit(r: &mut Reader<'_>) -> Result<Edit, SnapshotError> {
    let (key, mut t) = r.any_record()?;
    let edit = match key {
        "insert" | "remove" => {
            let index = t.number("index")?;
            let id = t.get()?;
            let shape = t.get()?;
            if key == "insert" {
                Edit::Insert { index, id, shape }
            } else {
                Edit::Remove { index, id, shape }
            }
        }
        "replace" => Edit::Replace {
            id: t.get()?,
            before: t.get()?,
            after: t.get()?,
        },
        "lock" | "unlock" => Edit::Lock {
            id: t.get()?,
            locked: key == "lock",
        },
        other => return Err(t.error(format!("unknown key `{other}` (expected an edit)"))),
    };
    t.end()?;
    Ok(edit)
}

/// Writes the undo (`undo`) or redo (`redo`) transactions.
pub(crate) fn write_transactions<'t>(
    w: &mut Writer,
    key: &str,
    transactions: impl IntoIterator<Item = &'t Transaction>,
) {
    for tx in transactions {
        w.record(key, |l| {
            l.word(tx.len());
        });
        for edit in tx.edits() {
            write_edit(w, edit);
        }
    }
}

/// Reads the `key` (`undo` or `redo`) transactions.
pub(crate) fn read_transactions(
    r: &mut Reader<'_>,
    key: &str,
) -> Result<Vec<Transaction>, SnapshotError> {
    let mut transactions = Vec::new();
    while r.peek_key() == Some(key) {
        let t = r.record(key)?;
        let len: usize = {
            let mut t = t;
            let len = t.number("edit count")?;
            t.end()?;
            len
        };
        let mut edits = Vec::new();
        for _ in 0..len {
            edits.push(read_edit(r)?);
        }
        transactions.push(Transaction(edits));
    }
    Ok(transactions)
}

/// Writes the `[document]` section.
pub(crate) fn write_document(w: &mut Writer, doc: &Document) {
    w.section("document");
    w.record("next-id", |l| {
        l.word(doc.peek_next_id());
    });
    for (id, shape) in doc.shapes() {
        w.record("shape", |l| {
            l.put(&id).put(shape);
        });
    }
    // Only when something is locked (ADR-T107-1).
    if doc.locked_ids().next().is_some() {
        w.record("locked", |l| {
            for id in doc.locked_ids() {
                l.put(&id);
            }
        });
    }
}

/// Reads the `[document]` section: ids unique and below `next-id`, locked
/// ids in the document and each once.
pub(crate) fn read_document(r: &mut Reader<'_>) -> Result<Document, SnapshotError> {
    r.section("document")?;
    let mut t = r.record("next-id")?;
    let next_id: u64 = t.number("next id")?;
    t.end()?;
    let mut shapes = Vec::new();
    let mut seen = std::collections::HashSet::new();
    while r.peek_key() == Some("shape") {
        let mut t = r.record("shape")?;
        let id: ShapeId = t.get()?;
        let shape = t.get()?;
        t.end()?;
        if id.get() >= next_id {
            return Err(t.error(format!("id {} is not below next-id {next_id}", id.get())));
        }
        if !seen.insert(id) {
            return Err(t.error(format!("duplicate id {}", id.get())));
        }
        shapes.push((id, shape));
    }
    let mut locked = IdSet::default();
    if r.peek_key() == Some("locked") {
        let mut t = r.record("locked")?;
        while !t.at_end() {
            let id: ShapeId = t.get()?;
            if !seen.contains(&id) {
                return Err(t.error(format!("locked id {} is not in the document", id.get())));
            }
            if !locked.insert(id) {
                return Err(t.error(format!("locked id {} appears twice", id.get())));
            }
        }
        if locked.is_empty() {
            return Err(t.error("empty locked record"));
        }
    }
    Ok(Document::from_parts(shapes, locked, next_id))
}

/// Writes the `[clipboard]` section.
pub(crate) fn write_clipboard(w: &mut Writer, shapes: &[Shape]) {
    w.section("clipboard");
    for shape in shapes {
        w.record("shape", |l| {
            l.put(shape);
        });
    }
}

/// Reads the `[clipboard]` section.
pub(crate) fn read_clipboard(r: &mut Reader<'_>) -> Result<Vec<Shape>, SnapshotError> {
    r.section("clipboard")?;
    let mut shapes = Vec::new();
    while r.peek_key() == Some("shape") {
        let mut t = r.record("shape")?;
        shapes.push(t.get()?);
        t.end()?;
    }
    Ok(shapes)
}

/// Checks that every id in `ids` is in `doc` and appears once.
///
/// # Errors
///
/// On the record's line, naming the first bad id.
pub(crate) fn check_ids(
    tokens: &Tokens<'_>,
    doc: &Document,
    ids: &[ShapeId],
) -> Result<(), SnapshotError> {
    let mut seen = std::collections::HashSet::new();
    for id in ids {
        if doc.get(*id).is_none() {
            return Err(tokens.error(format!("id {} is not in the document", id.get())));
        }
        if !seen.insert(*id) {
            return Err(tokens.error(format!("id {} appears twice", id.get())));
        }
    }
    Ok(())
}

/// [`check_ids`], and none of `ids` is locked: what the selection and the
/// select and eraser gestures hold (ADR-T107-1).
///
/// # Errors
///
/// On the record's line, naming the first bad id.
pub(crate) fn check_unlocked_ids(
    tokens: &Tokens<'_>,
    doc: &Document,
    ids: &[ShapeId],
) -> Result<(), SnapshotError> {
    check_ids(tokens, doc, ids)?;
    match ids.iter().find(|id| doc.is_locked(**id)) {
        Some(id) => Err(tokens.error(format!("id {} is locked", id.get()))),
        None => Ok(()),
    }
}

/// Golden files: compare a state's text with a committed one (AC-5). The
/// file system and `DRAW_UPDATE_SNAPSHOTS` stay with the caller.
pub mod golden {
    use std::fmt::Write as _;

    /// Changed lines shown per side before the diff is cut.
    const MAX_DIFF_LINES: usize = 20;

    /// The outcome of [`check`].
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Golden {
        /// The texts are equal.
        Match,
        /// The caller should write the actual text (update requested).
        Write,
        /// The texts differ: a readable line diff.
        Mismatch(String),
        /// There is no golden file and no update was requested.
        Missing,
    }

    /// Compares `actual` with the golden `expected` text.
    #[must_use]
    pub fn check(actual: &str, expected: Option<&str>, update: bool) -> Golden {
        match expected {
            Some(expected) if expected == actual => Golden::Match,
            _ if update => Golden::Write,
            None => Golden::Missing,
            Some(expected) => Golden::Mismatch(diff(expected, actual)),
        }
    }

    /// A short line diff from `expected` to `actual`: the changed block
    /// between the common head and tail, `-` for expected lines and `+`
    /// for actual ones, at most `MAX_DIFF_LINES` (20) per side.
    #[must_use]
    pub fn diff(expected: &str, actual: &str) -> String {
        let old: Vec<&str> = expected.lines().collect();
        let new: Vec<&str> = actual.lines().collect();
        let head = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
        let tail = old[head..]
            .iter()
            .rev()
            .zip(new[head..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        let mut out = format!("first difference at line {}\n", head + 1);
        for (lines, sign) in [
            (&old[head..old.len() - tail], '-'),
            (&new[head..new.len() - tail], '+'),
        ] {
            for line in lines.iter().take(MAX_DIFF_LINES) {
                let _ = writeln!(out, "{sign} {line}");
            }
            if lines.len() > MAX_DIFF_LINES {
                let _ = writeln!(out, "{sign} … {} more lines", lines.len() - MAX_DIFF_LINES);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::{Command, Tool};
    use crate::core::editor::Editor;
    use crate::core::geom::Vec2;
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use std::path::Path;

    const NONE: Modifiers = Modifiers::NONE;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn editor() -> Editor {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: v(800.0, 600.0),
        });
        ed
    }

    fn down(ed: &mut Editor, pos: Vec2) {
        ed.handle(InputEvent::PointerMove { pos, mods: NONE });
        ed.handle(InputEvent::PointerDown {
            pos,
            button: PointerButton::Left,
            mods: NONE,
        });
    }

    fn move_to(ed: &mut Editor, pos: Vec2) {
        ed.handle(InputEvent::PointerMove { pos, mods: NONE });
    }

    fn up(ed: &mut Editor, pos: Vec2) {
        ed.handle(InputEvent::PointerUp {
            pos,
            button: PointerButton::Left,
            mods: NONE,
        });
    }

    fn drag(ed: &mut Editor, a: Vec2, b: Vec2) {
        down(ed, a);
        for i in 1..=8 {
            let t = i as f32 / 8.0;
            move_to(
                ed,
                v(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t + t * 3.0),
            );
        }
        up(ed, b);
    }

    fn type_text(ed: &mut Editor, text: &str) {
        for ch in text.chars() {
            ed.handle(InputEvent::Char { ch, mods: NONE });
        }
    }

    fn key(ed: &mut Editor, key: Key) {
        ed.handle(InputEvent::KeyDown { key, mods: NONE });
        ed.handle(InputEvent::KeyUp { key, mods: NONE });
    }

    /// `ed` through text and back.
    fn round_trip(ed: &Editor) -> Editor {
        let text = ed.snapshot().to_text();
        let snapshot = Snapshot::parse(&text).unwrap_or_else(|err| panic!("{err}\n{text}"));
        Editor::from_snapshot(snapshot)
    }

    /// A few shapes, an undo step and a redo step, a selection and a
    /// clipboard.
    fn busy_editor() -> Editor {
        let mut ed = editor();
        drag(&mut ed, v(10.0, 10.0), v(200.0, 120.0));
        ed.apply(Command::SetTool(Tool::Rect));
        drag(&mut ed, v(300.0, 50.0), v(400.0, 150.0));
        ed.apply(Command::SetTool(Tool::Grid));
        drag(&mut ed, v(50.0, 300.0), v(250.0, 450.0));
        ed.apply(Command::SetTool(Tool::Arrow));
        drag(&mut ed, v(500.0, 300.0), v(700.0, 320.0));
        ed.apply(Command::SetTool(Tool::Text));
        down(&mut ed, v(450.0, 450.0));
        up(&mut ed, v(450.0, 450.0));
        type_text(&mut ed, "x = \"a\\b\"\n$\\sum_{i} i^2$");
        key(&mut ed, Key::Escape);
        ed.apply(Command::Undo);
        ed.apply(Command::SelectAll);
        ed.apply(Command::Copy);
        ed.apply(Command::SetColor(crate::core::palette::ColorId::INK));
        ed
    }

    fn fixtures() -> Vec<std::path::PathBuf> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
        let mut found = Vec::new();
        for dir in [root.clone(), root.join("regressions")] {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "snap") {
                    found.push(path);
                }
            }
        }
        found.sort();
        found
    }

    /// The text of `ed` with `line` (0-based) replaced by `with`.
    fn edited(ed: &Editor, line: usize, with: &str) -> String {
        let text = ed.snapshot().to_text();
        let mut lines: Vec<&str> = text.lines().collect();
        lines[line] = with;
        lines.join("\n") + "\n"
    }

    /// 0-based index of the first line starting with `prefix`.
    fn line_of(text: &str, prefix: &str) -> usize {
        text.lines()
            .position(|l| l.starts_with(prefix))
            .unwrap_or_else(|| panic!("no line {prefix:?} in\n{text}"))
    }

    // AC-1

    #[test]
    fn round_trip_new_editor_is_equal() {
        // Arrange
        let ed = Editor::new();

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded, ed);
    }

    #[test]
    fn round_trip_busy_editor_is_equal_and_undo_redo_agree() {
        // Arrange
        let mut ed = busy_editor();

        // Act
        let mut loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded, ed);
        for command in [Command::Redo, Command::Undo, Command::Undo, Command::Paste] {
            ed.apply(command);
            loaded.apply(command);
            assert_eq!(loaded, ed, "after {command:?}");
        }
    }

    #[test]
    fn round_trip_mid_gesture_states() {
        // Arrange: one editor per in-progress gesture.
        let mut pen = editor();
        down(&mut pen, v(10.0, 10.0));
        move_to(&mut pen, v(40.0, 30.0));
        move_to(&mut pen, v(80.0, 35.0));

        let mut shape = editor();
        shape.apply(Command::SetTool(Tool::Ellipse));
        down(&mut shape, v(10.0, 10.0));
        move_to(&mut shape, v(90.0, 70.0));

        let mut grid = editor();
        grid.apply(Command::SetTool(Tool::Grid));
        down(&mut grid, v(10.0, 10.0));
        move_to(&mut grid, v(90.0, 70.0));

        let mut select = busy_editor();
        select.apply(Command::SetTool(Tool::Select));
        down(&mut select, v(100.0, 60.0));
        move_to(&mut select, v(130.0, 90.0));

        let mut marquee = busy_editor();
        marquee.apply(Command::SetTool(Tool::Select));
        down(&mut marquee, v(780.0, 590.0));
        move_to(&mut marquee, v(600.0, 400.0));

        let mut eraser = busy_editor();
        eraser.apply(Command::SetTool(Tool::Eraser));
        down(&mut eraser, v(5.0, 5.0));
        move_to(&mut eraser, v(200.0, 120.0));

        let mut text = editor();
        text.apply(Command::SetTool(Tool::Text));
        down(&mut text, v(100.0, 100.0));
        up(&mut text, v(100.0, 100.0));
        type_text(&mut text, "ab $x^2$ cd");
        key(&mut text, Key::ArrowLeft);
        key(&mut text, Key::ArrowLeft);
        key(&mut text, Key::ArrowLeft);
        key(&mut text, Key::ArrowLeft);

        let mut pan = editor();
        pan.handle(InputEvent::PointerDown {
            pos: v(50.0, 50.0),
            button: PointerButton::Middle,
            mods: NONE,
        });
        move_to(&mut pan, v(70.0, 90.0));

        let mut tap = Editor::new().with_tap_slop(16.0);
        down(&mut tap, v(10.0, 10.0));
        move_to(&mut tap, v(12.0, 11.0));
        tap.handle(InputEvent::Pressure {
            pressure: Some(0.25),
        });

        for (name, ed) in [
            ("pen", pen),
            ("shape", shape),
            ("grid", grid),
            ("select", select),
            ("marquee", marquee),
            ("eraser", eraser),
            ("text", text),
            ("pan", pan),
            ("tap", tap),
        ] {
            // Act
            let mut loaded = round_trip(&ed);
            let mut ed = ed;

            // Assert: equal now and after the gesture ends the same way.
            assert_eq!(loaded, ed, "{name}");
            for event in [
                InputEvent::PointerMove {
                    pos: v(150.0, 140.0),
                    mods: NONE,
                },
                InputEvent::Char {
                    ch: 'z',
                    mods: NONE,
                },
                InputEvent::PointerUp {
                    pos: v(150.0, 140.0),
                    button: PointerButton::Left,
                    mods: NONE,
                },
                InputEvent::PointerUp {
                    pos: v(150.0, 140.0),
                    button: PointerButton::Middle,
                    mods: NONE,
                },
                InputEvent::KeyDown {
                    key: Key::Escape,
                    mods: NONE,
                },
            ] {
                ed.handle(event.clone());
                loaded.handle(event.clone());
                assert_eq!(loaded, ed, "{name} after {event:?}");
            }
        }
    }

    #[test]
    fn snapshot_ignores_document_revisions() {
        // Arrange
        let ed = busy_editor();
        assert!(ed.document_revision() > 0);

        // Act
        let loaded = round_trip(&ed);

        // Assert: a load is a fresh start for the shell (ADR-T74-1).
        assert_eq!(loaded.document_revision(), 0);
        assert_eq!(loaded.document_base_revision(), 0);
        assert_eq!(loaded, ed);
    }

    /// Zooms all the way out, so a huge screen position is an infinite
    /// world point.
    fn zoomed_out() -> Editor {
        let mut ed = editor();
        for _ in 0..200 {
            ed.handle(InputEvent::Scroll {
                pos: v(400.0, 300.0),
                delta: -1.0,
            });
        }
        ed
    }

    #[test]
    #[allow(non_snake_case)] // `regression_<ID>_<slug>` (Workflow)
    fn regression_T74_live_drag_at_infinity_round_trips() {
        // Arrange: a grid drag whose end is off the finite world.
        let mut ed = zoomed_out();
        ed.apply(Command::SetTool(Tool::Grid));
        down(&mut ed, v(10.0, 10.0));
        move_to(&mut ed, v(3.0e38, 10.0));

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded.snapshot().to_text(), ed.snapshot().to_text());
    }

    #[test]
    #[allow(non_snake_case)] // `regression_<ID>_<slug>` (Workflow)
    fn regression_T74_text_opened_at_infinity_round_trips() {
        // Arrange: a block opened at an infinite world point.
        let mut ed = zoomed_out();
        ed.apply(Command::SetTool(Tool::Text));
        down(&mut ed, v(-3.0e38, 10.0));
        up(&mut ed, v(-3.0e38, 10.0));
        type_text(&mut ed, "x");
        assert!(ed.text_edit().is_some_and(|e| !e.origin.is_finite()));

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded.snapshot().to_text(), ed.snapshot().to_text());
    }

    #[test]
    #[allow(non_snake_case)] // `regression_<ID>_<slug>` (Workflow)
    fn regression_T74_text_size_beyond_the_editor_is_rejected() {
        // Arrange: a size no zoom and width can make; typing more would
        // grow the block's bounds past f32.
        let mut ed = editor();
        ed.apply(Command::SetTool(Tool::Text));
        down(&mut ed, v(0.0, 100.0));
        up(&mut ed, v(0.0, 100.0));
        type_text(&mut ed, "ab");
        key(&mut ed, Key::Escape);
        let text = ed.snapshot().to_text();
        let huge = text.replace(" 18.0 \"ab\"", " 1e30 \"ab\"");
        assert_ne!(huge, text, "size token not found");

        // Act
        let result = Snapshot::parse(&huge);

        // Assert
        assert!(result.is_err(), "size 1e30 accepted");
    }

    // AC-2

    #[test]
    fn fixtures_are_canonical() {
        // Arrange
        let paths = fixtures();
        assert!(!paths.is_empty(), "no fixtures in tests/snapshots");

        for path in paths {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("{}: {err}", path.display()));

            // Act
            let snapshot =
                Snapshot::parse(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));

            // Assert
            assert_eq!(
                snapshot.to_text(),
                text,
                "{} is not canonical",
                path.display()
            );
        }
    }

    #[test]
    fn text_is_canonical_for_busy_editor() {
        // Arrange
        let text = busy_editor().snapshot().to_text();

        // Act
        let again = Snapshot::parse(&text).map(|s| s.to_text());

        // Assert
        assert_eq!(again.as_deref(), Ok(text.as_str()));
        assert!(text.starts_with(&format!("{HEADER} {VERSION}\n")));
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn strings_round_trip_with_escapes() {
        // Arrange
        let mut ed = editor();
        ed.apply(Command::SetTool(Tool::Text));
        down(&mut ed, v(100.0, 100.0));
        up(&mut ed, v(100.0, 100.0));
        type_text(&mut ed, "q\"uo\\te\ttab\nline é ∑ \u{7f} end ");

        // Act
        let text = ed.snapshot().to_text();
        let loaded = round_trip(&ed);

        // Assert: one record per line, and the text comes back.
        assert_eq!(loaded, ed);
        assert_eq!(
            loaded.text_edit().map(|e| e.text().to_owned()),
            ed.text_edit().map(|e| e.text().to_owned())
        );
        assert!(!text.contains('\t'), "tabs are escaped");
        assert!(!text.contains('\u{7f}'), "control characters are escaped");
    }

    #[test]
    fn floats_keep_their_bits() {
        // Arrange: awkward zoom, offsets and widths.
        let mut ed = editor();
        for delta in [0.37, -1.3, 2.9] {
            ed.handle(InputEvent::Scroll {
                pos: v(123.456, 78.9),
                delta,
            });
        }
        drag(&mut ed, v(0.1, 0.2), v(333.333, 1e-3));

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(
            loaded.camera().zoom().to_bits(),
            ed.camera().zoom().to_bits()
        );
        assert_eq!(
            loaded.camera().offset().x.to_bits(),
            ed.camera().offset().x.to_bits()
        );
        assert_eq!(loaded, ed);
    }

    // AC-3

    #[test]
    fn parse_error_names_line() {
        // Arrange
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "camera ");

        // Act
        let err = Snapshot::parse(&edited(&ed, line, "camera 1.0 oops 1.0"));

        // Assert
        let err = err.err().unwrap_or_else(|| panic!("bad camera accepted"));
        assert_eq!(err.line, line + 1);
        assert!(
            err.to_string().starts_with(&format!("line {}: ", line + 1)),
            "{err}"
        );
    }

    #[test]
    fn parse_rejects_non_finite_float() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "camera ");

        for bad in [
            "camera NaN 0.0 1.0",
            "camera 0.0 inf 1.0",
            "camera 0.0 0.0 -inf",
        ] {
            // Act
            let result = Snapshot::parse(&edited(&ed, line, bad));

            // Assert
            assert_eq!(result.err().map(|e| e.line), Some(line + 1), "{bad}");
        }
    }

    /// A zero-based editor holding the numbered node 0.
    fn zero_based_node() -> Editor {
        let mut ed = editor();
        ed.apply(Command::ToggleNumbering);
        ed.apply(Command::ToggleZeroBased);
        ed.apply(Command::SetTool(Tool::Ellipse));
        drag(&mut ed, v(10.0, 10.0), v(60.0, 60.0));
        ed
    }

    #[test]
    fn round_trip_keeps_zero_based() {
        // Arrange
        let ed = zero_based_node();

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded, ed);
        assert!(loaded.helpers().zero_based);
        assert_eq!(loaded.helpers().next_number, 1);
        assert_eq!(loaded.snapshot().to_text(), ed.snapshot().to_text());
    }

    #[test]
    fn parse_accepts_label_zero() {
        // Arrange: back to one-based, node 0 stays in the document.
        let mut ed = zero_based_node();
        ed.apply(Command::ToggleZeroBased);

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded, ed);
        let labels: Vec<_> = loaded.document().shapes().map(|(_, s)| s.label()).collect();
        assert_eq!(labels, vec![Some(0)]);
    }

    #[test]
    fn parse_checks_the_counter_against_the_first_number() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "helpers ");

        // Act
        let one_based = Snapshot::parse(&edited(&ed, line, "helpers 0 0 0 0 4 4 0"));
        let zero_based = Snapshot::parse(&edited(&ed, line, "helpers 0 0 0 0 4 4 1"));

        // Assert
        assert_eq!(one_based.err().map(|e| e.line), Some(line + 1));
        assert!(zero_based.is_ok());
    }

    #[test]
    fn parse_rejects_zoom_out_of_range() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "camera ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "camera 0.0 0.0 1000000.0"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn parse_rejects_duplicate_id() {
        // Arrange
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let first = line_of(&text, "shape ");
        let mut lines: Vec<&str> = text.lines().collect();
        let copy = lines[first];
        lines.insert(first + 1, copy);
        let text = lines.join("\n") + "\n";

        // Act
        let result = Snapshot::parse(&text);

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(first + 2));
    }

    #[test]
    fn parse_rejects_id_at_or_above_next_id() {
        // Arrange
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "next-id ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "next-id 1"));

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_selection_outside_document() {
        // Arrange
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "selection");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "selection 999"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn parse_rejects_unknown_color() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "style ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "style 200 3.0"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn parse_rejects_caret_off_char_boundary() {
        // Arrange: an open block holding "é", caret after it (2 bytes).
        let mut ed = editor();
        ed.apply(Command::SetTool(Tool::Text));
        down(&mut ed, v(100.0, 100.0));
        up(&mut ed, v(100.0, 100.0));
        type_text(&mut ed, "é");
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "text ");
        let bad = text
            .lines()
            .nth(line)
            .map(|l| l.replace("\"é\" 2 ", "\"é\" 1 "));
        let bad = bad.unwrap_or_default();
        assert_ne!(
            bad,
            text.lines().nth(line).unwrap_or_default(),
            "caret token not found"
        );

        // Act
        let result = Snapshot::parse(&edited(&ed, line, &bad));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn parse_rejects_truncated_text() {
        // Arrange
        let text = busy_editor().snapshot().to_text();

        for end in (0..text.len())
            .filter(|&i| text.is_char_boundary(i))
            .step_by(7)
        {
            // Act
            let result = Snapshot::parse(&text[..end]);

            // Assert: a prefix that drops whole records is still a valid
            // snapshot only if every section is there; never a panic.
            if let Err(err) = result {
                assert!(err.line >= 1, "{err}");
            }
        }
    }

    #[test]
    fn parse_rejects_empty_and_garbage() {
        for text in ["", "\n", "draw-snapshot", "hello\nworld\n", "\u{0}\u{0}"] {
            // Act
            let result = Snapshot::parse(text);

            // Assert
            assert_eq!(result.err().map(|e| e.line), Some(1), "{text:?}");
        }
    }

    // AC-4

    #[test]
    fn rejects_unknown_version() {
        // Arrange
        let ed = editor();
        let next = format!("{HEADER} {}", VERSION + 1);

        // Act
        let result = Snapshot::parse(&edited(&ed, 0, &next));

        // Assert
        let err = result
            .err()
            .unwrap_or_else(|| panic!("version {} accepted", VERSION + 1));
        assert_eq!(err.line, 1);
        assert!(err.message.contains("version"), "{err}");
    }

    #[test]
    fn rejects_unknown_section() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "[clipboard]");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "[pasteboard]"));

        // Assert
        let err = result
            .err()
            .unwrap_or_else(|| panic!("unknown section accepted"));
        assert_eq!(err.line, line + 1);
        assert!(err.message.contains("section"), "{err}");
    }

    #[test]
    fn rejects_unknown_key() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "toolbar ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "sidebar 1"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn rejects_trailing_tokens() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "toolbar ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "toolbar 1 1"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    // T107 locks

    /// [`busy_editor`] with every shape locked, a `lock` step in the undo
    /// history and an `unlock` step in the redo history.
    fn locked_editor() -> Editor {
        let mut ed = busy_editor();
        ed.apply(Command::SelectAll);
        assert!(ed.apply(Command::LockSelection));
        assert!(ed.apply(Command::UnlockAll));
        assert!(ed.apply(Command::Undo));
        ed
    }

    #[test]
    fn version_is_three() {
        assert_eq!(VERSION, 3);
    }

    /// [`busy_editor`] with a rectangle on page 2 and another on page 5,
    /// left on page 3 with its own view.
    fn paged_editor() -> Editor {
        let mut ed = busy_editor();
        ed.apply(Command::SetTool(Tool::Rect));
        ed.apply(Command::GoToPage(1));
        drag(&mut ed, v(30.0, 30.0), v(90.0, 80.0));
        ed.apply(Command::GoToPage(4));
        drag(&mut ed, v(40.0, 40.0), v(120.0, 90.0));
        ed.apply(Command::Undo);
        drag(&mut ed, v(50.0, 50.0), v(130.0, 95.0));
        ed.apply(Command::GoToPage(2));
        ed
    }

    #[test]
    fn round_trip_keeps_every_page() {
        // Arrange
        let ed = paged_editor();

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded.page(), 2);
        assert_eq!(loaded, ed);
        let again = loaded.snapshot().to_text();
        assert_eq!(again, ed.snapshot().to_text());
    }

    #[test]
    fn recovery_restores_each_pages_history() {
        // Arrange
        let mut loaded = round_trip(&paged_editor());

        // Act
        loaded.apply(Command::GoToPage(4));
        let shapes = loaded.document().len();
        loaded.apply(Command::Undo);
        loaded.apply(Command::Redo);
        loaded.apply(Command::GoToPage(0));

        // Assert
        assert_eq!(shapes, 1);
        assert!(loaded.can_undo());
        assert_eq!(loaded.document().len(), busy_editor().document().len());
    }

    #[test]
    fn version_two_still_loads() {
        // Arrange: a version 2 text has no `[pages]` section.
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let cut = line_of(&text, "[pages]");
        let old: String = text
            .lines()
            .take(cut)
            .enumerate()
            .map(|(i, l)| {
                if i == 0 {
                    "draw-snapshot 2\n".to_owned()
                } else {
                    format!("{l}\n")
                }
            })
            .collect();

        // Act
        let loaded = Snapshot::parse(&old).map(Editor::from_snapshot);

        // Assert
        assert_eq!(loaded.as_ref().ok().map(Editor::page), Some(0));
        assert_eq!(loaded.ok(), Some(ed));
    }

    #[test]
    fn rejects_bad_page_index() {
        // Arrange: the stored page claims to be the current one.
        let ed = paged_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "index ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "index 2"));

        // Assert
        let err = result.err().unwrap_or_else(|| panic!("accepted"));
        assert_eq!(err.line, line + 1);
        assert!(err.message.contains("page"), "{err}");
    }

    #[test]
    fn rejects_a_current_page_out_of_range() {
        // Arrange
        let ed = paged_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "current ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "current 10"));

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn snapshot_round_trip_keeps_locks_and_lock_edits() {
        // Arrange
        let mut ed = locked_editor();
        let text = ed.snapshot().to_text();

        // Act
        let mut loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded, ed);
        assert!(text.lines().any(|l| l.starts_with("locked ")), "{text}");
        assert!(text.lines().any(|l| l.starts_with("lock ")), "{text}");
        assert!(text.lines().any(|l| l.starts_with("unlock ")), "{text}");
        for command in [Command::Redo, Command::Undo, Command::Undo, Command::Redo] {
            ed.apply(command);
            loaded.apply(command);
            assert_eq!(loaded, ed, "after {command:?}");
        }
    }

    #[test]
    fn snapshot_writes_locked_record_only_when_needed() {
        // Arrange
        let ed = busy_editor();

        // Act
        let text = ed.snapshot().to_text();

        // Assert
        assert!(!text.lines().any(|l| l.starts_with("locked")), "{text}");
    }

    #[test]
    fn snapshot_rejects_unknown_locked_id() {
        // Arrange
        let ed = locked_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "locked ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "locked 999"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn snapshot_rejects_repeated_locked_id() {
        // Arrange
        let ed = locked_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "locked ");
        let id = ed
            .document()
            .shapes()
            .map(|(id, _)| id.get())
            .next()
            .unwrap_or_default();

        // Act
        let result = Snapshot::parse(&edited(&ed, line, &format!("locked {id} {id}")));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn snapshot_rejects_locked_selection() {
        // Arrange
        let ed = locked_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "selection");
        let id = ed
            .document()
            .shapes()
            .map(|(id, _)| id.get())
            .next()
            .unwrap_or_default();

        // Act
        let result = Snapshot::parse(&edited(&ed, line, &format!("selection {id}")));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }
}

#[cfg(test)]
mod golden_tests {
    use super::golden::{Golden, check, diff};
    use std::fmt::Write as _;

    // AC-5

    #[test]
    fn check_matches_equal_text() {
        // Arrange / Act / Assert
        assert_eq!(check("a\nb\n", Some("a\nb\n"), false), Golden::Match);
        assert_eq!(check("a\nb\n", Some("a\nb\n"), true), Golden::Match);
    }

    #[test]
    fn check_mismatch_has_line_diff() {
        // Arrange
        let expected = "head\nkeep\nold one\nold two\ntail\n";
        let actual = "head\nkeep\nnew one\ntail\n";

        // Act
        let result = check(actual, Some(expected), false);

        // Assert
        let Golden::Mismatch(text) = result else {
            panic!("expected a mismatch, got {result:?}");
        };
        assert_eq!(text, diff(expected, actual));
        assert!(text.contains("line 3"), "{text}");
        assert!(text.contains("- old one\n- old two\n+ new one\n"), "{text}");
        assert!(!text.contains("keep") && !text.contains("tail"), "{text}");
    }

    #[test]
    fn check_writes_only_when_updating() {
        // Arrange / Act / Assert
        assert_eq!(check("new\n", Some("old\n"), true), Golden::Write);
        assert_eq!(check("new\n", None, true), Golden::Write);
        assert!(matches!(
            check("new\n", Some("old\n"), false),
            Golden::Mismatch(_)
        ));
    }

    #[test]
    fn check_missing_without_update() {
        // Arrange / Act / Assert
        assert_eq!(check("new\n", None, false), Golden::Missing);
    }

    #[test]
    fn diff_caps_long_changes() {
        // Arrange
        let (mut expected, mut actual) = (String::new(), String::new());
        for i in 0..1000 {
            let _ = writeln!(expected, "a{i}");
            let _ = writeln!(actual, "b{i}");
        }

        // Act
        let text = diff(&expected, &actual);

        // Assert: readable, not the whole file.
        assert!(text.lines().count() < 100, "{} lines", text.lines().count());
        assert!(text.contains("more"), "{text}");
    }
}
