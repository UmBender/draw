//! A private headless sway for the Wayland tests (T168): its own
//! `XDG_RUNTIME_DIR`, `WLR_BACKENDS=headless`, `WLR_RENDERER=pixman`. A
//! missing `sway` fails the test, it never skips it.

#![allow(clippy::expect_used, dead_code)]

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, FileTypeExt};
use std::os::unix::net::UnixStream;
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

/// A running `wtype`: a virtual keyboard on the seat while it lives.
pub struct Keyboard(Child);

impl Drop for Keyboard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A virtual pointer held open on the sway socket: a headless seat has no
/// pointer device, so this gives it the capability (ADR-T75-1).
pub struct VirtualPointer(UnixStream);

fn wire(object: u32, opcode: u32, payload: &[u8]) -> Vec<u8> {
    let size = u32::try_from(8 + payload.len()).expect("small message");
    let mut message = Vec::new();
    message.extend(object.to_ne_bytes());
    message.extend(((size << 16) | opcode).to_ne_bytes());
    message.extend(payload);
    message
}

fn wire_string(text: &str) -> Vec<u8> {
    let mut out = u32::try_from(text.len() + 1)
        .expect("short name")
        .to_ne_bytes()
        .to_vec();
    out.extend(text.as_bytes());
    out.push(0);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    out
}

/// The registry name of `interface` among the events read so far, and
/// whether the `sync` callback (object 3) has fired.
fn scan_registry(mut bytes: &[u8], interface: &str) -> (Option<u32>, bool) {
    let word = |b: &[u8], i: usize| -> Option<u32> {
        Some(u32::from_ne_bytes(b.get(i..i + 4)?.try_into().ok()?))
    };
    let (mut found, mut done) = (None, false);
    while let (Some(object), Some(header)) = (word(bytes, 0), word(bytes, 4)) {
        let size = (header >> 16) as usize;
        let Some(body) = bytes.get(8..size) else {
            break;
        };
        done |= object == 3;
        if object == 2 && header.trailing_zeros() >= 16 {
            let length = word(body, 4).unwrap_or(0) as usize;
            if body.get(8..8 + length.saturating_sub(1)) == Some(interface.as_bytes()) {
                found = word(body, 0);
            }
        }
        bytes = bytes.get(size..).unwrap_or(&[]);
    }
    (found, done)
}

impl VirtualPointer {
    /// One vertical wheel notch (`axis_source` wheel, `axis_discrete`,
    /// `frame`).
    pub fn wheel(&mut self, notch: i32) {
        let mut discrete = 0u32.to_ne_bytes().to_vec();
        discrete.extend(0u32.to_ne_bytes());
        discrete.extend(((15 * notch) << 8).to_ne_bytes());
        discrete.extend(notch.to_ne_bytes());
        let mut message = wire(5, 5, &0u32.to_ne_bytes());
        message.extend(wire(5, 7, &discrete));
        message.extend(wire(5, 4, &[]));
        self.0.write_all(&message).expect("virtual pointer write");
    }
}

impl Sway {
    /// Starts `wtype args…` in the background. Its virtual keyboard is on
    /// the seat until the guard drops. Put a `-s <ms>` first to give the
    /// client time to get the keyboard before the keys are typed.
    ///
    /// # Panics
    /// If `wtype` is missing.
    pub fn keyboard(&self, args: &[&str]) -> Keyboard {
        let child = Command::new("wtype")
            .args(args)
            .env("WAYLAND_DISPLAY", &self.socket)
            .env("XDG_RUNTIME_DIR", &self.run)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("wtype is required for the Wayland input tests");
        Keyboard(child)
    }

    /// Creates a virtual pointer on the seat.
    ///
    /// # Panics
    /// If the compositor has no `zwlr_virtual_pointer_manager_v1`.
    pub fn virtual_pointer(&self) -> VirtualPointer {
        const INTERFACE: &str = "zwlr_virtual_pointer_manager_v1";
        let mut stream = UnixStream::connect(&self.socket).expect("sway socket");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("timeout");
        // wl_display.get_registry(2), wl_display.sync(3).
        let mut hello = wire(1, 1, &2u32.to_ne_bytes());
        hello.extend(wire(1, 0, &3u32.to_ne_bytes()));
        stream.write_all(&hello).expect("hello");
        let mut received = Vec::new();
        let mut chunk = [0u8; 4096];
        let name = loop {
            let n = stream.read(&mut chunk).expect("registry");
            assert!(n > 0, "sway hung up");
            received.extend_from_slice(&chunk[..n]);
            let (found, done) = scan_registry(&received, INTERFACE);
            if done {
                break found.expect("sway has no virtual pointer manager");
            }
        };
        // bind(manager id 4), create_virtual_pointer(no seat, id 5).
        let mut bind = name.to_ne_bytes().to_vec();
        bind.extend(wire_string(INTERFACE));
        bind.extend(1u32.to_ne_bytes());
        bind.extend(4u32.to_ne_bytes());
        let mut message = wire(2, 0, &bind);
        let mut create = 0u32.to_ne_bytes().to_vec();
        create.extend(5u32.to_ne_bytes());
        message.extend(wire(4, 0, &create));
        stream.write_all(&message).expect("create pointer");
        VirtualPointer(stream)
    }

    /// Runs `swaymsg seat - cursor <args>`.
    pub fn cursor(&self, args: &[&str]) -> String {
        let mut all = vec!["seat", "-", "cursor"];
        all.extend_from_slice(args);
        self.msg(&all)
    }
}

impl Drop for Sway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.run);
    }
}
