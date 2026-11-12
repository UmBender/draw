//! The control automaton: `step(Machine, Input) -> (Machine, Vec<Effect>)`
//! (ADR-T134-1).
//!
//! The old [`Editor`] stays inside the machine as an adapter: the routing
//! [`route`] table decides, state by state, what each kind of input does, and the
//! editor still does the work for the cells that say so.

mod effect;
mod gesture;
mod input;
mod regions;
mod state;
mod table;

#[cfg(test)]
mod tests;

pub use effect::{ClipboardOp, Effect};
pub use gesture::{End, Gesture, Press, Stage};
pub use input::{Input, InputKind, Reply};
pub use regions::{Modal, ModalKind, Pages, View};
pub use state::{Interaction, InteractionKind};
pub use table::{DIAGRAM_BEGIN, DIAGRAM_END, Route, Transition, diagram, route, transitions};

use crate::core::editor::Editor;

/// The whole state of the automaton.
#[derive(Debug, Clone, PartialEq)]
pub struct Machine {
    editor: Editor,
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

impl Machine {
    /// A machine over a fresh editor.
    #[must_use]
    pub fn new() -> Self {
        Self::from_editor(Editor::new())
    }

    /// A machine over an existing editor.
    #[must_use]
    pub fn from_editor(editor: Editor) -> Self {
        Self { editor }
    }

    /// The editor inside.
    #[must_use]
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// The editor inside, to hand back to code that still uses it directly.
    #[must_use]
    pub fn into_editor(self) -> Editor {
        self.editor
    }

    /// What the machine is doing with the input right now. Modal overlays
    /// come first because they capture every input.
    #[must_use]
    pub fn interaction(&self) -> Interaction {
        if self.editor.sheet().is_some() {
            Interaction::Sheet
        } else if self.editor.palette().is_some() {
            Interaction::Palette
        } else if self.editor.text_edit().is_some() {
            Interaction::TextEdit
        } else if let Some(gesture) = self.editor.active_gesture() {
            Interaction::Gesture(gesture)
        } else {
            Interaction::Idle
        }
    }
}

/// One transition: total and pure (the machine is moved in and out).
#[must_use]
#[allow(
    clippy::needless_pass_by_value,
    reason = "the input is moved in so later tasks can take its payload without a clone"
)]
pub fn step(machine: Machine, input: Input) -> (Machine, Vec<Effect>) {
    let mut machine = machine;
    let mut effects = Vec::new();
    let cell = route(machine.interaction().kind(), input.kind());
    match cell {
        Route::Ignore => {}
        Route::Editor => {
            if let Some(event) = input.to_event() {
                if machine.editor.handle(event) {
                    effects.push(Effect::Redraw);
                }
                collect_requests(&mut machine.editor, &mut effects);
            }
        }
    }
    (machine, effects)
}

/// Moves the editor's queued effects and its notice into `effects`.
fn collect_requests(editor: &mut Editor, effects: &mut Vec<Effect>) {
    effects.extend(editor.take_effects());
    if let Some(notice) = editor.take_notice() {
        effects.push(Effect::Notify(notice));
    }
}
