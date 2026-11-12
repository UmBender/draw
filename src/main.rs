//! Binary entry point: starts the crash hook, the diagnostics log and the
//! signal handlers, checks the system, then opens the window and hands
//! control to the shell. A quit signal closes the window in order and
//! then ends the process by that signal (ADR-T48-1).

#![forbid(unsafe_code)]

use draw::shell::app;
use draw::shell::diag::{self, Level};
use draw::shell::options::{self, Parsed};
use draw::shell::{backend, tablet};
use macroquad::miniquad::signal;

/// The text of `--version`.
fn version_text() -> String {
    format!("draw {}", env!("CARGO_PKG_VERSION"))
}

/// The text of `--help`.
fn help_text() -> String {
    format!(
        "{}\n\n\
Usage: draw [FILE.draw] [OPTIONS]\n\
       draw --help | --version\n\n\
Options:\n  \
  --recover [prev]  Open the last autosave (or the one before it)\n  \
  --help            Show this help\n  \
  --version         Show the version\n\n\
Debugging (for troubleshooting only):\n  \
  --log <level>        Log level: off, error, warn, info (default), debug, trace\n  \
  --frame-times        Log how long each re-render takes\n  \
  --skip-preflight     Skip the check for a display and libraries\n  \
  --backend <name>     Window backend on Linux: wayland or x11\n  \
  --msaa <n>           Anti-aliasing samples: off, 2, 4 (default), 8\n  \
  --tablet <mode>      Pen mode: auto (default), on, off",
        version_text()
    )
}

fn main() {
    let options = match options::parse(std::env::args_os().skip(1)) {
        Parsed::Run(options) => options,
        Parsed::Help => {
            println!("{}", help_text());
            return;
        }
        Parsed::Version => {
            println!("{}", version_text());
            return;
        }
        Parsed::Refused(problem) => {
            eprintln!("draw: {problem}\nTry `draw --help`.");
            std::process::exit(2);
        }
    };
    let backend = backend::choose(
        options.backend,
        std::env::var_os(backend::WAYLAND_DISPLAY_ENV).as_deref(),
    );
    let tablet = tablet::detect(options.tablet);
    draw::shell::crash::install_hook();
    diag::init(options.log, backend, tablet.pen);
    if !signal::install_fatal(app::on_fatal_signal) {
        diag::log(Level::Warn, "signal: fatal-signal handler not installed");
    }
    if let Err(problem) = draw::shell::preflight::run(options.skip_preflight, backend) {
        eprintln!("draw: {problem}");
        diag::log(Level::Error, &format!("preflight: {problem}"));
        diag::finish("preflight failed");
        std::process::exit(1);
    }
    if !signal::install_quit() {
        diag::log(Level::Warn, "signal: SIGINT/SIGTERM handler not installed");
    }
    macroquad::Window::from_config(
        app::window_conf(backend),
        app::run(options, backend, tablet),
    );
    let received = signal::received();
    if let Some(quit) = received {
        diag::log(Level::Info, &app::signal_line(quit));
    }
    diag::finish(app::exit_reason(received));
    if let Some(quit) = received {
        signal::reraise(quit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The prefix of the variables draw once read, split so the source guard
    /// `tests/no_draw_env.rs` does not match it.
    const OLD_PREFIX: &str = concat!("DRAW", "_");

    #[test]
    fn help_lists_the_flags() {
        let text = help_text();

        assert!(text.contains("Debugging"), "{text}");
        for flag in [
            "--log",
            "--frame-times",
            "--skip-preflight",
            "--backend",
            "--msaa",
            "--tablet",
            "--recover",
        ] {
            assert!(text.contains(flag), "{flag} missing: {text}");
        }
        assert!(!text.contains(OLD_PREFIX), "{text}");
    }

    #[test]
    fn help_hides_the_test_flags() {
        let text = help_text();

        for hidden in ["--snapshot", "--e2e-dump", "--e2e-idle", "--e2e-clipboard"] {
            assert!(!text.contains(hidden), "{hidden} shown: {text}");
        }
    }

    #[test]
    fn version_prints_package_version() {
        assert!(version_text().contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn readme_names_no_draw_variable() {
        let readme = include_str!("../README.md");
        for line in readme.lines() {
            assert!(
                !line.contains(OLD_PREFIX),
                "an old variable name is left in the README: {line}"
            );
        }
        assert!(readme.contains("\n### Debugging\n"));
    }
}
