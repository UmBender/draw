//! Binary entry point: starts the crash hook, the diagnostics log and the
//! signal handlers, checks the system, then opens the window and hands
//! control to the shell. A quit signal closes the window in order and
//! then ends the process by that signal (ADR-T48-1).

#![forbid(unsafe_code)]

use draw::shell::app;
use draw::shell::autosave::Launch;
use draw::shell::diag::{self, Level};
use macroquad::miniquad::signal;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

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
            assert!(matches!(parsed, Cli::Refused(ref m) if m.contains(flag)), "{parsed:?}");
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
