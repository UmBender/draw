//! Editor states built from input events: the sources of the committed
//! fixtures in `tests/snapshots/` (T74). `tests/snapshots.rs` rebuilds
//! each one and compares it with its fixture, so a fixture always means
//! "what these events produce".

use draw::core::command::{Command, Tool};
use draw::core::editor::Editor;
use draw::core::geom::Vec2;
use draw::core::history::HISTORY_LIMIT;
use draw::core::input::{InputEvent, Key, Modifiers, PointerButton};

const NONE: Modifiers = Modifiers::NONE;

/// Viewport of a 1080p screen, in pixels.
pub const VIEWPORT: Vec2 = Vec2::new(1920.0, 1080.0);

/// Pointer moves in one wobbly drag.
pub const DRAG_STEPS: usize = 20;

/// Strokes of [`strokes_1000`].
pub const LARGE_STROKES: usize = 1_000;

/// Text blocks of [`text_500`].
pub const TEXT_BLOCKS: usize = 500;

/// Characters per block of [`text_500`].
pub const TEXT_CHARS: usize = 20;

pub fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// A 1080p editor.
pub fn editor() -> Editor {
    let mut ed = Editor::new();
    ed.handle(InputEvent::Resize { size: VIEWPORT });
    ed
}

pub fn press(ed: &mut Editor, pos: Vec2) {
    ed.handle(InputEvent::PointerMove { pos, mods: NONE });
    ed.handle(InputEvent::PointerDown {
        pos,
        button: PointerButton::Left,
        mods: NONE,
    });
}

pub fn release(ed: &mut Editor, pos: Vec2) {
    ed.handle(InputEvent::PointerUp {
        pos,
        button: PointerButton::Left,
        mods: NONE,
    });
}

/// Left press and release at `pos`.
pub fn click(ed: &mut Editor, pos: Vec2) {
    press(ed, pos);
    release(ed, pos);
}

/// Left drag from `a` to `b` with a small wobble, so pen strokes keep
/// their points through smoothing (as in `tests/soak.rs`).
pub fn drag(ed: &mut Editor, a: Vec2, b: Vec2) {
    press(ed, a);
    for i in 1..=DRAG_STEPS {
        let t = i as f32 / DRAG_STEPS as f32;
        let wobble = (t * 9.0).sin() * 5.0;
        let pos = v(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t + wobble);
        ed.handle(InputEvent::PointerMove { pos, mods: NONE });
    }
    release(ed, b);
}

/// Straight left drag from `a` to `b`.
pub fn straight_drag(ed: &mut Editor, a: Vec2, b: Vec2) {
    press(ed, a);
    ed.handle(InputEvent::PointerMove { pos: b, mods: NONE });
    release(ed, b);
}

pub fn type_text(ed: &mut Editor, text: &str) {
    for ch in text.chars() {
        ed.handle(InputEvent::Char { ch, mods: NONE });
    }
}

pub fn key(ed: &mut Editor, key: Key) {
    ed.handle(InputEvent::KeyDown { key, mods: NONE });
    ed.handle(InputEvent::KeyUp { key, mods: NONE });
}

/// `empty`: a new editor.
pub fn empty() -> Editor {
    Editor::new()
}

/// `pen-mid-stroke`: a pen stroke still being drawn, pressure on.
pub fn pen_mid_stroke() -> Editor {
    let mut ed = editor();
    ed.handle(InputEvent::Pressure {
        pressure: Some(0.6),
    });
    press(&mut ed, v(100.0, 100.0));
    for i in 1..=12 {
        let t = i as f32;
        ed.handle(InputEvent::Pressure {
            pressure: Some(0.3 + t / 30.0),
        });
        ed.handle(InputEvent::PointerMove {
            pos: v(100.0 + t * 15.0, 100.0 + (t * 0.7).sin() * 20.0),
            mods: NONE,
        });
    }
    ed
}

/// `select-mid-drag`: three shapes, two selected, a move in progress.
pub fn select_mid_drag() -> Editor {
    let mut ed = editor();
    ed.apply(Command::SetTool(Tool::Rect));
    straight_drag(&mut ed, v(100.0, 100.0), v(300.0, 200.0));
    ed.apply(Command::SetTool(Tool::Ellipse));
    straight_drag(&mut ed, v(400.0, 100.0), v(550.0, 250.0));
    ed.apply(Command::SetTool(Tool::Line));
    straight_drag(&mut ed, v(100.0, 400.0), v(600.0, 450.0));
    ed.apply(Command::SetTool(Tool::Select));
    straight_drag(&mut ed, v(50.0, 50.0), v(600.0, 300.0));
    press(&mut ed, v(200.0, 100.0));
    for i in 1..=5 {
        let t = i as f32 * 12.0;
        ed.handle(InputEvent::PointerMove {
            pos: v(200.0 + t, 100.0 + t / 2.0),
            mods: NONE,
        });
    }
    ed
}

/// `text-mid-edit`: a committed block, and an open one with the caret
/// inside its formula.
pub fn text_mid_edit() -> Editor {
    let mut ed = editor();
    ed.apply(Command::SetTool(Tool::Text));
    click(&mut ed, v(100.0, 100.0));
    type_text(&mut ed, "dp[i] = min(dp[j])");
    key(&mut ed, Key::Escape);
    click(&mut ed, v(100.0, 300.0));
    type_text(&mut ed, "let $\\frac{a}{b} + x_i^2$ be");
    for _ in 0..6 {
        key(&mut ed, Key::ArrowLeft);
    }
    ed
}

/// `history-at-limit`: [`HISTORY_LIMIT`] + 5 lines drawn, then 3 undone,
/// so the undo stack is full and the redo stack is not empty.
pub fn history_at_limit() -> Editor {
    let mut ed = editor();
    ed.apply(Command::SetTool(Tool::Line));
    for i in 0..HISTORY_LIMIT + 5 {
        let x = (i % 40) as f32 * 40.0 + 20.0;
        let y = (i / 40) as f32 * 60.0 + 20.0;
        straight_drag(&mut ed, v(x, y), v(x + 30.0, y + 40.0));
    }
    for _ in 0..3 {
        ed.apply(Command::Undo);
    }
    ed
}

/// `strokes-1000`: rows of wobbly pen strokes, the large-selection scene
/// of `tests/soak.rs` before it selects.
pub fn strokes_1000() -> Editor {
    let mut ed = editor();
    ed.apply(Command::SetTool(Tool::Pen));
    for i in 0..LARGE_STROKES {
        let x = (i / 100) as f32 * 180.0 + 20.0;
        let y = (i % 100) as f32 * 9.0 + 20.0;
        drag(&mut ed, v(x, y), v(x + 150.0, y + 5.0));
    }
    ed
}

/// Deterministic xorshift generator (as in `tests/perf.rs`).
pub struct Rng(pub u64);

impl Rng {
    /// The next value in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    /// The next value in `[lo, hi)`.
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }
}

/// `text-500`: the typing scene of `tests/perf.rs`: 500 blocks of 20
/// characters typed with the Text tool, then an open block.
pub fn text_500() -> Editor {
    let mut rng = Rng(0x51_7cc1_b727_220a);
    let mut ed = editor();
    ed.apply(Command::SetTool(Tool::Text));
    for i in 0..TEXT_BLOCKS {
        let origin = v(rng.range(-200.0, 2_400.0), rng.range(-200.0, 1_400.0));
        let mut text: String = (0..TEXT_CHARS - 2)
            .map(|k| char::from(b'a' + ((i + k) % 26) as u8))
            .collect();
        text.insert(9, '\n');
        text.push('∑');
        let pos = ed.camera().world_to_screen(origin);
        ed.handle(InputEvent::PointerDown {
            pos,
            button: PointerButton::Left,
            mods: NONE,
        });
        release(&mut ed, pos);
        type_text(&mut ed, &text);
        ed.handle(InputEvent::KeyDown {
            key: Key::Escape,
            mods: NONE,
        });
    }
    let open = v(960.0, 1_060.0);
    ed.handle(InputEvent::PointerDown {
        pos: open,
        button: PointerButton::Left,
        mods: NONE,
    });
    release(&mut ed, open);
    ed
}
