//! Test-only hooks for the end-to-end suite (T75, ADR-T75-1).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::input::{InputEvent, Key, Modifiers};
    use std::ffi::OsStr;
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

    fn hooks(dir: &Path) -> Hooks {
        let (dump, idle) = (dir.join("dump.snap"), dir.join("idle"));
        Hooks::from_values(Some(dump.as_os_str()), Some(idle.as_os_str()))
            .unwrap_or_else(|| panic!("both variables are set"))
    }

    #[test]
    fn env_names_are_stable() {
        assert_eq!(DUMP_ENV, "DRAW_E2E_DUMP");
        assert_eq!(IDLE_ENV, "DRAW_E2E_IDLE");
    }

    #[test]
    fn hooks_from_env_unset_is_none() {
        assert!(Hooks::from_values(None, None).is_none());
    }

    #[test]
    fn hooks_from_env_empty_is_none() {
        let empty = Some(OsStr::new(""));

        assert!(Hooks::from_values(empty, empty).is_none());
        assert!(Hooks::from_values(empty, None).is_none());
    }

    #[test]
    fn hooks_from_env_one_variable_is_enough() {
        let path = Some(OsStr::new("/tmp/file"));

        assert!(Hooks::from_values(path, None).is_some());
        assert!(Hooks::from_values(None, path).is_some());
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
        assert_eq!(
            crate::core::keymap::resolve(Key::Digit9, CTRL_SHIFT),
            None
        );
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
    fn hooks_unwritable_path_does_not_panic() {
        // Arrange
        let bad = Path::new("/nonexistent-dir/deep/file");
        let mut hooks = Hooks::from_values(Some(bad.as_os_str()), Some(bad.as_os_str()))
            .unwrap_or_else(|| panic!("both variables are set"));
        hooks.note_sync();

        // Act / Assert: a failed write only loses the signal.
        assert!(hooks.frame_presented(true, String::new));
        hooks.after_swap();
    }
}
