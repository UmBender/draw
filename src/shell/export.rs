//! `Ctrl+E`: the view as a PNG file in the home directory (ADR-T92-1,
//! ADR-T92-2).
//!
//! Everything here is pure or plain file I/O, so it is tested without a
//! window; `shell::app` reads the pixels and calls [`view_pixels`] and
//! [`save`]. Every failure is a value: the release build aborts on panic,
//! which would lose the drawing.

use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};

use crate::shell::diag::{self, Level};

/// Bytes of one RGBA pixel.
const BYTES_PER_PIXEL: usize = 4;

/// How many file names [`write_new`] tries before giving up.
const MAX_ATTEMPTS: u32 = 10_000;

/// Why the view was not exported.
#[derive(Debug)]
pub enum ExportError {
    /// `HOME` is unset or empty.
    NoHome,
    /// The pixels do not match the size they were said to have.
    Pixels,
    /// The PNG encoder failed.
    Encode(png::EncodingError),
    /// The file could not be created or written.
    Write {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: std::io::Error,
    },
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHome => f.write_str("HOME is not set"),
            Self::Pixels => f.write_str("the view's pixels could not be read"),
            Self::Encode(err) => write!(f, "cannot encode the PNG: {err}"),
            Self::Write { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NoHome | Self::Pixels => None,
            Self::Encode(err) => Some(err),
            Self::Write { source, .. } => Some(source),
        }
    }
}

/// Bytes of `width` × `height` RGBA pixels; `None` if that does not fit.
fn rgba_len(width: usize, height: usize) -> Option<usize> {
    width.checked_mul(height)?.checked_mul(BYTES_PER_PIXEL)
}

/// The `view` rectangle of a read-back `layer` of RGBA pixels, `margin`
/// pixels in from every side: rows top-down, every pixel opaque.
///
/// # Errors
///
/// [`ExportError::Pixels`] if `layer` is not `layer_size` or the view with
/// its margin does not fit in it.
pub fn view_pixels(
    layer: &[u8],
    layer_size: (u32, u32),
    view: (u32, u32),
    margin: u32,
) -> Result<Vec<u8>, ExportError> {
    let fits = |side: u32, layer_side: u32| {
        u64::from(side) + 2 * u64::from(margin) <= u64::from(layer_side)
    };
    if !fits(view.0, layer_size.0) || !fits(view.1, layer_size.1) {
        return Err(ExportError::Pixels);
    }
    let [layer_width, layer_height, width, height, margin] =
        [layer_size.0, layer_size.1, view.0, view.1, margin].map(|side| side as usize);
    let stride = layer_width
        .checked_mul(BYTES_PER_PIXEL)
        .filter(|stride| *stride > 0)
        .ok_or(ExportError::Pixels)?;
    if rgba_len(layer_width, layer_height) != Some(layer.len()) {
        return Err(ExportError::Pixels);
    }
    let columns = margin * BYTES_PER_PIXEL..(margin + width) * BYTES_PER_PIXEL;
    let mut pixels = Vec::with_capacity(rgba_len(width, height).ok_or(ExportError::Pixels)?);
    // Read-back gives the bottom row first.
    for row in layer.chunks_exact(stride).skip(margin).take(height).rev() {
        let row = row.get(columns.clone()).ok_or(ExportError::Pixels)?;
        let (row, _) = row.as_chunks::<BYTES_PER_PIXEL>();
        for [red, green, blue, _] in row {
            pixels.extend([*red, *green, *blue, u8::MAX]);
        }
    }
    Ok(pixels)
}

/// `rgba` as the bytes of a PNG file of `width` × `height` pixels.
///
/// # Errors
///
/// [`ExportError::Pixels`] if `rgba` has the wrong length,
/// [`ExportError::Encode`] if the encoder refuses the image.
pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, ExportError> {
    if rgba_len(width as usize, height as usize) != Some(rgba.len()) {
        return Err(ExportError::Pixels);
    }
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().map_err(ExportError::Encode)?;
    writer.write_image_data(rgba).map_err(ExportError::Encode)?;
    writer.finish().map_err(ExportError::Encode)?;
    Ok(bytes)
}

/// The home directory named by the value of `HOME`.
///
/// # Errors
///
/// [`ExportError::NoHome`] if it is unset or empty.
pub fn home_dir(value: Option<&OsStr>) -> Result<PathBuf, ExportError> {
    value
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or(ExportError::NoHome)
}

/// Where an export taken at `unix_secs` goes: `draw-<UTC date>-<time>.png`
/// in `home`, with `-<attempt>` before the extension unless `attempt` is 0.
#[must_use]
pub fn export_path(home: &Path, unix_secs: u64, attempt: u32) -> PathBuf {
    // `20261109T143005Z` to `20261109-143005`.
    let stamp = diag::file::utc_stamp(unix_secs)
        .trim_end_matches('Z')
        .replacen('T', "-", 1);
    let name = if attempt == 0 {
        format!("draw-{stamp}.png")
    } else {
        format!("draw-{stamp}-{attempt}.png")
    };
    home.join(name)
}

/// Writes `bytes` to the first free [`export_path`] in `home`, never
/// replacing a file. A write that fails half-way removes its partial file.
///
/// # Errors
///
/// [`ExportError::Write`] if no file could be created or written.
pub fn write_new(home: &Path, unix_secs: u64, bytes: &[u8]) -> Result<PathBuf, ExportError> {
    for attempt in 0..MAX_ATTEMPTS {
        let path = export_path(home, unix_secs, attempt);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                return match file.write_all(bytes) {
                    Ok(()) => Ok(path),
                    Err(source) => {
                        // A truncated PNG is worse than none.
                        let _ = std::fs::remove_file(&path);
                        Err(ExportError::Write { path, source })
                    }
                };
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
            Err(source) => return Err(ExportError::Write { path, source }),
        }
    }
    Err(ExportError::Write {
        path: export_path(home, unix_secs, MAX_ATTEMPTS),
        source: ErrorKind::AlreadyExists.into(),
    })
}

/// Encodes `rgba` of `size` pixels and writes it to the directory named by
/// `home`, the value of `HOME`. Returns the new file.
///
/// # Errors
///
/// As [`home_dir`], [`encode_png`] and [`write_new`].
pub fn save(
    rgba: &[u8],
    size: (u32, u32),
    home: Option<&OsStr>,
    unix_secs: u64,
) -> Result<PathBuf, ExportError> {
    let home = home_dir(home)?;
    let bytes = encode_png(size.0, size.1, rgba)?;
    write_new(&home, unix_secs, &bytes)
}

/// The diagnostics log's level and line for the outcome of an export.
#[must_use]
pub fn log_line(result: &Result<PathBuf, ExportError>) -> (Level, String) {
    match result {
        Ok(path) => (Level::Info, format!("view exported to {}", path.display())),
        Err(err) => (Level::Error, format!("view not exported: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A fresh, empty directory of this test run.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("draw-export-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display()));
        dir
    }

    /// `(width, height, rgba)` of PNG `bytes`.
    fn decode(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
        let mut reader = png::Decoder::new(bytes)
            .read_info()
            .unwrap_or_else(|err| panic!("{err}"));
        let mut rgba = vec![0; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut rgba)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(info.color_type, png::ColorType::Rgba);
        assert_eq!(info.bit_depth, png::BitDepth::Eight);
        rgba.truncate(info.buffer_size());
        (info.width, info.height, rgba)
    }

    /// RGBA pixels whose red channel is `10 * row + column`, so each pixel
    /// says where it came from.
    fn numbered(width: u32, height: u32) -> Vec<u8> {
        let mut rgba = Vec::new();
        for row in 0..height {
            for column in 0..width {
                rgba.extend([(10 * row + column) as u8, 1, 2, 255]);
            }
        }
        rgba
    }

    /// The red channel of `rgba`.
    fn reds(rgba: &[u8]) -> Vec<u8> {
        rgba.iter().step_by(4).copied().collect()
    }

    fn files_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("{err}"))
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    // ---- AC-1 file name ----------------------------------------------------

    #[test]
    fn export_path_uses_the_utc_date_and_time() {
        // Arrange: 2026-11-09 14:30:05 UTC.
        let unix = 1_794_234_605;

        // Act
        let path = export_path(Path::new("/home/me"), unix, 0);

        // Assert
        assert_eq!(path, Path::new("/home/me/draw-20261109-143005.png"));
    }

    // ---- AC-2 no overwrite -------------------------------------------------

    #[test]
    fn export_path_adds_the_attempt_as_a_suffix() {
        assert_eq!(
            export_path(Path::new("/h"), 0, 1),
            Path::new("/h/draw-19700101-000000-1.png")
        );
        assert_eq!(
            export_path(Path::new("/h"), 0, 2),
            Path::new("/h/draw-19700101-000000-2.png")
        );
    }

    #[test]
    fn write_new_existing_file_takes_the_next_suffix() {
        // Arrange
        let dir = temp_dir("suffix");

        // Act
        let paths: Vec<PathBuf> = (0..3)
            .map(|_| write_new(&dir, 0, b"png").unwrap_or_else(|err| panic!("{err}")))
            .collect();

        // Assert
        assert_eq!(
            paths,
            [
                dir.join("draw-19700101-000000.png"),
                dir.join("draw-19700101-000000-1.png"),
                dir.join("draw-19700101-000000-2.png"),
            ]
        );
    }

    #[test]
    fn write_new_leaves_existing_files_untouched() {
        // Arrange
        let dir = temp_dir("untouched");
        let taken = dir.join("draw-19700101-000000.png");
        std::fs::write(&taken, b"earlier").unwrap_or_else(|err| panic!("{err}"));

        // Act
        let path = write_new(&dir, 0, b"later").unwrap_or_else(|err| panic!("{err}"));

        // Assert
        assert_eq!(std::fs::read(&taken).ok().as_deref(), Some(&b"earlier"[..]));
        assert_eq!(std::fs::read(&path).ok().as_deref(), Some(&b"later"[..]));
    }

    // ---- AC-3 failures -----------------------------------------------------

    #[test]
    fn home_dir_unset_or_empty_is_an_error() {
        assert!(matches!(home_dir(None), Err(ExportError::NoHome)));
        assert!(matches!(
            home_dir(Some(OsStr::new(""))),
            Err(ExportError::NoHome)
        ));
    }

    #[test]
    fn home_dir_set_is_that_directory() {
        let home = home_dir(Some(OsStr::new("/home/me"))).ok();

        assert_eq!(home.as_deref(), Some(Path::new("/home/me")));
    }

    #[test]
    fn write_new_in_an_unusable_directory_is_an_error() {
        // Arrange: a file where the directory should be fails for root too.
        let dir = temp_dir("unusable");
        let file = dir.join("not-a-directory");
        std::fs::write(&file, b"").unwrap_or_else(|err| panic!("{err}"));

        // Act
        let result = write_new(&file, 0, b"png");

        // Assert
        assert!(
            matches!(result, Err(ExportError::Write { .. })),
            "{result:?}"
        );
        assert_eq!(files_in(&dir), ["not-a-directory"]);
    }

    #[test]
    fn save_without_home_is_an_error() {
        // Arrange
        let rgba = numbered(2, 2);

        // Act
        let result = save(&rgba, (2, 2), None, 0);

        // Assert
        assert!(matches!(result, Err(ExportError::NoHome)), "{result:?}");
    }

    #[test]
    fn save_writes_a_decodable_file_in_home() {
        // Arrange
        let dir = temp_dir("save");
        let rgba = numbered(3, 2);

        // Act
        let path =
            save(&rgba, (3, 2), Some(dir.as_os_str()), 0).unwrap_or_else(|err| panic!("{err}"));

        // Assert
        let bytes = std::fs::read(&path).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(decode(&bytes), (3, 2, rgba));
    }

    #[test]
    fn log_line_success_is_info_with_the_path() {
        // Arrange
        let result = Ok(PathBuf::from("/home/me/draw-1.png"));

        // Act
        let (level, line) = log_line(&result);

        // Assert
        assert_eq!(level, Level::Info);
        assert_eq!(line, "view exported to /home/me/draw-1.png");
    }

    #[test]
    fn log_line_failure_is_an_error_with_the_reason() {
        // Arrange
        let result = Err(ExportError::NoHome);

        // Act
        let (level, line) = log_line(&result);

        // Assert
        assert_eq!(level, Level::Error);
        assert_eq!(line, "view not exported: HOME is not set");
    }

    #[test]
    fn write_error_names_the_path_and_the_cause() {
        // Arrange
        let err = ExportError::Write {
            path: PathBuf::from("/ro/draw.png"),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        };

        // Act
        let text = err.to_string();

        // Assert
        assert!(text.starts_with("/ro/draw.png: "), "{text}");
        assert!(text.contains("denied"), "{text}");
    }

    // ---- AC-4 rows ---------------------------------------------------------

    #[test]
    fn encode_png_round_trips_rows_in_order() {
        // Arrange
        let rgba = numbered(4, 3);

        // Act
        let bytes = encode_png(4, 3, &rgba).unwrap_or_else(|err| panic!("{err}"));

        // Assert
        assert_eq!(decode(&bytes), (4, 3, rgba));
    }

    #[test]
    fn encode_png_wrong_length_is_an_error() {
        // Arrange: one byte short.
        let rgba = vec![0; 4 * 2 * 2 - 1];

        // Act
        let result = encode_png(2, 2, &rgba);

        // Assert
        assert!(matches!(result, Err(ExportError::Pixels)), "{result:?}");
    }

    #[test]
    fn encode_png_zero_size_is_an_error() {
        assert!(encode_png(0, 0, &[]).is_err());
        assert!(encode_png(0, 5, &[]).is_err());
    }

    #[test]
    fn view_pixels_puts_the_rows_top_down() {
        // Arrange: read back, the first row is the bottom of the view.
        let layer = numbered(2, 3);

        // Act
        let view = view_pixels(&layer, (2, 3), (2, 3), 0).unwrap_or_else(|err| panic!("{err}"));

        // Assert
        assert_eq!(reds(&view), [20, 21, 10, 11, 0, 1]);
    }

    #[test]
    fn view_pixels_crops_the_margin() {
        // Arrange: a 2×1 view with a margin of 1 on every side.
        let layer = numbered(4, 3);

        // Act
        let view = view_pixels(&layer, (4, 3), (2, 1), 1).unwrap_or_else(|err| panic!("{err}"));

        // Assert
        assert_eq!(reds(&view), [11, 12]);
    }

    #[test]
    fn view_pixels_makes_every_pixel_opaque() {
        // Arrange
        let layer = [9, 8, 7, 0, 6, 5, 4, 128];

        // Act
        let view = view_pixels(&layer, (2, 1), (2, 1), 0).unwrap_or_else(|err| panic!("{err}"));

        // Assert
        assert_eq!(view, [9, 8, 7, 255, 6, 5, 4, 255]);
    }

    #[test]
    fn view_pixels_wrong_length_is_an_error() {
        // Arrange: three bytes where a 1×1 layer has four.
        let layer = [0; 3];

        // Act
        let result = view_pixels(&layer, (1, 1), (1, 1), 0);

        // Assert
        assert!(matches!(result, Err(ExportError::Pixels)), "{result:?}");
    }

    #[test]
    fn view_pixels_view_larger_than_layer_is_an_error() {
        // Arrange
        let layer = numbered(2, 2);

        // Act
        let wide = view_pixels(&layer, (2, 2), (3, 2), 0);
        let margin = view_pixels(&layer, (2, 2), (2, 2), 1);
        let huge = view_pixels(&layer, (2, 2), (u32::MAX, 1), u32::MAX);

        // Assert
        for result in [wide, margin, huge] {
            assert!(matches!(result, Err(ExportError::Pixels)), "{result:?}");
        }
    }
}
