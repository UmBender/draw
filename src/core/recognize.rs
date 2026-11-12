//! Shape recognition: the line, rectangle or ellipse a pen stroke looks
//! like, or nothing (ADR-T109-1).
//!
//! [`fit`] resamples the stroke evenly along its length, then tries, in
//! order: a straight line (open, nearly straight); a closed loop measured
//! in its minimum-area box, which is a rectangle when it passes near all
//! four box corners and hugs the box, or an ellipse when it passes near no
//! corner and hugs the inscribed ellipse. Anything else is no fit: a wrong
//! guess costs more than redrawing. Every length limit is a fraction of
//! the stroke's own size, so the result does not depend on zoom. The maths
//! runs in `f64`, so huge coordinates cannot overflow, and the returned
//! points come from the stroke's own `f32` points, so they are finite.

use crate::core::geom::Vec2;
use crate::core::shape::Shape;

/// Samples taken evenly along the stroke before measuring.
pub const SAMPLES: usize = 128;
/// A line's chord is at least this fraction of the path length.
pub const LINE_MIN_STRAIGHTNESS: f32 = 0.75;
/// No sample of a line lies farther than this fraction of the chord from
/// it.
pub const LINE_MAX_DEVIATION: f32 = 0.1;
/// A loop is closed when its ends are at most this fraction of the path
/// length apart.
pub const CLOSED_MAX_GAP: f32 = 0.3;
/// Rotations of the measuring box tried over a quarter turn.
pub const ROTATION_STEPS: u32 = 90;
/// The short side of a closed shape's box is at least this fraction of its
/// long side.
pub const MIN_ASPECT: f32 = 0.1;
/// A closed shape winds around its box centre at least this fraction of a
/// full turn.
pub const MIN_TURN: f32 = 0.7;
/// A box corner counts for a rectangle when a sample lies within this
/// fraction of the box diagonal.
pub const RECT_CORNER_TOLERANCE: f32 = 0.12;
/// A rectangle passes near at least this many of the four box corners.
pub const RECT_MIN_CORNERS: usize = 3;
/// A box corner counts against an ellipse when a sample lies within this
/// fraction of the box diagonal.
pub const ELLIPSE_CORNER_TOLERANCE: f32 = 0.06;
/// An ellipse passes near at most this many of the four box corners.
pub const ELLIPSE_MAX_CORNERS: usize = 1;
/// When both a rectangle and an ellipse fit, the better one wins only if
/// its score is below this fraction of the other's; otherwise the stroke
/// stays as drawn.
pub const NEAR_TIE: f32 = 0.9;
/// The largest residual is read after leaving out this fraction of the
/// worst samples, so a short tail or a flick does not veto the shape.
pub const TAIL_ALLOWANCE: f32 = 0.04;
/// Mean distance of the samples to the outline, as a fraction of the box
/// diagonal.
pub const MEAN_RESIDUAL: f32 = 0.05;
/// Largest distance of a sample to the outline, as a fraction of the box
/// diagonal.
pub const MAX_RESIDUAL: f32 = 0.14;

/// The shape a stroke looks like. For [`Fit::Rect`] and [`Fit::Ellipse`],
/// `a` and `b` are opposite corners of the stroke's axis-aligned bounding
/// box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fit {
    /// A straight segment from the stroke's first to its last point.
    Line {
        /// Start point.
        a: Vec2,
        /// End point.
        b: Vec2,
    },
    /// A rectangle.
    Rect {
        /// Smallest corner.
        a: Vec2,
        /// Largest corner.
        b: Vec2,
    },
    /// An ellipse inscribed in the box.
    Ellipse {
        /// Smallest corner of the box.
        a: Vec2,
        /// Largest corner of the box.
        b: Vec2,
    },
}

/// The shape the polyline `points` looks like, or `None` when it fits none
/// well enough or is degenerate (fewer than two points, a non-finite
/// point, no length).
#[must_use]
pub fn fit(points: &[Vec2]) -> Option<Fit> {
    let (&first, &last) = (points.first()?, points.last()?);
    if points.len() < 2 || points.iter().any(|p| !p.is_finite()) {
        return None;
    }
    let path: Vec<P> = points.iter().map(|&p| P::from(p)).collect();
    let length: f64 = path.windows(2).map(|w| w[0].distance(w[1])).sum();
    if !(length > 0.0 && length.is_finite()) {
        return None;
    }
    let samples = resample(&path, length, SAMPLES);
    let (start, end) = (P::from(first), P::from(last));
    let chord = start.distance(end);

    let straight = chord >= f64::from(LINE_MIN_STRAIGHTNESS) * length;
    let max_off = f64::from(LINE_MAX_DEVIATION) * chord;
    if straight
        && samples
            .iter()
            .all(|&p| p.distance_to_segment(start, end) <= max_off)
    {
        return Some(Fit::Line { a: first, b: last });
    }
    if chord > f64::from(CLOSED_MAX_GAP) * length {
        return None;
    }
    let kind = closed_kind(&samples)?;
    let bounds = crate::core::geom::Aabb::from_points(points)?;
    let (a, b) = (bounds.min, bounds.max);
    Some(match kind {
        Closed::Rect => Fit::Rect { a, b },
        Closed::Ellipse => Fit::Ellipse { a, b },
    })
}

/// The line, rectangle or ellipse replacing `shape`, with its colour and
/// width; `None` for anything but a [`Shape::Stroke`] that fits.
#[must_use]
pub fn recognize(shape: &Shape) -> Option<Shape> {
    let Shape::Stroke { points, style, .. } = shape else {
        return None;
    };
    let style = *style;
    Some(match fit(points)? {
        Fit::Line { a, b } => Shape::Line { a, b, style },
        Fit::Rect { a, b } => Shape::Rect {
            a,
            b,
            style,
            fill: None,
            label: None,
            text: None,
        },
        Fit::Ellipse { a, b } => Shape::Ellipse {
            a,
            b,
            style,
            fill: None,
            label: None,
            text: None,
        },
    })
}

/// A point in `f64`, so squares of huge `f32` coordinates stay finite.
#[derive(Debug, Clone, Copy, PartialEq)]
struct P {
    x: f64,
    y: f64,
}

impl From<Vec2> for P {
    fn from(p: Vec2) -> Self {
        Self {
            x: f64::from(p.x),
            y: f64::from(p.y),
        }
    }
}

impl P {
    fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    fn lerp(self, other: Self, t: f64) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Distance to the segment `a`–`b` (to `a` when it has no length).
    fn distance_to_segment(self, a: Self, b: Self) -> f64 {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len_sq = dx * dx + dy * dy;
        if len_sq <= 0.0 {
            return self.distance(a);
        }
        let t = (((self.x - a.x) * dx + (self.y - a.y) * dy) / len_sq).clamp(0.0, 1.0);
        self.distance(a.lerp(b, t))
    }
}

/// `n` (at least 2) points evenly spaced along `path`, whose length is
/// `length` (positive), from its first to its last point.
fn resample(path: &[P], length: f64, n: usize) -> Vec<P> {
    let n = n.max(2);
    let mut out = Vec::with_capacity(n);
    let mut seg = 0;
    // Path length up to `path[seg]`.
    let mut walked = 0.0;
    for k in 0..n {
        let at = length * k as f64 / (n - 1) as f64;
        while seg + 2 < path.len() && walked + path[seg].distance(path[seg + 1]) < at {
            walked += path[seg].distance(path[seg + 1]);
            seg += 1;
        }
        let (from, to) = (path[seg], path[seg + 1]);
        let step = from.distance(to);
        let along = if step > 0.0 {
            ((at - walked) / step).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push(from.lerp(to, along));
    }
    out
}

/// What a closed loop looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Closed {
    Rect,
    Ellipse,
}

/// The rotation (in radians, in `[0, π/2)`) of the smallest-area box
/// around `points`, ties to the smallest angle.
fn min_area_rotation(points: &[P]) -> f64 {
    let mut best: Option<(f64, f64)> = None;
    for step in 0..ROTATION_STEPS {
        let angle = std::f64::consts::FRAC_PI_2 * f64::from(step) / f64::from(ROTATION_STEPS);
        let (u, v) = extents(points, angle);
        let area = u * v;
        if best.is_none_or(|(least, _)| area < least * (1.0 - 1e-9)) {
            best = Some((area, angle));
        }
    }
    best.map_or(0.0, |(_, angle)| angle)
}

/// `p` in the frame turned by `angle`.
fn turn(p: P, angle: f64) -> P {
    let (s, c) = angle.sin_cos();
    P {
        x: p.x * c + p.y * s,
        y: -p.x * s + p.y * c,
    }
}

/// Width and height of the box around `points` in the frame turned by
/// `angle`.
fn extents(points: &[P], angle: f64) -> (f64, f64) {
    let (lo, hi) = bounds(points.iter().map(|&p| turn(p, angle)));
    (hi.x - lo.x, hi.y - lo.y)
}

/// Smallest and largest coordinates of `points` (zero for none).
fn bounds(points: impl Iterator<Item = P>) -> (P, P) {
    let mut lo = P {
        x: f64::INFINITY,
        y: f64::INFINITY,
    };
    let mut hi = P {
        x: f64::NEG_INFINITY,
        y: f64::NEG_INFINITY,
    };
    for p in points {
        lo = P {
            x: lo.x.min(p.x),
            y: lo.y.min(p.y),
        };
        hi = P {
            x: hi.x.max(p.x),
            y: hi.y.max(p.y),
        };
    }
    if lo.x > hi.x {
        let zero = P { x: 0.0, y: 0.0 };
        return (zero, zero);
    }
    (lo, hi)
}

/// How badly the residuals of the samples fit a box of diagonal
/// `diagonal`: the larger of the mean and the largest residual (leaving
/// out the [`TAIL_ALLOWANCE`] worst samples), each over its limit. A score
/// of `1` or less passes; a lower score is a better fit (ADR-T117-1).
fn score(residuals: impl Iterator<Item = f64>, diagonal: f64) -> f64 {
    let mut all: Vec<f64> = residuals.collect();
    all.sort_by(f64::total_cmp);
    let count = all.len().max(1);
    let mean = all.iter().sum::<f64>() / count as f64;
    let kept = count - (count as f64 * f64::from(TAIL_ALLOWANCE)).floor() as usize;
    let largest = all.get(kept.max(1) - 1).copied().unwrap_or(0.0);
    (mean / (f64::from(MEAN_RESIDUAL) * diagonal))
        .max(largest / (f64::from(MAX_RESIDUAL) * diagonal))
}

/// The rectangle or ellipse a closed loop of evenly spaced `samples`
/// looks like, measured in its smallest-area box (ADR-T109-1).
fn closed_kind(samples: &[P]) -> Option<Closed> {
    // Centre first: precision for far-away strokes.
    let count = samples.len().max(1) as f64;
    let mean = samples.iter().fold(P { x: 0.0, y: 0.0 }, |acc, p| P {
        x: acc.x + p.x / count,
        y: acc.y + p.y / count,
    });
    let centred: Vec<P> = samples
        .iter()
        .map(|p| P {
            x: p.x - mean.x,
            y: p.y - mean.y,
        })
        .collect();
    let angle = min_area_rotation(&centred);
    let turned: Vec<P> = centred.iter().map(|&p| turn(p, angle)).collect();
    let (lo, hi) = bounds(turned.iter().copied());
    let (hu, hv) = ((hi.x - lo.x) / 2.0, (hi.y - lo.y) / 2.0);
    let long = hu.max(hv);
    if !(long > 0.0 && long.is_finite()) || hu.min(hv) < f64::from(MIN_ASPECT) * long {
        return None;
    }
    let centre = P {
        x: f64::midpoint(lo.x, hi.x),
        y: f64::midpoint(lo.y, hi.y),
    };
    let local: Vec<P> = turned
        .iter()
        .map(|p| P {
            x: p.x - centre.x,
            y: p.y - centre.y,
        })
        .collect();
    let diagonal = 2.0 * hu.hypot(hv);

    if winding(&local, hu, hv) < f64::from(MIN_TURN) * std::f64::consts::TAU {
        return None;
    }
    let corners_within = |tolerance: f32| {
        let reach = f64::from(tolerance) * diagonal;
        [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .into_iter()
            .filter(|&(sx, sy)| {
                let corner = P {
                    x: sx * hu,
                    y: sy * hv,
                };
                local.iter().any(|p| p.distance(corner) <= reach)
            })
            .count()
    };
    let rect = (corners_within(RECT_CORNER_TOLERANCE) >= RECT_MIN_CORNERS)
        .then(|| {
            score(
                local
                    .iter()
                    .map(|p| (hu - p.x.abs()).min(hv - p.y.abs()).max(0.0)),
                diagonal,
            )
        })
        .filter(|&s| s <= 1.0);
    let ellipse = (corners_within(ELLIPSE_CORNER_TOLERANCE) <= ELLIPSE_MAX_CORNERS)
        .then(|| {
            score(
                local.iter().map(|p| {
                    let r = (p.x / hu).hypot(p.y / hv);
                    if r > 0.0 {
                        p.x.hypot(p.y) * (1.0 - 1.0 / r).abs()
                    } else {
                        hu.min(hv)
                    }
                }),
                diagonal,
            )
        })
        .filter(|&s| s <= 1.0);
    match (rect, ellipse) {
        (Some(_), None) => Some(Closed::Rect),
        (None, Some(_)) => Some(Closed::Ellipse),
        (Some(r), Some(e)) if r < f64::from(NEAR_TIE) * e => Some(Closed::Rect),
        (Some(r), Some(e)) if e < f64::from(NEAR_TIE) * r => Some(Closed::Ellipse),
        _ => None,
    }
}

/// The absolute angle, in radians, that `points` sweep around the origin,
/// measured on the box of half sides `hu`, `hv` scaled to a square.
fn winding(points: &[P], hu: f64, hv: f64) -> f64 {
    let mut total = 0.0;
    let mut previous: Option<f64> = None;
    for p in points {
        let (x, y) = (p.x / hu, p.y / hv);
        if x == 0.0 && y == 0.0 {
            continue;
        }
        let angle = y.atan2(x);
        if let Some(before) = previous {
            let mut delta = angle - before;
            if delta > std::f64::consts::PI {
                delta -= std::f64::consts::TAU;
            } else if delta < -std::f64::consts::PI {
                delta += std::f64::consts::TAU;
            }
            total += delta;
        }
        previous = Some(angle);
    }
    total.abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Aabb, Vec2, approx_eq};
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use proptest::prelude::*;
    use std::f32::consts::{FRAC_PI_2, PI, TAU};

    /// Deterministic noise in `[-1, 1]` (a 64-bit LCG).
    struct Noise(u64);

    impl Noise {
        fn next(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            // The top 24 bits, mapped to [-1, 1].
            let bits = (self.0 >> 40) as u32;
            (bits as f32 / 16_777_215.0) * 2.0 - 1.0
        }
    }

    fn rotate(p: Vec2, about: Vec2, angle: f32) -> Vec2 {
        let (s, c) = angle.sin_cos();
        let d = p - about;
        about + Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c)
    }

    /// A hand-drawn segment: a slow sideways wave of `wave` × length plus
    /// `jitter` pixels of noise.
    fn shaky_line(a: Vec2, b: Vec2, wave: f32, jitter: f32, seed: u64) -> Vec<Vec2> {
        let mut noise = Noise(seed);
        let span = b - a;
        let len = span.length();
        let normal = Vec2::new(-span.y, span.x) / len;
        let count = 60;
        (0..=count)
            .map(|i| {
                let t = i as f32 / count as f32;
                let off = wave * len * (PI * t).sin() * (2.0 * PI * t + 0.3).cos()
                    + jitter * noise.next();
                a + span * t + normal * off
            })
            .collect()
    }

    /// A hand-drawn ellipse: `turns` of a turn from angle `start`, the radii
    /// wobbling by `wobble` and `jitter` pixels of noise, rotated by `tilt`.
    #[allow(clippy::too_many_arguments)]
    fn shaky_ellipse(
        c: Vec2,
        rx: f32,
        ry: f32,
        start: f32,
        turns: f32,
        wobble: f32,
        jitter: f32,
        tilt: f32,
        seed: u64,
    ) -> Vec<Vec2> {
        let mut noise = Noise(seed);
        let count = 90;
        (0..=count)
            .map(|i| {
                let angle = start + TAU * turns * i as f32 / count as f32;
                let scale = 1.0 + wobble * (3.0 * angle + 0.7).sin();
                let on = c + Vec2::new(rx * scale * angle.cos(), ry * scale * angle.sin());
                let shaken = on + Vec2::new(noise.next(), noise.next()) * jitter;
                rotate(shaken, c, tilt)
            })
            .collect()
    }

    /// The point at `t` (`0..4`, one unit per side, clockwise from the
    /// top-left corner) on the outline of the box `min`–`max`.
    fn on_box(min: Vec2, max: Vec2, t: f32) -> Vec2 {
        let t = t.rem_euclid(4.0);
        let side = t.floor();
        let f = t - side;
        let (w, h) = (max.x - min.x, max.y - min.y);
        match side as u32 {
            0 => Vec2::new(min.x + w * f, min.y),
            1 => Vec2::new(max.x, min.y + h * f),
            2 => Vec2::new(max.x - w * f, max.y),
            _ => Vec2::new(min.x, max.y - h * f),
        }
    }

    /// A hand-drawn rectangle from `start` (in sides) around `turns` of the
    /// outline, `jitter` pixels of noise, rotated by `tilt` about its
    /// centre.
    fn shaky_rect(
        min: Vec2,
        max: Vec2,
        start: f32,
        turns: f32,
        jitter: f32,
        tilt: f32,
        seed: u64,
    ) -> Vec<Vec2> {
        let mut noise = Noise(seed);
        let c = (min + max) * 0.5;
        let n = 120;
        (0..=n)
            .map(|i| {
                let t = start + 4.0 * turns * i as f32 / n as f32;
                let p = on_box(min, max, t) + Vec2::new(noise.next(), noise.next()) * jitter;
                rotate(p, c, tilt)
            })
            .collect()
    }

    fn corners(fit: Option<Fit>) -> Option<(Vec2, Vec2)> {
        match fit {
            Some(Fit::Line { a, b } | Fit::Rect { a, b } | Fit::Ellipse { a, b }) => Some((a, b)),
            None => None,
        }
    }

    /// Asserts the corners `a`–`b` span the box of `points`.
    fn assert_box(a: Vec2, b: Vec2, points: &[Vec2]) {
        let Some(expected) = Aabb::from_points(points) else {
            panic!("no points");
        };
        let got = Aabb::from_corners(a, b);
        assert!(
            got.min.approx_eq(expected.min, 1e-3),
            "{got:?} vs {expected:?}"
        );
        assert!(
            got.max.approx_eq(expected.max, 1e-3),
            "{got:?} vs {expected:?}"
        );
    }

    fn style() -> Style {
        Style {
            color: ColorId::new(2).unwrap_or(ColorId::INK),
            width: 4.0,
        }
    }

    fn stroke(points: Vec<Vec2>) -> Shape {
        let widths = Some(vec![0.8; points.len()]);
        Shape::Stroke {
            points,
            widths,
            style: style(),
        }
    }

    // ---- T117: sloppy strokes ------------------------------------------------

    /// What a sloppy family should become.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Kind {
        Line,
        Rect,
        Ellipse,
    }

    fn kind_of(fit: Option<Fit>) -> Option<Kind> {
        match fit? {
            Fit::Line { .. } => Some(Kind::Line),
            Fit::Rect { .. } => Some(Kind::Rect),
            Fit::Ellipse { .. } => Some(Kind::Ellipse),
        }
    }

    /// The point `walked` pixels along the clockwise outline of a
    /// `width` × `height` rectangle whose corners are quarter circles of
    /// `radius`, starting at the top-left of the top side.
    fn rounded_point(walked: f32, width: f32, height: f32, radius: f32) -> Vec2 {
        let (side_w, side_h) = (width - 2.0 * radius, height - 2.0 * radius);
        let arc = FRAC_PI_2 * radius;
        let mut rest = walked;
        let mut take = |length: f32| {
            let inside = rest <= length;
            if !inside {
                rest -= length;
            }
            inside
        };
        let corner = |centre: Vec2, start: f32, along: f32| {
            let angle = start + along / radius;
            centre + Vec2::new(radius * angle.cos(), radius * angle.sin())
        };
        if take(side_w) {
            return Vec2::new(radius + rest, 0.0);
        }
        if take(arc) {
            return corner(Vec2::new(width - radius, radius), -FRAC_PI_2, rest);
        }
        if take(side_h) {
            return Vec2::new(width, radius + rest);
        }
        if take(arc) {
            return corner(Vec2::new(width - radius, height - radius), 0.0, rest);
        }
        if take(side_w) {
            return Vec2::new(width - radius - rest, height);
        }
        if take(arc) {
            return corner(Vec2::new(radius, height - radius), FRAC_PI_2, rest);
        }
        if take(side_h) {
            return Vec2::new(0.0, height - radius - rest);
        }
        corner(Vec2::new(radius, radius), PI, rest)
    }

    /// A rectangle outline with quarter-circle corners of radius `round`
    /// (a fraction of the shorter side), `jitter` pixels of noise.
    fn rounded_rect(width: f32, height: f32, round: f32, jitter: f32, seed: u64) -> Vec<Vec2> {
        let mut noise = Noise(seed);
        let radius = round * width.min(height);
        let perimeter = 2.0 * (width + height - 4.0 * radius) + TAU * radius;
        let steps = 200;
        (0..=steps)
            .map(|i| {
                let walked = perimeter * i as f32 / steps as f32;
                rounded_point(walked, width, height, radius)
                    + Vec2::new(noise.next(), noise.next()) * jitter
            })
            .collect()
    }

    /// `(name, expected, builder)` of every sloppy family. The builder gets
    /// a noise level (`1.0` is the level the family must survive) and a
    /// seed.
    #[allow(clippy::type_complexity, clippy::too_many_lines)]
    fn families() -> Vec<(&'static str, Kind, fn(f32, u64) -> Vec<Vec2>)> {
        vec![
            ("hurried circle", Kind::Ellipse, |k, seed| {
                shaky_ellipse(
                    Vec2::ZERO,
                    100.0,
                    100.0,
                    0.7,
                    1.0,
                    0.06 * k,
                    3.0 * k,
                    0.0,
                    seed,
                )
            }),
            ("circle that overshoots", Kind::Ellipse, |k, seed| {
                shaky_ellipse(
                    Vec2::ZERO,
                    100.0,
                    100.0,
                    1.9,
                    1.18,
                    0.04 * k,
                    2.0 * k,
                    0.0,
                    seed,
                )
            }),
            ("circle that stops short", Kind::Ellipse, |k, seed| {
                shaky_ellipse(
                    Vec2::ZERO,
                    100.0,
                    100.0,
                    4.0,
                    0.86,
                    0.04 * k,
                    2.0 * k,
                    0.0,
                    seed,
                )
            }),
            ("circle with a tail", Kind::Ellipse, |k, seed| {
                let mut points = shaky_ellipse(
                    Vec2::ZERO,
                    100.0,
                    100.0,
                    0.7,
                    1.0,
                    0.04 * k,
                    2.0 * k,
                    0.0,
                    seed,
                );
                let last = points[points.len() - 1];
                points.extend((1..=5).map(|i| last + Vec2::new(0.0, -3.0 * i as f32)));
                points
            }),
            ("squashed ellipse", Kind::Ellipse, |k, seed| {
                shaky_ellipse(
                    Vec2::ZERO,
                    220.0,
                    90.0,
                    0.2,
                    1.1,
                    0.05 * k,
                    3.0 * k,
                    0.2,
                    seed,
                )
            }),
            ("lopsided rectangle", Kind::Rect, |k, seed| {
                shaky_rect(
                    Vec2::ZERO,
                    Vec2::new(260.0, 150.0),
                    0.4,
                    1.05,
                    6.0 * k,
                    0.12 * k,
                    seed,
                )
            }),
            ("rectangle that stops short", Kind::Rect, |k, seed| {
                shaky_rect(
                    Vec2::ZERO,
                    Vec2::new(200.0, 200.0),
                    0.0,
                    0.9,
                    4.0 * k,
                    0.0,
                    seed,
                )
            }),
            ("rectangle that overshoots", Kind::Rect, |k, seed| {
                shaky_rect(
                    Vec2::ZERO,
                    Vec2::new(240.0, 160.0),
                    2.5,
                    1.16,
                    4.0 * k,
                    0.05 * k,
                    seed,
                )
            }),
            ("rounded rectangle", Kind::Rect, |k, seed| {
                rounded_rect(280.0, 180.0, 0.14, 3.0 * k, seed)
            }),
            ("wobbly line", Kind::Line, |k, seed| {
                shaky_line(Vec2::ZERO, Vec2::new(400.0, 60.0), 0.05 * k, 4.0 * k, seed)
            }),
            ("curved line", Kind::Line, |k, seed| {
                shaky_line(Vec2::ZERO, Vec2::new(300.0, 0.0), 0.09 * k, 1.0, seed)
            }),
        ]
    }

    /// Points on the superellipse `|x|^p + |y|^p = 1` scaled to `rx`, `ry`:
    /// a circle at `p = 2`, a rounded square as `p` grows.
    fn superellipse(power: f32, rx: f32, ry: f32, jitter: f32, seed: u64) -> Vec<Vec2> {
        let mut noise = Noise(seed);
        (0..=120)
            .map(|i| {
                let angle = TAU * i as f32 / 120.0;
                let (cos, sin) = (angle.cos(), angle.sin());
                let px = cos.signum() * cos.abs().powf(2.0 / power);
                let py = sin.signum() * sin.abs().powf(2.0 / power);
                Vec2::new(rx * px, ry * py) + Vec2::new(noise.next(), noise.next()) * jitter
            })
            .collect()
    }

    /// Asserts that the family called `name` becomes its shape at the
    /// level `1.0` of noise, for every seed.
    fn assert_family_is_recognised(name: &str) {
        let Some((_, kind, build)) = families().into_iter().find(|(n, _, _)| *n == name) else {
            panic!("no family {name}");
        };
        for seed in 0..16 {
            let fit = fit(&build(1.0, seed));
            assert_eq!(kind_of(fit), Some(kind), "{name}, seed {seed}: {fit:?}");
        }
    }

    #[test]
    fn fit_sloppy_hurried_circle_gives_an_ellipse() {
        assert_family_is_recognised("hurried circle");
    }

    #[test]
    fn fit_sloppy_circle_that_overshoots_gives_an_ellipse() {
        assert_family_is_recognised("circle that overshoots");
    }

    #[test]
    fn fit_sloppy_circle_that_stops_short_gives_an_ellipse() {
        assert_family_is_recognised("circle that stops short");
    }

    #[test]
    fn fit_sloppy_circle_with_a_tail_gives_an_ellipse() {
        assert_family_is_recognised("circle with a tail");
    }

    #[test]
    fn fit_sloppy_squashed_ellipse_gives_an_ellipse() {
        assert_family_is_recognised("squashed ellipse");
    }

    #[test]
    fn fit_sloppy_lopsided_rectangle_gives_a_rect() {
        assert_family_is_recognised("lopsided rectangle");
    }

    #[test]
    fn fit_sloppy_rectangle_that_stops_short_gives_a_rect() {
        assert_family_is_recognised("rectangle that stops short");
    }

    #[test]
    fn fit_sloppy_rectangle_that_overshoots_gives_a_rect() {
        assert_family_is_recognised("rectangle that overshoots");
    }

    #[test]
    fn fit_sloppy_rounded_rectangle_gives_a_rect() {
        assert_family_is_recognised("rounded rectangle");
    }

    #[test]
    fn fit_sloppy_wobbly_line_gives_a_line() {
        assert_family_is_recognised("wobbly line");
    }

    #[test]
    fn fit_sloppy_curved_line_gives_a_line() {
        assert_family_is_recognised("curved line");
    }

    /// The first noise level (in steps of `0.1`) at which a family fails
    /// for one of 16 seeds; `None` if it never does up to `6.0`.
    fn first_failing_level(build: fn(f32, u64) -> Vec<Vec2>, kind: Kind) -> Option<f32> {
        (1..=60)
            .map(|step| step as f32 * 0.1)
            .find(|&k| !(0..16u64).all(|seed| kind_of(fit(&build(k, seed))) == Some(kind)))
    }

    /// Where each family stops being recognised, in units of its own noise
    /// (`1.0` is the level it must survive): ADR-T117-1 lists the same
    /// numbers, so a change of the limits shows up here as a diff.
    const TOLERANCE: [(&str, f32); 11] = [
        ("hurried circle", 2.9),
        ("circle that overshoots", 4.4),
        ("circle that stops short", 3.6),
        ("circle with a tail", 4.3),
        ("squashed ellipse", 1.5),
        ("lopsided rectangle", 2.6),
        ("rectangle that stops short", 4.6),
        ("rectangle that overshoots", 3.3),
        ("rounded rectangle", 3.8),
        ("wobbly line", 1.6),
        ("curved line", 1.1),
    ];

    #[test]
    fn tolerance_table_matches_the_adr() {
        let all = families();
        assert_eq!(all.len(), TOLERANCE.len());
        for (name, kind, build) in all {
            let Some(&(_, expected)) = TOLERANCE.iter().find(|(n, _)| *n == name) else {
                panic!("{name} is not in the table");
            };
            let got = first_failing_level(build, kind).unwrap_or(f32::INFINITY);
            assert!(
                approx_eq(got, expected, 0.05),
                "{name}: {got} vs {expected}"
            );
        }
    }

    /// Strokes that sit nearest the new limits and must stay strokes.
    fn near_misses() -> Vec<(&'static str, Vec<Vec2>)> {
        let walk = |corners: &[Vec2]| -> Vec<Vec2> {
            let mut points = Vec::new();
            for pair in corners.windows(2) {
                for i in 0..30 {
                    points.push(pair[0].lerp(pair[1], i as f32 / 30.0));
                }
            }
            points.push(corners[corners.len() - 1]);
            points
        };
        vec![
            (
                "an L",
                walk(&[
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.0, 200.0),
                    Vec2::new(150.0, 200.0),
                ]),
            ),
            (
                "a U",
                walk(&[
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.0, 200.0),
                    Vec2::new(200.0, 200.0),
                    Vec2::new(200.0, 0.0),
                ]),
            ),
            (
                "an open C",
                shaky_ellipse(Vec2::ZERO, 100.0, 100.0, 0.5, 0.7, 0.0, 0.5, 0.0, 3),
            ),
            (
                "a half circle",
                shaky_ellipse(Vec2::ZERO, 100.0, 100.0, 0.0, 0.5, 0.0, 0.5, 0.0, 4),
            ),
            (
                "a line with a hook",
                walk(&[
                    Vec2::new(0.0, 0.0),
                    Vec2::new(250.0, 0.0),
                    Vec2::new(250.0, 120.0),
                    Vec2::new(300.0, 100.0),
                ]),
            ),
            (
                "a wave",
                (0..=60)
                    .map(|i| {
                        let t = i as f32 / 60.0;
                        Vec2::new(300.0 * t, 60.0 * (TAU * t).sin())
                    })
                    .collect(),
            ),
            (
                "a sloppy triangle",
                walk(&[
                    Vec2::new(0.0, 200.0),
                    Vec2::new(100.0, 0.0),
                    Vec2::new(210.0, 205.0),
                    Vec2::new(5.0, 195.0),
                ]),
            ),
        ]
    }

    #[test]
    fn fit_loose_still_rejects_the_near_misses() {
        for (name, points) in near_misses() {
            let fit = fit(&points);
            assert_eq!(fit, None, "{name}");
        }
    }

    #[test]
    fn fit_loose_still_rejects_noisy_scribbles() {
        // The scribbles of T109 with a hand's shake on top.
        for (name, points) in scribbles() {
            let mut noise = Noise(11);
            let shaken: Vec<Vec2> = points
                .iter()
                .map(|p| *p + Vec2::new(noise.next(), noise.next()) * 3.0)
                .collect();
            assert_eq!(fit(&shaken), None, "{name}");
        }
    }

    #[test]
    fn fit_rounded_square_picks_the_better_fit() {
        // A circle that squares off one step at a time: ellipse, then rect.
        let kind = |p: f32| kind_of(fit(&superellipse(p, 100.0, 100.0, 1.0, 0)));
        assert_eq!(kind(2.0), Some(Kind::Ellipse));
        assert_eq!(kind(2.5), Some(Kind::Ellipse));
        assert_eq!(kind(4.0), Some(Kind::Rect));
        assert_eq!(kind(8.0), Some(Kind::Rect));
    }

    #[test]
    fn fit_near_tie_gives_nothing() {
        // Between the two, where both fit alike, the stroke stays as drawn.
        let ties = (0..40)
            .map(|step| 2.0 + step as f32 * 0.05)
            .filter(|&p| fit(&superellipse(p, 100.0, 100.0, 1.0, 0)).is_none())
            .count();
        assert!(ties > 0, "no exponent leaves the stroke alone");
    }

    // ---- AC-2 lines, rectangles, ellipses ---------------------------------

    #[test]
    fn fit_shaky_lines_gives_lines() {
        let cases = [
            (Vec2::new(0.0, 0.0), Vec2::new(300.0, 0.0), 0.015, 1.0),
            (Vec2::new(10.0, 10.0), Vec2::new(410.0, 330.0), 0.02, 1.5),
            (Vec2::new(-50.0, 80.0), Vec2::new(-20.0, 60.0), 0.01, 0.05),
            (Vec2::new(0.0, 0.0), Vec2::new(0.0, 900.0), 0.02, 2.0),
            (Vec2::new(5.0, 5.0), Vec2::new(6.0, 5.0), 0.0, 0.0),
        ];
        for (seed, &(a, b, wave, jitter)) in cases.iter().enumerate() {
            // Arrange
            let points = shaky_line(a, b, wave, jitter, seed as u64);

            // Act
            let fit = fit(&points);

            // Assert
            assert!(
                matches!(fit, Some(Fit::Line { .. })),
                "case {seed}: {fit:?}"
            );
            let Some((fa, fb)) = corners(fit) else {
                continue;
            };
            assert!(fa.approx_eq(points[0], 1e-4), "case {seed}");
            assert!(fb.approx_eq(points[points.len() - 1], 1e-4), "case {seed}");
        }
    }

    #[test]
    fn fit_shaky_squares_and_rectangles_gives_rects() {
        let cases = [
            // min, max, start (sides), turns, jitter
            (Vec2::new(0.0, 0.0), Vec2::new(200.0, 200.0), 0.0, 1.0, 3.0),
            (Vec2::new(0.0, 0.0), Vec2::new(40.0, 40.0), 0.5, 0.97, 0.6),
            (
                Vec2::new(100.0, 50.0),
                Vec2::new(700.0, 650.0),
                2.3,
                1.03,
                8.0,
            ),
            (Vec2::new(0.0, 0.0), Vec2::new(300.0, 120.0), 1.0, 1.0, 2.5),
            (
                Vec2::new(-80.0, -400.0),
                Vec2::new(0.0, 0.0),
                3.6,
                0.98,
                1.5,
            ),
        ];
        for (seed, &(min, max, start, turns, jitter)) in cases.iter().enumerate() {
            // Arrange
            let points = shaky_rect(min, max, start, turns, jitter, 0.0, seed as u64);

            // Act
            let fit = fit(&points);

            // Assert
            assert!(
                matches!(fit, Some(Fit::Rect { .. })),
                "case {seed}: {fit:?}"
            );
            if let Some((a, b)) = corners(fit) {
                assert_box(a, b, &points);
            }
        }
    }

    #[test]
    fn fit_shaky_circles_and_ellipses_gives_ellipses() {
        let cases = [
            // centre, rx, ry, start, turns, wobble, jitter
            (Vec2::new(0.0, 0.0), 100.0, 100.0, 0.0, 1.0, 0.03, 2.0),
            (Vec2::new(50.0, 50.0), 20.0, 20.0, 1.0, 0.95, 0.04, 0.5),
            (Vec2::new(500.0, 300.0), 400.0, 400.0, 2.5, 1.05, 0.04, 6.0),
            (Vec2::new(0.0, 0.0), 300.0, 150.0, 4.0, 1.0, 0.03, 2.0),
            (Vec2::new(0.0, 0.0), 60.0, 200.0, 0.3, 0.97, 0.02, 1.0),
        ];
        for (seed, &(c, rx, ry, start, turns, wobble, jitter)) in cases.iter().enumerate() {
            // Arrange
            let points = shaky_ellipse(c, rx, ry, start, turns, wobble, jitter, 0.0, seed as u64);

            // Act
            let fit = fit(&points);

            // Assert
            assert!(
                matches!(fit, Some(Fit::Ellipse { .. })),
                "case {seed}: {fit:?}"
            );
            if let Some((a, b)) = corners(fit) {
                assert_box(a, b, &points);
            }
        }
    }

    /// Strokes that must stay strokes.
    fn scribbles() -> Vec<(&'static str, Vec<Vec2>)> {
        let mut noise = Noise(7);
        let zigzag = (0..=16)
            .map(|i| Vec2::new(if i % 2 == 0 { 0.0 } else { 200.0 }, i as f32 * 12.0))
            .collect();
        let spiral = (0..=270)
            .map(|i| {
                let a = i as f32 * TAU / 90.0;
                let r = 20.0 + a * 15.0;
                Vec2::new(r * a.cos(), r * a.sin())
            })
            .collect();
        let tri = [
            Vec2::new(0.0, 200.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(200.0, 200.0),
        ];
        let triangle = (0..=90)
            .map(|i| {
                let t = i as f32 / 30.0;
                let k = (t.floor() as usize).min(2);
                let f = t - k as f32;
                tri[k].lerp(tri[(k + 1) % 3], f)
            })
            .collect();
        let eight = (0..=120)
            .map(|i| {
                let a = i as f32 * TAU / 120.0;
                Vec2::new(150.0 * a.sin(), 80.0 * (2.0 * a).sin())
            })
            .collect();
        let mut pos = Vec2::ZERO;
        let walk = (0..200)
            .map(|_| {
                pos += Vec2::new(noise.next(), noise.next()) * 25.0;
                pos
            })
            .collect();
        let back_and_forth = (0..=40)
            .map(|i| {
                let t = (i % 20) as f32 / 20.0;
                let x = if (i / 20) % 2 == 0 { t } else { 1.0 - t };
                Vec2::new(x * 300.0, 2.0 * (i as f32 * 0.7).sin())
            })
            .collect();
        vec![
            ("zigzag", zigzag),
            ("spiral", spiral),
            ("triangle", triangle),
            ("figure eight", eight),
            ("random walk", walk),
            ("back and forth", back_and_forth),
        ]
    }

    #[test]
    fn fit_scribbles_gives_nothing() {
        for (name, points) in scribbles() {
            // Act
            let fit = fit(&points);

            // Assert
            assert_eq!(fit, None, "{name}");
        }
    }

    #[test]
    fn recognize_keeps_colour_and_width() {
        // Arrange
        let circle = stroke(shaky_ellipse(
            Vec2::new(0.0, 0.0),
            100.0,
            100.0,
            0.0,
            1.0,
            0.02,
            1.0,
            0.0,
            3,
        ));
        let square = stroke(shaky_rect(
            Vec2::ZERO,
            Vec2::new(100.0, 100.0),
            0.0,
            1.0,
            1.0,
            0.0,
            4,
        ));
        let line = stroke(shaky_line(Vec2::ZERO, Vec2::new(100.0, 40.0), 0.01, 0.5, 5));

        // Act
        let shapes = [recognize(&circle), recognize(&square), recognize(&line)];

        // Assert
        assert!(matches!(
            shapes[0],
            Some(Shape::Ellipse {
                fill: None,
                label: None,
                text: None,
                ..
            })
        ));
        assert!(matches!(
            shapes[1],
            Some(Shape::Rect {
                fill: None,
                label: None,
                text: None,
                ..
            })
        ));
        assert!(matches!(shapes[2], Some(Shape::Line { .. })));
        for shape in shapes.iter().flatten() {
            assert_eq!(shape.style(), style());
            assert!(approx_eq(shape.style().width, 4.0, 1e-6));
        }
    }

    #[test]
    fn recognize_ignores_non_strokes() {
        // Arrange
        let rect = Shape::Rect {
            a: Vec2::ZERO,
            b: Vec2::new(10.0, 10.0),
            style: style(),
            fill: None,
            label: None,
            text: None,
        };
        let line = Shape::Line {
            a: Vec2::ZERO,
            b: Vec2::new(10.0, 10.0),
            style: style(),
        };
        let (_, walk) = scribbles().swap_remove(4);

        // Act / Assert
        assert_eq!(recognize(&rect), None);
        assert_eq!(recognize(&line), None);
        assert_eq!(recognize(&stroke(walk)), None);
    }

    // ---- AC-4 rotation and open curves ------------------------------------

    #[test]
    fn fit_rotated_rect_gives_its_axis_aligned_box() {
        for (seed, tilt) in [0.2f32, 0.45, -0.6].into_iter().enumerate() {
            // Arrange
            let points = shaky_rect(
                Vec2::ZERO,
                Vec2::new(300.0, 160.0),
                0.4,
                1.0,
                2.0,
                tilt,
                seed as u64,
            );

            // Act
            let fit = fit(&points);

            // Assert
            assert!(
                matches!(fit, Some(Fit::Rect { .. })),
                "tilt {tilt}: {fit:?}"
            );
            if let Some((a, b)) = corners(fit) {
                assert_box(a, b, &points);
            }
        }
    }

    #[test]
    fn fit_rotated_ellipse_gives_its_axis_aligned_box() {
        for (seed, tilt) in [0.5f32, 0.8, -0.3].into_iter().enumerate() {
            // Arrange
            let points = shaky_ellipse(
                Vec2::new(10.0, 10.0),
                240.0,
                110.0,
                0.5,
                1.0,
                0.02,
                1.5,
                tilt,
                seed as u64,
            );

            // Act
            let fit = fit(&points);

            // Assert
            assert!(
                matches!(fit, Some(Fit::Ellipse { .. })),
                "tilt {tilt}: {fit:?}"
            );
            if let Some((a, b)) = corners(fit) {
                assert_box(a, b, &points);
            }
        }
    }

    #[test]
    fn fit_open_curves_never_close() {
        let c_arc = shaky_ellipse(Vec2::ZERO, 100.0, 100.0, 0.5, 0.75, 0.01, 0.5, 0.0, 1);
        let half = shaky_ellipse(Vec2::ZERO, 100.0, 100.0, 0.0, 0.5, 0.0, 0.0, 0.0, 2);
        let u = shaky_rect(Vec2::ZERO, Vec2::new(200.0, 200.0), 0.0, 0.75, 1.0, 0.0, 3);
        let l = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 200.0),
            Vec2::new(200.0, 200.0),
        ];
        for (name, points) in [("C", c_arc), ("half circle", half), ("U", u), ("L", l)] {
            // Act
            let fit = fit(&points);

            // Assert
            assert_eq!(fit, None, "{name}");
        }
    }

    // ---- AC-5 degenerate input --------------------------------------------

    #[test]
    fn fit_degenerate_strokes_gives_nothing() {
        let p = Vec2::new(3.0, 4.0);
        let cases: [(&str, Vec<Vec2>); 5] = [
            ("empty", vec![]),
            ("one point", vec![p]),
            ("all equal", vec![p; 50]),
            (
                "NaN",
                vec![p, Vec2::new(f32::NAN, 0.0), Vec2::new(10.0, 4.0)],
            ),
            ("infinite", vec![p, Vec2::new(f32::INFINITY, 4.0)]),
        ];
        for (name, points) in cases {
            assert_eq!(fit(&points), None, "{name}");
        }
    }

    fn fit_is_finite(fit: Option<Fit>) -> bool {
        corners(fit).is_none_or(|(a, b)| a.is_finite() && b.is_finite())
    }

    #[test]
    fn fit_huge_coordinates_stays_finite() {
        // Arrange
        let circle = shaky_ellipse(
            Vec2::new(1e37, -1e37),
            1e36,
            1e36,
            0.0,
            1.0,
            0.02,
            0.0,
            0.0,
            9,
        );
        let extremes = vec![
            Vec2::new(-f32::MAX, -f32::MAX),
            Vec2::new(f32::MAX, f32::MAX),
        ];
        let corners_max = vec![
            Vec2::new(-f32::MAX, -f32::MAX),
            Vec2::new(f32::MAX, -f32::MAX),
            Vec2::new(f32::MAX, f32::MAX),
            Vec2::new(-f32::MAX, f32::MAX),
            Vec2::new(-f32::MAX, -f32::MAX),
        ];

        // Act
        let fits = [fit(&circle), fit(&extremes), fit(&corners_max)];

        // Assert
        assert!(
            matches!(fits[0], Some(Fit::Ellipse { .. })),
            "{:?}",
            fits[0]
        );
        assert!(matches!(fits[1], Some(Fit::Line { .. })), "{:?}", fits[1]);
        for fit in fits {
            assert!(fit_is_finite(fit), "{fit:?}");
        }
    }

    fn arb_coord() -> impl Strategy<Value = f32> {
        prop_oneof![
            8 => -500.0f32..500.0,
            1 => Just(0.0f32),
            1 => prop_oneof![1e30f32..f32::MAX, -f32::MAX..-1e30f32],
            1 => any::<f32>(),
        ]
    }

    proptest! {
        #[test]
        fn fit_is_total_and_finite(
            points in prop::collection::vec(
                (arb_coord(), arb_coord()).prop_map(|(x, y)| Vec2::new(x, y)),
                0..80,
            ),
        ) {
            // Act
            let fit = fit(&points);
            let shape = recognize(&Shape::Stroke {
                points: points.clone(),
                widths: None,
                style: style(),
            });

            // Assert
            prop_assert!(fit_is_finite(fit), "{:?}", fit);
            if points.iter().any(|p| !p.is_finite()) {
                prop_assert_eq!(fit, None);
            }
            if let Some(shape) = shape {
                prop_assert!(shape.is_finite(), "{:?}", shape);
                prop_assert_eq!(shape.style(), style());
            }
        }
    }
}
