//! Binary entry point: starts the crash hook, the diagnostics log and the
//! signal handlers, checks the system, then opens the window and hands
//! control to the shell. A quit signal closes the window in order and
//! then ends the process by that signal (ADR-T48-1).

use draw::shell::app;
use draw::shell::diag::{self, Level};
use macroquad::miniquad::signal;

fn main() {
    draw::shell::crash::install_hook();
    diag::init_from_env();
    if !signal::install_fatal(app::on_fatal_signal) {
        diag::log(Level::Warn, "signal: fatal-signal handler not installed");
    }
    if let Err(problem) = draw::shell::preflight::run() {
        eprintln!("draw: {problem}");
        diag::log(Level::Error, &format!("preflight: {problem}"));
        diag::finish("preflight failed");
        std::process::exit(1);
    }
    if !signal::install_quit() {
        diag::log(Level::Warn, "signal: SIGINT/SIGTERM handler not installed");
    }
    macroquad::Window::from_config(app::window_conf(), app::run());
    let received = signal::received();
    if let Some(quit) = received {
        diag::log(Level::Info, &app::signal_line(quit));
    }
    diag::finish(app::exit_reason(received));
    if let Some(quit) = received {
        signal::reraise(quit);
    }
}
