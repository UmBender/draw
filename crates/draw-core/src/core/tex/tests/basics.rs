//! Tests of the TeX subset.

use super::*;

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
fn tex_parse_mathcal_takes_one_argument() {
    assert_eq!(
        math("\\mathcal{A}"),
        vec![Node::Command {
            name: "mathcal".to_owned(),
            args: vec![braced(vec![Node::Char('A')])],
        }]
    );
    assert_eq!(
        math("\\mathcal L"),
        vec![Node::Command {
            name: "mathcal".to_owned(),
            args: vec![Arg {
                lead: " ".to_owned(),
                braced: false,
                nodes: vec![Node::Char('L')],
            }],
        }]
    );
    assert_eq!(
        math("\\mathcal"),
        vec![Node::Literal("\\mathcal".to_owned())]
    );
}

#[test]
fn tex_parse_accents_take_one_argument() {
    for def in ACCENTS {
        // Act
        let braced_arg = math(&format!("\\{}{{x}}", def.name));
        let token_arg = math(&format!("\\{} \\alpha", def.name));

        // Assert
        assert_eq!(
            braced_arg,
            vec![Node::Command {
                name: def.name.to_owned(),
                args: vec![braced(vec![Node::Char('x')])],
            }],
            "{}",
            def.name
        );
        assert_eq!(
            token_arg,
            vec![Node::Command {
                name: def.name.to_owned(),
                args: vec![Arg {
                    lead: " ".to_owned(),
                    braced: false,
                    nodes: vec![sym("alpha")],
                }],
            }],
            "{}",
            def.name
        );
        assert!(symbol(def.name).is_none(), "{} is also a symbol", def.name);
        let rows = ACCENTS.iter().filter(|a| a.name == def.name).count();
        assert_eq!(rows, 1, "{} has one row", def.name);
        assert_eq!(accent(def.name), Some(def));
    }
    let names: Vec<&str> = ACCENTS.iter().map(|a| a.name).collect();
    assert_eq!(
        names,
        [
            "vec", "hat", "tilde", "dot", "ddot", "dddot", "bar", "overline"
        ]
    );
}

#[test]
fn tex_parse_accent_without_argument_is_literal() {
    assert_eq!(math("\\vecx"), vec![Node::Literal("\\vecx".to_owned())]);
    assert_eq!(math("\\vec"), vec![Node::Literal("\\vec".to_owned())]);
    assert_eq!(
        math("\\hat^2"),
        vec![Node::Scripts {
            base: Some(Box::new(Node::Literal("\\hat".to_owned()))),
            first: script(ScriptKind::Sup, tok(Node::Char('2'))),
            second: None,
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
fn tex_parse_integrals_are_symbols() {
    // Act
    let nodes = math("\\int\\iint\\iiint\\oint\\oiint");

    // Assert
    let names = ["int", "iint", "iiint", "oint", "oiint"];
    assert_eq!(nodes, names.map(sym).to_vec());
    let texts = names.map(|name| symbol(name).map(|s| (s.text, s.class)));
    assert_eq!(
        texts,
        ["∫", "∬", "∭", "∮", "∯"].map(|t| Some((t, Class::Integral)))
    );
    assert_eq!(math("\\intx"), vec![Node::Literal("\\intx".to_owned())]);
}

#[test]
fn tex_parse_physics_symbols_have_their_text_and_class() {
    // Arrange: the *Physics symbols* table of T36, and `\,`.
    let table = [
        ("partial", "∂", Class::Ord),
        ("nabla", "∇", Class::Ord),
        ("hbar", "ℏ", Class::Ord),
        ("ell", "ℓ", Class::Ord),
        ("Re", "ℜ", Class::Ord),
        ("Im", "ℑ", Class::Ord),
        ("AA", "Å", Class::Ord),
        ("degree", "°", Class::Ord),
        ("forall", "∀", Class::Ord),
        ("exists", "∃", Class::Ord),
        ("emptyset", "∅", Class::Ord),
        ("prime", "′", Class::Ord),
        ("vdots", "⋮", Class::Ord),
        ("ddots", "⋱", Class::Ord),
        ("vert", "|", Class::Ord),
        ("Vert", "‖", Class::Ord),
        ("propto", "∝", Class::Rel),
        ("sim", "∼", Class::Rel),
        ("simeq", "≃", Class::Rel),
        ("cong", "≅", Class::Rel),
        ("ll", "≪", Class::Rel),
        ("gg", "≫", Class::Rel),
        ("lesssim", "≲", Class::Rel),
        ("gtrsim", "≳", Class::Rel),
        ("doteq", "≐", Class::Rel),
        ("coloneqq", "≔", Class::Rel),
        ("parallel", "∥", Class::Rel),
        ("perp", "⊥", Class::Rel),
        ("mid", "∣", Class::Rel),
        ("rightarrow", "→", Class::Rel),
        ("leftrightarrow", "↔", Class::Rel),
        ("Leftarrow", "⇐", Class::Rel),
        ("Leftrightarrow", "⇔", Class::Rel),
        ("uparrow", "↑", Class::Rel),
        ("downarrow", "↓", Class::Rel),
        ("mapsto", "↦", Class::Rel),
        ("rightleftharpoons", "⇌", Class::Rel),
        ("mp", "∓", Class::Bin),
        ("otimes", "⊗", Class::Bin),
        ("wedge", "∧", Class::Bin),
        ("vee", "∨", Class::Bin),
        ("circ", "∘", Class::Bin),
        ("ast", "∗", Class::Bin),
        ("star", "⋆", Class::Bin),
        ("bullet", "∙", Class::Bin),
        ("dagger", "†", Class::Bin),
        ("langle", "⟨", Class::Open),
        ("rangle", "⟩", Class::Close),
        (",", "\u{2006}", Class::Ord),
    ];

    for (name, text, class) in table {
        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((text, class)),
            "{name}"
        );
    }
}

#[test]
fn tex_parse_hebrew_letters_have_their_text_and_class() {
    // Arrange: the table of T61.
    let table = [
        ("aleph", "\u{2135}"),
        ("beth", "\u{2136}"),
        ("gimel", "\u{2137}"),
        ("daleth", "\u{2138}"),
    ];

    for (name, text) in table {
        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((text, Class::Ord)),
            "{name}"
        );
    }
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
    // A control symbol outside the table; a `\` before the closing
    // `$` would escape it instead.
    assert_eq!(
        math("\\@x"),
        vec![Node::Literal("\\@".to_owned()), Node::Char('x')]
    );
}

// ---- T99 spacing commands (ADR-T99-1) ------------------------------------

/// The sources of the spacing commands and their widths in em.
const SPACINGS: [(&str, f32); 7] = [
    ("\\:", 4.0 / 18.0),
    ("\\;", 5.0 / 18.0),
    ("\\!", -3.0 / 18.0),
    ("\\quad", 1.0),
    ("\\qquad", 2.0),
    ("\\ ", INTERWORD_SPACE),
    ("~", INTERWORD_SPACE),
];

#[test]
fn tex_parse_spacing_commands_are_kerns() {
    for (src, _) in SPACINGS {
        assert_eq!(math(src), vec![Node::Kern(src.to_owned())], "{src:?}");
    }
    assert_eq!(
        math("a\\;b"),
        vec![
            Node::Char('a'),
            Node::Kern("\\;".to_owned()),
            Node::Char('b')
        ]
    );
    assert_eq!(
        math("\\quad x"),
        vec![
            Node::Kern("\\quad".to_owned()),
            Node::Space(" ".to_owned()),
            Node::Char('x'),
        ]
    );
    // A longer word is not a spacing command.
    assert_eq!(math("\\quadx"), vec![Node::Literal("\\quadx".to_owned())]);
}

#[test]
fn tex_parse_spacing_commands_print_back() {
    for src in [
        "$a\\:b\\;c\\!d\\quad e\\qquad f\\ g~h$",
        "$x^\\; y_{\\!}$",
        "a~b $\\,\\!\\,$ c\\ d",
    ] {
        assert_eq!(print(&parse(src)), src);
    }
}

#[test]
fn tex_layout_spacing_commands_have_their_widths() {
    let ab = mm(A).advance + mm(B).advance;
    for (src, em) in SPACINGS {
        // Act
        let w = boxed(&format!("a{src} b")).width;

        // Assert
        assert!(approx_eq(w - ab, em, EPS), "{src:?}: {}", w - ab);
    }
    assert!(approx_eq(INTERWORD_SPACE, 1.0 / 3.0, EPS));
}

#[test]
fn tex_layout_spacing_commands_scale_in_scripts() {
    let plain = boxed("a^{xy}").width;
    for (src, em) in SPACINGS {
        // Act
        let w = boxed(&format!("a^{{x{src} y}}")).width;

        // Assert
        assert!(
            approx_eq(w - plain, em * SCRIPT_SCALE, EPS),
            "{src:?}: {}",
            w - plain
        );
    }
}

#[test]
fn tex_layout_negative_space_pulls_neighbours_together() {
    // Act
    let b = boxed("a\\!b");

    // Assert
    let pulled = mm(A).advance - 3.0 / 18.0;
    assert!(
        approx_eq(b.width, pulled + mm(B).advance, EPS),
        "{}",
        b.width
    );
    let (pos, _) = glyph(&b.ink, B);
    assert!(approx_eq(pos.x, pulled, EPS), "{pos:?}");
}

#[test]
fn tex_layout_negative_space_never_makes_width_negative() {
    // Arrange
    let many = "\\!".repeat(8);

    // Act
    let alone = boxed(&many);
    let leading = boxed(&format!("{many}a"));
    let between = boxed(&format!("a{many}b"));
    let block = layout(&parse(&format!("${many}$")));

    // Assert
    assert!(approx_eq(alone.width, 0.0, EPS), "{}", alone.width);
    assert!(
        approx_eq(leading.width, mm(A).advance, EPS),
        "{}",
        leading.width
    );
    assert!(
        approx_eq(between.width, mm(B).advance, EPS),
        "{}",
        between.width
    );
    for b in [&alone, &leading, &between] {
        for (_, pos, _, _) in glyphs(&b.ink) {
            assert!(pos.x >= -EPS, "{pos:?}");
        }
    }
    assert!(block.width >= 0.0, "{}", block.width);
}

#[test]
fn tex_layout_spacing_commands_keep_atom_classes() {
    let width = |src| boxed(src).width;
    let ab = mm(A).advance + mm(B).advance;
    let thick = 5.0 / 18.0;

    // A binary operator keeps its space beside explicit spaces.
    assert!(approx_eq(
        width("a\\;+\\;b"),
        ab + adv("+") + 2.0 * BIN_SPACE + 2.0 * thick,
        EPS
    ));
    // A leading minus stays unary after a space.
    assert!(approx_eq(
        width("\\;-a"),
        thick + adv("\u{2212}") + mm(A).advance,
        EPS
    ));
    // A relation keeps its space too.
    assert!(approx_eq(
        width("a=\\quad b"),
        ab + adv("=") + 2.0 * REL_SPACE + 1.0,
        EPS
    ));
}

#[test]
fn tex_layout_only_spacing_commands_is_finite() {
    for src in [
        "$\\quad$",
        "$\\!$",
        "$\\!\\!\\!\\!$",
        "$~~~$",
        "$\\ \\;\\:\\qquad$",
        "$x^{\\!\\!}$",
        "$\\;^2$",
        "$\\frac{\\!}{\\quad}$",
        "$\\sqrt{\\!\\!\\!}$",
        "$\\left(\\!\\!\\right)$",
    ] {
        // Act
        let l = layout(&parse(src));

        // Assert
        assert!(l.width.is_finite() && l.width >= 0.0, "{src:?}");
        assert!(l.height.is_finite() && l.height > 0.0, "{src:?}");
        for ink in &l.ink {
            let ib = ink.bounds();
            assert!(ib.min.is_finite() && ib.max.is_finite(), "{src:?} {ink:?}");
        }
    }
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
            "\\mathcal", "f", "j", "\\text{日}", "\\int", "\\oint", "\\,",
            "\\partial", "\\vec", "\\hat", "\\bar", "\\dot", "\\ddot", "\\tilde",
            "\\overline", "\\mathrm", "\\mathit", "\\mathnormal", "\\Gamma",
            "\\left", "\\right", "\\left(", "\\right)", "\\left.", "\\right.",
            "\\left\\{", "\\right\\rangle", "\\left\\|",
            "\\begin", "\\end", "\\begin{matrix}", "\\end{matrix}", "\\begin{pmatrix}",
            "\\end{pmatrix}", "\\begin{cases}", "\\end{cases}", "\\begin{foo}", "\\end{foo}",
            "&", "\\\\",
            "\\:", "\\;", "\\!", "\\!\\!\\!", "\\quad", "\\qquad", "\\ ", "~",
            "\\xrightarrow", "\\xLeftrightarrow", "\\xmapsto", "\\xhookrightarrow",
            "[", "]", "\\xrightarrow[n]{f}",
            "$$", "$$", "\\dfrac", "\\tfrac", "\\dbinom", "\\tbinom", "\\displaystyle",
            "\\textstyle", "\\over",
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

#[test]
fn tex_layout_char_uses_its_glyph_metrics() {
    // Act
    let b = boxed("x");

    // Assert
    let x = mm(X);
    assert!(approx_eq(b.width, x.advance, EPS));
    assert!(approx_eq(b.ascent, x.y_max, EPS));
    assert!(approx_eq(b.descent, -x.y_min, EPS));
    assert_eq!(glyphs(&b.ink), vec![(X, v(0.0, 0.0), 1.0, false)]);
    assert_eq!(faces(&b.ink), vec![(X, Face::Math)]);
}

#[test]
fn tex_layout_superscript_is_raised_and_scaled() {
    // Act
    let b = boxed("x^2");

    // Assert: TeX's rule — the larger of the shift, the drop from the
    // base's top and the script's bottom limit.
    let s = SCRIPT_SCALE;
    let up = SUP_SHIFT
        .max(mm(X).y_max - SUP_DROP)
        .max(-mm('2').y_min * s + SUP_BOTTOM_MIN);
    let (pos, scale) = glyph(&b.ink, '2');
    let x = mm(X).advance + mm(X).italic;
    assert!(pos.approx_eq(v(x, -up), EPS), "{pos:?}");
    assert!(approx_eq(scale, s, EPS));
    assert!(approx_eq(b.width, x + adv("2") * s + SCRIPT_SPACE, EPS));
    // A script of a script stops at the smallest scale.
    let (_, scale) = glyph(&boxed("x^{y^z}").ink, Z);
    assert!(approx_eq(scale, MIN_SCALE, EPS));
    assert!(glyphs(&boxed("x^{y^z}").ink).iter().any(|g| g.0 == Y));
}

#[test]
fn tex_layout_subscript_is_lowered_and_scaled() {
    // Act
    let b = boxed("x_2");

    // Assert
    let s = SCRIPT_SCALE;
    let down = SUB_SHIFT
        .max(-mm(X).y_min - SUB_DROP)
        .max(mm('2').y_max * s - SUB_TOP_MAX);
    let (pos, scale) = glyph(&b.ink, '2');
    assert!(pos.approx_eq(v(mm(X).advance, down), EPS), "{pos:?}");
    assert!(approx_eq(scale, s, EPS));
}

#[test]
fn tex_layout_subscript_top_stays_low() {
    // Arrange: a subscript with a superscript of its own is tall.
    let sub = layout_math(&math("b^2"), SCRIPT_SCALE);
    assert!(sub.ascent - SUB_SHIFT > SUB_TOP_MAX, "the case is tall");

    // Act
    let b = boxed("x_{b^2}");

    // Assert: its top is no higher than SUB_TOP_MAX.
    let (pos, _) = glyph(&b.ink, B);
    assert!(approx_eq(pos.y - sub.ascent, -SUB_TOP_MAX, EPS), "{pos:?}");
}

#[test]
fn tex_layout_superscript_clears_the_italic_correction() {
    // Arrange
    let f = mm(F);
    assert!(f.italic > 0.0, "italic f has a correction");

    // Act
    let sup = boxed("f^2");
    let sub = boxed("f_2");

    // Assert: superscripts start after it, subscripts tuck under.
    let (p, _) = glyph(&sup.ink, '2');
    let (q, _) = glyph(&sub.ink, '2');
    assert!(approx_eq(p.x, f.advance + f.italic, EPS), "{p:?}");
    assert!(approx_eq(q.x, f.advance, EPS), "{q:?}");
}

#[test]
fn tex_layout_sub_and_sup_share_the_base() {
    for src in ["x_i^2", "x^2_i"] {
        // Act
        let b = boxed(src);

        // Assert: both start at the base's right edge, apart by at
        // least SCRIPT_GAP.
        let (sup, s) = glyph(&b.ink, '2');
        let (sub, _) = glyph(&b.ink, I);
        let x = mm(X).advance;
        assert!(
            approx_eq(sup.x, x + mm(X).italic, EPS) && approx_eq(sub.x, x, EPS),
            "{src}"
        );
        let sup_bottom = sup.y - mm('2').y_min * s;
        let sub_top = sub.y - mm(I).y_max * s;
        assert!(
            sub_top - sup_bottom >= SCRIPT_GAP - EPS,
            "{src}: {sup:?} {sub:?}"
        );
        let scripts = adv("2").max(mm(I).advance) * s;
        assert!(approx_eq(b.width, x + scripts + SCRIPT_SPACE, EPS));
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
    assert!(p.x >= 0.0 && q.x <= b.width && q.x - p.x >= mm(B).advance * SCRIPT_SCALE);
    let (num, ns) = glyph(&b.ink, A);
    let (den, ds) = glyph(&b.ink, B);
    assert!(approx_eq(ns, SCRIPT_SCALE, EPS) && approx_eq(ds, SCRIPT_SCALE, EPS));
    assert!(num.y - mm(A).y_min * ns <= -AXIS - RULE / 2.0 - FRAC_GAP + EPS);
    assert!(den.y - mm(B).y_max * ds >= -AXIS + RULE / 2.0 + FRAC_GAP - EPS);
    assert!(num.y <= -FRAC_NUM_SHIFT + EPS && den.y >= FRAC_DEN_SHIFT - EPS);
    let middle = b.width / 2.0;
    assert!(approx_eq(num.x + mm(A).advance * ns / 2.0, middle, EPS));
    assert!(approx_eq(den.x + mm(B).advance * ds / 2.0, middle, EPS));
    assert!(b.ascent > mm(A).y_max && b.descent > -mm(B).y_min);
}

#[test]
fn tex_layout_sqrt_draws_a_root_sign_over_the_content() {
    // Act
    let b = boxed("\\sqrt{x}");

    // Assert: four strokes, the last a bar over the whole content.
    let strokes = rules(&b.ink);
    assert_eq!(strokes.len(), 4);
    let (x, scale) = glyph(&b.ink, X);
    assert!(approx_eq(scale, 1.0, EPS) && approx_eq(x.y, 0.0, EPS));
    let (p, q, _) = strokes[3];
    assert!(approx_eq(p.y, q.y, EPS), "overbar is horizontal");
    assert!(p.y < -mm(X).y_max, "overbar above the content");
    assert!(
        p.x <= x.x && q.x >= x.x + mm(X).advance,
        "overbar covers the content"
    );
    assert!(b.ascent >= -p.y && b.width >= q.x);
}

#[test]
fn tex_layout_binom_has_parens_and_no_bar() {
    // Act
    let b = boxed("\\binom{n}{k}");

    // Assert
    assert_eq!(rules(&b.ink), []);
    let g = glyphs(&b.ink);
    let (first, last) = (g[0], g[g.len() - 1]);
    assert_eq!((first.0, last.0), ('(', ')'));
    assert!(first.2 > 1.0 && approx_eq(first.2, last.2, EPS), "{g:?}");
    assert!(approx_eq(first.1.x, 0.0, EPS));
    assert!(approx_eq(last.1.x + adv(")") * last.2, b.width, EPS));
    assert!(faces(&b.ink).iter().all(|f| f.1 == Face::Math));
    let (n, ns) = glyph(&b.ink, N);
    let (k, ks) = glyph(&b.ink, K);
    assert!(n.y < k.y);
    assert!(approx_eq(
        n.x + mm(N).advance * ns / 2.0,
        k.x + mm(K).advance * ks / 2.0,
        EPS
    ));
}

#[test]
fn tex_layout_sum_limits_are_centred_above_and_below() {
    // Act
    let b = boxed("\\sum_{k=1}^{n}");

    // Assert
    let (sum, big) = glyph(&b.ink, '∑');
    assert!(approx_eq(big, BIG_OP_SCALE, EPS));
    let centre = sum.x + adv("∑") * big / 2.0;
    let (n, ns) = glyph(&b.ink, N);
    let (k, ks) = glyph(&b.ink, K);
    assert!(
        approx_eq(n.x + mm(N).advance * ns / 2.0, centre, EPS),
        "{n:?} {centre}"
    );
    // "k=1" at script scale, without relation space.
    let below = (mm(K).advance + adv("=1")) * ks;
    assert!(approx_eq(k.x + below / 2.0, centre, EPS), "{k:?} {centre}");
    let sum_top = sum.y - mm('∑').y_max * big;
    let sum_bottom = sum.y - mm('∑').y_min * big;
    assert!(n.y - mm(N).y_min * ns <= sum_top - LIMIT_GAP + EPS);
    assert!(k.y - mm(K).y_max * ks >= sum_bottom + LIMIT_GAP - EPS);
    // Words with limits too, in upright type.
    let m = boxed("\\min_k");
    let (k, ks) = glyph(&m.ink, K);
    assert!(
        approx_eq(k.x + mm(K).advance * ks / 2.0, adv("min") / 2.0, EPS),
        "{k:?}"
    );
    assert!(k.y > -mm('n').y_min);
}

#[test]
fn tex_layout_integral_is_enlarged_like_sum() {
    // Act
    let int = boxed("\\int");
    let sum = boxed("\\sum");

    // Assert
    let (s, k) = glyph(&int.ink, '∫');
    let (_, sum_k) = glyph(&sum.ink, '∑');
    assert!(approx_eq(k, BIG_OP_SCALE, EPS));
    assert!(approx_eq(k, sum_k, EPS));
    // Its middle on the axis (y down).
    let middle = s.y - (mm('∫').y_max + mm('∫').y_min) * k / 2.0;
    assert!(approx_eq(middle, -AXIS, EPS), "{middle}");
    assert!(approx_eq(int.width, mm('∫').advance * k, EPS));
}

#[test]
fn tex_layout_integral_scripts_sit_beside_the_sign() {
    // Act
    let int = boxed("\\int_a^b");

    // Assert
    let (sign, k) = glyph(&int.ink, '∫');
    let (lo, _) = glyph(&int.ink, A);
    let (hi, _) = glyph(&int.ink, B);
    let advance = mm('∫').advance * k;
    let italic = mm('∫').italic * k;
    assert!(italic > 0.0);
    assert!(approx_eq(hi.x, sign.x + advance, EPS), "{hi:?}");
    assert!(approx_eq(lo.x, sign.x + advance - italic, EPS), "{lo:?}");
    assert!(lo.x > sign.x && hi.x > lo.x);
    assert!(hi.y < 0.0 && lo.y > 0.0, "{hi:?} {lo:?}");
    // Beside, not above and below: the scripts overlap the sign's
    // height.
    let top = sign.y - mm('∫').y_max * k;
    let bottom = sign.y - mm('∫').y_min * k;
    assert!(hi.y > top && lo.y < bottom, "{hi:?} {lo:?} {top} {bottom}");
    // The other integrals too.
    let oint = boxed("\\oint_C");
    let (sign, k) = glyph(&oint.ink, '∮');
    let (c, _) = glyph(&oint.ink, '\u{1D436}');
    let shift = (mm('∮').advance - mm('∮').italic) * k;
    assert!(approx_eq(c.x, sign.x + shift, EPS), "{c:?}");
}

#[test]
fn tex_layout_integral_superscript_starts_at_the_advance() {
    for (src, ch) in [
        ("\\int", '∫'),
        ("\\iint", '∬'),
        ("\\iiint", '∭'),
        ("\\oint", '∮'),
        ("\\oiint", '∯'),
    ] {
        // Act
        let int = boxed(&format!("{src}_a^b"));

        // Assert: the font's advance already holds the correction.
        let (sign, k) = glyph(&int.ink, ch);
        let (lo, _) = glyph(&int.ink, A);
        let (hi, _) = glyph(&int.ink, B);
        let advance = mm(ch).advance * k;
        let italic = mm(ch).italic * k;
        assert!(approx_eq(hi.x, sign.x + advance, EPS), "{src} {hi:?}");
        assert!(
            approx_eq(lo.x, sign.x + advance - italic, EPS),
            "{src} {lo:?}"
        );
    }
}

#[test]
fn tex_layout_integral_superscript_hugs_the_sign() {
    // Act
    let int = boxed("\\int^b");

    // Assert: it starts right of the ink, but not by more than 0.1 em.
    let (sign, k) = glyph(&int.ink, '∫');
    let (hi, _) = glyph(&int.ink, B);
    let gap = hi.x - (sign.x + mm('∫').x_max * k);
    assert!(gap > 0.0 && gap < 0.1, "{gap}");
}

#[test]
fn tex_layout_integral_gets_operator_space() {
    // Act
    let w = boxed("\\int f").width;

    // Assert
    let int = mm('∫').advance * BIG_OP_SCALE;
    assert!(approx_eq(w, int + OP_SPACE + mm(F).advance, EPS), "{w}");
    // A minus after the sign is unary.
    let w = boxed("\\int -f").width;
    let minus = adv("\u{2212}");
    assert!(
        approx_eq(w, int + OP_SPACE + minus + mm(F).advance, EPS),
        "{w}"
    );
}

#[test]
fn tex_layout_thin_space_is_a_sixth_of_an_em() {
    // Act
    let w = boxed("a\\,b").width;

    // Assert
    let thin = w - mm(A).advance - mm(B).advance;
    assert!(approx_eq(thin, mm('\u{2006}').advance, EPS), "{thin}");
    assert!(approx_eq(thin, 1.0 / 6.0, 1e-3), "{thin}");
}

#[test]
fn tex_layout_relations_and_binary_operators_get_space() {
    let width = |src| boxed(src).width;
    let ab = mm(A).advance + mm(B).advance;

    assert!(approx_eq(
        width("a=b"),
        ab + adv("=") + 2.0 * REL_SPACE,
        EPS
    ));
    assert!(approx_eq(
        width("a+b"),
        ab + adv("+") + 2.0 * BIN_SPACE,
        EPS
    ));
    // A leading minus is unary; spaces in the source are ignored.
    assert!(approx_eq(width("-a"), adv("\u{2212}") + mm(A).advance, EPS));
    assert!(approx_eq(
        width("a + b"),
        ab + adv("+") + 2.0 * BIN_SPACE,
        EPS
    ));
    // A function before an ordinary atom, not before a bracket.
    let n = mm(N).advance;
    assert!(approx_eq(width("\\log n"), adv("log") + OP_SPACE + n, EPS));
    assert!(approx_eq(width("\\log(n)"), adv("log()") + n, EPS));
    // Before an operator after an atom, and after punctuation.
    assert!(approx_eq(
        width("n\\log n"),
        n + adv("log") + 2.0 * OP_SPACE + n,
        EPS
    ));
    assert!(approx_eq(width("a,b"), ab + adv(",") + OP_SPACE, EPS));
    // No relation space inside scripts.
    let script = (mm(I).advance + adv("=1")) * SCRIPT_SCALE;
    assert!(approx_eq(
        width("a_{i=1}"),
        mm(A).advance + script + SCRIPT_SPACE,
        EPS
    ));
}

#[test]
fn tex_layout_minus_is_the_math_minus() {
    let chars: Vec<char> = glyphs(&boxed("a-b").ink).iter().map(|g| g.0).collect();

    assert_eq!(chars, vec![A, MINUS, B]);
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
        vec![('{', true), (A, false)]
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
    let first = frac.ascent.max(text_layout::BASELINE - MATH_PAD)
        + MATH_PAD
        + frac
            .descent
            .max(text_layout::LINE_HEIGHT - text_layout::BASELINE - MATH_PAD)
        + MATH_PAD;
    assert!(approx_eq(l.height, first + text_layout::LINE_HEIGHT, EPS));
    let (x, _) = glyph(&l.ink, 'x');
    assert!(
        x.approx_eq(v(0.0, first + text_layout::BASELINE), EPS),
        "{x:?}"
    );
    let (a, s) = glyph(&l.ink, A);
    assert!(a.y - mm(A).y_max * s >= -EPS, "inside the line");
    // A formula taller than a text line grows it.
    let tall = layout(&parse("$\\frac{\\sum_k^n}{b}$"));
    assert!(tall.height > text_layout::LINE_HEIGHT);
}

#[test]
fn tex_layout_mixes_text_and_formula_on_one_line() {
    // Act
    let l = layout(&parse("f $x^2$ g"));

    // Assert: plain cells, the formula, then cells after it, one line.
    let (f, _) = glyph(&l.ink, 'f');
    let (x, _) = glyph(&l.ink, X);
    let (two, _) = glyph(&l.ink, '2');
    let (g, _) = glyph(&l.ink, 'g');
    let formula = boxed("x^2");
    let (two_in, _) = glyph(&formula.ink, '2');
    assert!(f.approx_eq(v(0.0, 1.0), EPS));
    assert!(x.approx_eq(v(1.2, 1.0), EPS));
    assert!(two.approx_eq(v(1.2, 1.0) + two_in, EPS), "{two:?}");
    assert!(g.approx_eq(v(1.2 + formula.width + 0.6, 1.0), EPS), "{g:?}");
    assert!(approx_eq(l.width, 1.8 + formula.width + 0.6, EPS));
    assert!(approx_eq(l.height, text_layout::LINE_HEIGHT, EPS));
}

#[test]
fn tex_layout_line_holds_italic_overhangs() {
    // Arrange: italic j hangs left of its pen, italic f right of its
    // advance.
    let (j, f) = (mm(J), mm(F));
    assert!(j.x_min < 0.0 && f.x_max > f.advance);

    // Act
    let left = layout(&parse("$j$"));
    let right = layout(&parse("$f$ g"));
    let inside = layout(&parse("a$j$"));

    // Assert: a formula starting a line moves right by its overhang;
    // the line is as wide as its rightmost ink.
    let (pos, _) = glyph(&left.ink, J);
    assert!(approx_eq(pos.x, -j.x_min, EPS), "{pos:?}");
    let (g, _) = glyph(&right.ink, 'g');
    assert!(approx_eq(g.x, f.x_max + 0.6, EPS), "{g:?}");
    let (pos, _) = glyph(&inside.ink, J);
    assert!(approx_eq(pos.x, 0.6, EPS), "{pos:?}");
    for l in [&left, &right, &inside] {
        let b = l.bounds().expand(1e-4);
        assert!(l.ink.iter().all(|ink| {
            let ib = ink.bounds();
            b.contains(ib.min) && b.contains(ib.max)
        }));
    }
}

// ---- T39 accents (ADR-T39-1) -------------------------------------------

/// The ink accent `def` adds: its glyph, or its rule.
pub(super) fn mark_ink(ink: &[Ink], def: &AccentDef) -> Vec<Ink> {
    ink.iter()
        .copied()
        .filter(|ink| match (*ink, def.mark) {
            (Ink::Glyph { ch, .. }, Mark::Glyph(mark, _)) => ch == mark,
            (Ink::Rule { .. }, Mark::Bar | Mark::Overline) => true,
            _ => false,
        })
        .collect()
}

/// The bounds of all of `ink`; panics without ink.
pub(super) fn ink_box(ink: &[Ink]) -> Aabb {
    match ink.iter().map(Ink::bounds).reduce(|a, b| a.union(&b)) {
        Some(b) => b,
        None => panic!("no ink"),
    }
}

/// `\name{arg}`.
fn accented(def: &AccentDef, arg: &str) -> MathBox {
    boxed(&format!("\\{}{{{arg}}}", def.name))
}

#[test]
fn tex_layout_accent_sits_above_its_argument() {
    for def in ACCENTS {
        for arg in ["x", "F", "2", "AB"] {
            // Arrange
            let plain = boxed(arg);

            // Act
            let b = accented(def, arg);

            // Assert: the mark's bottom is the gap above the argument.
            let mark = mark_ink(&b.ink, def);
            assert_eq!(mark.len(), 1, "\\{}{{{arg}}}: {:?}", def.name, b.ink);
            let m = ink_box(&mark);
            let top = -(plain.ascent + ACCENT_GAP);
            assert!(
                approx_eq(m.max.y, top, EPS),
                "\\{}{{{arg}}}: {m:?}",
                def.name
            );
            assert!(
                m.max.y < ink_box(&plain.ink).min.y,
                "\\{}{{{arg}}}",
                def.name
            );
        }
    }
}

#[test]
fn tex_layout_accent_is_centred_on_an_upright_argument() {
    for def in ACCENTS.iter().filter(|d| d.mark != Mark::Overline) {
        for arg in ["2", "AB", "\\Gamma", "xy"] {
            // Arrange
            let centre = ink_box(&boxed(arg).ink).center().x;

            // Act
            let b = accented(def, arg);

            // Assert
            let m = ink_box(&mark_ink(&b.ink, def));
            assert!(
                approx_eq(m.center().x, centre, EPS),
                "\\{}{{{arg}}}: {} vs {centre}",
                def.name,
                m.center().x
            );
        }
    }
}

#[test]
fn tex_layout_accent_over_an_italic_letter_is_skewed() {
    for def in ACCENTS.iter().filter(|d| d.mark != Mark::Overline) {
        for arg in ["x", "F", "\\alpha", " v "] {
            // Arrange
            let plain = boxed(arg);
            let centre = ink_box(&plain.ink).center().x;

            // Act
            let b = accented(def, arg);

            // Assert: the top of a slanted letter lies right of its
            // middle.
            let m = ink_box(&mark_ink(&b.ink, def));
            let want = centre + ACCENT_SKEW * plain.ascent;
            assert!(
                approx_eq(m.center().x, want, EPS),
                "\\{}{{{arg}}}: {} vs {want}",
                def.name,
                m.center().x
            );
        }
    }
}

#[test]
fn tex_layout_bar_is_a_short_rule() {
    // Act
    let b = boxed("\\bar{AB}");

    // Assert: as long as the macron, whatever the argument.
    let strokes = rules(&b.ink);
    assert_eq!(strokes.len(), 1);
    let (p, q, width) = strokes[0];
    assert!(approx_eq(p.y, q.y, EPS), "horizontal");
    assert!(approx_eq((q.x - p.x).abs(), ACCENT_BAR, EPS), "{p:?} {q:?}");
    assert!(approx_eq(width, RULE, EPS));
}

#[test]
fn tex_layout_overline_spans_its_argument() {
    for arg in ["AB", "x", "x^2"] {
        // Arrange
        let plain = boxed(arg);

        // Act
        let b = boxed(&format!("\\overline{{{arg}}}"));

        // Assert
        let strokes = rules(&b.ink);
        assert_eq!(strokes.len(), 1, "{arg}");
        let (p, q, width) = strokes[0];
        assert!(approx_eq(p.y, q.y, EPS), "{arg}: horizontal");
        assert!(approx_eq(p.x.min(q.x), 0.0, EPS), "{arg}: {p:?}");
        assert!(approx_eq(p.x.max(q.x), plain.width, EPS), "{arg}: {q:?}");
        assert!(approx_eq(width, RULE, EPS));
    }
}

#[test]
fn tex_layout_accent_keeps_the_argument_width_and_descent() {
    for def in ACCENTS {
        for arg in ["y", "F", "AB", "x_i"] {
            // Arrange
            let plain = boxed(arg);

            // Act
            let b = accented(def, arg);

            // Assert: the ascent reaches the mark's top.
            let name = def.name;
            assert!(approx_eq(b.width, plain.width, EPS), "\\{name}{{{arg}}}");
            assert!(
                approx_eq(b.descent, plain.descent, EPS),
                "\\{name}{{{arg}}}"
            );
            assert!(approx_eq(b.italic, plain.italic, EPS), "\\{name}{{{arg}}}");
            let m = ink_box(&mark_ink(&b.ink, def));
            assert!(approx_eq(b.ascent, -m.min.y, EPS), "\\{name}{{{arg}}}");
        }
    }
}

#[test]
fn tex_layout_accent_takes_scripts() {
    // Arrange
    let vec_f = boxed("\\vec{F}");
    let dot_x = boxed("\\dot{x}");

    // Act
    let sub = boxed("\\vec{F}_x");
    let sup = boxed("\\dot{x}^2");
    let inner = boxed("x^{\\hat a}");

    // Assert: scripts start after the accented box, the superscript
    // above its accent's reach.
    let (x, _) = glyph(&sub.ink, X);
    assert!(approx_eq(x.x, vec_f.width, EPS), "{x:?}");
    assert!(x.y > 0.0);
    let (two, _) = glyph(&sup.ink, '2');
    assert!(approx_eq(two.x, dot_x.width + dot_x.italic, EPS), "{two:?}");
    assert!(-two.y >= dot_x.ascent - SUP_DROP - EPS, "{two:?}");
    let (hat, scale) = glyph(&inner.ink, '^');
    assert!(approx_eq(scale, SCRIPT_SCALE, EPS), "{hat:?}");
}

// ---- AC-3 faces and mapping (ADR-T33-1) --------------------------------

#[test]
fn tex_math_letters_are_italic() {
    // Act
    let got = faces(&boxed("aAzZh").ink);

    // Assert
    assert_eq!(
        got,
        vec![
            (A, Face::Math),
            ('\u{1D434}', Face::Math),
            (Z, Face::Math),
            ('\u{1D44D}', Face::Math),
            ('\u{210E}', Face::Math),
        ]
    );
}

#[test]
fn tex_lowercase_greek_is_italic_and_capitals_upright() {
    // Act
    let got = faces(
        &boxed("\\alpha\\omega\\varphi\\phi\\epsilon\\varepsilon\\varsigma\\Gamma\\Omega").ink,
    );

    // Assert
    let chars: Vec<char> = got.iter().map(|g| g.0).collect();
    assert_eq!(
        chars,
        vec![
            '\u{1D6FC}',
            '\u{1D714}',
            '\u{1D711}',
            '\u{1D719}',
            '\u{1D716}',
            '\u{1D700}',
            '\u{1D70D}',
            'Γ',
            'Ω',
        ]
    );
    assert!(got.iter().all(|g| g.1 == Face::Math));
}

#[test]
fn tex_digits_operators_and_functions_are_upright_math() {
    // Act
    let got = faces(&boxed("1+\\log x\\le\\infty\\text{ok}").ink);

    // Assert
    let chars: String = got.iter().map(|g| g.0).collect();
    assert_eq!(chars, format!("1+log{X}≤∞ok"));
    assert!(got.iter().all(|g| g.1 == Face::Math));
    let pmod: String = faces(&boxed("\\pmod{n}").ink).iter().map(|g| g.0).collect();
    assert_eq!(pmod, format!("(mod{N})"));
}

#[test]
fn tex_mathcal_maps_to_script_letters() {
    // Act
    let got = faces(&boxed("\\mathcal{ABE}x\\mathcal L\\mathcal{a1}").ink);

    // Assert: capitals turn script (holes from Letterlike Symbols),
    // anything else is drawn as outside `\mathcal`.
    assert_eq!(
        got.iter().map(|g| g.0).collect::<Vec<_>>(),
        vec!['\u{1D49C}', '\u{212C}', '\u{2130}', X, '\u{2112}', A, '1']
    );
    assert!(got.iter().all(|g| g.1 == Face::Math));
    assert!(math_metrics::get('\u{1D49C}').is_some());
}

#[test]
fn tex_hebrew_letters_are_upright_math() {
    // Arrange
    let hebrew = ['\u{2135}', '\u{2136}', '\u{2137}', '\u{2138}'];

    // Act
    let plain = boxed("\\aleph\\beth\\gimel\\daleth");
    let cal = boxed("\\mathcal{\\aleph\\beth\\gimel\\daleth}");

    // Assert: no style maps them, the math face draws them unwarned.
    for b in [plain, cal] {
        let got = glyphs(&b.ink);
        assert_eq!(got.iter().map(|g| g.0).collect::<Vec<_>>(), hebrew);
        assert!(got.iter().all(|g| !g.3), "{got:?}");
        assert!(faces(&b.ink).iter().all(|g| g.1 == Face::Math));
    }
    for ch in hebrew {
        assert_eq!(italic(ch), ch);
        assert_eq!(calligraphic(ch), ch);
        assert!(math_metrics::get(ch).is_some(), "{ch:?}");
    }
}

#[test]
fn tex_layout_hebrew_letter_takes_scripts() {
    // Arrange
    const ALEPH: char = '\u{2135}';
    let s = SCRIPT_SCALE;

    // Act
    let sub = boxed("\\aleph_0");
    let pow = boxed("2^{\\aleph_0}");

    // Assert: the subscript sits at the letter's advance, lowered.
    let down = SUB_SHIFT
        .max(-mm(ALEPH).y_min - SUB_DROP)
        .max(mm('0').y_max * s - SUB_TOP_MAX);
    let (pos, scale) = glyph(&sub.ink, '0');
    assert!(pos.approx_eq(v(mm(ALEPH).advance, down), EPS), "{pos:?}");
    assert!(approx_eq(scale, s, EPS));
    // As an exponent the letter is raised and scaled, and its own
    // subscript is smaller still and below it.
    let (aleph, a_scale) = glyph(&pow.ink, ALEPH);
    let (zero, z_scale) = glyph(&pow.ink, '0');
    assert!(aleph.y < 0.0, "{aleph:?}");
    assert!(approx_eq(a_scale, s, EPS));
    assert!(z_scale < a_scale);
    assert!(zero.y > aleph.y, "{zero:?}");
    assert!(zero.x > aleph.x, "{zero:?}");
}

/// Trigonometric and hyperbolic functions of T63; the last five are
/// not in LaTeX's base set.
const TRIG: [&str; 18] = [
    "sin", "cos", "tan", "cot", "sec", "csc", "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh",
    "coth", "arccot", "arcsec", "arccsc", "sech", "csch",
];

#[test]
fn tex_parse_trig_functions_are_operator_words() {
    for name in TRIG {
        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((name, Class::Op)),
            "{name}"
        );
    }
}

#[test]
fn tex_trig_functions_are_upright_math_words() {
    for name in TRIG {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: the name itself, upright, from the math face.
        let got = glyphs(&b.ink);
        let chars: String = got.iter().map(|g| g.0).collect();
        assert_eq!(chars, name);
        assert!(got.iter().all(|g| !g.3), "{name}: {got:?}");
        assert!(faces(&b.ink).iter().all(|g| g.1 == Face::Math), "{name}");
        for ch in name.chars() {
            assert!(math_metrics::get(ch).is_some(), "{ch:?}");
        }
        assert!(approx_eq(b.width, adv(name), EPS), "{name}");
    }
}

#[test]
fn tex_layout_trig_function_spacing_matches_log() {
    // Arrange
    let width = |src: &str| boxed(src).width;
    let (x, n) = (mm(X).advance, mm(N).advance);
    let log_gap = width("\\log x") - adv("log") - x;

    for name in TRIG {
        // Act
        let gap = width(&format!("\\{name} x")) - adv(name) - x;
        let both = width(&format!("n\\{name} n"));
        let paren = width(&format!("\\{name}(x)"));

        // Assert: the thin operator space, as for `\log`.
        assert!(approx_eq(gap, log_gap, EPS), "{name}: {gap}");
        assert!(approx_eq(gap, OP_SPACE, EPS), "{name}: {gap}");
        assert!(
            approx_eq(both, n + adv(name) + 2.0 * OP_SPACE + n, EPS),
            "{name}: {both}"
        );
        assert!(
            approx_eq(paren, adv(name) + adv("()") + x, EPS),
            "{name}: {paren}"
        );
    }
}

#[test]
fn tex_layout_trig_function_power_sits_beside_the_word() {
    // Arrange
    let s = SCRIPT_SCALE;
    let word = boxed("\\sin");

    // Act
    let b = boxed("\\sin^2\\theta");

    // Assert: the exponent at the word's advance, raised and scaled
    // like a letter's (not centred above it like a limit).
    let up = SUP_SHIFT
        .max(word.ascent - SUP_DROP)
        .max(-mm('2').y_min * s + SUP_BOTTOM_MIN);
    let at = word.width + word.italic;
    let (pos, scale) = glyph(&b.ink, '2');
    assert!(pos.approx_eq(v(at, -up), EPS), "{pos:?}");
    assert!(approx_eq(scale, s, EPS));
    // θ follows the script after the operator's thin space.
    let theta = italic('θ');
    let (t, t_scale) = glyph(&b.ink, theta);
    let after = at + adv("2") * s + SCRIPT_SPACE + OP_SPACE;
    assert!(t.approx_eq(v(after, 0.0), EPS), "{t:?}");
    assert!(approx_eq(t_scale, 1.0, EPS));
    let order: Vec<char> = glyphs(&b.ink).iter().map(|g| g.0).collect();
    assert_eq!(order, vec!['s', 'i', 'n', '2', theta]);
}

/// Named operators of T64 that take limits, with what they draw.
const NAMED_LIMITS: [(&str, &str); 7] = [
    ("inf", "inf"),
    ("sup", "sup"),
    ("liminf", "lim\u{2006}inf"),
    ("limsup", "lim\u{2006}sup"),
    ("det", "det"),
    ("Pr", "Pr"),
    ("gcd", "gcd"),
];

/// Named operators of T64 with scripts beside them.
const NAMED_OPS: [&str; 7] = ["exp", "lg", "deg", "dim", "ker", "hom", "arg"];

/// An `\operatorname` node.
fn opname(star: bool, lead: &str, body: &str) -> Node {
    Node::OperatorName {
        star,
        lead: lead.to_owned(),
        body: body.to_owned(),
    }
}

#[test]
fn tex_parse_named_limit_operators_take_limits() {
    for (name, text) in NAMED_LIMITS {
        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((text, Class::Limits)),
            "{name}"
        );
    }
}

#[test]
fn tex_parse_named_function_operators_are_operator_words() {
    for name in NAMED_OPS {
        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((name, Class::Op)),
            "{name}"
        );
    }
}

#[test]
fn tex_named_operators_are_upright_math_words() {
    let all = NAMED_LIMITS.into_iter().chain(NAMED_OPS.map(|n| (n, n)));
    for (name, text) in all {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: its text, upright, from the math face.
        let got = glyphs(&b.ink);
        let chars: String = got.iter().map(|g| g.0).collect();
        assert_eq!(chars, text, "{name}");
        assert!(got.iter().all(|g| !g.3), "{name}: {got:?}");
        assert!(faces(&b.ink).iter().all(|g| g.1 == Face::Math), "{name}");
        for ch in text.chars() {
            assert!(math_metrics::get(ch).is_some(), "{ch:?}");
        }
        assert!(approx_eq(b.width, adv(text), EPS), "{name}");
    }
}

#[test]
fn tex_layout_liminf_has_a_thin_space_between_the_words() {
    for (name, second) in [("liminf", "inf"), ("limsup", "sup")] {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: the second word starts one thin space after "lim".
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), 7, "{name}: {got:?}");
        let gap = got[4].1.x - adv("lim");
        assert!(approx_eq(gap, OP_SPACE, 1e-3), "{name}: {gap}");
        assert!(
            approx_eq(b.width, adv("lim") + gap + adv(second), EPS),
            "{name}"
        );
    }
}

#[test]
fn tex_layout_named_operators_place_scripts_by_class() {
    for (name, text) in NAMED_LIMITS {
        // Act
        let b = boxed(&format!("\\{name}_k"));

        // Assert: `k` centred below the word, like `\min_k`.
        let word = boxed(&format!("\\{name}"));
        let (k, ks) = glyph(&b.ink, K);
        assert!(
            approx_eq(k.x + mm(K).advance * ks / 2.0, adv(text) / 2.0, EPS),
            "{name}: {k:?}"
        );
        assert!(k.y > word.descent, "{name}: {k:?}");
        // The thin operator space before an ordinary atom.
        let gap = boxed(&format!("\\{name} x")).width - adv(text) - mm(X).advance;
        assert!(approx_eq(gap, OP_SPACE, EPS), "{name}: {gap}");
    }
    for name in NAMED_OPS {
        // Act
        let b = boxed(&format!("\\{name}^2"));

        // Assert: the exponent beside the word, raised.
        let word = boxed(&format!("\\{name}"));
        let (two, _) = glyph(&b.ink, '2');
        assert!(approx_eq(two.x, word.width + word.italic, EPS), "{name}");
        assert!(two.y < 0.0, "{name}: {two:?}");
        let gap = boxed(&format!("\\{name} x")).width - adv(name) - mm(X).advance;
        assert!(approx_eq(gap, OP_SPACE, EPS), "{name}: {gap}");
    }
}

#[test]
fn tex_parse_operatorname_keeps_its_raw_body_and_star() {
    // Act / Assert
    assert_eq!(
        math("\\operatorname{rank}"),
        vec![opname(false, "", "rank")]
    );
    assert_eq!(
        math("\\operatorname*{argmin}"),
        vec![opname(true, "", "argmin")]
    );
    assert_eq!(
        math("\\operatorname {a {b} c}"),
        vec![opname(false, " ", "a {b} c")]
    );
    assert_eq!(
        math("\\operatorname*{argmin}_x"),
        vec![Node::Scripts {
            base: Some(Box::new(opname(true, "", "argmin"))),
            first: script(ScriptKind::Sub, tok(Node::Char('x'))),
            second: None,
        }]
    );
    for src in [
        "$\\operatorname{rank}$",
        "$\\operatorname* {argmin}_x f$",
        "$\\operatorname {a {b} c}^2$",
    ] {
        assert_eq!(print(&parse(src)), src);
    }
}

#[test]
fn tex_parse_operatorname_without_body_is_literal() {
    // Act / Assert
    assert_eq!(
        math("\\operatorname x"),
        vec![
            Node::Literal("\\operatorname".to_owned()),
            Node::Space(" ".to_owned()),
            Node::Char('x'),
        ]
    );
    assert_eq!(
        math("\\operatorname*x"),
        vec![Node::Literal("\\operatorname*".to_owned()), Node::Char('x')]
    );
    assert_eq!(
        math("\\operatorname"),
        vec![Node::Literal("\\operatorname".to_owned())]
    );
    assert_eq!(
        math("\\operatorname{x"),
        vec![
            Node::Literal("\\operatorname".to_owned()),
            Node::Literal("{".to_owned()),
            Node::Char('x'),
        ]
    );
    // The star only counts directly after the name.
    assert_eq!(
        math("\\operatorname *{x}"),
        vec![
            Node::Literal("\\operatorname".to_owned()),
            Node::Space(" ".to_owned()),
            Node::Char('*'),
            Node::Group(vec![Node::Char('x')]),
        ]
    );
    // Drawn in the warning colour.
    let b = boxed("\\operatorname*");
    let got = glyphs(&b.ink);
    let chars: String = got.iter().map(|g| g.0).collect();
    assert_eq!(chars, "\\operatorname*");
    assert!(got.iter().all(|g| g.3), "{got:?}");
}

#[test]
fn tex_layout_operatorname_matches_the_named_operators() {
    // Act / Assert: the same boxes as the table rows.
    assert_eq!(boxed("\\operatorname{log} x"), boxed("\\log x"));
    assert_eq!(boxed("\\operatorname*{max}_k"), boxed("\\max_k"));
    assert_eq!(boxed("\\operatorname{exp}^2"), boxed("\\exp^2"));

    // An upright word of math-face letters, thin space before `x`.
    let b = boxed("\\operatorname{rank}x");
    let got = glyphs(&b.ink);
    let chars: String = got.iter().map(|g| g.0).collect();
    assert_eq!(chars, format!("rank{X}"));
    assert!(got.iter().all(|g| !g.3), "{got:?}");
    assert!(faces(&b.ink).iter().all(|g| g.1 == Face::Math));
    let (x, _) = glyph(&b.ink, X);
    assert!(approx_eq(x.x, adv("rank") + OP_SPACE, EPS), "{x:?}");

    // The starred form centres its limit below the word.
    let m = boxed("\\operatorname*{argmin}_x");
    let (x, xs) = glyph(&m.ink, X);
    assert!(
        approx_eq(x.x + mm(X).advance * xs / 2.0, adv("argmin") / 2.0, EPS),
        "{x:?}"
    );
    assert!(x.y > 0.0, "{x:?}");
}

// ---- T54 math alphabets (ADR-T54-1) -----------------------------------

/// The alphabet commands of T54, `\mathcal` included.
const ALPHABET_NAMES: [&str; 4] = ["mathcal", "mathrm", "mathit", "mathnormal"];

/// One character of every kind an alphabet maps: Latin, digits, Greek
/// capitals (without the unassigned U+03A2) and lowercase Greek with
/// its variants.
pub(super) fn every_kind() -> String {
    let greek_upper = ('Α'..='Ω').filter(|&c| c != '\u{3A2}');
    ('A'..='Z')
        .chain('a'..='z')
        .chain('0'..='9')
        .chain(greek_upper)
        .chain('α'..='ω')
        .chain("ϵϑϰϕϱϖ".chars())
        .collect()
}

/// The characters of every glyph of `src`, in order.
pub(super) fn drawn(src: &str) -> Vec<char> {
    glyphs(&boxed(src).ink).iter().map(|g| g.0).collect()
}

#[test]
fn tex_parse_alphabets_take_one_argument() {
    for name in ALPHABET_NAMES {
        // Act / Assert: braced or one token.
        assert_eq!(
            math(&format!("\\{name}{{d}}")),
            vec![Node::Command {
                name: name.to_owned(),
                args: vec![braced(vec![Node::Char('d')])],
            }],
            "{name}"
        );
        assert_eq!(
            math(&format!("\\{name}\\Gamma")),
            vec![Node::Command {
                name: name.to_owned(),
                args: vec![tok(sym("Gamma"))],
            }],
            "{name}"
        );
        for src in [
            format!("$\\{name}{{diff}}^2$"),
            format!("$\\{name} x$"),
            format!("$\\{name}{{\\mathrm{{A}}b}}$"),
        ] {
            assert_eq!(print(&parse(&src)), src);
        }
    }
}

#[test]
fn tex_parse_alphabet_without_argument_is_literal() {
    for name in ALPHABET_NAMES {
        // Act / Assert
        let literal = Node::Literal(format!("\\{name}"));
        assert_eq!(math(&format!("\\{name}")), vec![literal.clone()], "{name}");
        assert_eq!(
            math(&format!("\\{name}}}")),
            vec![literal, Node::Literal("}".to_owned())],
            "{name}"
        );
        // Drawn in the warning colour.
        let got = glyphs(&boxed(&format!("\\{name}")).ink);
        assert!(got.iter().all(|g| g.3), "{name}: {got:?}");
    }
}

#[test]
fn tex_alphabet_rows_draw_glyphs_of_the_math_face() {
    // Arrange
    let all = every_kind();
    for name in ALPHABET_NAMES {
        assert_eq!(alphabet(name).map(|a| a.name), Some(name));
    }
    assert!(alphabet("mathbbb").is_none());

    for src in std::iter::once(all.clone()).chain(
        ALPHABETS
            .iter()
            .map(|row| format!("\\{}{{{all}}}", row.name)),
    ) {
        // Act
        let b = boxed(&src);

        // Assert: one math-face glyph per character, none warned.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), all.chars().count(), "{src}");
        assert!(got.iter().all(|g| !g.3), "{src}");
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{src}: {ch:?}");
            assert!(math_metrics::get(ch).is_some(), "{src}: {ch:?}");
        }
    }
}

#[test]
fn tex_layout_mathrm_letters_are_upright_without_italic_correction() {
    // Act
    let d = boxed("\\mathrm{d}");
    let dx = boxed("\\mathrm{d}x");
    let pow = boxed("\\mathrm{d}^2");

    // Assert: the upright d, no italic correction, nothing between
    // it and the next atom or its exponent.
    assert_eq!(faces(&d.ink), vec![('d', Face::Math)]);
    assert!(approx_eq(d.italic, 0.0, EPS), "{}", d.italic);
    assert!(approx_eq(d.width, adv("d"), EPS));
    let (x, _) = glyph(&dx.ink, X);
    assert!(approx_eq(x.x, adv("d"), EPS), "{x:?}");
    let (two, _) = glyph(&pow.ink, '2');
    assert!(approx_eq(two.x, adv("d"), EPS), "{two:?}");
    assert_eq!(drawn("\\mathrm{Ab1}"), vec!['A', 'b', '1']);
    // Digits and Greek as outside `\mathrm`.
    assert_eq!(drawn("\\mathrm{2\\alpha\\Gamma}"), drawn("2\\alpha\\Gamma"));
    assert_eq!(drawn("\\mathrm{αΔ}"), drawn("αΔ"));
}

#[test]
fn tex_layout_mathit_sets_a_word_without_gaps() {
    // Arrange
    let word = [italic('d'), I, F, F];

    // Act
    let b = boxed("\\mathit{diff}");

    // Assert: italic letters side by side, as wide as their advances.
    let got = glyphs(&b.ink);
    assert_eq!(got.iter().map(|g| g.0).collect::<Vec<_>>(), word);
    let mut at = 0.0;
    for (ch, pos, _, _) in got {
        assert!(pos.approx_eq(v(at, 0.0), EPS), "{ch:?}: {pos:?}");
        at += mm(ch).advance;
    }
    assert!(approx_eq(b.width, at, EPS));
    // One ordinary atom, spaced like a group.
    assert_eq!(boxed("\\mathit{diff}=x"), boxed("{diff}=x"));
    // Digits stay upright.
    assert_eq!(drawn("\\mathit{x2}"), vec![X, '2']);
}

#[test]
fn tex_mathnormal_gives_italic_greek_capitals() {
    // Act / Assert
    assert_eq!(
        drawn("\\mathnormal{\\Gamma\\Omega}"),
        vec!['\u{1D6E4}', '\u{1D6FA}']
    );
    assert_eq!(boxed("\\mathnormal{x}"), boxed("x"));
    assert_eq!(drawn("\\mathnormal{a1\\alpha}"), drawn("a1\\alpha"));
    let capitals = SYMBOLS.iter().filter(|s| {
        let mut chars = s.text.chars();
        matches!((chars.next(), chars.next()), (Some('Α'..='Ω'), None))
    });
    let mut count = 0;
    for def in capitals {
        let ch = def.text.chars().next().map_or(0, u32::from);
        let want = char::from_u32(0x1D6E2 + ch - 0x391);
        assert_eq!(
            drawn(&format!("\\mathnormal{{\\{}}}", def.name))
                .first()
                .copied(),
            want,
            "{}",
            def.name
        );
        count += 1;
    }
    assert!(count >= 11, "{count}");
}

#[test]
fn tex_alphabets_nest_innermost_first() {
    // Act / Assert
    assert_eq!(drawn("\\mathrm{\\mathcal{A}}"), vec!['\u{1D49C}']);
    assert_eq!(drawn("\\mathcal{\\mathrm{A}}"), vec!['A']);
    assert_eq!(drawn("\\mathcal{\\mathrm{a}}"), vec!['a']);
    assert_eq!(drawn("\\mathrm{\\mathcal{a}}"), vec!['a']);
    assert_eq!(drawn("\\mathrm{\\mathcal{B}x}"), vec!['\u{212C}', 'x']);
    assert_eq!(drawn("\\mathcal{a}"), vec![A]);
    assert_eq!(drawn("\\mathnormal{\\mathrm{\\Gamma}}"), vec!['\u{1D6E4}']);
    assert_eq!(drawn("\\mathrm{\\mathit{x}}"), vec![X]);
    assert_eq!(drawn("\\mathit{\\mathrm{x}}"), vec!['x']);
    // Scripts and fractions inside an alphabet keep it.
    // (The superscript's ink comes before the subscript's.)
    assert_eq!(drawn("\\mathrm{d_x^y}"), vec!['d', 'y', 'x']);
    assert_eq!(drawn("\\mathrm{\\frac{a}{b}}"), vec!['a', 'b']);
}

#[test]
fn tex_alphabets_leave_operators_and_text_alone() {
    for name in ALPHABET_NAMES {
        for src in [
            "\\log+\\sum\\text{ab}",
            "\\operatorname{rank}",
            "\\sin\\gcd\\to\\infty",
            "\\aleph\\partial\\nabla",
        ] {
            // Act / Assert
            assert_eq!(
                boxed(&format!("\\{name}{{{src}}}")),
                boxed(src),
                "\\{name}{{{src}}}"
            );
        }
    }
}

#[test]
fn tex_layout_accent_skew_follows_the_alphabet() {
    // Arrange
    let Some(hat) = accent("hat") else {
        panic!("no \\hat");
    };
    let mark_centre = |src: &str| ink_box(&mark_ink(&boxed(src).ink, hat)).center().x;

    // Act / Assert: an upright letter is centred on.
    let plain = boxed("\\mathrm{x}");
    let centre = ink_box(&plain.ink).center().x;
    let got = mark_centre("\\mathrm{\\hat{x}}");
    assert!(approx_eq(got, centre, EPS), "{got} vs {centre}");

    // An italic capital Greek letter is skewed over.
    for src in ["\\mathnormal", "\\mathit"] {
        let plain = boxed(&format!("{src}{{\\Gamma}}"));
        let want = ink_box(&plain.ink).center().x + ACCENT_SKEW * plain.ascent;
        let got = mark_centre(&format!("{src}{{\\hat{{\\Gamma}}}}"));
        assert!(approx_eq(got, want, EPS), "{src}: {got} vs {want}");
    }
}

// ---- T62 complete Greek letters -----------------------------------------

/// `\varGamma` … `\varOmega`, the amsmath italic capitals.
const VAR_CAPITALS: [&str; 11] = [
    "varGamma",
    "varDelta",
    "varTheta",
    "varLambda",
    "varXi",
    "varPi",
    "varSigma",
    "varUpsilon",
    "varPhi",
    "varPsi",
    "varOmega",
];

/// The unicode-math capitals that look like Latin letters, with the
/// letter each draws.
const LATIN_LIKE_CAPITALS: [(&str, char); 13] = [
    ("Alpha", 'Α'),
    ("Beta", 'Β'),
    ("Epsilon", 'Ε'),
    ("Zeta", 'Ζ'),
    ("Eta", 'Η'),
    ("Iota", 'Ι'),
    ("Kappa", 'Κ'),
    ("Mu", 'Μ'),
    ("Nu", 'Ν'),
    ("Omicron", 'Ο'),
    ("Rho", 'Ρ'),
    ("Tau", 'Τ'),
    ("Chi", 'Χ'),
];

/// Whether `ch` is a Greek letter or a math alphanumeric Greek one.
fn is_greek(ch: char) -> bool {
    matches!(ch, '\u{370}'..='\u{3FF}' | '\u{1D6A8}'..='\u{1D7CB}')
}

#[test]
fn tex_greek_every_letter_has_a_command() {
    // Arrange
    let letters = ('Α'..='ω')
        .filter(|&c| c != '\u{3A2}' && !('\u{3AA}'..='\u{3B0}').contains(&c))
        .chain("ϵϑϰϕϱϖ".chars());
    let names = VAR_CAPITALS
        .iter()
        .copied()
        .chain(LATIN_LIKE_CAPITALS.iter().map(|&(name, _)| name))
        .chain(std::iter::once("varkappa"));

    // Act / Assert
    for ch in letters {
        assert!(
            SYMBOLS.iter().any(|s| s.text == ch.to_string()),
            "no command for {ch:?} U+{:04X}",
            u32::from(ch)
        );
    }
    for name in names {
        assert_eq!(math(&format!("\\{name}")), vec![sym(name)], "{name}");
        assert_eq!(symbol(name).map(|s| s.class), Some(Class::Ord), "{name}");
    }
}

#[test]
fn tex_greek_commands_draw_glyphs_of_the_math_face() {
    // Arrange
    let greek: Vec<&SymbolDef> = SYMBOLS
        .iter()
        .filter(|s| s.text.chars().all(is_greek))
        .collect();
    assert!(greek.len() >= 30 + 11 + 11 + 13, "{}", greek.len());

    for def in greek {
        // Act
        let b = boxed(&format!("\\{}", def.name));

        // Assert: one unwarned math-face glyph with metrics.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), 1, "{}", def.name);
        assert!(!got[0].3, "{}", def.name);
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{}: {ch:?}", def.name);
            assert!(math_metrics::get(ch).is_some(), "{}: {ch:?}", def.name);
        }
    }
    assert_eq!(drawn("\\varkappa"), vec!['\u{1D718}']);
}

#[test]
fn tex_var_greek_capitals_are_italic() {
    for name in VAR_CAPITALS {
        // Arrange
        let upright = name.trim_start_matches("var");

        // Act
        let got = drawn(&format!("\\{name}"));

        // Assert
        assert_eq!(
            got,
            drawn(&format!("\\mathnormal{{\\{upright}}}")),
            "{name}"
        );
        assert_ne!(got, drawn(&format!("\\{upright}")), "{name}");
        assert_eq!(drawn(&format!("\\mathrm{{\\{name}}}")), got, "{name}");
    }
    assert_eq!(drawn("\\varGamma"), vec!['\u{1D6E4}']);
    assert_eq!(drawn("\\varOmega"), vec!['\u{1D6FA}']);
}

#[test]
fn tex_latin_like_greek_capitals_are_upright() {
    for (name, ch) in LATIN_LIKE_CAPITALS {
        // Act / Assert
        assert_eq!(
            faces(&boxed(&format!("\\{name}")).ink),
            vec![(ch, Face::Math)]
        );
    }
    assert_eq!(drawn("\\mathnormal{\\Alpha}"), vec!['\u{1D6E2}']);
    assert_eq!(drawn("\\mathit{\\Chi}"), vec!['\u{1D6F8}']);
}

#[test]
fn tex_digamma_is_not_a_command() {
    for name in ["digamma", "Digamma"] {
        // Arrange
        assert!(symbol(name).is_none(), "{name}");

        // Act
        let got = glyphs(&boxed(&format!("\\{name}")).ink);

        // Assert: drawn as typed, in the warning colour.
        assert_eq!(
            got.iter().map(|g| g.0).collect::<String>(),
            format!("\\{name}")
        );
        assert!(got.iter().all(|g| g.3), "{name}: {got:?}");
    }
}

#[test]
fn tex_literals_and_plain_text_use_the_text_face() {
    // Act
    let l = layout(&parse("a$x\\foo$"));

    // Assert
    let got = faces(&l.ink);
    assert_eq!(got[0], ('a', Face::Text));
    assert_eq!(got[1], (X, Face::Math));
    assert!(got[2..].iter().all(|g| g.1 == Face::Text), "{got:?}");
}

#[test]
fn tex_char_without_math_glyph_falls_back_to_a_text_cell() {
    // Arrange
    assert!(math_metrics::get('日').is_none());

    // Act
    let typed = boxed("日");
    let text = boxed("\\text{日}");

    // Assert: one 0.6 em cell of the text font.
    for b in [typed, text] {
        assert_eq!(faces(&b.ink), vec![('日', Face::Text)]);
        assert!(approx_eq(b.width, ADVANCE, EPS));
        assert!(approx_eq(b.ascent, GLYPH_ASCENT, EPS));
    }
}
