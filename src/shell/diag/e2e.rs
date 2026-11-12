//! Test-only hooks for the end-to-end suite (T75, ADR-T75-1).
//!
//! Inert unless `--e2e-dump` or `--e2e-idle` is given. The harness sends
//! [`SYNC_CHORD`] after each input step. Wayland keeps a client's events in
//! order, so once the app has answered that chord it has handled everything
//! sent before it. The answer is the editor snapshot in the dump file, then
//! the count of syncs in the idle file, written after the frame is on screen.

use std::path::{Path, PathBuf};

use crate::core::input::{InputEvent, Key};
use crate::core::keymap::KeyChord;
use crate::shell::options::E2eFiles;

/// The chord the harness sends after each input step. Not in the keymap.
pub const SYNC_CHORD: KeyChord = KeyChord::ctrl_shift(Key::Digit9);

/// Whether `event` is a press of [`SYNC_CHORD`].
#[must_use]
pub fn is_sync(event: &InputEvent) -> bool {
    matches!(event, InputEvent::KeyDown { key, mods } if *key == SYNC_CHORD.key && *mods == SYNC_CHORD.mods)
}

/// Whether `event` is the text that [`SYNC_CHORD`] also produces.
#[must_use]
pub fn is_sync_char(event: &InputEvent) -> bool {
    // `(` is the shifted 9 of a real keyboard.
    matches!(event, InputEvent::Char { ch: '9' | '(', mods } if *mods == SYNC_CHORD.mods)
}

/// Replaces `path` in one step, so a reader never sees half a file. A
/// failed write only loses the signal, and the harness then times out.
fn write_atomic(path: &Path, text: &str) {
    let temporary = path.with_extension("tmp");
    if std::fs::write(&temporary, text).is_ok() {
        let _ = std::fs::rename(&temporary, path);
    }
}

/// The hooks of one session.
#[derive(Debug)]
pub struct Hooks {
    dump: Option<PathBuf>,
    idle: Option<PathBuf>,
    /// Syncs received.
    seen: u64,
    /// Syncs answered in the idle file.
    answered: u64,
    /// The sync whose dump is written, waiting for its frame to be swapped.
    armed: Option<u64>,
    /// File the harness leaves clipboard text in.
    clipboard: Option<PathBuf>,
}

impl Hooks {
    /// Hooks for the files of `files`; `None` when neither the dump nor
    /// the idle file is given, which leaves the app as it is without them.
    #[must_use]
    pub fn from_files(files: &E2eFiles) -> Option<Self> {
        (files.dump.is_some() || files.idle.is_some()).then(|| Self {
            dump: files.dump.clone(),
            idle: files.idle.clone(),
            seen: 0,
            answered: 0,
            armed: None,
            clipboard: files.clipboard.clone(),
        })
    }

    /// The text the harness left for the clipboard, once.
    #[must_use]
    pub fn take_clipboard(&self) -> Option<String> {
        let path = self.clipboard.as_ref()?;
        let text = std::fs::read_to_string(path).ok()?;
        let _ = std::fs::remove_file(path);
        Some(text)
    }

    /// Records one received [`SYNC_CHORD`].
    pub fn note_sync(&mut self) {
        self.seen += 1;
    }

    /// Takes the events of the sync chord; returns whether `event` was one
    /// and must not reach the editor.
    pub fn intercept(&mut self, event: &InputEvent) -> bool {
        if is_sync(event) {
            self.note_sync();
            return true;
        }
        is_sync_char(event)
    }

    /// Call after a frame is drawn. Writes the dump for a pending sync when
    /// the frame is `settled`; returns whether the loop must wake once more
    /// so [`Self::after_swap`] runs after this frame is on screen.
    pub fn frame_presented(&mut self, settled: bool, snapshot: impl FnOnce() -> String) -> bool {
        if self.armed.is_some() || self.seen == self.answered || !settled {
            return false;
        }
        if let Some(dump) = &self.dump {
            write_atomic(dump, &snapshot());
        }
        self.armed = Some(self.seen);
        true
    }

    /// Call once the previous frame was swapped: answers the armed sync.
    pub fn after_swap(&mut self) {
        let Some(count) = self.armed.take() else {
            return;
        };
        self.answered = count;
        if let Some(idle) = &self.idle {
            write_atomic(idle, &format!("{count}\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::input::{InputEvent, Key, Modifiers};
    use std::path::Path;

    const CTRL_SHIFT: Modifiers = Modifiers {
        shift: true,
        ctrl: true,
        alt: false,
    };

    /// An empty directory of its own for one test.
    fn temp_dir(name: &str) -> std::io::Result<PathBuf> {
        let dir = std::env::temp_dir().join(format!("draw-e2e-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn files(dump: Option<&str>, idle: Option<&str>) -> E2eFiles {
        E2eFiles {
            dump: dump.map(PathBuf::from),
            idle: idle.map(PathBuf::from),
            clipboard: None,
        }
    }

    fn hooks(dir: &Path) -> Hooks {
        let files = E2eFiles {
            dump: Some(dir.join("dump.snap")),
            idle: Some(dir.join("idle")),
            clipboard: None,
        };
        Hooks::from_files(&files).unwrap_or_else(|| panic!("both files are given"))
    }

    #[test]
    fn hooks_without_files_are_none() {
        assert!(Hooks::from_files(&E2eFiles::default()).is_none());
    }

    #[test]
    fn hooks_one_file_is_enough() {
        assert!(Hooks::from_files(&files(Some("/tmp/file"), None)).is_some());
        assert!(Hooks::from_files(&files(None, Some("/tmp/file"))).is_some());
    }

    #[test]
    fn hooks_is_sync_only_for_the_chord() {
        // Arrange
        let press = |key, mods| InputEvent::KeyDown { key, mods };

        // Act / Assert
        assert_eq!(SYNC_CHORD, KeyChord::ctrl_shift(Key::Digit9));
        assert!(is_sync(&press(Key::Digit9, CTRL_SHIFT)));
        assert!(!is_sync(&press(Key::Digit9, Modifiers::NONE)));
        assert!(!is_sync(&press(Key::Digit8, CTRL_SHIFT)));
        assert!(!is_sync(&InputEvent::KeyUp {
            key: Key::Digit9,
            mods: CTRL_SHIFT,
        }));
        // Not bound in the keymap: the chord never reaches a command.
        assert_eq!(crate::core::keymap::resolve(Key::Digit9, CTRL_SHIFT), None);
    }

    #[test]
    fn is_sync_char_matches_the_chord_text_only() {
        // Arrange
        let typed = |ch, mods| InputEvent::Char { ch, mods };

        // Act / Assert
        assert!(is_sync_char(&typed('9', CTRL_SHIFT)));
        assert!(is_sync_char(&typed('(', CTRL_SHIFT)));
        assert!(!is_sync_char(&typed('9', Modifiers::NONE)));
        assert!(!is_sync_char(&typed('a', CTRL_SHIFT)));
    }

    #[test]
    fn hooks_write_nothing_until_a_sync() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("nothing")?;
        let mut hooks = hooks(&dir);

        // Act
        let wake = hooks.frame_presented(true, || panic!("no snapshot is needed"));
        hooks.after_swap();

        // Assert
        assert!(!wake);
        assert!(!dir.join("dump.snap").exists());
        assert!(!dir.join("idle").exists());
        Ok(())
    }

    #[test]
    fn hooks_sync_writes_dump_then_idle_after_the_swap() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("sync")?;
        let mut hooks = hooks(&dir);
        hooks.note_sync();

        // Act
        let wake = hooks.frame_presented(true, || "state\n".to_owned());

        // Assert: the dump is ready, the idle signal waits for the swap.
        assert!(wake);
        assert_eq!(std::fs::read_to_string(dir.join("dump.snap"))?, "state\n");
        assert!(!dir.join("idle").exists());

        // Act
        hooks.after_swap();

        // Assert
        assert_eq!(std::fs::read_to_string(dir.join("idle"))?, "1\n");
        Ok(())
    }

    #[test]
    fn hooks_unsettled_frame_waits() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("unsettled")?;
        let mut hooks = hooks(&dir);
        hooks.note_sync();

        // Act
        let wake = hooks.frame_presented(false, || panic!("not settled yet"));
        hooks.after_swap();

        // Assert
        assert!(!wake);
        assert!(!dir.join("idle").exists());

        // Act: the frame that settles answers the sync.
        assert!(hooks.frame_presented(true, || "late\n".to_owned()));
        hooks.after_swap();

        // Assert
        assert_eq!(std::fs::read_to_string(dir.join("idle"))?, "1\n");
        Ok(())
    }

    #[test]
    fn hooks_each_sync_is_counted() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("count")?;
        let mut hooks = hooks(&dir);

        for expected in ["1\n", "2\n", "3\n"] {
            // Act
            hooks.note_sync();
            assert!(hooks.frame_presented(true, String::new));
            hooks.after_swap();

            // Assert
            assert_eq!(std::fs::read_to_string(dir.join("idle"))?, expected);
        }
        Ok(())
    }

    #[test]
    fn hooks_take_clipboard_returns_the_text_once() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("clipboard")?;
        let path = dir.join("clipboard");
        std::fs::write(&path, "hello\n")?;
        let hooks = Hooks::from_files(&E2eFiles {
            dump: Some(dir.join("dump.snap")),
            idle: None,
            clipboard: Some(path.clone()),
        })
        .unwrap_or_else(|| panic!("a dump file is given"));

        // Act
        let first = hooks.take_clipboard();
        let second = hooks.take_clipboard();

        // Assert
        assert_eq!(first.as_deref(), Some("hello\n"));
        assert_eq!(second, None);
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn hooks_without_clipboard_take_nothing() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("no-clipboard")?;
        let unset = hooks(&dir);
        let missing = Hooks::from_files(&E2eFiles {
            dump: Some(dir.join("dump.snap")),
            idle: None,
            clipboard: Some(dir.join("absent")),
        })
        .unwrap_or_else(|| panic!("a dump file is given"));

        // Act / Assert
        assert_eq!(unset.take_clipboard(), None);
        assert_eq!(missing.take_clipboard(), None);
        Ok(())
    }

    #[test]
    fn hooks_unwritable_path_does_not_panic() {
        // Arrange
        let bad = Path::new("/nonexistent-dir/deep/file");
        let mut hooks = Hooks::from_files(&E2eFiles {
            dump: Some(bad.to_path_buf()),
            idle: Some(bad.to_path_buf()),
            clipboard: None,
        })
        .unwrap_or_else(|| panic!("both files are given"));
        hooks.note_sync();

        // Act / Assert: a failed write only loses the signal.
        assert!(hooks.frame_presented(true, String::new));
        hooks.after_swap();
    }
}
