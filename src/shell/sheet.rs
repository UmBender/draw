//! The shortcut sheet overlay: layout and drawing (ADR-T113-1).
//!
//! A centred panel with the topic names on top (the current one
//! highlighted), the page title, up to [`crate::core::sheet::ROWS_PER_PAGE`] rows of key and
//! description, and a footer with the sheet's own keys. It is drawn on top of
//! everything, only while the editor holds an open sheet, and never asks for
//! a frame of its own.

use crate::shell::{ui_shapes, ui_text};
use macroquad::color::Color;
use macroquad::shapes::draw_rectangle;

use crate::core::geom::{Aabb, Vec2};
use crate::core::keymap::Topic;
use crate::core::sheet::Sheet;
use crate::shell::render::{hex_color, hex_color_alpha};
use crate::theme;

/// Widest the panel gets, in pixels.
const PANEL_WIDTH_PX: f32 = 880.0;
/// Space kept at the left and right of the window, in pixels.
const MARGIN_PX: f32 = 20.0;
/// Height of each line, in pixels.
const ROW_PX: f32 = 30.0;
/// Space around the content inside the panel, in pixels.
const PAD_PX: f32 = 14.0;
/// Text size in pixels.
const TEXT_PX: u16 = 18;
/// Width of the key column, in pixels.
const KEY_COLUMN_PX: f32 = 300.0;
/// Space between two topic names, in pixels.
const TAB_GAP_PX: f32 = 4.0;
/// Alpha of the dimming layer under the panel.
const DIM_ALPHA: u8 = 110;
/// Opacity of the stripe behind every second row of the sheet.
const STRIPE_ALPHA: u8 = 140;
/// Corner radius of the panel, in pixels.
const PANEL_RADIUS_PX: f32 = 8.0;
/// Corner radius of a topic pill, in pixels.
const PILL_RADIUS_PX: f32 = 6.0;
/// Space left and right of a topic name inside its pill, in pixels.
const PILL_PAD_PX: f32 = 8.0;
/// Space above and below a topic pill inside its line, in pixels.
const PILL_MARGIN_PX: f32 = 3.0;
/// The footer: the sheet's own keys.
const FOOTER: &str = "Left/Right page    1-9 topic    ? or Esc close";

/// The panel's rectangle for `lines` lines of content, in a window of
/// `viewport` pixels: centred, as wide as fits.
#[must_use]
pub fn panel(viewport: Vec2, lines: usize) -> Aabb {
    let width = PANEL_WIDTH_PX.min((viewport.x - 2.0 * MARGIN_PX).max(0.0));
    let height = ROW_PX * lines as f32 + 2.0 * PAD_PX;
    let left = (viewport.x - width) * 0.5;
    let top = ((viewport.y - height) * 0.5).max(0.0);
    Aabb {
        min: Vec2::new(left, top),
        max: Vec2::new(left + width, top + height),
    }
}

/// The topic labels as `(label, x, line)` for a panel `width` wide: laid
/// left to right and wrapped to a new line when the next one does not fit.
fn tab_layout(labels: &[String], widths: &[f32], width: f32) -> Vec<(f32, usize)> {
    let mut x = 0.0;
    let mut line = 0;
    let mut out = Vec::with_capacity(labels.len());
    for w in widths {
        if x > 0.0 && x + w > width {
            x = 0.0;
            line += 1;
        }
        out.push((x, line));
        x += w + TAB_GAP_PX;
    }
    out
}

/// Draws the sheet over the canvas of `viewport` pixels.
pub fn draw(sheet: &Sheet, viewport: Vec2) {
    let text = hex_color(theme::TOOLBAR_TEXT);
    let accent = hex_color(theme::TOOLBAR_ACTIVE);
    draw_rectangle(
        0.0,
        0.0,
        viewport.x,
        viewport.y,
        hex_color_alpha(theme::CANVAS, DIM_ALPHA),
    );
    let page = sheet.current();
    let rows = page.rows();
    let labels: Vec<String> = Topic::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| format!("{} {}", i + 1, t.name()))
        .collect();
    let widths: Vec<f32> = labels
        .iter()
        .map(|l| ui_text::measure(l, TEXT_PX).width + 2.0 * PILL_PAD_PX)
        .collect();
    let inner =
        (PANEL_WIDTH_PX.min((viewport.x - 2.0 * MARGIN_PX).max(0.0)) - 2.0 * PAD_PX).max(0.0);
    let tabs = tab_layout(&labels, &widths, inner);
    let tab_lines = tabs.last().map_or(1, |&(_, line)| line + 1);
    // Tabs, title, rows, footer.
    let rect = panel(viewport, tab_lines + 1 + rows.len().max(1) + 1);
    ui_shapes::fill(
        rect,
        PANEL_RADIUS_PX,
        hex_color_alpha(theme::MENU_ISLAND, theme::MENU_PANEL_ALPHA),
    );
    let edge = hex_color_alpha(theme::MENU_BORDER, theme::MENU_BORDER_ALPHA);
    ui_shapes::ring(rect, PANEL_RADIUS_PX, 1.0, edge);

    let x = rect.min.x + PAD_PX;
    let mut y = rect.min.y + PAD_PX;
    for (i, ((label, &(dx, line)), width)) in labels.iter().zip(&tabs).zip(&widths).enumerate() {
        let current = Topic::ALL.get(i) == Some(&page.topic);
        let top = y + line as f32 * ROW_PX;
        if current {
            let pill = Aabb {
                min: Vec2::new(x + dx, top + PILL_MARGIN_PX),
                max: Vec2::new(x + dx + width, top + ROW_PX - PILL_MARGIN_PX),
            };
            ui_shapes::fill(pill, PILL_RADIUS_PX, hex_color(theme::TOOLBAR_ACTIVE_FILL));
        }
        let color = if current {
            hex_color(theme::MENU_ACTIVE_BRIGHT)
        } else {
            hex_color(theme::MENU_DIM_TEXT)
        };
        draw_row_text(label, x + dx + PILL_PAD_PX, top, color);
    }
    y += tab_lines as f32 * ROW_PX;
    draw_row_text(&page.title(), x, y, hex_color(theme::MENU_ACTIVE_BRIGHT));
    y += ROW_PX;
    for (i, row) in rows.iter().enumerate() {
        if i % 2 == 1 {
            let stripe = Aabb {
                min: Vec2::new(rect.min.x + 4.0, y),
                max: Vec2::new(rect.max.x - 4.0, y + ROW_PX),
            };
            ui_shapes::fill(
                stripe,
                PILL_RADIUS_PX,
                hex_color_alpha(theme::MENU_KEY_FILL, STRIPE_ALPHA),
            );
        }
        draw_row_text(&row.keys, x, y, accent);
        draw_row_text(row.text, x + KEY_COLUMN_PX, y, text);
        y += ROW_PX;
    }
    draw_row_text(FOOTER, x, y, hex_color(theme::MENU_MUTED));
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

    /// The overlays draw with the built-in font, which has no arrows
    /// (T122).
    #[test]
    fn footer_is_ascii() {
        assert!(FOOTER.is_ascii(), "{FOOTER}");
    }
    use crate::core::geom::approx_eq;
    use crate::core::sheet::ROWS_PER_PAGE;

    /// The tallest page: the tab lines, title, rows and footer.
    const TALLEST_LINES: usize = 2 + 1 + ROWS_PER_PAGE + 1;

    #[test]
    fn panel_is_centred_and_fits_the_default_window() {
        // Arrange / Act
        let rect = panel(Vec2::new(1280.0, 800.0), TALLEST_LINES);

        // Assert
        assert!(approx_eq(rect.center().x, 640.0, 1e-3));
        assert!(rect.min.y >= 0.0 && rect.max.y <= 800.0);
        assert!(rect.width() <= PANEL_WIDTH_PX + 1e-3);
    }

    #[test]
    fn panel_shrinks_in_a_narrow_window() {
        let rect = panel(Vec2::new(300.0, 400.0), 5);
        assert!(rect.min.x >= 0.0 && rect.max.x <= 300.0);
    }

    #[test]
    fn panel_never_has_negative_width() {
        assert!(panel(Vec2::new(5.0, 5.0), 1).width() >= 0.0);
    }

    #[test]
    fn tabs_wrap_when_the_line_is_full() {
        // Arrange
        let labels = vec![String::new(); 4];
        let widths = [100.0, 100.0, 100.0, 100.0];

        // Act
        let tabs = tab_layout(&labels, &widths, 250.0);

        // Assert: two per line.
        let lines: Vec<usize> = tabs.iter().map(|&(_, line)| line).collect();
        assert_eq!(lines, [0, 0, 1, 1]);
        assert!(approx_eq(tabs[0].0, 0.0, 1e-3));
        assert!(approx_eq(tabs[2].0, 0.0, 1e-3));
    }
}
