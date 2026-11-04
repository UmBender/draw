//! The theme: every colour draw shows, in one place (ADR-T51-1).
//!
//! To restyle draw, change the values below and rebuild
//! (`cargo build --release`, or `scripts/package.sh` for the packaged
//! binary). There is no other way to set a colour: no option, variable or
//! file is read at run time.
//!
//! - A colour is `0xRRGGBB`: paste `#7fa8bc` from any palette as
//!   `0x7fa8bc`.
//! - An `_ALPHA` is the opacity of the colour above it: `0` is invisible,
//!   `255` is opaque.
//! - A light canvas needs a dark `INK` (and dark text on fills is then
//!   light): `cargo test theme` checks the contrast and says which pair is
//!   too close.
//!
//! The shipped values are Kanagawa Dragon (ADR-0015).
#![allow(
    clippy::unreadable_literal,
    reason = "colours are written 0xRRGGBB, as in a palette"
)]

// ---- Canvas -----------------------------------------------------------------

/// Canvas background.
pub const CANVAS: u32 = 0x181616;
/// Numbers, text and the text caret on a filled shape or grid cell.
pub const TEXT_ON_FILL: u32 = 0x181616;

// ---- Drawing colours (keys 1-6, in this order) ------------------------------

/// Key `1`: the default ink.
pub const INK: u32 = 0xc5c9c5;
/// Key `2`.
pub const RED: u32 = 0xd16961;
/// Key `3`.
pub const GREEN: u32 = 0x8aa86e;
/// Key `4`.
pub const BLUE: u32 = 0x7fa8bc;
/// Key `5`.
pub const YELLOW: u32 = 0xceb680;
/// Key `6`.
pub const MAGENTA: u32 = 0xaa88ac;

// ---- Selection and marquee --------------------------------------------------

/// Outline around the selection and around the text block being edited.
pub const SELECTION_OUTLINE: u32 = 0x7fa8bc;
/// Outline of the marquee (the box dragged to select).
pub const MARQUEE_OUTLINE: u32 = 0x7fa8bc;
/// Fill of the marquee.
pub const MARQUEE_FILL: u32 = 0x2d4f67;
/// Opacity of the marquee fill.
pub const MARQUEE_FILL_ALPHA: u8 = 63;

// ---- Snapping ---------------------------------------------------------------

/// Alignment guides shown while smart snap lines shapes up.
pub const GUIDE: u32 = 0x7fa8bc;
/// Dots of the snap grid (shown while grid snap is on).
pub const DOT_GRID: u32 = 0xa6a69c;
/// Opacity of the dot grid.
pub const DOT_GRID_ALPHA: u8 = 89;

// ---- Formulas ---------------------------------------------------------------

/// Formula source drawn as typed because it does not parse (`$\foo$`).
pub const FORMULA_SOURCE: u32 = 0xd16961;
/// The same, when the text itself is drawn in `FORMULA_SOURCE`.
pub const FORMULA_SOURCE_ALT: u32 = 0xceb680;

// ---- Toolbar ----------------------------------------------------------------

/// Toolbar strip and grid size flyout background.
pub const TOOLBAR_BACKGROUND: u32 = 0x0d0c0c;
/// Edge of the toolbar strip and outlines of the flyout buttons.
pub const TOOLBAR_BORDER: u32 = 0xa6a69c;
/// Tool letters, labels and the undo/redo arrows.
pub const TOOLBAR_TEXT: u32 = 0xc5c9c5;
/// Active tool or helper, current colour outline and the `pen` badge.
pub const TOOLBAR_ACTIVE: u32 = 0x7fa8bc;
/// Fill behind an active tool or helper.
pub const TOOLBAR_ACTIVE_FILL: u32 = 0x2d4f67;
/// Opacity of undo/redo when there is nothing to undo or redo.
pub const TOOLBAR_DISABLED_ALPHA: u8 = 76;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::palette::{Rgba, contrast_ratio};
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Lowest WCAG contrast ratio accepted against the canvas (graphics
    /// and large text, ADR-T51-1). A tweaker may lower it knowingly.
    const MIN_CONTRAST: f32 = 3.0;

    // ---- AC-2 the shipped theme --------------------------------------------

    /// Pins the Kanagawa Dragon values (ADR-0015). Changing the theme means
    /// updating this test; the sanity tests below should keep passing.
    #[test]
    fn theme_matches_kanagawa_dragon() {
        // Canvas.
        assert_eq!(CANVAS, 0x181616);
        assert_eq!(TEXT_ON_FILL, 0x181616);
        // Drawing colours, keys 1-6.
        assert_eq!(
            [INK, RED, GREEN, BLUE, YELLOW, MAGENTA],
            [0xc5c9c5, 0xd16961, 0x8aa86e, 0x7fa8bc, 0xceb680, 0xaa88ac]
        );
        // Selection and marquee.
        assert_eq!(SELECTION_OUTLINE, 0x7fa8bc);
        assert_eq!(MARQUEE_OUTLINE, 0x7fa8bc);
        assert_eq!((MARQUEE_FILL, MARQUEE_FILL_ALPHA), (0x2d4f67, 63));
        // Snapping.
        assert_eq!(GUIDE, 0x7fa8bc);
        assert_eq!((DOT_GRID, DOT_GRID_ALPHA), (0xa6a69c, 89));
        // Formulas.
        assert_eq!((FORMULA_SOURCE, FORMULA_SOURCE_ALT), (0xd16961, 0xceb680));
        // Toolbar.
        assert_eq!(TOOLBAR_BACKGROUND, 0x0d0c0c);
        assert_eq!(TOOLBAR_BORDER, 0xa6a69c);
        assert_eq!(TOOLBAR_TEXT, 0xc5c9c5);
        assert_eq!(TOOLBAR_ACTIVE, 0x7fa8bc);
        assert_eq!(TOOLBAR_ACTIVE_FILL, 0x2d4f67);
        assert_eq!(TOOLBAR_DISABLED_ALPHA, 76);
    }

    // ---- AC-3 sanity: what any theme should keep ---------------------------

    #[test]
    fn drawing_colours_are_distinct() {
        // Arrange
        let colours = [INK, RED, GREEN, BLUE, YELLOW, MAGENTA];

        // Act / Assert
        for (i, a) in colours.iter().enumerate() {
            for (j, b) in colours.iter().enumerate().skip(i + 1) {
                assert_ne!(a, b, "drawing colours {} and {} are the same", i + 1, j + 1);
            }
        }
    }

    #[test]
    fn ink_contrasts_with_canvas() {
        // Act
        let ratio = contrast_ratio(Rgba::from_hex(INK), Rgba::from_hex(CANVAS));

        // Assert
        assert!(
            ratio >= MIN_CONTRAST,
            "INK on CANVAS has contrast {ratio:.2}:1, below {MIN_CONTRAST}:1 \
             (a light canvas needs a dark ink)"
        );
    }

    #[test]
    fn selection_outline_contrasts_with_canvas() {
        // Act
        let ratio = contrast_ratio(Rgba::from_hex(SELECTION_OUTLINE), Rgba::from_hex(CANVAS));

        // Assert
        assert!(
            ratio >= MIN_CONTRAST,
            "SELECTION_OUTLINE on CANVAS has contrast {ratio:.2}:1, below {MIN_CONTRAST}:1"
        );
    }

    // ---- AC-1 guard: no colour outside this file ---------------------------

    /// Calls that build a colour; flagged when their first argument is a
    /// written number.
    const CONSTRUCTORS: [&str; 5] = [
        "Color::new(",
        "Color::from_rgba(",
        "from_hex(",
        "from_hex_alpha(",
        "color_u8!(",
    ];

    /// Struct names whose literals spell out a colour.
    const STRUCTS: [&str; 2] = ["Color", "Rgba"];

    /// Edits of a colour's alpha.
    const ALPHA_EDITS: [&str; 4] = [".a *=", ".a +=", ".a -=", ".with_alpha("];

    /// macroquad's named colours.
    const NAMED: [&str; 26] = [
        "LIGHTGRAY",
        "GRAY",
        "DARKGRAY",
        "YELLOW",
        "GOLD",
        "ORANGE",
        "PINK",
        "RED",
        "MAROON",
        "GREEN",
        "LIME",
        "DARKGREEN",
        "SKYBLUE",
        "BLUE",
        "DARKBLUE",
        "PURPLE",
        "VIOLET",
        "DARKPURPLE",
        "BEIGE",
        "BROWN",
        "DARKBROWN",
        "WHITE",
        "BLACK",
        "BLANK",
        "MAGENTA",
        "colors::",
    ];

    fn is_ident(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    /// Byte offsets of every occurrence of `needle` in `line`.
    fn find_all<'a>(line: &'a str, needle: &'a str) -> impl Iterator<Item = usize> + 'a {
        line.match_indices(needle).map(|(i, _)| i)
    }

    /// A six- or eight-digit hex literal such as `0x7fa8bc`.
    fn has_hex_colour(line: &str) -> bool {
        find_all(line, "0x").any(|i| {
            if line[..i].chars().next_back().is_some_and(is_ident) {
                return false;
            }
            let rest = &line[i + 2..];
            let digits = rest.chars().take_while(char::is_ascii_hexdigit).count();
            let after = rest[digits..].chars().next();
            matches!(digits, 6 | 8) && !after.is_some_and(is_ident)
        })
    }

    /// A colour constructor whose first argument is a number.
    fn has_numeric_constructor(line: &str) -> bool {
        CONSTRUCTORS.iter().any(|call| {
            find_all(line, call).any(|i| {
                let arg = line[i + call.len()..].trim_start();
                arg.starts_with(|c: char| c.is_ascii_digit() || c == '.' || c == '-')
            })
        })
    }

    /// A `Color { … }` or `Rgba { … }` expression (not a definition, an
    /// `impl` or a return type).
    fn has_struct_literal(line: &str) -> bool {
        STRUCTS.iter().any(|name| {
            find_all(line, &format!("{name} {{")).any(|i| {
                let before = line[..i].trim_end();
                let word_start = !line[..i].chars().next_back().is_some_and(is_ident);
                let declaration = ["struct", "impl", "enum", "for", "->"]
                    .iter()
                    .any(|w| before.ends_with(w));
                word_start && !declaration
            })
        })
    }

    /// An edit of a colour's alpha channel.
    fn has_alpha_edit(line: &str) -> bool {
        let assign = find_all(line, ".a =").any(|i| !line[i + 4..].starts_with('='));
        assign || ALPHA_EDITS.iter().any(|edit| line.contains(edit))
    }

    /// A macroquad named colour, used bare (not as `path::NAME`).
    fn has_named_colour(line: &str) -> bool {
        NAMED.iter().any(|name| {
            find_all(line, name).any(|i| {
                if name.ends_with("::") {
                    return !line[..i].chars().next_back().is_some_and(is_ident);
                }
                let before = &line[..i];
                let after = line[i + name.len()..].chars().next();
                let qualified = before.ends_with("::") && !before.ends_with("color::");
                !before.chars().next_back().is_some_and(is_ident)
                    && !after.is_some_and(is_ident)
                    && !qualified
            })
        })
    }

    /// A scanner rule: the line test and the name reported when it matches.
    type Check = (fn(&str) -> bool, &'static str);

    /// Every colour literal of `source` outside comments and test code, as
    /// `(line number, rule)`.
    fn colour_literals(source: &str) -> Vec<(usize, &'static str)> {
        let mut found = Vec::new();
        let mut lines = source.lines().enumerate().peekable();
        while let Some((n, raw)) = lines.next() {
            if raw.trim() == "#[cfg(test)]" {
                let indent = &raw[..raw.len() - raw.trim_start().len()];
                let single = lines
                    .peek()
                    .is_some_and(|(_, l)| l.trim_end().ends_with(';'));
                if single {
                    lines.next();
                } else {
                    let end = format!("{indent}}}");
                    for (_, l) in lines.by_ref() {
                        if l.trim_end() == end {
                            break;
                        }
                    }
                }
                continue;
            }
            let line = raw.find("//").map_or(raw, |i| &raw[..i]);
            let checks: [Check; 5] = [
                (has_hex_colour, "hex colour literal"),
                (has_numeric_constructor, "colour built from numbers"),
                (has_struct_literal, "colour struct literal"),
                (has_alpha_edit, "alpha edit"),
                (has_named_colour, "macroquad named colour"),
            ];
            for (check, rule) in checks {
                if check(line) {
                    found.push((n + 1, rule));
                }
            }
        }
        found
    }

    /// Every `.rs` file under `dir`, recursively.
    fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        for entry in entries {
            let path = entry
                .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
                .path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn guard_finds_no_colour_literal_outside_theme() {
        // Arrange
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let theme = src.join("theme.rs");
        let mut files = Vec::new();
        rust_files(&src, &mut files);
        files.sort();

        // Act
        let mut found = Vec::new();
        for file in files.iter().filter(|f| **f != theme) {
            let source =
                fs::read_to_string(file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
            for (line, rule) in colour_literals(&source) {
                let shown = file.strip_prefix(&src).unwrap_or(file).display();
                found.push(format!("src/{shown}:{line}: {rule}"));
            }
        }

        // Assert
        assert!(
            found.is_empty(),
            "colours belong in src/theme.rs (ADR-T51-1):\n{}",
            found.join("\n")
        );
    }

    #[test]
    fn scanner_flags_hex_colour_literal() {
        assert_eq!(
            colour_literals("const A: u32 = 0x7fa8bc;\nconst B: u32 = 0x7fa8bcff;"),
            vec![(1, "hex colour literal"), (2, "hex colour literal")]
        );
    }

    #[test]
    fn scanner_ignores_hashes_and_code_points() {
        // Arrange: hash seeds use `_`, code points have five digits.
        let source =
            "const K: u64 = 0x9e37_79b9_7f4a_7c15;\nconst P: u32 = 0x1D400;\nlet x = 0xff;";

        // Act / Assert
        assert_eq!(colour_literals(source), []);
    }

    #[test]
    fn scanner_flags_colour_constructor_with_number() {
        let source = "let c = Color::new(1.0, 0.0, 0.0, 1.0);\nlet d = Color::from_rgba(255, 0, 0, 255);\nlet e = Rgba::from_hex_alpha( 3, 4);";
        assert_eq!(
            colour_literals(source),
            vec![
                (1, "colour built from numbers"),
                (2, "colour built from numbers"),
                (3, "colour built from numbers"),
            ]
        );
    }

    #[test]
    fn scanner_allows_constructor_with_variable() {
        let source = "Color::new(red, green, blue, alpha)\nRgba::from_hex(theme::INK)\npub const fn from_hex(hex: u32) -> Self {";
        assert_eq!(colour_literals(source), []);
    }

    #[test]
    fn scanner_flags_struct_literal_and_alpha_edit() {
        // Arrange
        let source = "let c = Color { a: 0.5, ..base };\nfill.a *= FADE;\nc.a = 0.2;\nlet k = c.with_alpha(0.3);";

        // Act
        let found = colour_literals(source);

        // Assert
        assert_eq!(
            found,
            vec![
                (1, "colour struct literal"),
                (2, "alpha edit"),
                (3, "alpha edit"),
                (4, "alpha edit"),
            ]
        );
    }

    #[test]
    fn scanner_allows_definitions_and_comparisons() {
        let source = "pub struct Rgba {\nimpl Rgba {\nfn f(x: Rgba) -> Color {\nimpl From<X> for Color {\nif c.a == 0 {}\nstruct MyColor {";
        assert_eq!(colour_literals(source), []);
    }

    #[test]
    fn scanner_flags_macroquad_named_colour() {
        let source = "use macroquad::color::WHITE;\ndraw(x, BLACK);\nlet c = colors::RED;\nlet ok = theme::RED;\nconst REDRAW: u8 = 1;";
        assert_eq!(
            colour_literals(source),
            vec![
                (1, "macroquad named colour"),
                (2, "macroquad named colour"),
                (3, "macroquad named colour"),
            ]
        );
    }

    #[test]
    fn scanner_skips_comments_and_test_modules() {
        // Arrange
        let source = "\
/// Kanagawa `0x181616`.
let x = 1; // was Color::new(1.0, 1.0, 1.0, 1.0)
#[cfg(test)]
use something;
#[cfg(test)]
mod tests {
    const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
}
let late = 0x181616;";

        // Act / Assert: only the line after the test module counts.
        assert_eq!(colour_literals(source), vec![(9, "hex colour literal")]);
    }
}
