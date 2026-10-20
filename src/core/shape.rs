//! Vector shapes with bounds, hit-testing and translation.
//!
//! The document is an ordered list of [`Shape`]s in world coordinates
//! (ADR-0004, amended by ADR-T16-1 with grids and labels, by ADR-T29-1
//! with text blocks and by ADR-T31-1 with text in shapes and grid cells). Widths are world units (ADR-0013). Every query here is total:
//! non-finite input never panics, a non-finite query point never hits and is
//! never contained.

use crate::core::geom::{Aabb, Vec2, distance_to_segment};
use crate::core::palette::ColorId;
use crate::core::text_layout;

/// Arrow head length per unit of stroke width. The head has no absolute
/// minimum, so it keeps its proportion to the shaft at any zoom.
pub const ARROW_HEAD_LENGTH_PER_WIDTH: f32 = 4.0;

/// Half the width of the arrow head base, as a fraction of the head length.
pub const ARROW_HEAD_HALF_WIDTH_RATIO: f32 = 0.5;

/// Most columns or rows a [`Shape::Grid`] can have.
pub const GRID_MAX_CELLS: u32 = 64;

/// Fraction of a shape's or cell's box (an ellipse's inscribed rectangle)
/// that its text may use, per axis, about the centre (ADR-T31-1).
pub const SHAPE_TEXT_FILL: f32 = 0.8;

/// Outline style shared by every shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Outline colour.
    pub color: ColorId,
    /// Outline width in world units.
    pub width: f32,
}

/// A vector shape in world coordinates.
///
/// For [`Shape::Rect`], [`Shape::Ellipse`] and [`Shape::Grid`], `a` and `b`
/// are opposite corners of the bounding box, in any order.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Freehand polyline.
    Stroke {
        /// Polyline vertices in drawing order.
        points: Vec<Vec2>,
        /// Width factor of each point, multiplying `style.width`; `None`
        /// for a stroke drawn without pressure (ADR-T35-2).
        widths: Option<Vec<f32>>,
        /// Outline style.
        style: Style,
    },
    /// Straight segment from `a` to `b`.
    Line {
        /// Start point.
        a: Vec2,
        /// End point.
        b: Vec2,
        /// Outline style.
        style: Style,
    },
    /// Segment from `a` to `b` with a filled head at `b`.
    Arrow {
        /// Tail.
        a: Vec2,
        /// Tip (where the head is drawn).
        b: Vec2,
        /// Outline style.
        style: Style,
    },
    /// Axis-aligned rectangle.
    Rect {
        /// One corner.
        a: Vec2,
        /// The opposite corner.
        b: Vec2,
        /// Outline style.
        style: Style,
        /// Interior colour, if filled.
        fill: Option<ColorId>,
        /// Number drawn centred inside, if any.
        label: Option<u32>,
        /// Text drawn centred inside instead of the label (ADR-T31-1).
        text: Option<ShapeText>,
    },
    /// Axis-aligned ellipse inscribed in the box spanned by `a` and `b`.
    Ellipse {
        /// One corner of the bounding box.
        a: Vec2,
        /// The opposite corner of the bounding box.
        b: Vec2,
        /// Outline style.
        style: Style,
        /// Interior colour, if filled.
        fill: Option<ColorId>,
        /// Number drawn centred inside, if any.
        label: Option<u32>,
        /// Text drawn centred inside instead of the label (ADR-T31-1).
        text: Option<ShapeText>,
    },
    /// `cols × rows` uniform cells in the box spanned by `a` and `b`.
    ///
    /// An open shape: it has no interior and is never filled.
    Grid {
        /// One corner.
        a: Vec2,
        /// The opposite corner.
        b: Vec2,
        /// Number of columns, in `1..=GRID_MAX_CELLS`.
        cols: u32,
        /// Number of rows, in `1..=GRID_MAX_CELLS`.
        rows: u32,
        /// Outline style.
        style: Style,
        /// Draws 0-based column and row indices outside the grid, starting
        /// at corner `a` (ADR-T18-3).
        axes: bool,
        /// Filled cells, sorted by `(row, col)`, at most one per cell, all
        /// inside `cols × rows` (ADR-T21-1).
        fills: Vec<CellFill>,
        /// Cell texts, sorted by `(row, col)`, at most one per cell, all
        /// inside `cols × rows` (ADR-T31-1).
        texts: Vec<CellText>,
    },
    /// A block of text, laid out on a monospaced grid by
    /// [`text_layout`] (ADR-T29-1).
    ///
    /// An open shape: it is never filled, and it is hit anywhere inside its
    /// box.
    Text {
        /// Top-left corner of the block.
        origin: Vec2,
        /// The text; lines are separated by `\n`.
        text: String,
        /// Font size (em height) in world units.
        size: f32,
        /// `color` is the text colour; `width` is unused (0).
        style: Style,
    },
}

/// The fill of one grid cell; cells are counted from corner `a` toward `b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellFill {
    /// 0-based column.
    pub col: u32,
    /// 0-based row.
    pub row: u32,
    /// Fill colour.
    pub color: ColorId,
}

/// Text inside a rectangle, an ellipse or a grid cell (ADR-T31-1): never
/// blank, drawn centred in the shape's text frame at `size`, or smaller
/// when it does not fit.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeText {
    /// The text: lines split on `\n`, `$…$` formulas typeset.
    pub text: String,
    /// Largest font size (em height) in world units; finite and positive.
    pub size: f32,
}

impl ShapeText {
    /// `text` at `size`; `None` if `text` is blank (empty or whitespace)
    /// or `size` is not finite and positive.
    #[must_use]
    pub fn new(text: String, size: f32) -> Option<Self> {
        let sized = size.is_finite() && size > 0.0;
        (sized && !text.trim().is_empty()).then_some(Self { text, size })
    }
}

/// The text of one grid cell; cells are counted from corner `a` toward
/// `b`.
#[derive(Debug, Clone, PartialEq)]
pub struct CellText {
    /// 0-based column.
    pub col: u32,
    /// 0-based row.
    pub row: u32,
    /// The text.
    pub content: ShapeText,
}

/// Where a shape holds text: its own interior or one grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextSlot {
    /// The interior of a rectangle or an ellipse.
    Shape,
    /// Cell `(col, row)` of a grid.
    Cell {
        /// 0-based column.
        col: u32,
        /// 0-based row.
        row: u32,
    },
}

/// A move between the cells of a grid, as seen on screen (ADR-T31-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellStep {
    /// The next cell in reading order (left to right, top to bottom),
    /// wrapping from the last cell to the first.
    Next,
    /// The previous cell in reading order, wrapping from the first cell to
    /// the last.
    Prev,
    /// The cell below; none from the bottom row.
    Down,
    /// The cell above; none from the top row.
    Up,
}

/// Text of a shape placed for drawing: a text block at `origin` and `size`
/// fitted into its frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedText<'a> {
    /// Top-left corner of the laid-out block, world units.
    pub origin: Vec2,
    /// The text.
    pub text: &'a str,
    /// Font size it is drawn at, world units.
    pub size: f32,
}

/// One axis index of a grid with [`Shape::Grid::axes`]: the 0-based column
/// or row `index` and the world box it is drawn centred in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisLabel {
    /// 0-based column or row index.
    pub index: u32,
    /// World box outside the grid, one cell in size.
    pub rect: Aabb,
}

impl Shape {
    /// The outline style.
    #[must_use]
    pub fn style(&self) -> Style {
        match self {
            Self::Stroke { style, .. }
            | Self::Line { style, .. }
            | Self::Arrow { style, .. }
            | Self::Rect { style, .. }
            | Self::Ellipse { style, .. }
            | Self::Grid { style, .. }
            | Self::Text { style, .. } => *style,
        }
    }

    /// The fill colour; always `None` for open shapes.
    #[must_use]
    pub fn fill(&self) -> Option<ColorId> {
        match self {
            Self::Rect { fill, .. } | Self::Ellipse { fill, .. } => *fill,
            Self::Stroke { .. }
            | Self::Line { .. }
            | Self::Arrow { .. }
            | Self::Grid { .. }
            | Self::Text { .. } => None,
        }
    }

    /// The label; always `None` for shapes other than [`Shape::Rect`] and
    /// [`Shape::Ellipse`].
    #[must_use]
    pub fn label(&self) -> Option<u32> {
        match self {
            Self::Rect { label, .. } | Self::Ellipse { label, .. } => *label,
            Self::Stroke { .. }
            | Self::Line { .. }
            | Self::Arrow { .. }
            | Self::Grid { .. }
            | Self::Text { .. } => None,
        }
    }

    /// The largest width factor of a stroke with widths; `1` for every
    /// other shape (ADR-T35-2).
    #[must_use]
    pub fn max_width_factor(&self) -> f32 {
        let Self::Stroke {
            widths: Some(widths),
            ..
        } = self
        else {
            return 1.0;
        };
        let max = widths.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        if max.is_finite() && max > 0.0 {
            max
        } else {
            1.0
        }
    }

    /// `true` for shapes with an interior ([`Shape::Rect`], [`Shape::Ellipse`]).
    #[must_use]
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Rect { .. } | Self::Ellipse { .. })
    }

    /// Bounding box including half the outline width (and the arrow head).
    /// A text block's box is its layout box (ADR-T29-1).
    ///
    /// A stroke with no finite point gives the zero-size box at the origin.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        let geometry = match self {
            Self::Text {
                origin, text, size, ..
            } => return text_layout::bounds(*origin, text, *size),
            Self::Stroke { points, .. } => match Aabb::from_points(points) {
                Some(b) => b,
                None => return Aabb::from_corners(Vec2::ZERO, Vec2::ZERO),
            },
            Self::Arrow { a, b, style } => {
                let [tip, left, right] = arrow_head(*a, *b, style.width);
                Aabb::from_corners(*a, *b)
                    .union(&Aabb::from_corners(left, right))
                    .union(&Aabb::from_corners(tip, tip))
            }
            Self::Line { a, b, .. }
            | Self::Rect { a, b, .. }
            | Self::Ellipse { a, b, .. }
            | Self::Grid {
                a, b, axes: false, ..
            } => Aabb::from_corners(*a, *b),
            // The index bands lie one cell beyond `a`, away from `b`.
            Self::Grid {
                a,
                b,
                cols,
                rows,
                axes: true,
                ..
            } => {
                let (cols, rows) = (clamp_cells(*cols) as f32, clamp_cells(*rows) as f32);
                let cell = Vec2::new((b.x - a.x) / cols, (b.y - a.y) / rows);
                Aabb::from_corners(*a, *b).union(&Aabb::from_corners(*a - cell, *a - cell))
            }
        };
        geometry.expand(half_width(self.style().width * self.max_width_factor()))
    }

    /// `true` if `p` is within `tol + width / 2` of the outline, or inside a
    /// filled closed shape.
    ///
    /// Ellipse outlines use the gradient-normalised distance estimate, exact on
    /// the axes and on the outline itself.
    #[must_use]
    pub fn hit(&self, p: Vec2, tol: f32) -> bool {
        self.hit_outline(p, tol) || (self.fill().is_some() && self.contains(p))
    }

    /// [`Shape::hit`] without the filled-interior rule: only the outline
    /// counts.
    #[must_use]
    pub fn hit_outline(&self, p: Vec2, tol: f32) -> bool {
        let reach = tol + half_width(self.style().width * self.max_width_factor());
        if !p.is_finite() || !reach.is_finite() || !self.bounds().expand(tol).contains(p) {
            return false;
        }
        match self {
            Self::Stroke { points, .. } => match points.as_slice() {
                [] => false,
                [only] => p.distance(*only) <= reach,
                _ => points
                    .windows(2)
                    .any(|w| distance_to_segment(p, w[0], w[1]) <= reach),
            },
            Self::Line { a, b, .. } => distance_to_segment(p, *a, *b) <= reach,
            Self::Arrow { a, b, style } => {
                let head = arrow_head(*a, *b, style.width);
                distance_to_segment(p, *a, *b) <= reach
                    || point_in_triangle(p, head)
                    || polygon_edge_distance(p, &head) <= reach
            }
            Self::Rect { a, b, .. } => {
                let r = Aabb::from_corners(*a, *b);
                let corners = [
                    r.min,
                    Vec2::new(r.max.x, r.min.y),
                    r.max,
                    Vec2::new(r.min.x, r.max.y),
                ];
                polygon_edge_distance(p, &corners) <= reach
            }
            Self::Ellipse { a, b, .. } => ellipse_outline_distance(p, *a, *b) <= reach,
            Self::Grid {
                a, b, cols, rows, ..
            } => grid_lines(*a, *b, *cols, *rows)
                .any(|[start, end]| distance_to_segment(p, start, end) <= reach),
            // Already inside the box grown by `tol`: text is solid.
            Self::Text { .. } => true,
        }
    }

    /// `true` if `p` lies inside the geometry of a closed shape (boundary
    /// inclusive, outline width and fill ignored). Open shapes contain nothing.
    #[must_use]
    pub fn contains(&self, p: Vec2) -> bool {
        if !p.is_finite() {
            return false;
        }
        match self {
            Self::Rect { a, b, .. } => Aabb::from_corners(*a, *b).contains(p),
            Self::Ellipse { a, b, .. } => {
                let (center, radii) = ellipse_frame(*a, *b);
                if radii.x <= 0.0 || radii.y <= 0.0 {
                    return false;
                }
                let q = p - center;
                let (u, w) = (q.x / radii.x, q.y / radii.y);
                u * u + w * w <= 1.0
            }
            Self::Stroke { .. }
            | Self::Line { .. }
            | Self::Arrow { .. }
            | Self::Grid { .. }
            | Self::Text { .. } => false,
        }
    }

    /// Moves every point by `delta`.
    pub fn translate(&mut self, delta: Vec2) {
        match self {
            Self::Stroke { points, .. } => {
                for point in points {
                    *point += delta;
                }
            }
            Self::Line { a, b, .. }
            | Self::Arrow { a, b, .. }
            | Self::Rect { a, b, .. }
            | Self::Ellipse { a, b, .. }
            | Self::Grid { a, b, .. } => {
                *a += delta;
                *b += delta;
            }
            Self::Text { origin, .. } => *origin += delta,
        }
    }

    /// Returns the shape with its fill set to `fill`; open shapes are
    /// returned unchanged.
    #[must_use]
    pub fn with_fill(mut self, fill: Option<ColorId>) -> Self {
        if let Self::Rect { fill: f, .. } | Self::Ellipse { fill: f, .. } = &mut self {
            *f = fill;
        }
        self
    }

    /// Returns the shape with its label set to `label`; shapes other than
    /// [`Shape::Rect`] and [`Shape::Ellipse`] are returned unchanged.
    #[must_use]
    pub fn with_label(mut self, label: Option<u32>) -> Self {
        if let Self::Rect { label: l, .. } | Self::Ellipse { label: l, .. } = &mut self {
            *l = label;
        }
        self
    }

    /// The cell `(col, row)` of a grid that `p` lies in (box boundary
    /// inclusive), counted from `a`; `None` outside, for a non-finite `p` or
    /// for other shapes.
    #[must_use]
    pub fn grid_cell_at(&self, p: Vec2) -> Option<(u32, u32)> {
        let Self::Grid {
            a, b, cols, rows, ..
        } = *self
        else {
            return None;
        };
        if !p.is_finite() || !Aabb::from_corners(a, b).contains(p) {
            return None;
        }
        // Fraction of the way from `a` to `b`, in 0..=1 inside the box.
        // A zero-size side gives 0 / 0: treat it as the first cell.
        let along = |p: f32, a: f32, b: f32| {
            let t = (p - a) / (b - a);
            if t.is_finite() { t } else { 0.0 }
        };
        let index = |t: f32, n: u32| ((t * n as f32) as u32).min(n - 1);
        let (cols, rows) = (clamp_cells(cols), clamp_cells(rows));
        Some((
            index(along(p.x, a.x, b.x), cols),
            index(along(p.y, a.y, b.y), rows),
        ))
    }

    /// The cell fills of a grid, sorted by `(row, col)`; empty for other
    /// shapes.
    #[must_use]
    pub fn cell_fills(&self) -> &[CellFill] {
        match self {
            Self::Grid { fills, .. } => fills,
            _ => &[],
        }
    }

    /// The fill colour of cell `(col, row)` of a grid, if any.
    #[must_use]
    pub fn cell_fill(&self, col: u32, row: u32) -> Option<ColorId> {
        let fills = self.cell_fills();
        fills
            .binary_search_by_key(&(row, col), |f| (f.row, f.col))
            .ok()
            .map(|i| fills[i].color)
    }

    /// Returns the shape with cell `(col, row)` filled with `fill`, or
    /// cleared for `None`. Other shapes and cells outside the grid are
    /// returned unchanged.
    #[must_use]
    pub fn with_cell_fill(mut self, col: u32, row: u32, fill: Option<ColorId>) -> Self {
        let Self::Grid {
            cols, rows, fills, ..
        } = &mut self
        else {
            return self;
        };
        if col >= clamp_cells(*cols) || row >= clamp_cells(*rows) {
            return self;
        }
        match (
            fills.binary_search_by_key(&(row, col), |f| (f.row, f.col)),
            fill,
        ) {
            (Ok(i), Some(color)) => fills[i].color = color,
            (Ok(i), None) => {
                fills.remove(i);
            }
            (Err(i), Some(color)) => fills.insert(i, CellFill { col, row, color }),
            (Err(_), None) => {}
        }
        self
    }

    /// The slot of this shape that holds text at world point `p`: the
    /// interior of a rectangle or an ellipse, or the grid cell under `p`;
    /// `None` elsewhere and for other shapes.
    #[must_use]
    pub fn text_slot_at(&self, p: Vec2) -> Option<TextSlot> {
        match self {
            Self::Rect { .. } | Self::Ellipse { .. } => self.contains(p).then_some(TextSlot::Shape),
            Self::Grid { .. } => self
                .grid_cell_at(p)
                .map(|(col, row)| TextSlot::Cell { col, row }),
            _ => None,
        }
    }

    /// The text in `slot`, if any.
    #[must_use]
    pub fn slot_text(&self, slot: TextSlot) -> Option<&ShapeText> {
        match (self, slot) {
            (Self::Rect { text, .. } | Self::Ellipse { text, .. }, TextSlot::Shape) => {
                text.as_ref()
            }
            (Self::Grid { texts, .. }, TextSlot::Cell { col, row }) => texts
                .binary_search_by_key(&(row, col), |t| (t.row, t.col))
                .ok()
                .map(|i| &texts[i].content),
            _ => None,
        }
    }

    /// Returns the shape with the text in `slot` set to `text`, or cleared
    /// for `None`. A slot the shape does not have (a cell outside the grid,
    /// a cell of a rectangle, the interior of a grid) leaves it unchanged.
    #[must_use]
    pub fn with_slot_text(mut self, slot: TextSlot, text: Option<ShapeText>) -> Self {
        match (&mut self, slot) {
            (Self::Rect { text: t, .. } | Self::Ellipse { text: t, .. }, TextSlot::Shape) => {
                *t = text;
            }
            (
                Self::Grid {
                    cols, rows, texts, ..
                },
                TextSlot::Cell { col, row },
            ) if col < clamp_cells(*cols) && row < clamp_cells(*rows) => {
                match (
                    texts.binary_search_by_key(&(row, col), |t| (t.row, t.col)),
                    text,
                ) {
                    (Ok(i), Some(content)) => texts[i].content = content,
                    (Ok(i), None) => {
                        texts.remove(i);
                    }
                    (Err(i), Some(content)) => texts.insert(i, CellText { col, row, content }),
                    (Err(_), None) => {}
                }
            }
            _ => {}
        }
        self
    }

    /// The cell texts of a grid, sorted by `(row, col)`; empty for other
    /// shapes.
    #[must_use]
    pub fn cell_texts(&self) -> &[CellText] {
        match self {
            Self::Grid { texts, .. } => texts,
            _ => &[],
        }
    }

    /// The world box the text in `slot` is fitted into: the box of a
    /// rectangle, the inscribed rectangle of an ellipse or the grid cell,
    /// scaled by [`SHAPE_TEXT_FILL`] about its centre. `None` for a slot
    /// the shape does not have.
    #[must_use]
    pub fn text_frame(&self, slot: TextSlot) -> Option<Aabb> {
        let area = match (self, slot) {
            (Self::Rect { a, b, .. }, TextSlot::Shape) => Aabb::from_corners(*a, *b),
            (Self::Ellipse { a, b, .. }, TextSlot::Shape) => {
                let (center, radii) = ellipse_frame(*a, *b);
                let half = radii * std::f32::consts::FRAC_1_SQRT_2;
                Aabb::from_corners(center - half, center + half)
            }
            (
                Self::Grid {
                    a, b, cols, rows, ..
                },
                TextSlot::Cell { col, row },
            ) if col < clamp_cells(*cols) && row < clamp_cells(*rows) => {
                grid_cell_rect(*a, *b, *cols, *rows, col, row)
            }
            _ => return None,
        };
        let half = Vec2::new(area.width(), area.height()) * (0.5 * SHAPE_TEXT_FILL);
        let center = area.center();
        Some(Aabb::from_corners(center - half, center + half))
    }

    /// Every text of the shape placed for drawing, fitted and centred in
    /// its frame: the shape's own text, then cell texts in `(row, col)`
    /// order.
    #[must_use]
    pub fn placed_texts(&self) -> Vec<PlacedText<'_>> {
        let own = self
            .slot_text(TextSlot::Shape)
            .map(|t| (TextSlot::Shape, t));
        let cells = self.cell_texts().iter().map(|c| {
            (
                TextSlot::Cell {
                    col: c.col,
                    row: c.row,
                },
                &c.content,
            )
        });
        own.into_iter()
            .chain(cells)
            .filter_map(|(slot, content)| {
                let frame = self.text_frame(slot)?;
                let (origin, size) = text_layout::fit(&content.text, frame, content.size)?;
                Some(PlacedText {
                    origin,
                    text: &content.text,
                    size,
                })
            })
            .collect()
    }

    /// The cell one `step` away from cell `(col, row)` of a grid, in screen
    /// order whatever the corner `a`; `None` when there is none, for a cell
    /// outside the grid or for other shapes.
    #[must_use]
    pub fn grid_cell_step(&self, col: u32, row: u32, step: CellStep) -> Option<(u32, u32)> {
        let Self::Grid {
            a, b, cols, rows, ..
        } = *self
        else {
            return None;
        };
        let (cols, rows) = (clamp_cells(cols), clamp_cells(rows));
        if col >= cols || row >= rows {
            return None;
        }
        // Screen column and row: counted from the left and the top. The
        // mapping is its own inverse.
        let x = |c: u32| if a.x <= b.x { c } else { cols - 1 - c };
        let y = |r: u32| if a.y <= b.y { r } else { rows - 1 - r };
        let (sx, sy) = (x(col), y(row));
        let (sx, sy) = match step {
            CellStep::Next | CellStep::Prev => {
                let n = cols * rows;
                let index = sy * cols + sx;
                let index = if step == CellStep::Next {
                    (index + 1) % n
                } else {
                    (index + n - 1) % n
                };
                (index % cols, index / cols)
            }
            CellStep::Down => (sx, (sy + 1 < rows).then_some(sy + 1)?),
            CellStep::Up => (sx, sy.checked_sub(1)?),
        };
        Some((x(sx), y(sy)))
    }

    /// The axis indices to draw: those of a grid with `axes` on (see
    /// [`grid_axis_labels`]), empty for every other shape.
    #[must_use]
    pub fn axis_labels(&self) -> Vec<AxisLabel> {
        match *self {
            Self::Grid {
                a,
                b,
                cols,
                rows,
                axes: true,
                ..
            } => grid_axis_labels(a, b, cols, rows).collect(),
            _ => Vec::new(),
        }
    }

    /// `true` iff every coordinate and the width are finite.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.style().width.is_finite()
            && match self {
                Self::Stroke { points, widths, .. } => {
                    points.iter().all(|p| p.is_finite())
                        && widths.as_ref().is_none_or(|widths| {
                            widths.len() == points.len()
                                && widths.iter().all(|w| w.is_finite() && *w > 0.0)
                        })
                }
                Self::Line { a, b, .. }
                | Self::Arrow { a, b, .. }
                | Self::Rect { a, b, .. }
                | Self::Ellipse { a, b, .. }
                | Self::Grid { a, b, .. } => a.is_finite() && b.is_finite(),
                Self::Text { origin, size, .. } => origin.is_finite() && size.is_finite(),
            }
    }
}

/// The lines of a `cols × rows` grid in the box `a`–`b`: `cols + 1`
/// vertical lines left to right, then `rows + 1` horizontal lines top to
/// bottom, each as `[start, end]`. Dimensions are clamped to
/// `1..=GRID_MAX_CELLS`, so at most `2 * (GRID_MAX_CELLS + 1)` lines.
pub fn grid_lines(a: Vec2, b: Vec2, cols: u32, rows: u32) -> impl Iterator<Item = [Vec2; 2]> {
    let r = Aabb::from_corners(a, b);
    let (cols, rows) = (clamp_cells(cols), clamp_cells(rows));
    let (dx, dy) = (r.width() / cols as f32, r.height() / rows as f32);
    // The last line sits exactly on the far edge, free of rounding drift.
    let at = |min: f32, max: f32, step: f32, n: u32, i: u32| {
        if i == n { max } else { min + step * i as f32 }
    };
    let vertical = (0..=cols).map(move |i| {
        let x = at(r.min.x, r.max.x, dx, cols, i);
        [Vec2::new(x, r.min.y), Vec2::new(x, r.max.y)]
    });
    let horizontal = (0..=rows).map(move |j| {
        let y = at(r.min.y, r.max.y, dy, rows, j);
        [Vec2::new(r.min.x, y), Vec2::new(r.max.x, y)]
    });
    vertical.chain(horizontal)
}

/// The axis indices of a `cols × rows` grid dragged from `a` to `b`:
/// columns `0..cols` counted from `a.x` toward `b.x`, in boxes one cell tall
/// just outside the edge `y = a.y`, then rows `0..rows` counted from `a.y`
/// toward `b.y`, in boxes one cell wide just outside `x = a.x`
/// (ADR-T18-3). Dimensions are clamped like [`grid_lines`].
pub fn grid_axis_labels(a: Vec2, b: Vec2, cols: u32, rows: u32) -> impl Iterator<Item = AxisLabel> {
    let (cols, rows) = (clamp_cells(cols), clamp_cells(rows));
    // Signed cell size: negative when the drag went left or up.
    let cell = Vec2::new((b.x - a.x) / cols as f32, (b.y - a.y) / rows as f32);
    let columns = (0..cols).map(move |i| AxisLabel {
        index: i,
        rect: Aabb::from_corners(
            Vec2::new(a.x + cell.x * i as f32, a.y),
            Vec2::new(a.x + cell.x * (i + 1) as f32, a.y - cell.y),
        ),
    });
    let rows = (0..rows).map(move |j| AxisLabel {
        index: j,
        rect: Aabb::from_corners(
            Vec2::new(a.x, a.y + cell.y * j as f32),
            Vec2::new(a.x - cell.x, a.y + cell.y * (j + 1) as f32),
        ),
    });
    columns.chain(rows)
}

/// The world box of cell `(col, row)` of a `cols × rows` grid dragged from
/// `a` to `b`, counted from `a` like [`grid_axis_labels`]. Dimensions are
/// clamped like [`grid_lines`].
#[must_use]
pub fn grid_cell_rect(a: Vec2, b: Vec2, cols: u32, rows: u32, col: u32, row: u32) -> Aabb {
    let (cols, rows) = (clamp_cells(cols), clamp_cells(rows));
    // Signed cell size: negative when the drag went left or up.
    let cell = Vec2::new((b.x - a.x) / cols as f32, (b.y - a.y) / rows as f32);
    let at = |col: u32, row: u32| a + Vec2::new(cell.x * col as f32, cell.y * row as f32);
    Aabb::from_corners(at(col, row), at(col + 1, row + 1))
}

/// `n` clamped to the valid grid dimension range `1..=GRID_MAX_CELLS`.
fn clamp_cells(n: u32) -> u32 {
    n.clamp(1, GRID_MAX_CELLS)
}

/// Arrow head triangle `[tip, left, right]` for an arrow from `a` to `b`.
///
/// The tip is `b`; the base lies `ARROW_HEAD_LENGTH_PER_WIDTH * width` back
/// along the shaft (negative or NaN widths count as 0) and is
/// `2 * ARROW_HEAD_HALF_WIDTH_RATIO` times that length wide. "Left" is the
/// side reached by turning the shaft direction a quarter turn towards +y.
/// A degenerate arrow (`a` ≈ `b`) gives `[b, b, b]`.
#[must_use]
pub fn arrow_head(a: Vec2, b: Vec2, width: f32) -> [Vec2; 3] {
    let shaft = b - a;
    let len = shaft.length();
    if len.is_nan() || len <= f32::EPSILON {
        return [b, b, b];
    }
    let dir = shaft / len;
    let head_len = ARROW_HEAD_LENGTH_PER_WIDTH * width.max(0.0);
    let base = b - dir * head_len;
    let side = Vec2::new(-dir.y, dir.x) * (ARROW_HEAD_HALF_WIDTH_RATIO * head_len);
    [b, base + side, base - side]
}

/// Half of `width`, treating negative and NaN widths as 0.
fn half_width(width: f32) -> f32 {
    width.max(0.0) * 0.5
}

/// Centre and (non-negative) radii of the ellipse inscribed in the box `a`–`b`.
fn ellipse_frame(a: Vec2, b: Vec2) -> (Vec2, Vec2) {
    let r = Aabb::from_corners(a, b);
    (r.center(), Vec2::new(r.width() * 0.5, r.height() * 0.5))
}

/// Approximate distance from `p` to the outline of the ellipse in box `a`–`b`.
///
/// Uses `k0 (k0 − 1) / k1` with `k0 = |q / r|`, `k1 = |q / r²|`: the implicit
/// function divided by its gradient length. Exact on the axes and zero on the
/// outline. A zero radius collapses the ellipse to a segment.
fn ellipse_outline_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let (center, radii) = ellipse_frame(a, b);
    if radii.x <= 0.0 || radii.y <= 0.0 {
        return distance_to_segment(p, center - radii, center + radii);
    }
    let offset = p - center;
    let k0 = Vec2::new(offset.x / radii.x, offset.y / radii.y).length();
    let k1 = Vec2::new(
        offset.x / (radii.x * radii.x),
        offset.y / (radii.y * radii.y),
    )
    .length();
    if k1.is_nan() || k1 <= 0.0 {
        // At the centre the gradient vanishes; the nearest outline point is
        // the end of the shorter axis.
        return radii.x.min(radii.y);
    }
    (k0 * (k0 - 1.0) / k1).abs()
}

/// Smallest distance from `p` to the edges of the closed polygon `vertices`.
fn polygon_edge_distance(p: Vec2, vertices: &[Vec2]) -> f32 {
    let n = vertices.len();
    (0..n)
        .map(|i| distance_to_segment(p, vertices[i], vertices[(i + 1) % n]))
        .fold(f32::INFINITY, f32::min)
}

/// `true` if `p` is inside or on the triangle, for either winding.
/// A degenerate (zero-area or non-finite) triangle contains nothing.
fn point_in_triangle(p: Vec2, [t0, t1, t2]: [Vec2; 3]) -> bool {
    let area = cross(t1 - t0, t2 - t0).abs();
    if area.is_nan() || area <= f32::EPSILON {
        return false;
    }
    let d0 = cross(t1 - t0, p - t0);
    let d1 = cross(t2 - t1, p - t1);
    let d2 = cross(t0 - t2, p - t2);
    let has_neg = d0 < 0.0 || d1 < 0.0 || d2 < 0.0;
    let has_pos = d0 > 0.0 || d1 > 0.0 || d2 > 0.0;
    !(has_neg && has_pos)
}

/// 2D cross product (z component of the 3D cross product).
fn cross(u: Vec2, w: Vec2) -> f32 {
    u.x * w.y - u.y * w.x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Aabb, Vec2, approx_eq};
    use crate::core::palette::ColorId;
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn color(index: u8) -> ColorId {
        ColorId::new(index).unwrap_or(ColorId::INK)
    }

    fn style(width: f32) -> Style {
        Style {
            color: ColorId::INK,
            width,
        }
    }

    fn assert_vec_eq(actual: Vec2, expected: Vec2) {
        assert!(
            actual.approx_eq(expected, EPS),
            "expected {expected:?}, got {actual:?}"
        );
    }

    fn assert_aabb_eq(actual: Aabb, min: Vec2, max: Vec2) {
        assert_vec_eq(actual.min, min);
        assert_vec_eq(actual.max, max);
    }

    fn rect(a: Vec2, b: Vec2, width: f32, fill: Option<ColorId>) -> Shape {
        Shape::Rect {
            a,
            b,
            style: style(width),
            fill,
            label: None,
            text: None,
        }
    }

    fn ellipse(a: Vec2, b: Vec2, width: f32, fill: Option<ColorId>) -> Shape {
        Shape::Ellipse {
            a,
            b,
            style: style(width),
            fill,
            label: None,
            text: None,
        }
    }

    fn grid(a: Vec2, b: Vec2, cols: u32, rows: u32, width: f32) -> Shape {
        Shape::Grid {
            a,
            b,
            cols,
            rows,
            style: style(width),
            axes: false,
            fills: Vec::new(),
            texts: Vec::new(),
        }
    }

    fn grid_with_axes(a: Vec2, b: Vec2, cols: u32, rows: u32) -> Shape {
        Shape::Grid {
            a,
            b,
            cols,
            rows,
            style: style(0.0),
            axes: true,
            fills: Vec::new(),
            texts: Vec::new(),
        }
    }

    fn stroke(points: &[Vec2], width: f32) -> Shape {
        Shape::Stroke {
            points: points.to_vec(),
            widths: None,
            style: style(width),
        }
    }

    fn pressed_stroke(points: &[Vec2], widths: &[f32], width: f32) -> Shape {
        Shape::Stroke {
            points: points.to_vec(),
            widths: Some(widths.to_vec()),
            style: style(width),
        }
    }

    // ---- T35: per-point widths ----

    #[test]
    fn stroke_max_width_factor_without_widths_is_one() {
        let s = stroke(&[v(0.0, 0.0), v(10.0, 0.0)], 2.0);

        assert!(approx_eq(s.max_width_factor(), 1.0, EPS));
        let r = rect(v(0.0, 0.0), v(1.0, 1.0), 1.0, None);
        assert!(approx_eq(r.max_width_factor(), 1.0, EPS));
    }

    #[test]
    fn stroke_bounds_grow_with_the_widest_point() {
        // Arrange: width 2, widest factor 3 → half-width 3.
        let s = pressed_stroke(&[v(0.0, 0.0), v(10.0, 0.0)], &[0.5, 3.0], 2.0);

        // Act
        let b = s.bounds();

        // Assert
        assert!(approx_eq(s.max_width_factor(), 3.0, EPS));
        assert_vec_eq(b.min, v(-3.0, -3.0));
        assert_vec_eq(b.max, v(13.0, 3.0));
    }

    #[test]
    fn stroke_hit_reaches_a_thick_point() {
        // Arrange: half-width 3 with the factor, 1 without.
        let thick = pressed_stroke(&[v(0.0, 0.0), v(10.0, 0.0)], &[3.0, 3.0], 2.0);
        let plain = stroke(&[v(0.0, 0.0), v(10.0, 0.0)], 2.0);

        // Act / Assert
        assert!(thick.hit(v(5.0, 2.5), 0.0));
        assert!(!plain.hit(v(5.0, 2.5), 0.0));
    }

    #[test]
    fn stroke_with_bad_widths_is_not_finite() {
        let points = [v(0.0, 0.0), v(10.0, 0.0)];

        assert!(pressed_stroke(&points, &[1.0, 2.0], 1.0).is_finite());
        assert!(!pressed_stroke(&points, &[1.0], 1.0).is_finite());
        assert!(!pressed_stroke(&points, &[1.0, f32::NAN], 1.0).is_finite());
        assert!(!pressed_stroke(&points, &[1.0, 0.0], 1.0).is_finite());
        assert!(!pressed_stroke(&points, &[1.0, -1.0], 1.0).is_finite());
    }

    // ---- AC-1: construction and accessors ----

    #[test]
    fn construct_stroke_keeps_points_and_style() {
        let s = Shape::Stroke {
            points: vec![v(0.0, 0.0), v(1.0, 2.0)],
            widths: None,
            style: Style {
                color: color(2),
                width: 3.0,
            },
        };

        let Shape::Stroke { points, style, .. } = s else {
            panic!("expected Stroke");
        };
        assert_eq!(points.len(), 2);
        assert_vec_eq(points[1], v(1.0, 2.0));
        assert_eq!(style.color, color(2));
        assert!(approx_eq(style.width, 3.0, EPS));
    }

    #[test]
    fn construct_line_and_arrow_keep_endpoints() {
        let line = Shape::Line {
            a: v(1.0, 1.0),
            b: v(4.0, 5.0),
            style: style(2.0),
        };
        let arrow = Shape::Arrow {
            a: v(-1.0, 0.0),
            b: v(0.0, 7.0),
            style: style(2.0),
        };

        let Shape::Line { a, b, .. } = line else {
            panic!("expected Line");
        };
        assert_vec_eq(a, v(1.0, 1.0));
        assert_vec_eq(b, v(4.0, 5.0));
        let Shape::Arrow { a, b, .. } = arrow else {
            panic!("expected Arrow");
        };
        assert_vec_eq(a, v(-1.0, 0.0));
        assert_vec_eq(b, v(0.0, 7.0));
    }

    #[test]
    fn construct_rect_and_ellipse_keep_corners_and_fill() {
        let r = rect(v(0.0, 0.0), v(2.0, 3.0), 1.0, Some(color(3)));
        let e = ellipse(v(5.0, 5.0), v(1.0, 1.0), 1.0, None);

        let Shape::Rect { a, b, fill, .. } = r else {
            panic!("expected Rect");
        };
        assert_vec_eq(a, v(0.0, 0.0));
        assert_vec_eq(b, v(2.0, 3.0));
        assert_eq!(fill, Some(color(3)));
        let Shape::Ellipse { a, b, fill, .. } = e else {
            panic!("expected Ellipse");
        };
        assert_vec_eq(a, v(5.0, 5.0));
        assert_vec_eq(b, v(1.0, 1.0));
        assert_eq!(fill, None);
    }

    #[test]
    fn accessors_report_style_fill_and_closedness() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: Style {
                color: color(1),
                width: 4.0,
            },
        };
        let filled = rect(v(0.0, 0.0), v(1.0, 1.0), 2.0, Some(color(4)));
        let hollow = ellipse(v(0.0, 0.0), v(1.0, 1.0), 2.0, None);

        assert_eq!(line.style().color, color(1));
        assert!(approx_eq(line.style().width, 4.0, EPS));
        assert_eq!(line.fill(), None);
        assert!(!line.is_closed());
        assert!(!stroke(&[v(0.0, 0.0)], 1.0).is_closed());
        assert_eq!(filled.fill(), Some(color(4)));
        assert!(filled.is_closed());
        assert_eq!(hollow.fill(), None);
        assert!(hollow.is_closed());
    }

    // ---- AC-2: bounds ----

    #[test]
    fn bounds_stroke_includes_half_width() {
        let s = stroke(&[v(0.0, 0.0), v(4.0, -2.0), v(1.0, 3.0)], 2.0);

        assert_aabb_eq(s.bounds(), v(-1.0, -3.0), v(5.0, 4.0));
    }

    #[test]
    fn bounds_empty_stroke_is_point_at_origin() {
        let empty = stroke(&[], 2.0);
        let all_nan = stroke(&[v(f32::NAN, 1.0)], 2.0);

        assert_aabb_eq(empty.bounds(), Vec2::ZERO, Vec2::ZERO);
        assert_aabb_eq(all_nan.bounds(), Vec2::ZERO, Vec2::ZERO);
    }

    #[test]
    fn bounds_line_includes_half_width() {
        let line = Shape::Line {
            a: v(3.0, 1.0),
            b: v(-1.0, 2.0),
            style: style(1.0),
        };

        assert_aabb_eq(line.bounds(), v(-1.5, 0.5), v(3.5, 2.5));
    }

    #[test]
    fn bounds_arrow_includes_head() {
        let (a, b, width) = (v(0.0, 0.0), v(100.0, 0.0), 2.0);
        let arrow = Shape::Arrow {
            a,
            b,
            style: style(width),
        };
        let head = arrow_head(a, b, width);

        let bounds = arrow.bounds();

        let half_head = ARROW_HEAD_HALF_WIDTH_RATIO * ARROW_HEAD_LENGTH_PER_WIDTH * width;
        assert!(half_head > 1.0, "head must stick out past the half width");
        assert_aabb_eq(bounds, v(-1.0, -half_head - 1.0), v(101.0, half_head + 1.0));
        for p in head {
            assert!(bounds.contains(p), "{p:?} outside {bounds:?}");
        }
    }

    #[test]
    fn bounds_rect_is_corner_order_independent() {
        let r1 = rect(v(0.0, 0.0), v(4.0, 2.0), 2.0, None);
        let r2 = rect(v(4.0, 0.0), v(0.0, 2.0), 2.0, None);

        assert_aabb_eq(r1.bounds(), v(-1.0, -1.0), v(5.0, 3.0));
        assert_aabb_eq(r2.bounds(), v(-1.0, -1.0), v(5.0, 3.0));
    }

    #[test]
    fn bounds_ellipse_matches_bounding_box() {
        let e = ellipse(v(10.0, 4.0), v(2.0, -4.0), 4.0, Some(color(1)));

        assert_aabb_eq(e.bounds(), v(0.0, -6.0), v(12.0, 6.0));
    }

    // ---- AC-3: hit ----

    #[test]
    fn hit_stroke_near_segment_is_true() {
        let s = stroke(&[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)], 2.0);

        // reach = tol 1 + half width 1 = 2
        assert!(s.hit(v(5.0, 1.9), 1.0));
        assert!(s.hit(v(11.9, 5.0), 1.0));
        assert!(s.hit(v(10.0, 10.0), 0.0));
    }

    #[test]
    fn hit_stroke_far_is_false() {
        let s = stroke(&[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)], 2.0);

        assert!(!s.hit(v(5.0, 2.1), 1.0));
        assert!(!s.hit(v(5.0, 5.0), 1.0));
        assert!(!s.hit(v(-3.0, 0.0), 1.0));
    }

    #[test]
    fn hit_single_point_stroke_uses_distance_to_point() {
        let dot = stroke(&[v(3.0, 4.0)], 2.0);

        assert!(dot.hit(v(3.0, 4.0), 0.0));
        assert!(dot.hit(v(3.0, 5.9), 1.0));
        assert!(!dot.hit(v(3.0, 6.1), 1.0));
    }

    #[test]
    fn hit_empty_stroke_is_false() {
        let empty = stroke(&[], 2.0);

        assert!(!empty.hit(Vec2::ZERO, 100.0));
    }

    #[test]
    fn hit_line_edge_within_tolerance_plus_half_width() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(10.0, 0.0),
            style: style(4.0),
        };

        // reach = 0.5 + 2 = 2.5
        assert!(line.hit(v(5.0, 2.4), 0.5));
        assert!(line.hit(v(5.0, -2.4), 0.5));
        assert!(!line.hit(v(5.0, 2.6), 0.5));
        assert!(line.hit(v(12.4, 0.0), 0.5));
        assert!(!line.hit(v(12.6, 0.0), 0.5));
    }

    #[test]
    fn hit_arrow_shaft_and_head() {
        let (a, b, width) = (v(0.0, 0.0), v(100.0, 0.0), 1.0);
        let arrow = Shape::Arrow {
            a,
            b,
            style: style(width),
        };
        let [_, left, _] = arrow_head(a, b, width);

        assert!(arrow.hit(v(50.0, 0.4), 0.0), "shaft");
        assert!(!arrow.hit(v(50.0, 2.0), 0.0), "beside shaft");
        assert!(arrow.hit(left, 0.0), "head corner");
        let inside_head = v(
            100.0 - ARROW_HEAD_LENGTH_PER_WIDTH * width * 0.5,
            left.y * 0.4,
        );
        assert!(arrow.hit(inside_head, 0.0), "inside head");
        assert!(!arrow.hit(v(104.0, 0.0), 0.0), "beyond tip");
    }

    #[test]
    fn hit_rect_outline_inside_outside() {
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 2.0, None);

        assert!(r.hit(v(0.0, 5.0), 0.0), "on edge");
        assert!(r.hit(v(11.5, 5.0), 0.5), "just outside, within reach");
        assert!(r.hit(v(5.0, 8.5), 0.5), "just inside, within reach");
        assert!(!r.hit(v(5.0, 5.0), 0.5), "hollow interior");
        assert!(!r.hit(v(12.0, 5.0), 0.5), "outside");
    }

    #[test]
    fn hit_filled_rect_interior_is_true() {
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 2.0, Some(color(2)));

        assert!(r.hit(v(5.0, 5.0), 0.0));
        assert!(!r.hit(v(12.0, 5.0), 0.5));
    }

    #[test]
    fn hit_ellipse_outline_inside_outside() {
        // centre (0, 0), rx = 10, ry = 5
        let e = ellipse(v(-10.0, -5.0), v(10.0, 5.0), 2.0, None);
        let on_outline = v(10.0 * 0.6, 5.0 * 0.8);

        assert!(e.hit(on_outline, 0.0), "on outline");
        assert!(e.hit(v(11.9, 0.0), 1.0), "outside on x axis, within reach");
        assert!(e.hit(v(0.0, 3.1), 1.0), "inside on y axis, within reach");
        assert!(!e.hit(v(12.1, 0.0), 1.0), "outside on x axis");
        assert!(!e.hit(v(0.0, 2.9), 1.0), "inside on y axis");
        assert!(!e.hit(Vec2::ZERO, 1.0), "hollow centre");
        assert!(!e.hit(v(10.0, 5.0), 1.0), "box corner");
    }

    #[test]
    fn hit_filled_ellipse_interior_is_true() {
        let e = ellipse(v(-10.0, -5.0), v(10.0, 5.0), 2.0, Some(color(5)));

        assert!(e.hit(Vec2::ZERO, 0.0));
        assert!(!e.hit(v(10.0, 5.0), 1.0), "box corner is outside");
    }

    #[test]
    fn hit_degenerate_ellipse_is_segment() {
        let flat = ellipse(v(0.0, 2.0), v(10.0, 2.0), 2.0, None);

        assert!(flat.hit(v(5.0, 3.5), 0.5));
        assert!(!flat.hit(v(5.0, 4.0), 0.5));
        assert!(!flat.hit(v(11.6, 2.0), 0.5));
    }

    #[test]
    fn hit_non_finite_point_is_false() {
        let filled = rect(v(0.0, 0.0), v(10.0, 10.0), 2.0, Some(color(1)));
        let s = stroke(&[v(0.0, 0.0), v(1.0, 1.0)], 1.0);

        for p in [
            v(f32::NAN, 5.0),
            v(5.0, f32::INFINITY),
            v(f32::NEG_INFINITY, 0.0),
        ] {
            assert!(!filled.hit(p, 1.0), "{p:?}");
            assert!(!s.hit(p, f32::MAX), "{p:?}");
        }
        assert!(!s.hit(v(0.0, 0.0), f32::NAN));
    }

    // ---- AC-4: contains ----

    #[test]
    fn contains_rect_inside_and_boundary() {
        let r = rect(v(10.0, 10.0), v(0.0, 0.0), 2.0, None);

        assert!(r.contains(v(5.0, 5.0)));
        assert!(r.contains(v(0.0, 0.0)));
        assert!(r.contains(v(10.0, 3.0)));
    }

    #[test]
    fn contains_rect_outside_is_false() {
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 2.0, None);

        assert!(
            !r.contains(v(10.5, 5.0)),
            "within stroke but outside geometry"
        );
        assert!(!r.contains(v(-1.0, -1.0)));
        assert!(!r.contains(v(f32::NAN, 5.0)));
    }

    #[test]
    fn contains_ellipse_inside_and_outside() {
        let e = ellipse(v(-10.0, -5.0), v(10.0, 5.0), 2.0, None);

        assert!(e.contains(Vec2::ZERO));
        assert!(e.contains(v(9.9, 0.0)));
        assert!(e.contains(v(0.0, -4.9)));
        assert!(!e.contains(v(10.1, 0.0)));
        assert!(!e.contains(v(9.0, 4.0)), "inside box, outside ellipse");
        assert!(!e.contains(v(0.0, f32::NAN)));
    }

    #[test]
    fn contains_degenerate_ellipse_is_false() {
        let flat = ellipse(v(0.0, 0.0), v(10.0, 0.0), 2.0, Some(color(1)));
        let point = ellipse(v(3.0, 3.0), v(3.0, 3.0), 2.0, Some(color(1)));

        assert!(!flat.contains(v(5.0, 0.0)));
        assert!(!point.contains(v(3.0, 3.0)));
    }

    #[test]
    fn contains_open_shapes_is_false() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(10.0, 10.0),
            style: style(2.0),
        };
        let arrow = Shape::Arrow {
            a: v(0.0, 0.0),
            b: v(10.0, 10.0),
            style: style(2.0),
        };
        let s = stroke(
            &[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 0.0)],
            2.0,
        );

        assert!(!line.contains(v(5.0, 5.0)));
        assert!(!arrow.contains(v(5.0, 5.0)));
        assert!(!s.contains(v(7.0, 3.0)));
    }

    // ---- AC-5: translate and with_fill ----

    #[test]
    fn translate_moves_every_point() {
        let d = v(3.0, -2.0);
        let mut s = stroke(&[v(0.0, 0.0), v(1.0, 1.0)], 2.0);
        let mut line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: style(2.0),
        };
        let mut arrow = Shape::Arrow {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: style(2.0),
        };
        let mut r = rect(v(0.0, 0.0), v(2.0, 2.0), 2.0, Some(color(2)));
        let mut e = ellipse(v(0.0, 0.0), v(2.0, 2.0), 2.0, None);

        for shape in [&mut s, &mut line, &mut arrow, &mut r, &mut e] {
            shape.translate(d);
        }

        assert_eq!(
            s,
            stroke(&[v(3.0, -2.0), v(4.0, -1.0)], 2.0),
            "stroke points move"
        );
        let Shape::Line { a, b, .. } = line else {
            panic!("expected Line");
        };
        assert_vec_eq(a, v(3.0, -2.0));
        assert_vec_eq(b, v(4.0, -2.0));
        let Shape::Arrow { a, b, .. } = arrow else {
            panic!("expected Arrow");
        };
        assert_vec_eq(a, v(3.0, -2.0));
        assert_vec_eq(b, v(4.0, -2.0));
        assert_eq!(r, rect(v(3.0, -2.0), v(5.0, 0.0), 2.0, Some(color(2))));
        assert_eq!(e, ellipse(v(3.0, -2.0), v(5.0, 0.0), 2.0, None));
    }

    #[test]
    fn with_fill_sets_and_clears_fill_on_closed_shapes() {
        let r = rect(v(0.0, 0.0), v(1.0, 1.0), 1.0, None);
        let e = ellipse(v(0.0, 0.0), v(1.0, 1.0), 1.0, Some(color(1)));

        let filled = r.with_fill(Some(color(3)));
        let cleared = e.with_fill(None);

        assert_eq!(filled.fill(), Some(color(3)));
        assert_eq!(filled, rect(v(0.0, 0.0), v(1.0, 1.0), 1.0, Some(color(3))));
        assert_eq!(cleared.fill(), None);
        assert_eq!(cleared, ellipse(v(0.0, 0.0), v(1.0, 1.0), 1.0, None));
    }

    #[test]
    fn with_fill_on_open_shape_is_noop() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: style(1.0),
        };
        let s = stroke(&[v(0.0, 0.0)], 1.0);

        assert_eq!(line.clone().with_fill(Some(color(2))), line);
        assert_eq!(s.clone().with_fill(Some(color(2))), s);
    }

    // ---- AC-6: arrow_head ----

    #[test]
    fn arrow_head_tip_is_at_b() {
        let [tip, _, _] = arrow_head(v(0.0, 0.0), v(30.0, 40.0), 2.0);

        assert_vec_eq(tip, v(30.0, 40.0));
    }

    #[test]
    fn arrow_head_length_scales_with_width() {
        let width = 10.0;
        let length = ARROW_HEAD_LENGTH_PER_WIDTH * width;

        let [_, left, right] = arrow_head(v(0.0, 0.0), v(100.0, 0.0), width);

        let half = ARROW_HEAD_HALF_WIDTH_RATIO * length;
        assert_vec_eq(left, v(100.0 - length, half));
        assert_vec_eq(right, v(100.0 - length, -half));
    }

    #[test]
    fn arrow_head_scales_with_thin_width() {
        // An arrow drawn zoomed in has a thin world width; its head must
        // shrink with it, not stay at an absolute size.
        let width = 0.25;
        let length = ARROW_HEAD_LENGTH_PER_WIDTH * width;

        let [tip, left, right] = arrow_head(v(0.0, 0.0), v(0.0, 50.0), width);

        let base_mid = left.lerp(right, 0.5);
        assert!(approx_eq(tip.distance(base_mid), length, EPS));
        assert!(approx_eq(
            left.distance(right),
            2.0 * ARROW_HEAD_HALF_WIDTH_RATIO * length,
            EPS
        ));
    }

    proptest! {
        #[test]
        fn arrow_head_is_scale_invariant(
            (bx, by) in (-1e3f32..1e3, -1e3f32..1e3),
            width in 0.01f32..50.0,
            scale in 0.05f32..20.0,
        ) {
            let b = v(bx, by);
            prop_assume!(b.length() > 1.0);
            let head = arrow_head(Vec2::ZERO, b, width);
            let scaled = arrow_head(Vec2::ZERO, b * scale, width * scale);
            for (p, q) in head.iter().zip(scaled) {
                let tol = 1e-3 * (1.0 + (*p * scale).length());
                prop_assert!((*p * scale).approx_eq(q, tol), "{p:?} * {scale} != {q:?}");
            }
        }
    }

    #[test]
    fn arrow_head_is_symmetric_about_shaft() {
        let (a, b) = (v(-3.0, 7.0), v(20.0, -11.0));

        let [tip, left, right] = arrow_head(a, b, 3.0);

        assert!(approx_eq(tip.distance(left), tip.distance(right), EPS));
        let base_mid = left.lerp(right, 0.5);
        let shaft = b - a;
        let base = right - left;
        assert!(approx_eq(base.dot(shaft), 0.0, 1e-2), "base ⟂ shaft");
        let to_mid = base_mid - a;
        let cross = shaft.x * to_mid.y - shaft.y * to_mid.x;
        assert!(approx_eq(cross, 0.0, 1e-2), "base midpoint on shaft");
    }

    #[test]
    fn arrow_head_degenerate_collapses_to_tip() {
        let b = v(4.0, 4.0);

        for p in arrow_head(b, b, 2.0) {
            assert_vec_eq(p, b);
        }
    }

    // ---- AC-7: is_finite ----

    #[test]
    fn is_finite_true_for_finite_shapes() {
        assert!(stroke(&[], 1.0).is_finite());
        assert!(stroke(&[v(0.0, 0.0), v(1.0, 1.0)], 1.0).is_finite());
        assert!(rect(v(0.0, 0.0), v(1.0, 1.0), 1.0, Some(color(1))).is_finite());
        assert!(
            Shape::Arrow {
                a: v(0.0, 0.0),
                b: v(-1.0, 1.0),
                style: style(2.0)
            }
            .is_finite()
        );
    }

    #[test]
    fn is_finite_false_for_nan_or_inf_coordinate() {
        assert!(!stroke(&[v(0.0, 0.0), v(f32::NAN, 1.0)], 1.0).is_finite());
        assert!(
            !Shape::Line {
                a: v(0.0, f32::INFINITY),
                b: v(1.0, 1.0),
                style: style(1.0)
            }
            .is_finite()
        );
        assert!(!ellipse(v(0.0, 0.0), v(f32::NEG_INFINITY, 1.0), 1.0, None).is_finite());
    }

    #[test]
    fn is_finite_false_for_non_finite_width() {
        assert!(!stroke(&[v(0.0, 0.0)], f32::NAN).is_finite());
        assert!(!rect(v(0.0, 0.0), v(1.0, 1.0), f32::INFINITY, None).is_finite());
    }

    // ---- T16 AC-1: grid ----

    #[test]
    fn grid_bounds_include_half_width() {
        let g = grid(v(30.0, 20.0), v(0.0, 0.0), 3, 2, 2.0);

        assert_aabb_eq(g.bounds(), v(-1.0, -1.0), v(31.0, 21.0));
    }

    #[test]
    fn grid_hit_on_inner_line() {
        // 3 × 2 cells of 10 × 10; width 2 gives reach 1 at tol 0.
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        assert!(g.hit(v(10.0, 5.0), 0.0), "inner vertical line");
        assert!(g.hit(v(20.9, 15.0), 0.0), "within reach of x = 20");
        assert!(g.hit(v(5.0, 10.0), 0.0), "inner horizontal line");
        assert!(g.hit(v(30.0, 3.0), 0.0), "outer edge");
    }

    #[test]
    fn grid_hit_between_lines_is_false() {
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        assert!(!g.hit(v(15.0, 5.0), 0.0), "cell centre");
        assert!(!g.hit(v(12.5, 15.0), 0.5), "beyond reach");
        assert!(!g.hit(v(40.0, 5.0), 0.5), "outside");
        assert!(!g.hit(v(f32::NAN, 5.0), 1.0), "non-finite");
    }

    #[test]
    fn grid_contains_nothing_and_is_open() {
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        assert!(!g.contains(v(15.0, 5.0)));
        assert!(!g.is_closed());
        assert_eq!(g.fill(), None);
        assert_eq!(g.label(), None);
    }

    #[test]
    fn grid_with_fill_is_noop() {
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        assert_eq!(g.clone().with_fill(Some(color(2))), g);
    }

    #[test]
    fn grid_translate_moves_corners() {
        let mut g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        g.translate(v(5.0, -5.0));

        assert_eq!(g, grid(v(5.0, -5.0), v(35.0, 15.0), 3, 2, 2.0));
    }

    #[test]
    fn grid_is_finite_checks_corners() {
        assert!(grid(v(0.0, 0.0), v(1.0, 1.0), 4, 4, 1.0).is_finite());
        assert!(!grid(v(f32::NAN, 0.0), v(1.0, 1.0), 4, 4, 1.0).is_finite());
        assert!(!grid(v(0.0, 0.0), v(1.0, f32::INFINITY), 4, 4, 1.0).is_finite());
        assert!(!grid(v(0.0, 0.0), v(1.0, 1.0), 4, 4, f32::NAN).is_finite());
    }

    #[test]
    fn grid_lines_count_and_positions() {
        // Corners in reverse order: lines follow the normalised box.
        let lines: Vec<[Vec2; 2]> = grid_lines(v(30.0, 20.0), v(0.0, 0.0), 3, 2).collect();

        assert_eq!(lines.len(), 4 + 3, "cols + 1 vertical, rows + 1 horizontal");
        for (i, [p, q]) in lines[..4].iter().enumerate() {
            let x = 10.0 * i as f32;
            assert_vec_eq(*p, v(x, 0.0));
            assert_vec_eq(*q, v(x, 20.0));
        }
        for (j, [p, q]) in lines[4..].iter().enumerate() {
            let y = 10.0 * j as f32;
            assert_vec_eq(*p, v(0.0, y));
            assert_vec_eq(*q, v(30.0, y));
        }
    }

    #[test]
    fn grid_lines_clamps_dims() {
        let (a, b) = (v(0.0, 0.0), v(10.0, 10.0));

        assert_eq!(grid_lines(a, b, 0, 0).count(), 2 + 2);
        assert_eq!(
            grid_lines(a, b, u32::MAX, 1000).count(),
            2 * (GRID_MAX_CELLS as usize + 1)
        );
    }

    // ---- T16 AC-1: labels ----

    #[test]
    fn label_accessor_and_with_label() {
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 1.0, None);
        let e = ellipse(v(0.0, 0.0), v(10.0, 10.0), 1.0, None);

        assert_eq!(r.label(), None);
        let r7 = r.with_label(Some(7));
        let e3 = e.with_label(Some(3));

        assert_eq!(r7.label(), Some(7));
        assert_eq!(e3.label(), Some(3));
        assert!(matches!(r7, Shape::Rect { label: Some(7), .. }));
        assert_eq!(e3.with_label(None).label(), None);
    }

    #[test]
    fn label_with_label_on_open_shape_is_noop() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: style(1.0),
        };
        let g = grid(v(0.0, 0.0), v(10.0, 10.0), 2, 2, 1.0);
        let s = stroke(&[v(0.0, 0.0)], 1.0);

        for shape in [line, g, s] {
            assert_eq!(shape.clone().with_label(Some(4)), shape);
            assert_eq!(shape.label(), None);
        }
    }

    #[test]
    fn label_survives_translate_and_with_fill() {
        let mut r = rect(v(0.0, 0.0), v(10.0, 10.0), 1.0, None).with_label(Some(2));
        let e = ellipse(v(0.0, 0.0), v(10.0, 10.0), 1.0, None).with_label(Some(9));

        r.translate(v(1.0, 1.0));
        let filled = e.with_fill(Some(color(4)));

        assert_eq!(r.label(), Some(2));
        assert_eq!(filled.label(), Some(9));
        assert_eq!(filled.fill(), Some(color(4)));
    }

    // ---- properties ----

    fn coord() -> impl Strategy<Value = f32> {
        -1.0e3_f32..1.0e3_f32
    }

    fn point() -> impl Strategy<Value = Vec2> {
        (coord(), coord()).prop_map(|(x, y)| Vec2::new(x, y))
    }

    fn label() -> impl Strategy<Value = Option<u32>> {
        prop::option::of(1_u32..1000)
    }

    fn any_shape() -> impl Strategy<Value = Shape> {
        let width = 0.0_f32..20.0;
        prop_oneof![
            (prop::collection::vec(point(), 1..16), width.clone()).prop_map(|(points, w)| {
                Shape::Stroke {
                    points,
                    widths: None,
                    style: style(w),
                }
            }),
            (point(), point(), width.clone()).prop_map(|(a, b, w)| Shape::Line {
                a,
                b,
                style: style(w)
            }),
            (point(), point(), width.clone()).prop_map(|(a, b, w)| Shape::Arrow {
                a,
                b,
                style: style(w)
            }),
            (point(), point(), width.clone(), label())
                .prop_map(|(a, b, w, l)| rect(a, b, w, None).with_label(l)),
            (point(), point(), width.clone(), label())
                .prop_map(|(a, b, w, l)| ellipse(a, b, w, None).with_label(l)),
            (
                point(),
                point(),
                1..=GRID_MAX_CELLS,
                1..=GRID_MAX_CELLS,
                width
            )
                .prop_map(|(a, b, cols, rows, w)| grid(a, b, cols, rows, w)),
        ]
    }

    proptest! {
        #[test]
        fn translate_moves_bounds_by_delta(shape in any_shape(), delta in point()) {
            let before = shape.bounds();
            let mut moved = shape;

            let label = moved.label();
            moved.translate(delta);

            prop_assert_eq!(moved.label(), label);
            let expected = before.translate(delta);
            let after = moved.bounds();
            // Coordinates up to 2e3 in f32: rounding stays well below 1e-2.
            prop_assert!(after.min.approx_eq(expected.min, 1e-2), "{after:?} vs {expected:?}");
            prop_assert!(after.max.approx_eq(expected.max, 1e-2), "{after:?} vs {expected:?}");
        }
    }

    // ---- T18 AC-7 axis indices --------------------------------------------

    /// `(index, rect)` of each axis label of `shape`, columns then rows.
    fn axis(shape: &Shape) -> Vec<(u32, Aabb)> {
        shape
            .axis_labels()
            .into_iter()
            .map(|l| (l.index, l.rect))
            .collect()
    }

    fn rect_eq(got: Aabb, min: (f32, f32), max: (f32, f32)) -> bool {
        got.min.approx_eq(Vec2::new(min.0, min.1), 1e-4)
            && got.max.approx_eq(Vec2::new(max.0, max.1), 1e-4)
    }

    #[test]
    fn grid_axis_labels_top_left_drag() {
        // Arrange: 3 × 2 cells of 10 × 20, dragged from (0, 0) to (30, 40).
        let shape = grid_with_axes(Vec2::ZERO, Vec2::new(30.0, 40.0), 3, 2);

        // Act
        let labels = axis(&shape);

        // Assert: columns 0..3 above the top edge, rows 0..2 left of it.
        assert_eq!(labels.len(), 5);
        let indices: Vec<u32> = labels.iter().map(|l| l.0).collect();
        assert_eq!(indices, [0, 1, 2, 0, 1]);
        assert!(
            rect_eq(labels[0].1, (0.0, -20.0), (10.0, 0.0)),
            "{:?}",
            labels[0]
        );
        assert!(
            rect_eq(labels[2].1, (20.0, -20.0), (30.0, 0.0)),
            "{:?}",
            labels[2]
        );
        assert!(
            rect_eq(labels[3].1, (-10.0, 0.0), (0.0, 20.0)),
            "{:?}",
            labels[3]
        );
        assert!(
            rect_eq(labels[4].1, (-10.0, 20.0), (0.0, 40.0)),
            "{:?}",
            labels[4]
        );
    }

    #[test]
    fn grid_axis_labels_follow_drag_direction() {
        // Arrange: dragged from the bottom-right (30, 40) to the top-left.
        let shape = grid_with_axes(Vec2::new(30.0, 40.0), Vec2::ZERO, 3, 2);

        // Act
        let labels = axis(&shape);

        // Assert: column 0 is the rightmost, below the bottom edge; row 0 is
        // the bottom row, right of the right edge.
        assert!(
            rect_eq(labels[0].1, (20.0, 40.0), (30.0, 60.0)),
            "{:?}",
            labels[0]
        );
        assert!(
            rect_eq(labels[2].1, (0.0, 40.0), (10.0, 60.0)),
            "{:?}",
            labels[2]
        );
        assert!(
            rect_eq(labels[3].1, (30.0, 20.0), (40.0, 40.0)),
            "{:?}",
            labels[3]
        );
        assert!(
            rect_eq(labels[4].1, (30.0, 0.0), (40.0, 20.0)),
            "{:?}",
            labels[4]
        );
        let indices: Vec<u32> = labels.iter().map(|l| l.0).collect();
        assert_eq!(indices, [0, 1, 2, 0, 1]);
    }

    #[test]
    fn grid_axis_labels_none_without_axes() {
        let plain = grid(Vec2::ZERO, Vec2::new(30.0, 40.0), 3, 2, 1.0);
        let other = rect(Vec2::ZERO, Vec2::new(30.0, 40.0), 1.0, None);

        assert!(plain.axis_labels().is_empty());
        assert!(other.axis_labels().is_empty());
    }

    #[test]
    fn grid_bounds_include_axis_labels() {
        // Arrange
        let shape = grid_with_axes(Vec2::ZERO, Vec2::new(30.0, 40.0), 3, 2);

        // Act
        let bounds = shape.bounds();

        // Assert: one cell height above, one cell width to the left.
        assert!(rect_eq(bounds, (-10.0, -20.0), (30.0, 40.0)), "{bounds:?}");
        for (_, r) in axis(&shape) {
            assert!(bounds.contains(r.min) && bounds.contains(r.max));
        }
    }

    #[test]
    fn grid_axes_survive_translate() {
        // Arrange
        let mut shape = grid_with_axes(Vec2::new(30.0, 40.0), Vec2::ZERO, 3, 2);
        let before = axis(&shape);

        // Act
        shape.translate(Vec2::new(5.0, -7.0));

        // Assert: same orientation, moved by the delta.
        let after = axis(&shape);
        assert_eq!(before.len(), after.len());
        for ((i, r0), (j, r1)) in before.iter().zip(&after) {
            assert_eq!(i, j);
            assert!(r1.min.approx_eq(r0.min + Vec2::new(5.0, -7.0), 1e-4));
        }
    }

    // ---- T21 cell fills -----------------------------------------------------

    fn red() -> ColorId {
        ColorId::new(2).unwrap_or(ColorId::INK)
    }

    fn green() -> ColorId {
        ColorId::new(3).unwrap_or(ColorId::INK)
    }

    /// 4 × 2 cells of 10 × 10 dragged from (0, 0) to (40, 20).
    fn grid_4x2() -> Shape {
        grid(Vec2::ZERO, Vec2::new(40.0, 20.0), 4, 2, 1.0)
    }

    fn cells(shape: &Shape) -> Vec<(u32, u32)> {
        shape.cell_fills().iter().map(|f| (f.col, f.row)).collect()
    }

    #[test]
    fn grid_cell_at_counts_from_start_corner() {
        let g = grid_4x2();

        assert_eq!(g.grid_cell_at(Vec2::new(15.0, 5.0)), Some((1, 0)));
        assert_eq!(g.grid_cell_at(Vec2::new(35.0, 15.0)), Some((3, 1)));
        assert_eq!(
            g.grid_cell_at(Vec2::new(40.0, 20.0)),
            Some((3, 1)),
            "far edge"
        );
        assert_eq!(g.grid_cell_at(Vec2::ZERO), Some((0, 0)));
    }

    #[test]
    fn grid_cell_at_reversed_drag() {
        // Dragged from (40, 20) to the origin: cell (0, 0) is bottom-right.
        let g = grid(Vec2::new(40.0, 20.0), Vec2::ZERO, 4, 2, 1.0);

        assert_eq!(g.grid_cell_at(Vec2::new(35.0, 15.0)), Some((0, 0)));
        assert_eq!(g.grid_cell_at(Vec2::new(5.0, 5.0)), Some((3, 1)));
    }

    #[test]
    fn grid_cell_at_outside_is_none() {
        let g = grid_4x2();

        assert_eq!(g.grid_cell_at(Vec2::new(50.0, 5.0)), None);
        assert_eq!(g.grid_cell_at(Vec2::new(-1.0, 5.0)), None);
        assert_eq!(g.grid_cell_at(Vec2::new(f32::NAN, 5.0)), None);
        let other = rect(Vec2::ZERO, Vec2::new(40.0, 20.0), 1.0, None);
        assert_eq!(other.grid_cell_at(Vec2::new(5.0, 5.0)), None);
    }

    #[test]
    fn grid_cell_rect_matches_lines() {
        let a = Vec2::ZERO;
        let b = Vec2::new(40.0, 20.0);

        let cell = grid_cell_rect(a, b, 4, 2, 1, 0);
        let reversed = grid_cell_rect(b, a, 4, 2, 0, 0);

        assert!(rect_eq(cell, (10.0, 0.0), (20.0, 10.0)), "{cell:?}");
        assert!(
            rect_eq(reversed, (30.0, 10.0), (40.0, 20.0)),
            "{reversed:?}"
        );
        // Every cell edge lies on a grid line.
        let xs: Vec<f32> = grid_lines(a, b, 4, 2).take(5).map(|l| l[0].x).collect();
        assert!(xs.iter().any(|x| approx_eq(*x, cell.min.x, 1e-4)));
        assert!(xs.iter().any(|x| approx_eq(*x, cell.max.x, 1e-4)));
    }

    #[test]
    fn with_cell_fill_sets_and_replaces() {
        let g = grid_4x2().with_cell_fill(1, 0, Some(red()));
        assert_eq!(g.cell_fill(1, 0), Some(red()));
        assert_eq!(g.cell_fill(0, 0), None);

        let g = g.with_cell_fill(1, 0, Some(green()));

        assert_eq!(g.cell_fill(1, 0), Some(green()));
        assert_eq!(g.cell_fills().len(), 1);
    }

    #[test]
    fn with_cell_fill_none_clears() {
        let g = grid_4x2()
            .with_cell_fill(1, 0, Some(red()))
            .with_cell_fill(2, 1, Some(red()));

        let g = g.with_cell_fill(1, 0, None);

        assert_eq!(g.cell_fill(1, 0), None);
        assert_eq!(cells(&g), [(2, 1)]);
    }

    #[test]
    fn with_cell_fill_out_of_range_is_noop() {
        let g = grid_4x2();
        assert_eq!(g.clone().with_cell_fill(4, 0, Some(red())), g);
        assert_eq!(g.clone().with_cell_fill(0, 2, Some(red())), g);

        let r = rect(Vec2::ZERO, Vec2::new(10.0, 10.0), 1.0, None);
        assert_eq!(r.clone().with_cell_fill(0, 0, Some(red())), r);
        assert!(r.cell_fills().is_empty());
    }

    #[test]
    fn with_cell_fill_keeps_sorted_unique() {
        let g = grid_4x2()
            .with_cell_fill(3, 1, Some(red()))
            .with_cell_fill(0, 1, Some(red()))
            .with_cell_fill(2, 0, Some(red()))
            .with_cell_fill(0, 1, Some(green()));

        assert_eq!(cells(&g), [(2, 0), (0, 1), (3, 1)]);
        assert_eq!(g.cell_fill(0, 1), Some(green()));
    }

    #[test]
    fn grid_fills_survive_translate() {
        let mut g = grid_4x2().with_cell_fill(1, 1, Some(red()));

        g.translate(Vec2::new(100.0, 50.0));

        assert_eq!(g.cell_fill(1, 1), Some(red()));
        assert_eq!(g.grid_cell_at(Vec2::new(115.0, 65.0)), Some((1, 1)));
    }

    #[test]
    fn hit_outline_ignores_fill() {
        let filled = rect(Vec2::ZERO, Vec2::new(100.0, 100.0), 1.0, Some(red()));
        let inside = Vec2::new(50.0, 50.0);
        let edge = Vec2::new(0.0, 50.0);

        assert!(filled.hit(inside, 1.0));
        assert!(!filled.hit_outline(inside, 1.0));
        assert!(filled.hit_outline(edge, 1.0));
    }

    // ---- T29 text blocks ---------------------------------------------------

    fn text(origin: Vec2, content: &str, size: f32) -> Shape {
        Shape::Text {
            origin,
            text: content.to_owned(),
            size,
            style: style(0.0),
        }
    }

    #[test]
    fn text_bounds_follow_layout() {
        // Arrange: 3 columns × 2 lines at size 10 (cells 6 × 12.5).
        let shape = text(v(10.0, 20.0), "abc\nd", 10.0);

        // Act / Assert
        assert_aabb_eq(shape.bounds(), v(10.0, 20.0), v(28.0, 45.0));
    }

    #[test]
    fn text_hit_inside_box() {
        let shape = text(v(0.0, 0.0), "abcd", 10.0);

        assert!(shape.hit(v(12.0, 6.0), 0.0));
        assert!(shape.hit_outline(v(12.0, 6.0), 0.0));
        assert!(shape.hit(v(25.0, 6.0), 2.0), "within tolerance of the box");
        assert!(!shape.hit(v(40.0, 6.0), 2.0));
        assert!(!shape.hit(v(f32::NAN, 6.0), 2.0));
    }

    #[test]
    fn text_is_open_and_never_filled() {
        let shape = text(v(0.0, 0.0), "abcd", 10.0);

        assert!(!shape.is_closed());
        assert!(!shape.contains(v(5.0, 5.0)));
        assert_eq!(shape.clone().with_fill(Some(red())), shape);
        assert_eq!(shape.fill(), None);
    }

    #[test]
    fn text_has_no_label() {
        let shape = text(v(0.0, 0.0), "7", 10.0);

        assert_eq!(shape.label(), None);
        assert_eq!(shape.clone().with_label(Some(3)), shape);
    }

    #[test]
    fn text_translate_moves_origin() {
        let mut shape = text(v(1.0, 2.0), "x", 10.0);

        shape.translate(v(10.0, -5.0));

        assert_eq!(shape, text(v(11.0, -3.0), "x", 10.0));
    }

    #[test]
    fn text_is_finite_checks_origin_and_size() {
        assert!(text(v(1.0, 2.0), "x", 10.0).is_finite());
        assert!(!text(v(f32::NAN, 2.0), "x", 10.0).is_finite());
        assert!(!text(v(1.0, 2.0), "x", f32::INFINITY).is_finite());
        assert_eq!(text(v(1.0, 2.0), "x", 10.0).style(), style(0.0));
    }

    proptest! {
        #[test]
        fn cell_fills_stay_valid(
            cols in 1_u32..8,
            rows in 1_u32..8,
            ops in prop::collection::vec((0_u32..10, 0_u32..10, any::<bool>()), 0..40),
        ) {
            let mut g = grid(Vec2::ZERO, Vec2::new(80.0, 80.0), cols, rows, 1.0);
            for (col, row, set) in ops {
                g = g.with_cell_fill(col, row, set.then(red));
            }

            let keys: Vec<(u32, u32)> = g.cell_fills().iter().map(|f| (f.row, f.col)).collect();
            prop_assert!(keys.windows(2).all(|w| w[0] < w[1]), "{keys:?}");
            prop_assert!(g.cell_fills().iter().all(|f| f.col < cols && f.row < rows));
        }
    }
    // ---- T31 shape text ---------------------------------------------------

    fn st(text: &str, size: f32) -> Option<ShapeText> {
        ShapeText::new(text.to_owned(), size)
    }

    fn cell_texts_of(shape: &Shape) -> Vec<(u32, u32, String)> {
        shape
            .cell_texts()
            .iter()
            .map(|c| (c.col, c.row, c.content.text.clone()))
            .collect()
    }

    #[test]
    fn shape_text_new_rejects_blank_text_and_bad_sizes() {
        assert!(st("a", 10.0).is_some());
        assert!(st("", 10.0).is_none());
        assert!(st(" \n ", 10.0).is_none());
        assert!(st("a", 0.0).is_none());
        assert!(st("a", -1.0).is_none());
        assert!(st("a", f32::NAN).is_none());
        assert!(st("a", f32::INFINITY).is_none());
    }

    #[test]
    fn with_shape_text_sets_and_clears_rect_and_ellipse_text() {
        for shape in [
            rect(v(0.0, 0.0), v(10.0, 10.0), 1.0, None),
            ellipse(v(0.0, 0.0), v(10.0, 10.0), 1.0, None),
        ] {
            // Act
            let with = shape.clone().with_slot_text(TextSlot::Shape, st("ab", 5.0));

            // Assert
            assert_eq!(with.slot_text(TextSlot::Shape), st("ab", 5.0).as_ref());
            let cleared = with.with_slot_text(TextSlot::Shape, None);
            assert_eq!(cleared, shape);
        }
        // Other shapes and slots are left alone.
        let g = grid(v(0.0, 0.0), v(10.0, 10.0), 2, 2, 1.0);
        assert_eq!(g.clone().with_slot_text(TextSlot::Shape, st("a", 1.0)), g);
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 1.0, None);
        let cell = TextSlot::Cell { col: 0, row: 0 };
        assert_eq!(r.clone().with_slot_text(cell, st("a", 1.0)), r);
        assert_eq!(r.slot_text(cell), None);
    }

    #[test]
    fn with_cell_text_keeps_cells_sorted_unique_and_in_range() {
        // Arrange
        let cell = |col, row| TextSlot::Cell { col, row };
        let g = grid(v(0.0, 0.0), v(30.0, 30.0), 3, 3, 1.0);

        // Act
        let g = g
            .with_slot_text(cell(2, 1), st("c", 1.0))
            .with_slot_text(cell(0, 0), st("a", 1.0))
            .with_slot_text(cell(1, 1), st("b", 1.0))
            .with_slot_text(cell(0, 0), st("A", 2.0))
            .with_slot_text(cell(3, 0), st("x", 1.0))
            .with_slot_text(cell(0, 9), st("x", 1.0));

        // Assert: sorted by (row, col), one per cell, in range.
        assert_eq!(
            cell_texts_of(&g),
            [
                (0, 0, "A".to_owned()),
                (1, 1, "b".to_owned()),
                (2, 1, "c".to_owned())
            ]
        );
        assert_eq!(g.slot_text(cell(0, 0)), st("A", 2.0).as_ref());
        let g = g.with_slot_text(cell(1, 1), None);
        assert_eq!(cell_texts_of(&g).len(), 2);
        assert_eq!(g.slot_text(cell(1, 1)), None);
    }

    #[test]
    fn text_slot_at_finds_interior_or_cell() {
        let r = rect(v(0.0, 0.0), v(100.0, 50.0), 1.0, None);
        let e = ellipse(v(0.0, 0.0), v(100.0, 100.0), 1.0, None);
        let g = grid(v(0.0, 0.0), v(100.0, 100.0), 4, 4, 1.0);
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(10.0, 10.0),
            style: style(1.0),
        };

        assert_eq!(r.text_slot_at(v(50.0, 25.0)), Some(TextSlot::Shape));
        assert_eq!(r.text_slot_at(v(150.0, 25.0)), None);
        assert_eq!(e.text_slot_at(v(50.0, 50.0)), Some(TextSlot::Shape));
        assert_eq!(e.text_slot_at(v(2.0, 2.0)), None, "outside the ellipse");
        assert_eq!(
            g.text_slot_at(v(30.0, 60.0)),
            Some(TextSlot::Cell { col: 1, row: 2 })
        );
        assert_eq!(line.text_slot_at(v(5.0, 5.0)), None);
        assert_eq!(r.text_slot_at(v(f32::NAN, 0.0)), None);
    }

    #[test]
    fn text_frame_is_scaled_box_inscribed_rect_or_cell() {
        // Rectangle: its box scaled by SHAPE_TEXT_FILL about the centre.
        let r = rect(v(0.0, 0.0), v(100.0, 50.0), 1.0, None);
        let Some(frame) = r.text_frame(TextSlot::Shape) else {
            panic!("a rectangle has a text frame");
        };
        assert!(approx_eq(SHAPE_TEXT_FILL, 0.8, 1e-6));
        assert_aabb_eq(frame, v(10.0, 5.0), v(90.0, 45.0));

        // Ellipse: the inscribed rectangle (box / sqrt 2), scaled.
        let e = ellipse(v(0.0, 0.0), v(100.0, 100.0), 1.0, None);
        let Some(frame) = e.text_frame(TextSlot::Shape) else {
            panic!("an ellipse has a text frame");
        };
        let half = 50.0 / 2.0_f32.sqrt() * 0.8;
        assert_aabb_eq(
            frame,
            v(50.0 - half, 50.0 - half),
            v(50.0 + half, 50.0 + half),
        );

        // Grid cell (1, 2) of a 4 × 4 grid over 100 × 100.
        let g = grid(v(0.0, 0.0), v(100.0, 100.0), 4, 4, 1.0);
        let Some(frame) = g.text_frame(TextSlot::Cell { col: 1, row: 2 }) else {
            panic!("a grid cell has a text frame");
        };
        assert_aabb_eq(frame, v(27.5, 52.5), v(47.5, 72.5));

        // Corner `a` at the bottom right: cell (0, 0) is there.
        let flipped = grid(v(100.0, 100.0), v(0.0, 0.0), 4, 4, 1.0);
        let Some(frame) = flipped.text_frame(TextSlot::Cell { col: 0, row: 0 }) else {
            panic!("a grid cell has a text frame");
        };
        assert_aabb_eq(frame, v(77.5, 77.5), v(97.5, 97.5));

        // No frame for a missing cell, a wrong slot or another shape.
        assert!(g.text_frame(TextSlot::Cell { col: 4, row: 0 }).is_none());
        assert!(g.text_frame(TextSlot::Shape).is_none());
        assert!(r.text_frame(TextSlot::Cell { col: 0, row: 0 }).is_none());
    }

    #[test]
    fn shape_texts_are_centred_and_fitted_in_their_frames() {
        // Arrange: "ab" at size 10 is 12 × 12.5 and fits the 80 × 40 frame.
        let r = rect(v(0.0, 0.0), v(100.0, 50.0), 1.0, None)
            .with_slot_text(TextSlot::Shape, st("ab", 10.0));

        // Act
        let placed = r.placed_texts();

        // Assert
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].text, "ab");
        assert!(approx_eq(placed[0].size, 10.0, EPS));
        assert_vec_eq(placed[0].origin, v(44.0, 18.75));

        // Too wide: ten cells at size 20 shrink to 80 / 6.
        let wide = rect(v(0.0, 0.0), v(100.0, 50.0), 1.0, None)
            .with_slot_text(TextSlot::Shape, st("abcdefghij", 20.0));
        let placed = wide.placed_texts();
        assert!(approx_eq(placed[0].size, 80.0 / 6.0, 1e-3));
        assert_vec_eq(placed[0].origin, v(10.0, 25.0 - 1.25 * 40.0 / 6.0));

        // Grid cells, in (row, col) order.
        let g = grid(v(0.0, 0.0), v(100.0, 100.0), 4, 4, 1.0)
            .with_slot_text(TextSlot::Cell { col: 1, row: 1 }, st("b", 4.0))
            .with_slot_text(TextSlot::Cell { col: 0, row: 0 }, st("a", 4.0));
        let placed = g.placed_texts();
        assert_eq!(placed.len(), 2);
        assert_eq!(placed[0].text, "a");
        assert_vec_eq(placed[0].origin, v(12.5 - 1.2, 12.5 - 2.5));
        assert_eq!(placed[1].text, "b");

        // A shape without text places nothing.
        assert!(
            rect(v(0.0, 0.0), v(9.0, 9.0), 1.0, None)
                .placed_texts()
                .is_empty()
        );
    }

    #[test]
    fn grid_cell_step_follows_reading_order_on_screen() {
        // Arrange: 3 × 2, `a` top-left.
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 1.0);

        // Act / Assert
        assert_eq!(g.grid_cell_step(0, 0, CellStep::Next), Some((1, 0)));
        assert_eq!(g.grid_cell_step(2, 0, CellStep::Next), Some((0, 1)));
        assert_eq!(
            g.grid_cell_step(2, 1, CellStep::Next),
            Some((0, 0)),
            "wraps"
        );
        assert_eq!(
            g.grid_cell_step(0, 0, CellStep::Prev),
            Some((2, 1)),
            "wraps"
        );
        assert_eq!(g.grid_cell_step(0, 1, CellStep::Prev), Some((2, 0)));
        assert_eq!(g.grid_cell_step(1, 0, CellStep::Down), Some((1, 1)));
        assert_eq!(g.grid_cell_step(1, 1, CellStep::Down), None, "bottom row");
        assert_eq!(g.grid_cell_step(1, 1, CellStep::Up), Some((1, 0)));
        assert_eq!(g.grid_cell_step(1, 0, CellStep::Up), None, "top row");

        // `a` bottom-right: col 0 is the right column, row 0 the bottom row.
        let flipped = grid(v(30.0, 20.0), v(0.0, 0.0), 3, 2, 1.0);
        assert_eq!(flipped.grid_cell_step(2, 1, CellStep::Next), Some((1, 1)));
        assert_eq!(flipped.grid_cell_step(0, 1, CellStep::Next), Some((2, 0)));
        assert_eq!(flipped.grid_cell_step(2, 1, CellStep::Down), Some((2, 0)));
        assert_eq!(flipped.grid_cell_step(2, 0, CellStep::Down), None);

        // Out of range or not a grid.
        assert_eq!(g.grid_cell_step(3, 0, CellStep::Next), None);
        let r = rect(v(0.0, 0.0), v(9.0, 9.0), 1.0, None);
        assert_eq!(r.grid_cell_step(0, 0, CellStep::Next), None);
    }

    #[test]
    fn translate_keeps_shape_and_cell_text() {
        // Arrange
        let mut r = rect(v(0.0, 0.0), v(100.0, 50.0), 1.0, None)
            .with_slot_text(TextSlot::Shape, st("ab", 10.0));
        let mut g = grid(v(0.0, 0.0), v(100.0, 100.0), 4, 4, 1.0)
            .with_slot_text(TextSlot::Cell { col: 1, row: 1 }, st("b", 4.0));
        let (r0, g0) = (r.placed_texts()[0].origin, g.placed_texts()[0].origin);

        // Act
        r.translate(v(5.0, -3.0));
        g.translate(v(5.0, -3.0));

        // Assert
        assert_eq!(r.slot_text(TextSlot::Shape), st("ab", 10.0).as_ref());
        assert_vec_eq(r.placed_texts()[0].origin, r0 + v(5.0, -3.0));
        assert_vec_eq(g.placed_texts()[0].origin, g0 + v(5.0, -3.0));
    }
}
