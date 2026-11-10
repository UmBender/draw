//! Fuzz harness (ADR-0009): drives the headless `Editor` with random input
//! sessions and checks the invariants listed in `docs/architecture/Architecture.md`.
//!
//! `cargo test` runs a quick pass; `scripts/check.sh --fuzz` runs 20 000 cases.
//! Shrunk failures are stored in `tests/fuzz.proptest-regressions` and replayed
//! on every run.

use std::collections::HashSet;

use draw::core::camera::{ZOOM_MAX, ZOOM_MIN};
use draw::core::command::{Command, Tool};
use draw::core::document::ShapeId;
use draw::core::editor::Editor;
use draw::core::geom::Vec2;
use draw::core::history::HISTORY_LIMIT;
use draw::core::input::{InputEvent, Key, Modifiers, PointerButton};
use draw::core::keymap::BINDINGS;
use draw::core::pressure::{MAX_WIDTH_FACTOR, MIN_WIDTH_FACTOR};
use draw::core::shape::{GRID_MAX_CELLS, Shape, TextSlot};
use draw::core::smoothing::{Smoother, SmoothingParams, simplify_rdp};
use draw::core::snapshot::Snapshot;
use draw::core::tap::PEN_TAP_SLOP_PX;
use draw::core::tex;
use draw::core::text_layout::typeset;
use proptest::prelude::*;
use proptest::sample::select;
use proptest::strategy::ValueTree;
use proptest::test_runner::{FileFailurePersistence, TestRunner};

/// Upper bound on events in one session.
const MAX_SESSION_LEN: usize = 500;

/// Every `Key`, in declaration order.
const ALL_KEYS: [Key; 50] = [
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
    Key::G,
    Key::H,
    Key::I,
    Key::J,
    Key::K,
    Key::L,
    Key::M,
    Key::N,
    Key::O,
    Key::P,
    Key::Q,
    Key::R,
    Key::S,
    Key::T,
    Key::U,
    Key::V,
    Key::W,
    Key::X,
    Key::Y,
    Key::Z,
    Key::Digit0,
    Key::Digit1,
    Key::Digit2,
    Key::Digit3,
    Key::Digit4,
    Key::Digit5,
    Key::Digit6,
    Key::Digit7,
    Key::Digit8,
    Key::Digit9,
    Key::Delete,
    Key::Backspace,
    Key::Escape,
    Key::Space,
    Key::Tab,
    Key::BracketLeft,
    Key::BracketRight,
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::ArrowUp,
    Key::ArrowDown,
    Key::Enter,
    Key::Home,
    Key::End,
];

/// Keys that edit a text block (T29).
const TEXT_KEYS: [Key; 10] = [
    Key::Enter,
    Key::Backspace,
    Key::Delete,
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::ArrowUp,
    Key::ArrowDown,
    Key::Home,
    Key::End,
    Key::Tab,
];

/// The arrow keys: grid columns and rows.
const ARROW_KEYS: [Key; 4] = [
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::ArrowUp,
    Key::ArrowDown,
];

/// Every `PointerButton`.
const ALL_BUTTONS: [PointerButton; 3] = [
    PointerButton::Left,
    PointerButton::Middle,
    PointerButton::Right,
];

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// A coordinate: mostly on-screen, sometimes `0`, `NaN`, `±∞`, huge or any bit
/// pattern.
fn arb_coord() -> impl Strategy<Value = f32> {
    prop_oneof![
        12 => -100.0f32..2_100.0,
        1 => Just(0.0f32),
        1 => Just(f32::NAN),
        1 => Just(f32::INFINITY),
        1 => Just(f32::NEG_INFINITY),
        1 => prop_oneof![1e30f32..f32::MAX, -f32::MAX..-1e30f32],
        1 => any::<f32>(),
    ]
}

/// A screen position built from two [`arb_coord`]s.
fn arb_pos() -> impl Strategy<Value = Vec2> {
    (arb_coord(), arb_coord()).prop_map(|(x, y)| Vec2::new(x, y))
}

/// A finite, on-screen position.
fn arb_screen_pos() -> impl Strategy<Value = Vec2> {
    (0.0f32..2_000.0, 0.0f32..1_200.0).prop_map(|(x, y)| Vec2::new(x, y))
}

/// Any of the 8 modifier combinations.
fn arb_mods() -> impl Strategy<Value = Modifiers> {
    any::<(bool, bool, bool)>().prop_map(|(shift, ctrl, alt)| Modifiers { shift, ctrl, alt })
}

/// Any pointer button.
fn arb_button() -> impl Strategy<Value = PointerButton> {
    select(ALL_BUTTONS.to_vec())
}

/// Any key.
fn arb_key() -> impl Strategy<Value = Key> {
    select(ALL_KEYS.to_vec())
}

/// A typed character: mostly letters, sometimes line breaks, spaces,
/// multi-byte or control characters, or any `char`.
fn arb_char() -> impl Strategy<Value = char> {
    prop_oneof![
        8 => proptest::char::range('a', 'z'),
        1 => Just('\n'),
        1 => Just(' '),
        1 => select(vec!['β', '∑', '≤', 'é', '𝔸']),
        1 => select(vec!['\u{8}', '\r', '\t', '\u{1b}', '\u{7f}']),
        1 => any::<char>(),
    ]
}

/// Formula-ish source (T30, T33, T36, T39, T61, T63, T64, T54, T62, T56,
/// T69): TeX tokens, `$`s, braces and any `char`.
fn arb_formula_text() -> impl Strategy<Value = String> {
    let token = prop_oneof![
        6 => select(vec![
            "$", "\\$", "{", "}", "^", "_", "\\", " ", "x", "i", "2", "-", "+", "=", "(",
            ")", ",", "|", "\\frac", "\\sqrt", "\\binom", "\\text", "\\pmod",
            "\\alpha", "\\Omega", "\\sum", "\\prod", "\\min", "\\lim", "\\log",
            "\\le", "\\cdot", "\\lfloor", "\\{", "\\}", "\\foo",
            // T33: script capitals and italic overhangs at both ends.
            "\\mathcal", "A", "f", "j", "\\varphi",
            // T36: integrals with scripts beside them, physics symbols.
            "\\int", "\\oint", "\\iiint", "\\int_0^1", "\\,", "\\partial", "\\hbar",
            "\\langle", "\\rangle", "\\otimes",
            // T39: accents over letters, groups and scripts.
            "\\vec", "\\hat", "\\tilde", "\\dot", "\\ddot", "\\bar", "\\overline",
            "\\vec{F}_x",
            // T61: Hebrew letters, with a script.
            "\\aleph", "\\beth", "\\gimel", "\\daleth", "\\aleph_0",
            // T63: trigonometric and hyperbolic functions, with a power.
            "\\sin", "\\cos", "\\tan", "\\cot", "\\sec", "\\csc", "\\arcsin", "\\arccos",
            "\\arctan", "\\sinh", "\\cosh", "\\tanh", "\\coth", "\\arccot", "\\arcsec",
            "\\arccsc", "\\sech", "\\csch", "\\sin^2\\theta",
            // T64: named operators with limits, operator words and
            // \operatorname with and without star; its bodies and lead
            // come from the " ", "{", "x" and "}" tokens.
            "\\inf", "\\sup", "\\liminf", "\\limsup", "\\det", "\\Pr", "\\gcd", "\\exp",
            "\\lg", "\\deg", "\\dim", "\\ker", "\\hom", "\\arg", "\\limsup_n",
            "\\operatorname", "\\operatorname*",
            // T54: math alphabets, nested through the "{" and "}" tokens.
            "\\mathrm", "\\mathit", "\\mathnormal", "\\mathrm{d}", "\\mathnormal\\Gamma",
            // T62: the remaining Greek letters, italic and upright capitals.
            "\\varkappa", "\\varGamma", "\\Alpha", "\\mathnormal\\Alpha",
            // T56: bold alphabets, and \boldmath left unknown.
            "\\mathbf", "\\boldsymbol", "\\bm", "\\boldmath", "\\mathbf\\Gamma",
            "\\bm{\\alpha}",
            // T57: blackboard bold, and the \R shortcut left unknown.
            "\\mathbb", "\\mathds", "\\mathbb{R}", "\\mathds{1}", "\\R",
            // T58: fraktur, a hole and a nested bold.
            "\\mathfrak", "\\mathfrak{g}", "\\mathfrak{R}", "\\mathbf\\mathfrak",
            // T59: script capitals, a hole and lowercase (no script glyph).
            "\\mathscr", "\\mathscr{L}", "\\mathscr{l}", "\\mathcal\\mathscr",
            // T60: sans-serif and typewriter, bold sans and bold over mono.
            "\\mathsf", "\\mathtt", "\\mathsf{G}", "\\mathtt{x1}", "\\mathbf\\mathsf",
            "\\bm{\\mathsf{x}}", "\\mathsf\\mathbf", "\\mathbf\\mathtt",
            // T65: set and logic symbols, padded implications and bars.
            "\\setminus", "\\complement", "\\varnothing", "\\supset", "\\subseteq",
            "\\supseteq", "\\subsetneq", "\\supsetneq", "\\nsubseteq", "\\ni", "\\sqcup",
            "\\sqcap", "\\uplus", "\\lnot", "\\veebar", "\\barwedge", "\\top", "\\bot",
            "\\nexists", "\\implies", "\\impliedby", "\\vdash", "\\models", "\\lvert",
            "\\rvert", "\\lVert", "\\rVert", "x^{\\implies}",
            // T66: big set and logic operators, with limits.
            "\\bigcup", "\\bigcap", "\\bigwedge", "\\bigvee", "\\bigoplus", "\\bigotimes",
            "\\bigodot", "\\biguplus", "\\bigsqcup", "\\coprod", "\\bigcup_{i=1}^{n}",
            "\\coprod_",
            // T89: growing brackets, paired, unmatched and without a delimiter.
            "\\left", "\\right", "\\left(", "\\right)", "\\left.", "\\right.", "\\left[",
            "\\right]", "\\left\\{", "\\right\\}", "\\left|", "\\right\\|",
            "\\left\\langle", "\\right\\rangle", "\\left(\\frac12\\right)^2",
            // T90: environments, paired, unknown and without an end.
            "\\begin", "\\end", "\\begin{matrix}", "\\end{matrix}", "\\begin{pmatrix}",
            "\\end{pmatrix}", "\\begin{bmatrix}", "\\end{bmatrix}", "\\begin{vmatrix}",
            "\\end{vmatrix}", "\\begin{cases}", "\\end{cases}", "\\begin{foo}", "\\end{foo}",
            "&", "\\\\", "\\begin{pmatrix}a&b\\\\c&d\\end{pmatrix}",
            // T99: spacing commands, a negative run, a spaced operator.
            "\\:", "\\;", "\\!", "\\!\\!\\!\\!", "\\quad", "\\qquad", "\\ ", "~",
            "a\\;+\\;b", "x^{\\!}",
            // T103: extensible arrows, labels below in brackets, nested.
            "\\xrightarrow", "\\xleftarrow", "\\xleftrightarrow", "\\xRightarrow",
            "\\xLeftarrow", "\\xLeftrightarrow", "\\xmapsto", "\\xhookrightarrow", "[", "]",
            "\\xrightarrow[n\\to\\infty]{f_n}", "\\xrightarrow{\\xrightarrow{a}}",
            "\\begin{cases}1&x\\\\2\\end{cases}",
            // T71: infix fractions split their group, combinatorics.
            "\\choose", "\\brack", "\\brace", "\\over", "\\atop", "{n \\choose k}",
            "\\tbinom", "\\dbinom", "\\multiset", "\\multiset{n}{k}", "\\underline",
            "x^{\\underline{k}}", "\\nmid", "\\#",
            // T67: algebra and group theory symbols and operator names.
            "\\lhd", "\\rhd", "\\unlhd", "\\unrhd", "\\triangleleft", "\\triangleright",
            "\\trianglelefteq", "\\trianglerighteq", "\\ntriangleleft", "\\ntriangleright",
            "\\rtimes", "\\ltimes", "\\wr", "\\odot", "\\ominus", "\\oslash", "\\boxplus",
            "\\boxtimes", "\\amalg", "\\diamond", "\\Hom", "\\End", "\\Aut", "\\Inn",
            "\\Gal", "\\Ker", "\\im", "\\coker", "\\id", "\\sgn", "\\tr", "\\rank",
            "\\Hom(A, B)",
            // T68: category theory arrows, `\colon` and operator names.
            "\\colon", "\\hookrightarrow", "\\hookleftarrow", "\\twoheadrightarrow",
            "\\twoheadleftarrow", "\\rightarrowtail", "\\leftarrowtail", "\\longrightarrow",
            "\\longleftarrow", "\\longleftrightarrow", "\\Longrightarrow", "\\Longleftarrow",
            "\\Longleftrightarrow", "\\longmapsto", "\\rightrightarrows", "\\leftleftarrows",
            "\\leftrightarrows", "\\rightleftarrows", "\\nearrow", "\\searrow", "\\swarrow",
            "\\nwarrow", "\\Uparrow", "\\Downarrow", "\\Updownarrow", "\\updownarrow",
            "\\dashv", "\\op", "\\Ob", "\\Mor", "f\\colon A \\to B", "\\mathcal{C}^{\\op}",
            // T69: classical physics symbols, \dddot and the expanded
            // physics shortcuts with optional arguments.
            "\\dddot", "\\Box", "\\square", "\\triangle", "\\angle", "\\ohm", "\\micro",
            "\\grad", "\\curl", "\\dd", "\\dv", "\\pdv", "\\abs", "\\norm", "\\order",
            "\\vb", "\\vu", "\\dd{x}", "\\dv{f}{x}", "\\pdv{x}", "\\abs{\\vb{F}}",
            "\\curl\\vb{A}", "\\order{h^2}", "#", "#1",
            // T70: quantum notation, Dirac brackets and commutators.
            "\\bra", "\\ket", "\\braket", "\\ketbra", "\\expval", "\\mel", "\\comm",
            "\\acomm", "\\dag", "\\Tr", "\\braket{\\phi}{\\psi}", "\\mel{n}{H}{m}",
            "\\comm{A}{B}", "\\ket{0}\\otimes\\ket{1}", "A^\\dag", "\\expval{H}{\\psi}",
        ])
        .prop_map(str::to_owned),
        1 => arb_char().prop_map(String::from),
    ];
    prop::collection::vec(token, 0..24).prop_map(|tokens| tokens.concat())
}

/// A formula typed into a block: `$…$` around formula-ish source, or
/// the source alone.
fn arb_formula() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => arb_formula_text().prop_map(|src| format!("${src}$")),
        1 => arb_formula_text(),
    ]
}

/// A scroll delta: usually a few notches, sometimes extreme or non-finite.
fn arb_scroll_delta() -> impl Strategy<Value = f32> {
    prop_oneof![
        6 => -3.0f32..3.0,
        1 => arb_coord(),
    ]
}

/// Any single input event, every variant reachable.
fn arb_event() -> impl Strategy<Value = InputEvent> {
    prop_oneof![
        (arb_pos(), arb_button(), arb_mods())
            .prop_map(|(pos, button, mods)| InputEvent::PointerDown { pos, button, mods }),
        (arb_pos(), arb_mods()).prop_map(|(pos, mods)| InputEvent::PointerMove { pos, mods }),
        (arb_pos(), arb_button(), arb_mods())
            .prop_map(|(pos, button, mods)| InputEvent::PointerUp { pos, button, mods }),
        (arb_pos(), arb_scroll_delta()).prop_map(|(pos, delta)| InputEvent::Scroll { pos, delta }),
        (arb_key(), arb_mods()).prop_map(|(key, mods)| InputEvent::KeyDown { key, mods }),
        (arb_key(), arb_mods()).prop_map(|(key, mods)| InputEvent::KeyUp { key, mods }),
        arb_pos().prop_map(|size| InputEvent::Resize { size }),
        (arb_char(), arb_mods()).prop_map(|(ch, mods)| InputEvent::Char { ch, mods }),
        arb_pressure().prop_map(|pressure| InputEvent::Pressure { pressure }),
        arb_paste_text().prop_map(InputEvent::Paste),
        arb_paste_text().prop_map(InputEvent::PasteGrid),
    ]
}

/// Clipboard text: short, with line breaks, tabs, control characters and
/// formula source (T83).
fn arb_paste_text() -> impl Strategy<Value = String> {
    let ch = prop_oneof![
        3 => select(vec!['a', '\n', '\r', '\t', '\u{7}', 'β', '$', '\\', '{']),
        1 => arb_char(),
    ];
    prop::collection::vec(ch, 0..24).prop_map(|chars| chars.into_iter().collect())
}

/// A text block turned into a grid (T84): `T`, a click, pasted text, `Esc`,
/// then `V`, a click on the block and `Ctrl+T`.
fn arb_table_session() -> impl Strategy<Value = Vec<InputEvent>> {
    arb_block_key_session(arb_paste_text(), Key::T)
}

/// An edge list: mostly whole lines the parser reads, sometimes not.
fn arb_edge_text() -> impl Strategy<Value = String> {
    let line = select(vec![
        "1-2",
        "1-2 2-3",
        "2>3",
        "3-1",
        "0-4",
        "2-2",
        "2-1",
        "1 2",
        "2 3 7",
        "3 4 -1.5",
        "5 6",
        "1 2 x",
        // Spaced pairs (T97).
        "1 > 2",
        "2 - 3",
        "3> 1 1 -4",
        // Math names and weights (T98).
        "$a$ $b$",
        "$b$ 1 $w_1$",
        "$x y$ > 2",
        "$a 1",
    ]);
    // A keyword, and sometimes a count line, on top (T96).
    let header = select(vec![
        "",
        "",
        "",
        "graph:\n",
        "tree:\n",
        "dgraph:\n",
        "tree:\n3\n",
        "dgraph:\n9 2\n",
    ]);
    let edges = (header, prop::collection::vec(line, 1..12))
        .prop_map(|(header, lines)| format!("{header}{}", lines.join("\n")));
    let entry = select(vec!["1", "1", "2", "3", "0", "-1", "x"]);
    let parents = (
        select(vec!["parents:", "parents0:", "parents:\n4"]),
        prop::collection::vec(entry, 1..12),
    )
        .prop_map(|(keyword, entries)| format!("{keyword}\n{}", entries.join(" ")));
    prop_oneof![3 => edges, 1 => parents]
}

/// A text block turned into a graph (T87), as [`arb_table_session`].
fn arb_graph_session() -> impl Strategy<Value = Vec<InputEvent>> {
    arb_block_key_session(arb_edge_text(), Key::L)
}

/// A list of points: mostly whole lines the parser reads, sometimes not.
fn arb_point_text() -> impl Strategy<Value = String> {
    let line = select(vec![
        "0 0",
        "1 2",
        "-3 4.5",
        "10 -20",
        "2.5 2.5",
        "1e3 7",
        "1e300 -1e300",
        "1e-300 0",
        "5 5",
        "-0.001 0.002",
        "100 100",
        "1 nan",
        "1 2 A",
        "3 -1 top left",
        "0 0 $p_1$",
        "2 2 1 2 3",
    ]);
    prop::collection::vec(line, 1..12).prop_map(|lines| lines.join("\n"))
}

/// A text block turned into a plot (T93), as [`arb_table_session`].
fn arb_plot_session() -> impl Strategy<Value = Vec<InputEvent>> {
    arb_block_key_session(arb_point_text(), Key::P)
}

/// `T`, a click, pasted `text`, `Esc`, then `V`, a click on the block and
/// `Ctrl` + `key`.
fn arb_block_key_session(
    text: impl Strategy<Value = String>,
    key: Key,
) -> impl Strategy<Value = Vec<InputEvent>> {
    let none = Modifiers::NONE;
    let press = |key, mods| InputEvent::KeyDown { key, mods };
    let click = move |pos| {
        [
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
        ]
    };
    (arb_screen_pos(), text).prop_map(move |(pos, text)| {
        let ctrl = Modifiers { ctrl: true, ..none };
        let mut events = vec![press(Key::T, none)];
        events.extend(click(pos));
        events.push(InputEvent::Paste(text));
        events.push(press(Key::Escape, none));
        events.push(press(Key::V, none));
        events.extend(click(pos + Vec2::new(2.0, 4.0)));
        events.push(press(key, ctrl));
        events
    })
}

/// A pen pressure: mostly in range, sometimes out of it, non-finite or
/// absent. Decoded from one byte: nested unions overflow the test thread's
/// stack in debug builds.
fn arb_pressure() -> impl Strategy<Value = Option<f32>> {
    any::<u8>().prop_map(|b| match b {
        0..=199 => Some(f32::from(b) / 199.0),
        200..=219 => Some(f32::from(b) - 210.0),
        220..=229 => Some(f32::NAN),
        230..=239 => Some(f32::INFINITY),
        _ => None,
    })
}

/// [`arb_session`] with a pen: a pressure before most pointer events (T35).
fn arb_pressure_session() -> impl Strategy<Value = Vec<InputEvent>> {
    (
        arb_session(),
        prop::collection::vec(prop::option::of(arb_pressure()), 1..16),
    )
        .prop_map(|(session, pressures)| {
            let mut events = Vec::with_capacity(session.len() * 2);
            for (event, pressure) in session.into_iter().zip(pressures.into_iter().cycle()) {
                let pointer = matches!(
                    event,
                    InputEvent::PointerDown { .. }
                        | InputEvent::PointerMove { .. }
                        | InputEvent::PointerUp { .. }
                );
                if let (true, Some(pressure)) = (pointer, pressure) {
                    events.push(InputEvent::Pressure { pressure });
                }
                events.push(event);
            }
            events
        })
}

/// A Text-tool session (T29): `T`, a click, then typing and editing keys,
/// sometimes ended with `Esc`, `Ctrl+Z`, `Ctrl+Enter` or another click.
fn arb_text_session() -> impl Strategy<Value = Vec<InputEvent>> {
    let none = Modifiers::NONE;
    let step = prop_oneof![
        8 => arb_char().prop_map(move |ch| vec![InputEvent::Char { ch, mods: none }]),
        2 => arb_formula().prop_map(move |src| {
            src.chars().map(|ch| InputEvent::Char { ch, mods: none }).collect()
        }),
        3 => select(TEXT_KEYS.to_vec()).prop_map(move |key| vec![InputEvent::KeyDown { key, mods: none }]),
        1 => arb_paste_text().prop_map(|text| vec![InputEvent::Paste(text)]),
        1 => (arb_key(), arb_mods()).prop_map(|(key, mods)| vec![InputEvent::KeyDown { key, mods }]),
        1 => arb_screen_pos().prop_map(move |pos| vec![
            InputEvent::PointerDown { pos, button: PointerButton::Left, mods: none },
            InputEvent::PointerUp { pos, button: PointerButton::Left, mods: none },
        ]),
    ];
    let end = prop_oneof![
        3 => Just(vec![InputEvent::KeyDown { key: Key::Escape, mods: none }]),
        1 => Just(vec![InputEvent::KeyDown {
            key: Key::Z,
            mods: Modifiers { ctrl: true, ..none },
        }]),
        1 => Just(vec![InputEvent::KeyDown {
            key: Key::Enter,
            mods: Modifiers { ctrl: true, ..none },
        }]),
        1 => Just(Vec::new()),
    ];
    (arb_screen_pos(), prop::collection::vec(step, 0..30), end).prop_map(
        move |(pos, steps, end)| {
            let mut events = vec![
                InputEvent::KeyDown {
                    key: Key::T,
                    mods: none,
                },
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
            ];
            events.extend(steps.into_iter().flatten());
            events.extend(end);
            events
        },
    )
}

/// A shape-text session (T31): draw a rectangle, ellipse or grid, switch
/// to the Text tool, click inside it, then type, walk cells with `Tab` /
/// `Enter` (with or without `Shift`) and end like a text session.
fn arb_host_session() -> impl Strategy<Value = Vec<InputEvent>> {
    let none = Modifiers::NONE;
    let shift = Modifiers {
        shift: true,
        ..none
    };
    let step = prop_oneof![
        6 => arb_char().prop_map(move |ch| vec![InputEvent::Char { ch, mods: none }]),
        1 => arb_formula().prop_map(move |src| {
            src.chars().map(|ch| InputEvent::Char { ch, mods: none }).collect()
        }),
        3 => (select(vec![Key::Tab, Key::Enter]), any::<bool>()).prop_map(move |(key, s)| {
            vec![InputEvent::KeyDown { key, mods: if s { shift } else { none } }]
        }),
        2 => select(TEXT_KEYS.to_vec()).prop_map(move |key| vec![InputEvent::KeyDown { key, mods: none }]),
        1 => arb_paste_text().prop_map(|text| vec![InputEvent::Paste(text)]),
        1 => (-1.0f32..1.0, -1.0f32..1.0).prop_map(|(x, y)| vec![
            InputEvent::PointerDown { pos: Vec2::new(x, y), button: PointerButton::Left, mods: Modifiers::NONE },
            InputEvent::PointerUp { pos: Vec2::new(x, y), button: PointerButton::Left, mods: Modifiers::NONE },
        ]),
    ];
    let end = prop_oneof![
        3 => Just(vec![InputEvent::KeyDown { key: Key::Escape, mods: none }]),
        1 => Just(vec![InputEvent::KeyDown {
            key: Key::Z,
            mods: Modifiers { ctrl: true, ..none },
        }]),
        1 => Just(Vec::new()),
    ];
    (
        select(vec![Key::R, Key::C, Key::G]),
        (50.0f32..700.0, 50.0f32..500.0),
        (60.0f32..300.0, 60.0f32..300.0),
        prop::collection::vec(step, 0..30),
        end,
    )
        .prop_map(move |(tool, (x, y), (w, h), steps, end)| {
            let at = |dx: f32, dy: f32| Vec2::new(x + dx * w, y + dy * h);
            let press = |pos| InputEvent::PointerDown {
                pos,
                button: PointerButton::Left,
                mods: none,
            };
            let release = |pos| InputEvent::PointerUp {
                pos,
                button: PointerButton::Left,
                mods: none,
            };
            let mut events = vec![
                InputEvent::KeyDown {
                    key: tool,
                    mods: none,
                },
                press(at(0.0, 0.0)),
                InputEvent::PointerMove {
                    pos: at(0.5, 0.5),
                    mods: none,
                },
                release(at(1.0, 1.0)),
                InputEvent::KeyDown {
                    key: Key::T,
                    mods: none,
                },
                press(at(0.45, 0.45)),
                release(at(0.45, 0.45)),
            ];
            for step in steps {
                for event in step {
                    // Clicks are offsets inside the shape.
                    events.push(match event {
                        InputEvent::PointerDown { pos, .. } => {
                            press(at(0.5 + pos.x * 0.4, 0.5 + pos.y * 0.4))
                        }
                        InputEvent::PointerUp { pos, .. } => {
                            release(at(0.5 + pos.x * 0.4, 0.5 + pos.y * 0.4))
                        }
                        other => other,
                    });
                }
            }
            events.extend(end);
            events
        })
}

/// A realistic drag: down, a wobbly path of moves (rarely a wild point), up.
fn arb_gesture() -> impl Strategy<Value = Vec<InputEvent>> {
    let step = prop_oneof![
        12 => (-40.0f32..40.0, -40.0f32..40.0).prop_map(|(x, y)| Vec2::new(x, y)),
        1 => arb_pos(),
    ];
    // Mostly plain left drags: they reach the active tool.
    let button = prop_oneof![
        6 => Just(PointerButton::Left),
        1 => arb_button(),
    ];
    let mods = prop_oneof![
        4 => Just(Modifiers::NONE),
        1 => arb_mods(),
    ];
    (
        button,
        mods,
        arb_screen_pos(),
        prop::collection::vec(step, 0..30),
        any::<bool>(),
    )
        .prop_map(|(button, mods, start, steps, release)| {
            let mut events = vec![InputEvent::PointerDown {
                pos: start,
                button,
                mods,
            }];
            let mut pos = start;
            for step in steps {
                // Wild points replace the position; normal steps walk from it.
                pos = if step.is_finite() && step.length() <= 60.0 {
                    pos + step
                } else {
                    step
                };
                events.push(InputEvent::PointerMove { pos, mods });
            }
            if release {
                events.push(InputEvent::PointerUp { pos, button, mods });
            }
            events
        })
}

/// A burst of bound key chords: tool switches, undo/redo, clipboard, view.
fn arb_command_burst() -> impl Strategy<Value = Vec<InputEvent>> {
    let chord = select(
        BINDINGS
            .iter()
            .map(|&(chord, _, _)| chord)
            .collect::<Vec<_>>(),
    );
    prop::collection::vec(chord, 1..8).prop_map(|chords| {
        chords
            .into_iter()
            .map(|chord| InputEvent::KeyDown {
                key: chord.key,
                mods: chord.mods,
            })
            .collect()
    })
}

/// Holding `Space` around a left drag (space-pan), released or not.
fn arb_space_drag() -> impl Strategy<Value = Vec<InputEvent>> {
    (arb_gesture(), any::<bool>()).prop_map(|(gesture, release)| {
        let space = |down: bool| {
            let (key, mods) = (Key::Space, Modifiers::NONE);
            if down {
                InputEvent::KeyDown { key, mods }
            } else {
                InputEvent::KeyUp { key, mods }
            }
        };
        let mut events = vec![space(true)];
        events.extend(gesture);
        if release {
            events.push(space(false));
        }
        events
    })
}

/// A grid-tool drag with arrow keys pressed while the button is held.
fn arb_grid_drag() -> impl Strategy<Value = Vec<InputEvent>> {
    let arrow = select(ARROW_KEYS.to_vec()).prop_map(|key| InputEvent::KeyDown {
        key,
        mods: Modifiers::NONE,
    });
    (
        arb_gesture(),
        prop::collection::vec(arrow, 0..12),
        any::<u64>(),
    )
        .prop_map(|(gesture, arrows, seed)| {
            let mut events = vec![InputEvent::KeyDown {
                key: Key::G,
                mods: Modifiers::NONE,
            }];
            // Spread the arrows over the drag, after the press.
            let len = gesture.len().max(1) as u64;
            let mut at: Vec<usize> = (0..arrows.len() as u64)
                .map(|i| (1 + (seed.wrapping_add(i * 7919) % len)) as usize)
                .collect();
            at.sort_unstable();
            let mut arrows = arrows.into_iter();
            for (i, event) in gesture.into_iter().enumerate() {
                events.push(event);
                while at.first() == Some(&(i + 1)) {
                    at.remove(0);
                    events.extend(arrows.next());
                }
            }
            events.extend(arrows);
            events
        })
}

/// A resize session (T91): draw a box at a known place, select it, drag
/// one of its handles with any modifiers (released or not) and press the
/// scale keys. Handles sit about 5 px outside the drawn corners.
fn arb_resize_session() -> impl Strategy<Value = Vec<InputEvent>> {
    let none = Modifiers::NONE;
    let (left, top) = (Vec2::new(300.0, 300.0), Vec2::new(500.0, 420.0));
    let corners = vec![
        Vec2::new(left.x - 5.0, left.y - 5.0),
        Vec2::new(top.x + 5.0, left.y - 5.0),
        Vec2::new(top.x + 5.0, top.y + 5.0),
        Vec2::new(left.x - 5.0, top.y + 5.0),
    ];
    (
        select(vec![Key::R, Key::C, Key::G, Key::W, Key::D]),
        select(corners),
        arb_mods(),
        prop::collection::vec(arb_pos(), 0..6),
        any::<bool>(),
        prop::collection::vec(any::<bool>(), 0..12),
    )
        .prop_map(move |(tool, handle, mods, steps, release, grows)| {
            let left_button = PointerButton::Left;
            let mut events = vec![
                InputEvent::KeyDown {
                    key: tool,
                    mods: none,
                },
                InputEvent::PointerDown {
                    pos: left,
                    button: left_button,
                    mods: none,
                },
                InputEvent::PointerMove {
                    pos: top,
                    mods: none,
                },
                InputEvent::PointerUp {
                    pos: top,
                    button: left_button,
                    mods: none,
                },
                InputEvent::KeyDown {
                    key: Key::V,
                    mods: none,
                },
                InputEvent::KeyDown {
                    key: Key::A,
                    mods: Modifiers { ctrl: true, ..none },
                },
                InputEvent::PointerDown {
                    pos: handle,
                    button: left_button,
                    mods,
                },
            ];
            let mut last = handle;
            for pos in steps {
                last = pos;
                events.push(InputEvent::PointerMove { pos, mods });
            }
            if release {
                events.push(InputEvent::PointerUp {
                    pos: last,
                    button: left_button,
                    mods,
                });
            }
            let alt = Modifiers { alt: true, ..none };
            events.extend(grows.into_iter().map(|grow| InputEvent::KeyDown {
                key: if grow {
                    Key::BracketRight
                } else {
                    Key::BracketLeft
                },
                mods: alt,
            }));
            events
        })
}

/// A session of 1..=500 events, biased towards realistic gestures.
fn arb_session() -> impl Strategy<Value = Vec<InputEvent>> {
    let chunk = prop_oneof![
        6 => arb_gesture(),
        1 => arb_grid_drag(),
        1 => arb_resize_session(),
        3 => arb_command_burst(),
        1 => arb_space_drag(),
        2 => arb_text_session(),
        1 => arb_host_session(),
        1 => arb_table_session(),
        1 => arb_graph_session(),
        1 => arb_plot_session(),
        3 => arb_event().prop_map(|event| vec![event]),
        1 => arb_screen_pos().prop_map(|size| vec![InputEvent::Resize { size }]),
    ];
    prop::collection::vec(chunk, 1..60).prop_map(|chunks| {
        let mut events: Vec<InputEvent> = chunks.into_iter().flatten().collect();
        events.truncate(MAX_SESSION_LEN);
        events
    })
}

/// Smoothing parameters, sane or arbitrary (negative, NaN, ∞).
fn arb_smoothing_params() -> impl Strategy<Value = SmoothingParams> {
    let value = prop_oneof![3 => 0.0f32..10.0, 1 => arb_coord()];
    (value.clone(), value.clone(), value).prop_map(|(min_dist_px, alpha, epsilon_px)| {
        SmoothingParams {
            min_dist_px,
            alpha,
            epsilon_px,
        }
    })
}

/// Default config (honours `PROPTEST_CASES`), with shrunk failures stored in
/// `tests/fuzz.proptest-regressions` next to this file.
fn config() -> ProptestConfig {
    ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..ProptestConfig::default()
    }
}

// ---------------------------------------------------------------------------
// Invariants
// ---------------------------------------------------------------------------

/// Snapshot of the document: ids and shapes, bottom to top.
fn shapes_of(editor: &Editor) -> Vec<(ShapeId, Shape)> {
    editor
        .document()
        .shapes()
        .map(|(id, shape)| (id, shape.clone()))
        .collect()
}

/// T31, T52: shape text is non-blank and sized, a numbered rectangle or
/// ellipse holds none (`label.is_some() ⇒ text.is_none()`), and cell texts
/// lie inside the grid, sorted by `(row, col)`, unique.
fn check_shape_text(id: ShapeId, shape: &Shape, step: usize) -> Result<(), TestCaseError> {
    let own = shape.slot_text(TextSlot::Shape);
    let cell_texts = shape.cell_texts();
    prop_assert!(
        shape.label().is_none() || own.is_none(),
        "step {step}: numbered shape {id:?} holds text"
    );
    for content in own.into_iter().chain(cell_texts.iter().map(|c| &c.content)) {
        prop_assert!(
            !content.text.trim().is_empty() && content.size.is_finite() && content.size > 0.0,
            "step {step}: shape {id:?} has bad text {content:?}"
        );
    }
    if let Shape::Grid { cols, rows, .. } = shape {
        prop_assert!(
            cell_texts.iter().all(|c| c.col < *cols && c.row < *rows),
            "step {step}: grid {id:?} text outside {cols} × {rows}"
        );
        prop_assert!(
            cell_texts
                .windows(2)
                .all(|w| (w[0].row, w[0].col) < (w[1].row, w[1].col)),
            "step {step}: grid {id:?} texts not sorted/unique"
        );
    }
    Ok(())
}

/// T35: a stroke's widths are in step with its points.
fn check_stroke_widths(id: ShapeId, shape: &Shape, step: usize) -> Result<(), TestCaseError> {
    if let Shape::Stroke {
        points,
        widths: Some(widths),
        ..
    } = shape
    {
        prop_assert_eq!(
            widths.len(),
            points.len(),
            "step {}: stroke {:?} widths out of step",
            step,
            id
        );
    }
    Ok(())
}

/// Checks every editor invariant from the architecture note.
fn check_invariants(editor: &Editor, step: usize) -> Result<(), TestCaseError> {
    let cells = 1..=GRID_MAX_CELLS;
    for (id, shape) in editor.document().shapes() {
        prop_assert!(
            shape.is_finite(),
            "step {step}: shape {id:?} not finite: {shape:?}"
        );
        if let Shape::Grid { cols, rows, .. } = shape {
            prop_assert!(
                cells.contains(cols) && cells.contains(rows),
                "step {step}: grid {id:?} has {cols} × {rows} cells"
            );
            // T21: fills inside the grid, sorted by (row, col), unique.
            let fills = shape.cell_fills();
            prop_assert!(
                fills.iter().all(|f| f.col < *cols && f.row < *rows),
                "step {step}: grid {id:?} fill outside {cols} × {rows}: {fills:?}"
            );
            prop_assert!(
                fills
                    .windows(2)
                    .all(|w| (w[0].row, w[0].col) < (w[1].row, w[1].col)),
                "step {step}: grid {id:?} fills not sorted/unique: {fills:?}"
            );
        }
        check_stroke_widths(id, shape, step)?;
        // T29: text blocks have a positive size and visible text.
        if let Shape::Text { text, size, .. } = shape {
            prop_assert!(*size > 0.0, "step {step}: text {id:?} has size {size}");
            prop_assert!(
                !text.trim().is_empty(),
                "step {step}: text {id:?} is blank: {text:?}"
            );
        }
        // T30: a text block's box is finite and holds its typeset ink.
        if let Shape::Text {
            origin, text, size, ..
        } = shape
        {
            let b = shape.bounds();
            prop_assert!(
                b.min.is_finite() && b.max.is_finite(),
                "step {step}: text {id:?} bounds {b:?}"
            );
            let slack = 1e-3 * size + 1e-5 * (origin.x.abs() + origin.y.abs());
            let room = b.expand(slack);
            for ink in &typeset(text).ink {
                let ib = ink.bounds();
                let (lo, hi) = (*origin + ib.min * *size, *origin + ib.max * *size);
                prop_assert!(
                    room.contains(lo) && room.contains(hi),
                    "step {step}: text {id:?} ink {ink:?} outside {b:?}"
                );
            }
        }
        check_shape_text(id, shape, step)?;
        // Any number is a valid label, 0 included: switching back to
        // one-based leaves existing labels alone (ADR-T86-1).
    }

    let helpers = editor.helpers();
    prop_assert!(
        cells.contains(&helpers.grid_cols) && cells.contains(&helpers.grid_rows),
        "step {step}: editor grid dims {} × {}",
        helpers.grid_cols,
        helpers.grid_rows
    );
    prop_assert!(
        helpers.next_number >= helpers.first_number(),
        "step {step}: numbering counter {} below first number {}",
        helpers.next_number,
        helpers.first_number()
    );

    let camera = editor.camera();
    let zoom = camera.zoom();
    prop_assert!(
        (ZOOM_MIN..=ZOOM_MAX).contains(&zoom),
        "step {step}: zoom {zoom} out of range"
    );
    prop_assert!(
        camera.offset().is_finite(),
        "step {step}: camera offset not finite: {:?}",
        camera.offset()
    );

    let mut seen = HashSet::new();
    for &id in editor.selection() {
        prop_assert!(
            editor.document().get(id).is_some(),
            "step {step}: selected {id:?} not in document"
        );
        prop_assert!(seen.insert(id), "step {step}: {id:?} selected twice");
    }
    Ok(())
}

/// Runs `session`, checking invariants after every event.
fn run_session(session: &[InputEvent]) -> Result<Editor, TestCaseError> {
    run_session_on(Editor::new(), session)
}

/// [`run_session`] on `editor`.
fn run_session_on(mut editor: Editor, session: &[InputEvent]) -> Result<Editor, TestCaseError> {
    check_invariants(&editor, 0)?;
    for (i, event) in session.iter().enumerate() {
        editor.handle(event.clone());
        check_invariants(&editor, i + 1)?;
    }
    Ok(editor)
}

// ---------------------------------------------------------------------------
// AC-1: strategy coverage
// ---------------------------------------------------------------------------

/// Special coordinate classes the strategies must produce.
const COORD_CLASSES: [&str; 5] = ["zero", "nan", "+inf", "-inf", "huge"];

/// The special class of `v`, if any.
fn coord_class(v: f32) -> Option<&'static str> {
    if v.is_nan() {
        Some("nan")
    } else if v == f32::INFINITY {
        Some("+inf")
    } else if v == f32::NEG_INFINITY {
        Some("-inf")
    } else if v.abs() > 1e30 {
        Some("huge")
    } else if v == 0.0 {
        Some("zero")
    } else {
        None
    }
}

#[test]
fn strategies_generate_all_variants() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_event();
    let mut variants = HashSet::new();
    let mut keys = HashSet::new();
    let mut buttons = HashSet::new();
    let mut mods_seen = HashSet::new();
    let mut coords = HashSet::new();

    // Act
    for _ in 0..20_000 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_event failed to generate a value");
        };
        let event = tree.current();
        let (variant, pos) = match event {
            InputEvent::PointerDown { pos, button, mods } => {
                buttons.insert(button);
                mods_seen.insert(mods);
                (0, Some(pos))
            }
            InputEvent::PointerMove { pos, mods } => {
                mods_seen.insert(mods);
                (1, Some(pos))
            }
            InputEvent::PointerUp { pos, button, mods } => {
                buttons.insert(button);
                mods_seen.insert(mods);
                (2, Some(pos))
            }
            InputEvent::Scroll { pos, delta } => {
                coords.extend(coord_class(delta));
                (3, Some(pos))
            }
            InputEvent::KeyDown { key, mods } => {
                keys.insert(key);
                mods_seen.insert(mods);
                (4, None)
            }
            InputEvent::KeyUp { key, mods } => {
                keys.insert(key);
                mods_seen.insert(mods);
                (5, None)
            }
            InputEvent::Resize { size } => (6, Some(size)),
            InputEvent::Char { mods, .. } => {
                mods_seen.insert(mods);
                (7, None)
            }
            InputEvent::Pressure { .. } => (8, None),
            InputEvent::Paste(_) => (9, None),
            InputEvent::PasteGrid(_) => (10, None),
        };
        variants.insert(variant);
        if let Some(pos) = pos {
            coords.extend(coord_class(pos.x));
            coords.extend(coord_class(pos.y));
        }
    }

    // Assert
    assert_eq!(variants.len(), 11, "event variants seen: {variants:?}");
    assert_eq!(keys.len(), ALL_KEYS.len(), "keys seen: {keys:?}");
    assert_eq!(
        buttons.len(),
        ALL_BUTTONS.len(),
        "buttons seen: {buttons:?}"
    );
    assert_eq!(
        mods_seen.len(),
        8,
        "modifier combinations seen: {mods_seen:?}"
    );
    assert_eq!(
        coords.len(),
        COORD_CLASSES.len(),
        "special coordinates seen: {coords:?}"
    );
}

#[test]
fn grid_drags_press_arrows_while_the_grid_tool_drags() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_grid_drag();
    let mut live = 0;

    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_grid_drag failed to generate a value");
        };
        let mut editor = Editor::new();
        let mut changed_live = false;

        // Act
        for event in tree.current() {
            let during_grid_drag = editor.active_gesture()
                == Some(draw::core::editor::ActiveGesture::Tool(Tool::Grid));
            let is_arrow =
                matches!(event, InputEvent::KeyDown { key, .. } if ARROW_KEYS.contains(&key));
            changed_live |= editor.handle(event) && during_grid_drag && is_arrow;
        }
        live += usize::from(changed_live);
    }

    // Assert: dims really change in the middle of grid drags.
    assert!(live >= 50, "sessions resizing a grid mid-drag: {live}/200");
}

#[test]
fn table_sessions_turn_text_blocks_into_grids() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_table_session();
    let mut grids = 0;

    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_table_session failed to generate a value");
        };
        let mut editor = Editor::new();

        // Act
        for event in tree.current() {
            editor.handle(event);
        }
        grids += usize::from(
            editor
                .document()
                .shapes()
                .any(|(_, shape)| !shape.cell_texts().is_empty()),
        );
    }

    // Assert: a harness that never reaches `Ctrl+T` on a block proves nothing.
    assert!(grids >= 50, "sessions that built a grid: {grids}/200");
}

#[test]
fn graph_sessions_turn_text_blocks_into_graphs() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_graph_session();
    let mut graphs = 0;

    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_graph_session failed to generate a value");
        };
        let mut editor = Editor::new();

        // Act
        for event in tree.current() {
            editor.handle(event);
        }
        graphs += usize::from(
            editor
                .document()
                .shapes()
                .any(|(_, shape)| matches!(shape, Shape::Ellipse { .. })),
        );
    }

    // Assert: a harness that never reaches `Ctrl+L` on a block proves nothing.
    assert!(graphs >= 50, "sessions that built a graph: {graphs}/200");
}

#[test]
fn plot_sessions_turn_text_blocks_into_plots() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_plot_session();
    let mut plots = 0;

    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_plot_session failed to generate a value");
        };
        let mut editor = Editor::new();

        // Act
        for event in tree.current() {
            editor.handle(event);
        }
        plots += usize::from(
            editor
                .document()
                .shapes()
                .any(|(_, shape)| matches!(shape, Shape::Ellipse { fill: Some(_), .. })),
        );
    }

    // Assert: a harness that never reaches `Ctrl+P` on a block proves nothing.
    assert!(plots >= 50, "sessions that built a plot: {plots}/200");
}

#[test]
fn session_strategy_respects_length_bounds() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_session();

    for _ in 0..200 {
        // Act
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_session failed to generate a value");
        };
        let session = tree.current();

        // Assert
        assert!((1..=MAX_SESSION_LEN).contains(&session.len()));
    }
}

/// The resize sessions must really change the size of what they draw.
#[test]
fn resize_sessions_resize_shapes() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_resize_session();
    let mut resized = 0;

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_resize_session failed to generate a value");
        };
        let mut editor = Editor::new();
        let mut sizes: Vec<(f32, f32)> = Vec::new();
        for event in tree.current() {
            editor.handle(event);
            if let Some((_, shape)) = editor.document().shapes().next() {
                let b = shape.geometry_bounds();
                if sizes.last() != Some(&(b.width(), b.height())) {
                    sizes.push((b.width(), b.height()));
                }
            }
        }
        resized += usize::from(sizes.len() > 1);
    }

    // Assert
    assert!(resized >= 20, "sessions that resized: {resized}/200");
}

#[test]
fn sessions_reach_deep_editor_states() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_session();
    let (mut with_shapes, mut with_selection, mut zoomed, mut with_redo) = (0, 0, 0, 0);

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_session failed to generate a value");
        };
        let mut editor = Editor::new();
        let (mut shapes, mut selection, mut zoom, mut redo) = (false, false, false, false);
        for event in tree.current() {
            editor.handle(event);
            shapes |= !editor.document().is_empty();
            selection |= !editor.selection().is_empty();
            zoom |= !draw::core::geom::approx_eq(editor.camera().zoom(), 1.0, 1e-6);
            redo |= editor.can_redo();
        }
        with_shapes += usize::from(shapes);
        with_selection += usize::from(selection);
        zoomed += usize::from(zoom);
        with_redo += usize::from(redo);
    }

    // Assert: a harness that never reaches these states proves nothing.
    assert!(
        with_shapes >= 100,
        "sessions with shapes: {with_shapes}/200"
    );
    // About 11 % of sessions select something (measured over 2000), so
    // 5 % keeps the check from failing whenever a new token shifts the
    // deterministic stream (T54).
    assert!(
        with_selection >= 10,
        "sessions with a selection: {with_selection}/200"
    );
    assert!(zoomed >= 20, "sessions that zoomed: {zoomed}/200");
    assert!(with_redo >= 20, "sessions that could redo: {with_redo}/200");
}

#[test]
fn sessions_reach_text_edit_states() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_session();
    let (mut editing, mut committed) = (0, 0);

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_session failed to generate a value");
        };
        let mut editor = Editor::new();
        let (mut edit, mut text) = (false, false);
        for event in tree.current() {
            editor.handle(event);
            edit |= editor.text_edit().is_some_and(|e| !e.text().is_empty());
            text |= editor
                .document()
                .shapes()
                .any(|(_, shape)| matches!(shape, Shape::Text { .. }));
        }
        editing += usize::from(edit);
        committed += usize::from(text);
    }

    // Assert
    assert!(editing >= 40, "sessions typing into a block: {editing}/200");
    assert!(
        committed >= 20,
        "sessions with a text block: {committed}/200"
    );
}

#[test]
fn numbered_host_sessions_never_fill_numbered_shapes() {
    // Arrange: T52, numbering is turned on before the host is drawn.
    let mut runner = TestRunner::deterministic();
    let strategy = arb_host_session();
    let (mut numbered, mut blocks) = (0, 0);

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_host_session failed to generate a value");
        };
        let mut editor = Editor::new();
        editor.handle(InputEvent::Resize {
            size: Vec2::new(1024.0, 768.0),
        });
        editor.handle(InputEvent::KeyDown {
            key: Key::N,
            mods: Modifiers::NONE,
        });
        for event in tree.current() {
            editor.handle(event);
            let shapes = || editor.document().shapes().map(|(_, shape)| shape);
            assert!(
                shapes().all(|s| s.label().is_none() || s.slot_text(TextSlot::Shape).is_none()),
                "a numbered shape holds text"
            );
        }
        let shapes = || editor.document().shapes().map(|(_, shape)| shape);
        numbered += usize::from(shapes().any(|s| s.label().is_some()));
        blocks += usize::from(
            shapes().any(|s| s.label().is_some())
                && shapes().any(|s| matches!(s, Shape::Text { .. })),
        );
    }

    // Assert: typing inside a numbered node made free blocks instead.
    assert!(
        numbered >= 60,
        "sessions with a numbered shape: {numbered}/200"
    );
    assert!(
        blocks >= 20,
        "sessions with a numbered shape and a text block: {blocks}/200"
    );
}

#[test]
fn sessions_reach_shape_text() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_host_session();
    let (mut own, mut cells) = (0, 0);

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_host_session failed to generate a value");
        };
        let mut editor = Editor::new();
        editor.handle(InputEvent::Resize {
            size: Vec2::new(1024.0, 768.0),
        });
        for event in tree.current() {
            editor.handle(event);
        }
        let shapes = || editor.document().shapes().map(|(_, shape)| shape);
        own += usize::from(shapes().any(|s| s.slot_text(TextSlot::Shape).is_some()));
        cells += usize::from(shapes().any(|s| s.cell_texts().len() >= 2));
    }

    // Assert
    assert!(own >= 30, "sessions with rectangle/ellipse text: {own}/200");
    assert!(
        cells >= 15,
        "sessions with two or more cell texts: {cells}/200"
    );
}

#[test]
fn sessions_reach_committed_formulas() {
    // Arrange
    let mut runner = TestRunner::deterministic();
    let strategy = arb_text_session();
    let mut formulas = 0;

    // Act
    for _ in 0..200 {
        let Ok(tree) = strategy.new_tree(&mut runner) else {
            panic!("arb_text_session failed to generate a value");
        };
        let mut editor = Editor::new();
        for event in tree.current() {
            editor.handle(event);
        }
        let has_formula = editor.document().shapes().any(|(_, shape)| {
            matches!(shape, Shape::Text { text, .. }
                if tex::parse(text).iter().any(|s| matches!(s, tex::Segment::Math(_))))
        });
        formulas += usize::from(has_formula);
    }

    // Assert
    assert!(
        formulas >= 10,
        "sessions with a committed formula: {formulas}/200"
    );
}

// ---------------------------------------------------------------------------
// AC-2, AC-3: editor sessions
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn editor_never_panics_and_keeps_invariants(session in arb_session()) {
        // Arrange / Act / Assert: invariants are checked after every event.
        run_session(&session)?;
    }

    #[test]
    fn export_requests_never_change_the_editor(session in arb_session()) {
        // Arrange
        let mut editor = run_session_on(Editor::new(), &session)?;
        let before = editor.clone();

        // Act
        let redraw = editor.apply(Command::ExportView);

        // Assert
        prop_assert!(!redraw, "an export needs no redraw");
        prop_assert!(editor == before, "the export changed the editor");
    }

    #[test]
    fn undo_all_then_redo_all_is_symmetric(session in arb_session()) {
        check_undo_redo_symmetric(&session, Editor::new())?;
    }

    #[test]
    fn pen_tap_sessions_never_panic_and_keep_invariants(session in arb_session()) {
        // T34: every event goes through the tap filter first.
        run_session_on(pen_editor(), &session)?;
    }

    #[test]
    fn pen_tap_undo_all_then_redo_all_is_symmetric(session in arb_session()) {
        check_undo_redo_symmetric(&session, pen_editor())?;
    }

    #[test]
    fn pressure_sessions_keep_stroke_widths_in_step(session in arb_pressure_session()) {
        // T35: pressure reaches the pen; check_invariants checks the widths.
        let editor = run_session_on(pen_editor(), &session)?;
        for (id, shape) in editor.document().shapes() {
            if let Shape::Stroke { widths: Some(widths), .. } = shape {
                prop_assert!(
                    widths
                        .iter()
                        .all(|w| (MIN_WIDTH_FACTOR..=MAX_WIDTH_FACTOR).contains(w)),
                    "stroke {id:?} widths out of range: {widths:?}"
                );
            }
        }
    }

    #[test]
    fn pressure_undo_all_then_redo_all_is_symmetric(session in arb_pressure_session()) {
        check_undo_redo_symmetric(&session, pen_editor())?;
    }
}

/// An editor with the pen tap slop on (T34).
fn pen_editor() -> Editor {
    Editor::new().with_tap_slop(PEN_TAP_SLOP_PX)
}

/// Runs `session`, undoes everything (the document must be empty unless the
/// history was capped) and redoes everything (the document must be back).
fn check_undo_redo_symmetric(session: &[InputEvent], editor: Editor) -> Result<(), TestCaseError> {
    // Arrange
    let mut editor = run_session_on(editor, session)?;
    editor.apply(Command::Cancel);
    // Leftover redo entries would make "redo all" overshoot the snapshot.
    while editor.can_redo() {
        prop_assert!(editor.apply(Command::Redo), "redo reported no change");
    }
    let before = shapes_of(&editor);

    // Act: undo everything.
    let mut undos = 0usize;
    while editor.can_undo() {
        prop_assert!(editor.apply(Command::Undo), "undo reported no change");
        check_invariants(&editor, session.len() + undos)?;
        undos += 1;
        prop_assert!(undos <= HISTORY_LIMIT, "more undos than the history limit");
    }

    // Assert: empty unless the history was capped.
    if undos < HISTORY_LIMIT {
        prop_assert!(
            editor.document().is_empty(),
            "{} shapes left after {undos} undos",
            editor.document().len()
        );
    }

    // Act: redo everything.
    for _ in 0..undos {
        prop_assert!(editor.apply(Command::Redo), "redo reported no change");
        check_invariants(&editor, session.len() + undos)?;
    }

    // Assert
    prop_assert!(!editor.can_redo());
    prop_assert_eq!(shapes_of(&editor), before);
    Ok(())
}

// ---------------------------------------------------------------------------
// AC-4: smoothing
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn smoother_never_panics_on_arbitrary_points(
        params in arb_smoothing_params(),
        px_to_world in arb_coord(),
        points in prop::collection::vec(arb_pos(), 0..200),
    ) {
        // Arrange
        let mut smoother = Smoother::new(params, px_to_world);

        // Act
        for &p in &points {
            smoother.push(p);
            prop_assert!(smoother.points().iter().all(|q| q.is_finite()));
        }
        let finished = smoother.finish();

        // Assert
        prop_assert!(finished.iter().all(|q| q.is_finite()), "{finished:?}");
    }

    #[test]
    fn simplify_rdp_never_panics_on_arbitrary_points(
        points in prop::collection::vec(arb_pos(), 0..200),
        eps in arb_coord(),
    ) {
        // Arrange
        let finite: Vec<Vec2> = points.iter().copied().filter(|p| p.is_finite()).collect();

        // Act
        let _ = simplify_rdp(&points, eps);
        let simplified = simplify_rdp(&finite, eps);

        // Assert: finite, and a subsequence of the input.
        prop_assert!(simplified.iter().all(|p| p.is_finite()));
        let mut rest = finite.iter();
        for p in &simplified {
            prop_assert!(rest.any(|q| q == p), "{p:?} is not from the input in order");
        }
    }
}

proptest! {
    #![proptest_config(config())]

    /// T30 AC-6: parsing and layout are total on formula-ish text.
    #[test]
    fn fuzz_tex_parse_and_layout_are_total(src in arb_formula()) {
        // Act
        let segments = tex::parse(&src);
        let layout = tex::layout(&segments);

        // Assert
        prop_assert_eq!(tex::print(&segments), src);
        prop_assert!(layout.width.is_finite() && layout.height.is_finite());
        let room = layout.bounds().expand(1e-3);
        for ink in &layout.ink {
            let ib = ink.bounds();
            prop_assert!(room.contains(ib.min) && room.contains(ib.max), "{ink:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// T74: editor snapshots (ADR-T74-1)
// ---------------------------------------------------------------------------

/// `editor` through snapshot text and back.
fn snapshot_round_trip(editor: &Editor) -> Result<Editor, TestCaseError> {
    let text = editor.snapshot().to_text();
    let snapshot = Snapshot::parse(&text)
        .map_err(|err| TestCaseError::fail(format!("own text rejected: {err}\n{text}")))?;
    let loaded = Editor::from_snapshot(snapshot);
    prop_assert_eq!(loaded.snapshot().to_text(), text, "text not canonical");
    Ok(loaded)
}

/// T74 AC-1: runs `session[..cut]`, round-trips the state, then feeds
/// the rest to both editors and compares them after every event.
fn check_snapshot_continues_equal(
    session: &[InputEvent],
    cut: prop::sample::Index,
    start: Editor,
) -> Result<(), TestCaseError> {
    let cut = cut.index(session.len() + 1);
    let (head, tail) = session.split_at(cut);
    let mut editor = run_session_on(start, head)?;
    let mut loaded = snapshot_round_trip(&editor)?;
    prop_assert!(loaded == editor, "loaded state differs at {}", cut);
    for (i, event) in tail.iter().enumerate() {
        editor.handle(event.clone());
        loaded.handle(event.clone());
        prop_assert!(
            loaded == editor,
            "differs after event {} ({:?})",
            cut + i,
            event
        );
    }
    Ok(())
}

/// Tokens that appear in snapshot text, for mutations.
const SNAPSHOT_TOKENS: [&str; 34] = [
    "-",
    "0",
    "1",
    "2",
    "7",
    "64",
    "65",
    "999",
    "4294967295",
    "18446744073709551615",
    "-1",
    "0.0",
    "-0.0",
    "1e38",
    "1e-45",
    "NaN",
    "inf",
    "\"\"",
    "\"x\\n\"",
    "\"\\u{0}\"",
    "\"unterminated",
    "\\",
    "pen",
    "select",
    "left",
    "holding",
    "move",
    "marquee",
    "replace",
    "insert",
    "stroke",
    "grid",
    "[tools]",
    "draw-snapshot",
];

/// One edit of snapshot text.
#[derive(Debug, Clone)]
enum Mutation {
    /// Replace token `token` of line `line` with a pool token.
    Token {
        line: prop::sample::Index,
        token: prop::sample::Index,
        with: &'static str,
    },
    /// Drop a line.
    Drop(prop::sample::Index),
    /// Copy a line below itself.
    Duplicate(prop::sample::Index),
    /// Swap two lines.
    Swap(prop::sample::Index, prop::sample::Index),
    /// Cut the text at a byte offset (on a char boundary).
    Truncate(prop::sample::Index),
}

fn arb_mutation() -> impl Strategy<Value = Mutation> {
    let index = any::<prop::sample::Index>;
    prop_oneof![
        6 => (index(), index(), select(&SNAPSHOT_TOKENS[..]))
            .prop_map(|(line, token, with)| Mutation::Token { line, token, with }),
        2 => index().prop_map(Mutation::Drop),
        2 => index().prop_map(Mutation::Duplicate),
        1 => (index(), index()).prop_map(|(a, b)| Mutation::Swap(a, b)),
        1 => index().prop_map(Mutation::Truncate),
    ]
}

/// `text` with `mutations` applied.
fn mutate(text: &str, mutations: &[Mutation]) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    for mutation in mutations {
        if lines.is_empty() {
            break;
        }
        match mutation {
            Mutation::Token { line, token, with } => {
                let line = line.index(lines.len());
                let mut tokens: Vec<&str> = lines[line].split(' ').collect();
                let token = token.index(tokens.len());
                tokens[token] = with;
                lines[line] = tokens.join(" ");
            }
            Mutation::Drop(line) => {
                lines.remove(line.index(lines.len()));
            }
            Mutation::Duplicate(line) => {
                let line = line.index(lines.len());
                lines.insert(line, lines[line].clone());
            }
            Mutation::Swap(a, b) => {
                let (a, b) = (a.index(lines.len()), b.index(lines.len()));
                lines.swap(a, b);
            }
            Mutation::Truncate(at) => {
                let joined = lines.join("\n");
                let mut at = at.index(joined.len() + 1);
                while !joined.is_char_boundary(at) {
                    at -= 1;
                }
                lines = joined[..at].lines().map(str::to_owned).collect();
            }
        }
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

proptest! {
    #![proptest_config(config())]

    /// T85: resizing a committed grid at random keeps every invariant, and
    /// undoing everything empties the document.
    #[test]
    fn grid_resize_sessions_keep_invariants(
        commands in prop::collection::vec(
            select(vec![
                Command::GridCols(1),
                Command::GridCols(-1),
                Command::GridRows(1),
                Command::GridRows(-1),
                Command::Undo,
                Command::Redo,
            ]),
            0..200,
        ),
    ) {
        // Arrange: one 4 × 4 grid with a filled cell, selected.
        let pointer = |pos: Vec2| (pos, PointerButton::Left, Modifiers::NONE);
        let click = |editor: &mut Editor, from: Vec2, to: Vec2| {
            let (pos, button, mods) = pointer(from);
            editor.handle(InputEvent::PointerDown { pos, button, mods });
            editor.handle(InputEvent::PointerMove { pos: to, mods });
            editor.handle(InputEvent::PointerUp { pos: to, button, mods });
        };
        let mut editor = Editor::new();
        editor.apply(Command::SetTool(Tool::Grid));
        click(&mut editor, Vec2::new(100.0, 100.0), Vec2::new(500.0, 500.0));
        editor.apply(Command::SetTool(Tool::Bucket));
        click(&mut editor, Vec2::new(450.0, 450.0), Vec2::new(450.0, 450.0));
        editor.apply(Command::SetTool(Tool::Select));
        editor.apply(Command::SelectAll);
        prop_assert_eq!(editor.selection().len(), 1);

        // Act
        for (step, command) in commands.into_iter().enumerate() {
            editor.apply(command);
            check_invariants(&editor, step)?;
        }

        // Assert
        while editor.can_undo() {
            prop_assert!(editor.apply(Command::Undo));
        }
        prop_assert!(editor.document().is_empty());
    }

    /// T74 AC-1.
    #[test]
    fn snapshot_round_trip_continues_equal(
        session in arb_session(),
        cut in any::<prop::sample::Index>(),
    ) {
        check_snapshot_continues_equal(&session, cut, Editor::new())?;
    }

    /// T74 AC-1 with the pen tap filter on (mid-tap states).
    #[test]
    fn pen_tap_snapshot_round_trip_continues_equal(
        session in arb_pressure_session(),
        cut in any::<prop::sample::Index>(),
    ) {
        check_snapshot_continues_equal(&session, cut, pen_editor())?;
    }

    /// T74 AC-3: any text parses or fails with a line number.
    #[test]
    fn snapshot_parse_is_total(text in "(?s).{0,400}") {
        // Act
        let result = Snapshot::parse(&text);

        // Assert
        if let Err(err) = result {
            prop_assert!(err.line >= 1 && err.line <= text.lines().count().max(1) + 1, "{}", err);
        }
    }

    /// T74 AC-3: mutated real snapshots parse or fail cleanly; what loads
    /// keeps the invariants under further input.
    #[test]
    fn snapshot_mutated_text_is_total(
        session in arb_session(),
        mutations in prop::collection::vec(arb_mutation(), 1..6),
        more in arb_session(),
    ) {
        // Arrange
        let editor = run_session_on(Editor::new(), &session)?;
        let text = mutate(&editor.snapshot().to_text(), &mutations);

        // Act
        let result = Snapshot::parse(&text);

        // Assert
        match result {
            Err(err) => prop_assert!(err.line >= 1, "{}", err),
            Ok(snapshot) => {
                let canonical = snapshot.to_text();
                let loaded = Editor::from_snapshot(snapshot);
                prop_assert_eq!(loaded.snapshot().to_text(), canonical);
                run_session_on(loaded, &more)?;
            }
        }
    }
}
