//! Binary entry point: opens the window and hands control to the shell.

fn main() {
    macroquad::Window::from_config(draw::shell::app::window_conf(), draw::shell::app::run());
}
