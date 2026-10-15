//! Packaging checks that need no packaging tools (T27): the library list
//! matches the binary and the package metadata, and the desktop files
//! match the window.

use std::collections::BTreeSet;
use std::path::Path;

use draw::shell::app::WM_CLASS;
use draw::shell::preflight::{RUNTIME_LIBS, Role, RuntimeLib, parse_runtime_libs};

const APP_ID: &str = "io.github.UmBender.draw";

fn read(path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn runtime_libs() -> Vec<RuntimeLib> {
    parse_runtime_libs(RUNTIME_LIBS).unwrap_or_else(|err| panic!("runtime-libs: {err}"))
}

/// Packages a package manager must install: `required` and `gl` roles.
fn declared(pick: fn(&RuntimeLib) -> &str) -> BTreeSet<String> {
    runtime_libs()
        .iter()
        .filter(|lib| matches!(lib.role, Role::Required | Role::Gl))
        .map(|lib| pick(lib).to_owned())
        .collect()
}

/// Lines of the TOML table `[name]` up to the next table header.
fn toml_table<'a>(text: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    text.lines()
        .skip_while(|line| line.trim() != header)
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// The part of a soname before `.so`: `libX11.so.6` → `libX11`.
fn soname_base(soname: &str) -> &str {
    soname.find(".so").map_or(soname, |end| &soname[..end])
}

/// Every `lib<name>.so` string in `bytes`, as its base `lib<name>`.
fn library_bases(bytes: &[u8]) -> BTreeSet<String> {
    let is_name = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'-');
    let mut bases = BTreeSet::new();
    let mut i = 0;
    while let Some(offset) = bytes[i..].windows(3).position(|w| w == b"lib") {
        let start = i + offset;
        let mut end = start + 3;
        while end < bytes.len() && is_name(bytes[end]) && !bytes[end..].starts_with(b".so") {
            end += 1;
        }
        if end > start + 3 && bytes[end..].starts_with(b".so") {
            bases.insert(String::from_utf8_lossy(&bytes[start..end]).into_owned());
        }
        i = start + 3;
    }
    bases
}

#[test]
fn library_bases_finds_packed_strings() {
    // Arrange
    let bytes = b"\0libX11.solibX11.so.6Xfoo libGL.so.1X11 library libxkbcommon.so.0.0.0";

    // Act
    let bases = library_bases(bytes);

    // Assert
    let expected: BTreeSet<String> = ["libGL", "libX11", "libxkbcommon"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_eq!(bases, expected);
}

#[test]
fn runtime_libs_names_every_library_the_binary_mentions() {
    // Arrange
    let binary = std::fs::read(env!("CARGO_BIN_EXE_draw")).unwrap_or_default();
    let listed: BTreeSet<&str> = RUNTIME_LIBS
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|word| !word.starts_with('#'))
        .map(soname_base)
        .collect();

    // Act
    let mentioned = library_bases(&binary);

    // Assert
    assert!(
        mentioned.contains("libX11"),
        "scan found nothing: {mentioned:?}"
    );
    let unlisted: Vec<&String> = mentioned
        .iter()
        .filter(|base| !listed.contains(base.as_str()))
        .collect();
    assert!(
        unlisted.is_empty(),
        "add these to packaging/runtime-libs: {unlisted:?}"
    );
}

#[test]
fn deb_depends_match_runtime_libs() {
    // Arrange
    let cargo = read("Cargo.toml");
    let table = toml_table(&cargo, "package.metadata.deb");
    let depends = table
        .iter()
        .find_map(|line| line.strip_prefix("depends = "))
        .unwrap_or_default()
        .trim_matches('"');

    // Act
    let packages: BTreeSet<String> = depends
        .split(',')
        .map(|dep| dep.trim().to_owned())
        .filter(|dep| !dep.is_empty())
        .collect();

    // Assert
    assert_eq!(packages, declared(|lib| &lib.debian));
}

#[test]
fn rpm_requires_match_runtime_libs() {
    // Arrange
    let cargo = read("Cargo.toml");
    let table = toml_table(&cargo, "package.metadata.generate-rpm.requires");

    // Act
    let packages: BTreeSet<String> = table
        .iter()
        .filter_map(|line| line.split_once('='))
        .map(|(name, _)| name.trim().to_owned())
        .collect();

    // Assert
    assert_eq!(packages, declared(|lib| &lib.fedora));
}

#[test]
fn desktop_file_matches_wm_class_and_icon() {
    // Arrange
    let desktop = read(&format!("packaging/{APP_ID}.desktop"));

    // Act
    let entry: Vec<(&str, &str)> = desktop
        .lines()
        .skip_while(|line| line.trim() != "[Desktop Entry]")
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter_map(|line| line.split_once('='))
        .collect();
    let get = |key: &str| entry.iter().find(|(k, _)| *k == key).map(|(_, v)| *v);

    // Assert
    assert_eq!(get("Type"), Some("Application"));
    assert_eq!(get("Exec"), Some("draw"));
    assert_eq!(get("Icon"), Some(APP_ID));
    assert_eq!(get("StartupWMClass"), Some(WM_CLASS));
    assert!(get("Categories").unwrap_or_default().contains("Graphics"));
}

#[test]
fn metainfo_release_is_crate_version() {
    // Arrange
    let metainfo = read(&format!("packaging/{APP_ID}.metainfo.xml"));

    // Act
    let first_release = metainfo
        .split("<release ")
        .nth(1)
        .and_then(|rest| rest.split("version=\"").nth(1))
        .and_then(|rest| rest.split('"').next());

    // Assert
    assert!(metainfo.contains(&format!("<id>{APP_ID}</id>")));
    assert!(metainfo.contains(&format!(
        "<launchable type=\"desktop-id\">{APP_ID}.desktop</launchable>"
    )));
    assert_eq!(first_release, Some(env!("CARGO_PKG_VERSION")));
}

#[test]
fn icons_have_every_size() {
    // Arrange
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("packaging/icons");

    // Act / Assert
    let svg = std::fs::read_to_string(dir.join("draw.svg")).unwrap_or_default();
    assert!(svg.contains("<svg"), "draw.svg is not an SVG");
    for size in [48_u32, 128, 256] {
        let png = std::fs::read(dir.join(format!("draw-{size}.png"))).unwrap_or_default();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "draw-{size}.png");
        let dimension = |at: usize| {
            png.get(at..at + 4)
                .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        };
        assert_eq!(dimension(16), Some(size), "draw-{size}.png width");
        assert_eq!(dimension(20), Some(size), "draw-{size}.png height");
    }
}
