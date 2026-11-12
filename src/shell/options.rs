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
    let args: Vec<OsString> = args.into_iter().collect();
    if let Some(early) = args.iter().find_map(|arg| match arg.to_str() {
        Some("--help") => Some(Parsed::Help),
        Some("--version") => Some(Parsed::Version),
        _ => None,
    }) {
        return early;
    }
    match read(&args) {
        Ok(options) => Parsed::Run(options),
        Err(message) => Parsed::Refused(message),
    }
}

/// The flags in `args`, or why one is refused.
fn read(args: &[OsString]) -> Result<Options, String> {
    let mut options = Options::default();
    // What `--recover` and the file argument read: the words that are
    // neither a flag nor a flag's value.
    let mut rest: Vec<OsString> = Vec::new();
    let mut words = args.iter().peekable();
    while let Some(arg) = words.next() {
        let text = arg.to_string_lossy();
        let Some(flag) = text.strip_prefix("--").filter(|name| !name.is_empty()) else {
            if text.starts_with('-') && text.len() > 1 {
                return Err(format!("unknown option `{text}`"));
            }
            rest.push(arg.clone());
            continue;
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(OsString::from(value))),
            None => (flag, None),
        };
        match name {
            "frame-times" | "skip-preflight" => {
                if inline.is_some() {
                    return Err(format!("`--{name}` takes no value"));
                }
                if name == "frame-times" {
                    options.frame_times = true;
                } else {
                    options.skip_preflight = true;
                }
            }
            "recover" => {
                rest.push(OsString::from("--recover"));
                let value = inline.or_else(|| {
                    words
                        .next_if(|next| !next.to_string_lossy().starts_with('-'))
                        .cloned()
                });
                rest.extend(value);
            }
            "log" | "msaa" | "backend" | "tablet" | "snapshot" | "e2e-dump" | "e2e-idle"
            | "e2e-clipboard" => {
                let value = inline
                    .or_else(|| words.next().cloned())
                    .ok_or_else(|| format!("`--{name}` needs a value"))?;
                apply(&mut options, name, &value)?;
            }
            _ => return Err(format!("unknown option `--{flag}`")),
        }
    }
    options.launch = Launch::from_args(rest.iter().cloned());
    options.file = crate::shell::file::path_from_args(rest);
    Ok(options)
}

/// Sets the option `name` from `value`, or says why the value is not valid.
fn apply(options: &mut Options, name: &str, value: &OsString) -> Result<(), String> {
    let text = value.to_string_lossy();
    let bad = |expected: &str| format!("`--{name}`: unknown value `{text}` (expected {expected})");
    match name {
        "log" => {
            options.log = Level::from_name(&text)
                .ok_or_else(|| bad("off, error, warn, info, debug or trace"))?;
        }
        "msaa" => {
            options.msaa_samples = match text.trim().to_ascii_lowercase().as_str() {
                "off" | "1" => 1,
                "2" => 2,
                "4" => 4,
                "8" => 8,
                _ => return Err(bad("off, 2, 4 or 8")),
            };
        }
        "backend" => {
            options.backend = Some(Backend::from_name(&text).ok_or_else(|| bad("wayland or x11"))?);
        }
        "tablet" => {
            options.tablet = match text.trim().to_ascii_lowercase().as_str() {
                "auto" => TabletMode::Auto,
                "on" => TabletMode::On,
                "off" => TabletMode::Off,
                _ => return Err(bad("auto, on or off")),
            };
        }
        "snapshot" => options.snapshot = Some(PathBuf::from(value)),
        "e2e-dump" => options.e2e.dump = Some(PathBuf::from(value)),
        "e2e-idle" => options.e2e.idle = Some(PathBuf::from(value)),
        _ => options.e2e.clipboard = Some(PathBuf::from(value)),
    }
    Ok(())
}
