//! Coordinates that cannot be mixed up: [`Point`], [`Len`] and [`Rect`] carry a
//! unit marker ([`World`], [`Screen`] or [`Px`]) over the plain [`Vec2`],
//! [`Aabb`] and `f32` of [`crate::geom`].
//!
//! There is no public cast from one unit to another: the only conversions are
//! the methods of `Camera` in `draw-core` (ADR-T133-1). Getting in or out of
//! the plain types is explicit: [`Point::from_vec2`] and [`Point::vec2`].

use crate::geom::{Aabb, Vec2};
use std::marker::PhantomData;
use std::ops::{Add, Mul, Sub};

/// Unit marker: the infinite canvas, in world units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum World {}

/// Unit marker: the window, in logical screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {}

/// Unit marker: device pixels (after the display scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Px {}

/// A point in the unit `U`.
pub struct Point<U> {
    v: Vec2,
    unit: PhantomData<fn() -> U>,
}

/// A length in the unit `U`.
pub struct Len<U> {
    v: f32,
    unit: PhantomData<fn() -> U>,
}

/// An axis-aligned box in the unit `U`.
pub struct Rect<U> {
    b: Aabb,
    unit: PhantomData<fn() -> U>,
}

// Manual impls: a derive would require `U: Clone + PartialEq + Debug`.
macro_rules! plain_impls {
    ($ty:ident, $field:ident, $name:literal) => {
        impl<U> Clone for $ty<U> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<U> Copy for $ty<U> {}
        impl<U> PartialEq for $ty<U> {
            fn eq(&self, other: &Self) -> bool {
                self.$field == other.$field
            }
        }
        impl<U> std::fmt::Debug for $ty<U> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple($name).field(&self.$field).finish()
            }
        }
    };
}

plain_impls!(Point, v, "Point");
plain_impls!(Len, v, "Len");
plain_impls!(Rect, b, "Rect");

impl<U> Point<U> {
    /// Creates a point from its components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self::from_vec2(Vec2::new(x, y))
    }

    /// Tags a plain vector with this unit.
    #[must_use]
    pub const fn from_vec2(v: Vec2) -> Self {
        Self {
            v,
            unit: PhantomData,
        }
    }

    /// The plain vector, without the unit.
    #[must_use]
    pub const fn vec2(self) -> Vec2 {
        self.v
    }

    /// Horizontal component.
    #[must_use]
    pub const fn x(self) -> f32 {
        self.v.x
    }

    /// Vertical component.
    #[must_use]
    pub const fn y(self) -> f32 {
        self.v.y
    }

    /// Distance to a point of the same unit.
    #[must_use]
    pub fn distance(self, other: Self) -> Len<U> {
        Len::new(self.v.distance(other.v))
    }
}

impl<U> Len<U> {
    /// Creates a length.
    #[must_use]
    pub const fn new(v: f32) -> Self {
        Self {
            v,
            unit: PhantomData,
        }
    }

    /// The plain number, without the unit.
    #[must_use]
    pub const fn get(self) -> f32 {
        self.v
    }
}

impl<U> Add for Len<U> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.v + rhs.v)
    }
}

impl<U> Sub for Len<U> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.v - rhs.v)
    }
}

impl<U> Mul<f32> for Len<U> {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.v * rhs)
    }
}

impl<U> Rect<U> {
    /// Tags a plain box with this unit.
    #[must_use]
    pub const fn from_aabb(b: Aabb) -> Self {
        Self {
            b,
            unit: PhantomData,
        }
    }

    /// The box of two opposite corners, in any order.
    #[must_use]
    pub fn from_corners(a: Point<U>, b: Point<U>) -> Self {
        Self::from_aabb(Aabb::from_corners(a.v, b.v))
    }

    /// The plain box, without the unit.
    #[must_use]
    pub const fn aabb(self) -> Aabb {
        self.b
    }

    /// The corner with the smallest coordinates.
    #[must_use]
    pub const fn min(self) -> Point<U> {
        Point::from_vec2(self.b.min)
    }

    /// The corner with the largest coordinates.
    #[must_use]
    pub const fn max(self) -> Point<U> {
        Point::from_vec2(self.b.max)
    }

    /// Horizontal extent.
    #[must_use]
    pub fn width(self) -> Len<U> {
        Len::new(self.b.width())
    }

    /// Vertical extent.
    #[must_use]
    pub fn height(self) -> Len<U> {
        Len::new(self.b.height())
    }

    /// Whether `p` lies inside or on the border.
    #[must_use]
    pub fn contains(self, p: Point<U>) -> bool {
        self.b.contains(p.v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::approx_eq;

    #[test]
    fn point_new_keeps_components() {
        let p = Point::<World>::new(1.0, 2.0);

        assert!(approx_eq(p.x(), 1.0, 1e-6));
        assert!(approx_eq(p.y(), 2.0, 1e-6));
        assert!(p.vec2().approx_eq(Vec2::new(1.0, 2.0), 1e-6));
    }

    #[test]
    fn point_distance_keeps_unit() {
        let a = Point::<Screen>::new(0.0, 0.0);
        let b = Point::<Screen>::new(3.0, 4.0);

        let d: Len<Screen> = a.distance(b);

        assert!(approx_eq(d.get(), 5.0, 1e-6));
    }

    #[test]
    fn len_arithmetic_keeps_unit() {
        let a = Len::<World>::new(2.0);
        let b = Len::<World>::new(3.0);

        assert!(approx_eq((a + b).get(), 5.0, 1e-6));
        assert!(approx_eq((b - a).get(), 1.0, 1e-6));
        assert!(approx_eq((a * 4.0).get(), 8.0, 1e-6));
    }

    #[test]
    fn rect_from_corners_orders_and_measures() {
        let r = Rect::<Px>::from_corners(Point::new(4.0, 1.0), Point::new(1.0, 3.0));

        assert!(approx_eq(r.width().get(), 3.0, 1e-6));
        assert!(approx_eq(r.height().get(), 2.0, 1e-6));
        assert!(r.contains(Point::new(2.0, 2.0)));
        assert!(!r.contains(Point::new(5.0, 2.0)));
        assert!(r.min().vec2().approx_eq(Vec2::new(1.0, 1.0), 1e-6));
        assert!(r.max().vec2().approx_eq(Vec2::new(4.0, 3.0), 1e-6));
    }

    #[test]
    fn rect_aabb_round_trip() {
        let aabb = Aabb::from_corners(Vec2::new(0.0, 0.0), Vec2::new(2.0, 5.0));

        let back = Rect::<World>::from_aabb(aabb).aabb();

        assert!(back.min.approx_eq(aabb.min, 1e-6));
        assert!(back.max.approx_eq(aabb.max, 1e-6));
    }
}
