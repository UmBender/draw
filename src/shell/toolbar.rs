//! Minimal toolbar: layout, hit-testing and drawing.
//!
//! A vertical strip at the left edge: one button per tool, the six palette
//! swatches, then undo and redo. Layout, hit-testing and event routing are
//! pure functions of the viewport; [`draw`] paints the strip with the theme
//! tokens (ADR-0012), marking the active tool and colour with `accent`.

use crate::core::command::{Command, Tool};
use crate::core::editor::Editor;
use crate::core::geom::{Aabb, Vec2};
use crate::core::input::InputEvent;
use crate::core::palette::ColorId;

/// Side of a square button in pixels.
pub const BUTTON_PX: f32 = 32.0;
/// Space between the strip edges and the buttons, in pixels.
pub const PAD_PX: f32 = 4.0;
/// Space between buttons of one group, in pixels.
pub const GAP_PX: f32 = 2.0;
/// Space between groups (tools, colours, history), in pixels.
pub const GROUP_GAP_PX: f32 = 12.0;

/// Tools in toolbar order (the keymap's order).
pub const TOOLS: [Tool; 9] = [
    Tool::Pen,
    Tool::Line,
    Tool::Arrow,
    Tool::Rect,
    Tool::Ellipse,
    Tool::Eraser,
    Tool::Bucket,
    Tool::Select,
    Tool::Hand,
];

/// What a toolbar button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonKind {
    /// Selects a tool.
    Tool(Tool),
    /// Picks a palette colour.
    Color(ColorId),
    /// Undoes the last action.
    Undo,
    /// Redoes the last undone action.
    Redo,
}

impl ButtonKind {
    /// The command a click on this button runs.
    #[must_use]
    pub fn command(self) -> Command {
        todo!()
    }
}

/// A toolbar button and its screen rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Button {
    /// Screen rectangle in pixels.
    pub rect: Aabb,
    /// What the button does.
    pub kind: ButtonKind,
}

/// Where an input event goes once the toolbar has seen it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Route {
    /// To the editor's canvas.
    Forward(InputEvent),
    /// A toolbar button was pressed: run this command.
    Apply(Command),
    /// A press on the toolbar outside any button: drop it.
    Swallow,
}

/// The strip background for a viewport: full height at the left edge.
#[must_use]
pub fn panel(viewport: Vec2) -> Aabb {
    let _ = viewport;
    todo!()
}

/// The buttons, top to bottom: tools, colours, undo, redo.
#[must_use]
pub fn layout(viewport: Vec2) -> Vec<Button> {
    let _ = viewport;
    todo!()
}

/// The command of the button under `pos`, `None` if there is none or `pos`
/// is not finite.
#[must_use]
pub fn hit(buttons: &[Button], pos: Vec2) -> Option<Command> {
    let _ = (buttons, pos);
    todo!()
}

/// Routes `event`: pointer presses on the visible toolbar never reach the
/// canvas; everything else is forwarded.
#[must_use]
pub fn route(visible: bool, viewport: Vec2, event: InputEvent) -> Route {
    let _ = (visible, viewport, event);
    todo!()
}

/// Draws the toolbar for `editor` in a viewport of `viewport` pixels.
pub fn draw(editor: &Editor, viewport: Vec2) {
    let _ = (editor, viewport);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::{Command, Tool};
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use crate::core::palette::{ColorId, PALETTE_LEN};

    const EPS: f32 = 1e-4;
    const VIEWPORT: Vec2 = Vec2::new(1280.0, 800.0);

    fn expected_kinds() -> Vec<ButtonKind> {
        let tools = [
            Tool::Pen,
            Tool::Line,
            Tool::Arrow,
            Tool::Rect,
            Tool::Ellipse,
            Tool::Eraser,
            Tool::Bucket,
            Tool::Select,
            Tool::Hand,
        ]
        .map(ButtonKind::Tool);
        let colors = (0..PALETTE_LEN as u8)
            .filter_map(ColorId::new)
            .map(ButtonKind::Color);
        tools
            .into_iter()
            .chain(colors)
            .chain([ButtonKind::Undo, ButtonKind::Redo])
            .collect()
    }

    fn press(pos: Vec2, button: PointerButton) -> InputEvent {
        InputEvent::PointerDown {
            pos,
            button,
            mods: Modifiers::NONE,
        }
    }

    // AC-2

    #[test]
    fn layout_has_every_button_in_order() {
        // Act
        let buttons = layout(VIEWPORT);
        // Assert
        let kinds: Vec<ButtonKind> = buttons.iter().map(|b| b.kind).collect();
        assert_eq!(kinds, expected_kinds());
        assert_eq!(buttons.len(), 17);
    }

    #[test]
    fn layout_buttons_are_32px_in_left_strip() {
        let buttons = layout(VIEWPORT);
        let x0 = buttons[0].rect.min.x;
        assert!(x0 >= 0.0 && x0 < BUTTON_PX, "strip starts at the left edge");
        for b in &buttons {
            assert!(approx_eq(b.rect.width(), 32.0, EPS), "{b:?}");
            assert!(approx_eq(b.rect.height(), 32.0, EPS), "{b:?}");
            assert!(approx_eq(b.rect.min.x, x0, EPS), "one column: {b:?}");
        }
    }

    #[test]
    fn layout_buttons_do_not_overlap() {
        let buttons = layout(VIEWPORT);
        for pair in buttons.windows(2) {
            assert!(
                pair[1].rect.min.y > pair[0].rect.max.y,
                "stacked top to bottom with a gap: {pair:?}"
            );
        }
        // Groups (tools | colours | history) get a larger gap.
        let gap = |i: usize| buttons[i + 1].rect.min.y - buttons[i].rect.max.y;
        assert!(gap(8) > gap(7), "gap after the tools");
        assert!(gap(14) > gap(13), "gap after the colours");
    }

    #[test]
    fn layout_buttons_inside_panel() {
        let strip = panel(VIEWPORT);
        assert!(approx_eq(strip.min.x, 0.0, EPS));
        assert!(approx_eq(strip.min.y, 0.0, EPS));
        assert!(approx_eq(strip.max.y, VIEWPORT.y, EPS));
        assert!(strip.width() < 2.0 * BUTTON_PX, "narrow strip");
        for b in layout(VIEWPORT) {
            assert!(strip.contains(b.rect.min) && strip.contains(b.rect.max), "{b:?}");
        }
    }

    // AC-3

    #[test]
    fn hit_each_button_returns_its_command() {
        // Arrange
        let buttons = layout(VIEWPORT);
        for b in &buttons {
            let expected = match b.kind {
                ButtonKind::Tool(tool) => Command::SetTool(tool),
                ButtonKind::Color(id) => Command::SetColor(id),
                ButtonKind::Undo => Command::Undo,
                ButtonKind::Redo => Command::Redo,
            };
            // Act
            let got = hit(&buttons, b.rect.center());
            // Assert
            assert_eq!(got, Some(expected), "{b:?}");
            assert_eq!(b.kind.command(), expected);
        }
    }

    #[test]
    fn hit_outside_buttons_is_none() {
        let buttons = layout(VIEWPORT);
        let first = buttons[0].rect;
        let between = Vec2::new(first.center().x, (first.max.y + buttons[1].rect.min.y) * 0.5);
        for pos in [Vec2::new(600.0, 400.0), between, Vec2::new(-5.0, 10.0)] {
            assert_eq!(hit(&buttons, pos), None, "{pos:?}");
        }
    }

    #[test]
    fn hit_non_finite_is_none() {
        let buttons = layout(VIEWPORT);
        for pos in [
            Vec2::new(f32::NAN, 20.0),
            Vec2::new(20.0, f32::INFINITY),
            Vec2::new(f32::NEG_INFINITY, f32::NAN),
        ] {
            assert_eq!(hit(&buttons, pos), None);
        }
    }

    #[test]
    fn toolbar_click_not_forwarded() {
        // Arrange
        let buttons = layout(VIEWPORT);
        let rect_tool = buttons[3];
        // Act
        let routed = route(true, VIEWPORT, press(rect_tool.rect.center(), PointerButton::Left));
        // Assert
        assert_eq!(routed, Route::Apply(Command::SetTool(Tool::Rect)));
        for button in [PointerButton::Right, PointerButton::Middle] {
            let routed = route(true, VIEWPORT, press(rect_tool.rect.center(), button));
            assert!(
                !matches!(routed, Route::Forward(_)),
                "{button:?} press on the toolbar reached the canvas"
            );
        }
    }

    #[test]
    fn route_press_between_buttons_is_swallowed() {
        let strip = panel(VIEWPORT);
        let bottom = Vec2::new(strip.center().x, strip.max.y - 1.0);
        assert_eq!(
            route(true, VIEWPORT, press(bottom, PointerButton::Left)),
            Route::Swallow
        );
    }

    #[test]
    fn route_hidden_toolbar_forwards() {
        let center = layout(VIEWPORT)[0].rect.center();
        let event = press(center, PointerButton::Left);
        assert_eq!(route(false, VIEWPORT, event), Route::Forward(event));
    }

    #[test]
    fn route_canvas_and_other_events_forward() {
        let on_toolbar = layout(VIEWPORT)[0].rect.center();
        let events = [
            press(Vec2::new(600.0, 400.0), PointerButton::Left),
            InputEvent::PointerMove {
                pos: on_toolbar,
                mods: Modifiers::NONE,
            },
            InputEvent::PointerUp {
                pos: on_toolbar,
                button: PointerButton::Left,
                mods: Modifiers::NONE,
            },
            InputEvent::Scroll {
                pos: on_toolbar,
                delta: 1.0,
            },
            InputEvent::KeyDown {
                key: Key::Tab,
                mods: Modifiers::NONE,
            },
            InputEvent::Resize { size: VIEWPORT },
        ];
        for event in events {
            assert_eq!(route(true, VIEWPORT, event), Route::Forward(event));
        }
    }
}
