//! The command line as one typed value (T154, ADR-T154-1).
//!
//! [`parse`] turns the arguments into [`Options`] once, in `main`, before
//! anything else starts; nothing is read from the environment under a
//! `DRAW_` name. A bad value and an unknown flag are refused, never
//! guessed.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::shell::app::DEFAULT_MSAA_SAMPLES;
use crate::shell::autosave::Launch;
use crate::shell::backend::Backend;
use crate::shell::diag::Level;

/// Whether pen mode is forced (`--tablet`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabletMode {
    /// Detect a pen tablet from the input devices (the default).
    #[default]
    Auto,
    /// Pen mode on, whatever the devices say.
    On,
    /// Pen mode off, whatever the devices say.
    Off,
}

/// The files the end-to-end harness gives the app (`--e2e-*`, hidden from
/// `--help`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct E2eFiles {
    /// Receives the editor snapshot after each sync chord.
    pub dump: Option<PathBuf>,
    /// Receives the sync count once the frame is on screen.
    pub idle: Option<PathBuf>,
    /// Holds the text that goes on the clipboard.
    pub clipboard: Option<PathBuf>,
}

/// What the command line asks for, with a value for every setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// What the session log says (`--log`).
    pub log: Level,
    /// MSAA samples of the cached frame (`--msaa`).
    pub msaa_samples: i32,
    /// Whether every re-render logs its time (`--frame-times`).
    pub frame_times: bool,
    /// The window backend, or the session's default when `None`
    /// (`--backend`).
    pub backend: Option<Backend>,
    /// Whether to skip the start-up check (`--skip-preflight`).
    pub skip_preflight: bool,
    /// Pen mode (`--tablet`).
    pub tablet: TabletMode,
    /// A snapshot file to start from (`--snapshot`, hidden).
    pub snapshot: Option<PathBuf>,
    /// The end-to-end harness files (hidden).
    pub e2e: E2eFiles,
    /// Whether to start from an autosave (`--recover [prev]`).
    pub launch: Launch,
    /// The file to open.
    pub file: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            log: Level::Info,
            msaa_samples: DEFAULT_MSAA_SAMPLES,
            frame_times: false,
            backend: None,
            skip_preflight: false,
            tablet: TabletMode::Auto,
            snapshot: None,
            e2e: E2eFiles::default(),
            launch: Launch::Normal,
            file: None,
        }
    }
}

/// What [`parse`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// Start the app with these options.
    Run(Options),
    /// Print the help text and exit.
    Help,
    /// Print the version and exit.
    Version,
    /// A flag or value that is not valid; the message says which.
    Refused(String),
}

/// Reads the arguments (without the program name). `--help` and
/// `--version` win over everything else, even a bad value.
#[must_use]
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Parsed {
    let _ = args;
    todo!("T154: parse the flags")
}
