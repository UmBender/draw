//! Tests of the TeX subset.

use super::basics::{drawn, every_kind, ink_box, mark_ink};
use super::*;

// ---- T56 bold math (ADR-T56-1) ----------------------------------------

/// The bold alphabet commands of T56.
const BOLD_NAMES: [&str; 3] = ["mathbf", "boldsymbol", "bm"];

/// The character `index` places after `start`.
fn nth(start: u32, index: u32) -> char {
    char::from_u32(start + index).unwrap_or('\u{FFFD}')
}

/// Where `\mathbf` draws `ch`, or `None` when it inherits its kind.
fn mathbf_char(ch: char) -> Option<char> {
    let (kind, index) = Kind::of(ch)?;
    let start = match kind {
        Kind::Upper => 0x1D400,
        Kind::Lower => 0x1D41A,
        Kind::Digit => 0x1D7CE,
        Kind::GreekUpper => 0x1D6A8,
        Kind::GreekLower => return None,
    };
    Some(nth(start, index))
}

/// Where `\boldsymbol` draws `ch`.
fn boldsymbol_char(ch: char) -> Option<char> {
    let (kind, index) = Kind::of(ch)?;
    let start = match kind {
        Kind::Upper => 0x1D468,
        Kind::Lower => 0x1D482,
        Kind::Digit => 0x1D7CE,
        Kind::GreekUpper => 0x1D71C,
        Kind::GreekLower => 0x1D736,
    };
    Some(nth(start, index))
}

#[test]
fn tex_mathbf_maps_to_upright_bold() {
    // Arrange
    let all = every_kind();
    let want: Vec<char> = all
        .chars()
        .map(|ch| mathbf_char(ch).unwrap_or_else(|| italic(ch)))
        .collect();

    // Act
    let got = drawn(&format!("\\mathbf{{{all}}}"));

    // Assert
    assert_eq!(got, want);
    assert_eq!(drawn("\\mathbf{Av0}"), vec!['𝐀', '𝐯', '𝟎']);
    assert_eq!(drawn("\\mathbf\\Gamma"), vec!['𝚪']);
    assert_eq!(drawn("\\mathbf{\\Sigma\\Omega}"), vec!['𝚺', '𝛀']);
    // Lowercase Greek stays math italic, as in LaTeX.
    assert_eq!(drawn("\\mathbf{\\alpha\\varphi}"), drawn("\\alpha\\varphi"));
}

#[test]
fn tex_boldsymbol_and_bm_map_to_bold_italic() {
    // Arrange
    let all = every_kind();
    let want: Vec<char> = all.chars().filter_map(boldsymbol_char).collect();
    assert_eq!(want.len(), all.chars().count());

    for name in ["boldsymbol", "bm"] {
        // Act
        let got = drawn(&format!("\\{name}{{{all}}}"));

        // Assert
        assert_eq!(got, want, "{name}");
        assert_eq!(
            drawn(&format!("\\{name}{{xh1}}")),
            vec!['𝒙', '𝒉', '𝟏'],
            "{name}: bold italic h has no hole"
        );
        assert_eq!(
            drawn(&format!("\\{name}{{\\alpha\\Gamma\\varepsilon\\epsilon}}")),
            vec!['𝜶', '𝜞', '𝜺', '𝝐'],
            "{name}"
        );
    }
    assert_eq!(alphabet("bm").map(|a| (a.upper, a.lower, a.digit)), {
        alphabet("boldsymbol").map(|a| (a.upper, a.lower, a.digit))
    });
    assert_eq!(boxed("\\bm{v_1}"), boxed("\\boldsymbol{v_1}"));
}

#[test]
fn tex_parse_bold_commands_take_one_argument() {
    for name in BOLD_NAMES {
        // Act / Assert: braced or one token.
        assert_eq!(
            math(&format!("\\{name}{{v}}")),
            vec![Node::Command {
                name: name.to_owned(),
                args: vec![braced(vec![Node::Char('v')])],
            }],
            "{name}"
        );
        assert_eq!(
            math(&format!("\\{name}\\alpha")),
            vec![Node::Command {
                name: name.to_owned(),
                args: vec![tok(sym("alpha"))],
            }],
            "{name}"
        );
        for src in [
            format!("$\\{name}{{v}} \\cdot \\bm{{\\alpha}}$"),
            format!("$\\{name} x^2$"),
        ] {
            assert_eq!(print(&parse(&src)), src);
        }
        // Without an argument: a literal in the warning colour.
        let literal = Node::Literal(format!("\\{name}"));
        assert_eq!(math(&format!("\\{name}")), vec![literal], "{name}");
        let got = glyphs(&boxed(&format!("\\{name}")).ink);
        assert!(got.iter().all(|g| g.3), "{name}: {got:?}");
    }
}

#[test]
fn tex_bold_rows_draw_glyphs_of_the_math_face() {
    // Arrange
    let all = every_kind();
    for name in BOLD_NAMES {
        assert_eq!(alphabet(name).map(|a| a.name), Some(name));

        // Act
        let b = boxed(&format!("\\{name}{{{all}}}"));

        // Assert: one math-face glyph per character, none warned.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), all.chars().count(), "{name}");
        assert!(got.iter().all(|g| !g.3), "{name}");
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{name}: {ch:?}");
            assert!(math_metrics::get(ch).is_some(), "{name}: {ch:?}");
        }
    }
}

#[test]
fn tex_layout_accent_skew_over_bold_letters() {
    // Arrange
    let Some(hat) = accent("hat") else {
        panic!("no \\hat");
    };
    let mark_centre = |src: &str| ink_box(&mark_ink(&boxed(src).ink, hat)).center().x;
    let check = |alphabet: &str, arg: &str, skewed: bool| {
        let plain = boxed(&format!("\\{alphabet}{{{arg}}}"));
        let skew = if skewed {
            ACCENT_SKEW * plain.ascent
        } else {
            0.0
        };
        let want = ink_box(&plain.ink).center().x + skew;

        // Act
        let got = mark_centre(&format!("\\{alphabet}{{\\hat{{{arg}}}}}"));

        // Assert
        assert!(
            approx_eq(got, want, EPS),
            "\\{alphabet}{{{arg}}}: {got} vs {want}"
        );
    };

    // Upright bold: centred.
    for arg in ["v", "A", "\\Gamma", "1"] {
        check("mathbf", arg, false);
    }
    // Lowercase Greek inside \mathbf is math italic: skewed.
    check("mathbf", "\\alpha", true);
    // Bold italic: skewed; digits stay centred.
    for name in ["boldsymbol", "bm"] {
        for arg in ["v", "A", "\\Gamma", "\\alpha"] {
            check(name, arg, true);
        }
        check(name, "1", false);
    }
    // Existing alphabets keep their skew.
    check("mathcal", "A", true);
    check("mathit", "v", true);
    check("mathrm", "v", false);
}

#[test]
fn tex_bold_alphabets_nest_and_leave_operators_alone() {
    // Act / Assert: the innermost alphabet that maps a kind wins.
    assert_eq!(drawn("\\mathbf{\\boldsymbol{x}}"), vec!['𝒙']);
    assert_eq!(drawn("\\boldsymbol{\\mathbf{x}}"), vec!['𝐱']);
    assert_eq!(drawn("\\boldsymbol{\\mathrm{x}}"), vec!['x']);
    assert_eq!(drawn("\\boldsymbol{\\mathbf{\\alpha}}"), vec!['𝜶']);
    assert_eq!(drawn("\\mathbf{\\mathcal{A}b}"), vec!['𝒜', '𝐛']);
    assert_eq!(drawn("\\mathbf{v_1^2}"), vec!['𝐯', '𝟐', '𝟏']);
    for name in BOLD_NAMES {
        for src in [
            "\\log+\\sum\\text{ab}",
            "\\operatorname{rank}",
            "\\sin\\gcd\\to\\infty",
            "\\aleph\\partial\\nabla",
        ] {
            assert_eq!(
                boxed(&format!("\\{name}{{{src}}}")),
                boxed(src),
                "\\{name}{{{src}}}"
            );
        }
    }
}

#[test]
fn tex_boldmath_is_an_unknown_command() {
    // Act
    let alone = glyphs(&boxed("\\boldmath").ink);
    let before = drawn("\\boldmath x");

    // Assert: drawn in the warning colour; the x after it is not bold.
    assert!(alone.iter().all(|g| g.3), "{alone:?}");
    assert!(alphabet("boldmath").is_none());
    assert_eq!(before.last().copied(), Some(X));
}

// ---- T57 blackboard bold (ADR-T57-1) ----------------------------------

/// The blackboard bold alphabet commands of T57.
const BLACKBOARD_NAMES: [&str; 2] = ["mathbb", "mathds"];

/// The double-struck capitals Unicode placed in Letterlike Symbols.
const BLACKBOARD_HOLES: [(char, char); 7] = [
    ('C', '\u{2102}'),
    ('H', '\u{210D}'),
    ('N', '\u{2115}'),
    ('P', '\u{2119}'),
    ('Q', '\u{211A}'),
    ('R', '\u{211D}'),
    ('Z', '\u{2124}'),
];

/// Where `\mathbb` draws `ch`, or `None` when it inherits its kind.
fn mathbb_char(ch: char) -> Option<char> {
    if let Some(&(_, drawn)) = BLACKBOARD_HOLES.iter().find(|h| h.0 == ch) {
        return Some(drawn);
    }
    let (kind, index) = Kind::of(ch)?;
    let start = match kind {
        Kind::Upper => 0x1D538,
        Kind::Lower => 0x1D552,
        Kind::Digit => 0x1D7D8,
        Kind::GreekUpper | Kind::GreekLower => return None,
    };
    Some(nth(start, index))
}

#[test]
fn tex_mathbb_maps_to_double_struck() {
    // Arrange
    let all = every_kind();
    let want: Vec<char> = all
        .chars()
        .map(|ch| mathbb_char(ch).unwrap_or_else(|| italic(ch)))
        .collect();

    // Act
    let got = drawn(&format!("\\mathbb{{{all}}}"));

    // Assert
    assert_eq!(got, want);
    assert_eq!(drawn("\\mathbb{Ax0}"), vec!['𝔸', '𝕩', '𝟘']);
    assert_eq!(drawn("\\mathbb{Fk9}"), vec!['𝔽', '𝕜', '𝟡']);
    // Greek is not double-struck: as outside.
    assert_eq!(drawn("\\mathbb{\\alpha\\Gamma}"), drawn("\\alpha\\Gamma"));
}

#[test]
fn tex_mathbb_holes_map_to_letterlike_symbols() {
    for (typed, want) in BLACKBOARD_HOLES {
        // Act
        let got = drawn(&format!("\\mathbb{{{typed}}}"));

        // Assert
        assert_eq!(got, vec![want], "{typed}");
    }
    assert_eq!(drawn("\\mathbb{RNZQC}"), vec!['ℝ', 'ℕ', 'ℤ', 'ℚ', 'ℂ']);
    // Lowercase has no holes.
    assert_eq!(drawn("\\mathbb{rnz}"), vec!['𝕣', '𝕟', '𝕫']);
}

#[test]
fn tex_mathds_is_the_mathbb_row() {
    // Arrange
    let all = every_kind();

    // Act
    let ds = alphabet("mathds");
    let bb = alphabet("mathbb");

    // Assert
    let rows = |a: Option<&AlphabetDef>| {
        a.map(|a| {
            let maps: Vec<Map> = Kind::ALL.iter().map(|&k| a.get(k)).collect();
            (maps, a.holes, a.slant)
        })
    };
    assert!(bb.is_some());
    assert_eq!(rows(ds), rows(bb));
    assert_eq!(drawn("\\mathds{1}"), vec!['𝟙']);
    assert_eq!(drawn("\\mathds{1}_A"), vec!['𝟙', italic('A')]);
    assert_eq!(
        drawn(&format!("\\mathds{{{all}}}")),
        drawn(&format!("\\mathbb{{{all}}}"))
    );
}

#[test]
fn tex_parse_blackboard_commands_take_one_argument() {
    for name in BLACKBOARD_NAMES {
        // Act / Assert: braced or one token.
        assert_eq!(
            math(&format!("\\{name}{{R}}")),
            vec![Node::Command {
                name: name.to_owned(),
                args: vec![braced(vec![Node::Char('R')])],
            }],
            "{name}"
        );
        assert_eq!(
            math(&format!("\\{name} N")),
            vec![Node::Command {
                name: name.to_owned(),
                args: vec![Arg {
                    lead: " ".to_owned(),
                    ..tok(Node::Char('N'))
                }],
            }],
            "{name}"
        );
        for src in [
            format!("$\\{name}{{Z}}_p, \\{name}{{1}}_A$"),
            format!("$x \\in \\{name} R^n$"),
        ] {
            assert_eq!(print(&parse(&src)), src);
        }
        // Without an argument: a literal in the warning colour.
        let literal = Node::Literal(format!("\\{name}"));
        assert_eq!(math(&format!("\\{name}")), vec![literal], "{name}");
        let got = glyphs(&boxed(&format!("\\{name}")).ink);
        assert!(got.iter().all(|g| g.3), "{name}: {got:?}");
    }
}

#[test]
fn tex_blackboard_rows_draw_glyphs_of_the_math_face() {
    // Arrange
    let all = every_kind();
    for name in BLACKBOARD_NAMES {
        assert_eq!(alphabet(name).map(|a| a.name), Some(name));

        // Act
        let b = boxed(&format!("\\{name}{{{all}}}"));

        // Assert: one math-face glyph per character, none warned.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), all.chars().count(), "{name}");
        assert!(got.iter().all(|g| !g.3), "{name}");
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{name}: {ch:?}");
            assert!(math_metrics::get(ch).is_some(), "{name}: {ch:?}");
        }
    }
}

#[test]
fn tex_layout_accent_centred_over_blackboard_letters() {
    // Arrange
    let Some(hat) = accent("hat") else {
        panic!("no \\hat");
    };
    for name in BLACKBOARD_NAMES {
        for arg in ["R", "A", "x", "1"] {
            let plain = boxed(&format!("\\{name}{{{arg}}}"));
            let want = ink_box(&plain.ink).center().x;

            // Act
            let hatted = boxed(&format!("\\{name}{{\\hat{{{arg}}}}}"));
            let got = ink_box(&mark_ink(&hatted.ink, hat)).center().x;

            // Assert
            assert!(
                approx_eq(got, want, EPS),
                "\\{name}{{{arg}}}: {got} vs {want}"
            );
        }
    }
}

#[test]
fn tex_blackboard_nests_and_leaves_operators_alone() {
    // Act / Assert: the innermost alphabet that maps a kind wins.
    assert_eq!(drawn("\\mathbb{\\mathrm{x}}"), vec!['x']);
    assert_eq!(drawn("\\mathbf{\\mathbb{R}}"), vec!['ℝ']);
    assert_eq!(drawn("\\mathbb{\\mathbf{R}}"), vec!['𝐑']);
    assert_eq!(drawn("\\mathbb{\\mathcal{A}B}"), vec!['𝒜', '𝔹']);
    assert_eq!(drawn("\\mathbb{Z}_p"), vec!['ℤ', italic('p')]);
    assert_eq!(drawn("\\mathbb{F_q}"), vec!['𝔽', '𝕢']);
    for name in BLACKBOARD_NAMES {
        for src in [
            "\\log+\\sum\\text{ab}",
            "\\operatorname{rank}",
            "\\aleph\\partial\\nabla",
        ] {
            assert_eq!(
                boxed(&format!("\\{name}{{{src}}}")),
                boxed(src),
                "\\{name}{{{src}}}"
            );
        }
    }
}

#[test]
fn tex_blackboard_shortcuts_are_unknown() {
    for name in ["R", "N", "Z", "Q", "C"] {
        // Act
        let got = glyphs(&boxed(&format!("\\{name}")).ink);

        // Assert: drawn in the warning colour, no alphabet row.
        assert!(got.iter().all(|g| g.3), "\\{name}: {got:?}");
        assert!(alphabet(name).is_none(), "{name}");
    }
}

// ---- T58 fraktur ------------------------------------------------------

/// The fraktur capitals Unicode placed in Letterlike Symbols.
const FRAKTUR_HOLES: [(char, char); 5] = [
    ('C', '\u{212D}'),
    ('H', '\u{210C}'),
    ('I', '\u{2111}'),
    ('R', '\u{211C}'),
    ('Z', '\u{2128}'),
];

/// Where `\mathfrak` draws `ch`, or `None` when it inherits its kind.
fn mathfrak_char(ch: char) -> Option<char> {
    if let Some(&(_, drawn)) = FRAKTUR_HOLES.iter().find(|h| h.0 == ch) {
        return Some(drawn);
    }
    let (kind, index) = Kind::of(ch)?;
    let start = match kind {
        Kind::Upper => 0x1D504,
        Kind::Lower => 0x1D51E,
        Kind::Digit | Kind::GreekUpper | Kind::GreekLower => return None,
    };
    Some(nth(start, index))
}

#[test]
fn tex_mathfrak_maps_to_fraktur() {
    // Arrange
    let all = every_kind();
    let want: Vec<char> = all
        .chars()
        .map(|ch| mathfrak_char(ch).unwrap_or_else(|| italic(ch)))
        .collect();

    // Act
    let got = drawn(&format!("\\mathfrak{{{all}}}"));

    // Assert
    assert_eq!(got, want);
    assert_eq!(drawn("\\mathfrak{gpS}"), vec!['𝔤', '𝔭', '𝔖']);
    assert_eq!(drawn("\\mathfrak{Aaz}"), vec!['𝔄', '𝔞', '𝔷']);
    // Digits and Greek are not fraktur: as outside.
    assert_eq!(
        drawn("\\mathfrak{1\\alpha\\Gamma}"),
        drawn("1\\alpha\\Gamma")
    );
}

#[test]
fn tex_mathfrak_holes_map_to_letterlike_symbols() {
    for (typed, want) in FRAKTUR_HOLES {
        // Act
        let got = drawn(&format!("\\mathfrak{{{typed}}}"));

        // Assert
        assert_eq!(got, vec![want], "{typed}");
    }
    assert_eq!(drawn("\\mathfrak{CHIRZ}"), vec!['ℭ', 'ℌ', 'ℑ', 'ℜ', 'ℨ']);
    // The unassigned block positions are never drawn.
    let got = drawn(&format!("\\mathfrak{{{}}}", every_kind()));
    for (typed, _) in FRAKTUR_HOLES {
        let gap = nth(0x1D504, Kind::of(typed).map_or(0, |k| k.1));
        assert!(!got.contains(&gap), "{typed}: {gap:?}");
    }
    // Lowercase has no holes.
    assert_eq!(drawn("\\mathfrak{chirz}"), vec!['𝔠', '𝔥', '𝔦', '𝔯', '𝔷']);
}

#[test]
fn tex_parse_mathfrak_takes_one_argument() {
    // Act / Assert: braced or one token.
    assert_eq!(
        math("\\mathfrak{g}"),
        vec![Node::Command {
            name: "mathfrak".to_owned(),
            args: vec![braced(vec![Node::Char('g')])],
        }]
    );
    assert_eq!(
        math("\\mathfrak p"),
        vec![Node::Command {
            name: "mathfrak".to_owned(),
            args: vec![Arg {
                lead: " ".to_owned(),
                ..tok(Node::Char('p'))
            }],
        }]
    );
    for src in [
        "$\\mathfrak{g}, \\mathfrak{S}_n$",
        "$\\mathfrak p \\subset R$",
    ] {
        assert_eq!(print(&parse(src)), src);
    }
    // Without an argument: a literal in the warning colour.
    let literal = Node::Literal("\\mathfrak".to_owned());
    assert_eq!(math("\\mathfrak"), vec![literal]);
    let got = glyphs(&boxed("\\mathfrak").ink);
    assert!(got.iter().all(|g| g.3), "{got:?}");
}

#[test]
fn tex_mathfrak_draws_glyphs_of_the_math_face() {
    // Arrange
    let all = every_kind();
    assert_eq!(alphabet("mathfrak").map(|a| a.name), Some("mathfrak"));

    // Act
    let b = boxed(&format!("\\mathfrak{{{all}}}"));

    // Assert: one math-face glyph per character, none warned.
    let got = glyphs(&b.ink);
    assert_eq!(got.len(), all.chars().count());
    assert!(got.iter().all(|g| !g.3));
    for (ch, face) in faces(&b.ink) {
        assert_eq!(face, Face::Math, "{ch:?}");
        assert!(math_metrics::get(ch).is_some(), "{ch:?}");
    }
}

#[test]
fn tex_layout_accent_centred_over_fraktur_letters() {
    // Arrange
    let Some(hat) = accent("hat") else {
        panic!("no \\hat");
    };
    for arg in ["g", "S", "R", "p"] {
        let plain = boxed(&format!("\\mathfrak{{{arg}}}"));
        let want = ink_box(&plain.ink).center().x;

        // Act
        let hatted = boxed(&format!("\\mathfrak{{\\hat{{{arg}}}}}"));
        let got = ink_box(&mark_ink(&hatted.ink, hat)).center().x;

        // Assert
        assert!(approx_eq(got, want, EPS), "{arg}: {got} vs {want}");
    }
}

#[test]
fn tex_mathfrak_nests_and_leaves_operators_alone() {
    // Act / Assert: the innermost alphabet that maps a kind wins.
    assert_eq!(drawn("\\mathfrak{\\mathrm{x}}"), vec!['x']);
    assert_eq!(drawn("\\mathbf{\\mathfrak{g}}"), vec!['𝔤']);
    assert_eq!(drawn("\\mathfrak{\\mathbf{g}}"), vec!['𝐠']);
    assert_eq!(drawn("\\mathfrak{\\mathbb{R}S}"), vec!['ℝ', '𝔖']);
    assert_eq!(drawn("\\mathfrak{S}_n"), vec!['𝔖', italic('n')]);
    assert_eq!(drawn("\\mathfrak{g_1}"), vec!['𝔤', '1']);
    // \Re and \Im are unchanged symbols.
    assert_eq!(drawn("\\Re\\Im"), vec!['ℜ', 'ℑ']);
    assert_eq!(drawn("\\mathfrak{R}\\Re"), vec!['ℜ', 'ℜ']);
    for src in [
        "\\log+\\sum\\text{ab}",
        "\\operatorname{rank}",
        "\\aleph\\partial\\nabla",
    ] {
        assert_eq!(
            boxed(&format!("\\mathfrak{{{src}}}")),
            boxed(src),
            "\\mathfrak{{{src}}}"
        );
    }
}

// ---- T59 script letters (ADR-T59-1) -----------------------------------

#[test]
fn tex_mathscr_maps_like_mathcal() {
    // Arrange
    let all = every_kind();

    // Act
    let got = drawn(&format!("\\mathscr{{{all}}}"));

    // Assert: every kind as \mathcal draws it.
    assert_eq!(got, drawn(&format!("\\mathcal{{{all}}}")));
    assert_eq!(drawn("\\mathscr{LF}"), vec!['ℒ', 'ℱ']);
    assert_eq!(drawn("\\mathscr{AZ}"), vec!['𝒜', '𝒵']);
    assert_eq!(
        drawn("\\mathscr{BEFHILMR}"),
        vec!['ℬ', 'ℰ', 'ℱ', 'ℋ', 'ℐ', 'ℒ', 'ℳ', 'ℛ']
    );
    // Digits and Greek are not script: as outside.
    assert_eq!(
        drawn("\\mathscr{1\\alpha\\Gamma}"),
        drawn("1\\alpha\\Gamma")
    );
}

#[test]
fn tex_mathscr_lowercase_falls_back_to_the_outside_style() {
    // Arrange: the font has no script lowercase (ADR-T59-1).
    let lower = (0x1D4B6..=0x1D4CF).filter_map(char::from_u32);
    for ch in lower.chain(['\u{212F}', '\u{210A}', '\u{2134}']) {
        assert!(math_metrics::get(ch).is_none(), "{ch:?}");
    }

    // Act
    let b = boxed("\\mathscr{l}");

    // Assert: the math italic l of the math face, not warned.
    assert_eq!(drawn("\\mathscr{l}"), vec![italic('l')]);
    assert_eq!(drawn("\\mathscr{l}"), drawn("\\mathcal{l}"));
    assert_eq!(drawn("\\mathrm{\\mathscr{l}}"), vec!['l']);
    assert!(glyphs(&b.ink).iter().all(|g| !g.3));
    assert_eq!(faces(&b.ink), vec![(italic('l'), Face::Math)]);
}

#[test]
fn tex_parse_mathscr_takes_one_argument() {
    // Act / Assert: braced or one token.
    assert_eq!(
        math("\\mathscr{L}"),
        vec![Node::Command {
            name: "mathscr".to_owned(),
            args: vec![braced(vec![Node::Char('L')])],
        }]
    );
    assert_eq!(
        math("\\mathscr F"),
        vec![Node::Command {
            name: "mathscr".to_owned(),
            args: vec![Arg {
                lead: " ".to_owned(),
                ..tok(Node::Char('F'))
            }],
        }]
    );
    for src in [
        "$\\mathscr{L}, \\mathscr{l}$",
        "$\\mathscr F(\\mathscr{H})$",
    ] {
        assert_eq!(print(&parse(src)), src);
    }
    // Without an argument: a literal in the warning colour.
    let literal = Node::Literal("\\mathscr".to_owned());
    assert_eq!(math("\\mathscr"), vec![literal]);
    let got = glyphs(&boxed("\\mathscr").ink);
    assert!(got.iter().all(|g| g.3), "{got:?}");
}

#[test]
fn tex_mathscr_draws_glyphs_of_the_math_face() {
    // Arrange
    let all = every_kind();
    assert_eq!(alphabet("mathscr").map(|a| a.name), Some("mathscr"));

    // Act
    let b = boxed(&format!("\\mathscr{{{all}}}"));

    // Assert: one math-face glyph per character, none warned.
    let got = glyphs(&b.ink);
    assert_eq!(got.len(), all.chars().count());
    assert!(got.iter().all(|g| !g.3));
    for (ch, face) in faces(&b.ink) {
        assert_eq!(face, Face::Math, "{ch:?}");
        assert!(math_metrics::get(ch).is_some(), "{ch:?}");
    }
}

#[test]
fn tex_layout_accent_over_mathscr_as_over_mathcal() {
    // Arrange
    let Some(hat) = accent("hat") else {
        panic!("no \\hat");
    };
    for arg in ["L", "F", "A", "l"] {
        let plain = boxed(&format!("\\mathscr{{{arg}}}"));
        let want = ink_box(&plain.ink).center().x + ACCENT_SKEW * plain.ascent;

        // Act
        let scr = boxed(&format!("\\mathscr{{\\hat{{{arg}}}}}"));
        let cal = boxed(&format!("\\mathcal{{\\hat{{{arg}}}}}"));
        let got = ink_box(&mark_ink(&scr.ink, hat)).center().x;

        // Assert: skewed, and laid out as under \mathcal.
        assert!(approx_eq(got, want, EPS), "{arg}: {got} vs {want}");
        assert_eq!(scr, cal, "{arg}");
    }
}

#[test]
fn tex_mathscr_nests_and_leaves_operators_alone() {
    // Act / Assert: the innermost alphabet that maps a kind wins.
    assert_eq!(drawn("\\mathscr{\\mathrm{L}}"), vec!['L']);
    assert_eq!(drawn("\\mathbf{\\mathscr{L}}"), vec!['ℒ']);
    assert_eq!(drawn("\\mathscr{\\mathbf{L}}"), vec!['𝐋']);
    assert_eq!(drawn("\\mathscr{\\mathcal{L}F}"), vec!['ℒ', 'ℱ']);
    assert_eq!(drawn("\\mathscr{\\mathfrak{g}L}"), vec!['𝔤', 'ℒ']);
    assert_eq!(drawn("\\mathscr{L}_n"), vec!['ℒ', italic('n')]);
    assert_eq!(drawn("\\mathscr{F_1}"), vec!['ℱ', '1']);
    for src in [
        "\\log+\\sum\\text{ab}",
        "\\operatorname{rank}",
        "\\aleph\\partial\\nabla",
    ] {
        assert_eq!(
            boxed(&format!("\\mathscr{{{src}}}")),
            boxed(src),
            "\\mathscr{{{src}}}"
        );
    }
}

// ---- T60 sans-serif and typewriter (ADR-T60-1) ------------------------

/// The sans-serif and typewriter alphabet commands of T60.
const SANS_MONO_NAMES: [&str; 2] = ["mathsf", "mathtt"];

/// Where the Latin-and-digits row starting at `(upper, lower, digit)`
/// draws `ch`, or `None` for Greek.
fn latin_digit_char(ch: char, upper: u32, lower: u32, digit: u32) -> Option<char> {
    let (kind, index) = Kind::of(ch)?;
    let start = match kind {
        Kind::Upper => upper,
        Kind::Lower => lower,
        Kind::Digit => digit,
        Kind::GreekUpper | Kind::GreekLower => return None,
    };
    Some(nth(start, index))
}

#[test]
fn tex_mathsf_maps_to_sans_serif() {
    // Arrange
    let all = every_kind();
    let want: Vec<char> = all
        .chars()
        .map(|ch| latin_digit_char(ch, 0x1D5A0, 0x1D5BA, 0x1D7E2).unwrap_or_else(|| italic(ch)))
        .collect();

    // Act
    let got = drawn(&format!("\\mathsf{{{all}}}"));

    // Assert
    assert_eq!(got, want);
    assert_eq!(drawn("\\mathsf{G}"), vec!['𝖦']);
    assert_eq!(drawn("\\mathsf{x1}"), vec!['𝗑', '𝟣']);
    assert_eq!(drawn("\\mathsf{Az09}"), vec!['𝖠', '𝗓', '𝟢', '𝟫']);
    // Greek is not sans-serif: as outside.
    assert_eq!(drawn("\\mathsf{\\alpha\\Gamma}"), drawn("\\alpha\\Gamma"));
}

#[test]
fn tex_mathtt_maps_to_monospace() {
    // Arrange
    let all = every_kind();
    let want: Vec<char> = all
        .chars()
        .map(|ch| latin_digit_char(ch, 0x1D670, 0x1D68A, 0x1D7F6).unwrap_or_else(|| italic(ch)))
        .collect();

    // Act
    let got = drawn(&format!("\\mathtt{{{all}}}"));

    // Assert
    assert_eq!(got, want);
    assert_eq!(drawn("\\mathtt{x1}"), vec!['𝚡', '𝟷']);
    assert_eq!(drawn("\\mathtt{Az09}"), vec!['𝙰', '𝚣', '𝟶', '𝟿']);
    // Greek is not monospace: as outside.
    assert_eq!(drawn("\\mathtt{\\alpha\\Gamma}"), drawn("\\alpha\\Gamma"));
}

#[test]
fn tex_parse_sans_and_mono_take_one_argument() {
    for name in SANS_MONO_NAMES {
        // Act / Assert: braced or one token.
        assert_eq!(
            math(&format!("\\{name}{{G}}")),
            vec![Node::Command {
                name: name.to_owned(),
                args: vec![braced(vec![Node::Char('G')])],
            }],
            "{name}"
        );
        assert_eq!(
            math(&format!("\\{name} x")),
            vec![Node::Command {
                name: name.to_owned(),
                args: vec![Arg {
                    lead: " ".to_owned(),
                    ..tok(Node::Char('x'))
                }],
            }],
            "{name}"
        );
        for src in [
            format!("$\\{name}{{G}}, \\{name}{{x1}}$"),
            format!("$f \\colon \\{name} C \\to D$"),
        ] {
            assert_eq!(print(&parse(&src)), src);
        }
        // Without an argument: a literal in the warning colour.
        let literal = Node::Literal(format!("\\{name}"));
        assert_eq!(math(&format!("\\{name}")), vec![literal], "{name}");
        let got = glyphs(&boxed(&format!("\\{name}")).ink);
        assert!(got.iter().all(|g| g.3), "{name}: {got:?}");
    }
}

#[test]
fn tex_sans_and_mono_draw_glyphs_of_the_math_face() {
    // Arrange
    let all = every_kind();
    for name in SANS_MONO_NAMES {
        assert_eq!(alphabet(name).map(|a| a.name), Some(name));
    }
    let sources = SANS_MONO_NAMES
        .iter()
        .map(|name| format!("\\{name}{{{all}}}"))
        .chain([format!("\\mathbf{{\\mathsf{{{all}}}}}")]);

    for src in sources {
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
fn tex_layout_accent_centred_over_sans_and_mono_letters() {
    // Arrange
    let Some(hat) = accent("hat") else {
        panic!("no \\hat");
    };
    let wraps: [fn(&str) -> String; 3] = [
        |s| format!("\\mathsf{{{s}}}"),
        |s| format!("\\mathtt{{{s}}}"),
        |s| format!("\\mathbf{{\\mathsf{{{s}}}}}"),
    ];
    for wrap in wraps {
        for arg in ["G", "x", "1"] {
            let plain = boxed(&wrap(arg));
            let hatted_src = wrap(&format!("\\hat{{{arg}}}"));
            let want = ink_box(&plain.ink).center().x;

            // Act
            let hatted = boxed(&hatted_src);
            let got = ink_box(&mark_ink(&hatted.ink, hat)).center().x;

            // Assert
            assert!(approx_eq(got, want, EPS), "{hatted_src}: {got} vs {want}");
        }
    }
}

#[test]
fn tex_mathsf_in_a_bold_alphabet_is_bold_sans() {
    // Arrange
    let all = every_kind();
    let bold_sans = |ch: char| latin_digit_char(ch, 0x1D5D4, 0x1D5EE, 0x1D7EC);

    for name in BOLD_NAMES {
        for src in [
            format!("\\{name}{{\\mathsf{{x}}}}"),
            format!("\\mathsf{{\\{name}{{x}}}}"),
            format!("\\{name}{{\\mathsf{{\\{name}{{x}}}}}}"),
            format!("\\mathbf{{\\mathsf{{\\{name}{{x}}}}}}"),
        ] {
            // Act / Assert: bold sans, upright, either way round.
            assert_eq!(drawn(&src), vec!['𝘅'], "{src}");
        }
        assert_eq!(
            drawn(&format!("\\{name}{{\\mathsf{{A1}}}}")),
            vec!['𝗔', '𝟭'],
            "{name}"
        );
        // Every Latin letter and digit is bold sans; Greek as the bold
        // alphabet draws it.
        let inner = drawn(&format!("\\{name}{{\\mathsf{{{all}}}}}"));
        let outer = drawn(&format!("\\mathsf{{\\{name}{{{all}}}}}"));
        let bold = drawn(&format!("\\{name}{{{all}}}"));
        let want: Vec<char> = all
            .chars()
            .zip(&bold)
            .map(|(ch, &b)| bold_sans(ch).unwrap_or(b))
            .collect();
        assert_eq!(inner, want, "{name}");
        assert_eq!(outer, want, "{name}");
    }
    // \mathrm in between breaks the chain: innermost wins.
    assert_eq!(drawn("\\mathbf{\\mathrm{\\mathsf{x}}}"), vec!['𝗑']);
    assert_eq!(drawn("\\mathsf{\\mathrm{\\mathbf{x}}}"), vec!['𝐱']);
    assert_eq!(drawn("\\mathbf{\\mathsf{\\mathrm{x}}}"), vec!['x']);
    // Bold sans is not a command.
    assert!(ALPHABETS.iter().all(|a| !(a.name == "mathsf" && a.bold)));
}

#[test]
fn tex_sans_and_mono_nest_and_leave_operators_alone() {
    // Act / Assert: \mathtt has no bold; innermost wins.
    for name in BOLD_NAMES {
        assert_eq!(
            drawn(&format!("\\{name}{{\\mathtt{{x1}}}}")),
            vec!['𝚡', '𝟷']
        );
    }
    assert_eq!(drawn("\\mathtt{\\mathbf{x}}"), vec!['𝐱']);
    assert_eq!(drawn("\\mathtt{\\mathsf{x}}"), vec!['𝗑']);
    assert_eq!(drawn("\\mathsf{\\mathtt{x}}"), vec!['𝚡']);
    assert_eq!(drawn("\\mathsf{\\mathrm{x}}"), vec!['x']);
    assert_eq!(drawn("\\mathsf{\\mathbb{R}C}"), vec!['ℝ', '𝖢']);
    assert_eq!(drawn("\\mathsf{A}_n"), vec!['𝖠', italic('n')]);
    assert_eq!(drawn("\\mathtt{x_1}"), vec!['𝚡', '𝟷']);
    // The T54/T56 bold nestings are unchanged.
    assert_eq!(drawn("\\mathbf{\\boldsymbol{x}}"), vec!['𝒙']);
    assert_eq!(drawn("\\boldsymbol{\\mathbf{x}}"), vec!['𝐱']);
    assert_eq!(drawn("\\bm{\\mathbf{x}}"), vec!['𝐱']);
    assert_eq!(drawn("\\mathbf{\\mathfrak{g}}"), vec!['𝔤']);
    for name in SANS_MONO_NAMES {
        for src in [
            "\\log+\\sum\\text{ab}",
            "\\operatorname{rank}",
            "\\aleph\\partial\\nabla",
        ] {
            assert_eq!(
                boxed(&format!("\\{name}{{{src}}}")),
                boxed(src),
                "\\{name}{{{src}}}"
            );
        }
    }
}

// ---- T65 set and logic symbols -----------------------------------------

/// The *Spec* table of T65: name, text, class.
const SET_LOGIC: [(&str, &str, Class); 27] = [
    ("setminus", "∖", Class::Bin),
    ("complement", "∁", Class::Ord),
    ("varnothing", "∅", Class::Ord),
    ("supset", "⊃", Class::Rel),
    ("subseteq", "⊆", Class::Rel),
    ("supseteq", "⊇", Class::Rel),
    ("subsetneq", "⊊", Class::Rel),
    ("supsetneq", "⊋", Class::Rel),
    ("nsubseteq", "⊈", Class::Rel),
    ("ni", "∋", Class::Rel),
    ("sqcup", "⊔", Class::Bin),
    ("sqcap", "⊓", Class::Bin),
    ("uplus", "⊎", Class::Bin),
    ("lnot", "¬", Class::Ord),
    ("veebar", "⊻", Class::Bin),
    ("barwedge", "⊼", Class::Bin),
    ("top", "⊤", Class::Ord),
    ("bot", "⊥", Class::Ord),
    ("nexists", "∄", Class::Ord),
    ("implies", "⟹", Class::Rel),
    ("impliedby", "⟸", Class::Rel),
    ("vdash", "⊢", Class::Rel),
    ("models", "⊨", Class::Rel),
    ("lvert", "|", Class::Open),
    ("rvert", "|", Class::Close),
    ("lVert", "‖", Class::Open),
    ("rVert", "‖", Class::Close),
];

#[test]
fn tex_parse_set_and_logic_symbols_have_their_text_and_class() {
    for (name, text, class) in SET_LOGIC {
        // Arrange
        let src = format!("$\\{name}$");

        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((text, class)),
            "{name}"
        );
        assert_eq!(print(&parse(&src)), src, "{name}");
    }
}

#[test]
fn tex_set_and_logic_symbols_draw_glyphs_of_the_math_face() {
    for (name, text, _) in SET_LOGIC {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: one unwarned math-face glyph with metrics.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), 1, "{name}");
        assert!(!got[0].3, "{name}");
        assert_eq!(text.chars().collect::<Vec<_>>(), vec![got[0].0], "{name}");
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{name}: {ch:?}");
            assert!(math_metrics::get(ch).is_some(), "{name}: {ch:?}");
        }
    }
}

#[test]
fn tex_layout_implies_pads_a_thick_space_beyond_rel() {
    let p = mm('\u{1D45D}').advance;
    let q = mm('\u{1D45E}').advance;
    for (name, arrow) in [("implies", '⟹'), ("impliedby", '⟸')] {
        // Arrange
        let src = format!("p\\{name} q");

        // Act
        let b = boxed(&src);
        let small = layout_math(&math(&src), SCRIPT_SCALE);

        // Assert: relation space plus a thick space each side.
        let wide = 2.0 * REL_SPACE + 2.0 * WIDE_REL_SPACE;
        assert!(approx_eq(WIDE_REL_SPACE, 5.0 / 18.0, EPS));
        assert!(
            approx_eq(b.width, p + mm(arrow).advance + q + wide, EPS),
            "{name}: {}",
            b.width
        );
        let (pos, _) = glyph(&b.ink, arrow);
        assert!(
            approx_eq(pos.x, p + REL_SPACE + WIDE_REL_SPACE, EPS),
            "{name}: {pos:?}"
        );
        // In a script only the padding stays.
        let want = SCRIPT_SCALE * (p + mm(arrow).advance + q + 2.0 * WIDE_REL_SPACE);
        assert!(approx_eq(small.width, want, EPS), "{name}: {}", small.width);
    }
    // A plain long arrow relation is not padded.
    assert!(approx_eq(
        boxed("p\\Rightarrow q").width,
        p + adv("⇒") + q + 2.0 * REL_SPACE,
        EPS
    ));
}

#[test]
fn tex_layout_set_and_logic_symbols_space_by_class() {
    let width = |src: &str| boxed(src).width;
    let ab = mm(A).advance + mm(B).advance;

    // Binary operators.
    for (src, op) in [
        ("a\\setminus b", "∖"),
        ("a\\veebar b", "⊻"),
        ("a\\sqcup b", "⊔"),
    ] {
        assert!(
            approx_eq(width(src), ab + adv(op) + 2.0 * BIN_SPACE, EPS),
            "{src}"
        );
    }
    // Relations.
    for (src, rel) in [
        ("a\\subseteq b", "⊆"),
        ("a\\vdash b", "⊢"),
        ("a\\ni b", "∋"),
    ] {
        assert!(
            approx_eq(width(src), ab + adv(rel) + 2.0 * REL_SPACE, EPS),
            "{src}"
        );
    }
    // Ordinary atoms get no space.
    assert!(approx_eq(width("\\lnot a"), adv("¬") + mm(A).advance, EPS));
    assert!(approx_eq(width("\\top\\bot"), adv("⊤⊥"), EPS));
    // No space inside the bars; a minus after an opening bar is unary.
    let bars = |src: &str, open: &str| {
        assert!(
            approx_eq(width(src), 2.0 * adv(open) + mm(A).advance, EPS),
            "{src}"
        );
    };
    bars("\\lvert a \\rvert", "|");
    bars("\\lVert a \\rVert", "‖");
    assert!(approx_eq(
        width("\\lvert -a\\rvert"),
        2.0 * adv("|") + adv("\u{2212}") + mm(A).advance,
        EPS
    ));
}

// ---- T66 big set and logic operators -----------------------------------

/// The *Spec* table of T66: name and glyph, all `Class::BigOp`.
const BIG_SET_OPS: [(&str, char); 10] = [
    ("bigcup", '⋃'),
    ("bigcap", '⋂'),
    ("bigwedge", '⋀'),
    ("bigvee", '⋁'),
    ("bigoplus", '⨁'),
    ("bigotimes", '⨂'),
    ("bigodot", '⨀'),
    ("biguplus", '⨄'),
    ("bigsqcup", '⨆'),
    ("coprod", '∐'),
];

#[test]
fn tex_parse_big_set_operators_have_their_text_and_class() {
    for (name, ch) in BIG_SET_OPS {
        // Arrange
        let src = format!("$\\{name}$");
        let text = ch.to_string();

        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((text.as_str(), Class::BigOp)),
            "{name}"
        );
        assert_eq!(print(&parse(&src)), src, "{name}");
    }
}

#[test]
fn tex_big_set_operators_draw_enlarged_glyphs_of_the_math_face() {
    for (name, ch) in BIG_SET_OPS {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: one unwarned math-face glyph with metrics.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), 1, "{name}");
        assert_eq!(got[0].0, ch, "{name}");
        assert!(!got[0].3, "{name}");
        for (c, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{name}: {c:?}");
            assert!(math_metrics::get(c).is_some(), "{name}: {c:?}");
        }
        // Enlarged like \sum, its middle on the axis (y down).
        let (pos, k) = glyph(&b.ink, ch);
        assert!(approx_eq(k, BIG_OP_SCALE, EPS), "{name}: {k}");
        let middle = pos.y - (mm(ch).y_max + mm(ch).y_min) * k / 2.0;
        assert!(approx_eq(middle, -AXIS, EPS), "{name}: {middle}");
        // The box is the glyph's advance and holds its ink.
        assert!(approx_eq(b.width, mm(ch).advance * k, EPS), "{name}");
        assert!(pos.y - mm(ch).y_max * k >= -b.ascent - EPS, "{name}");
        assert!(pos.y - mm(ch).y_min * k <= b.descent + EPS, "{name}");
    }
}

#[test]
fn tex_layout_big_set_operator_limits_are_centred_above_and_below() {
    for (name, ch) in BIG_SET_OPS {
        // Act
        let b = boxed(&format!("\\{name}_{{i=1}}^{{n}} A_i"));

        // Assert
        let (op, big) = glyph(&b.ink, ch);
        let centre = op.x + mm(ch).advance * big / 2.0;
        let (n, ns) = glyph(&b.ink, N);
        assert!(
            approx_eq(n.x + mm(N).advance * ns / 2.0, centre, EPS),
            "{name}: {n:?} {centre}"
        );
        // "i=1" at script scale, without relation space.
        let (i, is) = glyph(&b.ink, I);
        let below = (mm(I).advance + adv("=1")) * is;
        assert!(
            approx_eq(i.x + below / 2.0, centre, EPS),
            "{name}: {i:?} {centre}"
        );
        let top = op.y - mm(ch).y_max * big;
        let bottom = op.y - mm(ch).y_min * big;
        assert!(n.y - mm(N).y_min * ns <= top - LIMIT_GAP + EPS, "{name}");
        assert!(i.y - mm(I).y_max * is >= bottom + LIMIT_GAP - EPS, "{name}");
        // The operand follows the operator, not under it.
        let (a, _) = glyph(&b.ink, '\u{1D434}');
        assert!(a.x >= op.x + mm(ch).advance * big - EPS, "{name}: {a:?}");
    }
}

// ---- T71 combinatorics notation ----------------------------------------

/// The infix commands of ADR-T71-1.
const INFIXES: [&str; 5] = ["choose", "brack", "brace", "over", "atop"];

fn infix(name: &str, left: Vec<Node>, right: Vec<Node>) -> Node {
    Node::Infix {
        name: name.to_owned(),
        left,
        right,
    }
}

pub(super) fn space() -> Node {
    Node::Space(" ".to_owned())
}

#[test]
fn tex_parse_infix_commands_split_their_group() {
    for name in INFIXES {
        // Arrange
        let src = format!("{{n \\{name} k}}");

        // Act
        let nodes = math(&src);

        // Assert
        let split = infix(
            name,
            vec![Node::Char('n'), space()],
            vec![space(), Node::Char('k')],
        );
        assert_eq!(nodes, vec![Node::Group(vec![split])], "{name}");
        let dollars = format!("${src}$");
        assert_eq!(print(&parse(&dollars)), dollars, "{name}");
    }
}

#[test]
fn tex_parse_infix_at_top_level_splits_the_formula() {
    // Act
    let nodes = math("a+1\\over b");

    // Assert
    assert_eq!(
        nodes,
        vec![infix(
            "over",
            vec![Node::Char('a'), Node::Char('+'), Node::Char('1')],
            vec![space(), Node::Char('b')],
        )]
    );
}

#[test]
fn tex_parse_infix_inside_an_argument_splits_the_argument() {
    // Act
    let nodes = math("\\frac{a\\over b}{c}");

    // Assert
    assert_eq!(
        nodes,
        vec![Node::Command {
            name: "frac".to_owned(),
            args: vec![
                braced(vec![infix(
                    "over",
                    vec![Node::Char('a')],
                    vec![space(), Node::Char('b')],
                )]),
                braced(vec![Node::Char('c')]),
            ],
        }]
    );
}

#[test]
fn tex_parse_second_infix_in_a_group_is_literal() {
    // Arrange
    let src = "a\\over b\\choose c";

    // Act
    let nodes = math(src);
    let b = boxed(src);

    // Assert
    assert_eq!(
        nodes,
        vec![infix(
            "over",
            vec![Node::Char('a')],
            vec![
                space(),
                Node::Char('b'),
                Node::Literal("\\choose".to_owned()),
                space(),
                Node::Char('c'),
            ],
        )]
    );
    assert!(glyphs(&b.ink).iter().any(|g| g.0 == 'c' && g.3), "warned");
    let dollars = format!("${src}$");
    assert_eq!(print(&parse(&dollars)), dollars);
}

#[test]
fn tex_parse_infix_in_nested_groups_splits_each() {
    // Act
    let nodes = math("{a\\over b}\\over c");

    // Assert
    let inner = infix(
        "over",
        vec![Node::Char('a')],
        vec![space(), Node::Char('b')],
    );
    assert_eq!(
        nodes,
        vec![infix(
            "over",
            vec![Node::Group(vec![inner])],
            vec![space(), Node::Char('c')],
        )]
    );
}

#[test]
fn tex_parse_script_after_infix_has_no_base() {
    // Act
    let nodes = math("n\\choose^2");

    // Assert
    assert_eq!(
        nodes,
        vec![infix(
            "choose",
            vec![Node::Char('n')],
            vec![Node::Scripts {
                base: None,
                first: script(ScriptKind::Sup, tok(Node::Char('2'))),
                second: None,
            }],
        )]
    );
}

#[test]
fn tex_layout_over_matches_frac() {
    // Act
    let infix = boxed("a \\over b");

    // Assert
    assert_eq!(infix, boxed("\\frac{a}{b}"));
}

#[test]
fn tex_layout_atop_is_frac_without_bar() {
    // Arrange
    let frac = boxed("\\frac{a}{b}");

    // Act
    let b = boxed("a \\atop b");

    // Assert
    assert_eq!(rules(&b.ink), []);
    assert_eq!(glyphs(&b.ink), glyphs(&frac.ink));
    assert!(approx_eq(b.width, frac.width, EPS));
}

#[test]
fn tex_layout_choose_and_tbinom_match_binom() {
    // Arrange
    let binom = boxed("\\binom{n}{k}");

    // Act
    let choose = boxed("n \\choose k");
    let tbinom = boxed("\\tbinom{n}{k}");

    // Assert: `\dbinom` forces display parts (T105 AC-4).
    assert_eq!(choose, binom);
    assert_eq!(tbinom, binom);
    for src in ["\\tbinom{n}{k}", "\\dbinom n k"] {
        let dollars = format!("${src}$");
        assert_eq!(print(&parse(&dollars)), dollars);
    }
}

#[test]
fn tex_layout_brack_and_brace_use_their_brackets() {
    for (name, open, close) in [("brack", '[', ']'), ("brace", '{', '}')] {
        // Act
        let b = boxed(&format!("n \\{name} k"));

        // Assert
        assert_eq!(rules(&b.ink), [], "{name}");
        let g = glyphs(&b.ink);
        let (first, last) = (g[0], g[g.len() - 1]);
        assert_eq!((first.0, last.0), (open, close), "{name}");
        assert!(first.2 > 1.0 && approx_eq(first.2, last.2, EPS), "{g:?}");
        assert!(g.iter().all(|g| !g.3), "{name}: unwarned");
        assert!(faces(&b.ink).iter().all(|f| f.1 == Face::Math), "{name}");
        assert!(approx_eq(first.1.x, 0.0, EPS), "{name}");
        assert!(
            approx_eq(last.1.x + mm(close).advance * last.2, b.width, EPS),
            "{name}"
        );
        let (n, _) = glyph(&b.ink, N);
        let (k, _) = glyph(&b.ink, K);
        assert!(n.y < k.y, "{name}");
    }
}

#[test]
fn tex_layout_multiset_has_double_parentheses() {
    // Arrange
    let src = "\\multiset{n}{k}";

    // Act
    let b = boxed(src);

    // Assert
    assert_eq!(rules(&b.ink), []);
    let g = glyphs(&b.ink);
    let chars: Vec<char> = g.iter().map(|g| g.0).collect();
    assert_eq!(chars, ['(', '(', N, K, ')', ')']);
    assert!(g.iter().all(|g| !g.3));
    let paren = g[0].2;
    assert!(paren > 1.0);
    assert!(g[1].1.x > g[0].1.x);
    // The binomial with one more parenthesis each side, next to it.
    let binom = boxed("\\binom{n}{k}");
    assert!(approx_eq(paren, glyphs(&binom.ink)[0].2, EPS));
    assert!(approx_eq(g[1].1.x, adv("(") * paren, EPS));
    assert!(approx_eq(b.width, binom.width + adv("()") * paren, EPS));
    assert!(approx_eq(g[5].1.x + mm(')').advance * g[5].2, b.width, EPS));
    let dollars = format!("${src}$");
    assert_eq!(print(&parse(&dollars)), dollars);
}

#[test]
fn tex_parse_multiset_without_two_arguments_is_literal() {
    // Act
    let nodes = math("\\multiset{n}");

    // Assert
    assert_eq!(nodes[0], Node::Literal("\\multiset".to_owned()));
}

#[test]
fn tex_layout_underline_spans_its_argument_below() {
    for arg in ["AB", "x", "y", "x_i"] {
        // Arrange
        let plain = boxed(arg);

        // Act
        let b = boxed(&format!("\\underline{{{arg}}}"));

        // Assert
        let strokes = rules(&b.ink);
        assert_eq!(strokes.len(), 1, "{arg}");
        let (p, q, width) = strokes[0];
        assert!(approx_eq(p.y, q.y, EPS), "{arg}: horizontal");
        assert!(approx_eq(p.x.min(q.x), 0.0, EPS), "{arg}: {p:?}");
        assert!(approx_eq(p.x.max(q.x), plain.width, EPS), "{arg}: {q:?}");
        assert!(approx_eq(width, RULE, EPS));
        assert!(p.y - width / 2.0 > plain.descent, "{arg}: below the ink");
        assert!(approx_eq(b.descent, p.y + width / 2.0, EPS), "{arg}");
        assert!(approx_eq(b.width, plain.width, EPS), "{arg}");
        assert!(approx_eq(b.ascent, plain.ascent, EPS), "{arg}");
        assert!(approx_eq(b.italic, plain.italic, EPS), "{arg}");
    }
}

#[test]
fn tex_layout_falling_factorial_underlines_the_superscript() {
    // Act
    let b = boxed("x^{\\underline{k}}");

    // Assert: one rule, under the raised k, at script size.
    let strokes = rules(&b.ink);
    assert_eq!(strokes.len(), 1);
    let (p, _, width) = strokes[0];
    let (k, ks) = glyph(&b.ink, K);
    assert!(ks < 1.0);
    assert!(p.y > k.y && p.y < 0.0, "{p:?} {k:?}");
    assert!(approx_eq(width, RULE * ks, EPS));
}

#[test]
fn tex_parse_nmid_and_hash_have_their_text_and_class() {
    for (name, text, class) in [("nmid", "∤", Class::Rel), ("#", "#", Class::Ord)] {
        // Act
        let nodes = math(&format!("\\{name}"));
        let b = boxed(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((text, class)),
            "{name}"
        );
        let g = glyphs(&b.ink);
        assert_eq!(g.len(), 1, "{name}");
        assert!(!g[0].3, "{name}");
        assert_eq!(faces(&b.ink)[0].1, Face::Math, "{name}");
    }
}

// ---- T67 algebra and group theory symbols ------------------------------

/// The *Spec* table of T67: name, text, class.
const ALGEBRA: [(&str, &str, Class); 20] = [
    ("lhd", "⊲", Class::Rel),
    ("rhd", "⊳", Class::Rel),
    ("unlhd", "⊴", Class::Rel),
    ("unrhd", "⊵", Class::Rel),
    ("triangleleft", "⊲", Class::Rel),
    ("triangleright", "⊳", Class::Rel),
    ("trianglelefteq", "⊴", Class::Rel),
    ("trianglerighteq", "⊵", Class::Rel),
    ("ntriangleleft", "⋪", Class::Rel),
    ("ntriangleright", "⋫", Class::Rel),
    ("rtimes", "⋊", Class::Bin),
    ("ltimes", "⋉", Class::Bin),
    ("wr", "≀", Class::Bin),
    ("odot", "⊙", Class::Bin),
    ("ominus", "⊖", Class::Bin),
    ("oslash", "⊘", Class::Bin),
    ("boxplus", "⊞", Class::Bin),
    ("boxtimes", "⊠", Class::Bin),
    ("amalg", "⨿", Class::Bin),
    ("diamond", "⋄", Class::Bin),
];

/// The operator names of T67, `Class::Op`, drawn as their name.
const ALGEBRA_OPS: [&str; 12] = [
    "Hom", "End", "Aut", "Inn", "Gal", "Ker", "im", "coker", "id", "sgn", "tr", "rank",
];

#[test]
fn tex_parse_algebra_symbols_have_their_text_and_class() {
    for (name, text, class) in ALGEBRA {
        // Arrange
        let src = format!("$\\{name}$");

        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((text, class)),
            "{name}"
        );
        assert_eq!(print(&parse(&src)), src, "{name}");
    }
}

#[test]
fn tex_algebra_symbols_draw_glyphs_of_the_math_face() {
    for (name, text, _) in ALGEBRA {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: one unwarned math-face glyph with metrics.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), 1, "{name}");
        assert!(!got[0].3, "{name}");
        assert_eq!(text.chars().collect::<Vec<_>>(), vec![got[0].0], "{name}");
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{name}: {ch:?}");
            assert!(math_metrics::get(ch).is_some(), "{name}: {ch:?}");
        }
    }
}

#[test]
fn tex_parse_algebra_operator_names_are_operator_words() {
    for name in ALGEBRA_OPS {
        // Arrange
        let src = format!("$\\{name}(A, B)$");

        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((name, Class::Op)),
            "{name}"
        );
        assert_eq!(print(&parse(&src)), src, "{name}");
    }
}

#[test]
fn tex_algebra_operator_names_are_upright_math_words() {
    for name in ALGEBRA_OPS {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: its letters, upright, from the math face.
        let got = glyphs(&b.ink);
        let chars: String = got.iter().map(|g| g.0).collect();
        assert_eq!(chars, name, "{name}");
        assert!(got.iter().all(|g| !g.3), "{name}: {got:?}");
        assert!(faces(&b.ink).iter().all(|g| g.1 == Face::Math), "{name}");
        for ch in name.chars() {
            assert!(math_metrics::get(ch).is_some(), "{ch:?}");
        }
        assert!(approx_eq(b.width, adv(name), EPS), "{name}");
    }
}

#[test]
fn tex_layout_algebra_operator_names_match_operatorname() {
    for name in ALGEBRA_OPS {
        for tail in ["", "(A, B)", " x", "^2", "_k x"] {
            // Act
            let short = boxed(&format!("\\{name}{tail}"));
            let long = boxed(&format!("\\operatorname{{{name}}}{tail}"));

            // Assert
            assert_eq!(short, long, "{name}{tail}");
        }
    }
}

#[test]
fn tex_layout_algebra_symbols_space_by_class() {
    let width = |src: &str| boxed(src).width;
    let ab = mm(A).advance + mm(B).advance;

    // Relations.
    for (src, rel) in [
        ("a\\trianglelefteq b", "⊴"),
        ("a\\lhd b", "⊲"),
        ("a\\ntriangleleft b", "⋪"),
    ] {
        assert!(
            approx_eq(width(src), ab + adv(rel) + 2.0 * REL_SPACE, EPS),
            "{src}"
        );
    }
    // Binary operators.
    for (src, op) in [
        ("a\\rtimes b", "⋊"),
        ("a\\wr b", "≀"),
        ("a\\odot b", "⊙"),
        ("a\\amalg b", "⨿"),
    ] {
        assert!(
            approx_eq(width(src), ab + adv(op) + 2.0 * BIN_SPACE, EPS),
            "{src}"
        );
    }
}

// ---- T68 category theory symbols ---------------------------------------

/// The *Spec* table of T68: name, text, class.
const CATEGORY: [(&str, &str, Class); 27] = [
    ("colon", ":", Class::Punct),
    ("hookrightarrow", "↪", Class::Rel),
    ("hookleftarrow", "↩", Class::Rel),
    ("twoheadrightarrow", "↠", Class::Rel),
    ("twoheadleftarrow", "↞", Class::Rel),
    ("rightarrowtail", "↣", Class::Rel),
    ("leftarrowtail", "↢", Class::Rel),
    ("longrightarrow", "⟶", Class::Rel),
    ("longleftarrow", "⟵", Class::Rel),
    ("longleftrightarrow", "⟷", Class::Rel),
    ("Longrightarrow", "⟹", Class::Rel),
    ("Longleftarrow", "⟸", Class::Rel),
    ("Longleftrightarrow", "⟺", Class::Rel),
    ("longmapsto", "⟼", Class::Rel),
    ("rightrightarrows", "⇉", Class::Rel),
    ("leftleftarrows", "⇇", Class::Rel),
    ("leftrightarrows", "⇆", Class::Rel),
    ("rightleftarrows", "⇄", Class::Rel),
    ("nearrow", "↗", Class::Rel),
    ("searrow", "↘", Class::Rel),
    ("swarrow", "↙", Class::Rel),
    ("nwarrow", "↖", Class::Rel),
    ("Uparrow", "⇑", Class::Rel),
    ("Downarrow", "⇓", Class::Rel),
    ("Updownarrow", "⇕", Class::Rel),
    ("updownarrow", "↕", Class::Rel),
    ("dashv", "⊣", Class::Rel),
];

/// The operator names of T68, `Class::Op`, drawn as their name.
const CATEGORY_OPS: [&str; 3] = ["op", "Ob", "Mor"];

#[test]
fn tex_parse_category_symbols_have_their_text_and_class() {
    for (name, text, class) in CATEGORY {
        // Arrange
        let src = format!("$\\{name}$");

        // Act
        let nodes = math(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((text, class)),
            "{name}"
        );
        assert_eq!(print(&parse(&src)), src, "{name}");
    }
}

#[test]
fn tex_category_symbols_draw_glyphs_of_the_math_face() {
    for (name, text, _) in CATEGORY {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: one unwarned math-face glyph with metrics.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), 1, "{name}");
        assert!(!got[0].3, "{name}");
        assert_eq!(text.chars().collect::<Vec<_>>(), vec![got[0].0], "{name}");
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{name}: {ch:?}");
            assert!(math_metrics::get(ch).is_some(), "{name}: {ch:?}");
        }
    }
}

#[test]
fn tex_parse_category_operator_names_are_operator_words() {
    for name in CATEGORY_OPS {
        // Arrange
        let src = format!("$\\{name}(\\mathcal{{C}})$");

        // Act
        let nodes = math(&format!("\\{name}"));
        let b = boxed(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![sym(name)], "{name}");
        assert_eq!(
            symbol(name).map(|s| (s.text, s.class)),
            Some((name, Class::Op)),
            "{name}"
        );
        assert_eq!(print(&parse(&src)), src, "{name}");
        let chars: String = glyphs(&b.ink).iter().map(|g| g.0).collect();
        assert_eq!(chars, name, "{name}");
        assert!(glyphs(&b.ink).iter().all(|g| !g.3), "{name}");
        assert!(faces(&b.ink).iter().all(|g| g.1 == Face::Math), "{name}");
        assert!(approx_eq(b.width, adv(name), EPS), "{name}");
    }
}

#[test]
fn tex_layout_category_operator_names_match_operatorname() {
    for name in CATEGORY_OPS {
        for tail in ["", "(\\mathcal{C})", " x", "^2", "_k x"] {
            // Act
            let short = boxed(&format!("\\{name}{tail}"));
            let long = boxed(&format!("\\operatorname{{{name}}}{tail}"));

            // Assert
            assert_eq!(short, long, "{name}{tail}");
        }
    }
}

#[test]
fn tex_layout_colon_is_punctuation() {
    // Arrange
    let f_a_b = mm(F).advance + mm(A).advance + mm(B).advance;
    let glyphs_width = f_a_b + adv(":") + adv("→");

    // Act
    let colon = boxed("f\\colon a\\to b").width;
    let plain = boxed("f:a\\to b").width;

    // Assert: nothing before the colon, a thin space after it.
    assert!(
        approx_eq(colon, glyphs_width + OP_SPACE + 2.0 * REL_SPACE, EPS),
        "{colon}"
    );
    assert!(
        approx_eq(plain, glyphs_width + 4.0 * REL_SPACE, EPS),
        "{plain}"
    );
    assert!(colon < plain);
}

#[test]
fn tex_layout_category_arrows_space_as_relations() {
    let width = |src: &str| boxed(src).width;
    let ab = mm(A).advance + mm(B).advance;

    for (src, rel) in [
        ("a\\hookrightarrow b", "↪"),
        ("a\\longrightarrow b", "⟶"),
        ("a\\dashv b", "⊣"),
        ("a\\rightrightarrows b", "⇉"),
        ("a\\Longrightarrow b", "⟹"),
    ] {
        assert!(
            approx_eq(width(src), ab + adv(rel) + 2.0 * REL_SPACE, EPS),
            "{src}"
        );
    }
    // `\implies` draws the same arrow with amsmath's extra padding.
    assert!(width("a\\implies b") > width("a\\Longrightarrow b"));
}

#[test]
fn tex_layout_op_superscript_is_upright() {
    // Act
    let b = boxed("\\mathcal{C}^{\\op}");

    // Assert: script C at full size, then `op` smaller and upright.
    let got = glyphs(&b.ink);
    let chars: String = got.iter().map(|g| g.0).collect();
    assert_eq!(chars, "\u{1D49E}op");
    assert!(approx_eq(got[0].2, 1.0, EPS));
    assert!(got[1].2 < 1.0 && approx_eq(got[1].2, got[2].2, EPS));
    assert!(got.iter().all(|g| !g.3), "{got:?}");
    assert!(faces(&b.ink).iter().all(|g| g.1 == Face::Math));
    assert!(got[1].1.y < got[0].1.y, "superscript is raised (y down)");
}

// ---- T69 classical physics notation ------------------------------------

/// The symbol rows of the T69 *Spec* table: name, text, class.
const PHYSICS: [(&str, &str, Class); 7] = [
    ("Box", "□", Class::Ord),
    ("square", "□", Class::Ord),
    ("triangle", "△", Class::Ord),
    ("angle", "∠", Class::Ord),
    ("ohm", "\u{2126}", Class::Ord),
    ("micro", "\u{B5}", Class::Ord),
    ("grad", "∇", Class::Ord),
];

/// Every physics shortcut source next to its long form (T69 AC-5).
const LONG_FORMS: [(&str, &str); 25] = [
    ("\\dd", "\\,\\mathrm{d}"),
    ("\\dd{x}", "\\,\\mathrm{d}x"),
    ("\\dd x", "\\,\\mathrm{d} x"),
    ("\\dv{x}", "\\frac{\\mathrm{d}}{\\mathrm{d}x}"),
    ("\\dv{f}{x}", "\\frac{\\mathrm{d}f}{\\mathrm{d}x}"),
    ("\\dv f{x}", "\\frac{\\mathrm{d}f}{\\mathrm{d}x}"),
    ("\\pdv{x}", "\\frac{\\partial}{\\partial x}"),
    ("\\pdv{f}{x}", "\\frac{\\partial f}{\\partial x}"),
    ("\\pdv{f}{t}", "\\frac{\\partial f}{\\partial t}"),
    ("\\abs{x}", "\\lvert x\\rvert"),
    ("\\abs{a+b}", "\\lvert a+b\\rvert"),
    ("\\norm{v}", "\\lVert v\\rVert"),
    ("\\order{h^2}", "\\mathcal{O}(h^2)"),
    ("\\vb{F}", "\\mathbf{F}"),
    ("\\vu{n}", "\\hat{\\mathbf{n}}"),
    ("\\curl", "\\nabla\\times"),
    ("\\grad f", "\\nabla f"),
    ("a\\abs{x}b", "a\\lvert x\\rvert b"),
    ("\\curl\\vb{A}", "\\nabla\\times\\mathbf{A}"),
    ("\\dd{x}\\,\\dd{y}", "\\,\\mathrm{d}x\\,\\,\\mathrm{d}y"),
    ("\\abs{\\vb{F}}", "\\lvert\\mathbf{F}\\rvert"),
    (
        "\\dv{\\vb{r}}{t}",
        "\\frac{\\mathrm{d}\\mathbf{r}}{\\mathrm{d}t}",
    ),
    ("e^{\\abs{x}}", "e^{\\lvert x\\rvert}"),
    ("\\vb{F} = m\\dddot{x}", "\\mathbf{F} = m\\dddot{x}"),
    ("\\abs{\\curl x}", "\\lvert\\nabla\\times x\\rvert"),
];

/// Whether `nodes` hold a literal at any depth.
fn has_literal(nodes: &[Node]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Literal(_) => true,
        Node::Group(nodes) => has_literal(nodes),
        Node::Command { args, .. } => args.iter().any(|a| has_literal(&a.nodes)),
        Node::Scripts {
            base,
            first,
            second,
        } => {
            base.as_deref()
                .is_some_and(|b| has_literal(std::slice::from_ref(b)))
                || std::iter::once(first)
                    .chain(second)
                    .any(|s| has_literal(&s.arg.nodes))
        }
        Node::Infix { left, right, .. } => has_literal(left) || has_literal(right),
        _ => false,
    })
}

#[test]
fn tex_parse_classical_physics_symbols_have_their_text_and_class() {
    for (name, text, class) in PHYSICS {
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
fn tex_classical_physics_symbols_draw_glyphs_of_the_math_face() {
    for (name, text, _) in PHYSICS {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: one unwarned math-face glyph with metrics.
        let got = glyphs(&b.ink);
        assert_eq!(got.len(), 1, "{name}");
        assert!(!got[0].3, "{name}");
        assert_eq!(text.chars().collect::<Vec<_>>(), vec![got[0].0], "{name}");
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{name}: {ch:?}");
            assert!(math_metrics::get(ch).is_some(), "{name}: {ch:?}");
        }
    }
}

#[test]
fn tex_layout_units_stay_upright_in_every_style() {
    for style in ["", "\\mathbf", "\\mathit", "\\mathrm", "\\boldsymbol"] {
        for (name, ch) in [("ohm", '\u{2126}'), ("micro", '\u{B5}')] {
            // Act
            let b = boxed(&format!("{style}{{\\{name}}}"));

            // Assert
            let got: Vec<char> = glyphs(&b.ink).iter().map(|g| g.0).collect();
            assert_eq!(got, vec![ch], "{style} {name}");
        }
    }
}

#[test]
fn tex_parse_div_stays_division() {
    assert_eq!(
        symbol("div").map(|s| (s.text, s.class)),
        Some(("÷", Class::Bin))
    );
    assert!(expansion("div").is_none());
    assert!(symbol("laplacian").is_none() && expansion("laplacian").is_none());
}

#[test]
fn tex_layout_dddot_draws_three_dots_over_its_argument() {
    // Arrange
    let x = boxed("x");
    let ddot = boxed("\\ddot{x}");
    let three = '\u{20DB}';
    let m = mm(three);

    // Act
    let b = boxed("\\dddot{x}");

    // Assert: the mark is an unwarned math-face glyph at full size.
    let got = glyphs(&b.ink);
    assert_eq!(got.len(), 2, "{got:?}");
    let (pos, scale) = glyph(&b.ink, three);
    assert!(approx_eq(scale, 1.0, EPS));
    assert!(got.iter().all(|g| !g.3));
    assert!(faces(&b.ink).iter().all(|g| g.1 == Face::Math));
    // Its ink sits above the letter (y down).
    assert!(pos.y - m.y_min <= -x.ascent + EPS, "{pos:?}");
    // Centred where `\ddot` centres its two dots (the middle of the
    // mark's bounds, as accents measure it).
    let centre_of = |ink: &[Ink], ch: char| {
        ink.iter()
            .find(|i| matches!(i, Ink::Glyph { ch: c, .. } if *c == ch))
            .map(|i| {
                let b = i.bounds();
                f32::midpoint(b.min.x, b.max.x)
            })
    };
    let centre = centre_of(&b.ink, three).unwrap_or(f32::NAN);
    let dcentre = centre_of(&ddot.ink, '\u{A8}').unwrap_or(f32::NAN);
    assert!(approx_eq(centre, dcentre, EPS), "{centre} {dcentre}");
    // The box keeps the letter's width and descent.
    assert!(approx_eq(b.width, x.width, EPS));
    assert!(approx_eq(b.descent, x.descent, EPS));
    assert!(b.ascent > x.ascent);
}

#[test]
fn tex_parse_physics_shortcuts_take_their_arguments() {
    let x = || braced(vec![Node::Char('x')]);
    let command = |name: &str, args: Vec<Arg>| Node::Command {
        name: name.to_owned(),
        args,
    };

    // Act + Assert: required arguments, then braced optional ones.
    assert_eq!(math("\\dv{x}"), vec![command("dv", vec![x()])]);
    assert_eq!(
        math("\\dv{f}{x}"),
        vec![command("dv", vec![braced(vec![Node::Char('f')]), x()])]
    );
    assert_eq!(
        math("\\dv f {x}"),
        vec![command(
            "dv",
            vec![
                Arg {
                    lead: " ".to_owned(),
                    braced: false,
                    nodes: vec![Node::Char('f')],
                },
                Arg {
                    lead: " ".to_owned(),
                    braced: true,
                    nodes: vec![Node::Char('x')],
                },
            ]
        )]
    );
    assert_eq!(
        math("\\dv{f}x"),
        vec![
            command("dv", vec![braced(vec![Node::Char('f')])]),
            Node::Char('x')
        ]
    );
    assert_eq!(math("\\dd"), vec![command("dd", vec![])]);
    assert_eq!(math("\\dd{x}"), vec![command("dd", vec![x()])]);
    assert_eq!(
        math("\\dd x"),
        vec![
            command("dd", vec![]),
            Node::Space(" ".to_owned()),
            Node::Char('x')
        ]
    );
    assert_eq!(math("\\curl"), vec![command("curl", vec![])]);
    assert_eq!(
        math("\\abs{x}{y}"),
        vec![
            command("abs", vec![x()]),
            Node::Group(vec![Node::Char('y')])
        ]
    );
    assert_eq!(
        math("\\vb\\alpha"),
        vec![command("vb", vec![tok(sym("alpha"))])]
    );
    for (short, _) in LONG_FORMS {
        let src = format!("${short}$");
        assert_eq!(print(&parse(&src)), src);
    }
    for src in [
        "$\\dd {x}$",
        "$\\pdv f {x} y$",
        "$\\curl^2$",
        "$\\dv{\\dv{x}}$",
    ] {
        assert_eq!(print(&parse(src)), src);
    }
}

#[test]
fn tex_parse_physics_shortcut_without_required_argument_is_literal() {
    for name in ["dv", "pdv", "abs", "norm", "order", "vb", "vu"] {
        // Act
        let nodes = math(&format!("\\{name}"));
        let b = boxed(&format!("\\{name}"));

        // Assert
        assert_eq!(nodes, vec![Node::Literal(format!("\\{name}"))], "{name}");
        assert!(glyphs(&b.ink).iter().all(|g| g.3), "{name}");
    }
    assert_eq!(
        math("\\vb^2"),
        vec![Node::Scripts {
            base: Some(Box::new(Node::Literal("\\vb".to_owned()))),
            first: script(ScriptKind::Sup, tok(Node::Char('2'))),
            second: None,
        }]
    );
}

#[test]
fn tex_layout_physics_shortcuts_match_their_long_forms() {
    for (short, long) in LONG_FORMS {
        // Act
        let got = boxed(short);
        let want = boxed(long);

        // Assert
        assert_eq!(got, want, "{short} vs {long}");
        assert!(glyphs(&got.ink).iter().all(|g| !g.3), "{short}");
    }
}

#[test]
fn tex_layout_physics_shortcut_as_script_base_is_one_atom() {
    // Act
    let squared = boxed("\\abs{x}^2");
    let derivative = boxed("\\dv{f}{x}^2");

    // Assert
    assert_eq!(squared, boxed("{\\lvert x\\rvert}^2"));
    assert_eq!(derivative, boxed("{\\frac{\\mathrm{d}f}{\\mathrm{d}x}}^2"));
}

#[test]
fn tex_expansions_are_well_formed() {
    let commands = [
        "frac",
        "binom",
        "tbinom",
        "dbinom",
        "multiset",
        "sqrt",
        "pmod",
        "underline",
        "text",
        "operatorname",
    ];
    let names: Vec<&str> = EXPANSIONS.iter().map(|e| e.name).collect();
    for name in [
        "dd", "dv", "pdv", "abs", "norm", "order", "vb", "vu", "curl", "bra", "ket", "braket",
        "ketbra", "expval", "mel", "comm", "acomm",
    ] {
        assert!(names.contains(&name), "{name} is an expansion");
    }
    for def in EXPANSIONS {
        let name = def.name;
        assert_eq!(expansion(name), Some(def), "{name}");
        assert_eq!(names.iter().filter(|n| **n == name).count(), 1, "{name}");
        assert!(symbol(name).is_none(), "{name} is also a symbol");
        assert!(accent(name).is_none(), "{name} is also an accent");
        assert!(alphabet(name).is_none(), "{name} is also an alphabet");
        assert!(!commands.contains(&name), "{name} is also a command");
        assert!(!INFIX.contains(&name), "{name} is also infix");
        assert!(!def.templates.is_empty(), "{name}");
        for (extra, template) in def.templates.iter().enumerate() {
            // Each template is subset source without literals.
            assert!(!has_literal(&math(template)), "{name}: {template}");
            // Its slots are the arguments it is used with.
            let count = def.required + extra;
            let chars: Vec<char> = template.chars().collect();
            for w in chars.windows(2).filter(|w| w[0] == '#') {
                let n = w[1].to_digit(10).unwrap_or(0) as usize;
                assert!((1..=count).contains(&n), "{name}: {template}");
            }
        }
    }
}

#[test]
fn tex_expand_replaces_slots_with_arguments() {
    let command = |name: &str, args: Vec<Arg>| Node::Command {
        name: name.to_owned(),
        args,
    };
    let f = braced(vec![Node::Char('f')]);
    let x = braced(vec![Node::Char('x')]);

    // Act + Assert
    assert_eq!(
        expand(&command("dv", vec![f.clone(), x.clone()])),
        Some(math("\\frac{\\mathrm{d}f}{\\mathrm{d}x}"))
    );
    assert_eq!(
        expand(&command("pdv", vec![x.clone()])),
        Some(math("\\frac{\\partial}{\\partial x}"))
    );
    assert_eq!(
        expand(&command("abs", vec![braced(math("a+b"))])),
        Some(math("\\lvert a+b\\rvert"))
    );
    assert_eq!(
        expand(&command("curl", vec![])),
        Some(math("\\nabla\\times"))
    );
    // Not an expansion, or not its argument count.
    assert_eq!(expand(&command("dv", vec![])), None);
    assert_eq!(expand(&command("abs", vec![f, x.clone()])), None);
    assert_eq!(expand(&command("frac", vec![x.clone(), x])), None);
    assert_eq!(expand(&Node::Char('x')), None);
    assert_eq!(expand(&sym("grad")), None);
}

// ---- T70 quantum notation ----------------------------------------------

/// The symbol rows of the T70 *Spec* table: name, text, class.
const QUANTUM: [(&str, &str, Class); 2] = [("dag", "†", Class::Ord), ("Tr", "Tr", Class::Op)];

/// The quantum shortcuts of the T70 *Spec* table.
const QUANTUM_SHORTCUTS: [&str; 8] = [
    "bra", "ket", "braket", "ketbra", "expval", "mel", "comm", "acomm",
];

/// Every quantum source next to its hand-written form (T70 AC-4).
const QUANTUM_LONG_FORMS: [(&str, &str); 24] = [
    ("\\bra{\\psi}", "\\langle \\psi\\rvert"),
    ("\\ket{0}", "\\lvert 0\\rangle"),
    ("\\braket{a}", "\\langle a\\vert a\\rangle"),
    (
        "\\braket{\\phi}{\\psi}",
        "\\langle \\phi\\vert \\psi\\rangle",
    ),
    ("\\ketbra{0}", "\\lvert 0\\rangle\\langle 0\\rvert"),
    ("\\ketbra{0}{1}", "\\lvert 0\\rangle\\langle 1\\rvert"),
    ("\\expval{H}", "\\langle H\\rangle"),
    (
        "\\expval{H}{\\psi}",
        "\\langle \\psi\\vert H\\vert \\psi\\rangle",
    ),
    ("\\mel{n}{H}{m}", "\\langle n\\vert H\\vert m\\rangle"),
    ("\\comm{A}{B}", "[A,B]"),
    ("\\acomm{A}{B}", "\\{A,B\\}"),
    ("\\ket 0", "\\lvert 0\\rangle"),
    ("\\mel n H m", "\\langle n\\vert H\\vert m\\rangle"),
    (
        "\\ket{0}\\otimes\\ket{1}",
        "\\lvert 0\\rangle\\otimes\\lvert 1\\rangle",
    ),
    ("\\comm{A}{B} = AB - BA", "[A,B] = AB - BA"),
    ("\\bra{\\vb{r}}", "\\langle \\mathbf{r}\\rvert"),
    ("\\expval{\\comm{A}{B}}", "\\langle [A,B]\\rangle"),
    ("A^\\dag", "A^\\dagger"),
    ("\\ket{\\psi}^\\dag", "{\\lvert \\psi\\rangle}^\\dagger"),
    ("\\Tr\\rho^2", "\\operatorname{Tr}\\rho^2"),
    ("\\Tr A", "\\operatorname{Tr} A"),
    ("\\tr A", "\\operatorname{tr} A"),
    ("e^{\\braket{a}{b}}", "e^{\\langle a\\vert b\\rangle}"),
    (
        "\\ket{0}\\bra{0} + \\ketbra{1}",
        "\\lvert 0\\rangle\\langle 0\\rvert + \\lvert 1\\rangle\\langle 1\\rvert",
    ),
];

#[test]
fn tex_parse_quantum_symbols_have_their_text_and_class() {
    for (name, text, class) in QUANTUM {
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
    assert_eq!(
        symbol("tr").map(|s| (s.text, s.class)),
        Some(("tr", Class::Op))
    );
    assert_eq!(
        symbol("dagger").map(|s| (s.text, s.class)),
        Some(("†", Class::Bin))
    );
}

#[test]
fn tex_quantum_symbols_draw_glyphs_of_the_math_face() {
    for (name, text, _) in QUANTUM {
        // Act
        let b = boxed(&format!("\\{name}"));

        // Assert: unwarned math-face glyphs with metrics.
        let got = glyphs(&b.ink);
        let chars: String = got.iter().map(|g| g.0).collect();
        assert_eq!(chars, text, "{name}");
        assert!(got.iter().all(|g| !g.3), "{name}");
        for (ch, face) in faces(&b.ink) {
            assert_eq!(face, Face::Math, "{name}: {ch:?}");
            assert!(math_metrics::get(ch).is_some(), "{name}: {ch:?}");
        }
    }
}

#[test]
fn tex_parse_quantum_shortcuts_take_their_arguments() {
    let arg = |c: char| braced(vec![Node::Char(c)]);
    let command = |name: &str, args: Vec<Arg>| Node::Command {
        name: name.to_owned(),
        args,
    };

    // Act + Assert: the argument counts of the table.
    for (name, required, optional) in [
        ("bra", 1, 0),
        ("ket", 1, 0),
        ("braket", 1, 1),
        ("ketbra", 1, 1),
        ("expval", 1, 1),
        ("mel", 3, 0),
        ("comm", 2, 0),
        ("acomm", 2, 0),
    ] {
        let def = expansion(name);
        assert_eq!(
            def.map(|d| (d.required, d.templates.len() - 1)),
            Some((required, optional)),
            "{name}"
        );
    }
    assert_eq!(math("\\ket{0}"), vec![command("ket", vec![arg('0')])]);
    assert_eq!(
        math("\\braket{a}{b}"),
        vec![command("braket", vec![arg('a'), arg('b')])]
    );
    assert_eq!(
        math("\\braket{a}b"),
        vec![command("braket", vec![arg('a')]), Node::Char('b')]
    );
    assert_eq!(
        math("\\mel{n}{H}{m}"),
        vec![command("mel", vec![arg('n'), arg('H'), arg('m')])]
    );
    assert_eq!(
        math("\\comm{A}{B}{C}"),
        vec![
            command("comm", vec![arg('A'), arg('B')]),
            Node::Group(vec![Node::Char('C')])
        ]
    );
    for (short, _) in QUANTUM_LONG_FORMS {
        let src = format!("${short}$");
        assert_eq!(print(&parse(&src)), src);
    }
    for src in ["$\\ket {0}$", "$\\mel n {H} m$", "$\\ketbra{0} {1}^\\dag$"] {
        assert_eq!(print(&parse(src)), src);
    }
}

#[test]
fn tex_parse_quantum_shortcut_without_required_argument_is_literal() {
    for src in QUANTUM_SHORTCUTS
        .iter()
        .map(|n| format!("\\{n}"))
        .chain(["\\mel{n}{H}".to_owned(), "\\comm{A}".to_owned()])
    {
        // Act
        let nodes = math(&src);
        let b = boxed(&src);

        // Assert
        assert!(has_literal(&nodes), "{src}: {nodes:?}");
        assert!(glyphs(&b.ink).iter().any(|g| g.3), "{src}");
    }
    assert_eq!(math("\\ket"), vec![Node::Literal("\\ket".to_owned())]);
}

#[test]
fn tex_layout_quantum_shortcuts_match_their_long_forms() {
    for name in QUANTUM_SHORTCUTS {
        assert!(
            QUANTUM_LONG_FORMS
                .iter()
                .any(|(s, _)| s.starts_with(&format!("\\{name}"))),
            "{name} is tested"
        );
    }
    for (short, long) in QUANTUM_LONG_FORMS {
        // Act
        let got = boxed(short);
        let want = boxed(long);

        // Assert
        assert_eq!(got, want, "{short} vs {long}");
        assert!(glyphs(&got.ink).iter().all(|g| !g.3), "{short}");
    }
}
