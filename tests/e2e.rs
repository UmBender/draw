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
    /// Output size to set, waited for in the snapshot.
    Output(u32, u32),
    /// Compare the state with `<name>.snap` here.
    Checkpoint(String),
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
    /// Snapshot sections to compare; empty means the whole snapshot.
    sections: Vec<String>,
    /// Whether to compare a screenshot, and how strictly.
    screenshot: Option<Tolerance>,
    /// The input.
    steps: Vec<Step>,
    /// How the case ends.
    ending: Ending,
}

fn number<T: std::str::FromStr>(word: &str) -> Result<T, String> {
    word.parse().map_err(|_| format!("`{word}` is not a number"))
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
        // wtype would read a leading dash as an option.
        ("text", _) if !rest.is_empty() && !rest.starts_with('-') => {
            case.steps.push(Step::Type(vec![rest.to_owned()]));
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
        ("press", []) | ("press", [_]) => {
            case.steps
                .push(pointer(&["press", button(args.first().copied())?]));
        }
        ("release", []) | ("release", [_]) => {
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
        ("drag", [x1, y1, x2, y2]) | ("drag", [x1, y1, x2, y2, _]) => {
            let segments = args.get(4).map_or(Ok(DRAG_SEGMENTS), |s| number(s))?;
            case.steps.extend(drag(
                (number(x1)?, number(y1)?),
                (number(x2)?, number(y2)?),
                segments,
            ));
        }
        ("wheel", ["up"]) => case.steps.push(pointer(&["press", "button4"])),
        ("wheel", ["down"]) => case.steps.push(pointer(&["press", "button5"])),
        ("resize", [w, h]) => case.steps.push(Step::Output(number(w)?, number(h)?)),
        ("checkpoint", [name]) => case.steps.push(Step::Checkpoint((*name).to_owned())),
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
/// | `sections <name>...` | compare only these snapshot sections |
/// | `screenshot [pixels channel]` | also compare `expected.ppm` |
/// | `text <text>` | type literally, through `wtype` |
/// | `key <keysym>` | one key |
/// | `chord <mod>... <keysym>` | key with modifiers |
/// | `hold <keysym> <ms>` | hold a key down |
/// | `move x y`, `press [button]`, `release [button]` | pointer |
/// | `click x y`, `drag x1 y1 x2 y2 [segments]` | left button |
/// | `wheel up/down` | one wheel notch |
/// | `resize w h` | resize the output, wait for the window |
/// | `checkpoint <name>` | compare with `<name>.snap` |
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
        Golden::Mismatch(diff) => Err(format!(
            "state differs from {}\n{diff}",
            path.display()
        )),
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
    for (a, e) in actual.rgb.chunks_exact(3).zip(expected.rgb.chunks_exact(3)) {
        if a.iter().zip(e).any(|(x, y)| x.abs_diff(*y) > tolerance.channel) {
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
        if object == 2 && header & 0xffff == 0 {
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

/// Creates a virtual pointer and keeps it while the stream is open. It
/// sends no events: it only gives the seat a pointer (ADR-T75-1).
fn announce_pointer(socket: &Path) -> Result<UnixStream, String> {
    const INTERFACE: &str = "zwlr_virtual_pointer_manager_v1";
    let fail = |err: std::io::Error| format!("virtual pointer: {err}");
    let mut stream = UnixStream::connect(socket).map_err(fail)?;
    stream
        .set_read_timeout(Some(SYNC_TIMEOUT))
        .map_err(fail)?;
    // wl_display.get_registry(2), wl_display.sync(3).
    let mut hello = wire(1, 1, &2u32.to_ne_bytes());
    hello.extend(wire(1, 0, &3u32.to_ne_bytes()));
    stream.write_all(&hello).map_err(fail)?;
    let mut received = Vec::new();
    let mut chunk = [0u8; 4096];
    let name = loop {
        let n = stream.read(&mut chunk).map_err(fail)?;
        if n == 0 {
            return Err("virtual pointer: the compositor hung up".to_owned());
        }
        received.extend_from_slice(chunk.get(..n).unwrap_or(&[]));
        let (found, done) = scan_registry(&received, INTERFACE);
        if done {
            break found.ok_or("virtual pointer: the compositor has no manager")?;
        }
    };
    // wl_registry.bind(name, interface, 1, id 4).
    let mut bind = name.to_ne_bytes().to_vec();
    bind.extend(wire_string(INTERFACE));
    bind.extend(1u32.to_ne_bytes());
    bind.extend(4u32.to_ne_bytes());
    // create_virtual_pointer(no seat, id 5).
    let mut create = 0u32.to_ne_bytes().to_vec();
    create.extend(5u32.to_ne_bytes());
    let mut message = wire(2, 0, &bind);
    message.extend(wire(4, 0, &create));
    stream.write_all(&message).map_err(fail)?;
    Ok(stream)
}

/// The running private session: tools get its sockets and nothing else.
struct Session {
    guard: Guard,
    /// Holds the seat's pointer.
    _pointer: UnixStream,
    /// `wtype` processes still alive, oldest first.
    keyboards: VecDeque<Child>,
}

impl Session {
    /// Joins the session named by the environment, after checking it.
    fn join() -> Result<Self, String> {
        let guard = session_guard(
            std::env::var_os("DRAW_E2E_RUNTIME"),
            std::env::var_os("WAYLAND_DISPLAY"),
            std::env::var_os("SWAYSOCK"),
        )?;
        let pointer = announce_pointer(&guard.wayland)?;
        let mut session = Self {
            guard,
            _pointer: pointer,
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
            .env("WAYLAND_DISPLAY", &self.guard.wayland)
            .env("SWAYSOCK", &self.guard.sway)
            .stdin(Stdio::null());
        command
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
    fn wait_for_window(&self, child: &mut Child) -> Result<(), String> {
        let started = Instant::now();
        loop {
            if self
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
        scenes::drag(
            &mut editor,
            scenes::v(x, y),
            scenes::v(x + 9.0, y + 9.0),
        );
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
    /// Syncs sent so far.
    sent: u64,
}

impl Running<'_> {
    /// Types `args` and then the sync chord, and waits for the app.
    fn sync_with(&mut self, args: &[String]) -> Result<(), String> {
        let mut all = args.to_vec();
        all.extend(SYNC_ARGS.iter().map(|&a| a.to_owned()));
        all.extend(["-s".to_owned(), KEEP_ALIVE_MS.to_owned()]);
        let keyboard = self.session.spawn_wtype(&all)?;
        self.session.keyboards.push_back(keyboard);
        self.sent += 1;
        wait_for_sync(&self.idle, self.sent, SYNC_TIMEOUT, &mut self.child, &self.log)?;
        self.session.retire_old_keyboards();
        Ok(())
    }

    fn snapshot(&self) -> Result<String, String> {
        std::fs::read_to_string(&self.dump)
            .map_err(|err| format!("{}: {err}", self.dump.display()))
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
                let mut command = vec!["seat", "-", "cursor"];
                command.extend(args.iter().map(String::as_str));
                self.session.swaymsg(&command)?;
                self.sync_with(&[])
            }
            Step::Output(w, h) => {
                let mode = format!("{w}x{h}");
                self.session
                    .swaymsg(&["output", OUTPUT_NAME, "resolution", &mode])?;
                self.wait_for_viewport((*w, *h))
            }
            Step::Checkpoint(name) => {
                let actual = select_sections(&self.snapshot()?, sections);
                compare_snapshot(&actual, &source.join(format!("{name}.snap")), update_requested())
            }
        }
    }
}

impl Drop for Running<'_> {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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
    let log = state.join("draw").join("draw.log");
    let mode = format!("{}x{}", OUTPUT.0, OUTPUT.1);
    session.swaymsg(&["output", OUTPUT_NAME, "resolution", &mode])?;
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
        .stdout(stdout)
        .stderr(stderr);
    if let Some(start) = start {
        command.env("DRAW_SNAPSHOT", start);
    }
    let mut child = command.spawn().map_err(|err| format!("draw: {err}"))?;
    session.wait_for_window(&mut child)?;
    let mut running = Running {
        session,
        child,
        idle,
        dump,
        log: log.clone(),
        sent: 0,
    };
    running.wait_for_viewport(OUTPUT)?;
    for step in &case.steps {
        running.run_step(step, source, &case.sections)?;
    }

    let update = update_requested();
    let mut problems = Vec::new();
    let actual = select_sections(&running.snapshot()?, &case.sections);
    if let Err(problem) = compare_snapshot(&actual, &source.join("expected.snap"), update) {
        problems.push(problem);
    }
    if let Some(tolerance) = case.screenshot {
        let shot = run.join("actual.ppm");
        let status = running
            .session
            .tool("grim")
            .args(["-t", "ppm"])
            .arg(&shot)
            .status()
            .map_err(|err| format!("grim: {err}"))?;
        if !status.success() {
            return Err(format!("grim failed: {status}"));
        }
        if let Err(problem) = compare_screenshot(&shot, &source.join("expected.ppm"), run, tolerance, update) {
            problems.push(problem);
        }
    }

    match case.ending {
        Ending::Close => {
            running.session.swaymsg(&["[app_id=\"draw\"]", "kill"])?;
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

/// Compares the screenshot at `shot`; writes `diff.ppm` into `run` on a
/// mismatch, or the golden when `update`.
fn compare_screenshot(
    shot: &Path,
    golden: &Path,
    run: &Path,
    tolerance: Tolerance,
    update: bool,
) -> Result<(), String> {
    let bytes = std::fs::read(shot).map_err(|err| format!("{}: {err}", shot.display()))?;
    if update {
        return std::fs::write(golden, &bytes)
            .map_err(|err| format!("cannot write {}: {err}", golden.display()));
    }
    let actual = Image::parse_ppm(&bytes)?;
    let expected = std::fs::read(golden)
        .map_err(|err| format!("no golden at {}: {err}", golden.display()))?;
    let expected = Image::parse_ppm(&expected)?;
    compare_images(&actual, &expected, tolerance).map_err(|diff| {
        let path = run.join("diff.ppm");
        let saved = std::fs::write(&path, diff.image.to_ppm());
        format!("{} (diff: {} {saved:?})", diff.message, path.display())
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
            only.is_none_or(|name| path.file_name().is_some_and(|n| n == std::ffi::OsStr::new(name)))
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

        let expected = ["-M", "ctrl", "-M", "shift", "-k", "z", "-m", "shift", "-m", "ctrl"]
            .map(str::to_owned)
            .to_vec();
        assert_eq!(case.steps, [Step::Type(expected)]);
    }

    #[test]
    fn parse_input_hold_presses_waits_and_releases() {
        let case = parsed("hold r 900\n");

        let expected = ["-P", "r", "-s", "900", "-p", "r"].map(str::to_owned).to_vec();
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
    fn parse_input_reports_the_bad_line() {
        let err = parse_input("key a\nfrobnicate 1\n").unwrap_err();

        assert!(err.contains("line 2") && err.contains("frobnicate"), "{err}");
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
        let err = compare_snapshot("tool rect\n", &path, false).unwrap_err();

        // Assert
        assert!(err.contains("tool pen") && err.contains("tool rect"), "{err}");
    }

    #[test]
    fn compare_snapshot_missing_golden_says_how_to_create_it() {
        let path = temp_dir("missing").join("expected.snap");

        let err = compare_snapshot("x\n", &path, false).unwrap_err();

        assert!(err.contains("DRAW_UPDATE_SNAPSHOTS=1"), "{err}");
    }

    #[test]
    fn compare_snapshot_update_writes_the_golden() {
        let path = temp_dir("update").join("expected.snap");

        assert_eq!(compare_snapshot("new\n", &path, true), Ok(()));

        assert_eq!(std::fs::read_to_string(&path).ok().as_deref(), Some("new\n"));
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
        let err = result.unwrap_err();
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
        let result = wait_for_sync(&dir.join("idle"), 1, SYNC_TIMEOUT, &mut child, &dir.join("log"));
        let _ = child.wait();

        // Assert
        assert!(result.unwrap_err().contains("exited"));
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
        let sway = UnixListener::bind(dir.join("sway-ipc.sock")).unwrap_or_else(|err| panic!("{err}"));
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

        let err = session_guard(None, wayland, sway).unwrap_err();

        assert!(err.contains("scripts/e2e.sh"), "{err}");
    }

    #[test]
    fn session_guard_refuses_a_socket_outside_the_runtime() {
        // Arrange: the desktop's socket, somewhere else.
        let (dir, _wayland, _sway) = private_session("outside");
        let desktop = temp_dir("desktop");
        let _real = UnixListener::bind(desktop.join("wayland-0")).unwrap_or_else(|err| panic!("{err}"));

        // Act
        let err = session_guard(
            Some(dir.clone().into_os_string()),
            Some(desktop.join("wayland-0").into_os_string()),
            Some(dir.join("sway-ipc.sock").into_os_string()),
        )
        .unwrap_err();

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
        .unwrap_err();

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
        .unwrap_err();

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
        ] {
            assert!(script.contains(needle), "scripts/e2e.sh lacks {needle}");
        }
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
