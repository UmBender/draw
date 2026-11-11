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
pub const LINE_MIN_STRAIGHTNESS: f32 = 0.8;
/// No sample of a line lies farther than this fraction of the chord from
/// it.
pub const LINE_MAX_DEVIATION: f32 = 0.05;
/// A loop is closed when its ends are at most this fraction of the path
/// length apart.
pub const CLOSED_MAX_GAP: f32 = 0.15;
/// Rotations of the measuring box tried over a quarter turn.
pub const ROTATION_STEPS: u32 = 90;
/// The short side of a closed shape's box is at least this fraction of its
/// long side.
pub const MIN_ASPECT: f32 = 0.1;
/// A closed shape winds around its box centre at least this fraction of a
/// full turn.
pub const MIN_TURN: f32 = 0.8;
/// A box corner counts when a sample lies within this fraction of the box
/// diagonal.
pub const CORNER_TOLERANCE: f32 = 0.08;
/// Mean distance of the samples to the outline, as a fraction of the box
/// diagonal.
pub const MEAN_RESIDUAL: f32 = 0.035;
/// Largest distance of a sample to the outline, as a fraction of the box
/// diagonal.
pub const MAX_RESIDUAL: f32 = 0.1;

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
        let (a, b) = (path[seg], path[seg + 1]);
        let d = a.distance(b);
        let t = if d > 0.0 {
            ((at - walked) / d).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push(a.lerp(b, t));
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

/// Whether the residuals of `samples` are small for a box of diagonal
/// `diagonal`.
fn hugs(residuals: impl Iterator<Item = f64>, count: usize, diagonal: f64) -> bool {
    let (mut sum, mut max) = (0.0, 0.0f64);
    for r in residuals {
        sum += r;
        max = max.max(r);
    }
    let mean = sum / count.max(1) as f64;
    mean <= f64::from(MEAN_RESIDUAL) * diagonal && max <= f64::from(MAX_RESIDUAL) * diagonal
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
        x: (lo.x + hi.x) / 2.0,
        y: (lo.y + hi.y) / 2.0,
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
    let reach = f64::from(CORNER_TOLERANCE) * diagonal;
    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .into_iter()
        .filter(|&(sx, sy)| {
            let corner = P {
                x: sx * hu,
                y: sy * hv,
            };
            local.iter().any(|p| p.distance(corner) <= reach)
        })
        .count();
    let n = local.len();
    match corners {
        4 if hugs(
            local
                .iter()
                .map(|p| (hu - p.x.abs()).min(hv - p.y.abs()).max(0.0)),
            n,
            diagonal,
        ) =>
        {
            Some(Closed::Rect)
        }
        0 if hugs(
            local.iter().map(|p| {
                let r = (p.x / hu).hypot(p.y / hv);
                if r > 0.0 {
                    p.x.hypot(p.y) * (1.0 - 1.0 / r).abs()
                } else {
                    hu.min(hv)
                }
            }),
            n,
            diagonal,
        ) =>
        {
            Some(Closed::Ellipse)
        }
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
    use std::f32::consts::{PI, TAU};

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
        let d = b - a;
        let len = d.length();
        let normal = Vec2::new(-d.y, d.x) / len;
        let n = 60;
        (0..=n)
            .map(|i| {
                let t = i as f32 / n as f32;
                let off = wave * len * (PI * t).sin() * (2.0 * PI * t + 0.3).cos()
                    + jitter * noise.next();
                a + d * t + normal * off
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
        let n = 90;
        (0..=n)
            .map(|i| {
                let a = start + TAU * turns * i as f32 / n as f32;
                let k = 1.0 + wobble * (3.0 * a + 0.7).sin();
                let p = c + Vec2::new(rx * k * a.cos(), ry * k * a.sin());
                let p = p + Vec2::new(noise.next(), noise.next()) * jitter;
                rotate(p, c, tilt)
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
        assert!(got.min.approx_eq(expected.min, 1e-3), "{got:?} vs {expected:?}");
        assert!(got.max.approx_eq(expected.max, 1e-3), "{got:?} vs {expected:?}");
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
            assert!(matches!(fit, Some(Fit::Line { .. })), "case {seed}: {fit:?}");
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
            (Vec2::new(100.0, 50.0), Vec2::new(700.0, 650.0), 2.3, 1.03, 8.0),
            (Vec2::new(0.0, 0.0), Vec2::new(300.0, 120.0), 1.0, 1.0, 2.5),
            (Vec2::new(-80.0, -400.0), Vec2::new(0.0, 0.0), 3.6, 0.98, 1.5),
        ];
        for (seed, &(min, max, start, turns, jitter)) in cases.iter().enumerate() {
            // Arrange
            let points = shaky_rect(min, max, start, turns, jitter, 0.0, seed as u64);

            // Act
            let fit = fit(&points);

            // Assert
            assert!(matches!(fit, Some(Fit::Rect { .. })), "case {seed}: {fit:?}");
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
        let line = stroke(shaky_line(
            Vec2::ZERO,
            Vec2::new(100.0, 40.0),
            0.01,
            0.5,
            5,
        ));

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
            assert!(matches!(fit, Some(Fit::Rect { .. })), "tilt {tilt}: {fit:?}");
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
        let u = shaky_rect(
            Vec2::ZERO,
            Vec2::new(200.0, 200.0),
            0.0,
            0.75,
            1.0,
            0.0,
            3,
        );
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
            ("NaN", vec![p, Vec2::new(f32::NAN, 0.0), Vec2::new(10.0, 4.0)]),
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
        assert!(matches!(fits[0], Some(Fit::Ellipse { .. })), "{:?}", fits[0]);
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
