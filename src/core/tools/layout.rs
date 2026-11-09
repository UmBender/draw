#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;
    use crate::core::palette::ColorId;
    use crate::core::shape::TextSlot;
    use crate::core::text_layout;
    use crate::core::tools::text::PASTE_MAX_BYTES;
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
            assert!(approx_eq(left, origin.x, 1e-2), "{text}: {left}");
            assert!(approx_eq(top, origin.y, 1e-2), "{text}: {top}");
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
            [
                (2, 1, true, None),
                (1, 3, true, None),
                (3, 4, false, None)
            ]
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
        let (root, first, last) = (
            centre(&shapes, 1),
            centre(&shapes, 2),
            centre(&shapes, 4),
        );
        assert!(approx_eq(root.x, (first.x + last.x) * 0.5, 1e-3));
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
        assert!(parsed.edges().is_empty());
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
}
