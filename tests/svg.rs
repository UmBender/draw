//! SVG export goldens (T114 AC-4): sample documents rendered with the real
//! embedded fonts and compared with `tests/svg/<name>.svg`. To regenerate
//! them after a deliberate change:
//!
//! ```sh
//! DRAW_UPDATE_SNAPSHOTS=1 cargo test --test svg
//! ```

use std::path::{Path, PathBuf};

use draw::core::geom::Vec2;
use draw::core::palette::{ColorId, Rgba};
use draw::core::shape::{CellFill, CellText, Shape, ShapeText, Style};
use draw::core::snapshot::golden::{self, Golden};
use draw::core::svg::{is_well_formed, to_svg};
use draw::shell::app::{MATH_FONT, TEXT_FONT};
use draw::shell::glyphs::FontOutlines;
use draw::theme;

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn style(color: u8, width: f32) -> Style {
    Style {
        color: ColorId::new(color).unwrap_or(ColorId::INK),
        width,
    }
}

fn render(shapes: &[Shape]) -> String {
    let outlines =
        FontOutlines::new(TEXT_FONT, MATH_FONT).unwrap_or_else(|err| panic!("fonts: {err}"));
    to_svg(shapes, Rgba::from_hex(theme::CANVAS), &outlines)
}

fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/svg")
        .join(format!("{name}.svg"))
}

/// Compares `svg` with the golden `name`, or rewrites it on request.
fn assert_golden(svg: &str, name: &str) {
    let path = golden_path(name);
    let update = std::env::var("DRAW_UPDATE_SNAPSHOTS").is_ok_and(|v| v == "1");
    let expected = std::fs::read_to_string(&path).ok();
    match golden::check(svg, expected.as_deref(), update) {
        Golden::Match => {}
        Golden::Write => {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).unwrap_or_else(|err| panic!("{err}"));
            }
            std::fs::write(&path, svg).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        }
        Golden::Missing => panic!("svg {name}: no golden; run with DRAW_UPDATE_SNAPSHOTS=1"),
        Golden::Mismatch(diff) => panic!("svg {name}: differs from golden\n{diff}"),
    }
}

fn check(svg: &str, name: &str) {
    assert!(is_well_formed(svg), "{name} is not well formed");
    assert!(
        !svg.contains("NaN") && !svg.contains("inf"),
        "{name} has a number that is not finite"
    );
    assert_golden(svg, name);
}

fn shapes_scene() -> Vec<Shape> {
    vec![
        Shape::Stroke {
            points: vec![v(0.0, 0.0), v(20.0, 12.5), v(45.0, 8.0), v(60.0, 30.0)],
            widths: None,
            style: style(0, 3.0),
        },
        Shape::Line {
            a: v(80.0, 0.0),
            b: v(180.0, 40.0),
            style: style(2, 2.0),
        },
        Shape::Arrow {
            a: v(80.0, 60.0),
            b: v(200.0, 60.0),
            style: style(3, 4.0),
        },
        Shape::Rect {
            a: v(0.0, 100.0),
            b: v(120.0, 180.0),
            style: style(0, 2.0),
            fill: ColorId::new(1),
            label: Some(7),
            text: None,
        },
        Shape::Ellipse {
            a: v(150.0, 100.0),
            b: v(270.0, 180.0),
            style: style(4, 2.0),
            fill: None,
            label: Some(12),
            text: None,
        },
    ]
}

fn grid_scene() -> Vec<Shape> {
    vec![Shape::Grid {
        a: v(0.0, 0.0),
        b: v(240.0, 160.0),
        cols: 3,
        rows: 2,
        style: style(0, 2.0),
        axes: true,
        fills: vec![CellFill {
            col: 1,
            row: 0,
            color: ColorId::new(2).unwrap_or(ColorId::INK),
        }],
        texts: ShapeText::new("ab".to_owned(), 30.0)
            .map(|content| CellText {
                col: 2,
                row: 1,
                content,
            })
            .into_iter()
            .collect(),
    }]
}

fn text_scene() -> Vec<Shape> {
    vec![
        Shape::Text {
            origin: v(0.0, 0.0),
            text: "n = 5\nsum".to_owned(),
            size: 24.0,
            style: style(0, 0.0),
        },
        Shape::Text {
            origin: v(0.0, 80.0),
            text: "$\\sum_{i=1}^{n} \\frac{a_i}{b_i}$".to_owned(),
            size: 32.0,
            style: style(3, 0.0),
        },
        Shape::Rect {
            a: v(0.0, 180.0),
            b: v(200.0, 260.0),
            style: style(0, 2.0),
            fill: None,
            label: None,
            text: ShapeText::new("$x^2$".to_owned(), 40.0),
        },
    ]
}

/// Rewrites every golden: `cargo test --test svg -- --ignored regenerate`.
#[test]
#[ignore = "rewrites the goldens; run on purpose"]
fn regenerate_goldens() {
    let scenes = [
        ("shapes", shapes_scene()),
        ("grid", grid_scene()),
        ("text", text_scene()),
        ("empty", Vec::new()),
    ];
    for (name, scene) in scenes {
        let path = golden_path(name);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).unwrap_or_else(|err| panic!("{err}"));
        }
        std::fs::write(&path, render(&scene))
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    }
}

#[test]
fn golden_shapes() {
    check(&render(&shapes_scene()), "shapes");
}

#[test]
fn golden_grid() {
    check(&render(&grid_scene()), "grid");
}

#[test]
fn golden_text_and_formulas() {
    check(&render(&text_scene()), "text");
}

#[test]
fn golden_empty() {
    check(&render(&[]), "empty");
}

#[test]
fn every_scene_is_deterministic() {
    for scene in [shapes_scene(), grid_scene(), text_scene()] {
        assert_eq!(render(&scene), render(&scene));
    }
}

#[test]
fn text_scene_needs_no_font_and_defines_each_glyph_once() {
    let svg = render(&text_scene());

    assert!(
        !svg.contains("font-family") && !svg.contains("<text"),
        "outlines only"
    );
    let ids: Vec<&str> = svg.split("<path id=\"").skip(1).collect();
    let mut names: Vec<&str> = ids
        .iter()
        .filter_map(|rest| rest.split('"').next())
        .collect();
    let total = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(total, names.len(), "a glyph is defined twice");
    assert!(total > 0);
}
