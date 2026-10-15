#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::text_layout;
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    /// The nodes of formula `src` (written without its `$`s).
    fn math(src: &str) -> Vec<Node> {
        match parse(&format!("${src}$")).as_slice() {
            [Segment::Math(nodes)] => nodes.clone(),
            other => panic!("one formula expected for {src:?}: {other:?}"),
        }
    }

    fn tok(node: Node) -> Arg {
        Arg {
            lead: String::new(),
            braced: false,
            nodes: vec![node],
        }
    }

    fn braced(nodes: Vec<Node>) -> Arg {
        Arg {
            lead: String::new(),
            braced: true,
            nodes,
        }
    }

    fn sym(name: &str) -> Node {
        Node::Symbol(name.to_owned())
    }

    fn script(kind: ScriptKind, arg: Arg) -> Script {
        Script { kind, arg }
    }

    /// `(char, position, scale, warn)` of every glyph, in order.
    fn glyphs(ink: &[Ink]) -> Vec<(char, Vec2, f32, bool)> {
        ink.iter()
            .filter_map(|ink| match *ink {
                Ink::Glyph {
                    ch,
                    pos,
                    scale,
                    warn,
                } => Some((ch, pos, scale, warn)),
                Ink::Rule { .. } => None,
            })
            .collect()
    }

    /// `(a, b, width)` of every rule, in order.
    fn rules(ink: &[Ink]) -> Vec<(Vec2, Vec2, f32)> {
        ink.iter()
            .filter_map(|ink| match *ink {
                Ink::Rule { a, b, width } => Some((a, b, width)),
                Ink::Glyph { .. } => None,
            })
            .collect()
    }

    /// The glyph `ch` of `ink`; panics if it is missing.
    fn glyph(ink: &[Ink], ch: char) -> (Vec2, f32) {
        match glyphs(ink).into_iter().find(|g| g.0 == ch) {
            Some((_, pos, scale, _)) => (pos, scale),
            None => panic!("no glyph {ch:?} in {ink:?}"),
        }
    }

    fn boxed(src: &str) -> MathBox {
        layout_math(&math(src), 1.0)
    }

    // ---- AC-1 parse --------------------------------------------------------

    #[test]
    fn tex_parse_plain_text_is_one_text_segment() {
        assert_eq!(parse("a b\nc"), vec![Segment::Text("a b\nc".to_owned())]);
        assert_eq!(parse(""), Vec::<Segment>::new());
    }

    #[test]
    fn tex_parse_dollar_pair_is_a_formula() {
        // Act
        let segments = parse("x $a$ y");

        // Assert
        assert_eq!(
            segments,
            vec![
                Segment::Text("x ".to_owned()),
                Segment::Math(vec![Node::Char('a')]),
                Segment::Text(" y".to_owned()),
            ]
        );
    }

    #[test]
    fn tex_parse_escaped_dollar_is_a_dollar_segment() {
        assert_eq!(
            parse("\\$5"),
            vec![Segment::Dollar, Segment::Text("5".to_owned())]
        );
        // Inside a formula `\$` does not close it.
        assert_eq!(math("\\$"), vec![sym("$")]);
    }

    #[test]
    fn tex_parse_unclosed_or_empty_dollar_is_text() {
        assert_eq!(
            parse("costs $5"),
            vec![Segment::Text("costs $5".to_owned())]
        );
        assert_eq!(parse("$$"), vec![Segment::Text("$$".to_owned())]);
        assert_eq!(
            parse("$$x$"),
            vec![
                Segment::Text("$".to_owned()),
                Segment::Math(vec![Node::Char('x')]),
            ]
        );
    }

    #[test]
    fn tex_parse_formula_does_not_cross_a_line_break() {
        assert_eq!(parse("$a\nb$"), vec![Segment::Text("$a\nb$".to_owned())]);
        assert_eq!(
            parse("$a\n$b$"),
            vec![
                Segment::Text("$a\n".to_owned()),
                Segment::Math(vec![Node::Char('b')]),
            ]
        );
    }

    #[test]
    fn tex_parse_scripts_bind_to_the_previous_atom() {
        // Act / Assert: both scripts on one base, in either order.
        assert_eq!(
            math("x_i^2"),
            vec![Node::Scripts {
                base: Some(Box::new(Node::Char('x'))),
                first: script(ScriptKind::Sub, tok(Node::Char('i'))),
                second: Some(script(ScriptKind::Sup, tok(Node::Char('2')))),
            }]
        );
        // Whitespace before `^` leaves it without a base.
        assert_eq!(
            math("x ^2"),
            vec![
                Node::Char('x'),
                Node::Space(" ".to_owned()),
                Node::Scripts {
                    base: None,
                    first: script(ScriptKind::Sup, tok(Node::Char('2'))),
                    second: None,
                },
            ]
        );
    }

    #[test]
    fn tex_parse_script_takes_one_token_or_a_group() {
        let sup = |arg| Node::Scripts {
            base: Some(Box::new(Node::Char('x'))),
            first: script(ScriptKind::Sup, arg),
            second: None,
        };

        assert_eq!(
            math("x^ab"),
            vec![sup(tok(Node::Char('a'))), Node::Char('b')]
        );
        assert_eq!(
            math("x^{ab}"),
            vec![sup(braced(vec![Node::Char('a'), Node::Char('b')]))]
        );
        assert_eq!(math("x^\\alpha"), vec![sup(tok(sym("alpha")))]);
    }

    #[test]
    fn tex_parse_commands_take_braced_or_single_token_arguments() {
        assert_eq!(
            math("\\frac{a}{b}"),
            vec![Node::Command {
                name: "frac".to_owned(),
                args: vec![braced(vec![Node::Char('a')]), braced(vec![Node::Char('b')]),],
            }]
        );
        assert_eq!(
            math("\\frac12"),
            vec![Node::Command {
                name: "frac".to_owned(),
                args: vec![tok(Node::Char('1')), tok(Node::Char('2'))],
            }]
        );
        assert_eq!(
            math("\\sqrt {x}"),
            vec![Node::Command {
                name: "sqrt".to_owned(),
                args: vec![Arg {
                    lead: " ".to_owned(),
                    braced: true,
                    nodes: vec![Node::Char('x')],
                }],
            }]
        );
    }

    #[test]
    fn tex_parse_symbols_come_from_the_table() {
        // Act
        let nodes = math("\\alpha\\le\\sum");

        // Assert
        assert_eq!(nodes, vec![sym("alpha"), sym("le"), sym("sum")]);
        let def = |name| symbol(name).map(|s| (s.text, s.class));
        assert_eq!(def("alpha"), Some(("α", Class::Ord)));
        assert_eq!(def("le"), Some(("≤", Class::Rel)));
        assert_eq!(def("times"), Some(("×", Class::Bin)));
        assert_eq!(def("log"), Some(("log", Class::Op)));
        assert_eq!(def("sum"), Some(("∑", Class::BigOp)));
        assert_eq!(def("min"), Some(("min", Class::Limits)));
        assert_eq!(def("lfloor"), Some(("⌊", Class::Open)));
        assert_eq!(def("foo"), None);
        let mut names: Vec<&str> = SYMBOLS.iter().map(|s| s.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), SYMBOLS.len(), "symbol names are unique");
    }

    #[test]
    fn tex_parse_text_command_keeps_its_raw_body() {
        assert_eq!(
            math("\\text{a {b} c}"),
            vec![Node::Text {
                lead: String::new(),
                body: "a {b} c".to_owned(),
            }]
        );
    }

    #[test]
    fn tex_parse_unknown_command_is_literal() {
        assert_eq!(
            math("\\foo x"),
            vec![
                Node::Literal("\\foo".to_owned()),
                Node::Space(" ".to_owned()),
                Node::Char('x'),
            ]
        );
        assert_eq!(
            math("a\\"),
            vec![Node::Char('a'), Node::Literal("\\".to_owned())]
        );
    }

    #[test]
    fn tex_parse_unbalanced_braces_are_literal() {
        assert_eq!(
            math("{a"),
            vec![Node::Literal("{".to_owned()), Node::Char('a')]
        );
        assert_eq!(
            math("a}"),
            vec![Node::Char('a'), Node::Literal("}".to_owned())]
        );
        assert_eq!(math("{a}"), vec![Node::Group(vec![Node::Char('a')])]);
    }

    #[test]
    fn tex_parse_deep_nesting_is_literal_past_the_limit() {
        // Arrange: far deeper than any formula, deeper than the stack allows
        // for naive recursion.
        let src = format!("${}x{}$", "{".repeat(100_000), "}".repeat(100_000));

        // Act
        let segments = parse(&src);

        // Assert
        assert_eq!(print(&segments), src);
        let l = layout(&segments);
        assert!(l.width.is_finite() && l.height.is_finite());
    }

    #[test]
    fn tex_parse_missing_argument_is_literal() {
        assert_eq!(
            math("x^"),
            vec![Node::Char('x'), Node::Literal("^".to_owned())]
        );
        assert_eq!(
            math("\\frac{a}"),
            vec![
                Node::Literal("\\frac".to_owned()),
                Node::Group(vec![Node::Char('a')]),
            ]
        );
        assert_eq!(math("\\sqrt"), vec![Node::Literal("\\sqrt".to_owned())]);
        assert_eq!(
            math("\\text x"),
            vec![
                Node::Literal("\\text".to_owned()),
                Node::Space(" ".to_owned()),
                Node::Char('x'),
            ]
        );
    }

    #[test]
    fn tex_parse_double_superscript_is_literal() {
        assert_eq!(
            math("x^1^2"),
            vec![
                Node::Scripts {
                    base: Some(Box::new(Node::Char('x'))),
                    first: script(ScriptKind::Sup, tok(Node::Char('1'))),
                    second: None,
                },
                Node::Literal("^".to_owned()),
                Node::Char('2'),
            ]
        );
        assert_eq!(
            math("x_1^2_3"),
            vec![
                Node::Scripts {
                    base: Some(Box::new(Node::Char('x'))),
                    first: script(ScriptKind::Sub, tok(Node::Char('1'))),
                    second: Some(script(ScriptKind::Sup, tok(Node::Char('2')))),
                },
                Node::Literal("_".to_owned()),
                Node::Char('3'),
            ]
        );
    }

    /// Text built from formula-ish tokens and arbitrary characters.
    fn arb_source() -> impl Strategy<Value = String> {
        let token = prop_oneof![
            4 => prop::sample::select(vec![
                "$", "$", "\\$", "{", "}", "^", "_", "\\", " ", "\n", "x", "1", "-", "=",
                "\\frac", "\\sqrt", "\\binom", "\\text", "\\pmod", "\\alpha", "\\sum",
                "\\min", "\\log", "\\le", "\\{", "\\}", "\\foo", "(", ")", "|",
            ])
            .prop_map(str::to_owned),
            1 => any::<char>().prop_map(String::from),
        ];
        prop::collection::vec(token, 0..60).prop_map(|tokens| tokens.concat())
    }

    proptest! {
        #[test]
        fn tex_parse_never_panics_and_prints_back_the_source(src in arb_source()) {
            // Act
            let segments = parse(&src);

            // Assert
            prop_assert_eq!(print(&segments), src);
        }

        #[test]
        fn tex_layout_bounds_contain_every_ink(src in arb_source()) {
            // Act
            let l = layout(&parse(&src));

            // Assert
            let b = l.bounds().expand(1e-3);
            for ink in &l.ink {
                let ib = ink.bounds();
                prop_assert!(b.contains(ib.min) && b.contains(ib.max), "{ink:?} {b:?}");
            }
        }

        #[test]
        fn tex_layout_is_finite(src in arb_source()) {
            // Act
            let l = layout(&parse(&src));

            // Assert
            prop_assert!(l.width.is_finite() && l.width >= 0.0);
            prop_assert!(l.height.is_finite() && l.height > 0.0);
            for ink in &l.ink {
                let ib = ink.bounds();
                prop_assert!(ib.min.is_finite() && ib.max.is_finite(), "{ink:?}");
            }
        }
    }

    // ---- AC-2 layout -------------------------------------------------------

    #[test]
    fn tex_layout_char_is_one_cell() {
        // Act
        let b = boxed("x");

        // Assert
        assert!(approx_eq(b.width, 0.6, EPS));
        assert!(approx_eq(b.ascent, GLYPH_ASCENT, EPS));
        assert!(approx_eq(b.descent, GLYPH_DESCENT, EPS));
        assert_eq!(glyphs(&b.ink), vec![('x', v(0.0, 0.0), 1.0, false)]);
    }

    #[test]
    fn tex_layout_superscript_is_raised_and_scaled() {
        // Act
        let b = boxed("x^2");

        // Assert
        let (pos, scale) = glyph(&b.ink, '2');
        assert!(pos.approx_eq(v(0.6, -0.4), EPS), "{pos:?}");
        assert!(approx_eq(scale, SCRIPT_SCALE, EPS));
        assert!(approx_eq(b.width, 0.6 + 0.6 * SCRIPT_SCALE, EPS));
        // A script of a script stops at the smallest scale.
        let (_, scale) = glyph(&boxed("x^{y^z}").ink, 'z');
        assert!(approx_eq(scale, MIN_SCALE, EPS));
    }

    #[test]
    fn tex_layout_subscript_is_lowered_and_scaled() {
        // Act
        let b = boxed("x_2");

        // Assert
        let (pos, scale) = glyph(&b.ink, '2');
        assert!(pos.approx_eq(v(0.6, 0.2), EPS), "{pos:?}");
        assert!(approx_eq(scale, SCRIPT_SCALE, EPS));
    }

    #[test]
    fn tex_layout_sub_and_sup_share_the_base() {
        for src in ["x_i^2", "x^2_i"] {
            // Act
            let b = boxed(src);

            // Assert: both start at the base's right edge, apart by 0.1 em.
            let (sup, s) = glyph(&b.ink, '2');
            let (sub, _) = glyph(&b.ink, 'i');
            assert!(
                approx_eq(sup.x, 0.6, EPS) && approx_eq(sub.x, 0.6, EPS),
                "{src}"
            );
            let sup_bottom = sup.y + GLYPH_DESCENT * s;
            let sub_top = sub.y - GLYPH_ASCENT * s;
            assert!(sub_top - sup_bottom >= 0.1 - EPS, "{src}: {sup:?} {sub:?}");
            assert!(approx_eq(b.width, 0.6 + 0.6 * SCRIPT_SCALE, EPS));
        }
    }

    #[test]
    fn tex_layout_frac_centres_parts_around_a_bar_on_the_axis() {
        // Act
        let b = boxed("\\frac{a}{b}");

        // Assert: one bar on the axis, numerator above, denominator below.
        let bars = rules(&b.ink);
        assert_eq!(bars.len(), 1);
        let (p, q, width) = bars[0];
        assert!(approx_eq(p.y, -AXIS, EPS) && approx_eq(q.y, -AXIS, EPS));
        assert!(approx_eq(width, RULE, EPS));
        assert!(p.x >= 0.0 && q.x <= b.width && q.x - p.x >= 0.6 * SCRIPT_SCALE);
        let (num, ns) = glyph(&b.ink, 'a');
        let (den, ds) = glyph(&b.ink, 'b');
        assert!(approx_eq(ns, SCRIPT_SCALE, EPS) && approx_eq(ds, SCRIPT_SCALE, EPS));
        assert!(num.y + GLYPH_DESCENT * ns < -AXIS - RULE / 2.0);
        assert!(den.y - GLYPH_ASCENT * ds > -AXIS + RULE / 2.0);
        let middle = b.width / 2.0;
        assert!(approx_eq(num.x + 0.3 * ns, middle, EPS));
        assert!(approx_eq(den.x + 0.3 * ds, middle, EPS));
        assert!(b.ascent > GLYPH_ASCENT && b.descent > GLYPH_DESCENT);
    }

    #[test]
    fn tex_layout_sqrt_draws_a_root_sign_over_the_content() {
        // Act
        let b = boxed("\\sqrt{x}");

        // Assert: four strokes, the last a bar over the whole content.
        let strokes = rules(&b.ink);
        assert_eq!(strokes.len(), 4);
        let (x, scale) = glyph(&b.ink, 'x');
        assert!(approx_eq(scale, 1.0, EPS) && approx_eq(x.y, 0.0, EPS));
        let (p, q, _) = strokes[3];
        assert!(approx_eq(p.y, q.y, EPS), "overbar is horizontal");
        assert!(p.y < -GLYPH_ASCENT, "overbar above the content");
        assert!(p.x <= x.x && q.x >= x.x + 0.6, "overbar covers the content");
        assert!(b.ascent >= -p.y && b.width >= q.x);
    }

    #[test]
    fn tex_layout_binom_has_parens_and_no_bar() {
        // Act
        let b = boxed("\\binom{n}{k}");

        // Assert
        assert!(rules(&b.ink).is_empty());
        let g = glyphs(&b.ink);
        let (first, last) = (g[0], g[g.len() - 1]);
        assert_eq!((first.0, last.0), ('(', ')'));
        assert!(first.2 > 1.0 && approx_eq(first.2, last.2, EPS), "{g:?}");
        assert!(approx_eq(first.1.x, 0.0, EPS));
        assert!(approx_eq(last.1.x + 0.6 * last.2, b.width, EPS));
        let (n, _) = glyph(&b.ink, 'n');
        let (k, _) = glyph(&b.ink, 'k');
        assert!(n.y < k.y && approx_eq(n.x, k.x, EPS));
    }

    #[test]
    fn tex_layout_sum_limits_are_centred_above_and_below() {
        // Act
        let b = boxed("\\sum_{k=1}^{n}");

        // Assert
        let (sum, big) = glyph(&b.ink, '∑');
        assert!(approx_eq(big, BIG_OP_SCALE, EPS));
        let centre = sum.x + 0.3 * big;
        let (n, ns) = glyph(&b.ink, 'n');
        let (k, ks) = glyph(&b.ink, 'k');
        assert!(approx_eq(n.x + 0.3 * ns, centre, EPS), "{n:?} {centre}");
        // "k=1" is three cells at script scale, without relation space.
        assert!(approx_eq(k.x + 0.9 * ks, centre, EPS), "{k:?} {centre}");
        assert!(n.y + GLYPH_DESCENT * ns <= sum.y - TALL_ASCENT * big + EPS);
        assert!(k.y - GLYPH_ASCENT * ks >= sum.y + TALL_DESCENT * big - EPS);
        // Words with limits too.
        let m = boxed("\\min_i");
        let (i, is) = glyph(&m.ink, 'i');
        assert!(approx_eq(i.x + 0.3 * is, 0.9, EPS), "{i:?}");
        assert!(i.y > GLYPH_DESCENT);
    }

    #[test]
    fn tex_layout_relations_and_binary_operators_get_space() {
        let width = |src| boxed(src).width;

        assert!(approx_eq(width("a=b"), 1.8 + 2.0 * REL_SPACE, EPS));
        assert!(approx_eq(width("a+b"), 1.8 + 2.0 * BIN_SPACE, EPS));
        // A leading minus is unary; spaces in the source are ignored.
        assert!(approx_eq(width("-a"), 1.2, EPS));
        assert!(approx_eq(width("a + b"), 1.8 + 2.0 * BIN_SPACE, EPS));
        // A function before an ordinary atom, not before a bracket.
        assert!(approx_eq(width("\\log n"), 2.4 + OP_SPACE, EPS));
        assert!(approx_eq(width("\\log(n)"), 3.6, EPS));
        // No relation space inside scripts.
        assert!(approx_eq(width("a_{i=1}"), 0.6 + 1.8 * SCRIPT_SCALE, EPS));
    }

    #[test]
    fn tex_layout_minus_is_the_math_minus() {
        let chars: Vec<char> = glyphs(&boxed("a-b").ink).iter().map(|g| g.0).collect();

        assert_eq!(chars, vec!['a', MINUS, 'b']);
    }

    #[test]
    fn tex_layout_literal_is_warned() {
        // Act
        let unknown = glyphs(&boxed("\\foo").ink);
        let brace = glyphs(&boxed("{a").ink);

        // Assert
        let chars: String = unknown.iter().map(|g| g.0).collect();
        assert_eq!(chars, "\\foo");
        assert!(unknown.iter().all(|g| g.3));
        assert_eq!(
            brace.iter().map(|g| (g.0, g.3)).collect::<Vec<_>>(),
            vec![('{', true), ('a', false)]
        );
    }

    #[test]
    fn tex_layout_plain_text_matches_the_cell_layout() {
        // Act
        let l = layout(&parse("a b\ncd"));

        // Assert
        let cells: Vec<(char, Vec2)> = text_layout::glyphs(Vec2::ZERO, "a b\ncd", 1.0)
            .map(|g| (g.ch, g.pos))
            .collect();
        let got: Vec<(char, Vec2)> = glyphs(&l.ink).iter().map(|g| (g.0, g.1)).collect();
        assert_eq!(got.len(), cells.len());
        for (g, c) in got.iter().zip(&cells) {
            assert!(g.0 == c.0 && g.1.approx_eq(c.1, EPS), "{g:?} {c:?}");
        }
        assert!(approx_eq(l.width, 1.8, EPS) && approx_eq(l.height, 2.5, EPS));
        // `\$` is one `$` cell.
        let d = layout(&parse("\\$5"));
        let chars: Vec<(char, Vec2)> = glyphs(&d.ink).iter().map(|g| (g.0, g.1)).collect();
        assert_eq!(chars, vec![('$', v(0.0, 1.0)), ('5', v(0.6, 1.0))]);
        assert!(approx_eq(d.width, 1.2, EPS));
    }

    #[test]
    fn tex_layout_tall_formula_grows_its_line() {
        // Arrange
        let frac = boxed("\\frac{a}{b}");

        // Act
        let l = layout(&parse("$\\frac{a}{b}$\nx"));

        // Assert
        let first = frac.ascent + MATH_PAD + frac.descent + MATH_PAD;
        assert!(first > text_layout::LINE_HEIGHT);
        assert!(approx_eq(l.height, first + text_layout::LINE_HEIGHT, EPS));
        let (x, _) = glyph(&l.ink, 'x');
        assert!(
            x.approx_eq(v(0.0, first + text_layout::BASELINE), EPS),
            "{x:?}"
        );
        let (a, _) = glyph(&l.ink, 'a');
        assert!(a.y - GLYPH_ASCENT * SCRIPT_SCALE >= -EPS, "inside the line");
    }

    #[test]
    fn tex_layout_mixes_text_and_formula_on_one_line() {
        // Act
        let l = layout(&parse("f $x^2$ g"));

        // Assert: plain cells, the formula, then cells after it, one line.
        let (f, _) = glyph(&l.ink, 'f');
        let (x, _) = glyph(&l.ink, 'x');
        let (two, _) = glyph(&l.ink, '2');
        let (g, _) = glyph(&l.ink, 'g');
        let formula = 0.6 + 0.6 * SCRIPT_SCALE;
        assert!(f.approx_eq(v(0.0, 1.0), EPS));
        assert!(x.approx_eq(v(1.2, 1.0), EPS));
        assert!(two.approx_eq(v(1.8, 0.6), EPS));
        assert!(g.approx_eq(v(1.2 + formula + 0.6, 1.0), EPS), "{g:?}");
        assert!(approx_eq(l.width, 1.8 + formula + 0.6, EPS));
        assert!(approx_eq(l.height, text_layout::LINE_HEIGHT, EPS));
    }
}
