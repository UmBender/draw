//! Crash report: a panic hook that records what crashed before the
//! abort (T27, ADR-T27-3).
//!
//! The release build aborts on panic and ships stripped, so the hook
//! writes the panic message, its source location, the app version and
//! the GNU build id (which pairs the binary with its `draw.debug`) to
//! stderr and appends them to [`crash_log_path`]. Everything but the hook
//! itself is a pure function.

use std::any::Any;
use std::backtrace::{Backtrace, BacktraceStatus};
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// What one crash report says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report<'a> {
    /// The panic message.
    pub message: &'a str,
    /// `file:line:column` of the panic, when known.
    pub location: Option<String>,
    /// The crate version.
    pub version: &'a str,
    /// GNU build id of the running binary, in lowercase hex.
    pub build_id: Option<String>,
    /// Seconds since the Unix epoch.
    pub unix_time: u64,
    /// A captured backtrace, when `RUST_BACKTRACE` asks for one.
    pub backtrace: Option<String>,
}

/// ELF note type of a GNU build id (`NT_GNU_BUILD_ID`).
const NT_GNU_BUILD_ID: u32 = 3;

/// ELF program header type of a note segment.
const PT_NOTE: u32 = 4;

/// Where the crash log goes: `$XDG_STATE_HOME/draw/crash.log` when that is
/// an absolute path, else `$HOME/.local/state/draw/crash.log`; `None`
/// without an absolute home either.
#[must_use]
pub fn crash_log_path(xdg_state_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    fn absolute(dir: Option<&OsStr>) -> Option<&Path> {
        dir.map(Path::new).filter(|dir| dir.is_absolute())
    }
    absolute(xdg_state_home)
        .map(Path::to_path_buf)
        .or_else(|| absolute(home).map(|home| home.join(".local/state")))
        .map(|state| state.join("draw/crash.log"))
}

/// The report as text, one `key: value` line per field, ending in a
/// newline.
#[must_use]
pub fn format_report(report: &Report<'_>) -> String {
    let unknown = |field: &Option<String>| field.clone().unwrap_or_else(|| "unknown".to_owned());
    let mut text = String::new();
    // Writing to a String cannot fail.
    let _ = write!(
        text,
        "draw crashed\nmessage: {}\nlocation: {}\nversion: {}\nbuild id: {}\nunix time: {}\n",
        report.message,
        unknown(&report.location),
        report.version,
        unknown(&report.build_id),
        report.unix_time,
    );
    if let Some(backtrace) = &report.backtrace {
        let _ = writeln!(text, "backtrace:\n{}", backtrace.trim_end());
    }
    text
}

/// The message of a panic payload (`&str` or `String`).
#[must_use]
pub fn panic_message(payload: &(dyn Any + Send)) -> &str {
    if let Some(message) = payload.downcast_ref::<&str>() {
        message
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message
    } else {
        "Box<dyn Any>"
    }
}

/// `N` bytes of `bytes` at `at`, if they are all there.
fn bytes_at<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    bytes.get(at..at.checked_add(N)?)?.try_into().ok()
}

fn u16_at(bytes: &[u8], at: usize) -> Option<usize> {
    bytes_at(bytes, at).map(|b| usize::from(u16::from_le_bytes(b)))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    bytes_at(bytes, at).map(u32::from_le_bytes)
}

fn u64_at(bytes: &[u8], at: usize) -> Option<usize> {
    bytes_at(bytes, at).and_then(|b| usize::try_from(u64::from_le_bytes(b)).ok())
}

/// `len` rounded up to a multiple of `align` (a power of two or 0/1).
fn padded(len: usize, align: usize) -> Option<usize> {
    if align <= 1 {
        Some(len)
    } else {
        len.checked_next_multiple_of(align)
    }
}

/// The build id among the notes of one `PT_NOTE` segment.
fn build_id_in_notes(notes: &[u8], align: usize) -> Option<String> {
    let mut at = 0;
    while at < notes.len() {
        let name_len = usize::try_from(u32_at(notes, at)?).ok()?;
        let desc_len = usize::try_from(u32_at(notes, at + 4)?).ok()?;
        let kind = u32_at(notes, at + 8)?;
        let name_at = at + 12;
        let desc_at = name_at.checked_add(padded(name_len, align)?)?;
        let next = desc_at.checked_add(padded(desc_len, align)?)?;
        let name = notes.get(name_at..name_at.checked_add(name_len)?)?;
        let desc = notes.get(desc_at..desc_at.checked_add(desc_len)?)?;
        if name == b"GNU\0" && kind == NT_GNU_BUILD_ID && !desc.is_empty() {
            return Some(desc.iter().fold(String::new(), |mut hex, byte| {
                let _ = write!(hex, "{byte:02x}");
                hex
            }));
        }
        at = next;
    }
    None
}

/// The GNU build id of a 64-bit little-endian ELF file, read from its
/// `PT_NOTE` segments, in lowercase hex. `None` for anything else.
#[must_use]
pub fn build_id(elf: &[u8]) -> Option<String> {
    if elf.get(..6)? != b"\x7fELF\x02\x01" {
        return None;
    }
    let ph_off = u64_at(elf, 32)?;
    let ph_size = u16_at(elf, 54)?;
    let ph_count = u16_at(elf, 56)?;
    (0..ph_count).find_map(|index| {
        let header = ph_off.checked_add(index.checked_mul(ph_size)?)?;
        if u32_at(elf, header)? != PT_NOTE {
            return None;
        }
        let offset = u64_at(elf, header + 8)?;
        let size = u64_at(elf, header + 32)?;
        let align = u64_at(elf, header + 48)?.max(4);
        let notes = elf.get(offset..offset.checked_add(size)?)?;
        build_id_in_notes(notes, align)
    })
}

/// Installs the panic hook. Call first in `main`.
pub fn install_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|at| format!("{}:{}:{}", at.file(), at.line(), at.column()));
        let build_id = std::fs::read("/proc/self/exe")
            .ok()
            .and_then(|exe| build_id(&exe));
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        let backtrace = Backtrace::capture();
        let backtrace =
            (backtrace.status() == BacktraceStatus::Captured).then(|| backtrace.to_string());
        let text = format_report(&Report {
            message: panic_message(info.payload()),
            location,
            version: env!("CARGO_PKG_VERSION"),
            build_id,
            unix_time,
            backtrace,
        });
        // The hook must not panic: every write failure is ignored.
        let mut stderr = std::io::stderr().lock();
        let _ = stderr.write_all(text.as_bytes());
        let path = crash_log_path(
            std::env::var_os("XDG_STATE_HOME").as_deref(),
            std::env::var_os("HOME").as_deref(),
        );
        if let Some(path) = path {
            if append(&path, &text).is_ok() {
                let _ = writeln!(stderr, "crash report appended to {}", path.display());
            }
        }
    }));
}

/// Appends `text` to the file at `path`, creating it and its directory.
fn append(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(format!("{text}\n").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::path::PathBuf;

    /// A minimal 64-bit little-endian ELF header with one `PT_NOTE`
    /// program header pointing at `notes`.
    fn elf_with_notes(notes: &[u8]) -> Vec<u8> {
        let ph_off: u64 = 64;
        let note_off: u64 = 64 + 56;
        let mut elf = Vec::new();
        elf.extend_from_slice(b"\x7fELF");
        elf.extend_from_slice(&[2, 1, 1, 0]); // 64-bit, little endian, version 1
        elf.extend_from_slice(&[0; 8]);
        elf.extend_from_slice(&3u16.to_le_bytes()); // e_type: shared object
        elf.extend_from_slice(&62u16.to_le_bytes()); // e_machine: x86-64
        elf.extend_from_slice(&1u32.to_le_bytes()); // e_version
        elf.extend_from_slice(&0u64.to_le_bytes()); // e_entry
        elf.extend_from_slice(&ph_off.to_le_bytes()); // e_phoff
        elf.extend_from_slice(&0u64.to_le_bytes()); // e_shoff
        elf.extend_from_slice(&0u32.to_le_bytes()); // e_flags
        elf.extend_from_slice(&64u16.to_le_bytes()); // e_ehsize
        elf.extend_from_slice(&56u16.to_le_bytes()); // e_phentsize
        elf.extend_from_slice(&1u16.to_le_bytes()); // e_phnum
        elf.extend_from_slice(&64u16.to_le_bytes()); // e_shentsize
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shnum
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shstrndx
        assert_eq!(elf.len(), 64);
        elf.extend_from_slice(&4u32.to_le_bytes()); // p_type: PT_NOTE
        elf.extend_from_slice(&4u32.to_le_bytes()); // p_flags
        elf.extend_from_slice(&note_off.to_le_bytes()); // p_offset
        elf.extend_from_slice(&note_off.to_le_bytes()); // p_vaddr
        elf.extend_from_slice(&note_off.to_le_bytes()); // p_paddr
        elf.extend_from_slice(&(notes.len() as u64).to_le_bytes()); // p_filesz
        elf.extend_from_slice(&(notes.len() as u64).to_le_bytes()); // p_memsz
        elf.extend_from_slice(&4u64.to_le_bytes()); // p_align
        elf.extend_from_slice(notes);
        elf
    }

    fn note(name: &[u8], kind: u32, desc: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(name.len() as u32).to_le_bytes());
        out.extend_from_slice(&(desc.len() as u32).to_le_bytes());
        out.extend_from_slice(&kind.to_le_bytes());
        out.extend_from_slice(name);
        out.resize(out.len().next_multiple_of(4), 0);
        out.extend_from_slice(desc);
        out.resize(out.len().next_multiple_of(4), 0);
        out
    }

    #[test]
    fn crash_log_path_prefers_absolute_xdg_state_home() {
        // Arrange / Act
        let path = crash_log_path(Some(OsStr::new("/state")), Some(OsStr::new("/home/u")));

        // Assert
        assert_eq!(path, Some(PathBuf::from("/state/draw/crash.log")));
    }

    #[test]
    fn crash_log_path_ignores_relative_or_empty_xdg_state_home() {
        // Arrange
        let expected = Some(PathBuf::from("/home/u/.local/state/draw/crash.log"));

        for xdg in [Some(OsStr::new("state")), Some(OsStr::new("")), None] {
            // Act
            let path = crash_log_path(xdg, Some(OsStr::new("/home/u")));

            // Assert
            assert_eq!(path, expected, "{xdg:?}");
        }
    }

    #[test]
    fn crash_log_path_without_home_is_none() {
        // Arrange / Act / Assert
        assert_eq!(crash_log_path(None, None), None);
        assert_eq!(crash_log_path(None, Some(OsStr::new(""))), None);
        assert_eq!(
            crash_log_path(Some(OsStr::new("rel")), Some(OsStr::new("rel"))),
            None
        );
    }

    #[test]
    fn format_report_has_message_location_version_and_build_id() {
        // Arrange
        let report = Report {
            message: "index out of bounds",
            location: Some("src/core/shape.rs:12:5".to_owned()),
            version: "1.2.3",
            build_id: Some("abcdef".to_owned()),
            unix_time: 1_760_000_000,
            backtrace: None,
        };

        // Act
        let text = format_report(&report);

        // Assert
        assert!(text.contains("index out of bounds"), "{text}");
        assert!(text.contains("src/core/shape.rs:12:5"), "{text}");
        assert!(text.contains("1.2.3"), "{text}");
        assert!(text.contains("abcdef"), "{text}");
        assert!(text.contains("1760000000"), "{text}");
        assert!(text.ends_with('\n'), "{text}");
    }

    #[test]
    fn format_report_marks_unknown_fields() {
        // Arrange
        let report = Report {
            message: "boom",
            location: None,
            version: "1.2.3",
            build_id: None,
            unix_time: 0,
            backtrace: Some("0: main".to_owned()),
        };

        // Act
        let text = format_report(&report);

        // Assert
        assert!(text.contains("unknown"), "{text}");
        assert!(text.contains("0: main"), "{text}");
    }

    #[test]
    fn panic_message_reads_str_and_string_payloads() {
        // Arrange
        let static_str: Box<dyn std::any::Any + Send> = Box::new("static");
        let string: Box<dyn std::any::Any + Send> = Box::new(String::from("owned"));
        let other: Box<dyn std::any::Any + Send> = Box::new(42_u8);

        // Act / Assert
        assert_eq!(panic_message(static_str.as_ref()), "static");
        assert_eq!(panic_message(string.as_ref()), "owned");
        assert_eq!(panic_message(other.as_ref()), "Box<dyn Any>");
    }

    #[test]
    fn build_id_reads_gnu_note() {
        // Arrange
        let mut notes = note(b"XYZ\0", 3, &[9, 9]);
        notes.extend(note(b"GNU\0", 1, &[0; 16]));
        notes.extend(note(b"GNU\0", 3, &[0xde, 0xad, 0xbe, 0xef, 0x01]));
        let elf = elf_with_notes(&notes);

        // Act
        let id = build_id(&elf);

        // Assert
        assert_eq!(id.as_deref(), Some("deadbeef01"));
    }

    #[test]
    fn build_id_of_elf_without_note_is_none() {
        // Arrange
        let elf = elf_with_notes(&note(b"GNU\0", 1, &[0; 16]));

        // Act / Assert
        assert_eq!(build_id(&elf), None);
    }

    #[test]
    fn build_id_of_garbage_is_none() {
        // Arrange
        let mut truncated = elf_with_notes(&note(b"GNU\0", 3, &[1, 2, 3, 4]));
        truncated.truncate(100);
        let mut huge_note = elf_with_notes(&note(b"GNU\0", 3, &[1, 2, 3, 4]));
        huge_note[64 + 56 + 4..64 + 56 + 8].copy_from_slice(&u32::MAX.to_le_bytes());
        let cases: [&[u8]; 4] = [b"", b"\x7fELF", b"not an elf file at all", &truncated];

        for bytes in cases.into_iter().chain([huge_note.as_slice()]) {
            // Act / Assert
            assert_eq!(build_id(bytes), None);
        }
    }
}
