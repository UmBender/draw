//! Binary entry point: starts the crash hook and the diagnostics log,
//! checks the system, then opens the window and hands control to the
//! shell.

use draw::shell::diag::{self, Level};

fn main() {
    draw::shell::crash::install_hook();
    diag::init_from_env();
    if let Err(problem) = draw::shell::preflight::run() {
        eprintln!("draw: {problem}");
        diag::log(Level::Error, &format!("preflight: {problem}"));
        diag::finish("preflight failed");
        std::process::exit(1);
    }
    macroquad::Window::from_config(draw::shell::app::window_conf(), draw::shell::app::run());
    diag::finish("window closed");
}
