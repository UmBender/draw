//! Model-based tests of the control automaton (T138).
//!
//! `machine_matches_the_model` drives the machine and a tiny reference model
//! with the same generated actions and compares what the user can see.
//! `no_state_traps_the_user` explores every reachable state up to a depth and
//! checks that each one can get back to `Idle`.
//!
//! `cargo test` runs a quick pass; `PROPTEST_CASES=20000` runs the long one.

use std::collections::HashSet;

use draw_core::core::command::Tool;
use draw_core::core::geom::Vec2;
use draw_core::core::input::{Key, Modifiers, PointerButton};
use draw_core::machine::{Input, Interaction, Machine, step};
use proptest::prelude::*;

const PAGES: usize = 6;

const CTRL: Modifiers = Modifiers {
    shift: false,
    ctrl: true,
    alt: false,
};
const ALT: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: true,
};

/// What a user does, at the level the model understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    /// A bare tool key.
    SetTool(Tool),
    /// `Alt+digit`.
    GoToPage(usize),
    /// Rectangle tool, then one drag of the given size class.
    DrawRect(u8),
    /// `Ctrl+A`.
    SelectAll,
    /// `Delete`.
    Delete,
    /// `Ctrl+Z`.
    Undo,
    /// `Escape`.
    Escape,
    /// Text tool, press at spot `n`, type `x`, `Escape` to commit (T153): a
    /// new block, or an edit of the block already at that spot.
    WriteText(u8),
    /// Select tool, marquee over the whole page (T153).
    MarqueeAll,
    /// Select tool, marquee over empty space (T153).
    MarqueeEmpty,
}

/// Spots a text block can be written at; far from the rectangles.
const TEXT_SPOTS: u8 = 3;

const TOOL_KEYS: [(Tool, Key); 11] = [
    (Tool::Pen, Key::D),
    (Tool::Line, Key::W),
    (Tool::Arrow, Key::A),
    (Tool::Rect, Key::R),
    (Tool::Ellipse, Key::C),
    (Tool::Eraser, Key::E),
    (Tool::Bucket, Key::B),
    (Tool::Select, Key::V),
    (Tool::Hand, Key::H),
    (Tool::Grid, Key::G),
    (Tool::Text, Key::T),
];

const DIGITS: [Key; PAGES] = [
    Key::Digit1,
    Key::Digit2,
    Key::Digit3,
    Key::Digit4,
    Key::Digit5,
    Key::Digit6,
];

fn key_of(tool: Tool) -> Key {
    TOOL_KEYS
        .iter()
        .find(|(t, _)| *t == tool)
        .map_or(Key::V, |(_, k)| *k)
}

fn key(key: Key, mods: Modifiers) -> Input {
    Input::KeyDown { key, mods }
}

fn pointer(f: impl Fn(Vec2) -> Input, x: f32, y: f32) -> Input {
    f(Vec2::new(x, y))
}

/// The inputs an [`Action`] stands for.
fn inputs(action: Action) -> Vec<Input> {
    match action {
        Action::SetTool(tool) => vec![key(key_of(tool), Modifiers::NONE)],
        Action::GoToPage(page) => vec![key(DIGITS[page % PAGES], ALT)],
        Action::DrawRect(size) => {
            let far = 60.0 + f32::from(size) * 40.0;
            vec![
                key(Key::R, Modifiers::NONE),
                pointer(
                    |pos| Input::PointerDown {
                        pos,
                        button: PointerButton::Left,
                        mods: Modifiers::NONE,
                    },
                    300.0,
                    300.0,
                ),
                pointer(
                    |pos| Input::PointerMove {
                        pos,
                        mods: Modifiers::NONE,
                    },
                    300.0 + far / 2.0,
                    300.0 + far / 2.0,
                ),
                pointer(
                    |pos| Input::PointerUp {
                        pos,
                        button: PointerButton::Left,
                        mods: Modifiers::NONE,
                    },
                    300.0 + far,
                    300.0 + far,
                ),
            ]
        }
        Action::SelectAll => vec![key(Key::A, CTRL)],
        Action::Delete => vec![key(Key::Delete, Modifiers::NONE)],
        Action::Undo => vec![key(Key::Z, CTRL)],
        Action::Escape => vec![key(Key::Escape, Modifiers::NONE)],
        Action::WriteText(spot) => {
            let x = 800.0 + 150.0 * f32::from(spot % TEXT_SPOTS);
            vec![
                key(Key::T, Modifiers::NONE),
                left_down(x, 40.0),
                left_up(x, 40.0),
                Input::Char {
                    ch: 'x',
                    mods: Modifiers::NONE,
                },
                key(Key::Escape, Modifiers::NONE),
            ]
        }
        Action::MarqueeAll => marquee(5.0, 5.0, 3000.0, 3000.0),
        Action::MarqueeEmpty => marquee(2500.0, 2500.0, 2600.0, 2600.0),
    }
}

fn left_down(x: f32, y: f32) -> Input {
    pointer(
        |pos| Input::PointerDown {
            pos,
            button: PointerButton::Left,
            mods: Modifiers::NONE,
        },
        x,
        y,
    )
}

fn left_up(x: f32, y: f32) -> Input {
    pointer(
        |pos| Input::PointerUp {
            pos,
            button: PointerButton::Left,
            mods: Modifiers::NONE,
        },
        x,
        y,
    )
}

/// Select tool, a press at `(x0, y0)`, a move halfway and a release at
/// `(x1, y1)`.
fn marquee(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Input> {
    vec![
        key(Key::V, Modifiers::NONE),
        left_down(x0, y0),
        pointer(
            |pos| Input::PointerMove {
                pos,
                mods: Modifiers::NONE,
            },
            (x0 + x1) / 2.0,
            (y0 + y1) / 2.0,
        ),
        left_up(x1, y1),
    ]
}

fn run(machine: Machine, action: Action) -> Machine {
    inputs(action)
        .into_iter()
        .fold(machine, |machine, input| step(machine, input).0)
}

/// What the user can see, in the model's terms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Observed {
    tool: Tool,
    page: usize,
    selection: usize,
    can_undo: bool,
}

fn observe(machine: &Machine) -> Observed {
    let editor = machine.editor();
    Observed {
        tool: editor.tool(),
        page: editor.page(),
        selection: editor.selection().len(),
        can_undo: editor.can_undo(),
    }
}

/// The reference model: a few dozen lines, no editor inside.
#[derive(Debug, Clone)]
struct Model {
    tool: Tool,
    page: usize,
    /// Shape ids on each page.
    shapes: [Vec<u32>; PAGES],
    /// Shape lists before each undoable change, per page.
    history: [Vec<Vec<u32>>; PAGES],
    /// Selected shape ids per page.
    selection: [Vec<u32>; PAGES],
    /// Text blocks per page: spot and shape id.
    texts: [Vec<(u8, u32)>; PAGES],
    /// Next fresh shape id.
    next: u32,
}

impl Model {
    fn new() -> Self {
        Self {
            tool: Tool::Pen,
            page: 0,
            shapes: Default::default(),
            history: Default::default(),
            selection: Default::default(),
            texts: Default::default(),
            next: 0,
        }
    }

    fn apply(&mut self, action: Action) {
        let p = self.page;
        match action {
            Action::SetTool(tool) => self.tool = tool,
            Action::GoToPage(page) => self.page = page % PAGES,
            Action::DrawRect(_) => {
                self.tool = Tool::Rect;
                self.history[p].push(self.shapes[p].clone());
                self.shapes[p].push(self.next);
                self.next += 1;
            }
            Action::SelectAll => self.selection[p].clone_from(&self.shapes[p]),
            Action::Delete => {
                if !self.selection[p].is_empty() {
                    self.history[p].push(self.shapes[p].clone());
                    let gone = std::mem::take(&mut self.selection[p]);
                    self.shapes[p].retain(|id| !gone.contains(id));
                }
            }
            Action::Undo => {
                if let Some(before) = self.history[p].pop() {
                    self.shapes[p] = before;
                    // The selection keeps only the shapes that still exist.
                    let alive = &self.shapes[p];
                    self.selection[p].retain(|id| alive.contains(id));
                }
            }
            Action::Escape => self.selection[p].clear(),
            Action::WriteText(spot) => {
                let spot = spot % TEXT_SPOTS;
                self.tool = Tool::Text;
                self.history[p].push(self.shapes[p].clone());
                let alive = &self.shapes[p];
                let there = self.texts[p]
                    .iter()
                    .any(|(at, id)| *at == spot && alive.contains(id));
                if !there {
                    self.shapes[p].push(self.next);
                    self.texts[p].push((spot, self.next));
                    self.next += 1;
                }
            }
            Action::MarqueeAll => {
                self.tool = Tool::Select;
                self.selection[p].clone_from(&self.shapes[p]);
            }
            Action::MarqueeEmpty => {
                self.tool = Tool::Select;
                self.selection[p].clear();
            }
        }
    }

    fn observed(&self) -> Observed {
        Observed {
            tool: self.tool,
            page: self.page,
            selection: self.selection[self.page].len(),
            can_undo: !self.history[self.page].is_empty(),
        }
    }
}

fn arb_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        4 => proptest::sample::select(TOOL_KEYS.to_vec()).prop_map(|(t, _)| Action::SetTool(t)),
        2 => (0..PAGES).prop_map(Action::GoToPage),
        4 => (0u8..4).prop_map(Action::DrawRect),
        2 => Just(Action::SelectAll),
        2 => Just(Action::Delete),
        2 => Just(Action::Undo),
        1 => Just(Action::Escape),
        3 => (0..TEXT_SPOTS).prop_map(Action::WriteText),
        1 => Just(Action::MarqueeAll),
        1 => Just(Action::MarqueeEmpty),
    ]
}

proptest! {
    /// AC-1: the machine and the model agree after every action.
    #[test]
    fn machine_matches_the_model(actions in proptest::collection::vec(arb_action(), 0..40)) {
        let mut machine = Machine::new();
        let mut model = Model::new();
        for action in actions {
            machine = run(machine, action);
            model.apply(action);
            prop_assert_eq!(observe(&machine), model.observed(), "after {:?}", action);
            prop_assert_eq!(machine.interaction(), Interaction::Idle);
        }
    }
}

/// The alphabet the exploration uses: everything above plus the moves that
/// leave something open (a press without release).
fn alphabet() -> Vec<Vec<Input>> {
    let mut all: Vec<Vec<Input>> = TOOL_KEYS
        .iter()
        .map(|(_, k)| vec![key(*k, Modifiers::NONE)])
        .collect();
    all.push(vec![key(Key::Digit2, ALT)]);
    all.push(vec![key(Key::A, CTRL)]);
    all.push(vec![key(Key::Delete, Modifiers::NONE)]);
    all.push(vec![key(Key::Z, CTRL)]);
    all.push(vec![key(Key::Escape, Modifiers::NONE)]);
    all.push(vec![key(
        Key::P,
        Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
        },
    )]);
    all.push(vec![Input::Char {
        ch: '?',
        mods: Modifiers::NONE,
    }]);
    all.push(vec![pointer(
        |pos| Input::PointerDown {
            pos,
            button: PointerButton::Left,
            mods: Modifiers::NONE,
        },
        200.0,
        200.0,
    )]);
    all.push(vec![pointer(
        |pos| Input::PointerMove {
            pos,
            mods: Modifiers::NONE,
        },
        260.0,
        240.0,
    )]);
    // T153: text opened and typed into, and a select marquee left open.
    all.push(vec![
        key(Key::T, Modifiers::NONE),
        left_down(200.0, 200.0),
        left_up(200.0, 200.0),
        Input::Char {
            ch: 'q',
            mods: Modifiers::NONE,
        },
    ]);
    all.push(vec![key(Key::V, Modifiers::NONE), left_down(900.0, 900.0)]);
    all
}

/// The coarse state used to prune the search.
fn abstract_key(machine: &Machine) -> (Interaction, Observed, usize) {
    (
        machine.interaction(),
        observe(machine),
        machine.editor().document().len(),
    )
}

/// Gets back to `Idle` the way a user would: release, then `Escape`.
fn recover(machine: Machine) -> Machine {
    let mut machine = machine;
    machine = step(
        machine,
        pointer(
            |pos| Input::PointerUp {
                pos,
                button: PointerButton::Left,
                mods: Modifiers::NONE,
            },
            260.0,
            240.0,
        ),
    )
    .0;
    for _ in 0..3 {
        if machine.interaction() == Interaction::Idle {
            break;
        }
        machine = step(machine, key(Key::Escape, Modifiers::NONE)).0;
    }
    machine
}

/// Depth of the exploration.
const DEPTH: usize = 3;

/// AC-2: every state reachable within [`DEPTH`] actions can return to `Idle`.
#[test]
fn no_state_traps_the_user() {
    let alphabet = alphabet();
    let mut seen: HashSet<_> = HashSet::new();
    let mut frontier = vec![Machine::new()];
    seen.insert(abstract_key(&frontier[0]));
    let mut explored = 0usize;
    for _ in 0..DEPTH {
        let mut next = Vec::new();
        for machine in &frontier {
            for action in &alphabet {
                let after = action
                    .iter()
                    .fold(machine.clone(), |m, input| step(m, input.clone()).0);
                let back = recover(after.clone());
                assert_eq!(
                    back.interaction(),
                    Interaction::Idle,
                    "stuck after {action:?} from {:?}",
                    machine.interaction()
                );
                if seen.insert(abstract_key(&after)) {
                    next.push(after);
                }
                explored += 1;
            }
        }
        frontier = next;
    }
    assert!(
        explored > 100,
        "the exploration reached too little: {explored}"
    );
}
