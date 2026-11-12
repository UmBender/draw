//! Tests of the TeX subset.

use super::alphabets::space;
use super::*;

// ---- T89 \left and \right ------------------------------------------------

/// Every delimiter that draws a glyph: its source and the glyph.
const DRAWN_DELIMITERS: &[(&str, char)] = &[
    ("(", '('),
    (")", ')'),
    ("[", '['),
    ("]", ']'),
    ("\\{", '{'),
    ("\\}", '}'),
    ("|", '|'),
    ("\\|", '‖'),
    ("\\langle", '⟨'),
    ("\\rangle", '⟩'),
    ("\\lfloor", '⌊'),
    ("\\rfloor", '⌋'),
    ("\\lceil", '⌈'),
    ("\\rceil", '⌉'),
];

const TALL: &str = "\\frac{a}{b}";

fn delim(lead: &str, source: &str) -> Delimiter {
    Delimiter {
        lead: lead.to_owned(),
        source: source.to_owned(),
    }
}

fn literal(text: &str) -> Node {
    Node::Literal(text.to_owned())
}

fn pair(open: &str, body: Vec<Node>, close: &str) -> Node {
    Node::LeftRight {
        open: delim("", open),
        body,
        close: delim("", close),
    }
}

/// Height a bracket must reach for `inner`, centred on the axis.
fn target_height(inner: &MathBox) -> f32 {
    2.0 * (inner.ascent - AXIS).max(inner.descent + AXIS) * DELIM_FACTOR
}

/// Top and bottom (y down) of glyph `ch` at `pos` and `scale`.
fn ink_top_bottom(ch: char, pos: Vec2, scale: f32) -> (f32, f32) {
    let m = mm(ch);
    (pos.y - m.y_max * scale, pos.y - m.y_min * scale)
}

#[test]
fn tex_parse_left_right_is_one_node_keeping_its_source() {
    // Act + Assert
    assert_eq!(
        math("\\left(x\\right)"),
        vec![pair("(", vec![Node::Char('x')], ")")]
    );
    assert_eq!(
        math("\\left \\langle x\\right  \\rangle"),
        vec![Node::LeftRight {
            open: delim(" ", "\\langle"),
            body: vec![space(), Node::Char('x')],
            close: delim("  ", "\\rangle"),
        }]
    );
    let src = "$a\\left [ \\frac12 \\right\\}^2 b$";
    assert_eq!(print(&parse(src)), src);
}

#[test]
fn tex_layout_left_right_scales_brackets_to_the_content() {
    // Arrange
    let inner = boxed(TALL);
    let want = target_height(&inner);

    // Act
    let b = boxed(&format!("\\left({TALL}\\right)"));

    // Assert
    let g = glyphs(&b.ink);
    let (first, last) = (g[0], g[g.len() - 1]);
    assert_eq!((first.0, last.0), ('(', ')'));
    let tall = mm('(').y_max - mm('(').y_min;
    assert!(first.2 > 1.0, "{g:?}");
    assert!(approx_eq(first.2, want / tall, EPS), "{g:?}");
    assert!(approx_eq(last.2, want / tall, EPS), "{g:?}");
    assert!(approx_eq(first.1.x, 0.0, EPS));
    assert!(approx_eq(last.1.x + adv(")") * last.2, b.width, EPS));
}

#[test]
fn tex_layout_left_right_grows_every_delimiter() {
    let want = target_height(&boxed(TALL));
    for (src, ch) in DRAWN_DELIMITERS {
        // Act
        let b = boxed(&format!("\\left{src}{TALL}\\right{src}"));

        // Assert
        let g = glyphs(&b.ink);
        let (first, last) = (g[0], g[g.len() - 1]);
        assert_eq!((first.0, last.0), (*ch, *ch), "{src}: {g:?}");
        let tall = mm(*ch).y_max - mm(*ch).y_min;
        assert!(approx_eq(first.2, want / tall, EPS), "{src}: {g:?}");
        assert!(approx_eq(last.2, want / tall, EPS), "{src}: {g:?}");
        assert!(g.iter().all(|g| !g.3), "{src}: {g:?}");
    }
}

#[test]
fn tex_layout_left_right_centres_brackets_on_the_axis() {
    for (src, ch) in DRAWN_DELIMITERS {
        // Act
        let b = boxed(&format!("\\left{src}{TALL}\\right{src}"));

        // Assert
        let (pos, scale) = glyph(&b.ink, *ch);
        let (top, bottom) = ink_top_bottom(*ch, pos, scale);
        assert!(approx_eq(f32::midpoint(top, bottom), -AXIS, EPS), "{src}");
        assert!(b.ascent >= -top - EPS && b.descent >= bottom - EPS, "{src}");
    }
}

#[test]
fn tex_layout_left_right_stops_at_the_scale_limit() {
    // Arrange: a stack far taller than the largest bracket.
    let tower = "\\frac{".repeat(28) + "x" + &"}{x}".repeat(28);

    // Act
    let b = boxed(&format!("\\left({tower}\\right)"));

    // Assert
    let tall = mm('(').y_max - mm('(').y_min;
    assert!(b.ascent + b.descent > LEFT_RIGHT_MAX_SCALE * tall);
    let (_, scale) = glyph(&b.ink, '(');
    assert!(approx_eq(scale, LEFT_RIGHT_MAX_SCALE, EPS), "{scale}");
}

#[test]
fn tex_layout_left_right_null_delimiter_draws_nothing() {
    // Act
    let one = boxed("\\left.x\\right|");
    let other = boxed("\\left(x\\right.");
    let both = boxed("\\left.x\\right.");

    // Assert
    let chars = |b: &MathBox| glyphs(&b.ink).iter().map(|g| g.0).collect::<Vec<_>>();
    assert_eq!(chars(&one), [X, '|']);
    assert_eq!(chars(&other), ['(', X]);
    assert_eq!(chars(&both), [X]);
    assert!(approx_eq(
        both.width,
        2.0 * NULL_DELIM_SPACE + mm(X).advance,
        EPS
    ));
}

#[test]
fn tex_parse_unmatched_left_or_right_is_literal() {
    // Act + Assert
    assert_eq!(
        math("\\left( x"),
        vec![literal("\\left"), Node::Char('('), space(), Node::Char('x')]
    );
    assert_eq!(
        math("x \\right)"),
        vec![
            Node::Char('x'),
            space(),
            literal("\\right"),
            Node::Char(')')
        ]
    );
    assert_eq!(
        math("\\left(\\right)\\right)"),
        vec![pair("(", vec![], ")"), literal("\\right"), Node::Char(')')]
    );
}

#[test]
fn tex_parse_left_or_right_without_a_delimiter_is_literal() {
    // Act + Assert
    assert_eq!(math("\\left"), vec![literal("\\left")]);
    assert_eq!(
        math("\\left x\\right)"),
        vec![
            literal("\\left"),
            space(),
            Node::Char('x'),
            literal("\\right"),
            Node::Char(')'),
        ]
    );
    assert_eq!(
        math("\\left(x\\right y"),
        vec![
            literal("\\left"),
            Node::Char('('),
            Node::Char('x'),
            literal("\\right"),
            space(),
            Node::Char('y'),
        ]
    );
    assert_eq!(
        math("\\left{x}\\right)"),
        vec![
            literal("\\left"),
            Node::Group(vec![Node::Char('x')]),
            literal("\\right"),
            Node::Char(')'),
        ]
    );
}

#[test]
fn tex_parse_left_right_across_a_group_is_literal() {
    // Act + Assert
    assert_eq!(
        math("\\left({x\\right)}"),
        vec![
            literal("\\left"),
            Node::Char('('),
            Node::Group(vec![Node::Char('x'), literal("\\right"), Node::Char(')'),]),
        ]
    );
    assert_eq!(
        math("{\\left(x}\\right)"),
        vec![
            Node::Group(vec![literal("\\left"), Node::Char('('), Node::Char('x'),]),
            literal("\\right"),
            Node::Char(')'),
        ]
    );
    // A stray `}` stays inside the pair, as a literal.
    assert_eq!(
        math("\\left(}\\right)"),
        vec![pair("(", vec![literal("}")], ")")]
    );
}

#[test]
fn tex_parse_left_after_text_is_not_a_body() {
    // Act
    let nodes = math("\\text\\left(x\\right)");

    // Assert
    assert_eq!(
        nodes,
        vec![literal("\\text"), pair("(", vec![Node::Char('x')], ")")]
    );
}

#[test]
fn tex_parse_deep_left_right_is_literal_past_the_limit() {
    // Arrange: deeper than naive recursion could go.
    let src = format!(
        "${}x{}$",
        "\\left(".repeat(10_000),
        "\\right)".repeat(10_000)
    );

    // Act
    let segments = parse(&src);

    // Assert
    assert_eq!(print(&segments), src);
    let l = layout(&segments);
    assert!(l.width.is_finite() && l.height.is_finite());
}

#[test]
fn tex_layout_unmatched_left_right_is_drawn_in_the_warning_colour() {
    for (src, word) in [("\\left( x", "\\left"), ("x \\right)", "\\right")] {
        // Act
        let b = boxed(src);

        // Assert
        let warned: String = glyphs(&b.ink).iter().filter(|g| g.3).map(|g| g.0).collect();
        assert_eq!(warned, word, "{src}");
    }
}

#[test]
fn tex_parse_left_right_pairs_nest() {
    // Act + Assert
    assert_eq!(
        math("\\left(\\left[x\\right]\\right)"),
        vec![pair("(", vec![pair("[", vec![Node::Char('x')], "]")], ")")]
    );
}

#[test]
fn tex_layout_left_right_outer_pair_is_at_least_as_tall_as_inner() {
    // Act
    let b = boxed(&format!("\\left(\\left[{TALL}\\right]\\right)"));

    // Assert
    let g = glyphs(&b.ink);
    let chars: Vec<char> = g.iter().map(|g| g.0).collect();
    assert_eq!(chars, ['(', '[', A, B, ']', ')'], "{g:?}");
    assert!(g[0].2 >= g[1].2 - EPS, "{g:?}");
    assert!(approx_eq(g[0].2, g[5].2, EPS), "{g:?}");
    assert!(approx_eq(g[1].2, g[4].2, EPS), "{g:?}");
    assert!(g[0].1.x < g[1].1.x && g[4].1.x < g[5].1.x, "{g:?}");
}

#[test]
fn tex_layout_left_right_script_attaches_to_the_closing_bracket() {
    // Arrange
    let bare = format!("\\left({TALL}\\right)");
    let pair_box = boxed(&bare);

    // Act
    let sup = boxed(&format!("{bare}^2"));
    let sub = boxed(&format!("{bare}_2"));

    // Assert
    assert!(matches!(
        math(&format!("{bare}^2")).as_slice(),
        [Node::Scripts { base: Some(node), .. }] if matches!(**node, Node::LeftRight { .. })
    ));
    let (up, up_scale) = glyph(&sup.ink, '2');
    assert!(approx_eq(up.x, pair_box.width, EPS), "{up:?}");
    assert!(approx_eq(up_scale, SCRIPT_SCALE, EPS));
    assert!(
        approx_eq(up.y, -(pair_box.ascent - SUP_DROP), EPS),
        "{up:?}"
    );
    let (down, _) = glyph(&sub.ink, '2');
    assert!(approx_eq(down.x, pair_box.width, EPS), "{down:?}");
    assert!(down.y > 0.0, "{down:?}");
}

#[test]
fn tex_layout_left_right_short_content_keeps_normal_size() {
    for content in ["x", "a+b", "2", ""] {
        // Act
        let got = boxed(&format!("\\left({content}\\right)"));
        let plain = boxed(&format!("({content})"));

        // Assert
        let g = glyphs(&got.ink);
        let (first, last) = (g[0], g[g.len() - 1]);
        assert!(approx_eq(first.2, 1.0, EPS), "{content}: {g:?}");
        assert!(approx_eq(last.2, 1.0, EPS), "{content}: {g:?}");
        assert!(approx_eq(first.1.y, 0.0, EPS), "{content}: {g:?}");
        assert!(approx_eq(got.width, plain.width, EPS), "{content}");
    }
    // A superscript barely reaches past the normal bracket.
    let (_, scale) = glyph(&boxed("\\left(x^2\\right)").ink, '(');
    assert!(scale < 1.1, "{scale}");
}

// ---- T90 matrices and cases ----------------------------------------------

const ENVIRONMENT_NAMES: [&str; 5] = ["matrix", "pmatrix", "bmatrix", "vmatrix", "cases"];

fn env_node(name: &str, rows: Vec<Vec<Vec<Node>>>) -> Node {
    Node::Environment {
        name: name.to_owned(),
        lead: String::new(),
        end_lead: String::new(),
        rows,
    }
}

/// A cell holding one typed character.
fn cell(ch: char) -> Vec<Node> {
    vec![Node::Char(ch)]
}

fn matrix_of(body: &str) -> String {
    format!(r"\begin{{matrix}}{body}\end{{matrix}}")
}

fn glyph_chars(b: &MathBox) -> Vec<char> {
    glyphs(&b.ink).iter().map(|g| g.0).collect()
}

#[test]
fn tex_parse_environment_is_one_node_keeping_its_source() {
    // Act + Assert
    assert_eq!(
        math(r"\begin{pmatrix}a&b\\c&d\end{pmatrix}"),
        vec![env_node(
            "pmatrix",
            vec![vec![cell('a'), cell('b')], vec![cell('c'), cell('d')]]
        )]
    );
    assert_eq!(
        math(r"\begin {matrix}a\end  {matrix}"),
        vec![Node::Environment {
            name: "matrix".to_owned(),
            lead: " ".to_owned(),
            end_lead: "  ".to_owned(),
            rows: vec![vec![cell('a')]],
        }]
    );
    let src = r"$x \begin {bmatrix} a & {b} \\ \frac12 \end  {bmatrix}^2 y$";
    assert_eq!(print(&parse(src)), src);
}

#[test]
fn tex_parse_every_environment_splits_cells_and_rows() {
    for name in ENVIRONMENT_NAMES {
        // Act
        let got = math(&format!(r"\begin{{{name}}}x&1\\1&x&2\end{{{name}}}"));

        // Assert
        assert_eq!(
            got,
            vec![env_node(
                name,
                vec![
                    vec![cell('x'), cell('1')],
                    vec![cell('1'), cell('x'), cell('2')],
                ]
            )],
            "{name}"
        );
    }
}

#[test]
fn tex_parse_cells_keep_their_whitespace() {
    // Act + Assert
    assert_eq!(
        math(&matrix_of("a & b")),
        vec![env_node(
            "matrix",
            vec![vec![
                vec![Node::Char('a'), space()],
                vec![space(), Node::Char('b')],
            ]]
        )]
    );
}

#[test]
fn tex_parse_separators_in_a_nested_part_belong_to_it() {
    // Act + Assert: one cell each, the separators are not the table's.
    assert_eq!(
        math(&matrix_of("{a&b}")),
        vec![env_node(
            "matrix",
            vec![vec![vec![Node::Group(vec![
                Node::Char('a'),
                Node::Char('&'),
                Node::Char('b'),
            ])]]]
        )]
    );
    assert_eq!(
        math(&matrix_of(r"{a\\b}")),
        vec![env_node(
            "matrix",
            vec![vec![vec![Node::Group(vec![
                Node::Char('a'),
                literal(r"\\"),
                Node::Char('b'),
            ])]]]
        )]
    );
    assert_eq!(
        math(&matrix_of(r"\text{a&b}&c")),
        vec![env_node(
            "matrix",
            vec![vec![
                vec![Node::Text {
                    lead: String::new(),
                    body: "a&b".to_owned(),
                }],
                cell('c'),
            ]]
        )]
    );
    assert_eq!(
        math(&matrix_of(r"\left(a&b\right)&c")),
        vec![env_node(
            "matrix",
            vec![vec![
                vec![pair(
                    "(",
                    vec![Node::Char('a'), Node::Char('&'), Node::Char('b')],
                    ")"
                )],
                cell('c'),
            ]]
        )]
    );
    let Node::Environment { rows, .. } = &math(&matrix_of(r"a\&b"))[0] else {
        panic!("an environment expected");
    };
    assert_eq!(rows[0].len(), 1);
}

#[test]
fn tex_parse_nested_environments_keep_their_own_cells() {
    // Act
    let got = math(&matrix_of(&format!(r"{}&c", matrix_of(r"a&b\\d"))));

    // Assert
    let inner = env_node("matrix", vec![vec![cell('a'), cell('b')], vec![cell('d')]]);
    assert_eq!(
        got,
        vec![env_node("matrix", vec![vec![vec![inner], cell('c')]])]
    );
}

#[test]
fn tex_parse_script_before_a_separator_stays_literal() {
    // Act + Assert: the `&` still ends the cell.
    assert_eq!(
        math(&matrix_of("x^&y")),
        vec![env_node(
            "matrix",
            vec![vec![vec![Node::Char('x'), literal("^")], cell('y')]]
        )]
    );
}

#[test]
fn tex_parse_environment_at_the_cell_limit_is_a_table() {
    // Arrange
    let wide = vec!["x"; ENVIRONMENT_MAX_CELLS].join("&");
    let tall = vec!["x"; ENVIRONMENT_MAX_CELLS].join(r"\\");

    // Act
    let wide = math(&matrix_of(&wide));
    let tall = math(&matrix_of(&tall));
    let trailing = math(&matrix_of(
        &(vec!["x"; ENVIRONMENT_MAX_CELLS].join(r"\\") + r"\\"),
    ));

    // Assert
    let dims = |nodes: &[Node]| match nodes {
        [Node::Environment { rows, .. }] => (rows.len(), rows[0].len()),
        other => panic!("one environment expected: {other:?}"),
    };
    assert_eq!(dims(&wide), (1, ENVIRONMENT_MAX_CELLS));
    assert_eq!(dims(&tall), (ENVIRONMENT_MAX_CELLS, 1));
    assert_eq!(dims(&trailing), (ENVIRONMENT_MAX_CELLS + 1, 1));
}

#[test]
fn tex_layout_every_environment_draws_without_warning() {
    for name in ENVIRONMENT_NAMES {
        // Act
        let b = boxed(&format!(r"\begin{{{name}}}x&1\\1&x\end{{{name}}}"));

        // Assert
        let g = glyphs(&b.ink);
        assert!(g.iter().all(|g| !g.3), "{name}: {g:?}");
        assert_eq!(g.iter().filter(|g| g.0 == X).count(), 2, "{name}: {g:?}");
        assert_eq!(g.iter().filter(|g| g.0 == '1').count(), 2, "{name}: {g:?}");
    }
}

#[test]
fn tex_layout_matrix_columns_are_centred() {
    // Act
    let b = boxed(&matrix_of(r"x&1\\xxx&1"));

    // Assert
    let (wide, narrow, one) = (boxed("xxx").width, boxed("x").width, boxed("1").width);
    let g = glyphs(&b.ink);
    assert_eq!(glyph_chars(&b), [X, '1', X, X, X, '1']);
    assert!(approx_eq(g[0].1.x, (wide - narrow) / 2.0, EPS), "{g:?}");
    assert!(approx_eq(g[2].1.x, 0.0, EPS), "{g:?}");
    assert!(approx_eq(g[1].1.x, wide + ARRAY_COLUMN_GAP, EPS), "{g:?}");
    assert!(approx_eq(g[5].1.x, wide + ARRAY_COLUMN_GAP, EPS), "{g:?}");
    assert!(approx_eq(b.width, wide + ARRAY_COLUMN_GAP + one, EPS));
}

#[test]
fn tex_layout_matrix_column_is_as_wide_as_its_widest_cell() {
    // Act
    let b = boxed(&matrix_of(r"1&1\\11&1\\1&111"));

    // Assert
    let (first, last, one) = (boxed("11").width, boxed("111").width, boxed("1").width);
    let g = glyphs(&b.ink);
    assert!(approx_eq(b.width, first + ARRAY_COLUMN_GAP + last, EPS));
    assert!(
        approx_eq(g[1].1.x, first + ARRAY_COLUMN_GAP + (last - one) / 2.0, EPS),
        "{g:?}"
    );
}

#[test]
fn tex_layout_matrix_rows_are_a_strut_apart() {
    // Act
    let b = boxed(&matrix_of(r"x\\x"));

    // Assert
    let g = glyphs(&b.ink);
    let step = ARRAY_ROW_ASCENT + ARRAY_ROW_DESCENT;
    assert!(approx_eq(g[1].1.y - g[0].1.y, step, EPS), "{g:?}");
}

#[test]
fn tex_layout_tall_row_pushes_the_next_one_down() {
    // Arrange
    let inner_src = matrix_of(r"x\\x\\x");
    let inner = boxed(&inner_src);

    // Act
    let b = boxed(&matrix_of(&format!(r"{inner_src}\\1")));

    // Assert
    let total = inner.ascent + inner.descent + ARRAY_ROW_ASCENT + ARRAY_ROW_DESCENT;
    let want = -AXIS - total / 2.0 + inner.ascent + inner.descent + ARRAY_ROW_ASCENT;
    let (pos, _) = glyph(&b.ink, '1');
    assert!(approx_eq(pos.y, want, EPS), "{} vs {want}", pos.y);
}

#[test]
fn tex_layout_cases_columns_are_left_aligned() {
    // Act
    let b = boxed(r"\begin{cases}x&1\\xxx&1\end{cases}");

    // Assert
    let (wide, one) = (boxed("xxx").width, boxed("1").width);
    let g = glyphs(&b.ink);
    assert_eq!(glyph_chars(&b), ['{', X, '1', X, X, X, '1']);
    let column = g[1].1.x;
    assert!(column > 0.0, "{g:?}");
    assert!(approx_eq(g[3].1.x, column, EPS), "{g:?}");
    assert!(
        approx_eq(g[2].1.x, column + wide + ARRAY_COLUMN_GAP, EPS),
        "{g:?}"
    );
    assert!(approx_eq(g[6].1.x, g[2].1.x, EPS), "{g:?}");
    assert!(
        approx_eq(
            b.width,
            column + wide + ARRAY_COLUMN_GAP + one + NULL_DELIM_SPACE,
            EPS
        ),
        "{} {g:?}",
        b.width
    );
}

#[test]
fn tex_layout_cases_rows_are_stretched() {
    // Act
    let b = boxed(r"\begin{cases}x\\x\end{cases}");

    // Assert
    let g = glyphs(&b.ink);
    let step = (ARRAY_ROW_ASCENT + ARRAY_ROW_DESCENT) * CASES_STRETCH;
    assert!(approx_eq(g[2].1.y - g[1].1.y, step, EPS), "{g:?}");
}

#[test]
fn tex_layout_cases_has_one_growing_brace() {
    // Act
    let b = boxed(r"\begin{cases}x\\1\\x\end{cases}");

    // Assert
    let g = glyphs(&b.ink);
    assert_eq!(g.iter().filter(|g| g.0 == '{').count(), 1, "{g:?}");
    assert_eq!(g.iter().filter(|g| g.0 == '}').count(), 0, "{g:?}");
    assert!(g[0].2 > 1.5, "{g:?}");
}

#[test]
fn tex_layout_environment_is_centred_on_the_axis() {
    for name in ENVIRONMENT_NAMES {
        // Act: rows of different heights.
        let b = boxed(&format!(
            r"\begin{{{name}}}\frac{{x}}{{\frac{{1}}{{2}}}}\\x\\1\end{{{name}}}"
        ));

        // Assert
        assert!(
            approx_eq((b.ascent - b.descent) / 2.0, AXIS, EPS),
            "{name}: {} {}",
            b.ascent,
            b.descent
        );
    }
}

#[test]
fn tex_layout_matrix_brackets_match_left_right() {
    for (name, open, close) in [
        ("pmatrix", "(", ")"),
        ("bmatrix", "[", "]"),
        ("vmatrix", "|", "|"),
    ] {
        // Arrange
        let body = r"\frac{x}{2}&1\\1&x\\1&1\\x&x";

        // Act
        let got = boxed(&format!(r"\begin{{{name}}}{body}\end{{{name}}}"));
        let want = boxed(&format!(
            r"\left{open}\begin{{matrix}}{body}\end{{matrix}}\right{close}"
        ));

        // Assert
        assert_eq!(got, want, "{name}");
        let (_, scale) = glyph(&got.ink, open.chars().next().unwrap_or('('));
        assert!(scale > 1.5, "{name}: {scale}");
    }
}

#[test]
fn tex_layout_plain_matrix_draws_no_brackets() {
    // Act
    let b = boxed(&matrix_of("x"));

    // Assert
    assert_eq!(glyph_chars(&b), [X]);
    assert!(approx_eq(b.width, boxed("x").width, EPS));
}

#[test]
fn tex_layout_environment_is_an_ordinary_atom_for_scripts() {
    // Arrange
    let src = r"\begin{pmatrix}x\\x\end{pmatrix}";
    let base = boxed(src);

    // Act
    let parsed = math(&format!("{src}^2"));
    let b = boxed(&format!("{src}^2"));

    // Assert
    assert!(
        matches!(parsed.as_slice(), [Node::Scripts { base: Some(node), .. }]
                if matches!(**node, Node::Environment { .. })),
        "{parsed:?}"
    );
    let (pos, _) = glyph(&b.ink, '2');
    assert!(
        approx_eq(pos.x, base.width, EPS),
        "{} {}",
        pos.x,
        base.width
    );
}

#[test]
fn tex_parse_unknown_environment_is_literal() {
    // Act + Assert
    assert_eq!(
        math(r"\begin{foo}a&b\\c\end{foo}"),
        vec![
            literal(r"\begin{foo}"),
            Node::Char('a'),
            Node::Char('&'),
            Node::Char('b'),
            literal(r"\\"),
            Node::Char('c'),
            literal(r"\end{foo}"),
        ]
    );
    assert_eq!(math(r"\begin {foo}"), vec![literal(r"\begin {foo}")]);
    let src = r"$\begin {foo}a&b\\c\end  {foo}$";
    assert_eq!(print(&parse(src)), src);
}

#[test]
fn tex_parse_environment_without_end_is_literal() {
    // Act + Assert
    assert_eq!(
        math(r"\begin{pmatrix}x"),
        vec![literal(r"\begin{pmatrix}"), Node::Char('x')]
    );
}

#[test]
fn tex_parse_end_without_begin_is_literal() {
    // Act + Assert
    assert_eq!(
        math(r"x\end{pmatrix}"),
        vec![Node::Char('x'), literal(r"\end{pmatrix}")]
    );
}

#[test]
fn tex_parse_mismatched_environment_names_are_literal() {
    // Act + Assert
    assert_eq!(
        math(r"\begin{pmatrix}x\end{bmatrix}"),
        vec![
            literal(r"\begin{pmatrix}"),
            Node::Char('x'),
            literal(r"\end{bmatrix}"),
        ]
    );
}

#[test]
fn tex_parse_begin_without_a_name_is_literal() {
    // Act + Assert
    assert_eq!(math(r"\begin"), vec![literal(r"\begin")]);
    assert_eq!(math(r"\end"), vec![literal(r"\end")]);
    assert_eq!(
        math(r"\begin x"),
        vec![literal(r"\begin"), space(), Node::Char('x')]
    );
    assert_eq!(
        math(r"\begin{a b}"),
        vec![
            literal(r"\begin"),
            Node::Group(vec![Node::Char('a'), space(), Node::Char('b')]),
        ]
    );
}

#[test]
fn tex_parse_environment_across_a_group_is_literal() {
    // Act + Assert
    assert_eq!(
        math(r"{\begin{matrix}}\end{matrix}"),
        vec![
            Node::Group(vec![literal(r"\begin{matrix}")]),
            literal(r"\end{matrix}"),
        ]
    );
}

#[test]
fn tex_parse_environment_across_a_left_pair_is_literal() {
    // Act + Assert
    assert_eq!(
        math(r"\begin{matrix}\left(\end{matrix}\right)"),
        vec![
            literal(r"\begin{matrix}"),
            pair("(", vec![literal(r"\end{matrix}")], ")"),
        ]
    );
    assert_eq!(
        math(r"\left(\begin{matrix}\right)\end{matrix}"),
        vec![
            literal(r"\left"),
            Node::Char('('),
            env_node(
                "matrix",
                vec![vec![vec![literal(r"\right"), Node::Char(')')]]]
            ),
        ]
    );
}

#[test]
fn tex_parse_environment_past_the_cell_limit_is_literal() {
    // Arrange
    let wide = vec!["x"; ENVIRONMENT_MAX_CELLS + 1].join("&");
    let tall = vec!["x"; ENVIRONMENT_MAX_CELLS + 1].join(r"\\");

    for body in [wide, tall] {
        // Act
        let got = math(&matrix_of(&body));

        // Assert
        assert_eq!(got.first(), Some(&literal(r"\begin{matrix}")));
        assert_eq!(got.last(), Some(&literal(r"\end{matrix}")));
        assert_eq!(
            print(&[Segment::Math(got)]),
            format!("${}$", matrix_of(&body))
        );
    }
}

#[test]
fn tex_parse_deep_environments_are_literal_past_the_limit() {
    // Arrange
    let src =
        r"\begin{matrix}".repeat(MAX_DEPTH + 8) + "x" + &r"\end{matrix}".repeat(MAX_DEPTH + 8);

    // Act
    let mut nodes = math(&src);

    // Assert
    let mut depth = 0;
    while let [Node::Environment { rows, .. }] = nodes.as_slice() {
        let inner = rows[0][0].clone();
        nodes = inner;
        depth += 1;
    }
    assert_eq!(depth, MAX_DEPTH);
    assert_eq!(nodes.first(), Some(&literal(r"\begin{matrix}")));
    assert_eq!(print(&parse(&format!("${src}$"))), format!("${src}$"));
}

#[test]
fn tex_layout_invalid_environment_is_drawn_in_the_warning_colour() {
    for src in [
        r"\begin{foo}x\end{foo}",
        r"\begin{pmatrix}x",
        r"x\end{pmatrix}",
        r"\begin{pmatrix}x\end{bmatrix}",
    ] {
        // Act
        let g = glyphs(&boxed(src).ink);

        // Assert: the commands warn, the content does not.
        assert!(g.iter().any(|g| g.0 == '\\' && g.3), "{src}: {g:?}");
        assert!(g.iter().any(|g| g.0 == X && !g.3), "{src}: {g:?}");
    }
}

#[test]
fn tex_layout_short_row_leaves_empty_cells() {
    // Arrange
    let full = boxed(&matrix_of(r"x&1\\x&1"));

    // Act
    let short_last = boxed(&matrix_of(r"x&1\\x"));
    let short_first = boxed(&matrix_of(r"x\\x&1"));

    // Assert
    assert_eq!(glyph_chars(&short_last), [X, '1', X]);
    assert_eq!(glyph_chars(&short_first), [X, X, '1']);
    for b in [&short_last, &short_first] {
        assert!(approx_eq(b.width, full.width, EPS), "{}", b.width);
        assert!(approx_eq(b.ascent, full.ascent, EPS), "{}", b.ascent);
        assert!(approx_eq(b.descent, full.descent, EPS), "{}", b.descent);
    }
    let g = glyphs(&short_last.ink);
    let f = glyphs(&full.ink);
    assert!(approx_eq(g[2].1.x, f[2].1.x, EPS), "{g:?}");
}

#[test]
fn tex_layout_trailing_row_break_adds_no_row() {
    // Arrange
    let want = boxed(&matrix_of(r"x\\1"));

    // Act + Assert
    assert_eq!(boxed(&matrix_of(r"x\\1\\")), want);
    assert_eq!(boxed(&matrix_of(r"x\\1\\ ")), want);
}

#[test]
fn tex_layout_empty_environment_draws_only_its_brackets() {
    // Act
    let b = boxed(r"\begin{pmatrix}\end{pmatrix}");
    let plain = boxed(&matrix_of(""));

    // Assert
    let g = glyphs(&b.ink);
    assert_eq!(glyph_chars(&b), ['(', ')']);
    assert!(g.iter().all(|g| approx_eq(g.2, 1.0, EPS)), "{g:?}");
    assert_eq!(plain.ink.len(), 0);
}

// ---- T103 labelled extensible arrows (ADR-T103-1) ---------------------

/// Each extensible arrow, the plain arrow command it grows from and
/// that arrow's glyph.
const XARROW_CASES: &[(&str, &str, char)] = &[
    ("xrightarrow", "rightarrow", '→'),
    ("xleftarrow", "leftarrow", '←'),
    ("xleftrightarrow", "leftrightarrow", '↔'),
    ("xRightarrow", "Rightarrow", '⇒'),
    ("xLeftarrow", "Leftarrow", '⇐'),
    ("xLeftrightarrow", "Leftrightarrow", '⇔'),
    ("xmapsto", "mapsto", '↦'),
    ("xhookrightarrow", "hookrightarrow", '↪'),
];

/// A label wide enough to stretch every arrow.
const LONG: &str = "{1111111}";

fn xarrow(name: &str, below: Option<Vec<Node>>, above: Arg) -> Node {
    Node::XArrow {
        name: name.to_owned(),
        below: below.map(|nodes| Bracket {
            lead: String::new(),
            nodes,
        }),
        above,
    }
}

fn lit(text: &str) -> Node {
    Node::Literal(text.to_owned())
}

/// The x range of a rule's ends.
fn x_span(a: Vec2, b: Vec2) -> (f32, f32) {
    (a.x.min(b.x), a.x.max(b.x))
}

#[test]
fn tex_parse_xarrows_take_a_label_and_an_optional_below_label() {
    for &(name, _, _) in XARROW_CASES {
        // Act
        let above = math(&format!("\\{name}{{f}}"));
        let both = math(&format!("\\{name}[n]{{f}}"));

        // Assert
        let f = || braced(vec![Node::Char('f')]);
        assert_eq!(above, vec![xarrow(name, None, f())], "{name}");
        let below = Some(vec![Node::Char('n')]);
        assert_eq!(both, vec![xarrow(name, below, f())], "{name}");
    }
    assert_eq!(
        math("\\xrightarrow[n\\to\\infty]f"),
        vec![xarrow(
            "xrightarrow",
            Some(vec![Node::Char('n'), sym("to"), sym("infty")]),
            tok(Node::Char('f')),
        )]
    );
    assert_eq!(
        math("\\xleftarrow [a] {b}"),
        vec![Node::XArrow {
            name: "xleftarrow".to_owned(),
            below: Some(Bracket {
                lead: " ".to_owned(),
                nodes: vec![Node::Char('a')],
            }),
            above: Arg {
                lead: " ".to_owned(),
                braced: true,
                nodes: vec![Node::Char('b')],
            },
        }]
    );
}

#[test]
fn tex_parse_xarrow_below_label_ends_at_the_first_bracket_outside_braces() {
    // A `]` in braces belongs to the label.
    assert_eq!(
        math("\\xrightarrow[{]}]{f}"),
        vec![xarrow(
            "xrightarrow",
            Some(vec![Node::Group(vec![Node::Char(']')])]),
            braced(vec![Node::Char('f')]),
        )]
    );
    // Brackets do not nest: the first `]` ends the label, the second
    // is the required one.
    assert_eq!(
        math("\\xrightarrow[[a]]{f}"),
        vec![
            xarrow(
                "xrightarrow",
                Some(vec![Node::Char('['), Node::Char('a')]),
                tok(Node::Char(']')),
            ),
            Node::Group(vec![Node::Char('f')]),
        ]
    );
}

#[test]
fn tex_parse_xarrows_print_back() {
    for src in [
        "\\xrightarrow{f}",
        "\\xrightarrow[n\\to\\infty]{f_n}",
        "\\xleftarrow g",
        "\\xmapsto [ a ] { b }",
        "\\xRightarrow[]{}",
        "\\xrightarrow[{]}]{f}",
        "\\xrightarrow[a",
        "\\xrightarrow[a]",
        "\\xrightarrow{\\xrightarrow{a}}",
        "\\xhookrightarrow",
        "A \\xrightarrow[n\\to\\infty]{f_n} B \\xleftarrow{g} C",
    ] {
        // Arrange
        let text = format!("${src}$");

        // Act + Assert
        assert_eq!(print(&parse(&text)), text);
    }
}

#[test]
fn tex_layout_xarrow_labels_are_script_size_and_centred() {
    // Act: digits are upright, so they draw themselves.
    let b = boxed("\\xrightarrow[2]{1}");

    // Assert
    let (ss, arrow, one_m, two_m) = (SCRIPT_SCALE, mm('→'), mm('1'), mm('2'));
    let (one, s1) = glyph(&b.ink, '1');
    let (two, s2) = glyph(&b.ink, '2');
    assert!(approx_eq(s1, ss, EPS) && approx_eq(s2, ss, EPS));
    assert!(approx_eq(one.x, (b.width - one_m.advance * ss) / 2.0, EPS));
    assert!(approx_eq(two.x, (b.width - two_m.advance * ss) / 2.0, EPS));
    // The label's box ends XARROW_GAP above the arrow's ink.
    let one_bottom = one.y - one_m.y_min.min(0.0) * ss;
    assert!(approx_eq(one_bottom, -arrow.y_max - XARROW_GAP, EPS));
    let two_top = two.y - two_m.y_max * ss;
    assert!(approx_eq(two_top, -arrow.y_min + XARROW_GAP, EPS));
    assert!(approx_eq(b.ascent, one_m.y_max * ss - one.y, EPS));
    assert!(approx_eq(b.descent, two.y - two_m.y_min.min(0.0) * ss, EPS));
}

#[test]
fn tex_layout_xarrow_grows_to_the_wider_label_plus_padding() {
    // Act
    let above = boxed("\\xrightarrow{1111111}");
    let below = boxed("\\xrightarrow[111111]{1}");
    let small = layout_math(&math("\\xrightarrow{1111111}"), MIN_SCALE);

    // Assert
    let one = mm('1').advance;
    let ss = SCRIPT_SCALE;
    assert!(approx_eq(XARROW_PAD, 0.5, EPS));
    assert!(approx_eq(
        above.width,
        7.0 * one * ss + 2.0 * XARROW_PAD,
        EPS
    ));
    assert!(approx_eq(
        below.width,
        6.0 * one * ss + 2.0 * XARROW_PAD,
        EPS
    ));
    // At the smallest size, labels stay at it and padding scales.
    let want = MIN_SCALE * (7.0 * one + 2.0 * XARROW_PAD);
    assert!(approx_eq(small.width, want, EPS), "{}", small.width);
}

#[test]
fn tex_layout_xarrow_is_never_shorter_than_its_arrow() {
    for &(name, _, ch) in XARROW_CASES {
        // Act
        let b = boxed(&format!("\\{name}{{1}}"));

        // Assert
        let label = mm('1').advance * SCRIPT_SCALE + 2.0 * XARROW_PAD;
        let want = label.max(mm(ch).advance);
        assert!(b.width >= mm(ch).advance - EPS, "{name}");
        assert!(
            approx_eq(b.width, want, EPS),
            "{name}: {} vs {want}",
            b.width
        );
    }
}

#[test]
fn tex_layout_stretched_xarrow_draws_its_heads_and_a_shaft() {
    let (right, left) = (mm('→'), mm('←'));

    // → : head at the right end, shaft from the left end of the ink.
    let b = boxed(&format!("\\xrightarrow{LONG}"));
    let w = b.width;
    let (pos, scale) = glyph(&b.ink, '→');
    assert!(approx_eq(scale, 1.0, EPS));
    assert!(approx_eq(pos.x, w - right.advance, EPS) && approx_eq(pos.y, 0.0, EPS));
    let r = rules(&b.ink);
    assert_eq!(r.len(), 1, "{r:?}");
    let (lo, hi) = x_span(r[0].0, r[0].1);
    assert!(approx_eq(lo, right.x_min, EPS), "{lo}");
    assert!(approx_eq(hi, w - right.advance / 2.0, EPS), "{hi}");
    assert!(approx_eq(r[0].2, RULE, EPS));

    // ← : head at the left end, shaft to the right end of the ink.
    let b = boxed(&format!("\\xleftarrow{LONG}"));
    let w = b.width;
    let (pos, _) = glyph(&b.ink, '←');
    assert!(approx_eq(pos.x, 0.0, EPS));
    let r = rules(&b.ink);
    assert_eq!(r.len(), 1, "{r:?}");
    let (lo, hi) = x_span(r[0].0, r[0].1);
    assert!(approx_eq(lo, left.advance / 2.0, EPS), "{lo}");
    assert!(approx_eq(hi, w - (left.advance - left.x_max), EPS), "{hi}");

    // ↔ : both heads, the shaft between their middles.
    let b = boxed(&format!("\\xleftrightarrow{LONG}"));
    let w = b.width;
    assert!(approx_eq(glyph(&b.ink, '←').0.x, 0.0, EPS));
    assert!(approx_eq(glyph(&b.ink, '→').0.x, w - right.advance, EPS));
    assert!(glyphs(&b.ink).iter().all(|g| g.0 != '↔'));
    let r = rules(&b.ink);
    assert_eq!(r.len(), 1, "{r:?}");
    let (lo, hi) = x_span(r[0].0, r[0].1);
    assert!(approx_eq(lo, left.advance / 2.0, EPS), "{lo}");
    assert!(approx_eq(hi, w - right.advance / 2.0, EPS), "{hi}");
}

#[test]
fn tex_layout_stretched_double_xarrow_has_two_shaft_rules() {
    // The bars of `=`, y down.
    let eq = mm('=');
    let (low, high) = (-(eq.y_min + RULE / 2.0), -(eq.y_max - RULE / 2.0));
    for (name, heads) in [
        ("xRightarrow", "⇒"),
        ("xLeftarrow", "⇐"),
        ("xLeftrightarrow", "⇐⇒"),
    ] {
        // Act
        let b = boxed(&format!("\\{name}{LONG}"));

        // Assert
        for ch in heads.chars() {
            glyph(&b.ink, ch);
        }
        let r = rules(&b.ink);
        assert_eq!(r.len(), 2, "{name}: {r:?}");
        let mut ys: Vec<f32> = r.iter().map(|(a, _, _)| a.y).collect();
        ys.sort_by(f32::total_cmp);
        assert!(
            approx_eq(ys[0], high, EPS) && approx_eq(ys[1], low, EPS),
            "{ys:?}"
        );
        for (a, e, t) in r {
            assert!(approx_eq(a.y, e.y, EPS) && approx_eq(t, RULE, EPS));
        }
    }
}

#[test]
fn tex_layout_stretched_xmapsto_and_xhookrightarrow_draw_their_tails() {
    let (to, maps, hook) = (mm('→'), mm('↦'), mm('↪'));

    // ↦ : a → head and a bar as tall as the ↦ glyph's ink.
    let mapsto = boxed(&format!("\\xmapsto{LONG}"));
    let width = mapsto.width;
    assert!(approx_eq(
        glyph(&mapsto.ink, '→').0.x,
        width - to.advance,
        EPS
    ));
    assert!(glyphs(&mapsto.ink).iter().all(|g| g.0 != '↦'));
    let drawn = rules(&mapsto.ink);
    assert_eq!(drawn.len(), 2, "{drawn:?}");
    let bar_x = maps.x_min + RULE / 2.0;
    let bar = drawn.iter().find(|(a, e, _)| approx_eq(a.x, e.x, EPS));
    let Some(&(a, e, _)) = bar else {
        panic!("no bar in {drawn:?}");
    };
    assert!(approx_eq(a.x, bar_x, EPS));
    let (top, bottom) = (a.y.min(e.y), a.y.max(e.y));
    assert!(approx_eq(top, -maps.y_max, EPS) && approx_eq(bottom, -maps.y_min, EPS));
    let shaft = drawn.iter().find(|(a, e, _)| approx_eq(a.y, e.y, EPS));
    let Some(&(a, e, _)) = shaft else {
        panic!("no shaft in {drawn:?}");
    };
    let (lo, hi) = x_span(a, e);
    assert!(approx_eq(lo, bar_x, EPS) && approx_eq(hi, width - to.advance / 2.0, EPS));

    // ↪ : a → head and a hook of rules as high as the ↪ glyph's ink.
    let hooked = boxed(&format!("\\xhookrightarrow{LONG}"));
    let width = hooked.width;
    assert!(approx_eq(
        glyph(&hooked.ink, '→').0.x,
        width - to.advance,
        EPS
    ));
    assert!(glyphs(&hooked.ink).iter().all(|g| g.0 != '↪'));
    let drawn = rules(&hooked.ink);
    assert_eq!(drawn.len(), 1 + XARROW_HOOK_SEGMENTS, "{drawn:?}");
    let ends = drawn.iter().flat_map(|&(a, e, _)| [a, e]);
    let left = ends.clone().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let top = ends.map(|p| p.y).fold(f32::INFINITY, f32::min);
    assert!(approx_eq(left, hook.x_min + RULE / 2.0, 1e-3), "{left}");
    assert!(approx_eq(top, -(hook.y_max - RULE / 2.0), 1e-3), "{top}");
    assert!(drawn.iter().all(|&(_, _, t)| approx_eq(t, RULE, EPS)));
}

#[test]
fn tex_layout_xarrow_is_spaced_as_a_relation() {
    // Act
    let plain = boxed("a\\rightarrow b");
    let empty = boxed("a\\xrightarrow{}b");
    let long = boxed(&format!("a\\xrightarrow{LONG}b"));
    let arrow = boxed(&format!("\\xrightarrow{LONG}"));

    // Assert
    assert_eq!(empty, plain);
    let a = mm(A).advance;
    let want = a + REL_SPACE + arrow.width + REL_SPACE + mm(B).advance;
    assert!(approx_eq(long.width, want, EPS), "{} vs {want}", long.width);
    let (pb, _) = glyph(&long.ink, B);
    assert!(approx_eq(pb.x, a + 2.0 * REL_SPACE + arrow.width, EPS));
}

#[test]
fn tex_layout_xarrow_shaft_sits_on_the_axis() {
    for name in [
        "xrightarrow",
        "xleftarrow",
        "xleftrightarrow",
        "xmapsto",
        "xhookrightarrow",
    ] {
        // Act
        let b = boxed(&format!("\\{name}{LONG}"));

        // Assert
        let flat: Vec<_> = rules(&b.ink)
            .into_iter()
            .filter(|(a, e, _)| approx_eq(a.y, e.y, EPS))
            .collect();
        assert!(!flat.is_empty(), "{name}");
        assert!(
            flat.iter().all(|(a, _, _)| approx_eq(a.y, -AXIS, EPS)),
            "{name}"
        );
        for (ch, pos, _, _) in glyphs(&b.ink) {
            if "→←".contains(ch) {
                assert!(approx_eq(pos.y, 0.0, EPS), "{name}");
            }
        }
    }
    // The axis scales with the arrow.
    let small = layout_math(&math(&format!("\\xrightarrow{LONG}")), MIN_SCALE);
    let r = rules(&small.ink);
    assert!(approx_eq(r[0].0.y, -AXIS * MIN_SCALE, EPS));
    assert!(approx_eq(r[0].2, RULE * MIN_SCALE, EPS));
}

#[test]
fn tex_layout_xarrow_with_empty_labels_is_the_plain_arrow() {
    for &(name, plain, _) in XARROW_CASES {
        // Arrange
        let want = boxed(&format!("\\{plain}"));

        for src in [
            format!("\\{name}{{}}"),
            format!("\\{name}[]{{}}"),
            format!("\\{name}[ ]{{ }}"),
        ] {
            // Act + Assert
            assert_eq!(boxed(&src), want, "{src}");
        }
    }
}

#[test]
fn tex_parse_xarrow_without_label_is_literal() {
    let xr = || lit("\\xrightarrow");
    assert_eq!(math("\\xrightarrow"), vec![xr()]);
    assert_eq!(
        math("\\xrightarrow[a]"),
        vec![xr(), Node::Char('['), Node::Char('a'), Node::Char(']')]
    );
    assert_eq!(
        math("\\xrightarrow[a"),
        vec![xr(), Node::Char('['), Node::Char('a')]
    );
    assert_eq!(
        math("{\\xrightarrow}x"),
        vec![Node::Group(vec![xr()]), Node::Char('x')]
    );
    // The optional label never leaves its group.
    assert_eq!(
        math("{\\xrightarrow[a}]{f}"),
        vec![
            Node::Group(vec![xr(), Node::Char('['), Node::Char('a')]),
            Node::Char(']'),
            Node::Group(vec![Node::Char('f')]),
        ]
    );
}

#[test]
fn tex_layout_xarrow_without_label_is_drawn_in_the_warning_colour() {
    // Act
    let b = boxed("\\xrightarrow");

    // Assert
    let g = glyphs(&b.ink);
    assert!(!g.is_empty() && g.iter().all(|g| g.3), "{g:?}");
    assert!(g.iter().all(|g| g.0 != '→'));
}

#[test]
fn tex_layout_nested_xarrows_stay_finite_and_grow() {
    // Act
    let inner = layout_math(&math(&format!("\\xrightarrow{LONG}")), SCRIPT_SCALE);
    let b = boxed(&format!("\\xrightarrow{{\\xrightarrow{LONG}}}"));
    let deep = format!("${}a{}$", "\\xrightarrow{".repeat(40), "}".repeat(40));
    let l = layout(&parse(&deep));

    // Assert
    assert!(b.width.is_finite() && b.ascent.is_finite() && b.descent.is_finite());
    assert!(approx_eq(b.width, inner.width + 2.0 * XARROW_PAD, EPS));
    let heads: Vec<_> = glyphs(&b.ink).into_iter().filter(|g| g.0 == '→').collect();
    assert_eq!(heads.len(), 2, "{heads:?}");
    assert!(heads.iter().any(|g| approx_eq(g.2, 1.0, EPS)));
    assert!(
        heads
            .iter()
            .any(|g| approx_eq(g.2, SCRIPT_SCALE, EPS) && g.1.y < 0.0)
    );
    assert!(l.width.is_finite() && l.height.is_finite());
}

// ---- T105 display mode (ADR-T105-1) -------------------------------------

/// Math italic `c`.
const C: char = '\u{1D450}';

/// Formula `src` (written without dollars) laid out in display style.
fn shown(src: &str) -> MathBox {
    layout_display(&math(src), 1.0)
}

/// The scale of the first glyph `ch` of `b`.
fn scale_of(b: &MathBox, ch: char) -> f32 {
    glyph(&b.ink, ch).1
}

#[test]
fn tex_parse_double_dollar_is_a_display_formula() {
    // Act
    let alone = parse("$$x$$");
    let in_text = parse("a $$ x+1 $$ b");
    let after_inline = parse("$y$$$x$$");

    // Assert
    assert_eq!(alone, vec![Segment::Display(vec![Node::Char('x')])]);
    assert_eq!(
        in_text,
        vec![
            Segment::Text("a ".to_owned()),
            Segment::Display(vec![
                Node::Space(" ".to_owned()),
                Node::Char('x'),
                Node::Char('+'),
                Node::Char('1'),
                Node::Space(" ".to_owned()),
            ]),
            Segment::Text(" b".to_owned()),
        ]
    );
    assert_eq!(
        after_inline,
        vec![
            Segment::Math(vec![Node::Char('y')]),
            Segment::Display(vec![Node::Char('x')]),
        ]
    );
}

#[test]
fn tex_parse_unclosed_double_dollar_reads_as_before() {
    // Act + Assert: the first `$` is text, the rest reads as before.
    assert_eq!(
        parse("$$x$"),
        vec![
            Segment::Text("$".to_owned()),
            Segment::Math(vec![Node::Char('x')]),
        ]
    );
    assert_eq!(parse("$$x"), vec![Segment::Text("$$x".to_owned())]);
    assert_eq!(parse("$$$$"), vec![Segment::Text("$$$$".to_owned())]);
    // A single `$` inside closes an inline formula instead.
    assert_eq!(
        parse("$$a$b$$"),
        vec![
            Segment::Text("$".to_owned()),
            Segment::Math(vec![Node::Char('a')]),
            Segment::Text("b$$".to_owned()),
        ]
    );
}

#[test]
fn tex_parse_display_formula_does_not_cross_a_line_break() {
    // Act
    let segments = parse("$$a\nb$$");

    // Assert
    assert_eq!(segments, vec![Segment::Text("$$a\nb$$".to_owned())]);
}

#[test]
fn tex_parse_escaped_dollar_inside_display_stays_literal() {
    // Act
    let segments = parse("$$\\$5$$");

    // Assert
    match segments.as_slice() {
        [Segment::Display(nodes)] => {
            // The symbol `\$` draws a dollar, as in `$…$`.
            assert_eq!(nodes[0], sym("$"));
            assert_eq!(nodes[1], Node::Char('5'));
        }
        other => panic!("one display formula expected: {other:?}"),
    }
    assert_eq!(print(&segments), "$$\\$5$$");
}

#[test]
fn tex_print_display_formula_round_trips() {
    for src in [
        "$$x$$",
        "a $$\\sum_{i=1}^n \\frac{a_i}{b_i}$$ b",
        "$$x$$$$y$$",
        "$x$ and $$\\displaystyle y \\textstyle z$$\n$$1$$",
        "$\\dfrac12 + \\tfrac{1}{2}$",
    ] {
        // Act
        let segments = parse(src);

        // Assert
        assert_eq!(print(&segments), src);
    }
}

#[test]
fn tex_layout_display_big_operators_use_the_display_scale() {
    for (name, ch) in [("sum", '∑'), ("prod", '∏'), ("int", '∫'), ("bigcup", '⋃')] {
        // Act
        let display = shown(&format!("\\{name}"));
        let inline = boxed(&format!("\\{name}"));

        // Assert
        let (pos, k) = glyph(&display.ink, ch);
        assert!(approx_eq(k, DISPLAY_OP_SCALE, EPS), "{name}: {k}");
        assert!(
            approx_eq(scale_of(&inline, ch), BIG_OP_SCALE, EPS),
            "{name}"
        );
        // Its middle on the axis (y down).
        let middle = pos.y - (mm(ch).y_max + mm(ch).y_min) * k / 2.0;
        assert!(approx_eq(middle, -AXIS, EPS), "{name}: {middle}");
        assert!(approx_eq(display.width, mm(ch).advance * k, EPS), "{name}");
    }
    assert!(approx_eq(DISPLAY_OP_SCALE, 1.8, EPS));
}

#[test]
fn tex_layout_display_sum_limits_are_centred_above_and_below() {
    // Act
    let b = shown("\\sum_{k=1}^{n}");

    // Assert
    let (sum, big) = glyph(&b.ink, '∑');
    assert!(approx_eq(big, DISPLAY_OP_SCALE, EPS));
    let centre = sum.x + adv("∑") * big / 2.0;
    let (n, ns) = glyph(&b.ink, N);
    let (k, ks) = glyph(&b.ink, K);
    assert!(approx_eq(ns, SCRIPT_SCALE, EPS) && approx_eq(ks, SCRIPT_SCALE, EPS));
    assert!(approx_eq(n.x + mm(N).advance * ns / 2.0, centre, EPS));
    let below = (mm(K).advance + adv("=1")) * ks;
    assert!(approx_eq(k.x + below / 2.0, centre, EPS), "{k:?} {centre}");
    let sum_top = sum.y - mm('∑').y_max * big;
    let sum_bottom = sum.y - mm('∑').y_min * big;
    assert!(n.y - mm(N).y_min * ns <= sum_top - LIMIT_GAP + EPS);
    assert!(k.y - mm(K).y_max * ks >= sum_bottom + LIMIT_GAP - EPS);
}

#[test]
fn tex_layout_display_integral_scripts_sit_beside_the_sign() {
    // Act
    let b = shown("\\int_a^b");

    // Assert
    let (sign, k) = glyph(&b.ink, '∫');
    assert!(approx_eq(k, DISPLAY_OP_SCALE, EPS));
    let (lo, _) = glyph(&b.ink, A);
    let (hi, _) = glyph(&b.ink, B);
    let advance = mm('∫').advance * k;
    let italic = mm('∫').italic * k;
    assert!(approx_eq(hi.x, sign.x + advance, EPS), "{hi:?}");
    assert!(approx_eq(lo.x, sign.x + advance - italic, EPS), "{lo:?}");
    let top = sign.y - mm('∫').y_max * k;
    let bottom = sign.y - mm('∫').y_min * k;
    assert!(hi.y > top && lo.y < bottom, "{hi:?} {lo:?} {top} {bottom}");
}

#[test]
fn tex_layout_display_scripts_are_text_style() {
    // Act
    let op = shown("x^{\\sum}");
    let frac = shown("x^{\\frac{a}{b}}");

    // Assert: a script is laid out as in an inline formula.
    assert!(approx_eq(
        scale_of(&op, '∑'),
        SCRIPT_SCALE * BIG_OP_SCALE,
        EPS
    ));
    assert_eq!(op, boxed("x^{\\sum}"));
    assert!(approx_eq(
        scale_of(&frac, A),
        script_scale(SCRIPT_SCALE),
        EPS
    ));
}

#[test]
fn tex_layout_display_fraction_parts_are_full_size() {
    // Act
    let b = shown("\\frac{a}{b}");

    // Assert
    let bars = rules(&b.ink);
    assert_eq!(bars.len(), 1);
    let (p, q, width) = bars[0];
    assert!(approx_eq(p.y, -AXIS, EPS) && approx_eq(q.y, -AXIS, EPS));
    assert!(approx_eq(width, RULE, EPS));
    let (num, ns) = glyph(&b.ink, A);
    let (den, ds) = glyph(&b.ink, B);
    assert!(approx_eq(ns, 1.0, EPS) && approx_eq(ds, 1.0, EPS));
    assert!(num.y - mm(A).y_min <= -AXIS - RULE / 2.0 - FRAC_GAP + EPS);
    assert!(den.y - mm(B).y_max >= -AXIS + RULE / 2.0 + FRAC_GAP - EPS);
    let middle = b.width / 2.0;
    assert!(approx_eq(num.x + mm(A).advance / 2.0, middle, EPS));
    assert!(approx_eq(den.x + mm(B).advance / 2.0, middle, EPS));
    assert!(b.width > boxed("\\frac{a}{b}").width);
}

#[test]
fn tex_layout_display_nested_fraction_shrinks() {
    // Act
    let b = shown("\\frac{\\frac{a}{b}}{c}");

    // Assert: the outer parts are text style, the inner ones script.
    assert!(approx_eq(scale_of(&b, A), SCRIPT_SCALE, EPS));
    assert!(approx_eq(scale_of(&b, B), SCRIPT_SCALE, EPS));
    assert!(approx_eq(scale_of(&b, C), 1.0, EPS));
}

#[test]
fn tex_layout_display_binomial_and_infix_parts_are_full_size() {
    // Act
    let binom = shown("\\binom{n}{k}");
    let choose = shown("n \\choose k");
    let over = shown("a \\over b");

    // Assert
    assert!(approx_eq(scale_of(&binom, N), 1.0, EPS));
    assert!(approx_eq(scale_of(&binom, K), 1.0, EPS));
    assert!(scale_of(&binom, '(') > scale_of(&boxed("\\binom{n}{k}"), '('));
    assert_eq!(choose, binom);
    assert_eq!(over, shown("\\frac{a}{b}"));
}

#[test]
fn tex_parse_dfrac_and_tfrac_take_two_arguments() {
    // Act + Assert
    assert_eq!(
        math("\\dfrac{a}{b}"),
        vec![Node::Command {
            name: "dfrac".to_owned(),
            args: vec![braced(vec![Node::Char('a')]), braced(vec![Node::Char('b')]),],
        }]
    );
    assert_eq!(
        math("\\tfrac12"),
        vec![Node::Command {
            name: "tfrac".to_owned(),
            args: vec![tok(Node::Char('1')), tok(Node::Char('2'))],
        }]
    );
    assert_eq!(
        math("\\dfrac a"),
        vec![
            Node::Literal("\\dfrac".to_owned()),
            Node::Space(" ".to_owned()),
            Node::Char('a'),
        ]
    );
}

#[test]
fn tex_layout_dfrac_and_dbinom_have_full_size_parts_inline() {
    // Act
    let dfrac = boxed("\\dfrac{a}{b}");
    let dbinom = boxed("\\dbinom{n}{k}");

    // Assert
    assert_eq!(dfrac, shown("\\frac{a}{b}"));
    assert_eq!(dbinom, shown("\\binom{n}{k}"));
    assert!(approx_eq(scale_of(&dfrac, A), 1.0, EPS));
    // Its parts are text style: an operator in them is inline-sized.
    let part = boxed("\\dfrac{\\sum}{2}");
    assert!(approx_eq(scale_of(&part, '∑'), BIG_OP_SCALE, EPS));
}

#[test]
fn tex_layout_tfrac_and_tbinom_have_script_parts_in_display() {
    // Act
    let tfrac = shown("\\tfrac{a}{b}");
    let tbinom = shown("\\tbinom{n}{k}");

    // Assert
    assert_eq!(tfrac, boxed("\\frac{a}{b}"));
    assert_eq!(tbinom, boxed("\\binom{n}{k}"));
    assert_eq!(boxed("\\tfrac{a}{b}"), boxed("\\frac{a}{b}"));
}

#[test]
fn tex_parse_style_switches_are_nodes() {
    // Act + Assert
    assert_eq!(
        math("\\displaystyle x"),
        vec![
            Node::MathStyle(true),
            Node::Space(" ".to_owned()),
            Node::Char('x'),
        ]
    );
    assert_eq!(math("\\textstyle"), vec![Node::MathStyle(false)]);
    assert_eq!(
        math("\\displaystylex"),
        vec![Node::Literal("\\displaystylex".to_owned())]
    );
    for src in [
        "$\\displaystyle\\sum$",
        "${\\textstyle x}^2$",
        "$\\displaystyle^2$",
    ] {
        assert_eq!(print(&parse(src)), src);
    }
}

#[test]
fn tex_layout_displaystyle_switches_the_rest_of_its_group() {
    // Act
    let after = boxed("\\sum\\displaystyle\\sum");
    let grouped = boxed("{\\displaystyle\\int}\\int");
    let part = boxed("\\frac{\\displaystyle\\sum}{b}");

    // Assert
    let sums: Vec<f32> = glyphs(&after.ink)
        .iter()
        .filter(|g| g.0 == '∑')
        .map(|g| g.2)
        .collect();
    assert_eq!(sums.len(), 2);
    assert!(approx_eq(sums[0], BIG_OP_SCALE, EPS), "{sums:?}");
    assert!(approx_eq(sums[1], DISPLAY_OP_SCALE, EPS), "{sums:?}");
    let ints: Vec<f32> = glyphs(&grouped.ink)
        .iter()
        .filter(|g| g.0 == '∫')
        .map(|g| g.2)
        .collect();
    assert_eq!(ints.len(), 2);
    assert!(approx_eq(ints[0], DISPLAY_OP_SCALE, EPS), "{ints:?}");
    assert!(approx_eq(ints[1], BIG_OP_SCALE, EPS), "{ints:?}");
    assert!(approx_eq(
        scale_of(&part, '∑'),
        SCRIPT_SCALE * DISPLAY_OP_SCALE,
        EPS
    ));
    // A display fraction after the switch has full-size parts.
    assert_eq!(boxed("\\displaystyle\\frac{a}{b}"), shown("\\frac{a}{b}"));
}

#[test]
fn tex_layout_textstyle_switches_back_in_display() {
    // Act
    let op = shown("\\textstyle\\sum");
    let frac = shown("\\textstyle\\frac{a}{b}");

    // Assert
    assert!(approx_eq(scale_of(&op, '∑'), BIG_OP_SCALE, EPS));
    assert_eq!(frac, boxed("\\frac{a}{b}"));
}

#[test]
fn tex_layout_style_switch_is_not_an_atom() {
    // Act
    let spaced = boxed("a\\displaystyle+b");
    let base = boxed("\\displaystyle^2");

    // Assert: `+` keeps its binary spacing.
    assert_eq!(spaced, boxed("a+b"));
    assert!(base.width.is_finite() && base.width > 0.0);
    assert!(approx_eq(scale_of(&base, '2'), SCRIPT_SCALE, EPS));
}

#[test]
fn tex_layout_display_formula_sits_on_its_own_line() {
    // Act
    let l = layout(&parse("ab $$x$$ cd"));

    // Assert: "ab " on the first line, the formula on the second, " cd"
    // on the third with its space kept.
    let line = text_layout::LINE_HEIGHT;
    let (a, _) = glyph(&l.ink, 'a');
    let (x, _) = glyph(&l.ink, X);
    let (c, _) = glyph(&l.ink, 'c');
    assert!(a.approx_eq(v(0.0, text_layout::BASELINE), EPS), "{a:?}");
    assert!(approx_eq(x.y, line + text_layout::BASELINE, EPS), "{x:?}");
    assert!(
        c.approx_eq(
            v(text_layout::ADVANCE, 2.0 * line + text_layout::BASELINE),
            EPS
        ),
        "{c:?}"
    );
    assert!(approx_eq(l.height, 3.0 * line, EPS));
}

#[test]
fn tex_layout_display_formula_is_centred_on_the_block_width() {
    // Arrange
    let two = shown("2");

    // Act
    let l = layout(&parse("abcdefghij\n$$2$$"));

    // Assert
    let width = 10.0 * text_layout::ADVANCE;
    assert!(approx_eq(l.width, width, EPS));
    let (pos, _) = glyph(&l.ink, '2');
    assert!(
        approx_eq(pos.x, (width - two.width) / 2.0, EPS),
        "{pos:?} {}",
        two.width
    );
    // A display formula before the text is centred as well.
    let first = layout(&parse("$$2$$abcdefghij"));
    let (pos, _) = glyph(&first.ink, '2');
    assert!(approx_eq(pos.x, (width - two.width) / 2.0, EPS), "{pos:?}");
}

#[test]
fn tex_layout_line_break_around_display_adds_no_empty_line() {
    // Act
    let around = layout(&parse("a\n$$2$$\nb"));
    let alone = layout(&parse("$$2$$"));
    let pair = layout(&parse("$$2$$$$3$$"));
    let trailing = layout(&parse("$$2$$\n"));

    // Assert
    let line = text_layout::LINE_HEIGHT;
    assert!(
        approx_eq(around.height, 3.0 * line, EPS),
        "{}",
        around.height
    );
    assert!(approx_eq(alone.height, line, EPS), "{}", alone.height);
    assert!(approx_eq(pair.height, 2.0 * line, EPS), "{}", pair.height);
    assert!(approx_eq(trailing.height, 2.0 * line, EPS));
    let (b, _) = glyph(&around.ink, 'b');
    assert!(b.approx_eq(v(0.0, 2.0 * line + text_layout::BASELINE), EPS));
}

#[test]
fn tex_layout_wide_display_formula_sets_the_block_width() {
    // Arrange
    let src = "\\sum_{i=1}^{n} \\frac{a_i}{b_i} + \\sum_{i=1}^{n} \\frac{a_i}{b_i}";
    let b = shown(src);

    // Act
    let l = layout(&parse(&format!("ab\n$${src}$$")));

    // Assert
    assert!(b.width > 2.0 * text_layout::ADVANCE);
    assert!(l.width >= b.width - EPS, "{} {}", l.width, b.width);
    assert!(l.height > 2.0 * text_layout::LINE_HEIGHT);
    let (a, _) = glyph(&l.ink, 'a');
    assert!(approx_eq(a.x, 0.0, EPS));
    let room = l.bounds().expand(1e-4);
    assert!(l.ink.iter().all(|ink| {
        let ib = ink.bounds();
        room.contains(ib.min) && room.contains(ib.max)
    }));
}
