//! Autosave and crash recovery (T94, ADR-T94-1).
//!
//! The shell calls [`Autosaver::on_input`] once per frame that had input,
//! so nothing wakes the blocked loop (ADR-T12-1). When a save is due the
//! main thread only clones the editor state; a short-lived thread makes
//! the text and writes the file. `draw --recover` reads it back.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::{JoinHandle, ThreadId};
use std::time::{Duration, Instant};

use crate::core::editor::Editor;
use crate::core::snapshot::Snapshot;

/// Shortest time between two autosaves (AC-1).
pub const INTERVAL: Duration = Duration::from_secs(30);

/// The newest autosave, in the state directory.
pub const FILE: &str = "autosave.snap";

/// The autosave a session found when it first saved (AC-3).
pub const PREV_FILE: &str = "autosave.prev.snap";

/// Which autosave to recover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    /// [`FILE`].
    Latest,
    /// [`PREV_FILE`].
    Previous,
}

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    /// No `--recover`.
    Normal,
    /// `--recover` or `--recover prev`.
    Recover(Which),
    /// `--recover` with a value it does not know, and why.
    Invalid(String),
}

impl Launch {
    /// Reads the arguments after the program name. Arguments other than
    /// `--recover` are ignored, as they always were.
    #[must_use]
    pub fn from_args(args: impl IntoIterator<Item = OsString>) -> Self {
        let _ = args.into_iter();
        todo!()
    }
}

/// The snapshot a launch starts from and the reason it starts blank, for
/// the user and the log. [`Launch::Normal`] asks for neither.
#[must_use]
pub fn recover(launch: &Launch, dir: Option<&Path>) -> (Option<Snapshot>, Option<String>) {
    let _ = (launch, dir);
    todo!()
}

/// Decides when an autosave is due (AC-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// The last capture, or the session start.
    last: Instant,
    /// The document revision on disk; `None` after a failed write.
    saved: Option<u64>,
}

impl Policy {
    /// A session that starts at `now` with the document at `revision`.
    #[must_use]
    pub fn new(now: Instant, revision: u64) -> Self {
        let _ = (now, revision);
        todo!()
    }

    /// Whether an input at `now` saves: [`INTERVAL`] has passed, the
    /// document is not the saved one, and nothing is being edited.
    #[must_use]
    pub fn is_due(&self, now: Instant, revision: u64, editing: bool) -> bool {
        let _ = (now, revision, editing);
        todo!()
    }

    /// Records a capture of `revision` at `now`.
    pub fn captured(&mut self, now: Instant, revision: u64) {
        let _ = (now, revision);
        todo!()
    }

    /// Records that the last capture never reached the disk.
    pub fn failed(&mut self) {
        todo!()
    }
}

/// The temporary file of this process in `dir`; two windows cannot mix
/// their writes (ADR-T94-1).
fn temporary_path(dir: &Path) -> PathBuf {
    let _ = dir;
    todo!()
}

/// Writes `text` as [`FILE`] in `dir`, creating `dir`: a mode `0600`
/// temporary file, synced, then renamed. With `rotate` the old [`FILE`]
/// first becomes [`PREV_FILE`], once the temporary file is complete.
///
/// # Errors
///
/// The first file-system error; [`FILE`] is then unchanged, unless the
/// final rename failed after the rotation.
pub fn write_atomic(dir: &Path, text: &str, rotate: bool) -> std::io::Result<()> {
    let _ = (dir, text, rotate);
    todo!()
}

/// Runs the autosaves of one session.
#[derive(Debug)]
pub struct Autosaver {
    /// The state directory; without one the autosaver does nothing.
    dir: Option<PathBuf>,
    /// When to save.
    policy: Policy,
    /// Whether the first autosave has yet to succeed (AC-3).
    rotate: bool,
    /// The write in progress, with its failure line.
    writer: Option<JoinHandle<Result<(), String>>>,
}

impl Autosaver {
    /// An autosaver for a session whose editor starts as `editor` at
    /// `now`, writing to `dir`.
    #[must_use]
    pub fn new(dir: Option<PathBuf>, editor: &Editor, now: Instant) -> Self {
        let _ = (dir, editor, now);
        todo!()
    }

    /// Call once per frame that had input: saves `editor` if it is due.
    /// A failed write goes to the diagnostics log.
    pub fn on_input(&mut self, editor: &Editor, now: Instant) {
        let _ = (editor, now);
        todo!()
    }

    /// The thread of the write in progress, if any.
    #[must_use]
    pub fn writer_thread(&self) -> Option<ThreadId> {
        todo!()
    }

    /// Blocks until the write in progress ends. Its failure line, if any.
    pub fn wait(&mut self) -> Option<String> {
        let _ = fs::metadata("");
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::{Command, Tool};
    use crate::core::geom::Vec2;
    use crate::core::input::{InputEvent, Modifiers, PointerButton};
    use proptest::prelude::*;
    use std::ffi::OsString;
    use std::io;
    use std::os::unix::fs::PermissionsExt as _;
    use std::time::Instant;

    const SECOND: Duration = Duration::from_secs(1);

    fn temp_dir(name: &str) -> io::Result<PathBuf> {
        let dir = std::env::temp_dir().join(format!("draw-autosave-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn mode(path: &Path) -> io::Result<u32> {
        Ok(fs::metadata(path)?.permissions().mode() & 0o777)
    }

    fn entries(dir: &Path) -> io::Result<Vec<String>> {
        let mut names = fs::read_dir(dir)?
            .map(|entry| Ok(entry?.file_name().to_string_lossy().into_owned()))
            .collect::<io::Result<Vec<_>>>()?;
        names.sort();
        Ok(names)
    }

    fn pointer_down(ed: &mut Editor, pos: Vec2) {
        ed.handle(InputEvent::PointerMove {
            pos,
            mods: Modifiers::NONE,
        });
        ed.handle(InputEvent::PointerDown {
            pos,
            button: PointerButton::Left,
            mods: Modifiers::NONE,
        });
    }

    fn pointer_up(ed: &mut Editor, pos: Vec2) {
        ed.handle(InputEvent::PointerUp {
            pos,
            button: PointerButton::Left,
            mods: Modifiers::NONE,
        });
    }

    /// Draws one more line, which changes the document.
    fn draw_line(ed: &mut Editor) {
        ed.apply(Command::SetTool(Tool::Line));
        let y = ed.document().len() as f32 * 10.0 + 10.0;
        pointer_down(ed, Vec2::new(10.0, y));
        let end = Vec2::new(200.0, y + 5.0);
        ed.handle(InputEvent::PointerMove {
            pos: end,
            mods: Modifiers::NONE,
        });
        pointer_up(ed, end);
    }

    fn with_lines(count: usize) -> Editor {
        let mut ed = Editor::new();
        for _ in 0..count {
            draw_line(&mut ed);
        }
        assert_eq!(ed.document().len(), count);
        ed
    }

    fn load(dir: &Path, file: &str) -> io::Result<Editor> {
        let text = fs::read_to_string(dir.join(file))?;
        let snapshot = Snapshot::parse(&text).map_err(io::Error::other)?;
        Ok(Editor::from_snapshot(snapshot))
    }

    /// An autosaver in `dir`, past its first autosave of `ed`.
    fn after_first_save(dir: &Path, ed: &mut Editor, start: Instant) -> Autosaver {
        let mut saver = Autosaver::new(Some(dir.to_path_buf()), ed, start);
        draw_line(ed);
        saver.on_input(ed, start + INTERVAL);
        assert_eq!(saver.wait(), None);
        saver
    }

    fn launch(args: &[&str]) -> Launch {
        Launch::from_args(args.iter().map(OsString::from))
    }

    // ---- AC-1 when an autosave happens -------------------------------------

    #[test]
    fn interval_is_30_seconds() {
        assert_eq!(INTERVAL, Duration::from_secs(30));
    }

    #[test]
    fn policy_is_not_due_before_the_interval() {
        // Arrange
        let start = Instant::now();
        let policy = Policy::new(start, 0);

        // Act / Assert
        assert!(!policy.is_due(start, 1, false));
        assert!(!policy.is_due(start + INTERVAL - SECOND, 1, false));
    }

    #[test]
    fn policy_is_due_after_the_interval_with_a_change() {
        // Arrange
        let start = Instant::now();
        let policy = Policy::new(start, 0);

        // Act / Assert
        assert!(policy.is_due(start + INTERVAL, 1, false));
        assert!(policy.is_due(start + INTERVAL * 10, 7, false));
    }

    #[test]
    fn policy_counts_from_the_last_capture() {
        // Arrange
        let start = Instant::now();
        let mut policy = Policy::new(start, 0);

        // Act
        policy.captured(start + INTERVAL * 2, 1);

        // Assert
        assert!(!policy.is_due(start + INTERVAL * 3 - SECOND, 2, false));
        assert!(policy.is_due(start + INTERVAL * 3, 2, false));
    }

    #[test]
    fn policy_is_not_due_without_a_document_change() {
        // Arrange
        let start = Instant::now();
        let mut policy = Policy::new(start, 4);

        // Act / Assert
        assert!(!policy.is_due(start + INTERVAL, 4, false));
        policy.captured(start + INTERVAL, 9);
        assert!(!policy.is_due(start + INTERVAL * 2, 9, false));
    }

    #[test]
    fn policy_is_not_due_while_editing() {
        // Arrange
        let start = Instant::now();
        let policy = Policy::new(start, 0);

        // Act / Assert
        assert!(!policy.is_due(start + INTERVAL, 1, true));
    }

    #[test]
    fn autosaver_saves_on_input_after_the_interval() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("saves")?;
        let start = Instant::now();
        let mut ed = Editor::new();
        let mut saver = Autosaver::new(Some(dir.clone()), &ed, start);
        draw_line(&mut ed);

        // Act
        saver.on_input(&ed, start + INTERVAL - SECOND);
        let early = entries(&dir)?;
        saver.on_input(&ed, start + INTERVAL);
        let failure = saver.wait();

        // Assert
        assert_eq!(early, Vec::<String>::new());
        assert_eq!(failure, None);
        assert_eq!(load(&dir, FILE)?, ed);
        Ok(())
    }

    #[test]
    fn autosaver_skips_an_unchanged_document() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("unchanged")?;
        let start = Instant::now();
        let ed = with_lines(2);
        let mut saver = Autosaver::new(Some(dir.clone()), &ed, start);

        // Act
        saver.on_input(&ed, start + INTERVAL);
        let failure = saver.wait();

        // Assert
        assert_eq!(failure, None);
        assert_eq!(entries(&dir)?, Vec::<String>::new());
        Ok(())
    }

    #[test]
    fn autosaver_waits_for_an_open_gesture() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("gesture")?;
        let start = Instant::now();
        let mut ed = Editor::new();
        let mut saver = Autosaver::new(Some(dir.clone()), &ed, start);
        draw_line(&mut ed);
        let at = Vec2::new(300.0, 300.0);
        pointer_down(&mut ed, at);
        assert!(ed.active_gesture().is_some());

        // Act
        saver.on_input(&ed, start + INTERVAL);
        let during = entries(&dir)?;
        pointer_up(&mut ed, at);
        saver.on_input(&ed, start + INTERVAL + SECOND);
        let failure = saver.wait();

        // Assert
        assert_eq!(during, Vec::<String>::new());
        assert_eq!(failure, None);
        assert_eq!(load(&dir, FILE)?, ed);
        Ok(())
    }

    #[test]
    fn autosaver_waits_for_an_open_text_edit() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("text")?;
        let start = Instant::now();
        let mut ed = Editor::new();
        let mut saver = Autosaver::new(Some(dir.clone()), &ed, start);
        draw_line(&mut ed);
        ed.apply(Command::SetTool(Tool::Text));
        let at = Vec2::new(300.0, 300.0);
        pointer_down(&mut ed, at);
        pointer_up(&mut ed, at);
        assert!(ed.text_edit().is_some());

        // Act
        saver.on_input(&ed, start + INTERVAL);

        // Assert
        assert_eq!(saver.wait(), None);
        assert_eq!(entries(&dir)?, Vec::<String>::new());
        Ok(())
    }

    #[test]
    fn autosave_uses_no_timer() {
        // Arrange: only the code above the tests.
        let source = include_str!("autosave.rs");
        let code = source.split("#[cfg(test)]").next().unwrap_or(source);

        // Act / Assert: nothing wakes the loop or reads the clock; the
        // shell hands in the time of an input frame.
        for timer in ["schedule_update", "sleep", "Instant::now"] {
            assert!(!code.contains(timer), "autosave uses {timer}");
        }
        assert!(include_str!("app.rs").contains("if had_input"));
    }

    // ---- AC-2 how the file is written --------------------------------------

    #[test]
    fn write_atomic_leaves_only_the_file() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("only")?;

        // Act
        write_atomic(&dir, "text", false)?;

        // Assert
        assert_eq!(entries(&dir)?, [FILE]);
        assert_eq!(fs::read_to_string(dir.join(FILE))?, "text");
        Ok(())
    }

    #[test]
    fn write_atomic_creates_the_directory() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("mkdir")?.join("state/draw");

        // Act
        write_atomic(&dir, "text", false)?;

        // Assert
        assert_eq!(fs::read_to_string(dir.join(FILE))?, "text");
        Ok(())
    }

    #[test]
    fn write_atomic_replaces_the_old_contents() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("replace")?;
        write_atomic(&dir, "a much longer first version", false)?;

        // Act
        write_atomic(&dir, "second", false)?;

        // Assert
        assert_eq!(fs::read_to_string(dir.join(FILE))?, "second");
        Ok(())
    }

    #[test]
    fn write_atomic_sets_mode_0600() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("mode")?;

        // Act
        write_atomic(&dir, "text", false)?;

        // Assert
        assert_eq!(mode(&dir.join(FILE))?, 0o600);
        Ok(())
    }

    #[test]
    fn write_atomic_tightens_a_stale_temporary_file() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("stale")?;
        let temporary = temporary_path(&dir);
        fs::write(&temporary, "stale and longer than the new text")?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o644))?;

        // Act
        write_atomic(&dir, "text", false)?;

        // Assert
        assert_eq!(mode(&dir.join(FILE))?, 0o600);
        assert_eq!(fs::read_to_string(dir.join(FILE))?, "text");
        Ok(())
    }

    #[test]
    fn temporary_file_is_named_per_process() {
        // Arrange / Act
        let path = temporary_path(Path::new("/state"));

        // Assert
        let expected = format!("/state/autosave.snap.tmp.{}", std::process::id());
        assert_eq!(path, Path::new(&expected));
    }

    #[test]
    fn autosaver_writes_on_another_thread() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("thread")?;
        let start = Instant::now();
        let mut ed = Editor::new();
        let mut saver = Autosaver::new(Some(dir.clone()), &ed, start);
        draw_line(&mut ed);

        // Act
        saver.on_input(&ed, start + INTERVAL);
        let writer = saver.writer_thread();
        let failure = saver.wait();

        // Assert
        assert!(writer.is_some_and(|id| id != std::thread::current().id()));
        assert_eq!(failure, None);
        Ok(())
    }

    #[test]
    fn autosaver_starts_no_second_writer_while_one_runs() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("busy")?;
        let start = Instant::now();
        let mut ed = Editor::new();
        let mut saver = Autosaver::new(Some(dir.clone()), &ed, start);
        draw_line(&mut ed);
        saver.on_input(&ed, start + INTERVAL);
        let first = saver.writer_thread();
        draw_line(&mut ed);

        // Act: a capture still counts as the last one until it is done.
        saver.on_input(&ed, start + INTERVAL);

        // Assert
        assert_eq!(saver.writer_thread(), first);
        assert_eq!(saver.wait(), None);
        Ok(())
    }

    // ---- AC-3 the previous file --------------------------------------------

    #[test]
    fn first_autosave_moves_the_existing_file_to_prev() -> io::Result<()> {
        // Arrange: what a crashed session left.
        let dir = temp_dir("rotate")?;
        let crashed = with_lines(3);
        fs::write(dir.join(FILE), crashed.snapshot().to_text())?;
        let start = Instant::now();
        let mut ed = Editor::new();

        // Act
        after_first_save(&dir, &mut ed, start);

        // Assert
        assert_eq!(load(&dir, PREV_FILE)?, crashed);
        assert_eq!(load(&dir, FILE)?, ed);
        Ok(())
    }

    #[test]
    fn later_autosaves_keep_prev() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("keep")?;
        let crashed = with_lines(3);
        fs::write(dir.join(FILE), crashed.snapshot().to_text())?;
        let start = Instant::now();
        let mut ed = Editor::new();
        let mut saver = after_first_save(&dir, &mut ed, start);
        draw_line(&mut ed);

        // Act
        saver.on_input(&ed, start + INTERVAL * 2);
        let failure = saver.wait();

        // Assert
        assert_eq!(failure, None);
        assert_eq!(load(&dir, PREV_FILE)?, crashed);
        assert_eq!(load(&dir, FILE)?, ed);
        Ok(())
    }

    #[test]
    fn first_autosave_without_a_file_makes_no_prev() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("no-prev")?;
        let start = Instant::now();
        let mut ed = Editor::new();

        // Act
        after_first_save(&dir, &mut ed, start);

        // Assert
        assert_eq!(entries(&dir)?, [FILE]);
        Ok(())
    }

    #[test]
    fn failed_write_keeps_the_existing_autosave() -> io::Result<()> {
        // Arrange: a directory in the way of the temporary file.
        let dir = temp_dir("keeps")?;
        fs::write(dir.join(FILE), "old")?;
        fs::create_dir(temporary_path(&dir))?;

        // Act
        let result = write_atomic(&dir, "new", true);

        // Assert
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(dir.join(FILE))?, "old");
        assert!(!dir.join(PREV_FILE).exists());
        Ok(())
    }

    // ---- AC-4 draw --recover -----------------------------------------------

    #[test]
    fn launch_without_recover_is_normal() {
        assert_eq!(launch(&[]), Launch::Normal);
        assert_eq!(launch(&["--other"]), Launch::Normal);
    }

    #[test]
    fn launch_recover_is_latest() {
        assert_eq!(launch(&["--recover"]), Launch::Recover(Which::Latest));
    }

    #[test]
    fn launch_recover_prev_is_previous() {
        assert_eq!(
            launch(&["--recover", "prev"]),
            Launch::Recover(Which::Previous)
        );
    }

    #[test]
    fn launch_recover_other_is_invalid() {
        // Arrange / Act
        let parsed = launch(&["--recover", "older"]);

        // Assert
        let Launch::Invalid(reason) = parsed else {
            panic!("expected Invalid, got {parsed:?}");
        };
        assert!(reason.contains("older"), "{reason}");
    }

    #[test]
    fn recover_latest_opens_autosave() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("latest")?;
        let (latest, older) = (with_lines(2), with_lines(1));
        fs::write(dir.join(FILE), latest.snapshot().to_text())?;
        fs::write(dir.join(PREV_FILE), older.snapshot().to_text())?;

        // Act
        let (snapshot, error) = recover(&Launch::Recover(Which::Latest), Some(&dir));

        // Assert
        assert_eq!(error, None);
        assert_eq!(snapshot.map(Editor::from_snapshot), Some(latest));
        Ok(())
    }

    #[test]
    fn recover_prev_opens_the_older_file() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("prev")?;
        let (latest, older) = (with_lines(2), with_lines(1));
        fs::write(dir.join(FILE), latest.snapshot().to_text())?;
        fs::write(dir.join(PREV_FILE), older.snapshot().to_text())?;

        // Act
        let (snapshot, error) = recover(&Launch::Recover(Which::Previous), Some(&dir));

        // Assert
        assert_eq!(error, None);
        assert_eq!(snapshot.map(Editor::from_snapshot), Some(older));
        Ok(())
    }

    #[test]
    fn recover_normal_launch_asks_for_nothing() {
        // Arrange / Act
        let (snapshot, error) = recover(&Launch::Normal, None);

        // Assert
        assert!(snapshot.is_none() && error.is_none());
    }

    // ---- AC-5 failures -----------------------------------------------------

    fn failed_recovery(dir: Option<&Path>, launch: &Launch) -> String {
        let (snapshot, error) = recover(launch, dir);
        assert!(snapshot.is_none(), "a failed recovery starts blank");
        let error = error.unwrap_or_default();
        assert!(error.contains("starting blank"), "{error}");
        error
    }

    #[test]
    fn recover_missing_file_gives_the_reason() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("missing")?;

        // Act
        let error = failed_recovery(Some(&dir), &Launch::Recover(Which::Latest));

        // Assert
        assert!(error.contains(FILE), "{error}");
        Ok(())
    }

    #[test]
    fn recover_corrupt_file_gives_the_line() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("corrupt")?;
        fs::write(dir.join(PREV_FILE), "draw-snapshot 1\n[nonsense]\n")?;

        // Act
        let error = failed_recovery(Some(&dir), &Launch::Recover(Which::Previous));

        // Assert
        assert!(error.contains(PREV_FILE) && error.contains("line 2"), "{error}");
        Ok(())
    }

    #[test]
    fn recover_wrong_version_gives_the_reason() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("version")?;
        fs::write(dir.join(FILE), "draw-snapshot 4000000\n")?;

        // Act
        let error = failed_recovery(Some(&dir), &Launch::Recover(Which::Latest));

        // Assert
        assert!(error.contains("version 4000000"), "{error}");
        Ok(())
    }

    #[test]
    fn recover_without_a_state_directory_gives_the_reason() {
        // Arrange / Act
        let error = failed_recovery(None, &Launch::Recover(Which::Latest));

        // Assert
        assert!(error.contains("state directory"), "{error}");
    }

    #[test]
    fn recover_invalid_launch_gives_the_reason() {
        // Arrange / Act
        let error = failed_recovery(None, &Launch::Invalid("bad value".to_owned()));

        // Assert
        assert!(error.contains("bad value"), "{error}");
    }

    proptest! {
        #[test]
        fn recover_any_bytes_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..400)) {
            // Arrange
            let dir = temp_dir("bytes").map_err(|e| TestCaseError::fail(e.to_string()))?;
            fs::write(dir.join(FILE), &bytes).map_err(|e| TestCaseError::fail(e.to_string()))?;

            // Act
            let (snapshot, error) = recover(&Launch::Recover(Which::Latest), Some(&dir));

            // Assert
            prop_assert!(snapshot.is_some() != error.is_some());
        }
    }

    #[test]
    fn failed_write_is_reported_and_retried() -> io::Result<()> {
        // Arrange: a file where the directory should go.
        let blocked = temp_dir("failed")?.join("blocked");
        fs::write(&blocked, "in the way")?;
        let start = Instant::now();
        let mut ed = Editor::new();
        let mut saver = Autosaver::new(Some(blocked.clone()), &ed, start);
        draw_line(&mut ed);

        // Act
        saver.on_input(&ed, start + INTERVAL);
        let failure = saver.wait();
        fs::remove_file(&blocked)?;
        saver.on_input(&ed, start + INTERVAL * 2);
        let retry = saver.wait();

        // Assert
        let failure = failure.unwrap_or_default();
        assert!(failure.contains("autosave not written"), "{failure}");
        assert_eq!(retry, None);
        assert_eq!(load(&blocked, FILE)?, ed);
        Ok(())
    }

    #[test]
    fn autosaver_without_a_state_directory_is_inert() {
        // Arrange
        let start = Instant::now();
        let mut ed = Editor::new();
        let mut saver = Autosaver::new(None, &ed, start);
        draw_line(&mut ed);

        // Act
        saver.on_input(&ed, start + INTERVAL);

        // Assert
        assert_eq!(saver.writer_thread(), None);
        assert_eq!(saver.wait(), None);
    }

    // ---- AC-6 a clean exit -------------------------------------------------

    #[test]
    fn dropping_the_autosaver_leaves_the_files() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("drop")?;
        let crashed = with_lines(1);
        fs::write(dir.join(FILE), crashed.snapshot().to_text())?;
        let start = Instant::now();
        let mut ed = Editor::new();
        let saver = after_first_save(&dir, &mut ed, start);

        // Act
        drop(saver);

        // Assert
        assert_eq!(load(&dir, PREV_FILE)?, crashed);
        assert_eq!(load(&dir, FILE)?, ed);
        Ok(())
    }

    // ---- AC-8 the docs -----------------------------------------------------

    #[test]
    fn readme_and_home_do_not_claim_nothing_is_saved() {
        // Arrange
        let readme = include_str!("../../README.md");
        let home = include_str!("../../docs/Home.md");

        // Act / Assert
        for text in [readme, home] {
            assert!(!text.contains("Nothing is saved"));
        }
        assert!(readme.contains("draw --recover"));
    }
}
