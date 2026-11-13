//! The command palette overlay: layout and drawing (ADR-T111-1).
//!
//! A panel near the top of the window with the query line and up to
//! [`VISIBLE_ROWS`] rows, each a description and its key. It is drawn on
//! top of everything, only while the editor holds an open palette, and it
//! never asks for a frame of its own.

use crate::shell::{ui_shapes, ui_text};
use macroquad::color::Color;

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

/// Corner radius of the panel, in pixels.
const PANEL_RADIUS_PX: f32 = 8.0;
/// Corner radius of the selected row and of a key cap, in pixels.
const ROW_RADIUS_PX: f32 = 6.0;
/// Text size inside a key cap, in pixels.
const CAP_TEXT_PX: u16 = 14;
/// Space left and right of the text inside a key cap, in pixels.
const CAP_PAD_PX: f32 = 6.0;
/// Space between two key caps, in pixels.
const CAP_GAP_PX: f32 = 4.0;
/// Space above and below a key cap inside its row, in pixels.
const CAP_MARGIN_PX: f32 = 5.0;

/// The keys of a chord label (`Ctrl+Shift+[`), one per key cap.
#[must_use]
pub fn chord_keys(label: &str) -> Vec<&str> {
    label.split('+').filter(|key| !key.is_empty()).collect()
}

/// The key caps for keys of the given text `widths`, in a row whose content
/// ends at `right`, laid right to left: each as `(left, width)`.
#[must_use]
pub fn key_caps(widths: &[f32], right: f32) -> Vec<(f32, f32)> {
    let mut caps = Vec::with_capacity(widths.len());
    let mut edge = right;
    for text in widths.iter().rev() {
        let width = text + 2.0 * CAP_PAD_PX;
        edge -= width;
        caps.push((edge, width));
        edge -= CAP_GAP_PX;
    }
    caps.reverse();
    caps
}

/// Draws the palette over the canvas of `viewport` pixels.
pub fn draw(palette: &CommandPalette, viewport: Vec2) {
    let text = hex_color(theme::TOOLBAR_TEXT);
    let accent = hex_color(theme::TOOLBAR_ACTIVE);
    ui_shapes::rect_xywh(
        0.0,
        0.0,
        viewport.x,
        viewport.y,
        hex_color_alpha(theme::CANVAS, DIM_ALPHA),
    );
    let window = visible_window(palette.selected(), palette.len(), VISIBLE_ROWS);
    let rect = panel(viewport, window.len().max(1));
    ui_shapes::fill(
        rect,
        PANEL_RADIUS_PX,
        hex_color_alpha(theme::MENU_ISLAND, theme::MENU_PANEL_ALPHA),
    );
    let edge = hex_color_alpha(theme::MENU_BORDER, theme::MENU_BORDER_ALPHA);
    ui_shapes::ring(rect, PANEL_RADIUS_PX, 1.0, edge);

    let x = rect.min.x + PAD_PX;
    let right = rect.max.x - PAD_PX;
    let mut y = rect.min.y + PAD_PX;
    let query = format!("> {}_", palette.query());
    draw_row_text(&query, x, y, accent);
    y += ROW_PX;
    // A thin line under the query.
    ui_shapes::rect_xywh(x, y - 1.0, right - x, 1.0, edge);
    if palette.is_empty() {
        draw_row_text("No matching command", x, y, hex_color(theme::MENU_DIM_TEXT));
        return;
    }
    for (offset, (chord, _, description)) in palette.rows().skip(window.start).enumerate() {
        if offset >= window.len() {
            break;
        }
        let selected = window.start + offset == palette.selected();
        if selected {
            let band = Aabb {
                min: Vec2::new(rect.min.x + 4.0, y),
                max: Vec2::new(rect.max.x - 4.0, y + ROW_PX),
            };
            ui_shapes::fill(band, ROW_RADIUS_PX, hex_color(theme::TOOLBAR_ACTIVE_FILL));
        }
        let color = if selected {
            hex_color(theme::MENU_ACTIVE_BRIGHT)
        } else {
            text
        };
        draw_row_text(description, x, y, color);
        draw_key_caps(&chord_label(chord), right, y);
        y += ROW_PX;
    }
}

/// Draws the keys of `label` as key caps ending at `right`, in the row
/// starting at `y`.
fn draw_key_caps(label: &str, right: f32, y: f32) {
    let keys = chord_keys(label);
    let widths: Vec<f32> = keys
        .iter()
        .map(|key| ui_text::measure(key, CAP_TEXT_PX).width)
        .collect();
    for (key, (left, width)) in keys.iter().zip(key_caps(&widths, right)) {
        let cap = Aabb {
            min: Vec2::new(left, y + CAP_MARGIN_PX),
            max: Vec2::new(left + width, y + ROW_PX - CAP_MARGIN_PX),
        };
        ui_shapes::fill(cap, ROW_RADIUS_PX - 2.0, hex_color(theme::MENU_KEY_FILL));
        ui_shapes::ring(
            cap,
            ROW_RADIUS_PX - 2.0,
            1.0,
            hex_color_alpha(theme::MENU_BORDER, theme::MENU_BORDER_ALPHA),
        );
        let size = ui_text::measure(key, CAP_TEXT_PX);
        ui_text::draw(
            key,
            left + CAP_PAD_PX,
            cap.center().y + size.offset_y * 0.5,
            CAP_TEXT_PX,
            hex_color(theme::MENU_DIM_TEXT),
        );
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
    fn key_caps_split_a_chord() {
        assert_eq!(chord_keys("Ctrl+Shift+["), ["Ctrl", "Shift", "["]);
        assert_eq!(chord_keys("D"), ["D"]);
        assert_eq!(chord_keys("Alt+Left"), ["Alt", "Left"]);
        assert_eq!(chord_keys("").len(), 0);
    }

    #[test]
    fn key_caps_fit_inside_the_row() {
        // Arrange: three keys ending at the right edge of the content.
        let right = 500.0;
        let widths = [30.0, 40.0, 8.0];

        // Act
        let caps = key_caps(&widths, right);

        // Assert: in order, apart, the last ending exactly at `right`.
        assert_eq!(caps.len(), 3);
        let (last_left, last_width) = caps[2];
        assert!(approx_eq(last_left + last_width, right, 1e-3));
        for pair in caps.windows(2) {
            let (left, width) = pair[0];
            assert!(left + width + CAP_GAP_PX <= pair[1].0 + 1e-3, "{pair:?}");
        }
        for (&(_, width), text) in caps.iter().zip(widths) {
            assert!(approx_eq(width, text + 2.0 * CAP_PAD_PX, 1e-3));
        }
        assert_eq!(key_caps(&[], right).len(), 0);
    }

    #[test]
    fn panel_never_has_negative_width() {
        let rect = panel(Vec2::new(5.0, 5.0), 1);
        assert!(rect.width() >= 0.0);
    }
}
