//! The routing table: what each state does with each kind of input.
//!
//! [`route`] is a `match` over `(InteractionKind, InputKind)` with no
//! wildcard arm, so a new state or input does not compile until its cells
//! are decided, "ignored" included (ADR-T134-1).

use super::input::InputKind;
use super::state::InteractionKind;

/// What a cell of the table decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// The editor handles the input.
    Editor,
    /// The input does nothing in this state, on purpose.
    Ignore,
}

/// One cell of the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    /// The state the machine is in.
    pub from: InteractionKind,
    /// The kind of input that arrives.
    pub input: InputKind,
    /// What happens.
    pub route: Route,
}

/// What `from` does with an input of kind `input`.
#[must_use]
pub fn route(from: InteractionKind, input: InputKind) -> Route {
    use InputKind as K;
    use InteractionKind as S;
    match (from, input) {
        (S::Idle, K::Tick | K::Reply) => Route::Ignore,
        (
            S::Idle,
            K::PointerDown
            | K::PointerMove
            | K::PointerUp
            | K::Scroll
            | K::KeyDown
            | K::KeyUp
            | K::Char
            | K::Pressure
            | K::Paste
            | K::PasteGrid
            | K::Resize,
        ) => Route::Editor,
        (S::Gesture, K::Tick | K::Reply) => Route::Ignore,
        (
            S::Gesture,
            K::PointerDown
            | K::PointerMove
            | K::PointerUp
            | K::Scroll
            | K::KeyDown
            | K::KeyUp
            | K::Char
            | K::Pressure
            | K::Paste
            | K::PasteGrid
            | K::Resize,
        ) => Route::Editor,
        (S::TextEdit, K::Tick | K::Reply) => Route::Ignore,
        (
            S::TextEdit,
            K::PointerDown
            | K::PointerMove
            | K::PointerUp
            | K::Scroll
            | K::KeyDown
            | K::KeyUp
            | K::Char
            | K::Pressure
            | K::Paste
            | K::PasteGrid
            | K::Resize,
        ) => Route::Editor,
        (S::Palette, K::Tick | K::Reply) => Route::Ignore,
        (
            S::Palette,
            K::PointerDown
            | K::PointerMove
            | K::PointerUp
            | K::Scroll
            | K::KeyDown
            | K::KeyUp
            | K::Char
            | K::Pressure
            | K::Paste
            | K::PasteGrid
            | K::Resize,
        ) => Route::Editor,
        (S::Sheet, K::Tick | K::Reply) => Route::Ignore,
        (
            S::Sheet,
            K::PointerDown
            | K::PointerMove
            | K::PointerUp
            | K::Scroll
            | K::KeyDown
            | K::KeyUp
            | K::Char
            | K::Pressure
            | K::Paste
            | K::PasteGrid
            | K::Resize,
        ) => Route::Editor,
    }
}

/// Every cell of the table, state by state.
#[must_use]
pub fn transitions() -> Vec<Transition> {
    InteractionKind::ALL
        .into_iter()
        .flat_map(|from| {
            InputKind::ALL.into_iter().map(move |input| Transition {
                from,
                input,
                route: route(from, input),
            })
        })
        .collect()
}

/// Line that opens the generated diagram in the architecture note.
pub const DIAGRAM_BEGIN: &str = "<!-- diagram:begin -->";
/// Line that closes the generated diagram in the architecture note.
pub const DIAGRAM_END: &str = "<!-- diagram:end -->";

/// The Mermaid diagram of the table, as embedded in
/// `docs/architecture/Control automaton.md`.
#[must_use]
pub fn diagram() -> String {
    let mut out = String::from("```mermaid\nstateDiagram-v2\n    [*] --> Idle\n");
    for from in InteractionKind::ALL {
        for (route_kind, label) in [(Route::Ignore, "ignored"), (Route::Editor, "editor")] {
            let inputs: Vec<String> = InputKind::ALL
                .into_iter()
                .filter(|input| route(from, *input) == route_kind)
                .map(|input| format!("{input:?}"))
                .collect();
            if inputs.is_empty() {
                continue;
            }
            let target = match route_kind {
                Route::Ignore => from.name(),
                Route::Editor => "Editor",
            };
            out.push_str(&format!(
                "    {} --> {target} : {} ({label})\n",
                from.name(),
                inputs.join(", ")
            ));
        }
    }
    out.push_str("```\n");
    out
}
