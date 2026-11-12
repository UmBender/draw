//! End-to-end suite (T75, ADR-T75-1): the real `draw` binary in a private
//! headless sway session, driven by real compositor input.
//!
//! `e2e_every_case` is `#[ignore]`d and refuses to start outside
//! `scripts/e2e.sh`. Everything else here tests the harness itself and needs
//! no display. A case is a directory in `tests/e2e/`:
//!
//! - `input`: one directive per line (see [`parse_input`]);
//! - `start.snap` (optional): the state to start from;
//! - `expected.snap`, and `expected.ppm` with `screenshot`;
//! - `<name>.snap` for each `checkpoint <name>`.
//!
//! `DRAW_UPDATE_SNAPSHOTS=1` rewrites the goldens. A failed case keeps its
//! snapshot, screenshot, diff image and `draw.log` in `target/e2e/<case>/`.

mod common;

use std::collections::VecDeque;
use std::ffi::OsString;
use std::io::{Read as _, Write as _};
use std::os::unix::fs::FileTypeExt as _;
use std::os::unix::net::UnixStream;
use std::os::unix::process::ExitStatusExt as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use common::scenes;
use draw::core::editor::Editor;
use draw::core::snapshot::golden::{self, Golden};

/// How long the app may take to answer one sync. Generous: the suite runs
/// at the lowest priority on a machine that may be busy.
const SYNC_TIMEOUT: Duration = Duration::from_secs(120);

/// How often a signal file or the compositor is polled while waiting.
const POLL: Duration = Duration::from_millis(5);

/// A `wtype` stays alive this long after typing (milliseconds), so it never
/// goes away while it is the seat's active keyboard (ADR-T75-1).
const KEEP_ALIVE_MS: &str = "3000000";

/// The sync chord, as `wtype` arguments: `Ctrl+Shift+9` (ADR-T75-1).
const SYNC_ARGS: [&str; 10] = [
    "-M", "ctrl", "-M", "shift", "-k", "9", "-m", "shift", "-m", "ctrl",
];

/// The output every case starts at.
const OUTPUT: (u32, u32) = (800, 600);

/// The name sway gives the headless output.
const OUTPUT_NAME: &str = "HEADLESS-1";

/// Segments of a `drag` that does not name its own.
const DRAG_SEGMENTS: u32 = 4;

/// Where the pointer waits on the canvas when a case starts.
const PARK: (i32, i32) = (400, 300);

/// Strokes of the `strokes-5000` scene.
const SCENE_STROKES: usize = 5000;

// ---- Case files --------------------------------------------------------

/// One input step of a case; each ends in a sync.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    /// `wtype` arguments.
    Type(Vec<String>),
    /// `swaymsg seat - cursor` arguments.
    Pointer(Vec<String>),
    /// One wheel notch: negative is up (ADR-T75-2).
    Wheel(i32),
    /// Output size to set, waited for in the snapshot.
    Output(u32, u32),
    /// Compare the state with `<name>.snap` here.
    Checkpoint(String),
    /// End the seat's active keyboard, then type through a new one (T79).
    EndKeyboard,
    /// Put this text on the system clipboard (ADR-T83-1).
    Clipboard(String),
}

/// How much two screenshots may differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tolerance {
    /// Pixels allowed to differ.
    pixels: usize,
    /// Largest difference, per channel, that does not count.
    channel: u8,
}

impl Default for Tolerance {
    fn default() -> Self {
        Self {
            pixels: 64,
            channel: 8,
        }
    }
}

/// How a case ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Ending {
    /// The compositor closes the window.
    #[default]
    Close,
    /// `SIGINT`.
    Sigint,
}

/// A parsed `input` file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Case {
    /// A generated start state, instead of `start.snap`.
    scene: Option<String>,
    /// Start from the state as an autosave, through `draw --recover`.
    recover: bool,
    /// Snapshot sections to compare; empty means the whole snapshot.
    sections: Vec<String>,
    /// Whether to compare a screenshot, and how strictly.
    screenshot: Option<Tolerance>,
    /// Whether the case must leave one exported PNG, showing the window.
    export: bool,
    /// The input.
    steps: Vec<Step>,
    /// How the case ends.
    ending: Ending,
}

fn number<T: std::str::FromStr>(word: &str) -> Result<T, String> {
    word.parse()
        .map_err(|_| format!("`{word}` is not a number"))
}

fn button(word: Option<&str>) -> Result<&'static str, String> {
    match word {
        None | Some("left") => Ok("button1"),
        Some("middle") => Ok("button2"),
        Some("right") => Ok("button3"),
        Some(other) => Err(format!("unknown button `{other}`")),
    }
}

fn pointer(args: &[&str]) -> Step {
    Step::Pointer(args.iter().map(|&a| a.to_owned()).collect())
}

fn move_to(x: i32, y: i32) -> Step {
    pointer(&["set", &x.to_string(), &y.to_string()])
}

/// The steps of a left drag through `segments` straight pieces.
fn drag(from: (i32, i32), to: (i32, i32), segments: u32) -> Vec<Step> {
    let mut steps = vec![move_to(from.0, from.1), pointer(&["press", "button1"])];
    let n = i32::try_from(segments.max(1)).unwrap_or(1);
    for i in 1..=n {
        steps.push(move_to(
            from.0 + (to.0 - from.0) * i / n,
            from.1 + (to.1 - from.1) * i / n,
        ));
    }
    steps.push(pointer(&["release", "button1"]));
    steps
}

/// `wtype` arguments of a chord: modifiers pressed in order and released in
/// reverse, around the last word as the key.
fn chord(words: &[&str]) -> Result<Vec<String>, String> {
    let (key, mods) = words.split_last().ok_or("chord needs a key")?;
    let mut args = Vec::with_capacity(2 * mods.len() + 2);
    for m in mods {
        args.extend(["-M".to_owned(), (*m).to_owned()]);
    }
    args.extend(["-k".to_owned(), (*key).to_owned()]);
    for m in mods.iter().rev() {
        args.extend(["-m".to_owned(), (*m).to_owned()]);
    }
    Ok(args)
}

/// Parses one directive into `case`.
fn parse_line(line: &str, case: &mut Case) -> Result<(), String> {
    let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
    let args: Vec<&str> = rest.split_whitespace().collect();
    match (word, args.as_slice()) {
        ("scene", [name]) => case.scene = Some((*name).to_owned()),
        ("recover", []) => case.recover = true,
        ("sections", names) if !names.is_empty() => {
            case.sections = names.iter().map(|&n| n.to_owned()).collect();
        }
        ("screenshot", []) => case.screenshot = Some(Tolerance::default()),
        ("screenshot", [pixels, channel]) => {
            case.screenshot = Some(Tolerance {
                pixels: number(pixels)?,
                channel: number(channel)?,
            });
        }
        ("export", []) => case.export = true,
        // wtype would read a leading dash as an option.
        ("text", _) if !rest.is_empty() && !rest.starts_with('-') => {
            case.steps.push(Step::Type(vec![rest.to_owned()]));
        }
        // wl-copy would read a leading dash as an option.
        ("clipboard", _) if !rest.is_empty() && !rest.starts_with('-') => {
            case.steps.push(Step::Clipboard(rest.to_owned()));
        }
        ("key", [key]) => {
            case.steps
                .push(Step::Type(vec!["-k".to_owned(), (*key).to_owned()]));
        }
        ("chord", words) if !words.is_empty() => case.steps.push(Step::Type(chord(words)?)),
        ("hold", [key, ms]) => {
            let ms: u32 = number(ms)?;
            case.steps.push(Step::Type(vec![
                "-P".to_owned(),
                (*key).to_owned(),
                "-s".to_owned(),
                ms.to_string(),
                "-p".to_owned(),
                (*key).to_owned(),
            ]));
        }
        ("move", [x, y]) => case.steps.push(move_to(number(x)?, number(y)?)),
        ("press", [] | [_]) => {
            case.steps
                .push(pointer(&["press", button(args.first().copied())?]));
        }
        ("release", [] | [_]) => {
            case.steps
                .push(pointer(&["release", button(args.first().copied())?]));
        }
        ("click", [x, y]) => {
            case.steps.extend([
                move_to(number(x)?, number(y)?),
                pointer(&["press", "button1"]),
                pointer(&["release", "button1"]),
            ]);
        }
        ("drag", [x1, y1, x2, y2] | [x1, y1, x2, y2, _]) => {
            let segments = args.get(4).map_or(Ok(DRAG_SEGMENTS), |s| number(s))?;
            case.steps.extend(drag(
                (number(x1)?, number(y1)?),
                (number(x2)?, number(y2)?),
                segments,
            ));
        }
        ("wheel", ["up"]) => case.steps.push(Step::Wheel(-1)),
        ("wheel", ["down"]) => case.steps.push(Step::Wheel(1)),
        ("resize", [w, h]) => case.steps.push(Step::Output(number(w)?, number(h)?)),
        ("checkpoint", [name]) => case.steps.push(Step::Checkpoint((*name).to_owned())),
        ("end-keyboard", []) => case.steps.push(Step::EndKeyboard),
        ("close", []) => case.ending = Ending::Close,
        ("sigint", []) => case.ending = Ending::Sigint,
        _ => return Err(format!("unknown directive: {line}")),
    }
    Ok(())
}

/// Parses an `input` file. Blank lines and lines starting with `#` are
/// skipped; a bad line is reported with its number.
///
/// | Directive | Meaning |
/// |-----------|---------|
/// | `scene <name>` | generated start state |
/// | `recover` | start from the state as an autosave, with `--recover` |
/// | `sections <name>...` | compare only these snapshot sections |
/// | `screenshot [pixels channel]` | also compare `expected.ppm` |
/// | `export` | `$HOME` must hold one PNG that shows the final window |
/// | `text <text>` | type literally, through `wtype` |
/// | `clipboard <text>` | put text on the system clipboard |
/// | `key <keysym>` | one key |
/// | `chord <mod>... <keysym>` | key with modifiers |
/// | `hold <keysym> <ms>` | hold a key down |
/// | `move x y`, `press [button]`, `release [button]` | pointer |
/// | `click x y`, `drag x1 y1 x2 y2 [segments]` | left button |
/// | `wheel up/down` | one wheel notch |
/// | `resize w h` | resize the output, wait for the window |
/// | `checkpoint <name>` | compare with `<name>.snap` |
/// | `end-keyboard` | end the active keyboard (may send `no_keymap`) |
/// | `close`, `sigint` | how the case ends (default `close`) |
fn parse_input(text: &str) -> Result<Case, String> {
    let mut case = Case::default();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim_end();
        if line.trim_start().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        parse_line(line.trim_start(), &mut case).map_err(|err| format!("line {}: {err}", n + 1))?;
    }
    Ok(case)
}

/// The named `[section]`s of a snapshot, header kept. No names keeps all.
fn select_sections(snapshot: &str, sections: &[String]) -> String {
    if sections.is_empty() {
        return snapshot.to_owned();
    }
    let mut out = String::new();
    let mut keep = true;
    for line in snapshot.lines() {
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            keep = sections.iter().any(|s| s == name);
        }
        if keep {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

// ---- Comparisons -------------------------------------------------------

/// Set to `1` to rewrite the goldens.
fn update_requested() -> bool {
    common::snapshots::update_requested()
}

/// Compares `actual` with the golden at `path`, or rewrites it when
/// `update`. The error is a printable line diff.
fn compare_snapshot(actual: &str, path: &Path, update: bool) -> Result<(), String> {
    let expected = std::fs::read_to_string(path).ok();
    match golden::check(actual, expected.as_deref(), update) {
        Golden::Match => Ok(()),
        Golden::Write => std::fs::write(path, actual)
            .map_err(|err| format!("cannot write {}: {err}", path.display())),
        Golden::Missing => Err(format!(
            "no golden at {}; run with DRAW_UPDATE_SNAPSHOTS=1",
            path.display()
        )),
        Golden::Mismatch(diff) => Err(format!("state differs from {}\n{diff}", path.display())),
    }
}

/// An RGB image, as `grim -t ppm` writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Image {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

impl Image {
    /// Reads a binary PPM (`P6`) with a maxval of 255.
    fn parse_ppm(bytes: &[u8]) -> Result<Self, String> {
        let mut at = 0;
        let mut token = || -> Result<String, String> {
            while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
                at += 1;
            }
            let start = at;
            while bytes.get(at).is_some_and(|b| !b.is_ascii_whitespace()) {
                at += 1;
            }
            let word = bytes.get(start..at).ok_or("ppm: truncated header")?;
            String::from_utf8(word.to_vec()).map_err(|_| "ppm: bad header".to_owned())
        };
        if token()? != "P6" {
            return Err("ppm: not a P6 file".to_owned());
        }
        let width: usize = number(&token()?)?;
        let height: usize = number(&token()?)?;
        if token()? != "255" {
            return Err("ppm: maxval is not 255".to_owned());
        }
        // Exactly one whitespace byte separates the header from the data.
        let data = bytes.get(at + 1..).ok_or("ppm: no pixel data")?;
        if data.len() != width * height * 3 {
            return Err(format!(
                "ppm: {} bytes of data for {width}x{height}",
                data.len()
            ));
        }
        Ok(Self {
            width,
            height,
            rgb: data.to_vec(),
        })
    }

    fn to_ppm(&self) -> Vec<u8> {
        let mut out = format!("P6\n{} {}\n255\n", self.width, self.height).into_bytes();
        out.extend_from_slice(&self.rgb);
        out
    }
}

/// Why two screenshots do not match, with an image showing where.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ImageDiff {
    message: String,
    image: Image,
}

/// Compares two screenshots within `tolerance`. In the diff image the
/// differing pixels are red and the rest a dimmed copy of `actual`.
fn compare_images(actual: &Image, expected: &Image, tolerance: Tolerance) -> Result<(), ImageDiff> {
    if (actual.width, actual.height) != (expected.width, expected.height) {
        return Err(ImageDiff {
            message: format!(
                "screenshot is {}x{}, golden is {}x{}",
                actual.width, actual.height, expected.width, expected.height
            ),
            image: actual.clone(),
        });
    }
    let mut rgb = Vec::with_capacity(actual.rgb.len());
    let mut differing = 0usize;
    let (actual_pixels, _) = actual.rgb.as_chunks::<3>();
    let (expected_pixels, _) = expected.rgb.as_chunks::<3>();
    for (a, e) in actual_pixels.iter().zip(expected_pixels) {
        if a.iter()
            .zip(e)
            .any(|(x, y)| x.abs_diff(*y) > tolerance.channel)
        {
            differing += 1;
            rgb.extend([255, 0, 0]);
        } else {
            rgb.extend(a.iter().map(|c| c / 3));
        }
    }
    if differing > tolerance.pixels {
        return Err(ImageDiff {
            message: format!(
                "{differing} pixels differ by more than {} (allowed: {})",
                tolerance.channel, tolerance.pixels
            ),
            image: Image {
                width: actual.width,
                height: actual.height,
                rgb,
            },
        });
    }
    Ok(())
}

/// Side of the box a screenshot comparison ignores under the pointer.
const CURSOR_BOX_SIZE: i32 = 26;

/// How far the ignored box starts above and left of the pointer.
const CURSOR_BOX_BEFORE: i32 = 2;

/// Copies `expected`'s pixels over the box at `pointer`, because whether a
/// capture shows the cursor depends on the compositor (ADR-T80-1).
fn hide_cursor_box(actual: &mut Image, expected: &Image, pointer: (i32, i32)) {
    if (actual.width, actual.height) != (expected.width, expected.height) {
        return;
    }
    let span = |at: i32, limit: usize| {
        let start = at.saturating_sub(CURSOR_BOX_BEFORE);
        let clip = |edge: i32| usize::try_from(edge).unwrap_or(0).min(limit);
        clip(start)..clip(start.saturating_add(CURSOR_BOX_SIZE))
    };
    let width = actual.width;
    let columns = span(pointer.0, width);
    for row in span(pointer.1, actual.height) {
        let line = (row * width + columns.start) * 3..(row * width + columns.end) * 3;
        actual.rgb[line.clone()].copy_from_slice(&expected.rgb[line]);
    }
}

/// The pointer's position after `steps`, which begin with it at `start`.
fn last_pointer(steps: &[Step], start: (i32, i32)) -> (i32, i32) {
    steps
        .iter()
        .rev()
        .find_map(|step| match step {
            Step::Pointer(args) => match args.as_slice() {
                [verb, x, y] if verb == "set" => Some((x.parse().ok()?, y.parse().ok()?)),
                _ => None,
            },
            Step::Type(_)
            | Step::Wheel(_)
            | Step::Output(..)
            | Step::Checkpoint(_)
            | Step::EndKeyboard
            | Step::Clipboard(_) => None,
        })
        .unwrap_or(start)
}

// ---- Exit checks -------------------------------------------------------

/// Checks how the app ended against the case's `ending`.
fn check_exit_status(status: ExitStatus, ending: Ending) -> Result<(), String> {
    let good = match ending {
        Ending::Close => status.code() == Some(0),
        // The app re-raises the signal after its ordered quit (ADR-T48-1).
        Ending::Sigint => status.signal() == Some(2),
    };
    if good {
        Ok(())
    } else {
        Err(format!("bad exit for {ending:?}: {status}"))
    }
}

/// The `WARN` and `ERROR` lines of a session log.
fn check_exit_log(log: &str) -> Result<(), String> {
    let bad: Vec<&str> = log
        .lines()
        .filter(|line| line.contains(" WARN ") || line.contains(" ERROR "))
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!("anomalies in draw.log:\n{}", bad.join("\n")))
    }
}

/// Fails if `state_dir` holds a crash report.
fn check_exit_crash(state_dir: &Path) -> Result<(), String> {
    let crash = state_dir.join("crash.log");
    match std::fs::read_to_string(&crash) {
        Ok(text) => Err(format!("crash.log was written:\n{text}")),
        Err(_) => Ok(()),
    }
}

// ---- Waiting -----------------------------------------------------------

/// Waits until the idle file at `idle` holds `target` or more. Fails after
/// `timeout`, or when `child` exits first, with the log at `log` attached.
fn wait_for_sync(
    idle: &Path,
    target: u64,
    timeout: Duration,
    child: &mut Child,
    log: &Path,
) -> Result<(), String> {
    let started = Instant::now();
    let failure = |what: String| {
        let text = std::fs::read_to_string(log).unwrap_or_else(|err| format!("<{err}>"));
        Err(format!("{what}\n--- {} ---\n{text}", log.display()))
    };
    loop {
        let reached = std::fs::read_to_string(idle)
            .ok()
            .and_then(|text| text.trim().parse::<u64>().ok())
            .is_some_and(|count| count >= target);
        if reached {
            return Ok(());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                return failure(format!("the app exited with {status} before sync {target}"));
            }
            Ok(None) => {}
            Err(err) => return failure(format!("cannot watch the app: {err}")),
        }
        if started.elapsed() >= timeout {
            return failure(format!(
                "timed out after {timeout:?} waiting for sync {target}"
            ));
        }
        std::thread::sleep(POLL);
    }
}

// ---- Session -----------------------------------------------------------

/// The private session's sockets, checked to be the script's own.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Guard {
    runtime: PathBuf,
    wayland: PathBuf,
    sway: PathBuf,
}

/// The session's sockets, or why they must not be used. Nothing may be
/// spawned before this succeeds (ADR-T75-1).
fn session_guard(
    runtime: Option<OsString>,
    wayland: Option<OsString>,
    sway: Option<OsString>,
) -> Result<Guard, String> {
    let runtime = runtime.ok_or("DRAW_E2E_RUNTIME is not set: run scripts/e2e.sh")?;
    let runtime = PathBuf::from(runtime);
    if !runtime.is_absolute() {
        return Err(format!("{} is not absolute", runtime.display()));
    }
    let private_socket = |name: &str, value: Option<OsString>| -> Result<PathBuf, String> {
        let path = PathBuf::from(value.ok_or_else(|| format!("{name} is not set"))?);
        if !path.starts_with(&runtime) || path.parent() != Some(runtime.as_path()) {
            return Err(format!(
                "{name}={} is outside the private {}",
                path.display(),
                runtime.display()
            ));
        }
        let kind = std::fs::metadata(&path)
            .map_err(|err| format!("{name}={}: {err}", path.display()))?
            .file_type();
        if !kind.is_socket() {
            return Err(format!("{name}={} is not a socket", path.display()));
        }
        Ok(path)
    };
    let wayland = private_socket("WAYLAND_DISPLAY", wayland)?;
    let sway = private_socket("SWAYSOCK", sway)?;
    Ok(Guard {
        runtime,
        wayland,
        sway,
    })
}

/// The display server a run drives (T129).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    /// The private headless sway.
    Wayland,
    /// The private Xvfb.
    X11,
}

impl Backend {
    /// The backend called `name` by `--backend`.
    fn from_name(name: &str) -> Result<Self, String> {
        match name {
            "wayland" => Ok(Self::Wayland),
            "x11" => Ok(Self::X11),
            other => Err(format!("unknown backend `{other}`")),
        }
    }

    /// The backend of this run, from `DRAW_E2E_BACKEND`; Wayland if unset.
    fn from_env() -> Result<Self, String> {
        std::env::var("DRAW_E2E_BACKEND").map_or(Ok(Self::Wayland), |name| Self::from_name(&name))
    }
}

/// The path a golden `name` of a case has for `backend`: `expected.x11.ppm`
/// for `expected.ppm` on X11. Only a golden that differs by backend has
/// such a file.
fn backend_golden(source: &Path, name: &str, backend: Backend) -> PathBuf {
    match (backend, name.rsplit_once('.')) {
        (Backend::X11, Some((stem, ext))) => source.join(format!("{stem}.x11.{ext}")),
        _ => source.join(name),
    }
}

/// The golden `name` of a case: the backend's own file if it exists, else
/// the shared one.
fn golden_path(source: &Path, name: &str, backend: Backend) -> PathBuf {
    let own = backend_golden(source, name, backend);
    if own.exists() { own } else { source.join(name) }
}

/// The private X11 session, checked to be the script's own.
#[derive(Debug, Clone, PartialEq, Eq)]
struct X11Guard {
    runtime: PathBuf,
    display: String,
    authority: PathBuf,
}

/// The X11 session's display and authority file, or why they must not be
/// used. Nothing may be spawned before this succeeds (ADR-T75-1).
fn x11_guard(
    runtime: Option<OsString>,
    display: Option<OsString>,
    authority: Option<OsString>,
) -> Result<X11Guard, String> {
    let runtime = PathBuf::from(runtime.ok_or("DRAW_E2E_RUNTIME is not set: run scripts/e2e.sh")?);
    if !runtime.is_absolute() {
        return Err(format!("{} is not absolute", runtime.display()));
    }
    let display = display
        .ok_or("DISPLAY is not set")?
        .into_string()
        .map_err(|_| "DISPLAY is not text".to_owned())?;
    let number = display.strip_prefix(':').unwrap_or("");
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("DISPLAY={display} is not a local display"));
    }
    let authority = PathBuf::from(authority.ok_or("XAUTHORITY is not set")?);
    if authority.parent() != Some(runtime.as_path()) {
        return Err(format!(
            "XAUTHORITY={} is outside the private {}",
            authority.display(),
            runtime.display()
        ));
    }
    if !authority.is_file() {
        return Err(format!("XAUTHORITY={} is not a file", authority.display()));
    }
    Ok(X11Guard {
        runtime,
        display,
        authority,
    })
}

/// X11 `InternAtom` (only-if-exists off) for `name`, little endian.
fn x11_intern_request(name: &str) -> Vec<u8> {
    let len = u16::try_from(name.len()).unwrap_or(u16::MAX);
    let words = 2 + name.len().div_ceil(4);
    let mut request = vec![16, 0];
    request.extend(u16::try_from(words).unwrap_or(u16::MAX).to_le_bytes());
    request.extend(len.to_le_bytes());
    request.extend([0, 0]);
    request.extend(name.as_bytes());
    request.resize(words * 4, 0);
    request
}

/// X11 `SendEvent` of a `WM_PROTOCOLS` client message that asks `window` to
/// close (`WM_DELETE_WINDOW`), little endian.
fn x11_delete_request(window: u32, protocols: u32, delete: u32) -> Vec<u8> {
    let mut request = vec![25, 0, 11, 0];
    request.extend(window.to_le_bytes());
    request.extend(0u32.to_le_bytes());
    request.extend([33, 32, 0, 0]);
    for word in [window, protocols, delete, 0, 0, 0, 0] {
        request.extend(word.to_le_bytes());
    }
    request
}

/// Asks the window `window` of the local `display` to close, as a window
/// manager does: the private server has none.
fn x11_close(display: &str, window: u32) -> Result<(), String> {
    let number = display.strip_prefix(':').unwrap_or("0");
    let path = format!("/tmp/.X11-unix/X{number}");
    let mut stream = UnixStream::connect(&path).map_err(|err| format!("{path}: {err}"))?;
    let io = |err: std::io::Error| format!("X11 {path}: {err}");
    // Handshake without authorisation: byte order, version 11.0.
    stream
        .write_all(&[b'l', 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        .map_err(io)?;
    let mut head = [0u8; 8];
    stream.read_exact(&mut head).map_err(io)?;
    if head.first() != Some(&1) {
        return Err(format!("the X server at {path} refused the connection"));
    }
    let extra = usize::from(u16::from_le_bytes([head[6], head[7]])) * 4;
    stream.read_exact(&mut vec![0u8; extra]).map_err(io)?;
    let atom = |stream: &mut UnixStream, name: &str| -> Result<u32, String> {
        stream.write_all(&x11_intern_request(name)).map_err(io)?;
        let mut reply = [0u8; 32];
        stream.read_exact(&mut reply).map_err(io)?;
        if reply.first() != Some(&1) {
            return Err(format!("X11 InternAtom {name} failed"));
        }
        Ok(u32::from_le_bytes([
            reply[8], reply[9], reply[10], reply[11],
        ]))
    };
    let protocols = atom(&mut stream, "WM_PROTOCOLS")?;
    let delete = atom(&mut stream, "WM_DELETE_WINDOW")?;
    stream
        .write_all(&x11_delete_request(window, protocols, delete))
        .map_err(io)?;
    // A round trip: the server has handled the event when it answers.
    atom(&mut stream, "WM_PROTOCOLS").map(drop)
}

/// The `xdotool` invocations that do what the `wtype` arguments `args`
/// do: modifiers, keys, holds, pauses and literal text.
fn xdotool_commands(args: &[String]) -> Result<Vec<Vec<String>>, String> {
    let modifier = |name: &str| match name {
        "ctrl" => "Control_L".to_owned(),
        "shift" => "Shift_L".to_owned(),
        "alt" => "Alt_L".to_owned(),
        "logo" => "Super_L".to_owned(),
        other => other.to_owned(),
    };
    let mut commands = Vec::new();
    let mut chain: Vec<String> = Vec::new();
    let mut words = args.iter();
    while let Some(word) = words.next() {
        let mut operand = |what: &str| {
            words
                .next()
                .cloned()
                .ok_or_else(|| format!("{word} needs {what}"))
        };
        match word.as_str() {
            "-M" => chain.extend(["keydown".to_owned(), modifier(&operand("a modifier")?)]),
            "-m" => chain.extend(["keyup".to_owned(), modifier(&operand("a modifier")?)]),
            "-k" => chain.extend(["key".to_owned(), operand("a key")?]),
            "-P" => chain.extend(["keydown".to_owned(), operand("a key")?]),
            "-p" => chain.extend(["keyup".to_owned(), operand("a key")?]),
            "-s" => {
                let ms: u32 = number(&operand("milliseconds")?)?;
                chain.extend(["sleep".to_owned(), format!("{}", f64::from(ms) / 1000.0)]);
            }
            text => {
                if !chain.is_empty() {
                    commands.push(std::mem::take(&mut chain));
                }
                commands.push(
                    ["type", "--delay", "0", "--", text]
                        .map(str::to_owned)
                        .to_vec(),
                );
            }
        }
    }
    if !chain.is_empty() {
        commands.push(chain);
    }
    Ok(commands)
}

/// One Wayland protocol message: header and payload.
fn wire(object: u32, opcode: u32, payload: &[u8]) -> Vec<u8> {
    let size = u32::try_from(8 + payload.len()).unwrap_or(u32::MAX);
    let mut message = Vec::with_capacity(8 + payload.len());
    message.extend(object.to_ne_bytes());
    message.extend(((size << 16) | opcode).to_ne_bytes());
    message.extend(payload);
    message
}

/// A Wayland string argument: length, bytes, NUL, padding to 4 bytes.
fn wire_string(text: &str) -> Vec<u8> {
    let length = u32::try_from(text.len() + 1).unwrap_or(u32::MAX);
    let mut out = length.to_ne_bytes().to_vec();
    out.extend(text.as_bytes());
    out.push(0);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    out
}

/// Protocol object id of the virtual pointer [`announce_pointer`] creates.
const VIRTUAL_POINTER_ID: u32 = 5;

/// Scroll distance of one notch, as a real wheel reports it.
const NOTCH_UNITS: i32 = 15;

/// One wheel notch on the virtual pointer: axis source `wheel`, a
/// vertical `axis_discrete` and a `frame` (ADR-T75-2).
fn wheel_message(notch: i32) -> Vec<u8> {
    const AXIS_SOURCE: u32 = 5;
    const AXIS_DISCRETE: u32 = 7;
    const FRAME: u32 = 4;
    let (time, vertical) = (0u32, 0u32);
    // wl_fixed_t: 24.8 fixed point.
    let value = (NOTCH_UNITS * notch) << 8;
    let mut discrete = time.to_ne_bytes().to_vec();
    discrete.extend(vertical.to_ne_bytes());
    discrete.extend(value.to_ne_bytes());
    discrete.extend(notch.to_ne_bytes());
    let mut message = wire(VIRTUAL_POINTER_ID, AXIS_SOURCE, &0u32.to_ne_bytes());
    message.extend(wire(VIRTUAL_POINTER_ID, AXIS_DISCRETE, &discrete));
    message.extend(wire(VIRTUAL_POINTER_ID, FRAME, &[]));
    message
}

/// The global name of `interface` in a stream of registry events, if the
/// messages seen so far announce it; and whether callback 3 has fired.
fn scan_registry(mut bytes: &[u8], interface: &str) -> (Option<u32>, bool) {
    let word = |b: &[u8], i: usize| -> Option<u32> {
        let raw = b.get(i..i + 4)?;
        Some(u32::from_ne_bytes(raw.try_into().ok()?))
    };
    let (mut found, mut done) = (None, false);
    while let (Some(object), Some(header)) = (word(bytes, 0), word(bytes, 4)) {
        let size = (header >> 16) as usize;
        let Some(body) = bytes.get(8..size) else {
            break;
        };
        if object == 3 {
            done = true;
        }
        let opcode = header & 0xffff;
        if object == 2 && opcode == 0 {
            let length = word(body, 4).unwrap_or(0) as usize;
            let name = body.get(8..8 + length.saturating_sub(1));
            if name == Some(interface.as_bytes()) {
                found = word(body, 0);
            }
        }
        bytes = bytes.get(size..).unwrap_or(&[]);
    }
    (found, done)
}

/// Connects to `socket` and reads every global the compositor announces.
fn connect_registry(socket: &Path, what: &str) -> Result<(UnixStream, Vec<u8>), String> {
    let fail = |err: std::io::Error| format!("{what}: {err}");
    let mut stream = UnixStream::connect(socket).map_err(fail)?;
    stream.set_read_timeout(Some(SYNC_TIMEOUT)).map_err(fail)?;
    // wl_display.get_registry(2), wl_display.sync(3).
    let mut hello = wire(1, 1, &2u32.to_ne_bytes());
    hello.extend(wire(1, 0, &3u32.to_ne_bytes()));
    stream.write_all(&hello).map_err(fail)?;
    let mut received = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = stream.read(&mut chunk).map_err(fail)?;
        if n == 0 {
            return Err(format!("{what}: the compositor hung up"));
        }
        received.extend_from_slice(chunk.get(..n).unwrap_or(&[]));
        if scan_registry(&received, "").1 {
            return Ok((stream, received));
        }
    }
}

/// `wl_registry.bind(name, interface, version 1, id)`.
fn bind_message(name: u32, interface: &str, id: u32) -> Vec<u8> {
    let mut bind = name.to_ne_bytes().to_vec();
    bind.extend(wire_string(interface));
    bind.extend(1u32.to_ne_bytes());
    bind.extend(id.to_ne_bytes());
    wire(2, 0, &bind)
}

/// Creates a virtual pointer and keeps it while the stream is open. It
/// gives the seat a pointer (ADR-T75-1) and carries wheel notches
/// (ADR-T75-2); `swaymsg` drives every other pointer event.
fn announce_pointer(socket: &Path) -> Result<UnixStream, String> {
    const INTERFACE: &str = "zwlr_virtual_pointer_manager_v1";
    let (mut stream, globals) = connect_registry(socket, "virtual pointer")?;
    let name = scan_registry(&globals, INTERFACE)
        .0
        .ok_or("virtual pointer: the compositor has no manager")?;
    // create_virtual_pointer(no seat, id 5).
    let mut create = 0u32.to_ne_bytes().to_vec();
    create.extend(VIRTUAL_POINTER_ID.to_ne_bytes());
    let mut message = bind_message(name, INTERFACE, 4);
    message.extend(wire(4, 0, &create));
    stream
        .write_all(&message)
        .map_err(|err| format!("virtual pointer: {err}"))?;
    Ok(stream)
}

/// Creates a virtual keyboard that never gets a keymap and keeps it while
/// the stream is open. On a seat with no keyboard sway adopts it, so
/// clients get `no_keymap` (ADR-T75-1, T79).
fn announce_bare_keyboard(socket: &Path) -> Result<UnixStream, String> {
    const MANAGER: &str = "zwp_virtual_keyboard_manager_v1";
    const SEAT: &str = "wl_seat";
    const MANAGER_ID: u32 = 4;
    const SEAT_ID: u32 = 5;
    const KEYBOARD_ID: u32 = 6;
    let (mut stream, globals) = connect_registry(socket, "virtual keyboard")?;
    let global = |interface: &str| {
        scan_registry(&globals, interface)
            .0
            .ok_or_else(|| format!("virtual keyboard: the compositor has no {interface}"))
    };
    let mut message = bind_message(global(MANAGER)?, MANAGER, MANAGER_ID);
    message.extend(bind_message(global(SEAT)?, SEAT, SEAT_ID));
    // create_virtual_keyboard(seat, id).
    let mut create = SEAT_ID.to_ne_bytes().to_vec();
    create.extend(KEYBOARD_ID.to_ne_bytes());
    message.extend(wire(MANAGER_ID, 0, &create));
    stream
        .write_all(&message)
        .map_err(|err| format!("virtual keyboard: {err}"))?;
    Ok(stream)
}

/// The running private session: tools get its sockets and nothing else.
struct Session {
    guard: Guard,
    backend: Backend,
    /// The private X server, on the X11 backend.
    x11: Option<X11Guard>,
    /// The app's window, once it has one, on the X11 backend.
    window: Option<String>,
    /// Holds the seat's pointer and carries wheel notches.
    pointer: Option<UnixStream>,
    /// `wtype` processes still alive, oldest first.
    keyboards: VecDeque<Child>,
}

impl Session {
    /// Joins the session named by the environment, after checking it.
    fn join() -> Result<Self, String> {
        match Backend::from_env()? {
            Backend::Wayland => Self::join_wayland(),
            Backend::X11 => Self::join_x11(),
        }
    }

    fn join_x11() -> Result<Self, String> {
        let x11 = x11_guard(
            std::env::var_os("DRAW_E2E_RUNTIME"),
            std::env::var_os("DISPLAY"),
            std::env::var_os("XAUTHORITY"),
        )?;
        Ok(Self {
            guard: Guard {
                runtime: x11.runtime.clone(),
                wayland: PathBuf::new(),
                sway: PathBuf::new(),
            },
            backend: Backend::X11,
            x11: Some(x11),
            window: None,
            pointer: None,
            keyboards: VecDeque::new(),
        })
    }

    fn join_wayland() -> Result<Self, String> {
        let guard = session_guard(
            std::env::var_os("DRAW_E2E_RUNTIME"),
            std::env::var_os("WAYLAND_DISPLAY"),
            std::env::var_os("SWAYSOCK"),
        )?;
        let pointer = announce_pointer(&guard.wayland)?;
        let mut session = Self {
            guard,
            backend: Backend::Wayland,
            x11: None,
            window: None,
            pointer: Some(pointer),
            keyboards: VecDeque::new(),
        };
        // A keyboard on the seat from before the app starts.
        let first = session.spawn_wtype(&["-s".to_owned(), KEEP_ALIVE_MS.to_owned()])?;
        session.keyboards.push_back(first);
        session.wait_for_seat()?;
        Ok(session)
    }

    /// A command for a tool, with the private sockets as its whole world.
    fn tool(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("XDG_RUNTIME_DIR", &self.guard.runtime)
            .stdin(Stdio::null());
        if let Some(x11) = &self.x11 {
            command
                .env("DISPLAY", &x11.display)
                .env("XAUTHORITY", &x11.authority);
        } else {
            command
                .env("WAYLAND_DISPLAY", &self.guard.wayland)
                .env("SWAYSOCK", &self.guard.sway);
        }
        command
    }

    /// Saves what the output (or the app's window on X11) shows to `path`
    /// as a PPM.
    fn screenshot(&self, path: &Path) -> Result<(), String> {
        let (tool, status) = if let Some(window) = &self.window {
            let target = format!("ppm:{}", path.display());
            let status = self
                .tool("import")
                .args(["-depth", "8", "-window", window, &target])
                .status();
            ("import", status)
        } else {
            let status = self.tool("grim").args(["-t", "ppm"]).arg(path).status();
            ("grim", status)
        };
        let status = status.map_err(|err| format!("{tool}: {err}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("{tool} failed: {status}"))
        }
    }

    /// Runs `xdotool` with `args` and returns its stdout.
    fn xdotool(&self, args: &[&str]) -> Result<String, String> {
        let output = self
            .tool("xdotool")
            .args(args)
            .output()
            .map_err(|err| format!("xdotool: {err}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.status.success() {
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("xdotool {args:?}: {stdout}{stderr}"))
        }
    }

    /// Types the `wtype` arguments `args` through the backend's tool. On
    /// Wayland the keyboard stays alive afterwards (ADR-T75-1).
    fn type_args(&mut self, args: &[String]) -> Result<(), String> {
        if self.backend == Backend::X11 {
            for command in xdotool_commands(args)? {
                let words: Vec<&str> = command.iter().map(String::as_str).collect();
                self.xdotool(&words)?;
            }
            return Ok(());
        }
        let mut all = args.to_vec();
        all.extend(["-s".to_owned(), KEEP_ALIVE_MS.to_owned()]);
        let keyboard = self.spawn_wtype(&all)?;
        self.keyboards.push_back(keyboard);
        Ok(())
    }

    /// One pointer step: `swaymsg seat - cursor` arguments on Wayland, the
    /// same words as `xdotool` calls on X11.
    fn pointer_step(&self, args: &[String]) -> Result<(), String> {
        if self.backend == Backend::Wayland {
            let mut command = vec!["seat", "-", "cursor"];
            command.extend(args.iter().map(String::as_str));
            return self.swaymsg(&command).map(drop);
        }
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        let number = |button: &str| button.strip_prefix("button").unwrap_or("1").to_owned();
        let call = match words.as_slice() {
            ["set", x, y] => vec!["mousemove".to_owned(), (*x).to_owned(), (*y).to_owned()],
            ["press", b] => vec!["mousedown".to_owned(), number(b)],
            ["release", b] => vec!["mouseup".to_owned(), number(b)],
            other => return Err(format!("no X11 pointer step for {other:?}")),
        };
        let call: Vec<&str> = call.iter().map(String::as_str).collect();
        self.xdotool(&call).map(drop)
    }

    /// One wheel notch: negative is up (ADR-T75-2).
    fn wheel(&mut self, notch: i32) -> Result<(), String> {
        if let Some(pointer) = &mut self.pointer {
            return pointer
                .write_all(&wheel_message(notch))
                .map_err(|err| format!("wheel: {err}"));
        }
        self.xdotool(&["click", if notch < 0 { "4" } else { "5" }])
            .map(drop)
    }

    /// Sizes the output (the window on X11) to `w` by `h`.
    fn resize(&self, w: u32, h: u32) -> Result<(), String> {
        if let Some(window) = &self.window {
            return self
                .xdotool(&["windowsize", window, &w.to_string(), &h.to_string()])
                .map(drop);
        }
        let mode = format!("{w}x{h}");
        self.swaymsg(&["output", OUTPUT_NAME, "resolution", &mode])
            .map(drop)
    }

    /// Asks the app's window to close.
    fn close_window(&self) -> Result<(), String> {
        match (&self.window, &self.x11) {
            (Some(_), Some(x11)) => {
                // Every window of the app: the private server has no others.
                let found = self.xdotool(&["search", "--class", "draw"])?;
                for id in found.split_whitespace() {
                    let id = id.parse().map_err(|_| format!("bad window id {id}"))?;
                    x11_close(&x11.display, id)?;
                }
                Ok(())
            }
            (Some(_), None) => Err("a window without an X11 session".to_owned()),
            (None, _) => self.swaymsg(&["[app_id=\"draw\"]", "kill"]).map(drop),
        }
    }

    fn spawn_wtype(&self, args: &[String]) -> Result<Child, String> {
        self.tool("wtype")
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("wtype: {err}"))
    }

    /// Runs `swaymsg` and returns its stdout.
    fn swaymsg(&self, args: &[&str]) -> Result<String, String> {
        let output = self
            .tool("swaymsg")
            .args(args)
            .output()
            .map_err(|err| format!("swaymsg: {err}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(format!("swaymsg {args:?}: {stdout}"))
        }
    }

    /// Waits until the seat has a pointer and a keyboard.
    fn wait_for_seat(&self) -> Result<(), String> {
        let started = Instant::now();
        loop {
            if self
                .swaymsg(&["-t", "get_seats"])?
                .contains("\"capabilities\": 3")
            {
                return Ok(());
            }
            if started.elapsed() >= SYNC_TIMEOUT {
                return Err("the seat never got a pointer and a keyboard".to_owned());
            }
            std::thread::sleep(POLL);
        }
    }

    /// Waits until the app's window is in the tree.
    fn wait_for_window(&mut self, child: &mut Child) -> Result<(), String> {
        let started = Instant::now();
        loop {
            if self.backend == Backend::X11 {
                // The private server holds no other client's window.
                let found = self
                    .xdotool(&["search", "--onlyvisible", "--class", "draw"])
                    .unwrap_or_default();
                if let Some(id) = found.split_whitespace().next() {
                    let id = id.to_owned();
                    self.xdotool(&["windowmove", &id, "0", "0"])?;
                    self.xdotool(&["windowfocus", &id])?;
                    self.window = Some(id);
                    return Ok(());
                }
            } else if self
                .swaymsg(&["-t", "get_tree"])?
                .contains("\"app_id\": \"draw\"")
            {
                return Ok(());
            }
            if let Ok(Some(status)) = child.try_wait() {
                return Err(format!("the app exited with {status} before its window"));
            }
            if started.elapsed() >= SYNC_TIMEOUT {
                return Err("the window never appeared".to_owned());
            }
            std::thread::sleep(POLL);
        }
    }

    /// Stops the newest keyboard, which is the active one, so that sway
    /// adopts a new keyboard without a keymap. An idle spare keeps the
    /// seat's keyboard capability, so the app keeps its `wl_keyboard`. The
    /// returned stream holds the new keyboard (ADR-T75-1, T79).
    fn replace_active_keyboard(&mut self) -> Result<UnixStream, String> {
        let Some(mut active) = self.keyboards.pop_back() else {
            return Err("no active keyboard to end".to_owned());
        };
        let spare = self.spawn_wtype(&["-s".to_owned(), KEEP_ALIVE_MS.to_owned()])?;
        self.keyboards.push_back(spare);
        self.wait_for_keyboards(2)?;
        let _ = active.kill();
        let _ = active.wait();
        self.wait_for_keyboards(1)?;
        let bare = announce_bare_keyboard(&self.guard.wayland)?;
        self.wait_for_keyboards(2)?;
        Ok(bare)
    }

    /// Waits until the compositor lists exactly `count` keyboards.
    fn wait_for_keyboards(&self, count: usize) -> Result<(), String> {
        let started = Instant::now();
        loop {
            let inputs = self.swaymsg(&["-t", "get_inputs"])?;
            if inputs.matches("\"type\": \"keyboard\"").count() == count {
                return Ok(());
            }
            if started.elapsed() >= SYNC_TIMEOUT {
                return Err(format!("the compositor never listed {count} keyboards"));
            }
            std::thread::sleep(POLL);
        }
    }

    /// Stops every keyboard but the newest, which is the active one.
    fn retire_old_keyboards(&mut self) {
        while self.keyboards.len() > 1 {
            if let Some(mut old) = self.keyboards.pop_front() {
                let _ = old.kill();
                let _ = old.wait();
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        for keyboard in &mut self.keyboards {
            let _ = keyboard.kill();
            let _ = keyboard.wait();
        }
    }
}

// ---- Running a case ----------------------------------------------------

/// The `strokes-5000` scene: rows of pen strokes.
fn strokes_5000() -> Editor {
    let mut editor = scenes::editor();
    for i in 0..SCENE_STROKES {
        let (x, y) = ((i % 100) as f32 * 12.0, (i / 100) as f32 * 12.0);
        scenes::drag(&mut editor, scenes::v(x, y), scenes::v(x + 9.0, y + 9.0));
    }
    editor
}

/// The start snapshot of a case, written into its run directory.
fn prepare_start(case: &Case, source: &Path, run: &Path) -> Result<Option<PathBuf>, String> {
    let path = run.join("start.snap");
    if let Some(scene) = &case.scene {
        let editor = match scene.as_str() {
            "strokes-5000" => strokes_5000(),
            other => return Err(format!("unknown scene {other}")),
        };
        std::fs::write(&path, editor.snapshot().to_text()).map_err(|err| err.to_string())?;
        return Ok(Some(path));
    }
    let start = source.join("start.snap");
    if start.exists() {
        return Ok(Some(start));
    }
    Ok(None)
}

/// One running case.
struct Running<'a> {
    session: &'a mut Session,
    child: Child,
    idle: PathBuf,
    dump: PathBuf,
    log: PathBuf,
    /// File the app's clipboard hook reads.
    clipboard: PathBuf,
    /// Syncs sent so far.
    sent: u64,
}

impl Running<'_> {
    /// Types `args` and then the sync chord, and waits for the app.
    fn sync_with(&mut self, args: &[String]) -> Result<(), String> {
        let mut all = args.to_vec();
        all.extend(SYNC_ARGS.iter().map(|&a| a.to_owned()));
        self.session.type_args(&all)?;
        self.sent += 1;
        wait_for_sync(
            &self.idle,
            self.sent,
            SYNC_TIMEOUT,
            &mut self.child,
            &self.log,
        )?;
        self.session.retire_old_keyboards();
        Ok(())
    }

    fn snapshot(&self) -> Result<String, String> {
        std::fs::read_to_string(&self.dump).map_err(|err| format!("{}: {err}", self.dump.display()))
    }

    /// Syncs until the window has the size `(w, h)`.
    fn wait_for_viewport(&mut self, (w, h): (u32, u32)) -> Result<(), String> {
        let wanted = format!("viewport {w}.0 {h}.0");
        let started = Instant::now();
        loop {
            self.sync_with(&[])?;
            if self.snapshot()?.lines().any(|line| line == wanted) {
                return Ok(());
            }
            if started.elapsed() >= SYNC_TIMEOUT {
                return Err(format!("the window never became {w}x{h}"));
            }
        }
    }

    fn run_step(&mut self, step: &Step, source: &Path, sections: &[String]) -> Result<(), String> {
        match step {
            Step::Type(args) => self.sync_with(args),
            Step::Pointer(args) => {
                self.session.pointer_step(args)?;
                self.sync_with(&[])
            }
            Step::Wheel(notch) => {
                self.session.wheel(*notch)?;
                self.sync_with(&[])
            }
            Step::Output(w, h) => {
                self.session.resize(*w, *h)?;
                self.wait_for_viewport((*w, *h))
            }
            Step::Checkpoint(name) => {
                let actual = select_sections(&self.snapshot()?, sections);
                compare_golden(
                    &actual,
                    source,
                    &format!("{name}.snap"),
                    self.session.backend,
                    update_requested(),
                )
            }
            Step::EndKeyboard => {
                // X11 has no keymap hand-over to lose.
                if self.session.backend == Backend::Wayland {
                    let _bare = self.session.replace_active_keyboard()?;
                }
                self.sync_with(&[])
            }
            Step::Clipboard(text) => {
                self.set_clipboard(text)?;
                self.sync_with(&[])
            }
        }
    }

    /// Puts `text` on the clipboard with `wl-copy`, or through the app's
    /// own hook where it is not installed, and always on X11, where the
    /// private server has no clipboard manager (ADR-T83-1).
    fn set_clipboard(&self, text: &str) -> Result<(), String> {
        let copied = if self.session.backend == Backend::X11 {
            Err(std::io::ErrorKind::NotFound.into())
        } else {
            self.session
                .tool("wl-copy")
                .arg(text)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        };
        match copied {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(format!("wl-copy failed: {status}")),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // In one step, so the app never reads half a text.
                let partial = self.clipboard.with_extension("tmp");
                std::fs::write(&partial, text)
                    .and_then(|()| std::fs::rename(&partial, &self.clipboard))
                    .map_err(|err| format!("clipboard hook: {err}"))
            }
            Err(err) => Err(format!("wl-copy: {err}")),
        }
    }
}

impl Drop for Running<'_> {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Makes `command` start from the snapshot `start`: as an autosave in the
/// `state` directory if `recover`, else through `DRAW_SNAPSHOT`.
fn start_from(
    command: &mut Command,
    start: &Path,
    recover: bool,
    state: &Path,
) -> Result<(), String> {
    if recover {
        let dir = state.join("draw");
        std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        std::fs::copy(start, dir.join("autosave.snap")).map_err(|err| err.to_string())?;
        command.arg("--recover");
    } else {
        command.env("DRAW_SNAPSHOT", start);
    }
    Ok(())
}

/// Runs the case in `source`, using `run` as its scratch directory.
fn run_case(session: &mut Session, source: &Path, run: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(source.join("input")).map_err(|err| err.to_string())?;
    let case = parse_input(&text)?;
    let home = run.join("home");
    let state = run.join("state");
    for dir in [&home, &state] {
        std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    }
    let (dump, idle) = (run.join("actual.snap"), run.join("idle"));
    let clipboard = run.join("clipboard");
    let log = state.join("draw").join("draw.log");
    if session.backend == Backend::Wayland {
        session.resize(OUTPUT.0, OUTPUT.1)?;
    }
    let start = prepare_start(&case, source, run)?;
    let stdout = std::fs::File::create(run.join("draw.out")).map_err(|err| err.to_string())?;
    let stderr = stdout.try_clone().map_err(|err| err.to_string())?;
    let mut command = session.tool(env!("CARGO_BIN_EXE_draw"));
    command
        .env("HOME", &home)
        .env("XDG_STATE_HOME", &state)
        .env("DRAW_LOG", "info")
        .env("DRAW_TABLET", "off")
        .env("LIBGL_ALWAYS_SOFTWARE", "1")
        .env("DRAW_E2E_DUMP", &dump)
        .env("DRAW_E2E_IDLE", &idle)
        .env("DRAW_E2E_CLIPBOARD", &clipboard)
        .stdout(stdout)
        .stderr(stderr);
    if let Some(start) = start {
        start_from(&mut command, &start, case.recover, &state)?;
    }
    let mut child = command.spawn().map_err(|err| format!("draw: {err}"))?;
    session.wait_for_window(&mut child)?;
    if session.backend == Backend::X11 {
        session.resize(OUTPUT.0, OUTPUT.1)?;
    }
    let mut running = Running {
        session,
        child,
        idle,
        dump,
        log: log.clone(),
        clipboard,
        sent: 0,
    };
    running.wait_for_viewport(OUTPUT)?;
    // Entering a surface gives no motion event, and the backend would
    // report the first press at a stale position: enter, then move.
    for step in [move_to(PARK.0, PARK.1), move_to(PARK.0 + 1, PARK.1)] {
        running.run_step(&step, source, &case.sections)?;
    }
    for step in &case.steps {
        running.run_step(step, source, &case.sections)?;
    }

    let update = update_requested();
    let mut problems = Vec::new();
    let actual = select_sections(&running.snapshot()?, &case.sections);
    if let Err(problem) = compare_golden(
        &actual,
        source,
        "expected.snap",
        running.session.backend,
        update,
    ) {
        problems.push(problem);
    }
    problems.extend(check_images(&running, &case, source, run, update)?);

    match case.ending {
        Ending::Close => {
            running.session.close_window()?;
        }
        Ending::Sigint => {
            let pid = running.child.id().to_string();
            let status = Command::new("kill")
                .args(["-INT", &pid])
                .status()
                .map_err(|err| format!("kill: {err}"))?;
            if !status.success() {
                return Err(format!("kill -INT failed: {status}"));
            }
        }
    }
    let status = wait_for_exit(&mut running.child, SYNC_TIMEOUT, &log)?;
    if let Err(problem) = check_exit_status(status, case.ending) {
        problems.push(problem);
    }
    let log_text = std::fs::read_to_string(&log).map_err(|err| format!("draw.log: {err}"))?;
    if let Err(problem) = check_exit_log(&log_text) {
        problems.push(problem);
    }
    if let Err(problem) = check_exit_crash(&state.join("draw")) {
        problems.push(problem);
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

/// The problems with the case's screenshot and exported file, if it asks
/// for them.
fn check_images(
    running: &Running<'_>,
    case: &Case,
    source: &Path,
    run: &Path,
    update: bool,
) -> Result<Vec<String>, String> {
    let mut problems = Vec::new();
    let pointer = last_pointer(&case.steps, (PARK.0 + 1, PARK.1));
    if let Some(tolerance) = case.screenshot {
        let shot = run.join("actual.ppm");
        running.session.screenshot(&shot)?;
        let backend = running.session.backend;
        let compared =
            compare_screenshot_for(&shot, source, backend, run, tolerance, pointer, update);
        if let Err(problem) = compared {
            problems.push(problem);
        }
    }
    if case.export {
        let shot = run.join("window.ppm");
        running.session.screenshot(&shot)?;
        if let Err(problem) = check_export(&run.join("home"), &shot, run, pointer) {
            problems.push(problem);
        }
    }
    Ok(problems)
}

/// Compares `actual` with the case's golden `name` for `backend`. On X11 an
/// update writes the backend's own file, and only when the shared golden
/// does not already match.
fn compare_golden(
    actual: &str,
    source: &Path,
    name: &str,
    backend: Backend,
    update: bool,
) -> Result<(), String> {
    let path = golden_path(source, name, backend);
    if !update || backend == Backend::Wayland {
        return compare_snapshot(actual, &path, update);
    }
    if compare_snapshot(actual, &path, false).is_ok() {
        return Ok(());
    }
    compare_snapshot(actual, &backend_golden(source, name, backend), true)
}

/// [`compare_screenshot`] against `expected.ppm` of the case, with the
/// same backend rule as [`compare_golden`].
fn compare_screenshot_for(
    shot: &Path,
    source: &Path,
    backend: Backend,
    run: &Path,
    tolerance: Tolerance,
    pointer: (i32, i32),
    update: bool,
) -> Result<(), String> {
    let path = golden_path(source, "expected.ppm", backend);
    if !update || backend == Backend::Wayland {
        return compare_screenshot(shot, &path, run, tolerance, pointer, update);
    }
    if compare_screenshot(shot, &path, run, tolerance, pointer, false).is_ok() {
        return Ok(());
    }
    let own = backend_golden(source, "expected.ppm", backend);
    compare_screenshot(shot, &own, run, tolerance, pointer, true)
}

/// Compares the screenshot at `shot`; writes `diff.ppm` into `run` on a
/// mismatch, or the golden when `update`.
fn compare_screenshot(
    shot: &Path,
    golden: &Path,
    run: &Path,
    tolerance: Tolerance,
    pointer: (i32, i32),
    update: bool,
) -> Result<(), String> {
    let bytes = std::fs::read(shot).map_err(|err| format!("{}: {err}", shot.display()))?;
    if update {
        return std::fs::write(golden, &bytes)
            .map_err(|err| format!("cannot write {}: {err}", golden.display()));
    }
    let mut actual = Image::parse_ppm(&bytes)?;
    let expected =
        std::fs::read(golden).map_err(|err| format!("no golden at {}: {err}", golden.display()))?;
    let expected = Image::parse_ppm(&expected)?;
    hide_cursor_box(&mut actual, &expected, pointer);
    compare_images(&actual, &expected, tolerance).map_err(|diff| {
        let path = run.join("diff.ppm");
        let saved = std::fs::write(&path, diff.image.to_ppm());
        format!("{} (diff: {} {saved:?})", diff.message, path.display())
    })
}

/// The one exported PNG in `home`; an error if there is none or several.
fn find_export(home: &Path) -> Result<PathBuf, String> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(home)
        .map_err(|err| format!("{}: {err}", home.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            name.starts_with("draw-") && name.ends_with(".png")
        })
        .collect();
    found.sort();
    match found.as_slice() {
        [path] => Ok(path.clone()),
        other => Err(format!("expected one exported PNG, found {other:?}")),
    }
}

/// Decodes 8-bit RGBA PNG `bytes` to an RGB [`Image`].
fn decode_png(bytes: &[u8]) -> Result<Image, String> {
    let bad = |err: png::DecodingError| format!("png: {err}");
    let mut reader = png::Decoder::new(bytes).read_info().map_err(bad)?;
    let mut rgba = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut rgba).map_err(bad)?;
    if (info.color_type, info.bit_depth) != (png::ColorType::Rgba, png::BitDepth::Eight) {
        return Err(format!("png: {:?} {:?}", info.color_type, info.bit_depth));
    }
    rgba.truncate(info.buffer_size());
    let (pixels, _) = rgba.as_chunks::<4>();
    let rgb = pixels
        .iter()
        .flat_map(|[red, green, blue, _]| [*red, *green, *blue])
        .collect();
    Ok(Image {
        width: info.width as usize,
        height: info.height as usize,
        rgb,
    })
}

/// Checks that `home` holds one PNG, of the window's size, showing what the
/// screenshot at `shot` shows.
fn check_export(home: &Path, shot: &Path, run: &Path, pointer: (i32, i32)) -> Result<(), String> {
    let path = find_export(home)?;
    let bytes = std::fs::read(&path).map_err(|err| format!("{}: {err}", path.display()))?;
    let exported = decode_png(&bytes)?;
    if (exported.width, exported.height) != (OUTPUT.0 as usize, OUTPUT.1 as usize) {
        return Err(format!(
            "the export is {}x{}, the window {}x{}",
            exported.width, exported.height, OUTPUT.0, OUTPUT.1
        ));
    }
    let bytes = std::fs::read(shot).map_err(|err| format!("{}: {err}", shot.display()))?;
    let mut window = Image::parse_ppm(&bytes)?;
    hide_cursor_box(&mut window, &exported, pointer);
    compare_images(&exported, &window, Tolerance::default()).map_err(|diff| {
        let saved = std::fs::write(run.join("export-diff.ppm"), diff.image.to_ppm());
        format!(
            "the export differs from the window: {} {saved:?}",
            diff.message
        )
    })
}

/// Waits for the app to end.
fn wait_for_exit(child: &mut Child, timeout: Duration, log: &Path) -> Result<ExitStatus, String> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            let text = std::fs::read_to_string(log).unwrap_or_default();
            return Err(format!("the app did not exit\n{text}"));
        }
        std::thread::sleep(POLL);
    }
}

/// Every case directory, sorted, optionally only the named one.
fn case_directories(root: &Path, only: Option<&str>) -> Result<Vec<PathBuf>, String> {
    let mut cases: Vec<PathBuf> = std::fs::read_dir(root)
        .map_err(|err| format!("{}: {err}", root.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.join("input").is_file())
        .filter(|path| {
            only.is_none_or(|name| {
                path.file_name()
                    .is_some_and(|n| n == std::ffi::OsStr::new(name))
            })
        })
        .collect();
    cases.sort();
    Ok(cases)
}

#[test]
#[ignore = "needs the private session of scripts/e2e.sh"]
fn e2e_every_case() {
    // Arrange: refuses to go on outside the private session.
    let mut session = Session::join().unwrap_or_else(|err| panic!("{err}"));
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join("tests/e2e");
    let only = std::env::var("DRAW_E2E_CASE").ok();
    let cases = case_directories(&root, only.as_deref()).unwrap_or_else(|err| panic!("{err}"));
    assert!(!cases.is_empty(), "no case to run");
    let target = manifest.join("target/e2e");

    // Act
    let mut failures = Vec::new();
    for source in &cases {
        let name = source
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let run = target.join(&name);
        let _ = std::fs::remove_dir_all(&run);
        std::fs::create_dir_all(&run).unwrap_or_else(|err| panic!("{}: {err}", run.display()));
        let started = Instant::now();
        match run_case(&mut session, source, &run) {
            Ok(()) => {
                println!("ok   {name} ({:.1?})", started.elapsed());
                let _ = std::fs::remove_dir_all(&run);
            }
            Err(problem) => {
                println!("FAIL {name}: kept in {}\n{problem}", run.display());
                failures.push(name);
            }
        }
    }

    // Assert
    assert!(failures.is_empty(), "failed cases: {failures:?}");
}

// ---- Self-tests: no display needed ---------------------------------------

#[cfg(test)]
mod self_tests {
    use super::*;
    use std::os::unix::net::UnixListener;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("draw-e2e-self-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display()));
        dir
    }

    /// The error of a result that must have failed.
    trait Failure {
        fn failure(self) -> String;
    }

    impl<T: std::fmt::Debug> Failure for Result<T, String> {
        fn failure(self) -> String {
            match self {
                Ok(value) => panic!("expected an error, got {value:?}"),
                Err(err) => err,
            }
        }
    }

    fn parsed(text: &str) -> Case {
        parse_input(text).unwrap_or_else(|err| panic!("{err}"))
    }

    fn solid(width: usize, height: usize, value: u8) -> Image {
        Image {
            width,
            height,
            rgb: vec![value; width * height * 3],
        }
    }

    // ---- input files ---------------------------------------------------

    #[test]
    fn parse_input_skips_comments_and_blank_lines() {
        assert_eq!(parsed("# note\n\n  # indented\n"), Case::default());
    }

    #[test]
    fn parse_input_text_keeps_the_whole_line() {
        let case = parsed("text let $x_i^2$ be\n");

        assert_eq!(case.steps, [Step::Type(vec!["let $x_i^2$ be".to_owned()])]);
    }

    #[test]
    fn parse_input_rejects_text_that_looks_like_an_option() {
        assert!(parse_input("text -oops\n").is_err());
    }

    #[test]
    fn parse_input_chord_releases_in_reverse() {
        let case = parsed("chord ctrl shift z\n");

        let expected = [
            "-M", "ctrl", "-M", "shift", "-k", "z", "-m", "shift", "-m", "ctrl",
        ]
        .map(str::to_owned)
        .to_vec();
        assert_eq!(case.steps, [Step::Type(expected)]);
    }

    #[test]
    fn parse_input_hold_presses_waits_and_releases() {
        let case = parsed("hold r 900\n");

        let expected = ["-P", "r", "-s", "900", "-p", "r"]
            .map(str::to_owned)
            .to_vec();
        assert_eq!(case.steps, [Step::Type(expected)]);
    }

    #[test]
    fn parse_input_drag_moves_presses_interpolates_and_releases() {
        let case = parsed("drag 0 0 40 80 2\n");

        assert_eq!(
            case.steps,
            [
                move_to(0, 0),
                pointer(&["press", "button1"]),
                move_to(20, 40),
                move_to(40, 80),
                pointer(&["release", "button1"]),
            ]
        );
    }

    #[test]
    fn parse_input_click_is_a_move_and_a_press_and_release() {
        let case = parsed("click 5 6\n");

        assert_eq!(
            case.steps,
            [
                move_to(5, 6),
                pointer(&["press", "button1"]),
                pointer(&["release", "button1"]),
            ]
        );
    }

    #[test]
    fn parse_input_reads_case_options() {
        let case = parsed("scene strokes-5000\nsections editor\nscreenshot 10 2\nsigint\n");

        assert_eq!(case.scene.as_deref(), Some("strokes-5000"));
        assert_eq!(case.sections, ["editor"]);
        assert_eq!(
            case.screenshot,
            Some(Tolerance {
                pixels: 10,
                channel: 2
            })
        );
        assert_eq!(case.ending, Ending::Sigint);
    }

    #[test]
    fn parse_input_reads_the_export_option() {
        assert!(parsed("export\n").export);
        assert!(!parsed("key a\n").export);
        assert!(parse_input("export now\n").is_err());
    }

    #[test]
    fn find_export_needs_exactly_one_file() {
        // Arrange
        let home = temp_dir("find-export");
        let write = |name: &str| {
            std::fs::write(home.join(name), b"").unwrap_or_else(|err| panic!("{err}"));
        };

        // Act / Assert
        assert!(find_export(&home).is_err(), "no file yet");
        write("draw-1.png");
        write("draw.log");
        assert_eq!(find_export(&home), Ok(home.join("draw-1.png")));
        write("draw-2.png");
        assert!(find_export(&home).is_err(), "two files");
    }

    #[test]
    fn decode_png_gives_the_rgb_rows_in_order() {
        // Arrange: a 1×2 image, red over blue.
        let rgba = [255, 0, 0, 255, 0, 0, 255, 255];
        let bytes =
            draw::shell::export::encode_png(1, 2, &rgba).unwrap_or_else(|err| panic!("{err}"));

        // Act
        let image = decode_png(&bytes);

        // Assert
        assert_eq!(
            image,
            Ok(Image {
                width: 1,
                height: 2,
                rgb: vec![255, 0, 0, 0, 0, 255],
            })
        );
    }

    #[test]
    fn decode_png_rejects_garbage() {
        assert!(decode_png(b"not a png").is_err());
    }

    #[test]
    fn parse_input_reports_the_bad_line() {
        let err = parse_input("key a\nfrobnicate 1\n").failure();

        assert!(
            err.contains("line 2") && err.contains("frobnicate"),
            "{err}"
        );
    }

    #[test]
    fn parse_input_end_keyboard_is_one_step() {
        assert_eq!(parsed("end-keyboard\n").steps, [Step::EndKeyboard]);
        assert!(parse_input("end-keyboard now\n").is_err());
    }

    #[test]
    fn parse_input_clipboard_is_one_step() {
        assert_eq!(
            parsed("clipboard pasted from outside\n").steps,
            [Step::Clipboard("pasted from outside".to_owned())]
        );
    }

    #[test]
    fn parse_input_rejects_clipboard_text_that_looks_like_an_option() {
        assert!(parse_input("clipboard -oops\n").is_err());
        assert!(parse_input("clipboard\n").is_err());
    }

    #[test]
    fn select_sections_keeps_only_the_named_ones_and_the_header() {
        let snapshot = "draw-snapshot 1\n[editor]\ntool pen\n[document]\nshape 0\n";

        assert_eq!(
            select_sections(snapshot, &["editor".to_owned()]),
            "draw-snapshot 1\n[editor]\ntool pen\n"
        );
        assert_eq!(select_sections(snapshot, &[]), snapshot);
    }

    // ---- AC-3 state and pixels ------------------------------------------

    #[test]
    fn compare_snapshot_equal_matches() {
        let path = temp_dir("same").join("expected.snap");
        std::fs::write(&path, "tool pen\n").unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(compare_snapshot("tool pen\n", &path, false), Ok(()));
    }

    #[test]
    fn compare_snapshot_wrong_golden_prints_diff() {
        // Arrange
        let path = temp_dir("wrong").join("expected.snap");
        std::fs::write(&path, "tool pen\n").unwrap_or_else(|err| panic!("{err}"));

        // Act
        let err = compare_snapshot("tool rect\n", &path, false).failure();

        // Assert
        assert!(
            err.contains("tool pen") && err.contains("tool rect"),
            "{err}"
        );
    }

    #[test]
    fn compare_snapshot_missing_golden_says_how_to_create_it() {
        let path = temp_dir("missing").join("expected.snap");

        let err = compare_snapshot("x\n", &path, false).failure();

        assert!(err.contains("DRAW_UPDATE_SNAPSHOTS=1"), "{err}");
    }

    #[test]
    fn compare_snapshot_update_writes_the_golden() {
        let path = temp_dir("update").join("expected.snap");

        assert_eq!(compare_snapshot("new\n", &path, true), Ok(()));

        assert_eq!(
            std::fs::read_to_string(&path).ok().as_deref(),
            Some("new\n")
        );
    }

    #[test]
    fn image_parse_ppm_round_trips() {
        let image = solid(3, 2, 7);

        assert_eq!(Image::parse_ppm(&image.to_ppm()), Ok(image));
    }

    #[test]
    fn image_parse_ppm_rejects_bad_files() {
        assert!(Image::parse_ppm(b"P5\n1 1\n255\nxxx").is_err());
        assert!(Image::parse_ppm(b"P6\n1 1\n255\nxx").is_err());
        assert!(Image::parse_ppm(b"P6\n1 1\n65535\nxxxxxx").is_err());
        assert!(Image::parse_ppm(b"").is_err());
    }

    #[test]
    fn compare_image_within_tolerance_matches() {
        // Arrange: one pixel off by a lot, the rest off by a little.
        let expected = solid(4, 4, 100);
        let mut actual = solid(4, 4, 104);
        actual.rgb[0] = 200;

        // Act
        let result = compare_images(
            &actual,
            &expected,
            Tolerance {
                pixels: 1,
                channel: 8,
            },
        );

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn compare_image_wrong_golden_reports_and_draws_diff() {
        // Arrange
        let expected = solid(2, 1, 100);
        let mut actual = solid(2, 1, 100);
        actual.rgb[3] = 200;

        // Act
        let Err(diff) = compare_images(
            &actual,
            &expected,
            Tolerance {
                pixels: 0,
                channel: 8,
            },
        ) else {
            panic!("the images differ");
        };

        // Assert
        assert!(diff.message.contains("1 pixels"), "{}", diff.message);
        assert_eq!(diff.image.rgb, [33, 33, 33, 255, 0, 0]);
    }

    #[test]
    fn compare_image_other_size_is_a_mismatch() {
        let err = compare_images(&solid(2, 2, 0), &solid(3, 2, 0), Tolerance::default());

        assert!(err.is_err_and(|diff| diff.message.contains("2x2")));
    }

    // ---- T80 cursor box -------------------------------------------------

    /// A solid image with one pixel set to white.
    fn with_white_pixel(width: usize, height: usize, at: (usize, usize)) -> Image {
        let mut image = solid(width, height, 100);
        let offset = (at.1 * width + at.0) * 3;
        image.rgb[offset..offset + 3].fill(255);
        image
    }

    const NO_ROOM: Tolerance = Tolerance {
        pixels: 0,
        channel: 8,
    };

    #[test]
    fn cursor_box_hides_a_difference_at_the_pointer() {
        // Arrange: the corners of the box, relative to the pointer.
        let expected = solid(100, 100, 100);
        for at in [(48, 48), (73, 73), (50, 50)] {
            let mut actual = with_white_pixel(100, 100, at);

            // Act
            hide_cursor_box(&mut actual, &expected, (50, 50));

            // Assert
            assert_eq!(
                compare_images(&actual, &expected, NO_ROOM),
                Ok(()),
                "{at:?}"
            );
        }
    }

    #[test]
    fn cursor_box_keeps_a_difference_outside_it() {
        // Arrange: one pixel past each side of the box.
        let expected = solid(100, 100, 100);
        for at in [(47, 50), (50, 47), (74, 50), (50, 74)] {
            let mut actual = with_white_pixel(100, 100, at);

            // Act
            hide_cursor_box(&mut actual, &expected, (50, 50));

            // Assert
            assert!(
                compare_images(&actual, &expected, NO_ROOM).is_err(),
                "{at:?}"
            );
        }
    }

    #[test]
    fn cursor_box_at_the_image_edge_is_clipped() {
        // Arrange
        let expected = solid(10, 10, 100);
        for pointer in [(9, 9), (0, 0), (-5, -5), (500, 500)] {
            let mut actual = solid(10, 10, 100);

            // Act
            hide_cursor_box(&mut actual, &expected, pointer);

            // Assert
            assert_eq!(actual, expected, "{pointer:?}");
        }
    }

    #[test]
    fn cursor_box_ignores_a_golden_of_another_size() {
        // Arrange
        let mut actual = with_white_pixel(4, 4, (1, 1));
        let before = actual.clone();

        // Act
        hide_cursor_box(&mut actual, &solid(2, 2, 100), (1, 1));

        // Assert
        assert_eq!(actual, before);
    }

    #[test]
    fn last_pointer_is_the_last_set_step() {
        // Arrange
        let mut steps = drag((10, 20), (440, 350), 4);
        steps.push(pointer(&["press", "button1"]));
        steps.push(Step::Wheel(1));

        // Act
        let last = last_pointer(&steps, (1, 2));

        // Assert
        assert_eq!(last, (440, 350));
    }

    #[test]
    fn last_pointer_without_moves_is_the_start() {
        assert_eq!(last_pointer(&[Step::Wheel(1)], (1, 2)), (1, 2));
    }

    // ---- AC-4 no sleeps -------------------------------------------------

    fn sleeper() -> Child {
        Command::new("sleep")
            .arg("30")
            .spawn()
            .unwrap_or_else(|err| panic!("sleep: {err}"))
    }

    #[test]
    fn wait_for_sync_hung_app_times_out_with_log() {
        // Arrange: a process that never signals.
        let dir = temp_dir("hung");
        let log = dir.join("draw.log");
        std::fs::write(&log, "the last thing the app said\n").unwrap_or_else(|err| panic!("{err}"));
        let mut child = sleeper();

        // Act
        let result = wait_for_sync(
            &dir.join("idle"),
            1,
            Duration::from_millis(50),
            &mut child,
            &log,
        );
        let _ = child.kill();
        let _ = child.wait();

        // Assert
        let err = result.failure();
        assert!(err.contains("timed out"), "{err}");
        assert!(err.contains("the last thing the app said"), "{err}");
    }

    #[test]
    fn wait_for_sync_returns_once_the_count_is_reached() {
        // Arrange
        let dir = temp_dir("reached");
        let idle = dir.join("idle");
        std::fs::write(&idle, "3\n").unwrap_or_else(|err| panic!("{err}"));
        let mut child = sleeper();

        // Act
        let result = wait_for_sync(&idle, 2, SYNC_TIMEOUT, &mut child, &dir.join("log"));
        let _ = child.kill();
        let _ = child.wait();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn wait_for_sync_dead_app_fails_at_once() {
        // Arrange
        let dir = temp_dir("dead");
        let mut child = Command::new("true")
            .spawn()
            .unwrap_or_else(|err| panic!("true: {err}"));
        let started = Instant::now();

        // Act
        let result = wait_for_sync(
            &dir.join("idle"),
            1,
            SYNC_TIMEOUT,
            &mut child,
            &dir.join("log"),
        );
        let _ = child.wait();

        // Assert
        assert!(result.failure().contains("exited"));
        assert!(started.elapsed() < SYNC_TIMEOUT);
    }

    // ---- AC-5 clean exit ------------------------------------------------

    fn exit_of(script: &str) -> ExitStatus {
        Command::new("sh")
            .args(["-c", script])
            .status()
            .unwrap_or_else(|err| panic!("sh: {err}"))
    }

    #[test]
    fn check_exit_status_close_needs_code_zero() {
        assert_eq!(check_exit_status(exit_of("exit 0"), Ending::Close), Ok(()));
        assert!(check_exit_status(exit_of("exit 3"), Ending::Close).is_err());
        assert!(check_exit_status(exit_of("kill -INT $$"), Ending::Close).is_err());
    }

    #[test]
    fn check_exit_status_sigint_needs_the_signal() {
        assert_eq!(
            check_exit_status(exit_of("kill -INT $$"), Ending::Sigint),
            Ok(())
        );
        assert!(check_exit_status(exit_of("exit 0"), Ending::Sigint).is_err());
    }

    #[test]
    fn check_exit_log_flags_warn_and_error_lines() {
        let clean = "t +0.0s INFO start\nt +1.0s INFO exit: window closed\n";
        let warned = "t +0.0s INFO start\nt +1.0s WARN anomaly: doubled key\n";
        let failed = "t +0.0s ERROR panic: boom\n";

        assert_eq!(check_exit_log(clean), Ok(()));
        assert!(check_exit_log(warned).is_err_and(|e| e.contains("doubled key")));
        assert!(check_exit_log(failed).is_err_and(|e| e.contains("boom")));
    }

    #[test]
    fn check_exit_crash_flags_a_crash_report() {
        let dir = temp_dir("crash");
        assert_eq!(check_exit_crash(&dir), Ok(()));

        std::fs::write(dir.join("crash.log"), "boom").unwrap_or_else(|err| panic!("{err}"));

        assert!(check_exit_crash(&dir).is_err_and(|e| e.contains("boom")));
    }

    // ---- AC-8 never the desktop -----------------------------------------

    fn private_session(name: &str) -> (PathBuf, UnixListener, UnixListener) {
        let dir = temp_dir(name);
        let wayland =
            UnixListener::bind(dir.join("wayland-1")).unwrap_or_else(|err| panic!("{err}"));
        let sway =
            UnixListener::bind(dir.join("sway-ipc.sock")).unwrap_or_else(|err| panic!("{err}"));
        (dir, wayland, sway)
    }

    #[test]
    fn session_guard_accepts_the_private_sockets() {
        let (dir, _wayland, _sway) = private_session("accepts");

        let guard = session_guard(
            Some(dir.clone().into_os_string()),
            Some(dir.join("wayland-1").into_os_string()),
            Some(dir.join("sway-ipc.sock").into_os_string()),
        )
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(guard.runtime, dir);
    }

    #[test]
    fn session_guard_refuses_without_the_script() {
        let (dir, _wayland, _sway) = private_session("unset");
        let wayland = Some(dir.join("wayland-1").into_os_string());
        let sway = Some(dir.join("sway-ipc.sock").into_os_string());

        let err = session_guard(None, wayland, sway).failure();

        assert!(err.contains("scripts/e2e.sh"), "{err}");
    }

    #[test]
    fn session_guard_refuses_a_socket_outside_the_runtime() {
        // Arrange: the desktop's socket, somewhere else.
        let (dir, _wayland, _sway) = private_session("outside");
        let desktop = temp_dir("desktop");
        let _real =
            UnixListener::bind(desktop.join("wayland-0")).unwrap_or_else(|err| panic!("{err}"));

        // Act
        let err = session_guard(
            Some(dir.clone().into_os_string()),
            Some(desktop.join("wayland-0").into_os_string()),
            Some(dir.join("sway-ipc.sock").into_os_string()),
        )
        .failure();

        // Assert
        assert!(err.contains("outside"), "{err}");
    }

    #[test]
    fn session_guard_refuses_a_relative_display_name() {
        let (dir, _wayland, _sway) = private_session("relative");

        let err = session_guard(
            Some(dir.clone().into_os_string()),
            Some(OsString::from("wayland-0")),
            Some(dir.join("sway-ipc.sock").into_os_string()),
        )
        .failure();

        assert!(err.contains("outside"), "{err}");
    }

    #[test]
    fn session_guard_refuses_a_plain_file() {
        let (dir, _wayland, _sway) = private_session("plain");
        std::fs::write(dir.join("wayland-9"), "").unwrap_or_else(|err| panic!("{err}"));

        let err = session_guard(
            Some(dir.clone().into_os_string()),
            Some(dir.join("wayland-9").into_os_string()),
            Some(dir.join("sway-ipc.sock").into_os_string()),
        )
        .failure();

        assert!(err.contains("not a socket"), "{err}");
    }

    #[test]
    fn script_isolates_the_session() {
        // Arrange
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/e2e.sh");
        let script = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));

        // Act / Assert
        for needle in [
            "unset DISPLAY WAYLAND_DISPLAY",
            "export XDG_RUNTIME_DIR=$run",
            "WLR_BACKENDS=headless",
            "export WAYLAND_DISPLAY=$socket",
            "refusing to run",
            "MemoryMax=",
            "--backend",
        ] {
            assert!(script.contains(needle), "scripts/e2e.sh lacks {needle}");
        }
    }

    // ---- flags, not variables (T154) --------------------------------------

    #[test]
    fn e2e_command_passes_flags_not_variables() {
        // Arrange
        let files = AppFiles {
            dump: Path::new("/r/actual.snap"),
            idle: Path::new("/r/idle"),
            clipboard: Path::new("/r/clipboard"),
        };

        // Act
        let fresh = app_args(&files, Some(Path::new("/r/start.snap")), false);
        let recovered = app_args(&files, Some(Path::new("/r/start.snap")), true);
        let blank = app_args(&files, None, false);

        // Assert
        let text = |args: &[OsString]| {
            args.iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(" ")
        };
        assert_eq!(
            text(&fresh),
            "--log info --tablet off --e2e-dump /r/actual.snap --e2e-idle /r/idle \
             --e2e-clipboard /r/clipboard --snapshot /r/start.snap"
        );
        assert!(text(&recovered).ends_with("--recover"), "{recovered:?}");
        assert!(!text(&recovered).contains("--snapshot"), "{recovered:?}");
        assert!(!text(&blank).contains("--snapshot"), "{blank:?}");
        assert!(!text(&fresh).contains("DRAW_"), "{fresh:?}");
    }

    // ---- release gate (T129) ---------------------------------------------

    fn script_path(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("scripts")
            .join(name)
    }

    fn script_text(name: &str) -> String {
        let path = script_path(name);
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
    }

    /// Runs a script with `args` and extra `env`; returns success, stdout
    /// and stderr.
    fn run_script(name: &str, args: &[&str], env: &[(&str, &str)]) -> (bool, String, String) {
        let output = Command::new("bash")
            .arg(script_path(name))
            .args(args)
            .envs(env.iter().copied())
            .stdin(Stdio::null())
            .output()
            .unwrap_or_else(|err| panic!("bash {name}: {err}"));
        (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    fn cargo_version() -> String {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let text = std::fs::read_to_string(manifest).unwrap_or_default();
        text.lines()
            .find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"'))
            .unwrap_or("0.0.0")
            .to_owned()
    }

    #[test]
    fn script_rejects_an_unknown_backend() {
        let (ok, _, err) = run_script("e2e.sh", &["--backend", "bogus"], &[]);

        assert!(!ok);
        assert!(err.contains("unknown backend"), "{err}");
    }

    #[test]
    fn script_x11_run_is_private() {
        let script = script_text("e2e.sh");

        for needle in [
            "Xvfb",
            "-nolisten tcp",
            "-displayfd",
            "export XAUTHORITY=",
            "unset DISPLAY WAYLAND_DISPLAY",
            "refusing to run: DISPLAY",
        ] {
            assert!(script.contains(needle), "scripts/e2e.sh lacks {needle}");
        }
    }

    fn x11_runtime(name: &str) -> (PathBuf, OsString) {
        let dir = temp_dir(name);
        let authority = dir.join("Xauthority");
        std::fs::write(&authority, b"").unwrap_or_else(|err| panic!("{err}"));
        (dir, authority.into_os_string())
    }

    #[test]
    fn x11_guard_accepts_the_private_display() {
        let (dir, authority) = x11_runtime("x11-ok");

        let guard = x11_guard(
            Some(dir.clone().into_os_string()),
            Some(":97".into()),
            Some(authority),
        );

        assert_eq!(guard.map(|g| g.display), Ok(":97".to_owned()));
    }

    #[test]
    fn x11_guard_rejects_an_authority_outside_the_runtime() {
        let (dir, _) = x11_runtime("x11-outside");
        let other = temp_dir("x11-other").join("Xauthority");
        std::fs::write(&other, b"").unwrap_or_else(|err| panic!("{err}"));

        let err = x11_guard(
            Some(dir.into_os_string()),
            Some(":97".into()),
            Some(other.into_os_string()),
        )
        .err()
        .unwrap_or_default();

        assert!(err.contains("outside the private"), "{err}");
    }

    #[test]
    fn x11_guard_rejects_a_missing_display() {
        let (dir, authority) = x11_runtime("x11-nodisplay");

        let err = x11_guard(Some(dir.into_os_string()), None, Some(authority))
            .err()
            .unwrap_or_default();

        assert!(err.contains("DISPLAY is not set"), "{err}");
    }

    #[test]
    fn release_dry_run_lists_the_steps_in_order() {
        let version = cargo_version();

        let (ok, out, err) = run_script("release.sh", &["--dry-run", &version], &[]);

        assert!(ok, "{err}");
        let at = |step: &str| {
            out.find(&format!("==> {step}"))
                .unwrap_or_else(|| panic!("no step {step} in {out}"))
        };
        assert!(at("check") < at("e2e-wayland"));
        assert!(at("e2e-wayland") < at("e2e-x11"));
        assert!(at("e2e-x11") < at("package"));
    }

    #[test]
    fn release_stops_on_a_failing_step() {
        let version = cargo_version();

        let (ok, out, err) = run_script(
            "release.sh",
            &["--dry-run", &version],
            &[("DRAW_RELEASE_FAIL", "e2e-x11")],
        );

        assert!(!ok);
        assert!(err.contains("step e2e-x11 failed"), "{err}");
        assert!(out.contains("==> e2e-wayland"), "{out}");
        assert!(!out.contains("==> package"), "{out}");
    }

    #[test]
    fn release_rejects_a_malformed_version() {
        for bad in ["2.0", "v2.0.0", "2.0.0-rc1", ""] {
            let (ok, out, err) = run_script("release.sh", &["--dry-run", bad], &[]);

            assert!(!ok, "{bad}: {out}");
            assert!(err.contains("X.Y.Z"), "{bad}: {err}");
        }
    }

    #[test]
    fn release_rejects_a_version_that_is_not_the_manifest_one() {
        let (ok, _, err) = run_script("release.sh", &["--dry-run", "999.0.0"], &[]);

        assert!(!ok);
        assert!(err.contains("Cargo.toml"), "{err}");
    }

    #[test]
    fn package_script_says_it_is_not_the_release_path() {
        let script = script_text("package.sh");

        assert!(script.contains("not the release path"));
        assert!(script.contains("DRAW_RELEASE"));
        assert!(script_text("release.sh").contains("DRAW_RELEASE=1"));
    }

    #[test]
    fn quality_gates_lists_the_release_gate() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/process/Quality Gates.md");
        let text = std::fs::read_to_string(&path).unwrap_or_default();

        assert!(text.contains("scripts/release.sh"), "no release gate row");
        assert!(text.contains("check.sh --fuzz"));
    }

    #[test]
    fn golden_path_prefers_the_backend_file() {
        let dir = temp_dir("golden-prefers");
        std::fs::write(dir.join("expected.ppm"), b"w").unwrap_or_else(|err| panic!("{err}"));
        std::fs::write(dir.join("expected.x11.ppm"), b"x").unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            golden_path(&dir, "expected.ppm", Backend::X11),
            dir.join("expected.x11.ppm")
        );
        assert_eq!(
            golden_path(&dir, "expected.ppm", Backend::Wayland),
            dir.join("expected.ppm")
        );
    }

    #[test]
    fn golden_path_falls_back_to_the_shared_file() {
        let dir = temp_dir("golden-fallback");
        std::fs::write(dir.join("expected.snap"), b"w").unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            golden_path(&dir, "expected.snap", Backend::X11),
            dir.join("expected.snap")
        );
    }

    #[test]
    fn x11_intern_request_pads_the_name_to_four_bytes() {
        let request = x11_intern_request("WM_PROTOCOLS");

        // Opcode 16, only-if-exists 0, 2 + 12/4 words, name length.
        assert_eq!(request.get(..4), Some(&[16, 0, 5, 0][..]));
        assert_eq!(request.get(4..6), Some(&[12, 0][..]));
        assert_eq!(request.len(), 20);
        assert_eq!(x11_intern_request("abc").len(), 12);
    }

    #[test]
    fn x11_delete_request_is_a_client_message_to_the_window() {
        let request = x11_delete_request(0x0020_0001, 300, 301);

        // SendEvent: opcode 25, 11 words, destination, empty mask.
        assert_eq!(request.len(), 44);
        assert_eq!(request.get(..4), Some(&[25, 0, 11, 0][..]));
        assert_eq!(request.get(4..8), Some(&0x0020_0001u32.to_le_bytes()[..]));
        assert_eq!(request.get(8..12), Some(&[0, 0, 0, 0][..]));
        // The event: ClientMessage (33), format 32, window, type, data.
        assert_eq!(request.get(12..14), Some(&[33, 32][..]));
        assert_eq!(request.get(16..20), Some(&0x0020_0001u32.to_le_bytes()[..]));
        assert_eq!(request.get(20..24), Some(&300u32.to_le_bytes()[..]));
        assert_eq!(request.get(24..28), Some(&301u32.to_le_bytes()[..]));
    }

    #[test]
    fn xdotool_commands_translate_a_chord_and_text() {
        let chord = chord(&["ctrl", "shift", "9"]).unwrap_or_default();

        let commands = xdotool_commands(&chord).unwrap_or_default();

        assert_eq!(
            commands,
            [[
                "keydown",
                "Control_L",
                "keydown",
                "Shift_L",
                "key",
                "9",
                "keyup",
                "Shift_L",
                "keyup",
                "Control_L"
            ]
            .map(str::to_owned)
            .to_vec()]
        );
        let text = xdotool_commands(&["a b".to_owned()]).unwrap_or_default();
        assert_eq!(
            text.first().and_then(|c| c.last()).map(String::as_str),
            Some("a b")
        );
    }

    #[test]
    fn backend_from_name_knows_both_and_rejects_others() {
        assert_eq!(Backend::from_name("x11"), Ok(Backend::X11));
        assert_eq!(Backend::from_name("wayland"), Ok(Backend::Wayland));
        assert!(Backend::from_name("mir").is_err());
    }

    // ---- wire format ----------------------------------------------------

    #[test]
    fn wire_string_pads_to_four_bytes() {
        let bytes = wire_string("abc");

        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes.get(4..).unwrap_or(&[]), b"abc\0");
        assert_eq!(wire_string("abcd").len(), 12);
    }

    #[test]
    fn parse_input_wheel_is_one_notch() {
        assert_eq!(parsed("wheel up\n").steps, [Step::Wheel(-1)]);
        assert_eq!(parsed("wheel down\n").steps, [Step::Wheel(1)]);
    }

    #[test]
    fn wheel_message_is_source_discrete_and_frame() {
        // Act
        let message = wheel_message(1);

        // Assert: three requests on the virtual pointer.
        let word = |i: usize| {
            u32::from_ne_bytes(
                message
                    .get(i..i + 4)
                    .and_then(|b| b.try_into().ok())
                    .unwrap_or([0; 4]),
            )
        };
        assert_eq!(message.len(), 12 + 24 + 8);
        assert_eq!((word(0), word(4) & 0xffff), (VIRTUAL_POINTER_ID, 5));
        assert_eq!((word(12), word(16) & 0xffff), (VIRTUAL_POINTER_ID, 7));
        assert_eq!(word(28), 15 << 8);
        assert_eq!(word(32), 1);
        assert_eq!((word(36), word(40) & 0xffff), (VIRTUAL_POINTER_ID, 4));
    }

    #[test]
    fn scan_registry_finds_the_global_and_the_sync() {
        // Arrange: wl_registry.global(7, "wanted", 1), then callback 3.
        let mut body = 7u32.to_ne_bytes().to_vec();
        body.extend(wire_string("wanted"));
        body.extend(1u32.to_ne_bytes());
        let mut stream = wire(2, 0, &body);
        let partial = stream.clone();
        stream.extend(wire(3, 0, &0u32.to_ne_bytes()));

        // Act / Assert
        assert_eq!(scan_registry(&partial, "wanted"), (Some(7), false));
        assert_eq!(scan_registry(&stream, "wanted"), (Some(7), true));
        assert_eq!(scan_registry(&stream, "other"), (None, true));
    }

    #[test]
    fn every_committed_case_parses() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/e2e");

        let cases = case_directories(&root, None).unwrap_or_else(|err| panic!("{err}"));

        assert!(cases.len() >= 10, "{cases:?}");
        for case in cases {
            let text = std::fs::read_to_string(case.join("input"))
                .unwrap_or_else(|err| panic!("{}: {err}", case.display()));
            parse_input(&text).unwrap_or_else(|err| panic!("{}: {err}", case.display()));
        }
    }
}
