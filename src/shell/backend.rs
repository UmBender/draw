//! Window backend chosen at start-up from `DRAW_BACKEND` and the session
//! (T28, ADR-T28-1, T46, ADR-T46-1).
//!
//! [`choose`] is a pure function over the variable's value and
//! `WAYLAND_DISPLAY`; [`from_env`] feeds it the real ones. Native Wayland
//! is the default on Wayland sessions and falls back to X11 when it
//! cannot connect; X11 (Xwayland on Wayland sessions) stays supported
//! (ADR-T128-1).

use std::ffi::OsStr;

use macroquad::miniquad::conf::LinuxBackend;

/// Environment variable naming the window backend.
pub const BACKEND_ENV: &str = "DRAW_BACKEND";

/// Environment variable set in a Wayland session.
pub const WAYLAND_DISPLAY_ENV: &str = "WAYLAND_DISPLAY";

/// The window backend draw asks miniquad for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    /// X11 only, through Xwayland on Wayland sessions.
    X11,
    /// Native Wayland, falling back to X11 (the default on Wayland
    /// sessions).
    Wayland,
}

/// The backend picked from [`BACKEND_ENV`] and the session, and what to
/// tell the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The backend to use.
    pub backend: Backend,
    /// One stderr line when the value was not understood.
    pub warning: Option<String>,
    /// One stderr line of information; always `None` since X11 is
    /// supported (ADR-T128-1), kept for the start-up printing in `app`.
    pub notice: Option<String>,
}

impl Backend {
    /// The miniquad setting for this backend.
    #[must_use]
    pub fn linux_backend(self) -> LinuxBackend {
        match self {
            Self::X11 => LinuxBackend::X11Only,
            Self::Wayland => LinuxBackend::WaylandWithX11Fallback,
        }
    }

    /// The value of [`BACKEND_ENV`] that names this backend.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::X11 => "x11",
            Self::Wayland => "wayland",
        }
    }
}

/// Picks the backend from the value of [`BACKEND_ENV`] (`None` when
/// unset) in a session whose [`WAYLAND_DISPLAY_ENV`] is
/// `wayland_display`.
#[must_use]
pub fn choose(value: Option<&OsStr>, wayland_display: Option<&OsStr>) -> Choice {
    let session = is_wayland_session(wayland_display);
    let default = if session {
        Backend::Wayland
    } else {
        Backend::X11
    };
    let text = value.map(OsStr::to_string_lossy).unwrap_or_default();
    let text = text.trim();
    let (backend, warning) = if text.is_empty() {
        (default, None)
    } else if text.eq_ignore_ascii_case(Backend::X11.name()) {
        (Backend::X11, None)
    } else if text.eq_ignore_ascii_case(Backend::Wayland.name()) {
        (Backend::Wayland, None)
    } else {
        let warning = format!(
            "draw: unknown {BACKEND_ENV} value {text:?}, using {} \
             (expected x11 or wayland)",
            default.name()
        );
        (default, Some(warning))
    };
    Choice {
        backend,
        warning,
        notice: None,
    }
}

/// Whether `wayland_display` (the value of [`WAYLAND_DISPLAY_ENV`])
/// names a Wayland session.
fn is_wayland_session(wayland_display: Option<&OsStr>) -> bool {
    wayland_display.is_some_and(|value| !value.is_empty())
}

/// [`choose`] applied to the real environment.
#[must_use]
pub fn from_env() -> Choice {
    choose(
        std::env::var_os(BACKEND_ENV).as_deref(),
        std::env::var_os(WAYLAND_DISPLAY_ENV).as_deref(),
    )
}

/// Wheel units a native Wayland compositor reports per wheel notch.
///
/// miniquad passes `wl_pointer.axis` values through unchanged; libinput
/// compositors (`wlroots`, Hyprland, `KWin`) send 15 per notch and Mutter 10,
/// where X11 sends 1. Dividing by 15 makes one notch one zoom step.
pub const WAYLAND_WHEEL_UNITS_PER_NOTCH: f32 = 15.0;

/// Wheel units per notch for `backend` in a session whose
/// `WAYLAND_DISPLAY` is `wayland_display`: [`WAYLAND_WHEEL_UNITS_PER_NOTCH`]
/// on native Wayland, `1` on X11 (including the fallback without a
/// Wayland session).
#[must_use]
pub fn wheel_units_per_notch(backend: Backend, wayland_display: Option<&OsStr>) -> f32 {
    if backend == Backend::Wayland && is_wayland_session(wayland_display) {
        WAYLAND_WHEEL_UNITS_PER_NOTCH
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::ffi::OsStr;

    fn display() -> &'static OsStr {
        OsStr::new("wayland-1")
    }

    fn choose_str(value: &str, wayland_display: Option<&OsStr>) -> Choice {
        choose(Some(OsStr::new(value)), wayland_display)
    }

    #[test]
    fn choose_unset_on_wayland_session_is_wayland() {
        // Arrange / Act
        let choices = [
            choose(None, Some(display())),
            choose_str("", Some(display())),
            choose_str("  ", Some(display())),
        ];

        // Assert
        for choice in choices {
            assert_eq!(choice.backend, Backend::Wayland);
            assert_eq!(choice.warning, None);
            assert_eq!(choice.notice, None);
        }
    }

    #[test]
    fn choose_unset_without_wayland_session_is_x11_without_notice() {
        // Arrange / Act
        let choices = [
            choose(None, None),
            choose(None, Some(OsStr::new(""))),
            choose_str("", None),
        ];

        // Assert
        for choice in choices {
            assert_eq!(choice.backend, Backend::X11);
            assert_eq!(choice.warning, None);
            assert_eq!(choice.notice, None);
        }
    }

    #[test]
    fn choose_x11_on_wayland_session_is_x11_without_notice() {
        // Arrange / Act
        let choice = choose_str("x11", Some(display()));

        // Assert
        assert_eq!(choice.backend, Backend::X11);
        assert_eq!(choice.warning, None);
        assert_eq!(choice.notice, None);
    }

    #[test]
    fn choose_wayland_on_wayland_session_has_no_notice() {
        // Arrange / Act
        let choice = choose_str("wayland", Some(display()));

        // Assert
        assert_eq!(choice.backend, Backend::Wayland);
        assert_eq!(choice.warning, None);
        assert_eq!(choice.notice, None);
    }

    #[test]
    fn choose_wayland_without_wayland_session_has_no_notice() {
        // Arrange / Act: miniquad falls back to X11 here
        let choice = choose_str("wayland", None);

        // Assert
        assert_eq!(choice.backend, Backend::Wayland);
        assert_eq!(choice.warning, None);
        assert_eq!(choice.notice, None);
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
            let choice = choose_str(value, Some(display()));

            // Assert
            assert_eq!(choice.backend, expected, "{value:?}");
            assert_eq!(choice.warning, None, "{value:?}");
            assert_eq!(choice.notice, None, "{value:?}");
        }
    }

    #[test]
    fn choose_unknown_value_warns_and_uses_session_default() {
        // Arrange
        let cases = ["wl", "xwayland", "way land", "1", "wayland,x11"];

        for value in cases {
            // Act
            let on_wayland = choose_str(value, Some(display()));
            let on_x11 = choose_str(value, None);

            // Assert
            assert_eq!(on_wayland.backend, Backend::Wayland, "{value:?}");
            assert_eq!(on_x11.backend, Backend::X11, "{value:?}");
            let warning = on_wayland.warning.unwrap_or_default();
            assert!(warning.contains(BACKEND_ENV), "{warning}");
            assert!(warning.contains(value.trim()), "{warning}");
            assert!(warning.contains("using wayland"), "{warning}");
            let warning = on_x11.warning.unwrap_or_default();
            assert!(warning.contains("using x11"), "{warning}");
        }
    }

    #[test]
    fn choose_non_utf8_value_warns_and_uses_session_default() {
        // Arrange
        #[cfg(unix)]
        let value = {
            use std::os::unix::ffi::OsStrExt;
            OsStr::from_bytes(b"way\xffland")
        };
        #[cfg(not(unix))]
        let value = OsStr::new("\u{fffd}");

        // Act
        let on_wayland = choose(Some(value), Some(display()));
        let on_x11 = choose(Some(value), None);

        // Assert
        assert_eq!(on_wayland.backend, Backend::Wayland);
        assert!(on_wayland.warning.is_some());
        assert_eq!(on_x11.backend, Backend::X11);
        assert!(on_x11.warning.is_some());
    }

    #[test]
    fn backend_linux_backend_maps_to_miniquad() {
        // Arrange / Act / Assert
        assert_eq!(Backend::X11.linux_backend(), LinuxBackend::X11Only);
        assert_eq!(
            Backend::Wayland.linux_backend(),
            LinuxBackend::WaylandWithX11Fallback
        );
    }

    #[test]
    fn warning_is_one_line() {
        // Arrange
        let lines = [
            choose_str("bogus", Some(display())).warning,
            choose_str("bogus", None).warning,
        ];

        for line in lines {
            // Assert
            let line = line.unwrap_or_default();
            assert!(line.starts_with("draw: "), "{line}");
            assert!(!line.contains('\n'), "{line}");
        }
    }

    #[test]
    fn x11_is_not_marked_deprecated() {
        // Arrange
        let texts = [
            include_str!("../../README.md"),
            include_str!("../../packaging/flatpak/io.github.UmBender.draw.yml"),
            include_str!("../../packaging/runtime-libs"),
            include_str!("../../docs/features/Native Wayland.md")
                .split("## Rollback")
                .next()
                .unwrap_or_default(),
        ];

        // Act / Assert
        for text in texts {
            // Titles of linked notes keep the word; the prose must not.
            let hit = text.lines().find(|line| {
                let prose = line.split("[[").next().unwrap_or_default();
                !line.starts_with("task:")
                    && !line.starts_with("adrs:")
                    && prose.to_lowercase().contains("deprecat")
            });
            assert_eq!(hit, None);
        }
    }

    #[test]
    fn readme_documents_wayland_default_and_x11_supported() {
        // Arrange
        let readme = include_str!("../../README.md");

        // Act
        let row = readme
            .lines()
            .find(|line| line.starts_with(&format!("| `{BACKEND_ENV}` |")))
            .unwrap_or_default();
        let experimental = readme
            .split("### Experimental")
            .nth(1)
            .and_then(|rest| rest.split("\n## ").next())
            .unwrap_or_default();

        // Assert
        assert!(row.contains("`wayland` (default"), "{row}");
        assert!(row.contains("`x11`"), "{row}");
        assert!(!row.contains("removed"), "{row}");
        assert!(!row.contains("experimental"), "{row}");
        assert!(
            !experimental.contains(BACKEND_ENV),
            "native Wayland is not experimental"
        );
    }

    #[test]
    fn wheel_units_per_notch_is_one_on_x11() {
        // Arrange / Act
        let on_x11 = wheel_units_per_notch(Backend::X11, Some(OsStr::new("wayland-1")));
        let no_session = wheel_units_per_notch(Backend::Wayland, None);
        let empty = wheel_units_per_notch(Backend::Wayland, Some(OsStr::new("")));

        // Assert: X11 reports one unit per notch, and Wayland without a
        // session falls back to X11
        assert!((on_x11 - 1.0).abs() < f32::EPSILON);
        assert!((no_session - 1.0).abs() < f32::EPSILON);
        assert!((empty - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wheel_units_per_notch_on_native_wayland() {
        // Arrange / Act
        let units = wheel_units_per_notch(Backend::Wayland, Some(OsStr::new("wayland-1")));

        // Assert
        assert!((units - WAYLAND_WHEEL_UNITS_PER_NOTCH).abs() < f32::EPSILON);
        assert!((WAYLAND_WHEEL_UNITS_PER_NOTCH - 15.0).abs() < f32::EPSILON);
    }

    proptest! {
        #[test]
        fn choose_any_value_follows_session(value in ".*", session in any::<bool>()) {
            let display = session.then_some(OsStr::new("wayland-0"));
            let choice = choose_str(&value, display);
            let lowered = value.trim().to_ascii_lowercase();
            let known = matches!(lowered.as_str(), "" | "x11" | "wayland");
            prop_assert_eq!(choice.warning.is_none(), known);
            let expected = match lowered.as_str() {
                "x11" => Backend::X11,
                "wayland" => Backend::Wayland,
                _ if session => Backend::Wayland,
                _ => Backend::X11,
            };
            prop_assert_eq!(choice.backend, expected);
            prop_assert_eq!(choice.notice, None);
        }
    }
}
