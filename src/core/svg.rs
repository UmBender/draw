//! The drawing as an SVG file (ADR-T114-2).
//!
//! [`to_svg`] is pure: it takes the shapes bottom to top and a source of
//! glyph outlines, and returns the text of the file. The world is the SVG's
//! user space (one world unit is one user unit), text and formulas go
//! through the same layout the renderer uses and are written as outlines,
//! each glyph defined once and reused.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use crate::core::geom::{Aabb, Vec2};
use crate::core::palette::{ColorId, Rgba, palette};
use crate::core::shape::{
    GRID_MAX_CELLS, Shape, TextSlot, arrow_head, grid_axis_labels, grid_cell_rect, grid_lines,
};
use crate::core::tex::{Face, Ink};
use crate::core::text_layout;
use crate::theme;

/// The outline of one character, as an SVG path.
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    /// Path data in font units, y down, so ink above the baseline has a
    /// negative y.
    pub path: String,
    /// Font units per em.
    pub units_per_em: f32,
    /// Advance width in font units.
    pub advance: f32,
    /// Height of the ink above the baseline in font units.
    pub ink_top: f32,
}

/// Where the glyph outlines come from: the embedded fonts, in the shell.
pub trait Outlines {
    /// The outline of `ch` in `face`; `None` for a character without ink
    /// (a space) or one the font lacks.
    fn outline(&self, face: Face, ch: char) -> Option<Outline>;
}

/// `shapes`, bottom to top, as an SVG document on a `background` canvas.
/// Non-finite shapes are skipped, so every number in it is finite.
#[must_use]
pub fn to_svg<'a>(
    shapes: impl IntoIterator<Item = &'a Shape>,
    background: Rgba,
    outlines: &dyn Outlines,
) -> String {
    let mut writer = Writer::new(outlines);
    let mut bounds: Option<Aabb> = None;
    for shape in shapes {
        if !shape.is_finite() {
            continue;
        }
        let b = shape.bounds();
        bounds = Some(bounds.map_or(b, |all| all.union(&b)));
        writer.shape(shape);
    }
    let view = bounds.map_or(Aabb::from_corners(Vec2::ZERO, Vec2::new(64.0, 64.0)), |b| {
        b.expand(MARGIN)
    });
    let (x, y, w, h) = (view.min.x, view.min.y, view.width(), view.height());
    let mut out = String::new();
    let _ = out.write_fmt(format_args!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" \
         width=\"{w}\" height=\"{h}\" viewBox=\"{x} {y} {w} {h}\">\n",
        w = number(w),
        h = number(h),
        x = number(x),
        y = number(y),
    ));
    if !writer.defs.is_empty() {
        out.push_str("<defs>\n");
        for def in &writer.defs {
            out.push_str(def);
            out.push('\n');
        }
        out.push_str("</defs>\n");
    }
    let _ = out.write_fmt(format_args!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
        number(x),
        number(y),
        number(w),
        number(h),
        hex(background),
    ));
    out.push_str(&writer.body);
    out.push_str("</svg>\n");
    out
}

/// World units of canvas around the shapes.
const MARGIN: f32 = 24.0;

/// Decimals of a glyph scale, which is far smaller than a world unit.
const SCALE_DECIMALS: i32 = 5;

/// A label's height as a fraction of its area's height (the renderer's
/// `LABEL_HEIGHT_RATIO`).
const LABEL_HEIGHT_RATIO: f32 = 0.6;

/// A label's width as a fraction of its area's width (the renderer's
/// `LABEL_WIDTH_RATIO`).
const LABEL_WIDTH_RATIO: f32 = 0.8;

/// Width of one label character per unit of size (the renderer's
/// `LABEL_CHAR_ASPECT`).
const LABEL_CHAR_ASPECT: f32 = 0.6;

/// `#rrggbb` of `color`.
fn hex(color: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

/// The colour of a shape's own ink.
fn ink(id: ColorId) -> Rgba {
    palette(id)
}

/// The colour of text on a shape with this `fill`: the outline colour, or
/// the dark text colour on a filled shape (the renderer's `label_color`).
fn text_color(fill: Option<ColorId>, outline: Rgba) -> Rgba {
    if fill.is_some() {
        Rgba::from_hex(theme::TEXT_ON_FILL)
    } else {
        outline
    }
}

/// The colour of formula source outside the subset (the renderer's
/// `warn_color`).
fn warn_color(color: Rgba) -> Rgba {
    let source = Rgba::from_hex(theme::FORMULA_SOURCE);
    if color.r == source.r && color.g == source.g && color.b == source.b {
        Rgba::from_hex(theme::FORMULA_SOURCE_ALT)
    } else {
        source
    }
}

/// Builds the body of the document and the glyph definitions it uses.
struct Writer<'o> {
    /// Where the glyph outlines come from.
    outlines: &'o dyn Outlines,
    /// The `<path>` of each defined glyph, in definition order.
    defs: Vec<String>,
    /// The glyphs seen so far: their definition, or `None` without ink.
    glyphs: HashMap<(Face, char), Option<(usize, Outline)>>,
    /// The shapes written so far.
    body: String,
}

impl<'o> Writer<'o> {
    /// Appends formatted text to the body; writing to a `String` cannot
    /// fail.
    fn emit(&mut self, args: std::fmt::Arguments<'_>) {
        let _ = self.body.write_fmt(args);
    }

    fn new(outlines: &'o dyn Outlines) -> Self {
        Self {
            outlines,
            defs: Vec::new(),
            glyphs: HashMap::new(),
            body: String::new(),
        }
    }

    /// The definition number and outline of `ch`, defining it on first use.
    fn glyph(&mut self, face: Face, ch: char) -> Option<(usize, Outline)> {
        if let Some(known) = self.glyphs.get(&(face, ch)) {
            return known.clone();
        }
        let found = self.outlines.outline(face, ch).and_then(|outline| {
            let valid = outline.units_per_em.is_finite()
                && outline.units_per_em > 0.0
                && outline.advance.is_finite()
                && outline.ink_top.is_finite();
            valid.then(|| {
                let id = self.defs.len();
                self.defs
                    .push(format!("<path id=\"g{id}\" d=\"{}\"/>", outline.path));
                (id, outline)
            })
        });
        self.glyphs.insert((face, ch), found.clone());
        found
    }

    /// One glyph with the left end of its baseline at `pos`, `size` world
    /// units per em. Returns its advance in world units.
    fn put_glyph(&mut self, face: Face, ch: char, pos: Vec2, size: f32, color: Rgba) -> f32 {
        let Some((id, outline)) = self.glyph(face, ch) else {
            return 0.0;
        };
        let scale = size / outline.units_per_em;
        self.emit(format_args!(
            "<use xlink:href=\"#g{id}\" transform=\"translate({} {}) scale({})\" fill=\"{}\"/>\n",
            number(pos.x),
            number(pos.y),
            fixed(scale, SCALE_DECIMALS),
            hex(color),
        ));
        outline.advance * scale
    }

    /// A text block at `origin`, laid out as the renderer lays it out.
    fn text_block(&mut self, origin: Vec2, text: &str, size: f32, color: Rgba) {
        if !(size.is_finite() && size > 0.0) || text.trim().is_empty() {
            return;
        }
        let layout = text_layout::typeset(text);
        let warn = warn_color(color);
        for item in &layout.ink {
            match *item {
                Ink::Glyph {
                    ch,
                    face,
                    pos,
                    scale,
                    warn: warned,
                } => {
                    let at = origin + pos * size;
                    let glyph_color = if warned { warn } else { color };
                    self.put_glyph(face, ch, at, size * scale, glyph_color);
                }
                Ink::Rule { a, b, width } => {
                    self.line(
                        origin + a * size,
                        origin + b * size,
                        width * size,
                        color,
                        "butt",
                    );
                }
            }
        }
    }

    /// A text of the text face centred in `rect`: its advance box
    /// horizontally and its ink vertically.
    fn centered(&mut self, text: &str, rect: Aabb, size: f32, color: Rgba) {
        let faces: Vec<Option<(usize, Outline)>> =
            text.chars().map(|ch| self.glyph(Face::Text, ch)).collect();
        let advance = |entry: &Outline| entry.advance / entry.units_per_em * size;
        let width: f32 = faces.iter().flatten().map(|(_, o)| advance(o)).sum();
        let top = faces
            .iter()
            .flatten()
            .map(|(_, o)| o.ink_top / o.units_per_em * size)
            .fold(0.0, f32::max);
        let c = rect.center();
        let mut pen = Vec2::new(c.x - width * 0.5, c.y + top * 0.5);
        for ch in text.chars() {
            pen.x += self.put_glyph(Face::Text, ch, pen, size, color);
        }
    }

    /// The label or axis index `text` of `size` in `rect`.
    fn label(&mut self, text: &str, rect: Aabb, ellipse: bool, color: Rgba) {
        let mut area = Vec2::new(rect.width(), rect.height());
        if ellipse {
            area = area * std::f32::consts::FRAC_1_SQRT_2;
        }
        if let Some(size) = label_size(text.len(), area) {
            self.centered(text, rect, size, color);
        }
    }

    fn line(&mut self, a: Vec2, b: Vec2, width: f32, color: Rgba, cap: &str) {
        self.emit(format_args!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" \
             stroke-linecap=\"{cap}\"/>\n",
            number(a.x),
            number(a.y),
            number(b.x),
            number(b.y),
            hex(color),
            number(width),
        ));
    }

    fn shape(&mut self, shape: &Shape) {
        let style = shape.style();
        let width = style.width;
        let color = ink(style.color);
        match shape {
            Shape::Stroke { points, widths, .. } => {
                self.stroke(points, widths.as_deref(), width, color);
            }
            Shape::Line { a, b, .. } => self.line(*a, *b, width, color, "round"),
            Shape::Arrow { a, b, .. } => {
                let [tip, left, right] = arrow_head(*a, *b, width);
                self.line(*a, left.lerp(right, 0.5), width, color, "round");
                self.emit(format_args!(
                    "<polygon points=\"{},{} {},{} {},{}\" fill=\"{}\"/>\n",
                    number(tip.x),
                    number(tip.y),
                    number(left.x),
                    number(left.y),
                    number(right.x),
                    number(right.y),
                    hex(color),
                ));
            }
            Shape::Rect { a, b, fill, .. } => {
                let rect = Aabb::from_corners(*a, *b);
                self.emit(format_args!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" \
                     stroke=\"{}\" stroke-width=\"{}\"/>\n",
                    number(rect.min.x),
                    number(rect.min.y),
                    number(rect.width()),
                    number(rect.height()),
                    fill_attr(*fill),
                    hex(color),
                    number(width),
                ));
                let on = text_color(*fill, color);
                self.shown_label(shape, rect, false, on);
                self.shape_texts(shape, |_| on);
            }
            Shape::Ellipse { a, b, fill, .. } => {
                let rect = Aabb::from_corners(*a, *b);
                let c = rect.center();
                self.emit(format_args!(
                    "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{}\" \
                     stroke=\"{}\" stroke-width=\"{}\"/>\n",
                    number(c.x),
                    number(c.y),
                    number(rect.width() * 0.5),
                    number(rect.height() * 0.5),
                    fill_attr(*fill),
                    hex(color),
                    number(width),
                ));
                let on = text_color(*fill, color);
                self.shown_label(shape, rect, true, on);
                self.shape_texts(shape, |_| on);
            }
            Shape::Grid {
                a,
                b,
                cols,
                rows,
                axes,
                fills,
                ..
            } => {
                // Cell fills first, so the lines stay on top.
                for fill in fills {
                    let cell = grid_cell_rect(*a, *b, *cols, *rows, fill.col, fill.row);
                    self.emit(format_args!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
                        number(cell.min.x),
                        number(cell.min.y),
                        number(cell.width()),
                        number(cell.height()),
                        hex(ink(fill.color)),
                    ));
                }
                for [start, end] in grid_lines(*a, *b, *cols, *rows) {
                    self.line(start, end, width, color, "square");
                }
                if *axes {
                    self.axis_labels(*a, *b, *cols, *rows, color);
                }
                self.shape_texts(shape, |p| {
                    let filled = shape
                        .grid_cell_at(p)
                        .and_then(|(col, row)| shape.cell_fill(col, row));
                    text_color(filled, color)
                });
            }
            Shape::Text {
                origin, text, size, ..
            } => self.text_block(*origin, text, *size, color),
        }
    }

    /// A stroke: a round polyline, or one segment per pair of points when
    /// the pen had pressure.
    fn stroke(&mut self, points: &[Vec2], widths: Option<&[f32]>, width: f32, color: Rgba) {
        let factor = |i: usize| widths.and_then(|w| w.get(i)).copied().unwrap_or(1.0);
        match points {
            [] => {}
            [p] => self.emit(format_args!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>\n",
                number(p.x),
                number(p.y),
                number(width * factor(0) * 0.5),
                hex(color),
            )),
            _ if widths.is_some() => {
                for (i, pair) in points.windows(2).enumerate() {
                    let mean = width * (factor(i) + factor(i + 1)) * 0.5;
                    self.line(pair[0], pair[1], mean, color, "round");
                }
            }
            _ => {
                let list: Vec<String> = points
                    .iter()
                    .map(|p| format!("{},{}", number(p.x), number(p.y)))
                    .collect();
                self.emit(format_args!(
                    "<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" \
                     stroke-linecap=\"round\" stroke-linejoin=\"round\"/>\n",
                    list.join(" "),
                    hex(color),
                    number(width),
                ));
            }
        }
    }

    /// The label of `shape` unless it has text, which is drawn instead.
    fn shown_label(&mut self, shape: &Shape, rect: Aabb, ellipse: bool, color: Rgba) {
        if shape.slot_text(TextSlot::Shape).is_some() {
            return;
        }
        if let Some(label) = shape.label() {
            self.label(&label.to_string(), rect, ellipse, color);
        }
    }

    /// Every text of a rectangle, ellipse or grid, in the colour
    /// `color_at` gives for the world centre of its block.
    fn shape_texts(&mut self, shape: &Shape, color_at: impl Fn(Vec2) -> Rgba) {
        for placed in shape.placed_texts() {
            let center = text_layout::bounds(placed.origin, placed.text, placed.size).center();
            self.text_block(placed.origin, placed.text, placed.size, color_at(center));
        }
    }

    /// The axis indices of a grid, all at one size.
    fn axis_labels(&mut self, a: Vec2, b: Vec2, cols: u32, rows: u32, color: Rgba) {
        let Some(first) = grid_axis_labels(a, b, cols, rows).next() else {
            return;
        };
        let longest = cols.max(rows).clamp(1, GRID_MAX_CELLS) - 1;
        let cell = Vec2::new(first.rect.width(), first.rect.height());
        let Some(size) = label_size(longest.to_string().len(), cell) else {
            return;
        };
        for label in grid_axis_labels(a, b, cols, rows) {
            self.centered(&label.index.to_string(), label.rect, size, color);
        }
    }
}

/// The font size of a label of `chars` characters in `area`: as tall as
/// [`LABEL_HEIGHT_RATIO`] of its height, narrowed to fit
/// [`LABEL_WIDTH_RATIO`] of its width. `None` unless it is finite and
/// positive.
fn label_size(chars: usize, area: Vec2) -> Option<f32> {
    if !area.is_finite() {
        return None;
    }
    let text_width = chars.max(1) as f32 * LABEL_CHAR_ASPECT;
    let size = (area.y * LABEL_HEIGHT_RATIO).min(area.x * LABEL_WIDTH_RATIO / text_width);
    (size.is_finite() && size > 0.0).then_some(size)
}

/// The `fill` attribute value of a shape.
fn fill_attr(fill: Option<ColorId>) -> String {
    fill.map_or_else(|| "none".to_owned(), |id| hex(ink(id)))
}

/// `value` rounded to `decimals` places, without trailing zeros; `0` when
/// not finite.
fn fixed(value: f32, decimals: i32) -> String {
    let value = f64::from(value);
    if !value.is_finite() {
        return "0".to_owned();
    }
    let factor = 10f64.powi(decimals);
    let rounded = (value * factor).round() / factor;
    if rounded == 0.0 || !rounded.is_finite() {
        "0".to_owned()
    } else {
        format!("{rounded}")
    }
}

/// `value` with at most 2 decimals and no trailing zeros; 0 when it is not
/// finite or rounds to zero.
#[must_use]
pub fn number(value: f32) -> String {
    fixed(value, 2)
}

/// Whether `xml` is one well-formed element tree: balanced and matching
/// tags, quoted and unique attributes. A small check for tests; it knows
/// no entities or comments beyond `<?…?>` declarations.
#[must_use]
pub fn is_well_formed(xml: &str) -> bool {
    let mut cursor = Cursor {
        rest: xml,
        stack: Vec::new(),
        roots: 0,
    };
    cursor.run()
}

/// A position in the markup being checked.
struct Cursor<'a> {
    /// What is left to read.
    rest: &'a str,
    /// The open elements.
    stack: Vec<&'a str>,
    /// The elements that were opened at the top level.
    roots: usize,
}

/// Whether `c` may be in a tag or attribute name.
fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '-' | '.')
}

impl Cursor<'_> {
    /// Reads the whole text; true if it is one well-formed element.
    fn run(&mut self) -> bool {
        loop {
            let Some(at) = self.rest.find('<') else {
                // Only blanks may follow the last tag, and the tree is whole.
                return self.rest.trim().is_empty() && self.stack.is_empty() && self.roots == 1;
            };
            let (text, tag) = self.rest.split_at(at);
            if self.stack.is_empty() && !text.trim().is_empty() {
                return false;
            }
            if text.contains('>') {
                return false;
            }
            self.rest = tag;
            if !self.tag() {
                return false;
            }
        }
    }

    /// Reads one tag at the start of the rest.
    fn tag(&mut self) -> bool {
        if let Some(after) = self.rest.strip_prefix("<?") {
            let Some(end) = after.find("?>") else {
                return false;
            };
            self.rest = &after[end + 2..];
            return true;
        }
        if let Some(after) = self.rest.strip_prefix("</") {
            let Some(end) = after.find('>') else {
                return false;
            };
            let name = after[..end].trim_end();
            self.rest = &after[end + 1..];
            return self.stack.pop() == Some(name) && !name.is_empty();
        }
        self.rest = &self.rest[1..];
        let name_len = self
            .rest
            .find(|c| !is_name_char(c))
            .unwrap_or(self.rest.len());
        let (name, after) = self.rest.split_at(name_len);
        if name.is_empty() {
            return false;
        }
        self.rest = after;
        let Some(self_closing) = self.attributes() else {
            return false;
        };
        if self.stack.is_empty() {
            self.roots += 1;
            if self.roots > 1 {
                return false;
            }
        }
        if !self_closing {
            self.stack.push(name);
        }
        true
    }

    /// Reads the attributes up to the end of the tag. `Some(true)` for
    /// `/>`, `Some(false)` for `>`, `None` when they are malformed.
    fn attributes(&mut self) -> Option<bool> {
        let mut seen = HashSet::new();
        loop {
            let trimmed = self.rest.trim_start();
            let blank = trimmed.len() != self.rest.len();
            self.rest = trimmed;
            if let Some(after) = self.rest.strip_prefix("/>") {
                self.rest = after;
                return Some(true);
            }
            if let Some(after) = self.rest.strip_prefix('>') {
                self.rest = after;
                return Some(false);
            }
            // An attribute needs a blank before it.
            if !blank {
                return None;
            }
            let name_len = self.rest.find(|c| !is_name_char(c))?;
            let (name, after) = self.rest.split_at(name_len);
            let after = after.strip_prefix('=')?;
            let quote = after.chars().next().filter(|q| matches!(q, '"' | '\''))?;
            let value = &after[1..];
            let end = value.find(quote)?;
            if name.is_empty() || value[..end].contains('<') || !seen.insert(name) {
                return None;
            }
            self.rest = &value[end + 1..];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::palette::ColorId;
    use crate::core::shape::{CellFill, ShapeText, Style};
    use crate::theme;

    /// A square glyph, 1000 units per em.
    struct Squares;

    impl Outlines for Squares {
        fn outline(&self, _face: Face, ch: char) -> Option<Outline> {
            (!ch.is_whitespace()).then(|| Outline {
                path: "M0 0L500 0L500 -700L0 -700Z".to_owned(),
                units_per_em: 1000.0,
                advance: 600.0,
                ink_top: 700.0,
            })
        }
    }

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn style(width: f32) -> Style {
        Style {
            color: ColorId::INK,
            width,
        }
    }

    fn render(shapes: &[Shape]) -> String {
        to_svg(shapes, Rgba::from_hex(theme::CANVAS), &Squares)
    }

    fn count(svg: &str, pattern: &str) -> usize {
        svg.matches(pattern).count()
    }

    fn text(origin: Vec2, content: &str) -> Shape {
        Shape::Text {
            origin,
            text: content.to_owned(),
            size: 20.0,
            style: style(0.0),
        }
    }

    fn rect(label: Option<u32>, fill: Option<ColorId>) -> Shape {
        Shape::Rect {
            a: v(0.0, 0.0),
            b: v(100.0, 60.0),
            style: style(2.0),
            fill,
            label,
            text: None,
        }
    }

    // ---- AC-4: the file ------------------------------------------------

    #[test]
    fn empty_document_is_valid_svg() {
        let svg = render(&[]);

        assert!(svg.starts_with("<svg"), "{svg}");
        assert!(svg.trim_end().ends_with("</svg>"), "{svg}");
        assert!(is_well_formed(&svg), "{svg}");
        assert!(!svg.contains("NaN") && !svg.contains("inf"), "{svg}");
    }

    #[test]
    fn to_svg_has_canvas_background_and_viewbox_of_all_shapes() {
        let shapes = [
            Shape::Line {
                a: v(0.0, 0.0),
                b: v(100.0, 50.0),
                style: style(2.0),
            },
            Shape::Line {
                a: v(-200.0, 10.0),
                b: v(10.0, 400.0),
                style: style(2.0),
            },
        ];

        let svg = render(&shapes);

        assert!(svg.contains("#181616"), "background colour: {svg}");
        let view = svg
            .split("viewBox=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_default();
        let n: Vec<f32> = view.split(' ').filter_map(|s| s.parse().ok()).collect();
        assert_eq!(n.len(), 4, "viewBox {view}");
        assert!(n[0] <= -200.0 && n[1] <= 0.0, "origin {view}");
        assert!(
            n[0] + n[2] >= 100.0 && n[1] + n[3] >= 400.0,
            "extent {view}"
        );
    }

    #[test]
    fn non_finite_shapes_are_skipped() {
        let shapes = [
            Shape::Line {
                a: v(f32::NAN, 0.0),
                b: v(1.0, f32::INFINITY),
                style: style(2.0),
            },
            Shape::Line {
                a: v(0.0, 0.0),
                b: v(10.0, 10.0),
                style: style(2.0),
            },
        ];

        let svg = render(&shapes);

        assert_eq!(count(&svg, "<line"), 1, "{svg}");
        assert!(!svg.contains("NaN") && !svg.contains("inf"), "{svg}");
        assert!(is_well_formed(&svg));
    }

    #[test]
    fn numbers_have_two_decimals_and_no_trailing_zeros() {
        assert_eq!(number(1.0), "1");
        assert_eq!(number(0.5), "0.5");
        assert_eq!(number(1.234), "1.23");
        assert_eq!(number(-2.5), "-2.5");
        assert_eq!(number(-0.001), "0");
        assert_eq!(number(f32::NAN), "0");
    }

    #[test]
    fn is_well_formed_accepts_and_rejects() {
        assert!(is_well_formed("<a b=\"1\"><c/></a>"));
        assert!(is_well_formed("<?xml version=\"1.0\"?>\n<a>\n</a>\n"));
        assert!(!is_well_formed("<a><b></a></b>"));
        assert!(!is_well_formed("<a>"));
        assert!(!is_well_formed("<a b=1/>"));
        assert!(!is_well_formed("<a/><b/>"));
        assert!(!is_well_formed("<a x=\"1\" x=\"2\"/>"));
        assert!(!is_well_formed(""));
    }

    // ---- AC-2: shapes --------------------------------------------------

    #[test]
    fn line_has_colour_width_and_round_caps() {
        let svg = render(&[Shape::Line {
            a: v(0.0, 0.0),
            b: v(100.0, 50.0),
            style: style(2.0),
        }]);

        assert!(
            svg.contains("<line x1=\"0\" y1=\"0\" x2=\"100\" y2=\"50\""),
            "{svg}"
        );
        assert!(svg.contains("stroke=\"#c5c9c5\""), "{svg}");
        assert!(svg.contains("stroke-width=\"2\""), "{svg}");
        assert!(svg.contains("stroke-linecap=\"round\""), "{svg}");
    }

    #[test]
    fn arrow_has_a_shaft_and_a_filled_head() {
        let svg = render(&[Shape::Arrow {
            a: v(0.0, 0.0),
            b: v(100.0, 0.0),
            style: style(2.0),
        }]);

        assert_eq!(count(&svg, "<line"), 1, "{svg}");
        assert_eq!(count(&svg, "<polygon"), 1, "{svg}");
        assert!(is_well_formed(&svg));
    }

    #[test]
    fn rect_has_outline_and_fill() {
        let red = ColorId::new(1);
        let svg = render(&[rect(None, red)]);

        assert!(
            svg.contains("<rect x=\"0\" y=\"0\" width=\"100\" height=\"60\""),
            "{svg}"
        );
        assert!(svg.contains("fill=\"#d16961\""), "{svg}");
        assert!(svg.contains("stroke=\"#c5c9c5\""), "{svg}");
    }

    #[test]
    fn rect_without_fill_has_fill_none() {
        let svg = render(&[rect(None, None)]);

        assert!(svg.contains("fill=\"none\""), "{svg}");
    }

    #[test]
    fn ellipse_has_centre_and_radii() {
        let svg = render(&[Shape::Ellipse {
            a: v(0.0, 0.0),
            b: v(100.0, 60.0),
            style: style(2.0),
            fill: None,
            label: None,
            text: None,
        }]);

        assert!(
            svg.contains("<ellipse cx=\"50\" cy=\"30\" rx=\"50\" ry=\"30\""),
            "{svg}"
        );
    }

    #[test]
    fn label_on_a_rect_is_drawn_as_glyphs() {
        let svg = render(&[rect(Some(12), None)]);

        assert_eq!(count(&svg, "<use"), 2, "two digits: {svg}");
    }

    #[test]
    fn grid_has_lines_cell_fills_and_index_bands() {
        let grid = Shape::Grid {
            a: v(0.0, 0.0),
            b: v(300.0, 200.0),
            cols: 3,
            rows: 2,
            style: style(2.0),
            axes: true,
            fills: vec![CellFill {
                col: 1,
                row: 0,
                color: ColorId::INK,
            }],
            texts: Vec::new(),
        };

        let svg = render(&[grid]);

        assert_eq!(count(&svg, "<line"), (3 + 1) + (2 + 1), "{svg}");
        assert_eq!(
            count(&svg, "<rect"),
            2,
            "background and one filled cell: {svg}"
        );
        // 0, 1, 2 over the columns and 0, 1 beside the rows.
        assert_eq!(count(&svg, "<use"), 5, "{svg}");
    }

    #[test]
    fn grid_cell_text_is_drawn() {
        let content = ShapeText::new("ab".to_owned(), 20.0);
        let grid = Shape::Grid {
            a: v(0.0, 0.0),
            b: v(200.0, 100.0),
            cols: 2,
            rows: 1,
            style: style(2.0),
            axes: false,
            fills: Vec::new(),
            texts: content
                .map(|content| crate::core::shape::CellText {
                    col: 1,
                    row: 0,
                    content,
                })
                .into_iter()
                .collect(),
        };

        let svg = render(&[grid]);

        assert_eq!(count(&svg, "<use"), 2, "{svg}");
    }

    #[test]
    fn stroke_is_a_round_polyline() {
        let svg = render(&[Shape::Stroke {
            points: vec![v(0.0, 0.0), v(10.0, 5.0), v(20.0, 0.0)],
            widths: None,
            style: style(3.0),
        }]);

        assert!(svg.contains("<polyline points=\"0,0 10,5 20,0\""), "{svg}");
        assert!(svg.contains("stroke-linejoin=\"round\""), "{svg}");
        assert!(svg.contains("fill=\"none\""), "{svg}");
    }

    #[test]
    fn single_point_stroke_is_a_dot() {
        let svg = render(&[Shape::Stroke {
            points: vec![v(4.0, 5.0)],
            widths: None,
            style: style(3.0),
        }]);

        assert!(svg.contains("<circle cx=\"4\" cy=\"5\" r=\"1.5\""), "{svg}");
    }

    #[test]
    fn pressure_stroke_is_one_line_per_segment() {
        let svg = render(&[Shape::Stroke {
            points: vec![v(0.0, 0.0), v(10.0, 0.0), v(20.0, 0.0)],
            widths: Some(vec![0.5, 1.0, 1.5]),
            style: style(2.0),
        }]);

        assert_eq!(count(&svg, "<line"), 2, "{svg}");
        assert!(
            svg.contains("stroke-width=\"1.5\""),
            "mean of 0.5 and 1.0: {svg}"
        );
    }

    // ---- AC-3: glyph outlines ------------------------------------------

    #[test]
    fn text_glyphs_are_defined_once_and_reused() {
        let svg = render(&[text(v(0.0, 0.0), "aaa")]);

        assert_eq!(count(&svg, "<path id="), 1, "{svg}");
        assert_eq!(count(&svg, "<use"), 3, "{svg}");
        assert!(svg.contains("<defs>"), "{svg}");
        assert!(is_well_formed(&svg));
    }

    #[test]
    fn text_with_different_characters_defines_each_once() {
        let svg = render(&[text(v(0.0, 0.0), "ab\nba")]);

        assert_eq!(count(&svg, "<path id="), 2, "{svg}");
        assert_eq!(count(&svg, "<use"), 4, "{svg}");
    }

    #[test]
    fn text_spaces_draw_nothing() {
        let svg = render(&[text(v(0.0, 0.0), "a b")]);

        assert_eq!(count(&svg, "<use"), 2, "{svg}");
    }

    #[test]
    fn text_glyph_is_placed_at_its_baseline_and_scaled_to_the_size() {
        // Size 20, 1000 units per em: scale 0.02; baseline at 1.0 em.
        let svg = render(&[text(v(10.0, 100.0), "a")]);

        assert!(
            svg.contains("transform=\"translate(10 120) scale(0.02)\""),
            "{svg}"
        );
    }

    #[test]
    fn formula_fraction_bar_is_a_line() {
        let svg = render(&[text(v(0.0, 0.0), "$\\frac{a}{b}$")]);

        assert!(count(&svg, "<line") >= 1, "{svg}");
        assert!(count(&svg, "<use") >= 2, "{svg}");
        assert!(is_well_formed(&svg));
    }

    #[test]
    fn shape_text_is_drawn_inside_the_shape() {
        let shape = Shape::Rect {
            a: v(0.0, 0.0),
            b: v(200.0, 100.0),
            style: style(2.0),
            fill: None,
            label: None,
            text: ShapeText::new("hi".to_owned(), 30.0),
        };

        let svg = render(&[shape]);

        assert_eq!(count(&svg, "<use"), 2, "{svg}");
    }

    #[test]
    fn text_on_a_filled_shape_takes_the_background_colour() {
        let shape = Shape::Rect {
            a: v(0.0, 0.0),
            b: v(200.0, 100.0),
            style: style(2.0),
            fill: ColorId::new(1),
            label: None,
            text: ShapeText::new("hi".to_owned(), 30.0),
        };

        let svg = render(&[shape]);

        assert!(svg.contains("fill=\"#181616\""), "{svg}");
    }

    #[test]
    fn shapes_keep_their_stacking_order() {
        let svg = render(&[
            Shape::Line {
                a: v(0.0, 0.0),
                b: v(1.0, 1.0),
                style: style(2.0),
            },
            rect(None, None),
        ]);

        let line = svg.find("<line");
        let rect = svg.rfind("<rect");
        assert!(line < rect, "{svg}");
    }
}
