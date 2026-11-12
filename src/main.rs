//! Binary entry point: starts the crash hook, the diagnostics log and the
//! signal handlers, checks the system, then opens the window and hands
//! control to the shell. A quit signal closes the window in order and
//! then ends the process by that signal (ADR-T48-1).

#![forbid(unsafe_code)]

use draw::shell::app;
use draw::shell::autosave::Launch;
use draw::shell::diag::{self, Level};
use macroquad::miniquad::signal;

use std::ffi::OsString;

/// What the command line asks for.
#[derive(Debug, PartialEq, Eq)]
enum Cli {
    /// Start the app (arguments are a file and `--recover`, read later).
    Run,
    /// Print the help text and exit.
    Help,
    /// Print the version and exit.
    Version,
    /// An option that does not exist; the message says which.
    Refused(String),
}

/// Reads the options: `--help`, `--version` and `--recover` (with its
/// optional `prev`) are the only ones; anything else starting with `-` is
/// refused. Other words are the file to open or a `--recover` value.
fn parse_cli(args: impl IntoIterator<Item = OsString>) -> Cli {
    let mut run = Cli::Run;
    for arg in args {
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "--help" => return Cli::Help,
            "--version" => return Cli::Version,
            "--recover" => {}
            flag if flag.starts_with('-') && flag.len() > 1 => {
                run = Cli::Refused(format!("unknown option `{flag}`"));
                break;
            }
            _ => {}
        }
    }
    run
}

/// The text of `--version`.
fn version_text() -> String {
    format!("draw {}", env!("CARGO_PKG_VERSION"))
}

/// The text of `--help`.
fn help_text() -> String {
    format!(
        "{}\n\n\
Usage: draw [FILE.draw] [--recover [prev]]\n\
       draw --help | --version\n\n\
Options:\n  \
  --recover [prev]  Open the last autosave (or the one before it)\n  \
  --help            Show this help\n  \
  --version         Show the version\n\n\
Debugging (environment variables, for troubleshooting only):\n  \
  DRAW_LOG=<level>        Log level: off, error, warn, info (default), debug, trace\n  \
  DRAW_FRAME_TIMES=1      Log how long each re-render takes\n  \
  DRAW_SKIP_PREFLIGHT=1   Skip the check for a display and libraries\n  \
  DRAW_BACKEND=<name>     Window backend on Linux: wayland or x11\n  \
  DRAW_MSAA=<n>           Anti-aliasing samples: off, 2, 4 (default), 8\n  \
  DRAW_TABLET=<mode>      Pen mode: auto (default), on, off",
        version_text()
    )
}

fn main() {
    match parse_cli(std::env::args_os().skip(1)) {
        Cli::Run => {}
        Cli::Help => {
            println!("{}", help_text());
            return;
        }
        Cli::Version => {
            println!("{}", version_text());
            return;
        }
        Cli::Refused(problem) => {
            eprintln!("draw: {problem}\nTry `draw --help`.");
            std::process::exit(2);
        }
    }
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
    let launch = Launch::from_args(std::env::args_os().skip(1));
    let file = draw::shell::file::path_from_args(std::env::args_os().skip(1));
    macroquad::Window::from_config(app::window_conf(), app::run(launch, file));
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

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn help_lists_debug_variables() {
        let text = help_text();

        assert!(text.contains("Debugging"), "{text}");
        for name in [
            "DRAW_LOG",
            "DRAW_FRAME_TIMES",
            "DRAW_SKIP_PREFLIGHT",
            "DRAW_BACKEND",
            "DRAW_MSAA",
            "DRAW_TABLET",
        ] {
            assert!(text.contains(name), "{name} missing: {text}");
        }
        assert!(!text.contains("DRAW_EXPERIMENTAL"));
        assert!(text.contains("--recover"));
    }

    #[test]
    fn version_prints_package_version() {
        assert_eq!(parse_cli(args(&["--version"])), Cli::Version);
        assert!(version_text().contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn help_flag_asks_for_help() {
        assert_eq!(parse_cli(args(&["--help"])), Cli::Help);
    }

    #[test]
    fn unknown_flag_is_refused() {
        for flag in ["--other", "--experimental", "-x"] {
            let parsed = parse_cli(args(&["a.draw", flag]));
            assert!(
                matches!(parsed, Cli::Refused(ref m) if m.contains(flag)),
                "{parsed:?}"
            );
        }
    }

    #[test]
    fn known_flags_and_files_are_accepted() {
        assert_eq!(parse_cli(args(&[])), Cli::Run);
        assert_eq!(parse_cli(args(&["a.draw"])), Cli::Run);
        assert_eq!(parse_cli(args(&["--recover"])), Cli::Run);
        assert_eq!(parse_cli(args(&["--recover", "prev", "a.draw"])), Cli::Run);
    }

    #[test]
    fn readme_names_variables_only_in_debugging_section() {
        let readme = include_str!("../README.md");
        let mut in_debugging = false;
        for line in readme.lines() {
            if line.starts_with('#') && !line.starts_with("# ") {
                in_debugging = line.trim_start_matches('#').trim() == "Debugging";
            }
            if line.starts_with("```") || line.starts_with('#') {
                continue;
            }
            assert!(
                in_debugging || !line.contains("DRAW_"),
                "DRAW_ variable outside the Debugging section: {line}"
            );
        }
        assert!(readme.contains("\n### Debugging\n"));
    }
}
