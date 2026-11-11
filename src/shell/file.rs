#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

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
