//! Minimal toolbar: layout, hit-testing and drawing.
//!
//! A vertical strip at the left edge: one button per tool, the six palette
//! swatches, the helper toggles (smart snap, grid snap, numbering), then undo
//! and redo. Layout, hit-testing and event routing are pure functions of the
//! viewport; [`draw`] paints the strip with the theme tokens (ADR-0012),
//! marking the active tool, colour and the helpers that are on with `accent`.

use macroquad::color::Color;
use macroquad::shapes::{draw_rectangle, draw_triangle};
use macroquad::text::{draw_text, measure_text};

use crate::core::command::{Command, Tool};
use crate::core::editor::Editor;
use crate::core::geom::{Aabb, Vec2};
use crate::core::input::InputEvent;
use crate::core::palette::{ColorId, PALETTE_LEN, THEME, palette};
use crate::shell::render::{to_mq, to_mq_color};

/// Side of a square button in pixels.
pub const BUTTON_PX: f32 = 32.0;
/// Space between the strip edges and the buttons, in pixels.
pub const PAD_PX: f32 = 4.0;
/// Space between buttons of one group, in pixels.
pub const GAP_PX: f32 = 2.0;
/// Space between groups (tools, colours, helpers, history), in pixels.
pub const GROUP_GAP_PX: f32 = 12.0;

/// Tools in toolbar order (the keymap's order).
pub const TOOLS: [Tool; 10] = [
    Tool::Pen,
    Tool::Line,
    Tool::Arrow,
    Tool::Rect,
    Tool::Ellipse,
    Tool::Grid,
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
    /// Toggles smart snapping.
    SmartSnap,
    /// Toggles grid snapping.
    GridSnap,
    /// Toggles auto-numbering.
    Numbering,
    /// Undoes the last action.
    Undo,
    /// Redoes the last undone action.
    Redo,
}

impl ButtonKind {
    /// The command a click on this button runs.
    #[must_use]
    pub fn command(self) -> Command {
        match self {
            Self::Tool(tool) => Command::SetTool(tool),
            Self::Color(id) => Command::SetColor(id),
            Self::SmartSnap => Command::ToggleSmartSnap,
            Self::GridSnap => Command::ToggleGridSnap,
            Self::Numbering => Command::ToggleNumbering,
            Self::Undo => Command::Undo,
            Self::Redo => Command::Redo,
        }
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

/// Width of the toolbar strip in pixels.
pub const STRIP_PX: f32 = BUTTON_PX + 2.0 * PAD_PX;

/// Width in pixels of the active-item outline.
const OUTLINE_PX: f32 = 2.0;
/// Inset in pixels of a colour swatch inside its button.
const SWATCH_INSET_PX: f32 = 6.0;
/// Size of a tool letter, in pixels.
const LABEL_SIZE: u16 = 20;
/// Opacity of an unavailable undo/redo button.
const DISABLED_ALPHA: f32 = 0.3;

/// The strip background for a viewport: full height at the left edge.
#[must_use]
pub fn panel(viewport: Vec2) -> Aabb {
    Aabb::from_corners(Vec2::ZERO, Vec2::new(STRIP_PX, viewport.y))
}

/// The buttons, top to bottom: tools, colours, helpers, undo, redo.
///
/// The layout is fixed; the viewport is taken for symmetry with [`panel`].
#[must_use]
pub fn layout(_viewport: Vec2) -> Vec<Button> {
    let colors = (0..PALETTE_LEN)
        .filter_map(|i| u8::try_from(i).ok().and_then(ColorId::new))
        .map(ButtonKind::Color);
    let groups: [Vec<ButtonKind>; 4] = [
        TOOLS.map(ButtonKind::Tool).to_vec(),
        colors.collect(),
        vec![
            ButtonKind::SmartSnap,
            ButtonKind::GridSnap,
            ButtonKind::Numbering,
        ],
        vec![ButtonKind::Undo, ButtonKind::Redo],
    ];
    let mut buttons = Vec::with_capacity(groups.iter().map(Vec::len).sum());
    let mut y = PAD_PX;
    for (g, group) in groups.into_iter().enumerate() {
        if g > 0 {
            y += GROUP_GAP_PX - GAP_PX;
        }
        for kind in group {
            let min = Vec2::new(PAD_PX, y);
            let rect = Aabb::from_corners(min, min + Vec2::new(BUTTON_PX, BUTTON_PX));
            buttons.push(Button { rect, kind });
            y += BUTTON_PX + GAP_PX;
        }
    }
    buttons
}

/// The command of the button under `pos`, `None` if there is none or `pos`
/// is not finite.
#[must_use]
pub fn hit(buttons: &[Button], pos: Vec2) -> Option<Command> {
    let pos = pos.sanitize()?;
    buttons
        .iter()
        .find(|b| b.rect.contains(pos))
        .map(|b| b.kind.command())
}

/// Routes `event`: pointer presses on the visible toolbar never reach the
/// canvas; everything else is forwarded.
#[must_use]
pub fn route(visible: bool, viewport: Vec2, event: InputEvent) -> Route {
    match event {
        InputEvent::PointerDown { pos, .. } if visible && panel(viewport).contains(pos) => {
            hit(&layout(viewport), pos).map_or(Route::Swallow, Route::Apply)
        }
        _ => Route::Forward(event),
    }
}

/// Draws the toolbar for `editor` in a viewport of `viewport` pixels.
pub fn draw(editor: &Editor, viewport: Vec2) {
    let strip = panel(viewport);
    fill(strip, to_mq_color(THEME.surface));
    let border = to_mq_color(THEME.border);
    draw_rectangle(strip.max.x - 1.0, 0.0, 1.0, strip.height(), border);

    let accent = to_mq_color(THEME.accent);
    let text = to_mq_color(THEME.text);
    for button in layout(viewport) {
        let rect = button.rect;
        match button.kind {
            ButtonKind::Tool(tool) => {
                let active = editor.tool() == tool;
                if active {
                    fill(rect, to_mq_color(THEME.selection));
                    outline(rect, accent);
                }
                draw_label(rect, tool_label(tool), if active { accent } else { text });
            }
            ButtonKind::Color(id) => {
                fill(rect.expand(-SWATCH_INSET_PX), to_mq_color(palette(id)));
                if editor.style().color == id {
                    outline(rect, accent);
                }
            }
            ButtonKind::SmartSnap | ButtonKind::GridSnap | ButtonKind::Numbering => {
                let helpers = editor.helpers();
                let (on, label) = match button.kind {
                    ButtonKind::SmartSnap => (helpers.smart_snap, "M"),
                    ButtonKind::GridSnap => (helpers.grid_snap, "#"),
                    _ => (helpers.numbering, "N"),
                };
                if on {
                    fill(rect, to_mq_color(THEME.selection));
                    outline(rect, accent);
                }
                draw_label(rect, label, if on { accent } else { text });
            }
            ButtonKind::Undo => draw_history_arrow(rect, -1.0, enabled(text, editor.can_undo())),
            ButtonKind::Redo => draw_history_arrow(rect, 1.0, enabled(text, editor.can_redo())),
        }
    }
}

/// The key that selects `tool`, shown as its label (see `core::keymap`).
fn tool_label(tool: Tool) -> &'static str {
    match tool {
        Tool::Pen => "P",
        Tool::Line => "L",
        Tool::Arrow => "A",
        Tool::Rect => "R",
        Tool::Ellipse => "C",
        Tool::Grid => "G",
        Tool::Eraser => "E",
        Tool::Bucket => "B",
        Tool::Select => "V",
        Tool::Hand => "H",
    }
}

/// `color`, faded when not `enabled`.
fn enabled(color: Color, enabled: bool) -> Color {
    if enabled {
        color
    } else {
        Color {
            a: color.a * DISABLED_ALPHA,
            ..color
        }
    }
}

/// Fills a screen rectangle.
fn fill(rect: Aabb, color: Color) {
    draw_rectangle(rect.min.x, rect.min.y, rect.width(), rect.height(), color);
}

/// Draws an [`OUTLINE_PX`] outline just inside a screen rectangle.
fn outline(rect: Aabb, color: Color) {
    let Aabb { min, max } = rect;
    let (width, height) = (rect.width(), rect.height());
    let band = OUTLINE_PX;
    draw_rectangle(min.x, min.y, width, band, color);
    draw_rectangle(min.x, max.y - band, width, band, color);
    draw_rectangle(min.x, min.y, band, height, color);
    draw_rectangle(max.x - band, min.y, band, height, color);
}

/// Draws `label` centred in `rect`.
fn draw_label(rect: Aabb, label: &str, color: Color) {
    let size = measure_text(label, None, LABEL_SIZE, 1.0);
    let c = rect.center();
    let x = c.x - size.width * 0.5;
    let y = c.y + size.offset_y * 0.5;
    draw_text(label, x, y, f32::from(LABEL_SIZE), color);
}

/// Draws a horizontal arrow in `rect`, pointing left (`dir < 0`) or right.
fn draw_history_arrow(rect: Aabb, dir: f32, color: Color) {
    let c = rect.center();
    let half = BUTTON_PX * 0.25;
    let head = BUTTON_PX * 0.2;
    let tip = Vec2::new(c.x + dir * half, c.y);
    let base = Vec2::new(tip.x - dir * head, c.y);
    let tail = c.x - dir * half;
    let bar = 2.0;
    draw_rectangle(
        base.x.min(tail),
        c.y - bar * 0.5,
        (base.x - tail).abs(),
        bar,
        color,
    );
    draw_triangle(
        to_mq(tip),
        to_mq(Vec2::new(base.x, c.y - head)),
        to_mq(Vec2::new(base.x, c.y + head)),
        color,
    );
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
            Tool::Grid,
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
            .chain([
                ButtonKind::SmartSnap,
                ButtonKind::GridSnap,
                ButtonKind::Numbering,
            ])
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
        assert_eq!(buttons.len(), 21);
    }

    #[test]
    fn layout_buttons_are_32px_in_left_strip() {
        let buttons = layout(VIEWPORT);
        let x0 = buttons[0].rect.min.x;
        assert!(
            (0.0..BUTTON_PX).contains(&x0),
            "strip starts at the left edge"
        );
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
        // Groups (tools | colours | helpers | history) get a larger gap.
        let gap = |i: usize| buttons[i + 1].rect.min.y - buttons[i].rect.max.y;
        assert!(gap(9) > gap(8), "gap after the tools");
        assert!(gap(15) > gap(14), "gap after the colours");
        assert!(gap(18) > gap(17), "gap after the helpers");
    }

    #[test]
    fn layout_fits_default_window() {
        let buttons = layout(VIEWPORT);
        let Some(last) = buttons.last() else {
            panic!("toolbar has buttons");
        };
        assert!(last.rect.max.y <= VIEWPORT.y, "{last:?}");
    }

    #[test]
    fn layout_buttons_inside_panel() {
        let strip = panel(VIEWPORT);
        assert!(approx_eq(strip.min.x, 0.0, EPS));
        assert!(approx_eq(strip.min.y, 0.0, EPS));
        assert!(approx_eq(strip.max.y, VIEWPORT.y, EPS));
        assert!(strip.width() < 2.0 * BUTTON_PX, "narrow strip");
        for b in layout(VIEWPORT) {
            assert!(
                strip.contains(b.rect.min) && strip.contains(b.rect.max),
                "{b:?}"
            );
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
                ButtonKind::SmartSnap => Command::ToggleSmartSnap,
                ButtonKind::GridSnap => Command::ToggleGridSnap,
                ButtonKind::Numbering => Command::ToggleNumbering,
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
        let between = Vec2::new(
            first.center().x,
            (first.max.y + buttons[1].rect.min.y) * 0.5,
        );
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
        let routed = route(
            true,
            VIEWPORT,
            press(rect_tool.rect.center(), PointerButton::Left),
        );
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
