//! The command palette overlay: layout and drawing (ADR-T111-1).
//!
//! A panel near the top of the window with the query line and up to
//! [`VISIBLE_ROWS`] rows, each a description and its key. It is drawn on
//! top of everything, only while the editor holds an open palette, and it
//! never asks for a frame of its own.

use crate::shell::ui_text;
use macroquad::color::Color;
use macroquad::shapes::draw_rectangle;

use crate::core::command_palette::{CommandPalette, VISIBLE_ROWS, chord_label, visible_window};
use crate::core::geom::{Aabb, Vec2};
use crate::shell::render::{hex_color, hex_color_alpha};
use crate::theme;

/// Widest the panel gets, in pixels.
const PANEL_WIDTH_PX: f32 = 520.0;
/// Space kept at the left and right of the window, in pixels.
const MARGIN_PX: f32 = 20.0;
/// Distance of the panel from the top of the window, in pixels.
const TOP_PX: f32 = 60.0;
/// Height of the query line and of each row, in pixels.
const ROW_PX: f32 = 30.0;
/// Space around the content inside the panel, in pixels.
const PAD_PX: f32 = 10.0;
/// Text size in pixels.
const TEXT_PX: u16 = 18;
/// Alpha of the dimming layer under the panel.
const DIM_ALPHA: u8 = 90;

/// The panel's rectangle for `rows` rows plus the query line, in a window
/// of `viewport` pixels: centred, as wide as fits.
#[must_use]
pub fn panel(viewport: Vec2, rows: usize) -> Aabb {
    let width = PANEL_WIDTH_PX.min((viewport.x - 2.0 * MARGIN_PX).max(0.0));
    let height = ROW_PX * (rows as f32 + 1.0) + 2.0 * PAD_PX;
    let left = (viewport.x - width) * 0.5;
    Aabb {
        min: Vec2::new(left, TOP_PX),
        max: Vec2::new(left + width, TOP_PX + height),
    }
}

/// Draws the palette over the canvas of `viewport` pixels.
pub fn draw(palette: &CommandPalette, viewport: Vec2) {
    let text = hex_color(theme::TOOLBAR_TEXT);
    let accent = hex_color(theme::TOOLBAR_ACTIVE);
    draw_rectangle(
        0.0,
        0.0,
        viewport.x,
        viewport.y,
        hex_color_alpha(theme::CANVAS, DIM_ALPHA),
    );
    let window = visible_window(palette.selected(), palette.len(), VISIBLE_ROWS);
    let rect = panel(viewport, window.len().max(1));
    fill(rect, hex_color(theme::TOOLBAR_BACKGROUND));
    outline(rect, hex_color(theme::TOOLBAR_BORDER));

    let x = rect.min.x + PAD_PX;
    let right = rect.max.x - PAD_PX;
    let mut y = rect.min.y + PAD_PX;
    let query = format!("> {}_", palette.query());
    draw_row_text(&query, x, y, accent);
    y += ROW_PX;
    if palette.is_empty() {
        draw_row_text("No matching command", x, y, text);
        return;
    }
    for (offset, (chord, _, description)) in palette.rows().skip(window.start).enumerate() {
        if offset >= window.len() {
            break;
        }
        if window.start + offset == palette.selected() {
            let band = Aabb {
                min: Vec2::new(rect.min.x + 2.0, y),
                max: Vec2::new(rect.max.x - 2.0, y + ROW_PX),
            };
            fill(band, hex_color(theme::TOOLBAR_ACTIVE_FILL));
        }
        draw_row_text(description, x, y, text);
        let label = chord_label(chord);
        let width = ui_text::measure(&label, TEXT_PX).width;
        draw_row_text(&label, right - width, y, accent);
        y += ROW_PX;
    }
}

/// Draws `label` vertically centred in the row starting at `y`.
fn draw_row_text(label: &str, x: f32, y: f32, color: Color) {
    let size = ui_text::measure(label, TEXT_PX);
    ui_text::draw(
        label,
        x,
        y + ROW_PX * 0.5 + size.offset_y * 0.5,
        TEXT_PX,
        color,
    );
}

/// Fills a screen rectangle.
fn fill(rect: Aabb, color: Color) {
    draw_rectangle(rect.min.x, rect.min.y, rect.width(), rect.height(), color);
}

/// Draws a 1 px outline just inside a screen rectangle.
fn outline(rect: Aabb, color: Color) {
    let (w, h) = (rect.width(), rect.height());
    draw_rectangle(rect.min.x, rect.min.y, w, 1.0, color);
    draw_rectangle(rect.min.x, rect.max.y - 1.0, w, 1.0, color);
    draw_rectangle(rect.min.x, rect.min.y, 1.0, h, color);
    draw_rectangle(rect.max.x - 1.0, rect.min.y, 1.0, h, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;

    #[test]
    fn panel_is_centred_and_capped_in_width() {
        // Arrange / Act
        let rect = panel(Vec2::new(1280.0, 720.0), VISIBLE_ROWS);

        // Assert
        assert!(approx_eq(rect.width(), PANEL_WIDTH_PX, 1e-3));
        assert!(approx_eq(rect.center().x, 640.0, 1e-3));
        assert!(rect.max.y < 720.0);
    }

    #[test]
    fn panel_shrinks_in_a_narrow_window() {
        // Arrange / Act
        let rect = panel(Vec2::new(300.0, 400.0), 3);

        // Assert
        assert!(approx_eq(rect.width(), 300.0 - 2.0 * MARGIN_PX, 1e-3));
        assert!(rect.min.x >= 0.0 && rect.max.x <= 300.0);
    }

    #[test]
    fn panel_never_has_negative_width() {
        let rect = panel(Vec2::new(5.0, 5.0), 1);
        assert!(rect.width() >= 0.0);
    }
}
