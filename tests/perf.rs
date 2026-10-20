//! Performance budgets on a 10 000-shape document (T14, AC-1).
//!
//! Ignored by default; run in release mode on the target laptop:
//!
//! ```sh
//! cargo test --release --test perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! One thread, so the measurements do not compete for the CPU.
//!
//! Each test times many runs of one operation and checks the median against
//! its budget, so a single scheduler hiccup does not fail it. Debug builds
//! skip the checks: unoptimised timings say nothing about the shipped binary.

use std::hint::black_box;
use std::time::{Duration, Instant};

use draw::core::camera::Camera;
use draw::core::command::{Command, Tool};
use draw::core::document::{Document, Edit, ShapeId, Transaction, tx_insert, tx_remove};
use draw::core::editor::Editor;
use draw::core::geom::Vec2;
use draw::core::input::{InputEvent, Key, Modifiers, PointerButton};
use draw::core::palette::ColorId;
use draw::core::shape::{CellFill, Shape, Style};
use draw::core::smoothing::{Smoother, SmoothingLevel, simplify_rdp};
use draw::core::snap::{DragKind, Snaps, Targets, snap_drag};
use draw::core::tex;
use draw::core::text_layout::glyphs;
use draw::shell::app::{MATH_FONT, TEXT_FONT};
use draw::shell::glyphs::GlyphAtlas;
use draw::shell::render::{
    Batch, MeshSink, cull_rect, is_visible, prepare_text_block, push_text_block,
};
use macroquad::color::WHITE;
use macroquad::models::Mesh;

/// Shapes in the benchmark document.
const SHAPES: usize = 10_000;

/// Points in the benchmark stroke.
const STROKE_POINTS: usize = 1_000;

/// Budget of hit-testing, culling and stroke finishing (AC-1).
const BUDGET: Duration = Duration::from_millis(1);

/// Budget of one snapped pointer move: a quarter of a 60 Hz frame
/// (ADR-T14-1).
const SNAP_BUDGET: Duration = Duration::from_millis(4);

/// Timed runs per measurement.
const RUNS: usize = 101;

/// Untimed runs before measuring, to warm caches.
const WARMUP: usize = 5;

/// Side of the square world area the shapes are scattered over.
const WORLD: f32 = 20_000.0;

/// Viewport of a 1080p screen, in pixels.
const VIEWPORT: Vec2 = Vec2::new(1920.0, 1080.0);

/// Text blocks in the T29 benchmark (AC-8).
const TEXT_BLOCKS: usize = 500;

/// Characters per benchmark text block.
const TEXT_CHARS: usize = 20;

/// Deterministic xorshift generator, so every run times the same document.
struct Rng(u64);

impl Rng {
    /// The next value in `[0, 1)`.
    fn unit(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    /// The next value in `[lo, hi)`.
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }

    /// A point in the world area.
    fn point(&mut self) -> Vec2 {
        Vec2::new(self.range(0.0, WORLD), self.range(0.0, WORLD))
    }
}

/// `true` when timings are meaningful (release build); prints why not.
fn optimised() -> bool {
    if cfg!(debug_assertions) {
        eprintln!("perf: skipped in a debug build, use `cargo test --release`");
        return false;
    }
    true
}

/// Median wall time of `op` over [`RUNS`] runs, after [`WARMUP`] runs.
/// `setup` builds each run's input outside the timed section, and the
/// output is dropped after the clock stops (T25: an `op` may return its
/// 10 000-shape input).
fn median_time<I, O>(mut setup: impl FnMut() -> I, mut op: impl FnMut(I) -> O) -> Duration {
    for _ in 0..WARMUP {
        black_box(op(black_box(setup())));
    }
    let mut times: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let input = black_box(setup());
            let start = Instant::now();
            let output = black_box(op(input));
            let elapsed = start.elapsed();
            drop(output);
            elapsed
        })
        .collect();
    times.sort_unstable();
    times[RUNS / 2]
}

/// Prints `name`'s median and checks it against `budget`.
fn assert_within_budget(name: &str, median: Duration, budget: Duration) {
    eprintln!("perf: {name}: median {median:?} (budget {budget:?})");
    assert!(
        median < budget,
        "{name} took {median:?}, budget is {budget:?}"
    );
}

/// A mix of every shape kind, sized like contest sketches: strokes, lines,
/// arrows, filled and numbered rectangles and ellipses, and grids with
/// indices and filled cells.
fn mixed_shapes(count: usize) -> Vec<Shape> {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let style = Style {
        color: ColorId::INK,
        width: 3.0,
    };
    let fill = ColorId::new(3);
    (0..count)
        .map(|i| {
            let a = rng.point();
            let b = a + Vec2::new(rng.range(20.0, 200.0), rng.range(20.0, 200.0));
            let label = u32::try_from(i).ok();
            match i % 6 {
                0 => Shape::Stroke {
                    points: (0..30)
                        .map(|k| a + Vec2::new(k as f32 * 5.0, rng.range(-10.0, 10.0)))
                        .collect(),
                    widths: None,
                    style,
                },
                1 => Shape::Line { a, b, style },
                2 => Shape::Arrow { a, b, style },
                3 => Shape::Rect {
                    a,
                    b,
                    style,
                    fill: if i % 12 == 3 { fill } else { None },
                    label,
                    text: None,
                },
                4 => Shape::Ellipse {
                    a,
                    b,
                    style,
                    fill: if i % 12 == 4 { fill } else { None },
                    label,
                    text: None,
                },
                _ => Shape::Grid {
                    a,
                    b,
                    cols: 8,
                    rows: 8,
                    style,
                    axes: true,
                    fills: (0..8)
                        .filter_map(|k| {
                            Some(CellFill {
                                col: k,
                                row: k,
                                color: fill?,
                            })
                        })
                        .collect(),
                    texts: Vec::new(),
                },
            }
        })
        .collect()
}

/// A document holding [`mixed_shapes`].
fn benchmark_document() -> Document {
    let mut doc = Document::new();
    let tx = tx_insert(&mut doc, mixed_shapes(SHAPES));
    assert!(doc.apply(&tx).is_ok(), "fresh ids always apply");
    assert_eq!(doc.len(), SHAPES);
    doc
}

/// A shaky hand-drawn loop of `n` raw pointer positions, in pixels.
fn shaky_loop(n: usize) -> Vec<Vec2> {
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let r = 300.0 + 40.0 * (3.0 * t).sin();
            Vec2::new(
                960.0 + r * t.cos() + rng.range(-2.0, 2.0),
                540.0 + r * t.sin() + rng.range(-2.0, 2.0),
            )
        })
        .collect()
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_hit_test_miss_with_10k_shapes_is_under_1ms() {
    // Arrange: a point outside every shape, so the whole document is scanned.
    if !optimised() {
        return;
    }
    let doc = benchmark_document();
    let p = Vec2::new(-500.0, -500.0);

    // Act
    let median = median_time(|| p, |p| doc.topmost_where(|shape| shape.hit(p, 4.0)));

    // Assert
    assert_eq!(doc.topmost_where(|shape| shape.hit(p, 4.0)), None);
    assert_within_budget("hit-test miss, 10k shapes", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_eraser_scan_with_10k_shapes_is_under_1ms() {
    // Arrange: random points across the world, as the eraser sweeps.
    if !optimised() {
        return;
    }
    let doc = benchmark_document();
    let mut rng = Rng(7);
    let points: Vec<Vec2> = (0..RUNS + WARMUP).map(|_| rng.point()).collect();
    let mut next = points.iter().copied().cycle();

    // Act
    let median = median_time(
        || next.next().unwrap_or(Vec2::ZERO),
        |p| doc.shapes().any(|(_, shape)| shape.hit_outline(p, 4.0)),
    );

    // Assert
    assert_within_budget("eraser outline scan, 10k shapes", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_culling_pass_with_10k_shapes_is_under_1ms() {
    // Arrange: a 1080p view zoomed out over part of the world.
    if !optimised() {
        return;
    }
    let doc = benchmark_document();
    let camera = Camera::new(Vec2::new(-2_000.0, -2_000.0), 0.25);

    // Act
    let median = median_time(
        || cull_rect(&camera, VIEWPORT),
        |view| {
            doc.shapes()
                .filter(|(_, shape)| is_visible(shape.bounds(), view))
                .count()
        },
    );

    // Assert
    let visible = doc
        .shapes()
        .filter(|(_, shape)| is_visible(shape.bounds(), cull_rect(&camera, VIEWPORT)))
        .count();
    assert!(
        visible > 0 && visible < SHAPES,
        "view must cull some shapes"
    );
    assert_within_budget("culling pass, 10k shapes", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_stroke_finish_with_1000_points_is_under_1ms() {
    // Arrange: the strongest smoothing level keeps the most RDP work.
    if !optimised() {
        return;
    }
    let mut smoother = Smoother::new(SmoothingLevel::High.params(), 1.0);
    for p in shaky_loop(STROKE_POINTS) {
        smoother.push(p);
    }

    // Act
    let median = median_time(|| smoother.clone(), Smoother::finish);

    // Assert
    assert!(smoother.clone().finish().len() >= 2);
    assert_within_budget("stroke finish (RDP), 1000 points", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_rdp_on_1000_raw_points_is_under_1ms() {
    // Arrange: unsmoothed input, every point still present.
    if !optimised() {
        return;
    }
    let points = shaky_loop(STROKE_POINTS);

    // Act
    let median = median_time(|| (), |()| simplify_rdp(&points, 1.5));

    // Assert
    assert_within_budget("RDP, 1000 raw points", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_snap_drag_with_10k_shapes_is_under_4ms() {
    // Arrange: one pointer move of a rectangle drag with grid and smart snap,
    // which rebuilds the targets from the document on every move.
    if !optimised() {
        return;
    }
    let doc = benchmark_document();
    let camera = Camera::new(Vec2::ZERO, 1.0);
    let snaps = Snaps {
        grid: true,
        smart: true,
    };
    let start = Vec2::new(5_003.0, 5_007.0);
    let end = Vec2::new(5_121.0, 5_089.0);

    // Act
    let median = median_time(
        || (),
        |()| {
            let targets = Targets::from_document(&doc);
            snap_drag(start, end, DragKind::Box, snaps, &targets, &camera, 3.0)
        },
    );

    // Assert
    assert_within_budget("snap drag move, 10k shapes", median, SNAP_BUDGET);
}

/// Every other shape of `doc`: half the document selected (T25).
fn half_selected(doc: &Document) -> Vec<ShapeId> {
    doc.shapes().map(|(id, _)| id).step_by(2).collect()
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_lookup_5k_selected_of_10k_is_under_1ms() {
    // Arrange: a document never queried, so each run also builds whatever
    // lookup structure the document keeps (worst case after an edit).
    if !optimised() {
        return;
    }
    let pristine = benchmark_document();
    let selected = half_selected(&pristine);

    // Act: what selection bounds and pruning do.
    let median = median_time(
        || pristine.clone(),
        |doc| {
            let found = selected.iter().filter(|id| doc.get(**id).is_some()).count();
            (doc, found)
        },
    );

    // Assert
    assert_within_budget("lookup 5k selected, 10k shapes", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_move_commit_5k_of_10k_is_under_1ms() {
    // Arrange: the Replace transaction of moving half the document.
    if !optimised() {
        return;
    }
    let pristine = benchmark_document();
    let edits: Vec<Edit> = half_selected(&pristine)
        .into_iter()
        .filter_map(|id| {
            let before = pristine.get(id)?.clone();
            let mut after = before.clone();
            after.translate(Vec2::new(5.0, 3.0));
            Some(Edit::Replace { id, before, after })
        })
        .collect();
    let tx = Transaction::from(edits);
    let pristine = benchmark_document();

    // Act
    let median = median_time(
        || pristine.clone(),
        |mut doc| {
            let ok = doc.apply(&tx).is_ok();
            (doc, ok)
        },
    );

    // Assert
    assert!(pristine.clone().apply(&tx).is_ok());
    assert_within_budget("move commit 5k, 10k shapes", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_delete_5k_of_10k_is_under_1ms() {
    // Arrange
    if !optimised() {
        return;
    }
    let pristine = benchmark_document();
    let tx = tx_remove(&pristine, &half_selected(&pristine));

    // Act
    let median = median_time(
        || pristine.clone(),
        |mut doc| {
            let ok = doc.apply(&tx).is_ok();
            (doc, ok)
        },
    );

    // Assert
    assert!(pristine.clone().apply(&tx).is_ok());
    assert_within_budget("delete 5k, 10k shapes", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_undo_delete_5k_of_10k_is_under_1ms() {
    // Arrange: the document after deleting half of it, and the undo.
    if !optimised() {
        return;
    }
    let mut deleted = benchmark_document();
    let tx = tx_remove(&deleted, &half_selected(&deleted));
    assert!(deleted.apply(&tx).is_ok());
    let undo = tx.inverse();

    // Act
    let median = median_time(
        || deleted.clone(),
        |mut doc| {
            let ok = doc.apply(&undo).is_ok();
            (doc, ok)
        },
    );

    // Assert
    assert!(deleted.clone().apply(&undo).is_ok());
    assert_within_budget("undo delete 5k, 10k shapes", median, BUDGET);
}

/// [`TEXT_BLOCKS`] blocks of [`TEXT_CHARS`] characters over a 1080p screen
/// area, each on two lines with a multi-byte character.
fn text_blocks() -> Vec<Shape> {
    let mut rng = Rng(0x51_7cc1_b727_220a);
    let style = Style {
        color: ColorId::INK,
        width: 0.0,
    };
    (0..TEXT_BLOCKS)
        .map(|i| {
            let origin = Vec2::new(rng.range(-200.0, 2_400.0), rng.range(-200.0, 1_400.0));
            let mut text: String = (0..TEXT_CHARS - 2)
                .map(|k| char::from(b'a' + ((i + k) % 26) as u8))
                .collect();
            text.insert(9, '\n');
            text.push('∑');
            Shape::Text {
                origin,
                text,
                size: 18.0,
                style,
            }
        })
        .collect()
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_text_layout_and_cull_500_blocks_is_under_1ms() {
    // Arrange
    if !optimised() {
        return;
    }
    let mut doc = Document::new();
    let tx = tx_insert(&mut doc, text_blocks());
    assert!(doc.apply(&tx).is_ok(), "fresh ids always apply");
    let camera = Camera::default();

    // Act: what a full re-render computes on the CPU before drawing.
    let median = median_time(
        || cull_rect(&camera, VIEWPORT),
        |view| {
            let mut sum = 0.0f32;
            for (_, shape) in doc.shapes() {
                if let Shape::Text {
                    origin, text, size, ..
                } = shape
                {
                    if is_visible(shape.bounds(), view) {
                        for glyph in glyphs(*origin, text, *size) {
                            sum += camera.world_to_screen(glyph.pos).x;
                        }
                    }
                }
            }
            sum
        },
    );

    // Assert
    assert_within_budget("text layout + cull, 500 blocks", median, BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_typing_with_500_blocks_is_under_1ms() {
    // Arrange: 500 committed blocks, then an open one.
    if !optimised() {
        return;
    }
    let none = Modifiers::NONE;
    let mut editor = Editor::new();
    editor.handle(InputEvent::Resize { size: VIEWPORT });
    editor.apply(Command::SetTool(Tool::Text));
    for shape in text_blocks() {
        let Shape::Text { origin, text, .. } = shape else {
            continue;
        };
        let pos = editor.camera().world_to_screen(origin);
        for event in [
            InputEvent::PointerDown {
                pos,
                button: PointerButton::Left,
                mods: none,
            },
            InputEvent::PointerUp {
                pos,
                button: PointerButton::Left,
                mods: none,
            },
        ] {
            editor.handle(event);
        }
        for ch in text.chars() {
            editor.handle(InputEvent::Char { ch, mods: none });
        }
        editor.handle(InputEvent::KeyDown {
            key: Key::Escape,
            mods: none,
        });
    }
    assert!(
        editor.document().len() >= TEXT_BLOCKS / 2,
        "blocks committed"
    );
    editor.handle(InputEvent::PointerDown {
        pos: Vec2::new(960.0, 1_060.0),
        button: PointerButton::Left,
        mods: none,
    });
    editor.handle(InputEvent::PointerUp {
        pos: Vec2::new(960.0, 1_060.0),
        button: PointerButton::Left,
        mods: none,
    });
    assert!(editor.text_edit().is_some(), "a block is open");

    // Act: one keystroke and the overlay the frame draws.
    let median = median_time(
        || editor.clone(),
        |mut editor| {
            editor.handle(InputEvent::Char {
                ch: 'x',
                mods: none,
            });
            let overlay = editor.overlay();
            (editor, overlay)
        },
    );

    // Assert
    assert_within_budget("typing, 500 blocks", median, BUDGET);
}

/// Counts the chunks and indices a batch submits, without a GPU.
#[derive(Default)]
struct CountingSink {
    chunks: usize,
    indices: usize,
}

impl MeshSink for CountingSink {
    fn draw(&mut self, mesh: &mut Mesh, _page: Option<u32>) {
        self.chunks += 1;
        self.indices += mesh.indices.len();
    }
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_tessellate_10k_glyphs_is_under_1ms() {
    // Arrange: the T29 blocks, 500 × 20 characters, glyphs already in the
    // atlas (a steady full re-render, T32 AC-6).
    if !optimised() {
        return;
    }
    let Ok(mut atlas) = GlyphAtlas::new(TEXT_FONT) else {
        panic!("embedded font loads");
    };
    let blocks = text_blocks();
    let camera = Camera::default();
    for shape in &blocks {
        if let Shape::Text { text, size, .. } = shape {
            prepare_text_block(&mut atlas, text, *size, &camera, 1.0);
        }
    }

    // Act: tessellate every glyph into the batch.
    let median = median_time(
        || Batch::new(CountingSink::default()),
        |mut batch| {
            for shape in &blocks {
                if let Shape::Text {
                    origin, text, size, ..
                } = shape
                {
                    push_text_block(
                        &mut batch, &atlas, *origin, text, *size, &camera, 1.0, WHITE,
                    );
                }
            }
            batch.finish()
        },
    );

    // Assert
    let mut batch = Batch::new(CountingSink::default());
    for shape in &blocks {
        if let Shape::Text {
            origin, text, size, ..
        } = shape
        {
            push_text_block(
                &mut batch, &atlas, *origin, text, *size, &camera, 1.0, WHITE,
            );
        }
    }
    let sink = batch.finish();
    assert_eq!(
        sink.indices / 6,
        TEXT_BLOCKS * (TEXT_CHARS - 1),
        "one quad per glyph"
    );
    assert_within_budget("tessellate 10k glyphs", median, BUDGET);
}

/// Text blocks with formulas in the T30 tests.
const FORMULA_BLOCKS: usize = 200;

/// Budget of parsing and laying out every formula block from scratch, as
/// after loading or when every text changes (T30 AC-5).
const TEX_BUDGET: Duration = Duration::from_millis(2);

/// `FORMULA_BLOCKS` texts built from the goal's example formulas, each
/// numbered so no two are equal.
fn formula_texts() -> Vec<String> {
    const FORMULAS: [&str; 4] = [
        "dp: $dp_{i,j} = \\min(dp_{i-1,j}, dp_{i,j-1}) + a_{i,j}$",
        "$O(n \\log n)$ time",
        "$\\sum_{k=1}^{n} k^2$\nsum of squares",
        "$\\frac{n(n+1)}{2}$ and $\\sqrt{x} \\le \\binom{n}{k}$",
    ];
    (0..FORMULA_BLOCKS)
        .map(|i| format!("{} #{i}", FORMULAS[i % FORMULAS.len()]))
        .collect()
}

/// The formula texts as blocks scattered over a 1080p view.
fn formula_blocks() -> Vec<Shape> {
    let mut rng = Rng(0x7e_c0de_f00d_5eed);
    let style = Style {
        color: ColorId::INK,
        width: 0.0,
    };
    formula_texts()
        .into_iter()
        .map(|text| Shape::Text {
            origin: Vec2::new(rng.range(-200.0, 1_800.0), rng.range(-100.0, 1_000.0)),
            text,
            size: 18.0,
            style,
        })
        .collect()
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_tex_layout_200_blocks_is_under_2ms() {
    // Arrange
    if !optimised() {
        return;
    }
    let texts = formula_texts();

    // Act: parse and lay out every block, uncached.
    let median = median_time(
        || (),
        |()| {
            texts
                .iter()
                .map(|text| tex::layout(&tex::parse(text)).ink.len())
                .sum::<usize>()
        },
    );

    // Assert
    assert_within_budget("tex parse + layout, 200 blocks", median, TEX_BUDGET);
}

#[test]
#[ignore = "perf: run with `cargo test --release -- --ignored perf`"]
fn perf_tessellate_200_formula_blocks_is_under_1ms() {
    // Arrange: layouts cached and glyphs in the atlas (a steady full
    // re-render).
    if !optimised() {
        return;
    }
    // Both fonts, as the app loads them (T33).
    let Ok(mut atlas) = GlyphAtlas::new(TEXT_FONT).and_then(|atlas| atlas.with_math(MATH_FONT))
    else {
        panic!("embedded fonts load");
    };
    let blocks = formula_blocks();
    let camera = Camera::default();
    for shape in &blocks {
        if let Shape::Text { text, size, .. } = shape {
            prepare_text_block(&mut atlas, text, *size, &camera, 1.0);
        }
    }
    let push_all = |mut batch: Batch<CountingSink>| {
        for shape in &blocks {
            if let Shape::Text {
                origin, text, size, ..
            } = shape
            {
                push_text_block(
                    &mut batch, &atlas, *origin, text, *size, &camera, 1.0, WHITE,
                );
            }
        }
        batch.finish()
    };

    // Act
    let median = median_time(|| Batch::new(CountingSink::default()), push_all);

    // Assert
    let sink = push_all(Batch::new(CountingSink::default()));
    assert!(
        sink.indices / 6 >= FORMULA_BLOCKS * 10,
        "glyphs and rules drawn"
    );
    assert_within_budget("tessellate 200 formula blocks", median, BUDGET);
}
