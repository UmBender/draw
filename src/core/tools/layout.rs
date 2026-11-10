//! A text block of edges read and drawn as a graph or tree (ADR-T87-1).
//!
//! [`EdgeList::parse`] reads the text; [`EdgeList::shapes`] lays it out as
//! ordinary lines, arrows, circles and text blocks. [`replace_block`] puts
//! them in the document in place of the block, as one undo step.
//!
//! [`PointList`] does the same for a text block of points, drawn as a
//! coordinate plane (ADR-T93-1).

use crate::core::document::{Edit, ShapeId, Transaction, tx_remove};
use crate::core::geom::Vec2;
use crate::core::numbering::FIRST_NUMBER;
use crate::core::shape::{Shape, ShapeText, Style, TextSlot};
use crate::core::text_layout;
use crate::core::tools::ToolCtx;
use crate::core::tools::text::PASTE_MAX_BYTES;
use std::collections::{HashMap, HashSet};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::fmt;

/// Most nodes a graph can have.
pub const MAX_NODES: usize = 200;

/// Radius of a node's circle, in ems of the block's text.
pub const NODE_RADIUS_EM: f32 = 1.25;

/// Space between a weight's box and its edge's stroke, in ems.
const WEIGHT_GAP_EM: f32 = 0.15;

/// Most points a plot can have.
pub const MAX_POINTS: usize = 1000;

/// Largest size of a coordinate; keeps every span finite (ADR-T93-1).
pub const MAX_COORDINATE: f64 = 1e300;

/// Length of a tick across its axis, in ems of the block's text.
pub const TICK_LENGTH_EM: f32 = 0.5;

/// Radius of a plotted point's circle, in ems.
pub const POINT_RADIUS_EM: f32 = 0.2;

/// Intervals the longer axis is divided into, at most, before rounding.
const PLOT_INTERVALS: f64 = 8.0;

/// A span below this has no usable scale of its own.
const DEGENERATE_SPAN: f64 = 1e-290;

/// A span below this share of the largest coordinate has none either.
const DEGENERATE_RATIO: f64 = 1e-9;

/// How close to a multiple of the step, in steps, counts as on it.
const TICK_SNAP: f64 = 1e-9;

/// Shortest distance between two ticks, in ems.
const MIN_TICK_GAP_EM: f32 = 3.0;

/// Space kept beside the widest tick number, in ems.
const NUMBER_ROOM_EM: f32 = 1.0;

/// Space between a tick's end and its number, in ems.
const NUMBER_GAP_EM: f32 = 0.25;

/// Exponents beyond this size are written `15e-9`.
const MAX_PLAIN_EXPONENT: i32 = 6;

/// What joins two node names in a pair token.
const SEPARATORS: [char; 2] = ['-', '>'];

/// Why a text cannot become a graph (ADR-T87-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphError {
    /// A line is not an edge. Names the line, never its text: the log
    /// does not record what was typed.
    NotAnEdge {
        /// 1-based line number.
        line: usize,
    },
    /// More than [`MAX_NODES`] nodes.
    TooManyNodes {
        /// Distinct nodes named.
        nodes: usize,
    },
    /// More than [`PASTE_MAX_BYTES`] of text.
    TooLong {
        /// Bytes of the text.
        bytes: usize,
    },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnEdge { line } => write!(f, "line {line} is not an edge"),
            Self::TooManyNodes { nodes } => {
                write!(f, "{nodes} nodes, a graph holds at most {MAX_NODES}")
            }
            Self::TooLong { bytes } => {
                write!(f, "{bytes} bytes of text, the limit is {PASTE_MAX_BYTES}")
            }
        }
    }
}

impl std::error::Error for GraphError {}

/// One edge of an [`EdgeList`]; never a self-loop.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    /// Index of its first node in [`EdgeList::names`].
    pub from: usize,
    /// Index of its second node in [`EdgeList::names`].
    pub to: usize,
    /// Whether it is drawn as an arrow from `from` to `to`.
    pub directed: bool,
    /// Its weight as written, if any.
    pub weight: Option<String>,
}

/// A text read as a graph (ADR-T87-1): at least one node, at most
/// [`MAX_NODES`], no self-loop and no repeated edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeList {
    /// Node names, in the order they were first named.
    names: Vec<u32>,
    /// The edges, in the order they were written.
    edges: Vec<Edge>,
}

impl EdgeList {
    /// The graph `text` holds; `Ok(None)` when it is blank.
    ///
    /// # Errors
    ///
    /// [`GraphError`] if a line is not an edge, there are too many nodes
    /// or the text is over [`PASTE_MAX_BYTES`].
    pub fn parse(text: &str) -> Result<Option<Self>, GraphError> {
        if text.len() > PASTE_MAX_BYTES {
            return Err(GraphError::TooLong { bytes: text.len() });
        }
        let mut names = Vec::new();
        let mut index_of = HashMap::new();
        let mut edges: Vec<Edge> = Vec::new();
        let mut seen = HashSet::new();
        for (number, line) in text.lines().enumerate() {
            let parsed = line_edges(line).ok_or(GraphError::NotAnEdge { line: number + 1 })?;
            for raw in parsed {
                let mut node = |name| {
                    *index_of.entry(name).or_insert_with(|| {
                        names.push(name);
                        names.len() - 1
                    })
                };
                let (from, to) = (node(raw.u), node(raw.v));
                if from == to {
                    continue;
                }
                // An undirected edge is the same edge read either way.
                let ends = if raw.directed {
                    (from, to)
                } else {
                    (from.min(to), from.max(to))
                };
                if seen.insert((ends, raw.directed, raw.weight)) {
                    edges.push(Edge {
                        from,
                        to,
                        directed: raw.directed,
                        weight: raw.weight.map(str::to_owned),
                    });
                }
            }
        }
        if names.len() > MAX_NODES {
            return Err(GraphError::TooManyNodes { nodes: names.len() });
        }
        Ok((!names.is_empty()).then_some(Self { names, edges }))
    }

    /// Node names, in the order they were first named.
    #[must_use]
    pub fn names(&self) -> &[u32] {
        &self.names
    }

    /// The edges, in the order they were written.
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// The drawing of this graph at text size `size`, the top-left of its
    /// nodes at `origin`: edges, then circles, then weights. `None` if the
    /// size is not finite and positive or a shape would not be finite.
    #[must_use]
    pub fn shapes(&self, origin: Vec2, size: f32, style: Style) -> Option<Vec<Shape>> {
        if !size.is_finite() || size <= 0.0 {
            return None;
        }
        let radius = NODE_RADIUS_EM * size;
        let widest_weight = self
            .edges
            .iter()
            .filter_map(|edge| edge.weight.as_deref())
            .map(|weight| text_layout::bounds(Vec2::ZERO, weight, size).width())
            .fold(0.0, f32::max);
        let pitch = 2.0 * radius + (2.0 * radius).max(widest_weight + size) + 2.0 * style.width;
        let centres = self.layers(pitch).unwrap_or_else(|| self.circle(pitch));
        let shift = origin + Vec2::new(radius, radius) - top_left(&centres);

        let mut lines = Vec::with_capacity(self.edges.len());
        let mut weights = Vec::new();
        for edge in &self.edges {
            let (from, to) = (centres[edge.from], centres[edge.to]);
            let delta = to - from;
            let length = delta.length();
            if length <= 0.0 {
                return None;
            }
            let along = delta / length;
            // Just off both strokes, as outline snapping leaves it.
            let reach = along * (radius + style.width);
            let (a, b) = (from + reach + shift, to - reach + shift);
            lines.push(if edge.directed {
                Shape::Arrow { a, b, style }
            } else {
                Shape::Line { a, b, style }
            });
            if let Some(weight) = &edge.weight {
                weights.push(weight_text(weight, a.lerp(b, 0.5), along, size, style));
            }
        }
        let circles = self
            .names
            .iter()
            .zip(&centres)
            .map(|(name, centre)| node_circle(*name, *centre + shift, radius, size, style));
        let shapes: Vec<Shape> = lines.into_iter().chain(circles).chain(weights).collect();
        shapes.iter().all(Shape::is_finite).then_some(shapes)
    }

    /// Node centres of the layered drawing, or `None` if the edges are not
    /// a tree. Arrow directions do not count. The root is the first node
    /// named; every leaf takes a column of its own and every parent sits
    /// centred over its children.
    fn layers(&self, pitch: f32) -> Option<Vec<Vec2>> {
        let n = self.names.len();
        if self.edges.len() + 1 != n {
            return None;
        }
        let mut neighbours = vec![Vec::new(); n];
        for edge in &self.edges {
            neighbours[edge.from].push(edge.to);
            neighbours[edge.to].push(edge.from);
        }
        // Preorder, children in the order they were named.
        let mut parent = vec![None; n];
        let mut depth = vec![0_u32; n];
        let mut seen = vec![false; n];
        seen[0] = true;
        let mut order = Vec::with_capacity(n);
        let mut stack = vec![0];
        while let Some(node) = stack.pop() {
            order.push(node);
            for &next in neighbours[node].iter().rev() {
                if !seen[next] {
                    seen[next] = true;
                    parent[next] = Some(node);
                    depth[next] = depth[node] + 1;
                    stack.push(next);
                }
            }
        }
        if order.len() != n {
            return None;
        }

        let children = |node: usize| {
            let own_parent = parent[node];
            neighbours[node]
                .iter()
                .copied()
                .filter(move |next| Some(*next) != own_parent)
        };
        let mut column = vec![0.0_f32; n];
        let mut next_leaf = 0.0;
        for &node in &order {
            if children(node).next().is_none() {
                column[node] = next_leaf;
                next_leaf += 1.0;
            }
        }
        for &node in order.iter().rev() {
            let mut below = children(node);
            if let Some(first) = below.next() {
                let last = below.next_back().unwrap_or(first);
                column[node] = f32::midpoint(column[first], column[last]);
            }
        }

        let layer_gap = pitch * (1.0 + (n as f32).sqrt() / 4.0);
        Some(
            column
                .iter()
                .zip(&depth)
                .map(|(x, y)| Vec2::new(x * pitch, *y as f32 * layer_gap))
                .collect(),
        )
    }

    /// Node centres on a circle whose neighbours are `pitch` apart, in the
    /// order named, from the top and clockwise on screen. Only for two
    /// nodes or more: a single node is always a tree.
    fn circle(&self, pitch: f32) -> Vec<Vec2> {
        let n = self.names.len() as f32;
        let radius = pitch / (2.0 * (PI / n).sin());
        (0..self.names.len())
            .map(|i| {
                let angle = TAU * i as f32 / n - FRAC_PI_2;
                Vec2::new(angle.cos(), angle.sin()) * radius
            })
            .collect()
    }
}

/// How the lines after a keyword first line are read (ADR-T96-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputFormat {
    /// `graph:`, `tree:` or `dgraph:`: edges, plain `u v` lines directed
    /// when `directed`.
    Edges {
        /// Whether a plain `u v` line is an arrow from `u` to `v`.
        directed: bool,
    },
    /// `parents:` or `parents0:`: one parent per node.
    Parents {
        /// The first node's name, and the root of a list without `-1`.
        first: u32,
    },
}

impl InputFormat {
    /// The format a keyword line names; `None` if it is not one.
    fn of_keyword(line: &str) -> Option<Self> {
        match line.trim() {
            "graph:" | "tree:" => Some(Self::Edges { directed: false }),
            "dgraph:" => Some(Self::Edges { directed: true }),
            "parents:" => Some(Self::Parents { first: 1 }),
            "parents0:" => Some(Self::Parents { first: 0 }),
            _ => None,
        }
    }
}

/// One edge as written, before its nodes get indices.
struct RawEdge<'a> {
    /// First node's name.
    u: u32,
    /// Second node's name.
    v: u32,
    /// Whether it was written `u>v`.
    directed: bool,
    /// Its weight as written.
    weight: Option<&'a str>,
}

/// The edges of one line, none for a blank line; `None` if it is not edges.
fn line_edges(line: &str) -> Option<Vec<RawEdge<'_>>> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    match tokens.as_slice() {
        [first, ..] if first.contains(SEPARATORS) => {
            tokens.iter().map(|token| pair(token)).collect()
        }
        [u, v] => Some(vec![plain_edge(u, v, None)?]),
        [u, v, weight] => Some(vec![plain_edge(u, v, Some(weight))?]),
        [] => Some(Vec::new()),
        _ => None,
    }
}

/// The edge a token `u-v` or `u>v` writes.
fn pair(token: &str) -> Option<RawEdge<'_>> {
    let at = token.find(SEPARATORS)?;
    let (u, rest) = token.split_at(at);
    Some(RawEdge {
        u: node_name(u)?,
        v: node_name(rest.get(1..)?)?,
        directed: rest.starts_with('>'),
        weight: None,
    })
}

/// The edge of a line `u v` or `u v w`.
fn plain_edge<'a>(u: &str, v: &str, weight: Option<&'a str>) -> Option<RawEdge<'a>> {
    let finite = |w: &str| w.parse::<f64>().is_ok_and(f64::is_finite);
    if weight.is_some_and(|w| !finite(w)) {
        return None;
    }
    Some(RawEdge {
        u: node_name(u)?,
        v: node_name(v)?,
        directed: false,
        weight,
    })
}

/// A node name: ASCII digits that fit a `u32`.
fn node_name(token: &str) -> Option<u32> {
    if token.starts_with('+') {
        return None;
    }
    token.parse().ok()
}

/// The smallest `x` and the smallest `y` of `points`.
fn top_left(points: &[Vec2]) -> Vec2 {
    let corner = Vec2::new(f32::MAX, f32::MAX);
    points.iter().fold(corner, |corner, p| {
        Vec2::new(corner.x.min(p.x), corner.y.min(p.y))
    })
}

/// The circle of node `name`: labelled, or holding the text `0`, a number
/// below every label (ADR-T87-1).
fn node_circle(name: u32, centre: Vec2, radius: f32, size: f32, style: Style) -> Shape {
    let corner = Vec2::new(radius, radius);
    let circle = Shape::Ellipse {
        a: centre - corner,
        b: centre + corner,
        style,
        fill: None,
        label: None,
        text: None,
    };
    if name >= FIRST_NUMBER {
        circle.with_label(Some(name))
    } else {
        circle.with_slot_text(TextSlot::Shape, ShapeText::new(name.to_string(), size))
    }
}

/// The text block of `weight`, beside the point `middle` of an edge
/// running along the unit vector `along`, on its upper right side.
fn weight_text(weight: &str, middle: Vec2, along: Vec2, size: f32, style: Style) -> Shape {
    let bounds = text_layout::bounds(Vec2::ZERO, weight, size);
    let half = Vec2::new(bounds.width(), bounds.height()) * 0.5;
    let normal = Vec2::new(along.y, -along.x);
    let normal = if normal.x - normal.y < 0.0 {
        -normal
    } else {
        normal
    };
    // How far the box's centre must be for the box to clear the stroke.
    let clear = half.x * normal.x.abs()
        + half.y * normal.y.abs()
        + style.width * 0.5
        + WEIGHT_GAP_EM * size;
    Shape::Text {
        origin: middle + normal * clear - half,
        text: weight.to_owned(),
        size,
        style: Style {
            color: style.color,
            width: 0.0,
        },
    }
}

/// Why a text cannot become a plot (ADR-T93-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotError {
    /// A line is not two usable numbers. Names the line, never its text:
    /// the log does not record what was typed.
    NotAPoint {
        /// 1-based line number.
        line: usize,
    },
    /// More than [`MAX_POINTS`] points.
    TooManyPoints {
        /// Points listed.
        points: usize,
    },
    /// More than [`PASTE_MAX_BYTES`] of text.
    TooLong {
        /// Bytes of the text.
        bytes: usize,
    },
}

impl fmt::Display for PlotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAPoint { line } => write!(f, "line {line} is not a point"),
            Self::TooManyPoints { points } => {
                write!(f, "{points} points, a plot holds at most {MAX_POINTS}")
            }
            Self::TooLong { bytes } => {
                write!(f, "{bytes} bytes of text, the limit is {PASTE_MAX_BYTES}")
            }
        }
    }
}

impl std::error::Error for PlotError {}

/// The distance between ticks: `mantissa × 10^exponent`, the mantissa 1, 2
/// or 5. Kept apart so that tick numbers come from integers (ADR-T93-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TickStep {
    /// 1, 2 or 5.
    mantissa: i64,
    /// The power of ten.
    exponent: i32,
}

impl TickStep {
    /// The smallest step that is at least `raw`, which must be positive
    /// and finite.
    fn covering(raw: f64) -> Self {
        let exponent = raw.log10().floor() as i32;
        let fraction = raw / 10_f64.powi(exponent);
        // The tolerance keeps 1.0000000000000002 at 1 instead of 2.
        let fits = |mantissa: &i32| f64::from(*mantissa) * (1.0 + 1e-9) >= fraction;
        match [1, 2, 5].iter().find(|m| fits(m)) {
            Some(mantissa) => Self {
                mantissa: i64::from(*mantissa),
                exponent,
            },
            None => Self {
                mantissa: 1,
                exponent: exponent + 1,
            },
        }
    }

    /// The step as a number.
    fn value(self) -> f64 {
        self.mantissa as f64 * 10_f64.powi(self.exponent)
    }

    /// The number of tick `k`, the multiple `k` of this step: built from
    /// integers, so it never shows float noise.
    fn label(self, k: i64) -> String {
        let value = k.saturating_mul(self.mantissa);
        if value == 0 {
            return "0".to_owned();
        }
        if self.exponent.abs() > MAX_PLAIN_EXPONENT {
            return format!("{value}e{}", self.exponent);
        }
        let Ok(places) = usize::try_from(-self.exponent) else {
            let zeros = usize::try_from(self.exponent).unwrap_or(0);
            return format!("{value}{}", "0".repeat(zeros));
        };
        let divisor = 10_u64.pow(self.exponent.unsigned_abs());
        let size = value.unsigned_abs();
        let sign = if value < 0 { "-" } else { "" };
        let whole = size / divisor;
        let digits = format!("{:0places$}", size % divisor);
        match digits.trim_end_matches('0') {
            "" => format!("{sign}{whole}"),
            fraction => format!("{sign}{whole}.{fraction}"),
        }
    }
}

/// Ticks of one axis that carry a number, with the number.
type TickNumbers = Vec<(i64, String)>;

/// What a plot covers: both axes, in multiples of one step, each `lo < hi`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Frame {
    /// The step of both axes.
    step: TickStep,
    /// First and last tick of the horizontal axis.
    x: (i64, i64),
    /// First and last tick of the vertical axis.
    y: (i64, i64),
}

/// A text read as points (ADR-T93-1): at least one, at most [`MAX_POINTS`],
/// every coordinate finite and at most [`MAX_COORDINATE`] in size.
#[derive(Debug, Clone, PartialEq)]
pub struct PointList {
    /// The points, in the order they were written.
    points: Vec<(f64, f64)>,
}

impl PointList {
    /// The points `text` holds; `Ok(None)` when it is blank.
    ///
    /// # Errors
    ///
    /// [`PlotError`] if a line is not a point, there are too many points
    /// or the text is over [`PASTE_MAX_BYTES`].
    pub fn parse(text: &str) -> Result<Option<Self>, PlotError> {
        if text.len() > PASTE_MAX_BYTES {
            return Err(PlotError::TooLong { bytes: text.len() });
        }
        let mut points = Vec::new();
        for (number, line) in text.lines().enumerate() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            match tokens.as_slice() {
                [] => {}
                [x, y] => points.push(
                    coordinate(x)
                        .zip(coordinate(y))
                        .ok_or(PlotError::NotAPoint { line: number + 1 })?,
                ),
                _ => return Err(PlotError::NotAPoint { line: number + 1 }),
            }
        }
        if points.len() > MAX_POINTS {
            return Err(PlotError::TooManyPoints {
                points: points.len(),
            });
        }
        Ok((!points.is_empty()).then_some(Self { points }))
    }

    /// The points, in the order they were written.
    #[must_use]
    pub fn points(&self) -> &[(f64, f64)] {
        &self.points
    }

    /// The plot of these points at text size `size`, the top-left of all
    /// it draws at `origin`: axes, ticks, numbers, then points. `None` if
    /// the size is not finite and positive or a shape would not be finite.
    #[must_use]
    pub fn shapes(&self, origin: Vec2, size: f32, style: Style) -> Option<Vec<Shape>> {
        if !size.is_finite() || size <= 0.0 {
            return None;
        }
        let frame = self.frame();
        let (x_numbers, y_numbers) = frame.numbers();
        let widest = x_numbers
            .iter()
            .chain(&y_numbers)
            .map(|(_, number)| text_layout::bounds(Vec2::ZERO, number, size).width())
            .fold(0.0, f32::max);
        let pitch = (MIN_TICK_GAP_EM * size).max(widest + NUMBER_ROOM_EM * size);
        if !pitch.is_finite() {
            return None;
        }

        // Where each axis runs: through the origin, or on the nearest edge.
        let cross_x = 0_i64.clamp(frame.x.0, frame.x.1);
        let cross_y = 0_i64.clamp(frame.y.0, frame.y.1);
        let at = |x: f64, y: f64| {
            Vec2::new(
                (x - frame.x.0 as f64) as f32 * pitch,
                (frame.y.1 as f64 - y) as f32 * pitch,
            )
        };
        let tick_x = |k: i64| at(k as f64, cross_y as f64);
        let tick_y = |k: i64| at(cross_x as f64, k as f64);
        let line = |a: Vec2, b: Vec2| Shape::Line { a, b, style };
        let reach = TICK_LENGTH_EM * size * 0.5;
        let across_x = Vec2::new(0.0, reach);
        let across_y = Vec2::new(reach, 0.0);
        let gap = reach + NUMBER_GAP_EM * size;

        let mut shapes = vec![
            line(tick_x(frame.x.0), tick_x(frame.x.1)),
            line(tick_y(frame.y.0), tick_y(frame.y.1)),
        ];
        shapes.extend(
            (frame.x.0..=frame.x.1).map(|k| line(tick_x(k) - across_x, tick_x(k) + across_x)),
        );
        shapes.extend(
            (frame.y.0..=frame.y.1).map(|k| line(tick_y(k) - across_y, tick_y(k) + across_y)),
        );
        shapes.extend(x_numbers.into_iter().map(|(k, number)| {
            let half = text_layout::bounds(Vec2::ZERO, &number, size).width() * 0.5;
            number_text(number, tick_x(k) + Vec2::new(-half, gap), size, style)
        }));
        shapes.extend(y_numbers.into_iter().map(|(k, number)| {
            let bounds = text_layout::bounds(Vec2::ZERO, &number, size);
            let corner = Vec2::new(-gap - bounds.width(), -bounds.height() * 0.5);
            number_text(number, tick_y(k) + corner, size, style)
        }));
        let unit = frame.step.value();
        shapes.extend(
            self.points
                .iter()
                .map(|(x, y)| point_dot(at(x / unit, y / unit), POINT_RADIUS_EM * size, style)),
        );

        if !shapes.iter().all(Shape::is_finite) {
            return None;
        }
        let corners: Vec<Vec2> = shapes.iter().map(|shape| shape.bounds().min).collect();
        let shift = origin - top_left(&corners);
        for shape in &mut shapes {
            shape.translate(shift);
        }
        shapes.iter().all(Shape::is_finite).then_some(shapes)
    }

    /// The step and the ticks both axes run between (ADR-T93-1).
    fn frame(&self) -> Frame {
        let (x_low, x_high) = extent(self.points.iter().map(|point| point.0));
        let (y_low, y_high) = extent(self.points.iter().map(|point| point.1));
        let span = (x_high - x_low).max(y_high - y_low);
        let largest = [x_low, x_high, y_low, y_high]
            .iter()
            .fold(0.0, |largest: f64, value| largest.max(value.abs()));
        let degenerate = span < DEGENERATE_SPAN || span < largest * DEGENERATE_RATIO;
        let raw = if degenerate {
            (largest / 10.0).max(1.0)
        } else {
            span / PLOT_INTERVALS
        };
        let step = TickStep::covering(raw);
        Frame {
            step,
            x: ticks_over(x_low, x_high, step.value()),
            y: ticks_over(y_low, y_high, step.value()),
        }
    }
}

impl Frame {
    /// The ticks that carry a number and their numbers, horizontal axis
    /// then vertical. When both axes pass through the origin only the
    /// horizontal one numbers it.
    fn numbers(&self) -> (TickNumbers, TickNumbers) {
        let holds_zero = |(low, high): (i64, i64)| low <= 0 && 0 <= high;
        let origin_inside = holds_zero(self.x) && holds_zero(self.y);
        let numbered = |k: i64| (k, self.step.label(k));
        let horizontal = (self.x.0..=self.x.1).map(numbered).collect();
        let vertical = (self.y.0..=self.y.1)
            .filter(|k| !origin_inside || *k != 0)
            .map(numbered)
            .collect();
        (horizontal, vertical)
    }
}

/// A coordinate: a finite number no larger than [`MAX_COORDINATE`].
fn coordinate(token: &str) -> Option<f64> {
    let value: f64 = token.parse().ok()?;
    // Also false for NaN.
    (value.abs() <= MAX_COORDINATE).then_some(value)
}

/// The smallest and the largest of `values`.
fn extent(values: impl Iterator<Item = f64>) -> (f64, f64) {
    values.fold((f64::MAX, f64::MIN), |(low, high), value| {
        (low.min(value), high.max(value))
    })
}

/// The ticks, in multiples of `step`, an axis over `low..=high` runs
/// between; one step either side when the values sit on one multiple.
fn ticks_over(low: f64, high: f64, step: f64) -> (i64, i64) {
    let first = (low / step + TICK_SNAP).floor() as i64;
    let last = (high / step - TICK_SNAP).ceil() as i64;
    if first < last {
        (first, last)
    } else {
        (first - 1, first + 1)
    }
}

/// The text block of a tick's `number`, its top-left at `origin`.
fn number_text(number: String, origin: Vec2, size: f32, style: Style) -> Shape {
    Shape::Text {
        origin,
        text: number,
        size,
        style: Style {
            color: style.color,
            width: 0.0,
        },
    }
}

/// The filled circle of a plotted point.
fn point_dot(centre: Vec2, radius: f32, style: Style) -> Shape {
    let corner = Vec2::new(radius, radius);
    Shape::Ellipse {
        a: centre - corner,
        b: centre + corner,
        style,
        fill: Some(style.color),
        label: None,
        text: None,
    }
}

/// Replaces the shape `id` by `shapes` at its place in the z-order, as one
/// undo step, and selects them. Returns whether the document changed.
pub fn replace_block(ctx: &mut ToolCtx<'_>, id: ShapeId, shapes: Vec<Shape>) -> bool {
    let Some(index) = ctx.doc.index_of(id) else {
        return false;
    };
    let mut edits = tx_remove(ctx.doc, &[id]).0;
    let mut ids = Vec::with_capacity(shapes.len());
    for (offset, shape) in shapes.into_iter().enumerate() {
        let new = ctx.doc.next_id();
        ids.push(new);
        edits.push(Edit::Insert {
            index: index + offset,
            id: new,
            shape,
        });
    }
    let committed = ctx.commit(Transaction(edits));
    if committed {
        *ctx.selection = ids;
    }
    committed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;
    use crate::core::palette::ColorId;
    use crate::core::shape::TextSlot;
    use crate::core::text_layout;
    use proptest::prelude::*;

    const SIZE: f32 = 20.0;
    const WIDTH: f32 = 2.0;
    const RADIUS: f32 = NODE_RADIUS_EM * SIZE;
    const STYLE: Style = Style {
        color: ColorId::INK,
        width: WIDTH,
    };

    fn list(text: &str) -> EdgeList {
        match EdgeList::parse(text) {
            Ok(Some(list)) => list,
            other => panic!("{text:?} gave {other:?}"),
        }
    }

    fn drawn(text: &str) -> Vec<Shape> {
        match list(text).shapes(Vec2::ZERO, SIZE, STYLE) {
            Some(shapes) => shapes,
            None => panic!("{text:?} has no shapes"),
        }
    }

    /// `(name of from, name of to, directed, weight)` of every edge.
    fn edges(list: &EdgeList) -> Vec<(u32, u32, bool, Option<&str>)> {
        list.edges()
            .iter()
            .map(|e| {
                (
                    list.names()[e.from],
                    list.names()[e.to],
                    e.directed,
                    e.weight.as_deref(),
                )
            })
            .collect()
    }

    /// The name a circle shows: its label, or its text for node 0.
    fn name(shape: &Shape) -> Option<u32> {
        shape.label().or_else(|| {
            shape
                .slot_text(TextSlot::Shape)
                .and_then(|text| text.text.parse().ok())
        })
    }

    /// `(name, centre)` of every circle.
    fn nodes(shapes: &[Shape]) -> Vec<(u32, Vec2)> {
        shapes
            .iter()
            .filter(|shape| matches!(shape, Shape::Ellipse { .. }))
            .filter_map(|shape| Some((name(shape)?, shape.bounds().center())))
            .collect()
    }

    fn centre(shapes: &[Shape], wanted: u32) -> Vec2 {
        match nodes(shapes).iter().find(|(name, _)| *name == wanted) {
            Some((_, centre)) => *centre,
            None => panic!("no node {wanted}"),
        }
    }

    /// The end points of every line and arrow.
    fn segments(shapes: &[Shape]) -> Vec<(Vec2, Vec2)> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Line { a, b, .. } | Shape::Arrow { a, b, .. } => Some((*a, *b)),
                _ => None,
            })
            .collect()
    }

    /// A path `1-2`, `2-3`, … over `n` nodes.
    fn path(n: u32) -> String {
        (1..n)
            .map(|i| format!("{i}-{}", i + 1))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// A ring over `n` nodes.
    fn ring(n: u32) -> String {
        format!("{}\n{n}-1", path(n))
    }

    fn mean(points: &[Vec2]) -> Vec2 {
        let sum = points.iter().fold(Vec2::ZERO, |sum, p| sum + *p);
        sum / points.len() as f32
    }

    fn assert_never_overlap(shapes: &[Shape]) -> Result<(), TestCaseError> {
        let nodes = nodes(shapes);
        for (i, (a, ca)) in nodes.iter().enumerate() {
            for (b, cb) in &nodes[i + 1..] {
                prop_assert!(
                    ca.distance(*cb) >= 2.0 * RADIUS + WIDTH,
                    "nodes {} and {} overlap",
                    a,
                    b
                );
            }
        }
        Ok(())
    }

    // ---- AC-1 -------------------------------------------------------------

    #[test]
    fn shapes_label_circles_with_node_names() {
        // Act
        let shapes = drawn("3-7");

        // Assert
        let circles: Vec<_> = shapes
            .iter()
            .filter(|s| matches!(s, Shape::Ellipse { .. }))
            .collect();
        assert_eq!(circles.len(), 2);
        assert_eq!(circles[0].label(), Some(3));
        assert_eq!(circles[1].label(), Some(7));
        for circle in circles {
            let bounds = circle.bounds();
            assert!(approx_eq(bounds.width(), bounds.height(), 1e-3));
            assert_eq!(circle.fill(), None);
            assert_eq!(circle.style(), STYLE);
        }
    }

    #[test]
    fn shapes_write_node_zero_as_text() {
        // Act
        let shapes = drawn("0-1");

        // Assert: label 0 is below the labels this base accepts.
        let circles: Vec<_> = shapes
            .iter()
            .filter(|s| matches!(s, Shape::Ellipse { .. }))
            .collect();
        assert_eq!(circles[0].label(), None);
        assert_eq!(
            circles[0]
                .slot_text(TextSlot::Shape)
                .map(|t| t.text.as_str()),
            Some("0")
        );
        assert_eq!(circles[1].label(), Some(1));
    }

    #[test]
    fn shapes_put_the_top_left_of_the_nodes_at_the_origin() {
        for text in ["1-2 1-3", "1-2 2-3 3-1"] {
            // Arrange
            let origin = Vec2::new(40.0, 60.0);

            // Act
            let shapes = list(text).shapes(origin, SIZE, STYLE).unwrap_or_default();

            // Assert
            let circles: Vec<_> = shapes
                .iter()
                .filter(|s| matches!(s, Shape::Ellipse { .. }))
                .map(Shape::bounds)
                .collect();
            let left = circles.iter().map(|b| b.min.x).fold(f32::MAX, f32::min);
            let top = circles.iter().map(|b| b.min.y).fold(f32::MAX, f32::min);
            // A shape's bounds include half its stroke.
            assert!(
                approx_eq(left, origin.x - WIDTH * 0.5, 1e-2),
                "{text}: {left}"
            );
            assert!(
                approx_eq(top, origin.y - WIDTH * 0.5, 1e-2),
                "{text}: {top}"
            );
        }
    }

    #[test]
    fn shapes_stack_edges_then_circles_then_weights() {
        // Act
        let shapes = drawn("1 2 5\n2 3 6");

        // Assert
        let rank = |shape: &Shape| match shape {
            Shape::Line { .. } | Shape::Arrow { .. } => 0,
            Shape::Ellipse { .. } => 1,
            _ => 2,
        };
        assert!(shapes.windows(2).all(|w| rank(&w[0]) <= rank(&w[1])));
        assert_eq!(shapes.len(), 2 + 3 + 2);
    }

    #[test]
    fn shapes_refuse_a_size_that_is_not_positive_and_finite() {
        for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(list("1-2").shapes(Vec2::ZERO, size, STYLE), None);
        }
    }

    #[test]
    fn shapes_refuse_a_result_that_is_not_finite() {
        // Act / Assert: far enough that the layout overflows.
        let huge = Vec2::new(f32::MAX, f32::MAX);
        assert_eq!(list("1-2").shapes(huge, f32::MAX, STYLE), None);
    }

    // ---- AC-2 -------------------------------------------------------------

    #[test]
    fn parse_reads_pairs() {
        assert_eq!(
            edges(&list("1-2 2-3")),
            [(1, 2, false, None), (2, 3, false, None)]
        );
    }

    #[test]
    fn parse_reads_directed_pairs() {
        assert_eq!(
            edges(&list("2>1\n1>3 3-4")),
            [(2, 1, true, None), (1, 3, true, None), (3, 4, false, None)]
        );
    }

    #[test]
    fn parse_reads_plain_lines_and_weights() {
        assert_eq!(
            edges(&list("1 2\n2 3 7\r\n3 4 -5\n\n  4\t5   1.5  ")),
            [
                (1, 2, false, None),
                (2, 3, false, Some("7")),
                (3, 4, false, Some("-5")),
                (4, 5, false, Some("1.5")),
            ]
        );
    }

    #[test]
    fn parse_names_nodes_in_first_named_order() {
        assert_eq!(list("5-3 3-9\n1 5").names(), [5, 3, 9, 1]);
    }

    #[test]
    fn parse_blank_text_is_no_graph() {
        for text in ["", "  \n\t\n "] {
            assert_eq!(EdgeList::parse(text), Ok(None));
        }
    }

    #[test]
    fn shapes_draw_arrows_only_for_directed_edges() {
        // Act
        let shapes = drawn("1>2 2-3");

        // Assert
        let arrows = shapes
            .iter()
            .filter(|s| matches!(s, Shape::Arrow { .. }))
            .count();
        let lines = shapes
            .iter()
            .filter(|s| matches!(s, Shape::Line { .. }))
            .count();
        assert_eq!((arrows, lines), (1, 1));
        let Some(Shape::Arrow { a, b, .. }) =
            shapes.iter().find(|s| matches!(s, Shape::Arrow { .. }))
        else {
            panic!("no arrow");
        };
        assert!(a.distance(centre(&shapes, 1)) < b.distance(centre(&shapes, 1)));
    }

    #[test]
    fn shapes_put_a_weight_beside_the_middle_of_its_edge() {
        for text in ["1 2 7", "1 2 7\n2 3 8\n3 1 9"] {
            // Act
            let shapes = drawn(text);

            // Assert: one per edge, close to its middle, never across it.
            let weights: Vec<_> = shapes
                .iter()
                .filter_map(|s| match s {
                    Shape::Text {
                        origin,
                        text,
                        size,
                        style,
                    } => Some((text_layout::bounds(*origin, text, *size), *size, *style)),
                    _ => None,
                })
                .collect();
            let segments = segments(&shapes);
            assert_eq!(weights.len(), segments.len());
            for ((bounds, size, style), (a, b)) in weights.iter().zip(&segments) {
                let middle = a.lerp(*b, 0.5);
                assert!(approx_eq(*size, SIZE, 1e-4));
                assert_eq!(style.color, STYLE.color);
                assert!(!bounds.contains(middle), "{text}: over the edge");
                assert!(bounds.expand(SIZE).contains(middle), "{text}: too far");
            }
        }
    }

    // ---- AC-3 -------------------------------------------------------------

    #[test]
    fn tree_is_drawn_in_layers_from_the_first_node_named() {
        // Act
        let shapes = drawn("2-1\n2-3\n1-4");

        // Assert
        let (root, left, right, deep) = (
            centre(&shapes, 2),
            centre(&shapes, 1),
            centre(&shapes, 3),
            centre(&shapes, 4),
        );
        assert!(approx_eq(left.y, right.y, 1e-3));
        assert!(root.y < left.y && left.y < deep.y);
        assert!(left.x < right.x, "children in the order named");
    }

    #[test]
    fn tree_parent_is_centred_over_its_children() {
        // Act
        let shapes = drawn("1-2 1-3 1-4");

        // Assert
        let (root, first, last) = (centre(&shapes, 1), centre(&shapes, 2), centre(&shapes, 4));
        assert!(approx_eq(root.x, f32::midpoint(first.x, last.x), 1e-3));
    }

    #[test]
    fn tree_ignores_arrow_direction() {
        // Act: every arrow points at 1, and 2 is named first.
        let shapes = drawn("2>1 3>1");

        // Assert
        let (root, middle, far) = (centre(&shapes, 2), centre(&shapes, 1), centre(&shapes, 3));
        assert!(root.y < middle.y && middle.y < far.y);
    }

    #[test]
    fn single_node_is_a_tree_at_the_origin() {
        // Act
        let shapes = drawn("4-4");

        // Assert
        assert_eq!(nodes(&shapes).len(), 1);
        assert!(centre(&shapes, 4).approx_eq(Vec2::new(RADIUS, RADIUS), 1e-3));
    }

    fn assert_on_a_circle(shapes: &[Shape]) {
        let centres: Vec<Vec2> = nodes(shapes).iter().map(|(_, c)| *c).collect();
        let middle = mean(&centres);
        let radius = centres[0].distance(middle);
        assert!(radius > RADIUS);
        for c in &centres {
            assert!(approx_eq(c.distance(middle), radius, 1e-2), "{c:?}");
        }
    }

    #[test]
    fn cycle_goes_on_a_circle() {
        // Act
        let shapes = drawn("1-2 2-3 3-1");

        // Assert: starts at the top.
        assert_on_a_circle(&shapes);
        let top = centre(&shapes, 1).y;
        assert!(nodes(&shapes).iter().all(|(_, c)| c.y >= top));
    }

    #[test]
    fn forest_goes_on_a_circle() {
        assert_on_a_circle(&drawn("1-2 3-4"));
    }

    #[test]
    fn opposite_arrows_are_not_a_tree() {
        assert_on_a_circle(&drawn("1>2 2>3 3>2 3>1"));
    }

    #[test]
    fn circle_radius_and_layer_gap_grow_with_the_node_count() {
        // Arrange
        let circle_radius = |n| {
            let shapes = drawn(&ring(n));
            let centres: Vec<Vec2> = nodes(&shapes).iter().map(|(_, c)| *c).collect();
            centres[0].distance(mean(&centres))
        };
        let layer_gap = |n| {
            let shapes = drawn(&path(n));
            centre(&shapes, 2).y - centre(&shapes, 1).y
        };

        // Act / Assert
        assert!(circle_radius(40) > 2.0 * circle_radius(10));
        assert!(layer_gap(64) > layer_gap(4));
    }

    #[test]
    fn long_weights_keep_the_nodes_apart() {
        // Act
        let shapes = drawn("1 2 100000000\n2 3 1\n3 1 1");
        let plain = drawn("1 2 1\n2 3 1\n3 1 1");

        // Assert
        let side = |s: &[Shape]| centre(s, 1).distance(centre(s, 2));
        assert!(side(&shapes) > side(&plain));
    }

    proptest! {
        #[test]
        fn nodes_never_overlap(
            pairs in prop::collection::vec((1_u32..40, 1_u32..40), 1..80),
        ) {
            // Arrange
            let text = pairs
                .iter()
                .map(|(u, v)| format!("{u}-{v}"))
                .collect::<Vec<_>>()
                .join(" ");

            // Act / Assert
            assert_never_overlap(&drawn(&text))?;
        }

        #[test]
        fn tree_nodes_never_overlap(parents in prop::collection::vec(0.0_f32..1.0, 1..120)) {
            // Arrange: node i + 2 hangs from any earlier node.
            let text = parents
                .iter()
                .enumerate()
                .map(|(i, r)| format!("{}-{}", 1 + (r * (i + 1) as f32) as usize, i + 2))
                .collect::<Vec<_>>()
                .join("\n");

            // Act / Assert
            assert_never_overlap(&drawn(&text))?;
        }
    }

    // ---- AC-4 -------------------------------------------------------------

    #[test]
    fn edges_end_on_the_outlines() {
        for text in ["1-2 1>3", "1-2 2>3 3-1"] {
            // Act
            let shapes = drawn(text);
            let reach = RADIUS + WIDTH;

            // Assert: on the line through the centres, just off each stroke.
            let ends = segments(&shapes);
            let named = list(text);
            for (edge, (a, b)) in named.edges().iter().zip(&ends) {
                let from = centre(&shapes, named.names()[edge.from]);
                let to = centre(&shapes, named.names()[edge.to]);
                assert!(approx_eq(a.distance(from), reach, 1e-2), "{text}");
                assert!(approx_eq(b.distance(to), reach, 1e-2), "{text}");
                assert!(
                    approx_eq(a.distance(*b) + 2.0 * reach, from.distance(to), 1e-2),
                    "{text}"
                );
            }
        }
    }

    // ---- AC-5 -------------------------------------------------------------

    #[test]
    fn parse_accepts_200_nodes() {
        assert_eq!(list(&path(200)).names().len(), MAX_NODES);
    }

    #[test]
    fn parse_refuses_more_than_200_nodes() {
        assert_eq!(
            EdgeList::parse(&path(201)),
            Err(GraphError::TooManyNodes { nodes: 201 })
        );
    }

    #[test]
    fn parse_refuses_what_is_not_an_edge() {
        for text in [
            "a b",
            "1",
            "1 2 3 4",
            "1-2 3",
            "1-2-3",
            "-1 2",
            "1 2 x",
            "1 2 inf",
            "1 2 NaN",
            "1 2 1e999",
            "1>",
            ">2",
            "1--2",
            "+1 2",
            "4294967296 1",
            "$x$ 2",
        ] {
            assert_eq!(
                EdgeList::parse(text),
                Err(GraphError::NotAnEdge { line: 1 }),
                "{text}"
            );
        }
    }

    #[test]
    fn parse_refuses_too_much_text() {
        // Arrange
        let text = "1-2 ".repeat(PASTE_MAX_BYTES / 4 + 1);

        // Act / Assert
        assert_eq!(
            EdgeList::parse(&text),
            Err(GraphError::TooLong { bytes: text.len() })
        );
    }

    #[test]
    fn parse_error_names_the_line_not_its_text() {
        // Act
        let err = EdgeList::parse("1-2\n\nsecret");

        // Assert
        assert_eq!(err, Err(GraphError::NotAnEdge { line: 3 }));
        let reason = GraphError::NotAnEdge { line: 3 }.to_string();
        assert!(reason.contains('3') && !reason.contains("secret"));
    }

    // ---- AC-6 -------------------------------------------------------------

    #[test]
    fn parse_skips_a_self_loop_but_keeps_its_node() {
        // Act
        let parsed = list("1-1\n2 2");

        // Assert
        assert_eq!(parsed.names(), [1, 2]);
        assert_eq!(parsed.edges(), []);
    }

    #[test]
    fn parse_draws_a_duplicate_once() {
        assert_eq!(edges(&list("1-2 1-2 2-1")), [(1, 2, false, None)]);
        assert_eq!(edges(&list("1>2 1>2")), [(1, 2, true, None)]);
        assert_eq!(edges(&list("1 2 5\n2 1 5")), [(1, 2, false, Some("5"))]);
    }

    #[test]
    fn parse_keeps_edges_that_differ_in_direction_or_weight() {
        assert_eq!(edges(&list("1>2 2>1")).len(), 2);
        assert_eq!(edges(&list("1-2 1>2")).len(), 2);
        assert_eq!(edges(&list("1 2 5\n1 2 7")).len(), 2);
    }

    proptest! {
        #[test]
        fn parse_and_shapes_hold_for_any_text(
            text in prop_oneof![
                any::<String>(),
                "[0-9 >\\-\n.]{0,80}",
            ],
            size in 0.01_f32..5000.0,
            width in 0.0_f32..100.0,
            x in -1e6_f32..1e6,
        ) {
            // Act: must never panic.
            let parsed = EdgeList::parse(&text);

            // Assert
            if let Ok(Some(list)) = parsed {
                prop_assert!(list.names().len() <= MAX_NODES);
                prop_assert!(list.edges().iter().all(|e| e.from != e.to));
                let style = Style { width, ..STYLE };
                if let Some(shapes) = list.shapes(Vec2::new(x, 0.0), size, style) {
                    prop_assert!(shapes.iter().all(Shape::is_finite));
                }
            }
        }
    }

    // ---- T93 coordinate plane ---------------------------------------------

    const TICK: f32 = TICK_LENGTH_EM * SIZE;

    // ---- T96 keywords for graph input ---------------------------------------

    fn graph(text: &str) -> EdgeList {
        match EdgeList::parse(text) {
            Ok(Some(list)) => list,
            other => panic!("{text:?} gave {other:?}"),
        }
    }

    #[test]
    fn parse_without_a_keyword_keeps_the_t87_rules() {
        // Act, Assert: a count line is still an edge, `u v` undirected.
        let list = graph("3 2\n1 2\n2 3");
        assert_eq!(list.names(), [3, 2, 1]);
        assert!(list.edges().iter().all(|edge| !edge.directed));
    }

    #[test]
    fn parse_refuses_an_unknown_keyword() {
        for text in ["digraph:\n1 2", "Tree:\n1 2", "graph\n1 2", "graph: 1 2"] {
            assert_eq!(
                EdgeList::parse(text),
                Err(GraphError::NotAnEdge { line: 1 }),
                "{text:?}"
            );
        }
    }

    #[test]
    fn parse_graph_keyword_reads_edges() {
        assert_eq!(
            graph("graph:\n1 2\n2>3 3-4\n4 5 7"),
            graph("1 2\n2>3 3-4\n4 5 7")
        );
    }

    #[test]
    fn parse_tree_keyword_reads_edges() {
        assert_eq!(graph("  tree:  \n\n1 2\n1 3"), graph("1 2\n1 3"));
    }

    #[test]
    fn parse_dgraph_keyword_makes_plain_edges_directed() {
        // Act
        let list = graph("dgraph:\n1 2\n2 3 5");

        // Assert
        let edges = list.edges();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().all(|edge| edge.directed));
        assert_eq!(edges[1].weight.as_deref(), Some("5"));
        assert_eq!(
            (list.names()[edges[1].from], list.names()[edges[1].to]),
            (2, 3)
        );
    }

    #[test]
    fn parse_dgraph_keyword_keeps_written_pairs() {
        // Act
        let list = graph("dgraph:\n1-2 2>3");

        // Assert
        let directed: Vec<bool> = list.edges().iter().map(|edge| edge.directed).collect();
        assert_eq!(directed, [false, true]);
    }

    #[test]
    fn parse_skips_a_count_line_after_a_keyword() {
        assert_eq!(graph("tree:\n3\n1 2\n1 3"), graph("1 2\n1 3"));
        assert_eq!(graph("graph:\n3 2\n1 2\n2 3"), graph("1 2\n2 3"));
        assert_eq!(
            graph("dgraph:\n4 3\n1 2\n2 3 9\n3 4"),
            graph("dgraph:\n1 2\n2 3 9\n3 4")
        );
        // `3 1` fits a count line before one edge, so it is one.
        assert_eq!(graph("graph:\n3 1\n1 2"), graph("1 2"));
    }

    #[test]
    fn parse_keeps_a_first_edge_that_is_not_a_count_line() {
        assert_eq!(graph("graph:\n1 2\n2 3"), graph("1 2\n2 3"));
        assert_eq!(graph("graph:\n1 2"), graph("1 2"));
        assert_eq!(graph("graph:\n1 0"), graph("1 0"));
        assert_eq!(graph("graph:\n2 1 5\n1 3"), graph("2 1 5\n1 3"));
    }

    #[test]
    fn parse_never_skips_without_a_keyword() {
        assert_eq!(
            EdgeList::parse("3\n1 2"),
            Err(GraphError::NotAnEdge { line: 1 })
        );
    }

    #[test]
    fn parse_parents_reads_p2_to_pn_with_root_1() {
        assert_eq!(graph("parents:\n1 1 2 2"), graph("1 2\n1 3\n2 4\n2 5"));
    }

    #[test]
    fn parse_parents0_reads_p1_onwards_with_root_0() {
        assert_eq!(graph("parents0:\n0 0 1"), graph("0 1\n0 2\n1 3"));
    }

    #[test]
    fn parse_parents_reads_a_full_list_with_minus_one_at_the_root() {
        assert_eq!(graph("parents:\n2 -1 2"), graph("2 1\n2 3"));
        assert_eq!(graph("parents0:\n-1 0 0 1"), graph("0 1\n0 2\n1 3"));
    }

    #[test]
    fn parse_parents_skips_a_count_line() {
        assert_eq!(graph("parents:\n5\n1 1 2 2"), graph("parents:\n1 1 2 2"));
        assert_eq!(graph("parents:\n3\n2 -1 2"), graph("parents:\n2 -1 2"));
        assert_eq!(graph("parents0:\n4\n0 0 1"), graph("parents0:\n0 0 1"));
    }

    #[test]
    fn parse_parents_reads_entries_across_lines() {
        // `1` is no count line: two entries follow, not 0 or 1.
        assert_eq!(graph("parents:\n1\n1\n2"), graph("parents:\n1 1 2"));
    }

    #[test]
    fn parse_parents_names_the_root_first() {
        // Act: node 2's parent is 3, node 3's is 1.
        let list = graph("parents:\n3 1");

        // Assert
        assert_eq!(list.names()[0], 1);
        assert_eq!(list.names().len(), 3);
    }

    #[test]
    fn parents_tree_is_drawn_from_its_root() {
        // Act
        let shapes = graph("parents:\n3 1")
            .shapes(Vec2::ZERO, SIZE, STYLE)
            .unwrap_or_default();

        // Assert: the root's circle is above the others.
        let top_of = |name| {
            shapes
                .iter()
                .find(|s| s.label() == Some(name))
                .map_or(f32::MAX, |s| s.bounds().min.y)
        };
        assert!(top_of(1) < top_of(3), "{shapes:?}");
        assert!(top_of(3) < top_of(2), "{shapes:?}");
    }

    #[test]
    fn parse_parents_refuses_bad_entries() {
        for (text, line) in [
            ("parents:\n1 x", 2),
            ("parents:\n-1 -1", 2),
            ("parents:\n1 -2", 2),
            ("parents:\n1 +1", 2),
            ("parents:\n1\n2 x", 3),
            ("parents0:\n0 1.5", 2),
        ] {
            assert_eq!(
                EdgeList::parse(text),
                Err(GraphError::NotAnEdge { line }),
                "{text:?}"
            );
        }
    }

    #[test]
    fn parse_keyword_alone_is_no_graph() {
        for text in [
            "graph:",
            "tree:\n\n",
            "dgraph:\n  ",
            "parents:",
            "parents0:\n",
        ] {
            assert_eq!(EdgeList::parse(text), Ok(None), "{text:?}");
        }
    }

    fn points(text: &str) -> PointList {
        match PointList::parse(text) {
            Ok(Some(list)) => list,
            other => panic!("{text:?} gave {other:?}"),
        }
    }

    fn plot(text: &str) -> Vec<Shape> {
        match points(text).shapes(Vec2::ZERO, SIZE, STYLE) {
            Some(shapes) => shapes,
            None => panic!("{text:?} has no plot"),
        }
    }

    fn step(mantissa: i64, exponent: i32) -> TickStep {
        TickStep { mantissa, exponent }
    }

    fn is_tick(segment: &(Vec2, Vec2)) -> bool {
        approx_eq(segment.0.distance(segment.1), TICK, 1e-2)
    }

    /// The horizontal axis, then the vertical one.
    fn plot_axes(shapes: &[Shape]) -> ((Vec2, Vec2), (Vec2, Vec2)) {
        let axes: Vec<_> = segments(shapes)
            .into_iter()
            .filter(|s| !is_tick(s))
            .collect();
        assert_eq!(axes.len(), 2, "{axes:?}");
        assert!(approx_eq(axes[0].0.y, axes[0].1.y, 1e-2), "x axis is level");
        assert!(approx_eq(axes[1].0.x, axes[1].1.x, 1e-2), "y axis is plumb");
        (axes[0], axes[1])
    }

    /// Where the ticks across the `x` axis are, along it.
    fn x_ticks(shapes: &[Shape]) -> Vec<f32> {
        segments(shapes)
            .iter()
            .filter(|s| is_tick(s) && approx_eq(s.0.x, s.1.x, 1e-3))
            .map(|s| s.0.x)
            .collect()
    }

    /// Where the ticks across the `y` axis are, along it.
    fn y_ticks(shapes: &[Shape]) -> Vec<f32> {
        segments(shapes)
            .iter()
            .filter(|s| is_tick(s) && approx_eq(s.0.y, s.1.y, 1e-3))
            .map(|s| s.0.y)
            .collect()
    }

    fn numbers(shapes: &[Shape]) -> Vec<(&str, crate::core::geom::Aabb)> {
        shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text {
                    origin, text, size, ..
                } => Some((text.as_str(), text_layout::bounds(*origin, text, *size))),
                _ => None,
            })
            .collect()
    }

    fn dots(shapes: &[Shape]) -> Vec<Vec2> {
        shapes
            .iter()
            .filter(|s| matches!(s, Shape::Ellipse { .. }))
            .map(|s| s.bounds().center())
            .collect()
    }

    /// Every plot, however odd its data: two axes, ticks on both, one dot
    /// per point and every dot on the axes' frame.
    fn assert_valid_plot(shapes: &[Shape], wanted_dots: usize) {
        assert!(shapes.iter().all(Shape::is_finite));
        let (x_axis, y_axis) = plot_axes(shapes);
        assert!(x_axis.0.distance(x_axis.1) > 0.0);
        assert!(y_axis.0.distance(y_axis.1) > 0.0);
        let (xs, ys) = (x_ticks(shapes), y_ticks(shapes));
        assert!(xs.len() >= 2, "{xs:?}");
        assert!(ys.len() >= 2, "{ys:?}");
        assert_ne!(numbers(shapes).len(), 0);
        let centres = dots(shapes);
        assert_eq!(centres.len(), wanted_dots);
        let reach = |values: &[f32]| {
            let low = values.iter().copied().fold(f32::MAX, f32::min);
            let high = values.iter().copied().fold(f32::MIN, f32::max);
            (low - 0.1, high + 0.1)
        };
        let ((left, right), (top, bottom)) = (reach(&xs), reach(&ys));
        for dot in centres {
            assert!(left <= dot.x && dot.x <= right, "{dot:?} {left} {right}");
            assert!(top <= dot.y && dot.y <= bottom, "{dot:?} {top} {bottom}");
        }
    }

    // ---- T93 AC-1 ---------------------------------------------------------

    #[test]
    fn plot_draws_axes_ticks_numbers_and_one_dot_per_point() {
        // Act: step 2, x over 0..=10 and y over 0..=6.
        let shapes = plot("0 0\n10 5");

        // Assert
        plot_axes(&shapes);
        assert_eq!(x_ticks(&shapes).len(), 6);
        assert_eq!(y_ticks(&shapes).len(), 4);
        assert_eq!(numbers(&shapes).len(), 6 + 3);
        assert_eq!(dots(&shapes).len(), 2);
        assert_eq!(shapes.len(), 2 + 10 + 9 + 2);
    }

    #[test]
    fn plot_points_are_small_filled_circles() {
        // Act
        let shapes = plot("1 2\n3 4");

        // Assert
        let wide = 2.0 * POINT_RADIUS_EM * SIZE;
        let mut seen = 0;
        for shape in &shapes {
            if let Shape::Ellipse {
                a,
                b,
                style,
                fill,
                label,
                text,
            } = shape
            {
                seen += 1;
                assert!(approx_eq(b.x - a.x, wide, 1e-3));
                assert!(approx_eq(b.y - a.y, wide, 1e-3));
                assert_eq!(*style, STYLE);
                assert_eq!(*fill, Some(STYLE.color));
                assert_eq!(*label, None);
                assert_eq!(*text, None);
            }
        }
        assert_eq!(seen, 2);
    }

    #[test]
    fn plot_points_sit_where_their_coordinates_say() {
        // Act: step 2, so one tick gap is two units.
        let shapes = plot("0 0\n10 5\n4 2");

        // Assert
        let xs = x_ticks(&shapes);
        let gap = xs[1] - xs[0];
        let centres = dots(&shapes);
        assert!(approx_eq(centres[0].x, xs[0], 1e-2));
        let far = centres[1] - centres[0];
        assert!(approx_eq(far.x, 5.0 * gap, 1e-2), "{far:?}");
        assert!(approx_eq(far.y, -2.5 * gap, 1e-2), "{far:?}");
        let near = centres[2] - centres[0];
        assert!(approx_eq(near.x, 2.0 * gap, 1e-2), "{near:?}");
        assert!(approx_eq(near.y, -gap, 1e-2), "{near:?}");
    }

    #[test]
    fn plot_y_grows_upward() {
        // Act
        let centres = dots(&plot("0 0\n0 9"));

        // Assert: screen y grows downward.
        assert!(centres[1].y < centres[0].y);
    }

    #[test]
    fn plot_puts_its_top_left_at_the_origin() {
        for text in ["0 0\n10 5", "-5 -5\n5 5", "3 4"] {
            // Arrange
            let origin = Vec2::new(40.0, 60.0);

            // Act
            let shapes = points(text).shapes(origin, SIZE, STYLE).unwrap_or_default();

            // Assert
            let left = shapes
                .iter()
                .map(|s| s.bounds().min.x)
                .fold(f32::MAX, f32::min);
            let top = shapes
                .iter()
                .map(|s| s.bounds().min.y)
                .fold(f32::MAX, f32::min);
            assert!(approx_eq(left, origin.x, 1e-2), "{text}: {left}");
            assert!(approx_eq(top, origin.y, 1e-2), "{text}: {top}");
        }
    }

    #[test]
    fn plot_stacks_axes_ticks_numbers_points() {
        // Act
        let shapes = plot("-4 -2\n6 8");

        // Assert
        let rank = |shape: &Shape| match shape {
            Shape::Line { a, b, .. } if is_tick(&(*a, *b)) => 1,
            Shape::Line { .. } => 0,
            Shape::Text { .. } => 2,
            _ => 3,
        };
        assert!(shapes.windows(2).all(|w| rank(&w[0]) <= rank(&w[1])));
    }

    #[test]
    fn plot_refuses_a_size_that_is_not_positive_and_finite() {
        for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(points("1 2").shapes(Vec2::ZERO, size, STYLE), None);
        }
        // Far enough that the layout overflows.
        let huge = Vec2::new(f32::MAX, f32::MAX);
        assert_eq!(points("0 0\n9 9").shapes(huge, f32::MAX, STYLE), None);
    }

    // ---- T93 AC-2 ---------------------------------------------------------

    proptest! {
        #[test]
        fn step_is_one_two_or_five_times_a_power_of_ten(
            exponent in -280_i32..280,
            mantissa in 1.0_f64..10.0,
        ) {
            // Arrange
            let raw = mantissa * 10_f64.powi(exponent);

            // Act
            let tick = TickStep::covering(raw);

            // Assert
            prop_assert!([1, 2, 5].contains(&tick.mantissa));
            prop_assert!(tick.value() >= raw * (1.0 - 1e-9));
            prop_assert!(tick.value() < raw * 2.5 * (1.0 + 1e-9));
        }
    }

    #[test]
    fn step_rounds_up_to_the_next_nice_value() {
        for (raw, wanted) in [
            (1.0, step(1, 0)),
            (1.25, step(2, 0)),
            (2.0, step(2, 0)),
            (2.1, step(5, 0)),
            (5.5, step(1, 1)),
            (1000.0, step(1, 3)),
            (0.03, step(5, -2)),
        ] {
            assert_eq!(TickStep::covering(raw), wanted, "{raw}");
        }
    }

    #[test]
    fn frame_uses_one_step_for_both_axes() {
        assert_eq!(
            points("0 0\n100 1").frame(),
            Frame {
                step: step(2, 1),
                x: (0, 5),
                y: (0, 1),
            }
        );
        assert_eq!(
            points("-5 -3\n7 1").frame(),
            Frame {
                step: step(2, 0),
                x: (-3, 4),
                y: (-2, 1),
            }
        );
    }

    #[test]
    fn plot_axes_share_one_scale() {
        // Act
        let shapes = plot("0 0\n100 40");

        // Assert
        let (xs, ys) = (x_ticks(&shapes), y_ticks(&shapes));
        let gap = xs[1] - xs[0];
        for pair in xs.windows(2) {
            assert!(approx_eq(pair[1] - pair[0], gap, 1e-2));
        }
        for pair in ys.windows(2) {
            assert!(approx_eq(pair[0] - pair[1], gap, 1e-2), "{ys:?}");
        }
    }

    #[test]
    fn ticks_are_numbered_with_their_values() {
        // Act
        let shapes = plot("0 0\n10 5");

        // Assert
        let found = numbers(&shapes);
        let texts: Vec<&str> = found.iter().map(|(text, _)| *text).collect();
        assert_eq!(texts, ["0", "2", "4", "6", "8", "10", "2", "4", "6"]);
        let (x_axis, y_axis) = plot_axes(&shapes);
        for ((_, bounds), x) in found[..6].iter().zip(x_ticks(&shapes)) {
            assert!(approx_eq(bounds.center().x, x, 1e-2));
            assert!(bounds.min.y > x_axis.0.y);
        }
        // The `y` numbers skip the zero, the lowest tick.
        for ((_, bounds), y) in found[6..].iter().zip(&y_ticks(&shapes)[1..]) {
            assert!(approx_eq(bounds.center().y, *y, 1e-2));
            assert!(bounds.max.x < y_axis.0.x);
        }
    }

    #[test]
    fn tick_labels_are_exact_decimals() {
        for (tick, k, wanted) in [
            (step(1, -1), 3, "0.3"),
            (step(5, -1), 3, "1.5"),
            (step(2, -1), 5, "1"),
            (step(5, -2), -3, "-0.15"),
            (step(1, -1), -1, "-0.1"),
            (step(2, 1), 3, "60"),
            (step(1, 0), 0, "0"),
            (step(5, -1), 0, "0"),
            (step(1, 6), 2, "2000000"),
            (step(1, -6), 15, "0.000015"),
        ] {
            assert_eq!(tick.label(k), wanted, "{tick:?} {k}");
        }
    }

    #[test]
    fn tick_labels_use_an_exponent_when_long() {
        assert_eq!(step(5, 7).label(3), "15e7");
        assert_eq!(step(1, -9).label(15), "15e-9");
        assert_eq!(step(1, 300).label(-2), "-2e300");
        assert_eq!(step(1, 300).label(0), "0");
    }

    #[test]
    fn long_numbers_widen_the_ticks() {
        // Act
        let long = x_ticks(&plot("0 0\n8000000 1"));
        let short = x_ticks(&plot("0 0\n8 1"));

        // Assert
        assert!(approx_eq(short[1] - short[0], 3.0 * SIZE, 1e-2));
        assert!(long[1] - long[0] > short[1] - short[0]);
    }

    // ---- T93 AC-3 ---------------------------------------------------------

    #[test]
    fn axes_cross_at_the_origin_when_it_is_in_range() {
        // Act: the third point is the origin.
        let shapes = plot("-4 -2\n6 8\n0 0");

        // Assert
        let (x_axis, y_axis) = plot_axes(&shapes);
        let origin = dots(&shapes)[2];
        assert!(approx_eq(x_axis.0.y, origin.y, 1e-2));
        assert!(approx_eq(y_axis.0.x, origin.x, 1e-2));
        let (xs, ys) = (x_ticks(&shapes), y_ticks(&shapes));
        assert!(approx_eq(x_axis.0.x, xs[0], 1e-2));
        assert!(approx_eq(x_axis.1.x, xs[xs.len() - 1], 1e-2));
        assert!(approx_eq(y_axis.0.y.max(y_axis.1.y), ys[0], 1e-2));
        assert!(approx_eq(
            y_axis.0.y.min(y_axis.1.y),
            ys[ys.len() - 1],
            1e-2
        ));
    }

    #[test]
    fn axes_sit_on_the_edge_nearest_the_origin() {
        for (text, corner) in [
            // All positive: the first point is the bottom-left corner.
            ("3 4\n9 8", 0),
            // All negative: the second point is the top-right corner.
            ("-9 -8\n-3 -4", 1),
        ] {
            // Act
            let shapes = plot(text);

            // Assert
            let (x_axis, y_axis) = plot_axes(&shapes);
            let at = dots(&shapes)[corner];
            assert!(approx_eq(x_axis.0.y, at.y, 1e-2), "{text}");
            assert!(approx_eq(y_axis.0.x, at.x, 1e-2), "{text}");
        }
    }

    #[test]
    fn axes_treat_each_direction_on_its_own() {
        // Act: x runs through 0, every y is positive.
        let shapes = plot("-3 4\n3 9");

        // Assert
        let (x_axis, y_axis) = plot_axes(&shapes);
        let centres = dots(&shapes);
        assert!(approx_eq(x_axis.0.y, centres[0].y, 1e-2));
        assert!(approx_eq(
            y_axis.0.x,
            centres[0].lerp(centres[1], 0.5).x,
            1e-2
        ));
    }

    #[test]
    fn origin_is_numbered_once() {
        for (text, zeros) in [("-4 -2\n6 8", 1), ("3 4\n9 8", 0), ("-3 4\n3 9", 1)] {
            // Act
            let shapes = plot(text);

            // Assert
            let found = numbers(&shapes);
            let count = found.iter().filter(|(number, _)| *number == "0").count();
            assert_eq!(count, zeros, "{text}");
        }
    }

    // ---- T93 AC-4 ---------------------------------------------------------

    #[test]
    fn parse_reads_integers_decimals_and_negatives() {
        assert_eq!(
            points("1 2\n-3.5 4.25\r\n\n  0\t-7  ").points(),
            [(1.0, 2.0), (-3.5, 4.25), (0.0, -7.0)]
        );
    }

    #[test]
    fn parse_blank_text_is_no_plot() {
        for text in ["", "  \n\t\n "] {
            assert_eq!(PointList::parse(text), Ok(None));
        }
    }

    #[test]
    fn parse_refuses_what_is_not_a_point() {
        for text in [
            "a b",
            "1",
            "1 2 3",
            "1,2",
            "1 x",
            "inf 2",
            "1 -inf",
            "NaN 1",
            "1 infinity",
            "1e999 0",
            "0 -1e999",
            "1e301 0",
            "$x$ 1",
        ] {
            assert_eq!(
                PointList::parse(text),
                Err(PlotError::NotAPoint { line: 1 }),
                "{text}"
            );
        }
    }

    #[test]
    fn parse_refuses_too_much_plot_text() {
        // Arrange
        let text = "1 2\n".repeat(PASTE_MAX_BYTES / 4 + 1);

        // Act / Assert
        assert_eq!(
            PointList::parse(&text),
            Err(PlotError::TooLong { bytes: text.len() })
        );
    }

    #[test]
    fn plot_error_names_the_line_not_its_text() {
        // Act
        let err = PointList::parse("1 2\n\nsecret");

        // Assert
        assert_eq!(err, Err(PlotError::NotAPoint { line: 3 }));
        let reason = PlotError::NotAPoint { line: 3 }.to_string();
        assert!(reason.contains('3') && !reason.contains("secret"));
    }

    // ---- T93 AC-5 ---------------------------------------------------------

    fn many_points(n: usize) -> String {
        (0..n)
            .map(|i| format!("{i} {}", 2 * i))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn parse_accepts_1000_points() {
        assert_eq!(points(&many_points(1000)).points().len(), MAX_POINTS);
    }

    #[test]
    fn parse_refuses_more_than_1000_points() {
        assert_eq!(
            PointList::parse(&many_points(1001)),
            Err(PlotError::TooManyPoints { points: 1001 })
        );
    }

    #[test]
    fn single_point_gives_a_valid_plot() {
        for text in ["3 4", "0 0", "-1000 2.5", "0.001 0.001"] {
            assert_valid_plot(&plot(text), 1);
        }
        assert_eq!(
            points("3 4").frame(),
            Frame {
                step: step(1, 0),
                x: (2, 4),
                y: (3, 5),
            }
        );
    }

    #[test]
    fn equal_coordinates_give_a_valid_plot() {
        assert_valid_plot(&plot("5 5\n5 5\n5 5"), 3);
    }

    #[test]
    fn one_flat_axis_gives_a_valid_plot() {
        // Act
        let shapes = plot("0 200\n1000 200");

        // Assert: the flat axis runs one step either side of the value.
        assert_valid_plot(&shapes, 2);
        assert_eq!(
            points("0 200\n1000 200").frame(),
            Frame {
                step: step(2, 2),
                x: (0, 5),
                y: (0, 2),
            }
        );
    }

    #[test]
    fn extreme_coordinates_give_a_valid_plot() {
        for text in [
            "1e300 -1e300",
            "1e300 1\n-1e300 1",
            "1e-300 2e-300",
            "5e-324 0",
            "1e15 0\n1000000000000001 1",
        ] {
            assert_valid_plot(&plot(text), text.lines().count());
        }
    }

    proptest! {
        #[test]
        fn plot_holds_for_any_points(
            coordinates in prop::collection::vec(
                (
                    prop_oneof![
                        -1e6_f64..1e6,
                        (-20_i32..20).prop_map(f64::from),
                        Just(1e300),
                        Just(-1e300),
                        Just(1e-300),
                    ],
                    prop_oneof![
                        -1e6_f64..1e6,
                        (-20_i32..20).prop_map(f64::from),
                        Just(7.0),
                    ],
                ),
                1..60,
            ),
        ) {
            // Arrange
            let text = coordinates
                .iter()
                .map(|(x, y)| format!("{x} {y}"))
                .collect::<Vec<_>>()
                .join("\n");
            let list = points(&text);

            // Act
            let frame = list.frame();

            // Assert: never many ticks, never a flat axis.
            prop_assert!(frame.x.0 < frame.x.1 && frame.x.1 - frame.x.0 <= 12);
            prop_assert!(frame.y.0 < frame.y.1 && frame.y.1 - frame.y.0 <= 12);
            assert_valid_plot(&plot(&text), coordinates.len());
        }

        #[test]
        fn parse_and_plot_hold_for_any_text(
            text in prop_oneof![
                any::<String>(),
                "[0-9 .\\-\ne]{0,80}",
            ],
            size in 0.01_f32..5000.0,
            width in 0.0_f32..100.0,
            x in -1e6_f32..1e6,
        ) {
            // Act: must never panic.
            let parsed = PointList::parse(&text);

            // Assert
            if let Ok(Some(list)) = parsed {
                prop_assert!(list.points().len() <= MAX_POINTS);
                let style = Style { width, ..STYLE };
                if let Some(shapes) = list.shapes(Vec2::new(x, 0.0), size, style) {
                    prop_assert!(shapes.iter().all(Shape::is_finite));
                }
            }
        }
    }
}
