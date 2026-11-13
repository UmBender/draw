//! A private X server for the GLX tests: `Xvfb -displayfd`, killed on drop.
//! A missing `Xvfb` fails the test, it never skips it (T145).

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

/// A running `Xvfb`.
pub struct Xvfb {
    child: Child,
    /// The display name, for example `:7`.
    pub display: String,
}

impl Xvfb {
    /// Starts `Xvfb` on a free display number.
    ///
    /// # Panics
    /// If `Xvfb` is not installed or does not report a display.
    pub fn start() -> Self {
        let mut child = Command::new("Xvfb")
            .args([
                "-displayfd",
                "1",
                "-screen",
                "0",
                "64x64x24",
                "-nolisten",
                "tcp",
                "-noreset",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| {
                panic!("Xvfb is required for the GLX tests (package xvfb or xorg-x11-server-Xvfb): {e}")
            });
        let stdout = child.stdout.take().expect("piped stdout");
        let mut line = String::new();
        BufReader::new(stdout)
            .read_line(&mut line)
            .expect("Xvfb reports its display");
        let number = line.trim();
        assert!(!number.is_empty(), "Xvfb did not report a display number");
        Self {
            child,
            display: format!(":{number}"),
        }
    }
}

impl Drop for Xvfb {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
