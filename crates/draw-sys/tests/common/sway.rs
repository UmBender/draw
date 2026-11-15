//! A private headless sway for the Wayland tests (T168): its own
//! `XDG_RUNTIME_DIR`, `WLR_BACKENDS=headless`, `WLR_RENDERER=pixman`. A
//! missing `sway` fails the test, it never skips it.

#![allow(clippy::expect_used, dead_code)]

use std::fs;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

static COUNTER: AtomicU32 = AtomicU32::new(0);

const CONFIG: &str = "output HEADLESS-1 resolution 800x600 position 0 0 scale 1\n\
default_border none\ndefault_floating_border none\nfocus_follows_mouse no\nxwayland disable\n";

/// A running headless sway.
pub struct Sway {
    child: Child,
    run: PathBuf,
    /// The Wayland socket of this sway.
    pub socket: PathBuf,
    ipc: PathBuf,
}

impl Sway {
    /// Starts sway with one 800x600 output at scale 1.
    ///
    /// # Panics
    /// If sway is missing or does not come up in ten seconds.
    pub fn start() -> Self {
        let run = std::env::temp_dir().join(format!(
            "draw-sway-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&run)
            .expect("private runtime dir");
        let config = run.join("sway.conf");
        fs::write(&config, CONFIG).expect("sway config");
        let child = Command::new("sway")
            .arg("-c")
            .arg(&config)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("SWAYSOCK")
            .env_remove("I3SOCK")
            .env("XDG_RUNTIME_DIR", &run)
            .env("WLR_BACKENDS", "headless")
            .env("WLR_HEADLESS_OUTPUTS", "1")
            .env("WLR_RENDERER", "pixman")
            .env("WLR_LIBINPUT_NO_DEVICES", "1")
            .env("LIBGL_ALWAYS_SOFTWARE", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("sway is required for the Wayland tests: {e}"));
        let mut sway = Self {
            child,
            run,
            socket: PathBuf::new(),
            ipc: PathBuf::new(),
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        while sway.socket.as_os_str().is_empty() || sway.ipc.as_os_str().is_empty() {
            assert!(Instant::now() < deadline, "sway did not start");
            assert!(
                sway.child.try_wait().expect("wait").is_none(),
                "sway exited early"
            );
            std::thread::sleep(Duration::from_millis(50));
            sway.look_for_sockets();
        }
        sway
    }

    fn look_for_sockets(&mut self) {
        let Ok(entries) = fs::read_dir(&self.run) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_socket = entry.file_type().is_ok_and(|t| t.is_socket());
            if !is_socket {
                continue;
            }
            if name.starts_with("wayland-") {
                self.socket = entry.path();
            } else if name.starts_with("sway-ipc.") {
                self.ipc = entry.path();
            }
        }
    }

    /// Runs a `swaymsg` command (or `-t` query) and returns its output.
    pub fn msg(&self, args: &[&str]) -> String {
        let out = Command::new("swaymsg")
            .arg("-s")
            .arg(&self.ipc)
            .args(args)
            .output()
            .expect("swaymsg is required for the Wayland tests");
        String::from_utf8_lossy(&out.stdout).into_owned()
    }
}

impl Drop for Sway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.run);
    }
}
