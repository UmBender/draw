//! Window backend chosen at start-up from `DRAW_BACKEND` (T28,
//! ADR-T28-1).
//!
//! [`choose`] is a pure function over the variable's value; [`from_env`]
//! feeds it the real one. X11 (on Wayland sessions through XWayland) is
//! the default; the native Wayland backend is experimental and falls back
//! to X11 when it cannot connect.

use std::ffi::OsStr;

use macroquad::miniquad::conf::LinuxBackend;

/// Environment variable naming the window backend.
pub const BACKEND_ENV: &str = "DRAW_BACKEND";

/// The window backend draw asks miniquad for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Backend {
    /// X11 only, through XWayland on Wayland sessions (the default).
    #[default]
    X11,
    /// Native Wayland, falling back to X11 (experimental).
    Wayland,
}

/// The backend picked from [`BACKEND_ENV`] and what to tell the user.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Choice {
    /// The backend to use.
    pub backend: Backend,
    /// One stderr line when the value was not understood.
    pub warning: Option<String>,
}

impl Backend {
    /// The miniquad setting for this backend.
    #[must_use]
    pub fn linux_backend(self) -> LinuxBackend {
        todo!()
    }

    /// One stderr line to print at start-up, if this backend needs one.
    #[must_use]
    pub fn notice(self) -> Option<&'static str> {
        todo!()
    }
}

/// Picks the backend from the value of [`BACKEND_ENV`] (`None` when
/// unset).
#[must_use]
pub fn choose(value: Option<&OsStr>) -> Choice {
    todo!()
}

/// [`choose`] applied to the real environment.
#[must_use]
pub fn from_env() -> Choice {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::ffi::OsStr;

    fn choose_str(value: &str) -> Choice {
        choose(Some(OsStr::new(value)))
    }

    #[test]
    fn choose_unset_or_empty_is_x11_without_warning() {
        // Arrange / Act
        let choices = [choose(None), choose_str(""), choose_str("  ")];

        // Assert
        for choice in choices {
            assert_eq!(choice.backend, Backend::X11);
            assert_eq!(choice.warning, None);
        }
    }

    #[test]
    fn choose_x11_is_x11_without_warning() {
        // Arrange / Act
        let choice = choose_str("x11");

        // Assert
        assert_eq!(choice.backend, Backend::X11);
        assert_eq!(choice.warning, None);
    }

    #[test]
    fn choose_wayland_is_wayland_without_warning() {
        // Arrange / Act
        let choice = choose_str("wayland");

        // Assert
        assert_eq!(choice.backend, Backend::Wayland);
        assert_eq!(choice.warning, None);
    }

    #[test]
    fn choose_ignores_case_and_surrounding_space() {
        // Arrange
        let cases = [
            (" Wayland ", Backend::Wayland),
            ("WAYLAND", Backend::Wayland),
            ("\tX11\n", Backend::X11),
        ];

        for (value, expected) in cases {
            // Act
            let choice = choose_str(value);

            // Assert
            assert_eq!(choice.backend, expected, "{value:?}");
            assert_eq!(choice.warning, None, "{value:?}");
        }
    }

    #[test]
    fn choose_unknown_value_warns_and_keeps_x11() {
        // Arrange
        let cases = ["wl", "xwayland", "way land", "1", "wayland,x11"];

        for value in cases {
            // Act
            let choice = choose_str(value);

            // Assert
            assert_eq!(choice.backend, Backend::X11, "{value:?}");
            let warning = choice.warning.unwrap_or_default();
            assert!(warning.contains(BACKEND_ENV), "{warning}");
            assert!(warning.contains(value.trim()), "{warning}");
        }
    }

    #[test]
    fn choose_non_utf8_value_warns_and_keeps_x11() {
        // Arrange
        #[cfg(unix)]
        let value = {
            use std::os::unix::ffi::OsStrExt;
            OsStr::from_bytes(b"way\xffland")
        };
        #[cfg(not(unix))]
        let value = OsStr::new("\u{fffd}");

        // Act
        let choice = choose(Some(value));

        // Assert
        assert_eq!(choice.backend, Backend::X11);
        assert!(choice.warning.is_some());
    }

    #[test]
    fn backend_linux_backend_maps_to_miniquad() {
        // Arrange / Act / Assert
        assert_eq!(Backend::X11.linux_backend(), LinuxBackend::X11Only);
        assert_eq!(
            Backend::Wayland.linux_backend(),
            LinuxBackend::WaylandWithX11Fallback
        );
        assert_eq!(Backend::default(), Backend::X11);
    }

    #[test]
    fn backend_notice_only_for_wayland() {
        // Arrange / Act
        let x11 = Backend::X11.notice();
        let wayland = Backend::Wayland.notice();

        // Assert
        assert_eq!(x11, None);
        let wayland = wayland.unwrap_or_default();
        assert!(wayland.contains("experimental"), "{wayland}");
        assert!(wayland.contains(BACKEND_ENV), "{wayland}");
    }

    #[test]
    fn warning_and_notice_are_one_line() {
        // Arrange
        let lines = [
            choose_str("bogus").warning.unwrap_or_default(),
            Backend::Wayland.notice().unwrap_or_default().to_owned(),
        ];

        for line in lines {
            // Assert
            assert!(line.starts_with("draw: "), "{line}");
            assert!(!line.contains('\n'), "{line}");
        }
    }

    #[test]
    fn readme_documents_draw_backend_as_experimental() {
        // Arrange
        let readme = include_str!("../../README.md");

        // Act
        let experimental = readme
            .split("### Experimental")
            .nth(1)
            .and_then(|rest| rest.split("\n## ").next())
            .unwrap_or_default();

        // Assert
        assert!(
            readme.contains(&format!("| `{BACKEND_ENV}` |")),
            "variable table"
        );
        assert!(
            experimental.contains(&format!("{BACKEND_ENV}=wayland")),
            "Experimental section"
        );
    }

    proptest! {
        #[test]
        fn choose_any_value_warns_unless_known(value in ".*") {
            let choice = choose_str(&value);
            let known = matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "" | "x11" | "wayland"
            );
            prop_assert_eq!(choice.warning.is_none(), known);
            if choice.backend == Backend::Wayland {
                prop_assert!(known);
            }
        }
    }
}
