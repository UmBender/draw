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
        assert_eq!(crash_log_path(Some(OsStr::new("rel")), Some(OsStr::new("rel"))), None);
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
