#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    const LIBS: &str = "\
# soname            role      fedora          debian
libX11.so.6          required  libX11          libx11-6
libXi.so.6           required  libXi           libxi6
libGL.so.1           gl        libglvnd-glx    libgl1
libEGL.so.1          gl        libglvnd-egl    libegl1

libwayland-client.so.0 unused  libwayland-client libwayland-client0
libc.so.6            linked    glibc           libc6
";

    fn libs() -> Vec<RuntimeLib> {
        parse_runtime_libs(LIBS).unwrap_or_default()
    }

    fn x11() -> Environment {
        Environment {
            display: Some(OsString::from(":0")),
            wayland_display: None,
            skip: None,
        }
    }

    fn all_found(_: &str) -> bool {
        true
    }

    #[test]
    fn parse_runtime_libs_reads_roles_and_packages() {
        // Arrange / Act
        let libs = parse_runtime_libs(LIBS);

        // Assert
        let libs = libs.unwrap_or_default();
        assert_eq!(libs.len(), 6);
        assert_eq!(
            libs[0],
            RuntimeLib {
                soname: "libX11.so.6".to_owned(),
                role: Role::Required,
                fedora: "libX11".to_owned(),
                debian: "libx11-6".to_owned(),
            }
        );
        assert_eq!(libs[2].role, Role::Gl);
        assert_eq!(libs[4].role, Role::Unused);
        assert_eq!(libs[5].role, Role::Linked);
    }

    #[test]
    fn parse_runtime_libs_rejects_bad_lines() {
        // Arrange
        let cases = [
            "libX11.so.6 required libX11",
            "libX11.so.6 sometimes libX11 libx11-6",
            "X11.so.6 required libX11 libx11-6",
            "libX11.so.6 required libX11 libx11-6 extra",
        ];

        for text in cases {
            // Act
            let parsed = parse_runtime_libs(text);

            // Assert
            assert_eq!(parsed.map_err(|err| err.line), Err(1), "{text}");
        }
    }

    #[test]
    fn embedded_runtime_libs_parse() {
        // Arrange / Act
        let libs = parse_runtime_libs(RUNTIME_LIBS);

        // Assert
        let libs = libs.unwrap_or_default();
        for soname in ["libX11.so.6", "libXi.so.6", "libxkbcommon.so.0"] {
            assert!(
                libs.iter()
                    .any(|lib| lib.soname == soname && lib.role == Role::Required),
                "{soname}"
            );
        }
        for soname in ["libGL.so.1", "libEGL.so.1"] {
            assert!(
                libs.iter()
                    .any(|lib| lib.soname == soname && lib.role == Role::Gl),
                "{soname}"
            );
        }
    }

    #[test]
    fn check_no_display_reports_no_display() {
        // Arrange
        let env = Environment {
            display: None,
            wayland_display: Some(OsString::new()),
            skip: None,
        };

        // Act
        let result = check(&env, &libs(), all_found);

        // Assert
        assert_eq!(result, Err(Problem::NoDisplay));
    }

    #[test]
    fn check_wayland_without_display_reports_xwayland() {
        // Arrange
        let env = Environment {
            display: None,
            wayland_display: Some(OsString::from("wayland-0")),
            skip: None,
        };

        // Act
        let result = check(&env, &libs(), all_found);

        // Assert
        assert_eq!(result, Err(Problem::NoXWayland));
    }

    #[test]
    fn check_x11_display_with_all_libs_is_ok() {
        // Arrange
        let mut env = x11();
        env.wayland_display = Some(OsString::from("wayland-0"));

        // Act
        let result = check(&env, &libs(), all_found);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn check_missing_required_lib_names_its_packages() {
        // Arrange
        let found = |soname: &str| soname != "libXi.so.6";

        // Act
        let result = check(&x11(), &libs(), found);

        // Assert
        let Err(Problem::MissingLibs(missing)) = result else {
            panic!("expected missing libraries, got {result:?}");
        };
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].soname, "libXi.so.6");
        let message = Problem::MissingLibs(missing).to_string();
        assert!(message.contains("libXi.so.6"), "{message}");
        assert!(message.contains("libXi"), "{message}");
        assert!(message.contains("libxi6"), "{message}");
    }

    #[test]
    fn check_one_gl_lib_is_enough() {
        // Arrange
        let found = |soname: &str| soname != "libGL.so.1";

        // Act
        let result = check(&x11(), &libs(), found);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn check_no_gl_lib_names_both() {
        // Arrange
        let found = |soname: &str| !matches!(soname, "libGL.so.1" | "libEGL.so.1");

        // Act
        let result = check(&x11(), &libs(), found);

        // Assert
        let Err(Problem::MissingLibs(missing)) = result else {
            panic!("expected missing libraries, got {result:?}");
        };
        let names: Vec<&str> = missing.iter().map(|lib| lib.soname.as_str()).collect();
        assert_eq!(names, ["libGL.so.1", "libEGL.so.1"]);
    }

    #[test]
    fn check_unused_libs_are_not_needed() {
        // Arrange
        let found = |soname: &str| !matches!(soname, "libwayland-client.so.0" | "libc.so.6");

        // Act
        let result = check(&x11(), &libs(), found);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn check_skip_variable_skips_everything() {
        // Arrange
        let env = Environment {
            display: None,
            wayland_display: None,
            skip: Some(OsString::from("1")),
        };

        // Act
        let result = check(&env, &libs(), |_| false);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn cache_contains_soname_matches_whole_names_only() {
        // Arrange
        let cache = b"glibc-ld.so.cache1.1\0libEGL.so.1\0/usr/lib64/libX11.so.6\0libXi.so.6.1\0";

        // Act / Assert
        assert!(cache_contains_soname(cache, "libEGL.so.1"));
        assert!(cache_contains_soname(cache, "libX11.so.6"));
        assert!(!cache_contains_soname(cache, "libGL.so.1"));
        assert!(!cache_contains_soname(cache, "libXi.so.6"));
        assert!(!cache_contains_soname(cache, ""));
    }

    #[test]
    fn problem_message_is_one_line() {
        // Arrange
        let problems = [
            Problem::NoDisplay,
            Problem::NoXWayland,
            Problem::MissingLibs(libs().into_iter().take(4).collect()),
        ];

        for problem in problems {
            // Act
            let message = problem.to_string();

            // Assert
            assert!(!message.is_empty());
            assert!(!message.contains('\n'), "{message}");
        }
    }
}
