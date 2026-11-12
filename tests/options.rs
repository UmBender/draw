//! The command line as `Options` (T154, ADR-T154-1): every flag, its value
//! check and the refusals.

use std::ffi::OsString;
use std::path::PathBuf;

use draw::shell::autosave::{Launch, Which};
use draw::shell::backend::Backend;
use draw::shell::diag::Level;
use draw::shell::options::{Options, Parsed, TabletMode, parse};

fn args(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

fn run(list: &[&str]) -> Options {
    match parse(args(list)) {
        Parsed::Run(options) => options,
        other => panic!("{list:?} gave {other:?}"),
    }
}

fn refused(list: &[&str]) -> String {
    match parse(args(list)) {
        Parsed::Refused(message) => message,
        other => panic!("{list:?} gave {other:?}"),
    }
}

#[test]
fn defaults_without_arguments() {
    // Arrange, Act
    let options = run(&[]);

    // Assert
    assert_eq!(options, Options::default());
    assert_eq!(options.log, Level::Info);
    assert_eq!(options.msaa_samples, 4);
    assert!(!options.frame_times);
    assert_eq!(options.backend, None);
    assert!(!options.skip_preflight);
    assert_eq!(options.tablet, TabletMode::Auto);
    assert_eq!(options.snapshot, None);
    assert_eq!(options.launch, Launch::Normal);
    assert_eq!(options.file, None);
}

#[test]
fn log_takes_every_level() {
    for (text, level) in [
        ("off", Level::Off),
        ("error", Level::Error),
        ("warn", Level::Warn),
        ("info", Level::Info),
        ("debug", Level::Debug),
        ("TRACE", Level::Trace),
    ] {
        assert_eq!(run(&["--log", text]).log, level, "{text}");
    }
}

#[test]
fn log_refuses_an_unknown_level() {
    let message = refused(&["--log", "loud"]);

    assert!(message.contains("--log"), "{message}");
    assert!(message.contains("loud"), "{message}");
}

#[test]
fn msaa_takes_off_and_the_counts() {
    for (text, samples) in [("off", 1), ("1", 1), ("2", 2), ("4", 4), ("8", 8)] {
        assert_eq!(run(&["--msaa", text]).msaa_samples, samples, "{text}");
    }
}

#[test]
fn msaa_refuses_other_counts() {
    for text in ["3", "16", "-1", "x", ""] {
        let message = refused(&["--msaa", text]);
        assert!(message.contains("--msaa"), "{text}: {message}");
    }
}

#[test]
fn backend_takes_x11_and_wayland() {
    assert_eq!(run(&["--backend", "x11"]).backend, Some(Backend::X11));
    assert_eq!(
        run(&["--backend", "Wayland"]).backend,
        Some(Backend::Wayland)
    );
}

#[test]
fn backend_refuses_an_unknown_name() {
    let message = refused(&["--backend", "mir"]);

    assert!(message.contains("--backend"), "{message}");
    assert!(message.contains("mir"), "{message}");
}

#[test]
fn tablet_takes_auto_on_off() {
    assert_eq!(run(&["--tablet", "auto"]).tablet, TabletMode::Auto);
    assert_eq!(run(&["--tablet", "on"]).tablet, TabletMode::On);
    assert_eq!(run(&["--tablet", "OFF"]).tablet, TabletMode::Off);
}

#[test]
fn tablet_refuses_an_unknown_mode() {
    let message = refused(&["--tablet", "maybe"]);

    assert!(message.contains("--tablet"), "{message}");
}

#[test]
fn frame_times_and_skip_preflight_take_no_value() {
    // Act
    let options = run(&["--frame-times", "--skip-preflight", "a.draw"]);

    // Assert: the word after them is the file, not a value
    assert!(options.frame_times);
    assert!(options.skip_preflight);
    assert_eq!(options.file, Some(PathBuf::from("a.draw")));
}

#[test]
fn a_flag_without_its_value_is_refused() {
    for flag in ["--log", "--msaa", "--backend", "--tablet", "--snapshot"] {
        let message = refused(&[flag]);
        assert!(message.contains(flag), "{message}");
        assert!(message.contains("value"), "{message}");
    }
}

#[test]
fn value_with_equals_sign_is_accepted() {
    // Act
    let options = run(&["--log=debug", "--msaa=2", "--backend=x11", "--tablet=on"]);

    // Assert
    assert_eq!(options.log, Level::Debug);
    assert_eq!(options.msaa_samples, 2);
    assert_eq!(options.backend, Some(Backend::X11));
    assert_eq!(options.tablet, TabletMode::On);
}

#[test]
fn flags_in_any_order_and_with_a_file_and_recover() {
    // Act
    let first = run(&["a.draw", "--log", "warn", "--recover", "prev"]);
    let second = run(&["--recover", "prev", "--log", "warn", "a.draw"]);

    // Assert
    assert_eq!(first.log, Level::Warn);
    assert_eq!(first.launch, Launch::Recover(Which::Previous));
    assert_eq!(first.file, Some(PathBuf::from("a.draw")));
    assert_eq!(first, second);
    assert_eq!(
        run(&["--recover"]).launch,
        Launch::Recover(Which::Latest),
        "a bare --recover"
    );
}

#[test]
fn hidden_flags_take_paths() {
    // Act
    let options = run(&[
        "--snapshot",
        "s.snap",
        "--e2e-dump",
        "d",
        "--e2e-idle=i",
        "--e2e-clipboard",
        "c",
    ]);

    // Assert
    assert_eq!(options.snapshot, Some(PathBuf::from("s.snap")));
    assert_eq!(options.e2e.dump, Some(PathBuf::from("d")));
    assert_eq!(options.e2e.idle, Some(PathBuf::from("i")));
    assert_eq!(options.e2e.clipboard, Some(PathBuf::from("c")));
}

#[test]
fn unknown_flag_is_refused() {
    for flag in ["--other", "--experimental", "-x", "--DRAW_LOG"] {
        let message = refused(&["a.draw", flag]);
        assert!(message.contains(flag), "{message}");
    }
}

#[test]
fn help_and_version_win() {
    assert!(matches!(parse(args(&["--help"])), Parsed::Help));
    assert!(matches!(parse(args(&["--version"])), Parsed::Version));
    assert!(matches!(
        parse(args(&["--log", "loud", "--help"])),
        Parsed::Help
    ));
    assert!(matches!(
        parse(args(&["--msaa", "3", "--version"])),
        Parsed::Version
    ));
}
