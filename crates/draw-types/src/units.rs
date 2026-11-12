#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::{approx_eq, Aabb, Vec2};

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
