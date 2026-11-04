//! Soak tests (T49): long synthetic sessions through the headless `Editor`
//! that check what a whole contest day of use must not do — panic, leak, or
//! let the undo history grow without bound
//! (ADR-T49-1).
//!
//! `cargo test` runs the short ones. The resident-memory run is ignored by
//! default; run it in release mode, alone, on Linux:
//!
//! ```sh
//! cargo test --release --test soak -- --ignored --nocapture --test-threads=1
//! ```

mod common;

use common::snapshots::load_fixture;
use draw::core::command::{Command, Tool};
use draw::core::editor::Editor;
use draw::core::geom::Vec2;
use draw::core::history::{HISTORY_BYTES, HISTORY_LIMIT};
use draw::core::input::{InputEvent, Key, Modifiers, PointerButton};

/// Pointer moves in one synthetic drag.
const DRAG_STEPS: usize = 20;

/// Cycles of the mixed session in the default run.
const MIXED_CYCLES: usize = 2_000;

/// Strokes in the large-selection document.
const LARGE_STROKES: usize = 1_000;

/// Moves of the whole large selection.
const LARGE_MOVES: usize = 600;

/// Cycles before the resident-memory baseline is taken.
const WARMUP_CYCLES: usize = 10_000;

/// Cycles of the resident-memory run.
const LONG_CYCLES: usize = 200_000;

/// Allowed resident-memory growth after the warm-up, in KiB.
const RSS_GROWTH_KIB: u64 = 1_024;

const NONE: Modifiers = Modifiers::NONE;

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// A 1080p editor.
fn editor() -> Editor {
    let mut ed = Editor::new();
    ed.handle(InputEvent::Resize {
        size: v(1920.0, 1080.0),
    });
    ed
}

/// Left drag from `a` to `b` with a small wobble, so pen strokes keep
/// their points through smoothing.
fn drag(ed: &mut Editor, a: Vec2, b: Vec2) {
    ed.handle(InputEvent::PointerMove { pos: a, mods: NONE });
    ed.handle(InputEvent::PointerDown {
        pos: a,
        button: PointerButton::Left,
        mods: NONE,
    });
    for i in 1..=DRAG_STEPS {
        let t = i as f32 / DRAG_STEPS as f32;
        let wobble = (t * 9.0).sin() * 5.0;
        let pos = v(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t + wobble);
        ed.handle(InputEvent::PointerMove { pos, mods: NONE });
    }
    ed.handle(InputEvent::PointerUp {
        pos: b,
        button: PointerButton::Left,
        mods: NONE,
    });
}

/// Straight left drag from `a` to `b`.
fn straight_drag(ed: &mut Editor, a: Vec2, b: Vec2) {
    ed.handle(InputEvent::PointerMove { pos: a, mods: NONE });
    ed.handle(InputEvent::PointerDown {
        pos: a,
        button: PointerButton::Left,
        mods: NONE,
    });
    ed.handle(InputEvent::PointerMove { pos: b, mods: NONE });
    ed.handle(InputEvent::PointerUp {
        pos: b,
        button: PointerButton::Left,
        mods: NONE,
    });
}

/// One net-zero cycle: every creation tool, a text block with a formula,
/// a zoom step, select-all, duplicate, delete, then undo back to the
/// cycle's start (a redo and an undo on the way).
fn cycle(ed: &mut Editor, k: usize) {
    let off = (k % 50) as f32 * 7.0;
    let mut commits = 0;
    for tool in [
        Tool::Pen,
        Tool::Line,
        Tool::Arrow,
        Tool::Rect,
        Tool::Ellipse,
        Tool::Grid,
    ] {
        ed.apply(Command::SetTool(tool));
        drag(ed, v(100.0 + off, 100.0), v(300.0 + off, 250.0));
        commits += 1;
    }
    ed.apply(Command::SetTool(Tool::Text));
    let at = v(500.0, 500.0 + off);
    straight_drag(ed, at, at);
    for ch in format!("x_{k} = $\\frac{{a}}{{b{k}}}$").chars() {
        ed.handle(InputEvent::Char { ch, mods: NONE });
    }
    ed.handle(InputEvent::KeyDown {
        key: Key::Escape,
        mods: NONE,
    });
    commits += 1;
    let delta = if k.is_multiple_of(2) { 1.0 } else { -1.0 };
    ed.handle(InputEvent::Scroll {
        pos: v(400.0, 300.0),
        delta,
    });
    ed.apply(Command::SelectAll);
    ed.apply(Command::Duplicate);
    ed.apply(Command::DeleteSelection);
    commits += 2;
    for _ in 0..=commits {
        ed.apply(Command::Undo);
    }
    ed.apply(Command::Redo);
    ed.apply(Command::Undo);
}

/// Resident set size of this process in KiB, from `/proc/self/statm`.
fn rss_kib() -> Option<u64> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages * 4)
}

#[test]
fn soak_mixed_session_stays_bounded() {
    // Arrange
    let mut ed = editor();

    // Act / Assert
    for k in 0..MIXED_CYCLES {
        cycle(&mut ed, k);
        assert!(ed.history_bytes() <= HISTORY_BYTES, "cycle {k}");
        assert!(ed.document().is_empty(), "cycle {k}");
    }
    assert!(ed.document_revision() > 0);
    let mut depth = 0;
    while ed.apply(Command::Undo) {
        depth += 1;
    }
    assert!(depth <= HISTORY_LIMIT);
}

#[test]
fn soak_large_selection_moves_keep_history_within_budget() {
    // Arrange: rows of wobbly strokes (fixture `strokes-1000`, built by
    // `tests/common/scenes.rs`), all selected.
    let mut ed = load_fixture("strokes-1000");
    assert_eq!(ed.document().len(), LARGE_STROKES);
    ed.apply(Command::SetTool(Tool::Select));
    ed.apply(Command::SelectAll);
    let first = v(20.0, 20.0);
    let moved = v(30.0, 20.0);

    // Act: drag the selection by its first stroke, back and forth.
    let mut peak = 0;
    for k in 0..LARGE_MOVES {
        let (from, to) = if k % 2 == 0 {
            (first, moved)
        } else {
            (moved, first)
        };
        straight_drag(&mut ed, from, to);
        peak = peak.max(ed.history_bytes());
    }

    // Assert: the budget was reached and held, and the last move undoes.
    assert!(peak > HISTORY_BYTES / 2, "budget never approached: {peak}");
    assert!(peak <= HISTORY_BYTES, "{peak} > {HISTORY_BYTES}");
    let revision = ed.document_revision();
    assert!(ed.apply(Command::Undo));
    assert!(ed.document_revision() > revision);
    assert_eq!(ed.document().len(), LARGE_STROKES);
}

#[test]
#[ignore = "long: run in release, alone (see the module docs)"]
fn soak_long_session_keeps_resident_memory_flat() {
    // Arrange
    let Some(_) = rss_kib() else {
        eprintln!("no /proc/self/statm: skipped");
        return;
    };
    let mut ed = editor();
    for k in 0..WARMUP_CYCLES {
        cycle(&mut ed, k);
    }
    let baseline = rss_kib().unwrap_or(0);

    // Act
    for k in WARMUP_CYCLES..LONG_CYCLES {
        cycle(&mut ed, k);
    }
    let end = rss_kib().unwrap_or(u64::MAX);

    // Assert
    eprintln!("resident memory: {baseline} KiB after warm-up, {end} KiB at the end");
    assert!(
        end.saturating_sub(baseline) < RSS_GROWTH_KIB,
        "grew from {baseline} KiB to {end} KiB"
    );
    assert!(ed.document().is_empty());
}
