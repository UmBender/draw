//! Window configuration and the main loop.
//!
//! The loop is completed by T12 (input, idle redraw, toolbar).

use macroquad::color::BLACK;
use macroquad::window::{Conf, clear_background, next_frame};

/// Window settings used by `main`.
#[must_use]
pub fn window_conf() -> Conf {
    Conf {
        window_title: "draw".to_owned(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        high_dpi: true,
        ..Conf::default()
    }
}

/// Runs the application until the window is closed.
///
/// Bootstrap version: clears the window every frame. T12 replaces it with the
/// redraw-on-demand loop from ADR-0006.
pub async fn run() {
    loop {
        clear_background(BLACK);
        next_frame().await;
    }
}
