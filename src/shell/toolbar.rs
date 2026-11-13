//! Minimal toolbar: layout, hit-testing and drawing.
//!
//! A vertical strip at the left edge: one button per tool, the six palette
//! swatches, the helper toggles (smart snap, grid snap, numbering), then undo
//! and redo. While the grid tool is active, a flyout right of the `G` button
//! shows the grid's columns and rows with `-`/`+` buttons (ADR-T18-2).
//! Layout, hit-testing and event routing are pure functions of the
//! viewport; [`draw`] paints the strip with the theme tokens (ADR-0012),
//! marking the active tool, colour and the helpers that are on with `accent`.

use macroquad::color::Color;

use crate::core::command::{Command, Tool};
use crate::core::editor::{Editor, PAGE_COUNT};
use crate::core::geom::{Aabb, Vec2};
use crate::core::input::InputEvent;
use crate::core::palette::{ColorId, PALETTE_LEN, Rgba, palette};
use crate::shell::render::{Batch, GlSink, hex_color, hex_color_alpha, to_mq_color};
use crate::shell::{ui_shapes, ui_text};
use crate::theme;

/// Side of a square button in pixels.
pub const BUTTON_PX: f32 = 32.0;
/// Space between the strip edges and the buttons, in pixels.
pub const PAD_PX: f32 = 4.0;
/// Space between buttons of one group, in pixels.
pub const GAP_PX: f32 = 2.0;
/// Space between the window edge and the first island, in pixels.
pub const OUTER_PX: f32 = 3.0;
/// Width of an island's border ring, in pixels (ADR-T126-1).
const ISLAND_BORDER_PX: f32 = 1.0;
/// Space between an island's border and its buttons, in pixels.
const ISLAND_PAD_PX: f32 = 3.0;
/// Space between two islands, in pixels.
const ISLAND_GAP_PX: f32 = 3.0;
/// Corner radius of an island, in pixels.
const ISLAND_RADIUS_PX: f32 = 8.0;
/// Corner radius of an active button, in pixels.
const BUTTON_RADIUS_PX: f32 = 6.0;
/// Corner radius of a colour swatch, in pixels.
const SWATCH_RADIUS_PX: f32 = 4.0;
/// Border plus padding: how far an island's frame lies outside its buttons.
const INSET_PX: f32 = ISLAND_BORDER_PX + ISLAND_PAD_PX;
/// Left edge of the button column.
const COLUMN_X_PX: f32 = OUTER_PX + INSET_PX;

/// Tools in toolbar order (the keymap's order).
pub const TOOLS: [Tool; 11] = [
    Tool::Pen,
    Tool::Line,
    Tool::Arrow,
    Tool::Rect,
    Tool::Ellipse,
    Tool::Grid,
    Tool::Text,
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
    /// Changes the grid's columns by the given step (flyout).
    GridCols(i32),
    /// Changes the grid's rows by the given step (flyout).
    GridRows(i32),
    /// Undoes the last action.
    Undo,
    /// Redoes the last undone action.
    Redo,
    /// Shows a page, `0` for page 1 (the switcher under undo and redo).
    Page(usize),
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
            Self::GridCols(delta) => Command::GridCols(delta),
            Self::GridRows(delta) => Command::GridRows(delta),
            Self::Undo => Command::Undo,
            Self::Redo => Command::Redo,
            Self::Page(page) => Command::GoToPage(page),
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
#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    /// To the editor's canvas.
    Forward(InputEvent),
    /// A toolbar button was pressed: run this command.
    Apply(Command),
    /// A press on the toolbar outside any button: drop it.
    Swallow,
}

/// Width of the toolbar strip in pixels.
pub const STRIP_PX: f32 = BUTTON_PX + 2.0 * COLUMN_X_PX;

/// Width in pixels of the active-item outline.
const OUTLINE_PX: f32 = 2.0;
/// Space between a colour swatch and the ring of the current colour.
const SWATCH_RING_GAP_PX: f32 = 3.0;
/// Inset in pixels of a colour swatch inside its button.
const SWATCH_INSET_PX: f32 = 6.0;
/// Size of a tool letter, in pixels.
const LABEL_SIZE: u16 = 20;
/// Side of a flyout `-`/`+` button, in pixels.
const FLY_BUTTON_PX: f32 = 24.0;
/// Width of a flyout row label (`cols`, `rows`), in pixels.
const FLY_LABEL_PX: f32 = 40.0;
/// Width of a flyout value cell, in pixels.
const FLY_VALUE_PX: f32 = 28.0;
/// Height of the `pen` badge, in pixels: it fits the 20 px left under the
/// buttons in the default 800 px window.
const BADGE_PX: f32 = 14.0;
/// Gap between the `pen` badge and the bottom of the strip, in pixels.
const BADGE_INSET_PX: f32 = 2.0;
/// Size of the `pen` badge's text, in pixels.
const BADGE_LABEL_SIZE: u16 = 12;
/// Size of the flyout row labels, in pixels.
const FLY_LABEL_SIZE: u16 = 16;

/// The strip background for a viewport: full height at the left edge.
#[must_use]
pub fn panel(viewport: Vec2) -> Aabb {
    Aabb::from_corners(Vec2::ZERO, Vec2::new(STRIP_PX, viewport.y))
}

/// The buttons of one island stacked from `top` (the island's own top
/// edge), and the `top` of the next island.
fn stack(kinds: impl IntoIterator<Item = ButtonKind>, top: f32) -> (Vec<Button>, f32) {
    let mut y = top + INSET_PX;
    let buttons: Vec<Button> = kinds
        .into_iter()
        .map(|kind| {
            let min = Vec2::new(COLUMN_X_PX, y);
            y += BUTTON_PX + GAP_PX;
            Button {
                rect: Aabb::from_corners(min, min + Vec2::new(BUTTON_PX, BUTTON_PX)),
                kind,
            }
        })
        .collect();
    let bottom = buttons.last().map_or(top, |b| b.rect.max.y + INSET_PX);
    (buttons, bottom + ISLAND_GAP_PX)
}

/// The frame of an island around `buttons`; `None` for no buttons.
fn frame(buttons: &[Button]) -> Option<Aabb> {
    let (first, last) = (buttons.first()?, buttons.last()?);
    Some(Aabb::from_corners(
        Vec2::new(OUTER_PX, first.rect.min.y - INSET_PX),
        Vec2::new(
            OUTER_PX + BUTTON_PX + 2.0 * INSET_PX,
            last.rect.max.y + INSET_PX,
        ),
    ))
}

/// The four islands of the strip, top to bottom: tools, colours, helpers,
/// undo and redo; and the top of the island that would follow.
fn groups() -> (Vec<Vec<Button>>, f32) {
    let colors = (0..PALETTE_LEN)
        .filter_map(|i| u8::try_from(i).ok().and_then(ColorId::new))
        .map(ButtonKind::Color);
    let kinds: [Vec<ButtonKind>; 4] = [
        TOOLS.map(ButtonKind::Tool).to_vec(),
        colors.collect(),
        vec![
            ButtonKind::SmartSnap,
            ButtonKind::GridSnap,
            ButtonKind::Numbering,
        ],
        vec![ButtonKind::Undo, ButtonKind::Redo],
    ];
    let mut top = OUTER_PX;
    let mut groups = Vec::with_capacity(kinds.len());
    for group in kinds {
        let (buttons, next) = stack(group, top);
        groups.push(buttons);
        top = next;
    }
    (groups, top)
}

/// The buttons, top to bottom: tools, colours, helpers, undo, redo.
///
/// The layout is fixed; the viewport is taken for symmetry with [`panel`].
#[must_use]
pub fn layout(_viewport: Vec2) -> Vec<Button> {
    groups().0.into_iter().flatten().collect()
}

/// Height of the bar under the number of a page that holds shapes.
const PAGE_BAR_PX: f32 = 2.0;
/// Space between the bar of a used page and the cell's left and right.
const PAGE_BAR_INSET_PX: f32 = 9.0;
/// Space between that bar and the bottom of the cell.
const PAGE_BAR_LIFT_PX: f32 = 4.0;

/// The page switcher (ADR-T125-1): one square button per page, 1…`PAGE_COUNT`,
/// one per row like the colours, in an island under the history buttons.
/// Buttons whose island would cross the bottom of the window are left out.
#[must_use]
pub fn page_layout(viewport: Vec2) -> Vec<Button> {
    let (_, top) = groups();
    let (buttons, _) = stack((0..PAGE_COUNT).map(ButtonKind::Page), top);
    buttons
        .into_iter()
        .filter(|cell| cell.rect.max.y + INSET_PX <= viewport.y)
        .collect()
}

/// The frames of the islands: the four of the strip and, when any page
/// button fits, the one around the page buttons (ADR-T126-1).
#[must_use]
pub fn islands(viewport: Vec2) -> Vec<Aabb> {
    let mut frames: Vec<Aabb> = groups().0.iter().filter_map(|g| frame(g)).collect();
    frames.extend(frame(&page_layout(viewport)));
    frames
}

/// Where the `pen` badge sits while pen taps are held still (T34): the
/// bottom of the strip, below every button.
#[must_use]
pub fn pen_badge(viewport: Vec2) -> Aabb {
    let max = Vec2::new(COLUMN_X_PX + BUTTON_PX, viewport.y - BADGE_INSET_PX);
    Aabb::from_corners(max - Vec2::new(BUTTON_PX, BADGE_PX), max)
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

/// The grid size flyout: right of the strip, top aligned with the `G`
/// button (ADR-T18-2).
#[must_use]
pub fn flyout_panel(viewport: Vec2) -> Aabb {
    let top = layout(viewport)
        .iter()
        .find(|b| b.kind == ButtonKind::Tool(Tool::Grid))
        .map_or(PAD_PX, |b| b.rect.min.y);
    let width = 2.0 * PAD_PX + FLY_LABEL_PX + FLY_VALUE_PX + 2.0 * FLY_BUTTON_PX + 3.0 * GAP_PX;
    let height = 2.0 * PAD_PX + 2.0 * FLY_BUTTON_PX + GAP_PX;
    let min = Vec2::new(STRIP_PX + GAP_PX, top);
    Aabb::from_corners(min, min + Vec2::new(width, height))
}

/// The flyout's buttons: columns `-`, `+`, then rows `-`, `+`.
#[must_use]
pub fn flyout_layout(viewport: Vec2) -> Vec<Button> {
    let origin = flyout_panel(viewport).min + Vec2::new(PAD_PX, PAD_PX);
    let minus_x = FLY_LABEL_PX + GAP_PX;
    let plus_x = minus_x + FLY_BUTTON_PX + GAP_PX + FLY_VALUE_PX + GAP_PX;
    let row_y = FLY_BUTTON_PX + GAP_PX;
    let button = |x: f32, y: f32, kind| {
        let min = origin + Vec2::new(x, y);
        let rect = Aabb::from_corners(min, min + Vec2::new(FLY_BUTTON_PX, FLY_BUTTON_PX));
        Button { rect, kind }
    };
    vec![
        button(minus_x, 0.0, ButtonKind::GridCols(-1)),
        button(plus_x, 0.0, ButtonKind::GridCols(1)),
        button(minus_x, row_y, ButtonKind::GridRows(-1)),
        button(plus_x, row_y, ButtonKind::GridRows(1)),
    ]
}

/// Routes `event`: pointer presses on the visible toolbar, or on the grid
/// size flyout when `flyout` is shown, never reach the canvas; everything
/// else is forwarded.
#[must_use]
pub fn route(visible: bool, flyout: bool, viewport: Vec2, event: InputEvent) -> Route {
    match event {
        InputEvent::PointerDown { pos, .. } if visible && panel(viewport).contains(pos) => {
            let mut buttons = layout(viewport);
            buttons.extend(page_layout(viewport));
            hit(&buttons, pos).map_or(Route::Swallow, Route::Apply)
        }
        InputEvent::PointerDown { pos, .. }
            if visible && flyout && flyout_panel(viewport).contains(pos) =>
        {
            hit(&flyout_layout(viewport), pos).map_or(Route::Swallow, Route::Apply)
        }
        InputEvent::PointerDown { .. }
        | InputEvent::PointerMove { .. }
        | InputEvent::PointerUp { .. }
        | InputEvent::Scroll { .. }
        | InputEvent::KeyDown { .. }
        | InputEvent::KeyUp { .. }
        | InputEvent::Char { .. }
        | InputEvent::Pressure { .. }
        | InputEvent::Paste(_)
        | InputEvent::PasteGrid(_)
        | InputEvent::Resize { .. } => Route::Forward(event),
    }
}

/// Draws one island: its translucent fill and its thin border ring
/// (ADR-T126-1).
fn draw_island(frame: Aabb) {
    ui_shapes::fill(
        frame,
        ISLAND_RADIUS_PX,
        hex_color_alpha(theme::MENU_ISLAND, theme::MENU_ISLAND_ALPHA),
    );
    ui_shapes::ring(
        frame,
        ISLAND_RADIUS_PX,
        ISLAND_BORDER_PX,
        hex_color_alpha(theme::MENU_BORDER, theme::MENU_BORDER_ALPHA),
    );
}

/// Draws the toolbar for `editor` in a viewport of `viewport` pixels, with
/// the grid size flyout while the grid tool is active.
pub fn draw(editor: &Editor, viewport: Vec2) {
    for frame in islands(viewport) {
        draw_island(frame);
    }
    let accent = hex_color(theme::TOOLBAR_ACTIVE);
    let text = hex_color(theme::TOOLBAR_TEXT);
    for button in layout(viewport) {
        let rect = button.rect;
        match button.kind {
            ButtonKind::Tool(tool) => {
                draw_toggle(rect, tool_label(tool), editor.tool() == tool, accent, text);
            }
            ButtonKind::Color(id) => {
                let swatch = rect.expand(-SWATCH_INSET_PX);
                ui_shapes::fill(swatch, SWATCH_RADIUS_PX, to_mq_color(palette(id)));
                if editor.style().color == id {
                    let ring = swatch.expand(SWATCH_RING_GAP_PX);
                    ui_shapes::ring(ring, BUTTON_RADIUS_PX, OUTLINE_PX, accent);
                }
            }
            ButtonKind::SmartSnap => {
                draw_toggle(
                    rect,
                    SMART_SNAP_LABEL,
                    editor.helpers().smart_snap,
                    accent,
                    text,
                );
            }
            ButtonKind::GridSnap => {
                draw_toggle(rect, "#", editor.helpers().grid_snap, accent, text);
            }
            ButtonKind::Numbering => {
                draw_toggle(rect, "N", editor.helpers().numbering, accent, text);
            }
            ButtonKind::GridCols(delta) | ButtonKind::GridRows(delta) => {
                draw_label(rect, step_label(delta), text);
            }
            ButtonKind::Undo => draw_history_arrow(rect, -1.0, history_color(editor.can_undo())),
            ButtonKind::Redo => draw_history_arrow(rect, 1.0, history_color(editor.can_redo())),
            ButtonKind::Page(_) => {}
        }
    }
    for cell in page_layout(viewport) {
        if let ButtonKind::Page(page) = cell.kind {
            draw_page_cell(cell.rect, page, page_look(editor, page));
        }
    }
    if editor.tap_slop() > 0.0 {
        draw_label_sized(pen_badge(viewport), "pen", accent, BADGE_LABEL_SIZE);
    }
    if editor.tool() == Tool::Grid {
        draw_flyout(editor, viewport);
    }
}

/// How a page cell looks (ADR-T124-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLook {
    /// The page on screen: filled.
    Current,
    /// Another page that holds shapes: full colour and a bar.
    Used,
    /// Another page with nothing on it: dimmed.
    Empty,
}

/// The look of page `page` for `editor`.
#[must_use]
pub fn page_look(editor: &Editor, page: usize) -> PageLook {
    if editor.page() == page {
        PageLook::Current
    } else if editor.page_has_shapes(page) {
        PageLook::Used
    } else {
        PageLook::Empty
    }
}

/// Draws the cell of page `page` (zero-based) in `rect`.
fn draw_page_cell(rect: Aabb, page: usize, look: PageLook) {
    let accent = hex_color(theme::TOOLBAR_ACTIVE);
    let label = (page + 1).to_string();
    match look {
        PageLook::Current => {
            ui_shapes::fill(
                rect,
                BUTTON_RADIUS_PX,
                hex_color(theme::TOOLBAR_ACTIVE_FILL),
            );
            draw_label(rect, &label, hex_color(theme::MENU_ACTIVE_BRIGHT));
        }
        PageLook::Used => {
            draw_label(rect, &label, hex_color(theme::TOOLBAR_TEXT));
            ui_shapes::rect_xywh(
                rect.min.x + PAGE_BAR_INSET_PX,
                rect.max.y - PAGE_BAR_LIFT_PX - PAGE_BAR_PX,
                rect.width() - 2.0 * PAGE_BAR_INSET_PX,
                PAGE_BAR_PX,
                accent,
            );
        }
        PageLook::Empty => draw_label(rect, &label, hex_color(theme::MENU_MUTED)),
    }
}

/// Draws the grid size flyout with the current columns and rows of
/// `editor`.
fn draw_flyout(editor: &Editor, viewport: Vec2) {
    let fly = flyout_panel(viewport);
    draw_island(fly);
    let border = hex_color(theme::MENU_BORDER);
    let text = hex_color(theme::TOOLBAR_TEXT);
    let helpers = editor.helpers();
    // The row's name, left of its first button.
    let name_box = |rect: Aabb| {
        Aabb::from_corners(
            Vec2::new(fly.min.x + PAD_PX, rect.min.y),
            Vec2::new(rect.min.x - GAP_PX, rect.max.y),
        )
    };
    for button in flyout_layout(viewport) {
        let rect = button.rect;
        ui_shapes::ring(rect, BUTTON_RADIUS_PX, ISLAND_BORDER_PX, border);
        let (delta, name, value) = match button.kind {
            ButtonKind::GridCols(delta) => (delta, "cols", helpers.grid_cols),
            ButtonKind::GridRows(delta) => (delta, "rows", helpers.grid_rows),
            ButtonKind::Tool(_)
            | ButtonKind::Color(_)
            | ButtonKind::SmartSnap
            | ButtonKind::GridSnap
            | ButtonKind::Numbering
            | ButtonKind::Undo
            | ButtonKind::Redo
            | ButtonKind::Page(_) => continue,
        };
        draw_label(rect, step_label(delta), text);
        if delta < 0 {
            // The value right of `-`.
            draw_label_sized(name_box(rect), name, text, FLY_LABEL_SIZE);
            let cell = Aabb::from_corners(
                Vec2::new(rect.max.x + GAP_PX, rect.min.y),
                Vec2::new(rect.max.x + GAP_PX + FLY_VALUE_PX, rect.max.y),
            );
            draw_label(cell, &value.to_string(), text);
        }
    }
}

/// The label of a `-`/`+` step button.
fn step_label(delta: i32) -> &'static str {
    if delta < 0 { "-" } else { "+" }
}

/// Label of the smart snap toggle: the key that toggles it (see
/// `core::keymap`).
const SMART_SNAP_LABEL: &str = "X";

/// The key that selects `tool`, shown as its label (see `core::keymap`).
fn tool_label(tool: Tool) -> &'static str {
    match tool {
        Tool::Pen => "D",
        Tool::Line => "W",
        Tool::Arrow => "A",
        Tool::Rect => "R",
        Tool::Ellipse => "C",
        Tool::Grid => "G",
        Tool::Text => "T",
        Tool::Eraser => "E",
        Tool::Bucket => "B",
        Tool::Select => "V",
        Tool::Hand => "H",
    }
}

/// Draws a labelled button: a rounded fill and the bright accent when `on`.
fn draw_toggle(rect: Aabb, label: &str, on: bool, _accent: Color, text: Color) {
    if on {
        ui_shapes::fill(
            rect,
            BUTTON_RADIUS_PX,
            hex_color(theme::TOOLBAR_ACTIVE_FILL),
        );
    }
    let bright = hex_color(theme::MENU_ACTIVE_BRIGHT);
    draw_label(rect, label, if on { bright } else { text });
}

/// Colour of an undo/redo arrow: the toolbar text colour, faded by
/// `theme::TOOLBAR_DISABLED_ALPHA` when not `enabled`.
fn history_color(enabled: bool) -> Color {
    let text = Rgba::from_hex(theme::TOOLBAR_TEXT);
    if enabled {
        to_mq_color(text)
    } else {
        to_mq_color(text.faded(theme::TOOLBAR_DISABLED_ALPHA))
    }
}

/// Draws `label` centred in `rect`.
fn draw_label(rect: Aabb, label: &str, color: Color) {
    draw_label_sized(rect, label, color, LABEL_SIZE);
}

/// Draws `label` centred in `rect` at `font_size` pixels.
fn draw_label_sized(rect: Aabb, label: &str, color: Color, font_size: u16) {
    let size = ui_text::measure(label, font_size);
    let c = rect.center();
    let x = c.x - size.width * 0.5;
    let y = c.y + size.offset_y * 0.5;
    ui_text::draw(label, x, y, font_size, color);
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
    let mut batch = Batch::new(GlSink);
    batch.rect(
        Aabb::from_corners(
            Vec2::new(base.x.min(tail), c.y - bar * 0.5),
            Vec2::new(base.x.min(tail) + (base.x - tail).abs(), c.y + bar * 0.5),
        ),
        color,
    );
    batch.triangle(
        tip,
        Vec2::new(base.x, c.y - head),
        Vec2::new(base.x, c.y + head),
        color,
    );
    batch.finish();
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
            Tool::Text,
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
        assert_eq!(buttons.len(), 22);
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
        assert!(gap(10) > gap(9), "gap after the tools");
        assert!(gap(16) > gap(15), "gap after the colours");
        assert!(gap(19) > gap(18), "gap after the helpers");
    }

    #[test]
    fn layout_fits_default_window() {
        let buttons = layout(VIEWPORT);
        let Some(last) = buttons.last() else {
            panic!("toolbar has buttons");
        };
        assert!(last.rect.max.y <= VIEWPORT.y, "{last:?}");
    }

    // ---- T124 page switcher ------------------------------------------------

    /// A window tall enough for the whole switcher.
    const TALL: Vec2 = Vec2::new(1280.0, 1080.0);

    #[test]
    fn page_cells_are_one_column_below_the_history_buttons() {
        // Act
        let cells = page_layout(TALL);

        // Assert
        let kinds: Vec<ButtonKind> = cells.iter().map(|c| c.kind).collect();
        let expected: Vec<ButtonKind> = (0..PAGE_COUNT).map(ButtonKind::Page).collect();
        assert_eq!(kinds, expected);
        let history_bottom = layout(TALL).last().map_or(0.0, |b| b.rect.max.y);
        for (i, cell) in cells.iter().enumerate() {
            assert!(approx_eq(cell.rect.min.x, COLUMN_X_PX, EPS), "{cell:?}");
            assert!(approx_eq(cell.rect.width(), BUTTON_PX, EPS), "{cell:?}");
            assert!(approx_eq(cell.rect.height(), BUTTON_PX, EPS), "{cell:?}");
            assert!(cell.rect.min.y > history_bottom, "{cell:?}");
            if i > 0 {
                assert!(cell.rect.min.y > cells[i - 1].rect.max.y, "one per row");
            }
        }
    }

    #[test]
    fn page_cells_do_not_overlap_buttons() {
        let buttons = layout(TALL);
        for cell in page_layout(TALL) {
            for button in &buttons {
                assert!(!cell.rect.intersects(&button.rect), "{cell:?} / {button:?}");
            }
        }
        let strip = panel(TALL);
        for cell in page_layout(TALL) {
            assert!(strip.contains(cell.rect.min) && strip.contains(cell.rect.max));
        }
    }

    #[test]
    fn page_cells_that_do_not_fit_are_left_out() {
        // Act: the default window is too short for the whole switcher.
        let cells = page_layout(VIEWPORT);

        // Assert
        assert!(cells.len() < PAGE_COUNT);
        assert!(cells.iter().all(|c| c.rect.max.y <= VIEWPORT.y));
        assert_eq!(page_layout(Vec2::new(1280.0, 100.0)), []);
    }

    #[test]
    fn page_cell_click_goes_to_its_page() {
        // Arrange
        let Some(cell) = page_layout(TALL).get(2).copied() else {
            panic!("no third cell");
        };

        // Act
        let routed = route(
            true,
            false,
            TALL,
            press(cell.rect.center(), PointerButton::Left),
        );

        // Assert
        assert_eq!(routed, Route::Apply(Command::GoToPage(2)));
    }

    #[test]
    fn page_cell_look_follows_the_editor() {
        // Arrange: a rectangle on page 1.
        let mut ed = Editor::new();
        ed.apply(Command::SetTool(Tool::Rect));
        let (a, b) = (Vec2::new(10.0, 10.0), Vec2::new(60.0, 40.0));
        ed.handle(press(a, PointerButton::Left));
        ed.handle(InputEvent::PointerUp {
            pos: b,
            button: PointerButton::Left,
            mods: Modifiers::NONE,
        });

        // Assert
        assert_eq!(page_look(&ed, 0), PageLook::Current);
        assert_eq!(page_look(&ed, 1), PageLook::Empty);
        ed.apply(Command::GoToPage(1));
        assert_eq!(page_look(&ed, 1), PageLook::Current);
        assert_eq!(page_look(&ed, 0), PageLook::Used);
    }

    // ---- T126 islands ------------------------------------------------------

    #[test]
    fn islands_frame_their_groups() {
        // Act
        let frames = islands(TALL);
        let buttons = layout(TALL);

        // Assert: four islands for the strip, a fifth for the pages; each
        // holds its buttons with the border and padding around them.
        assert_eq!(frames.len(), 5);
        let counts = [11, 6, 3, 2];
        let mut at = 0;
        for (frame, count) in frames.iter().zip(counts) {
            let group = &buttons[at..at + count];
            at += count;
            for b in group {
                assert!(
                    frame.contains(b.rect.min) && frame.contains(b.rect.max),
                    "{b:?}"
                );
            }
            assert!(approx_eq(frame.min.y, group[0].rect.min.y - INSET_PX, EPS));
            assert!(approx_eq(
                frame.max.y,
                group[count - 1].rect.max.y + INSET_PX,
                EPS
            ));
            assert!(approx_eq(frame.min.x, OUTER_PX, EPS));
            assert!(approx_eq(frame.width(), BUTTON_PX + 2.0 * INSET_PX, EPS));
        }
    }

    #[test]
    fn islands_fit_the_default_window() {
        // Act
        let frames = islands(VIEWPORT);

        // Assert: the four islands of the strip, no page island in 800 px.
        assert_eq!(frames.len(), 4);
        for frame in &frames {
            assert!(frame.max.y <= VIEWPORT.y, "{frame:?}");
            assert!(frame.max.x <= STRIP_PX + EPS, "{frame:?}");
        }
        let badge = pen_badge(VIEWPORT);
        assert!(
            badge.min.y >= frames[3].max.y - EPS,
            "{badge:?} under the last island"
        );
    }

    #[test]
    fn islands_do_not_overlap() {
        let frames = islands(TALL);
        for pair in frames.windows(2) {
            assert!(
                pair[1].min.y >= pair[0].max.y + ISLAND_GAP_PX - EPS,
                "{pair:?}"
            );
        }
    }

    #[test]
    fn page_island_frames_the_page_buttons() {
        let cells = page_layout(TALL);
        let frames = islands(TALL);
        let Some(last) = frames.last() else {
            panic!("no islands");
        };
        for cell in &cells {
            assert!(last.contains(cell.rect.min) && last.contains(cell.rect.max));
        }
        assert!(last.max.y <= TALL.y);
        // A window too short for any page button has no fifth island.
        assert_eq!(islands(Vec2::new(1280.0, 790.0)).len(), 4);
    }

    #[test]
    fn strip_swallows_a_press_between_islands() {
        // Arrange: the gap between the tools and the colours.
        let frames = islands(VIEWPORT);
        let pos = Vec2::new(
            OUTER_PX + 4.0,
            f32::midpoint(frames[0].max.y, frames[1].min.y),
        );

        // Act / Assert
        let routed = route(true, false, VIEWPORT, press(pos, PointerButton::Left));
        assert_eq!(routed, Route::Swallow);
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

    // T34

    #[test]
    fn pen_badge_sits_in_the_strip_below_the_buttons() {
        // Arrange
        let strip = panel(VIEWPORT);
        let buttons = layout(VIEWPORT);
        let Some(last) = buttons.last() else {
            panic!("toolbar has buttons");
        };

        // Act
        let badge = pen_badge(VIEWPORT);

        // Assert
        assert!(
            strip.contains(badge.min) && strip.contains(badge.max),
            "{badge:?}"
        );
        assert!(badge.min.y > last.rect.max.y, "{badge:?} below {last:?}");
        assert!(badge.width() > 0.0 && badge.height() > 0.0);
        assert!(hit(&buttons, badge.center()).is_none());
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
                ButtonKind::GridCols(d) => Command::GridCols(d),
                ButtonKind::GridRows(d) => Command::GridRows(d),
                ButtonKind::Undo => Command::Undo,
                ButtonKind::Redo => Command::Redo,
                ButtonKind::Page(page) => Command::GoToPage(page),
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
            f32::midpoint(first.max.y, buttons[1].rect.min.y),
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
            false,
            VIEWPORT,
            press(rect_tool.rect.center(), PointerButton::Left),
        );
        // Assert
        assert_eq!(routed, Route::Apply(Command::SetTool(Tool::Rect)));
        for button in [PointerButton::Right, PointerButton::Middle] {
            let routed = route(
                true,
                false,
                VIEWPORT,
                press(rect_tool.rect.center(), button),
            );
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
            route(true, false, VIEWPORT, press(bottom, PointerButton::Left)),
            Route::Swallow
        );
    }

    #[test]
    fn route_hidden_toolbar_forwards() {
        let center = layout(VIEWPORT)[0].rect.center();
        let event = press(center, PointerButton::Left);
        assert_eq!(
            route(false, false, VIEWPORT, event.clone()),
            Route::Forward(event)
        );
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
            assert_eq!(
                route(true, false, VIEWPORT, event.clone()),
                Route::Forward(event)
            );
        }
    }

    // AC-6 (T18): grid size flyout

    /// The toolbar's grid tool button.
    fn grid_button() -> Button {
        match layout(VIEWPORT)
            .into_iter()
            .find(|b| b.kind == ButtonKind::Tool(Tool::Grid))
        {
            Some(button) => button,
            None => panic!("toolbar has a grid button"),
        }
    }

    #[test]
    fn flyout_has_four_buttons_in_order() {
        // Act
        let kinds: Vec<ButtonKind> = flyout_layout(VIEWPORT).iter().map(|b| b.kind).collect();
        // Assert
        assert_eq!(
            kinds,
            [
                ButtonKind::GridCols(-1),
                ButtonKind::GridCols(1),
                ButtonKind::GridRows(-1),
                ButtonKind::GridRows(1),
            ]
        );
    }

    #[test]
    fn flyout_is_right_of_strip_aligned_with_grid_button() {
        let fly = flyout_panel(VIEWPORT);
        let strip = panel(VIEWPORT);
        assert!(fly.min.x >= strip.max.x, "{fly:?} overlaps the strip");
        assert!(approx_eq(fly.min.y, grid_button().rect.min.y, EPS));
        assert!(fly.max.x <= VIEWPORT.x && fly.max.y <= VIEWPORT.y);
        assert!(fly.width() > 0.0 && fly.height() > 0.0);
    }

    #[test]
    fn flyout_buttons_inside_panel_and_disjoint() {
        let fly = flyout_panel(VIEWPORT);
        let buttons = flyout_layout(VIEWPORT);
        for b in &buttons {
            assert!(
                fly.contains(b.rect.min) && fly.contains(b.rect.max),
                "{b:?}"
            );
        }
        for (i, a) in buttons.iter().enumerate() {
            for b in &buttons[i + 1..] {
                assert!(!a.rect.intersects(&b.rect), "{a:?} overlaps {b:?}");
            }
        }
        // Columns on the first row, rows below; minus left of plus.
        assert!(buttons[0].rect.max.y < buttons[2].rect.min.y);
        assert!(buttons[0].rect.max.x < buttons[1].rect.min.x);
        assert!(buttons[2].rect.max.x < buttons[3].rect.min.x);
    }

    #[test]
    fn flyout_buttons_return_their_commands() {
        let buttons = flyout_layout(VIEWPORT);
        let expected = [
            Command::GridCols(-1),
            Command::GridCols(1),
            Command::GridRows(-1),
            Command::GridRows(1),
        ];
        assert_eq!(buttons.len(), expected.len());
        for (b, want) in buttons.iter().zip(expected) {
            assert_eq!(hit(&buttons, b.rect.center()), Some(want), "{b:?}");
        }
    }

    #[test]
    fn route_flyout_click_applies() {
        // Arrange
        let plus_rows = flyout_layout(VIEWPORT)[3];
        // Act
        let routed = route(
            true,
            true,
            VIEWPORT,
            press(plus_rows.rect.center(), PointerButton::Left),
        );
        // Assert
        assert_eq!(routed, Route::Apply(Command::GridRows(1)));
        // The strip still works with the flyout shown.
        let rect_tool = layout(VIEWPORT)[3].rect.center();
        assert_eq!(
            route(true, true, VIEWPORT, press(rect_tool, PointerButton::Left)),
            Route::Apply(Command::SetTool(Tool::Rect))
        );
    }

    #[test]
    fn route_flyout_gap_is_swallowed() {
        let fly = flyout_panel(VIEWPORT);
        let corner = fly.min + Vec2::new(1.0, 1.0);
        assert_eq!(hit(&flyout_layout(VIEWPORT), corner), None);
        for button in [PointerButton::Left, PointerButton::Right] {
            assert_eq!(
                route(true, true, VIEWPORT, press(corner, button)),
                Route::Swallow
            );
        }
    }

    #[test]
    fn route_without_flyout_forwards() {
        let minus_cols = flyout_layout(VIEWPORT)[0].rect.center();
        let event = press(minus_cols, PointerButton::Left);
        assert_eq!(
            route(true, false, VIEWPORT, event.clone()),
            Route::Forward(event.clone())
        );
        assert_eq!(
            route(false, true, VIEWPORT, event.clone()),
            Route::Forward(event)
        );
    }

    /// The key bound bare to `command` in the keymap, as the toolbar spells it.
    fn bound_key(command: Command) -> String {
        let chord = crate::core::keymap::BINDINGS
            .iter()
            .find(|&&(chord, c, _)| c == command && chord.mods == Modifiers::NONE)
            .map(|&(chord, _, _)| chord);
        let Some(chord) = chord else {
            panic!("{command:?} has no bare key");
        };
        format!("{:?}", chord.key)
    }

    #[test]
    fn tool_labels_match_keymap() {
        for tool in [
            Tool::Pen,
            Tool::Line,
            Tool::Arrow,
            Tool::Rect,
            Tool::Ellipse,
            Tool::Grid,
            Tool::Text,
            Tool::Eraser,
            Tool::Bucket,
            Tool::Select,
            Tool::Hand,
        ] {
            assert_eq!(
                tool_label(tool),
                bound_key(Command::SetTool(tool)),
                "{tool:?}"
            );
        }
    }

    #[test]
    fn smart_snap_label_matches_keymap() {
        assert_eq!(SMART_SNAP_LABEL, bound_key(Command::ToggleSmartSnap));
    }
}
