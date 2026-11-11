//! Opening and saving the `.draw` file: the path, the atomic write and the
//! log lines (T108, ADR-T108-1). Pure of the window: tested on its own.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::core::file::DocFile;
use crate::shell::diag::Level;

/// What opening the command-line file gave.
#[derive(Debug)]
pub enum Opened {
    /// The file loaded.
    Loaded(DocFile),
    /// There is no such file: start empty, the first save creates it.
    Missing,
    /// The file exists but cannot be read or is damaged or newer; the
    /// message names the file. It is never overwritten.
    Refused(String),
}

/// The `.draw` file named on the command line: the first argument that is
/// not an option and not the value of `--recover`.
#[must_use]
pub fn path_from_args(args: impl IntoIterator<Item = OsString>) -> Option<PathBuf> {
    todo!("{:?}", args.into_iter().count())
}

/// `draw-<UTC date>-<time>.draw` in `home`, with `-<attempt>` before the
/// extension unless `attempt` is 0.
#[must_use]
pub fn default_path(home: &Path, unix_secs: u64, attempt: u32) -> PathBuf {
    todo!("{home:?} {unix_secs} {attempt}")
}

/// Writes `text` to a temporary file next to `path` and renames it over
/// `path`, so a reader never sees half a file.
///
/// # Errors
///
/// The I/O error of the first step that failed; the temporary file is
/// removed.
pub fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    todo!("{path:?} {}", text.len())
}

/// Reads and parses `path`.
#[must_use]
pub fn open(path: &Path) -> Opened {
    todo!("{path:?}")
}

/// The file `Ctrl+S` writes to after opening `arg` gave `opened`: the
/// argument, unless it was refused (a damaged file is never overwritten).
#[must_use]
pub fn save_target(opened: &Opened, arg: Option<PathBuf>) -> Option<PathBuf> {
    todo!("{opened:?} {arg:?}")
}

/// The log level and line for the outcome of a save.
#[must_use]
pub fn save_log_line(result: &Result<PathBuf, String>) -> (Level, String) {
    todo!("{result:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    /// A fresh, empty directory of this test run.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("draw-file-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display()));
        dir
    }

    const GOOD: &str = "draw-file 1\n[pages]\ncurrent 0\ncount 0\n";

    #[test]
    fn path_from_args_takes_the_first_plain_argument() {
        assert_eq!(path_from_args(args(&["a.draw"])), Some("a.draw".into()));
        assert_eq!(
            path_from_args(args(&["--recover", "a.draw"])),
            None,
            "the value of --recover is not a file"
        );
        assert_eq!(
            path_from_args(args(&["--recover", "prev", "a.draw"])),
            Some("a.draw".into())
        );
        assert_eq!(path_from_args(args(&["--other", "a.draw"])), Some("a.draw".into()));
        assert_eq!(path_from_args(args(&[])), None);
        assert_eq!(path_from_args(args(&["--recover"])), None);
    }

    #[test]
    fn default_path_is_draw_stamp_dot_draw_in_home() {
        let path = default_path(Path::new("/h"), 0, 0);
        assert_eq!(path, PathBuf::from("/h/draw-19700101-000000.draw"));
        assert_eq!(
            default_path(Path::new("/h"), 0, 2),
            PathBuf::from("/h/draw-19700101-000000-2.draw")
        );
    }

    #[test]
    fn write_atomic_creates_then_replaces_and_leaves_no_temporary() {
        let dir = temp_dir("atomic");
        let path = dir.join("a.draw");

        write_atomic(&path, "one").unwrap_or_else(|err| panic!("{err}"));
        write_atomic(&path, "two").unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(std::fs::read_to_string(&path).ok().as_deref(), Some("two"));
        let names: Vec<_> = std::fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("{err}"))
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(names, vec![OsString::from("a.draw")]);
    }

    #[test]
    fn write_atomic_into_a_missing_directory_is_an_error() {
        let dir = temp_dir("nodir");
        assert!(write_atomic(&dir.join("no/such/a.draw"), "x").is_err());
    }

    #[test]
    fn open_of_a_missing_file_is_missing() {
        let dir = temp_dir("missing");
        assert!(matches!(open(&dir.join("x.draw")), Opened::Missing));
    }

    #[test]
    fn open_of_a_good_file_loads_it() {
        let dir = temp_dir("good");
        let path = dir.join("x.draw");
        std::fs::write(&path, GOOD).unwrap_or_else(|err| panic!("{err}"));
        assert!(matches!(open(&path), Opened::Loaded(_)));
    }

    #[test]
    fn open_of_a_truncated_newer_or_binary_file_is_refused_and_untouched() {
        let dir = temp_dir("bad");
        let cases: [(&str, &[u8]); 3] = [
            ("cut.draw", &GOOD.as_bytes()[..20]),
            ("new.draw", b"draw-file 2\n"),
            ("bin.draw", &[0xff, 0xfe, 0x00, 0x01]),
        ];
        for (name, bytes) in cases {
            let path = dir.join(name);
            std::fs::write(&path, bytes).unwrap_or_else(|err| panic!("{err}"));

            let opened = open(&path);

            match opened {
                Opened::Refused(message) => assert!(message.contains(name), "{message}"),
                _ => panic!("{name} should be refused"),
            }
            assert_eq!(std::fs::read(&path).ok().as_deref(), Some(bytes));
        }
    }

    #[test]
    fn save_target_is_the_argument_unless_it_was_refused() {
        let arg = Some(PathBuf::from("a.draw"));
        assert_eq!(save_target(&Opened::Missing, arg.clone()), arg);
        assert_eq!(save_target(&Opened::Refused(String::new()), arg), None);
        assert_eq!(save_target(&Opened::Missing, None), None);
    }

    #[test]
    fn save_line_names_the_file() {
        let (level, line) = save_log_line(&Ok(PathBuf::from("/h/a.draw")));
        assert_eq!(level, Level::Info);
        assert!(line.contains("/h/a.draw"), "{line}");
        let (level, line) = save_log_line(&Err("disk full".to_owned()));
        assert_eq!(level, Level::Error);
        assert!(line.contains("disk full"), "{line}");
    }
}
