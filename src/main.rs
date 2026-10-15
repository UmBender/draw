//! Binary entry point: checks the system, then opens the window and hands
//! control to the shell.

fn main() {
    draw::shell::crash::install_hook();
    if let Err(problem) = draw::shell::preflight::run() {
        eprintln!("draw: {problem}");
        std::process::exit(1);
    }
    macroquad::Window::from_config(draw::shell::app::window_conf(), draw::shell::app::run());
}
