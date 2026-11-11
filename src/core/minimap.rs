//! Minimap geometry: where the overview panel sits and how world points
//! map into it (ADR-T110-1).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{Document, tx_insert};
    use crate::core::geom::approx_eq;
    use crate::core::history::History;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use proptest::prelude::*;

    const EPS: f32 = 1e-3;
    const VIEWPORT: Vec2 = Vec2::new(800.0, 600.0);

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn boxed(x0: f32, y0: f32, x1: f32, y1: f32) -> Aabb {
        Aabb::from_corners(v(x0, y0), v(x1, y1))
    }

    fn assert_rect_eq(actual: Aabb, expected: Aabb) {
        assert!(
            actual.min.approx_eq(expected.min, EPS) && actual.max.approx_eq(expected.max, EPS),
            "expected {expected:?}, got {actual:?}"
        );
    }

    fn rect_is_finite(rect: Aabb) -> bool {
        rect.min.is_finite() && rect.max.is_finite()
    }

    fn inside(outer: Aabb, inner: Aabb) -> bool {
        outer.expand(EPS).contains(inner.min) && outer.expand(EPS).contains(inner.max)
    }

    fn line(a: Vec2, b: Vec2) -> Shape {
        Shape::Line {
            a,
            b,
            style: Style {
                color: ColorId::INK,
                width: 2.0,
            },
        }
    }

    fn document(shapes: Vec<Shape>) -> Document {
        let mut doc = Document::new();
        let mut history = History::new();
        let tx = tx_insert(&mut doc, shapes);
        assert!(history.commit(&mut doc, tx).is_ok());
        doc
    }

    /// The minimap of an 800 × 600 viewport.
    fn minimap(content: Option<Aabb>, view: Aabb) -> Minimap {
        let Some(map) = Minimap::new(VIEWPORT, content, view) else {
            panic!("an 800 x 600 viewport holds the panel");
        };
        map
    }

    // ---- AC-1: panel, world box, frame --------------------------------

    #[test]
    fn panel_rect_sits_in_the_bottom_right_corner() {
        // Act
        let Some(panel) = panel_rect(VIEWPORT) else {
            panic!("an 800 x 600 viewport holds the panel");
        };

        // Assert
        assert!(approx_eq(PANEL_SIZE.x, 200.0, EPS) && approx_eq(PANEL_SIZE.y, 150.0, EPS));
        assert_rect_eq(panel, boxed(588.0, 438.0, 788.0, 588.0));
        assert!(approx_eq(VIEWPORT.x - panel.max.x, MARGIN_PX, EPS));
        assert!(approx_eq(VIEWPORT.y - panel.max.y, MARGIN_PX, EPS));
    }

    #[test]
    fn panel_rect_small_viewport_is_none() {
        assert!(panel_rect(Vec2::ZERO).is_none());
        assert!(panel_rect(v(223.0, 600.0)).is_none());
        assert!(panel_rect(v(800.0, 173.0)).is_none());
        assert!(panel_rect(v(f32::NAN, 600.0)).is_none());
        assert!(panel_rect(v(800.0, f32::INFINITY)).is_none());
        assert!(panel_rect(v(224.0, 174.0)).is_some());
    }

    #[test]
    fn new_world_is_content_union_view() {
        // Arrange
        let content = boxed(-500.0, -100.0, 300.0, 900.0);
        let view = boxed(0.0, 0.0, 800.0, 600.0);

        // Act
        let map = minimap(Some(content), view);

        // Assert
        assert_rect_eq(map.world(), boxed(-500.0, -100.0, 800.0, 900.0));
        assert_eq!(map.content(), Some(content));
        assert_rect_eq(map.panel(), boxed(588.0, 438.0, 788.0, 588.0));
        assert_rect_eq(map.area(), boxed(594.0, 444.0, 782.0, 582.0));
    }

    #[test]
    fn content_rect_lies_inside_the_drawing_area() {
        // Arrange: content twice as wide as the area's aspect allows.
        let content = boxed(0.0, 0.0, 3760.0, 1380.0);
        let map = minimap(Some(content), boxed(100.0, 100.0, 900.0, 700.0));

        // Act
        let Some(rect) = map.content_rect() else {
            panic!("there is content");
        };

        // Assert: scale 188 / 3760 = 0.05, centred vertically.
        assert_rect_eq(rect, boxed(594.0, 478.5, 782.0, 547.5));
        assert!(inside(map.area(), rect));
        assert!(map.point(v(1880.0, 690.0)).approx_eq(v(688.0, 513.0), EPS));
    }

    #[test]
    fn frame_follows_the_view() {
        // Arrange
        let content = boxed(0.0, 0.0, 3760.0, 1380.0);
        let view = boxed(0.0, 0.0, 800.0, 600.0);
        let moved = view.translate(v(400.0, 200.0));

        // Act
        let before = minimap(Some(content), view).frame(view);
        let after = minimap(Some(content), moved).frame(moved);

        // Assert: 0.05 px per world unit.
        assert_rect_eq(before, boxed(594.0, 478.5, 634.0, 508.5));
        assert_rect_eq(after, boxed(614.0, 488.5, 654.0, 518.5));
    }

    #[test]
    fn frame_tiny_view_keeps_a_minimum_size() {
        // Arrange: a view 1000 times smaller than the content.
        let content = boxed(0.0, 0.0, 188_000.0, 138_000.0);
        let middle = boxed(94_000.0, 69_000.0, 94_100.0, 69_100.0);
        let corner = boxed(0.0, 0.0, 100.0, 100.0);
        let map = minimap(Some(content), middle);

        // Act
        let (mid, edge) = (map.frame(middle), map.frame(corner));

        // Assert
        for frame in [mid, edge] {
            assert!(approx_eq(frame.width(), FRAME_MIN_PX, EPS), "{frame:?}");
            assert!(approx_eq(frame.height(), FRAME_MIN_PX, EPS), "{frame:?}");
            assert!(inside(map.area(), frame), "{frame:?}");
        }
        assert!(mid.center().approx_eq(v(688.05, 513.05), 0.1), "{mid:?}");
        assert!(edge.min.approx_eq(map.area().min, EPS), "{edge:?}");
    }

    // ---- AC-2: pointer to world ---------------------------------------

    #[test]
    fn world_at_inverts_point() {
        // Arrange
        let map = minimap(
            Some(boxed(-300.0, -200.0, 3000.0, 2500.0)),
            boxed(0.0, 0.0, 800.0, 600.0),
        );

        for world in [v(0.0, 0.0), v(1234.5, -100.0), v(3000.0, 2500.0)] {
            // Act
            let back = map.world_at(map.point(world));

            // Assert
            assert!(back.approx_eq(world, 0.05), "{world:?} came back {back:?}");
        }
    }

    #[test]
    fn world_at_outside_the_drawing_area_clamps() {
        // Arrange
        let world = boxed(0.0, 0.0, 3760.0, 2760.0);
        let map = minimap(Some(world), boxed(0.0, 0.0, 800.0, 600.0));

        // Act
        let far = map.world_at(v(-1e30, 1e30));
        let padding = map.world_at(map.panel().max);
        let nan = map.world_at(v(f32::NAN, 500.0));

        // Assert
        assert!(far.approx_eq(v(0.0, 2760.0), EPS), "{far:?}");
        assert!(padding.approx_eq(v(3760.0, 2760.0), EPS), "{padding:?}");
        assert!(nan.is_finite() && world.contains(nan), "{nan:?}");
        assert!(map.contains(map.panel().max) && !map.contains(v(587.0, 500.0)));
        assert!(!map.contains(v(f32::NAN, 500.0)));
    }

    // ---- AC-3: placing the cached overview ----------------------------

    #[test]
    fn place_same_world_is_the_drawing_area() {
        // Arrange: the view lies inside the content.
        let content = boxed(0.0, 0.0, 3760.0, 2760.0);
        let map = minimap(Some(content), boxed(100.0, 100.0, 900.0, 700.0));
        let image = Fit::new(content, boxed(0.0, 0.0, 188.0, 138.0));

        // Act
        let Some(dest) = map.place(&image) else {
            panic!("the image has a place");
        };

        // Assert
        assert_rect_eq(dest, map.area());
    }

    #[test]
    fn place_shrinks_when_the_view_leaves_the_content() {
        // Arrange: the view doubles the world box to the right and below.
        let content = boxed(0.0, 0.0, 3760.0, 2760.0);
        let view = boxed(6720.0, 4920.0, 7520.0, 5520.0);
        let map = minimap(Some(content), view);
        let image = Fit::new(content, boxed(0.0, 0.0, 188.0, 138.0));

        // Act
        let Some(dest) = map.place(&image) else {
            panic!("the image has a place");
        };

        // Assert: half the size, in the top-left quarter.
        assert_rect_eq(dest, boxed(594.0, 444.0, 688.0, 513.0));
        let Some(rect) = map.content_rect() else {
            panic!("there is content");
        };
        assert_rect_eq(rect, dest);
    }

    #[test]
    fn place_degenerate_image_is_none() {
        // Arrange: one point of content, seen from close up.
        let content = boxed(10.0, 10.0, 10.0, 10.0);
        let map = minimap(Some(content), boxed(0.0, 0.0, 40.0, 30.0));
        let image = Fit::new(content, boxed(0.0, 0.0, 188.0, 138.0));

        // Act / Assert: the image would be magnified.
        assert!(map.fit().scale() > image.scale());
        assert!(map.place(&image).is_none());
    }

    // ---- AC-4: empty and extreme documents ----------------------------

    #[test]
    fn content_bounds_empty_document_is_none() {
        assert!(content_bounds(&Document::new()).is_none());
    }

    #[test]
    fn content_bounds_unions_every_shape() {
        // Arrange
        let doc = document(vec![
            line(v(0.0, 0.0), v(100.0, 50.0)),
            line(v(-1e30, 5.0), v(-1e30, 6.0)),
        ]);

        // Act
        let Some(bounds) = content_bounds(&doc) else {
            panic!("the document has shapes");
        };

        // Assert: half the 2-unit width on every side.
        assert!(bounds.max.approx_eq(v(101.0, 51.0), EPS), "{bounds:?}");
        assert!(bounds.min.x <= -1e30 && approx_eq(bounds.min.y, -1.0, EPS));
    }

    #[test]
    fn new_empty_document_shows_the_view_only() {
        // Arrange
        let view = boxed(-400.0, -300.0, 400.0, 300.0);

        // Act
        let map = minimap(None, view);

        // Assert
        assert_rect_eq(map.world(), view);
        assert!(map.content().is_none() && map.content_rect().is_none());
        // 800 x 600 into 188 x 138: the height limits, 0.23 px per unit.
        assert_rect_eq(map.frame(view), boxed(596.0, 444.0, 780.0, 582.0));
    }

    #[test]
    fn new_non_finite_input_is_rejected_or_dropped() {
        let view = boxed(0.0, 0.0, 800.0, 600.0);
        let bad = Aabb {
            min: v(f32::NAN, 0.0),
            max: v(1.0, 1.0),
        };

        assert!(Minimap::new(VIEWPORT, None, bad).is_none());
        assert!(Minimap::with_world(VIEWPORT, None, bad).is_none());
        assert!(Minimap::new(v(f32::NAN, 600.0), None, view).is_none());
        let map = minimap(Some(bad), view);
        assert!(map.content().is_none());
        assert_rect_eq(map.world(), view);
    }

    #[test]
    fn fit_zero_extent_world_is_finite() {
        // Arrange
        let rect = boxed(0.0, 0.0, 188.0, 138.0);
        let point = boxed(5.0, 5.0, 5.0, 5.0);
        let column = boxed(5.0, 0.0, 5.0, 276.0);

        // Act
        let (dot, tall) = (Fit::new(point, rect), Fit::new(column, rect));

        // Assert: a point maps to the centre, a column fits its height.
        assert!(dot.scale().is_finite() && dot.scale() > 0.0);
        assert!(dot.point(v(5.0, 5.0)).approx_eq(v(94.0, 69.0), EPS));
        assert!(dot.world_at(v(0.0, 0.0)).approx_eq(v(5.0, 5.0), EPS));
        assert!(approx_eq(tall.scale() as f32, 0.5, EPS));
        assert!(tall.point(v(5.0, 276.0)).approx_eq(v(94.0, 138.0), EPS));
        assert!(tall.world_at(v(188.0, 0.0)).approx_eq(v(5.0, 0.0), EPS));
    }

    #[test]
    fn fit_huge_world_is_finite() {
        // Arrange: the extent overflows `f32`.
        let world = boxed(-f32::MAX, -3e38, f32::MAX, 3e38);
        let rect = boxed(0.0, 0.0, 188.0, 138.0);

        // Act
        let fit = Fit::new(world, rect);

        // Assert
        assert!(fit.scale().is_finite() && fit.scale() > 0.0);
        // The height limits: the box is centred along x.
        let left = (94.0 - f64::from(f32::MAX) * fit.scale()) as f32;
        assert!(fit.point(world.min).approx_eq(v(left, 0.0), 0.01));
        assert!(left > 1.0);
        assert!(fit.point(v(0.0, 0.0)).approx_eq(v(94.0, 69.0), EPS));
        assert!(rect_is_finite(fit.rect_of(world)));
        assert!(fit.world_at(v(0.0, 0.0)).is_finite());
        assert!(fit.world_at(v(188.0, 138.0)).is_finite());
        assert!(fit.point(v(f32::NAN, 1.0)).is_finite());
    }

    fn arb_coord() -> impl Strategy<Value = f32> {
        prop_oneof![
            6 => -5_000.0f32..5_000.0,
            1 => Just(0.0f32),
            2 => prop_oneof![1e30f32..f32::MAX, -f32::MAX..-1e30f32],
            1 => prop_oneof![-1e-30f32..1e-30],
            1 => any::<f32>(),
        ]
    }

    fn arb_box() -> impl Strategy<Value = Aabb> {
        (arb_coord(), arb_coord(), arb_coord(), arb_coord())
            .prop_map(|(a, b, c, d)| Aabb::from_corners(Vec2::new(a, b), Vec2::new(c, d)))
    }

    proptest! {
        #[test]
        fn minimap_coordinates_stay_finite(
            content in prop::option::of(arb_box()),
            view in arb_box(),
            frozen in prop::option::of(arb_box()),
            pointer in (arb_coord(), arb_coord()),
            width in 0.0f32..3_000.0,
            height in 0.0f32..2_000.0,
        ) {
            let viewport = Vec2::new(width, height);
            let map = match frozen {
                Some(world) => Minimap::with_world(viewport, content, world),
                None => Minimap::new(viewport, content, view),
            };
            if let Some(map) = map {
                let (panel, area, world) = (map.panel(), map.area(), map.world());
                prop_assert!(rect_is_finite(panel) && rect_is_finite(area));
                prop_assert!(rect_is_finite(world));
                prop_assert!(map.fit().scale().is_finite() && map.fit().scale() >= 0.0);

                let frame = map.frame(view);
                prop_assert!(rect_is_finite(frame), "frame {:?}", frame);
                prop_assert!(inside(area, frame), "frame {:?} outside {:?}", frame, area);

                if let Some(rect) = map.content_rect() {
                    prop_assert!(rect_is_finite(rect), "content {:?}", rect);
                    prop_assert!(inside(area, rect), "content {:?} outside {:?}", rect, area);
                }

                let at = map.world_at(Vec2::new(pointer.0, pointer.1));
                prop_assert!(at.is_finite(), "pointer maps to {:?}", at);
                prop_assert!(world.contains(at), "{:?} outside {:?}", at, world);
                let back = map.point(at);
                prop_assert!(back.is_finite() && area.expand(EPS).contains(back));

                if let Some(content) = map.content() {
                    let image = Fit::new(content, Aabb::from_corners(Vec2::ZERO, PANEL_SIZE));
                    if let Some(dest) = map.place(&image) {
                        prop_assert!(rect_is_finite(dest), "image at {:?}", dest);
                    }
                }
            }
        }
    }
}
