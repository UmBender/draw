//! Binary entry point: starts the crash hook, the diagnostics log and the
//! signal handlers, checks the system, then opens the window and hands
//! control to the shell. A quit signal closes the window in order and
//! then ends the process by that signal (ADR-T48-1).

#![forbid(unsafe_code)]

use std::future::Future;
use std::task::{Context, Poll, Waker};

use draw::shell::app;
use draw::shell::backend::Backend;
use draw::shell::diag::{self, Level};
use draw::shell::gpu_native::{ClosingWayland, ClosingX11, WaylandGpu, X11Gpu};
use draw::shell::options::{self, Parsed};
use draw::shell::{backend, tablet};
use draw_sys::signal;
use draw_sys::wayland::WaylandPlatform;
use draw_sys::x11::{GlBackend, X11Platform};

/// The window size in pixels of the native X11 window.
const NATIVE_SIZE: (u32, u32) = (1280, 800);

/// Runs `future` to its end on this thread. The native platform's futures
/// are ready at once, so polling in a loop is enough.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        std::thread::yield_now();
    }
}

/// The text of `--version`.
fn version_text() -> String {
    format!("draw {}", env!("CARGO_PKG_VERSION"))
}

/// The text of `--help`.
fn help_text() -> String {
    format!(
        "{}\n\n\
Usage: draw [FILE.draw] [OPTIONS]\n\
       draw --from-md FILE.md [OUT.draw]\n\
       draw --help | --version\n\n\
Options:\n  \
  --from-md <file>  Convert a Markdown file to .draw and exit\n  \
  --recover [prev]  Open the last autosave (or the one before it)\n  \
  --help            Show this help\n  \
  --version         Show the version\n\n\
Debugging (for troubleshooting only):\n  \
  --log <level>        Log level: off, error, warn, info (default), debug, trace\n  \
  --frame-times        Log how long each re-render takes\n  \
  --skip-preflight     Skip the check for a display and libraries\n  \
  --backend <name>     Window backend on Linux: wayland or x11\n  \
  --renderer <name>    Document drawing: layers (default) or retained (experimental)\n  \
  --msaa <n>           Anti-aliasing samples: off, 2, 4 (default), 8\n  \
  --tablet <mode>      Pen mode: auto (default), on, off",
        version_text()
    )
}

fn main() {
    let options = match options::parse(std::env::args_os().skip(1)) {
        Parsed::Run(options) => options,
        Parsed::Help => {
            println!("{}", help_text());
            return;
        }
        Parsed::Version => {
            println!("{}", version_text());
            return;
        }
        Parsed::Refused(problem) => {
            eprintln!("draw: {problem}\nTry `draw --help`.");
            std::process::exit(2);
        }
    };
    if let Some(input) = options.from_md.as_deref() {
        if let Err(problem) = convert_markdown(input, options.file.as_deref()) {
            eprintln!("draw: {problem}");
            std::process::exit(1);
        }
        return;
    }
    let backend = backend::choose(
        options.backend,
        std::env::var_os(backend::WAYLAND_DISPLAY_ENV).as_deref(),
    );
    let tablet = tablet::detect(options.tablet);
    draw::shell::crash::install_hook();
    diag::init(options.log, backend, tablet.pen);
    if !signal::install_fatal(app::on_fatal_signal) {
        diag::log(Level::Warn, "signal: fatal-signal handler not installed");
    }
    if let Err(problem) = draw::shell::preflight::run(options.skip_preflight, backend) {
        eprintln!("draw: {problem}");
        diag::log(Level::Error, &format!("preflight: {problem}"));
        diag::finish("preflight failed");
        std::process::exit(1);
    }
    if !signal::install_quit() {
        diag::log(Level::Warn, "signal: SIGINT/SIGTERM handler not installed");
    }
    match backend {
        // The own Wayland backend and the OpenGL ES Gpu (T171, ADR-T171-1).
        Backend::Wayland => run_wayland(options, backend, tablet),
        // The own X11 backend and the OpenGL ES Gpu (T178, ADR-T178-1).
        Backend::X11 => run_x11(options, backend, tablet),
    }
    finish(signal::received());
}

/// Converts a Markdown file at `input` into a `.draw` file at `output`
/// (defaulting to `input` with a `.draw` extension).
fn convert_markdown(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
) -> Result<(), String> {
    let source =
        std::fs::read_to_string(input).map_err(|err| format!("{}: {err}", input.display()))?;
    let target = output.map_or_else(
        || input.with_extension("draw"),
        std::path::Path::to_path_buf,
    );
    let doc = draw::core::markdown::markdown_to_doc(&source);
    std::fs::write(&target, doc.to_text()).map_err(|err| format!("{}: {err}", target.display()))?;
    println!("{}", target.display());
    Ok(())
}

/// Runs the app over the own X11 backend and the OpenGL ES Gpu
/// (T178, ADR-T178-1).
fn run_x11(options: options::Options, backend: Backend, tablet: tablet::Choice) {
    let opened = X11Platform::open(
        None,
        "draw",
        app::WM_CLASS,
        NATIVE_SIZE.0,
        NATIVE_SIZE.1,
        GlBackend::Glx,
    );
    let platform = match opened {
        Ok(platform) => platform,
        Err(problem) => {
            eprintln!("draw: X11 window: {problem}");
            diag::log(Level::Error, &format!("x11: {problem}"));
            diag::finish("x11 window failed");
            std::process::exit(1);
        }
    };
    let gpu = match X11Gpu::new(platform.share_gl()) {
        Ok(gpu) => gpu,
        Err(problem) => {
            eprintln!("draw: OpenGL: {problem}");
            diag::log(Level::Error, &format!("gl: {problem}"));
            diag::finish("gl failed");
            std::process::exit(1);
        }
    };
    let platform = ClosingX11::new(platform, finish_native);
    block_on(app::run(platform, gpu, options, backend, tablet));
}

/// Runs the app over the own Wayland backend and the OpenGL ES Gpu
/// (T171, ADR-T171-1).
fn run_wayland(options: options::Options, backend: Backend, tablet: tablet::Choice) {
    // `--renderer retained` already anti-aliases every world shape via
    // feathered edges (`FEATHERED_FRAGMENT_SOURCE`), so window MSAA is
    // redundant and multiplies zoomed-in fill-rate cost.
    let window_samples = if options.renderer == options::Renderer::Retained {
        0
    } else {
        options.msaa_samples
    };
    let opened = if window_samples == 4 {
        WaylandPlatform::connect(None, "draw", app::WM_CLASS)
    } else {
        WaylandPlatform::connect_with(None, "draw", app::WM_CLASS, window_samples, (1280, 800))
    };
    let platform = match opened {
        Ok(platform) => platform,
        Err(problem) => {
            eprintln!("draw: Wayland window: {problem}");
            diag::log(Level::Error, &format!("wayland: {problem}"));
            diag::finish("wayland window failed");
            std::process::exit(1);
        }
    };
    let gpu = match WaylandGpu::new(platform.share_gl()) {
        Ok(gpu) => gpu,
        Err(problem) => {
            eprintln!("draw: OpenGL: {problem}");
            diag::log(Level::Error, &format!("gl: {problem}"));
            diag::finish("gl failed");
            std::process::exit(1);
        }
    };
    let platform = ClosingWayland::new(platform, finish_native);
    block_on(app::run(platform, gpu, options, backend, tablet));
}

/// Logs the end of the session and ends the process by `received`'s signal.
fn finish(received: Option<signal::QuitSignal>) {
    if let Some(quit) = received {
        diag::log(Level::Info, &app::signal_line(quit));
    }
    diag::finish(app::exit_reason(received));
    if let Some(quit) = received {
        signal::reraise(quit);
    }
}

/// The end of a native run: the window closed, or the quit signal
/// `number` came. The frame loop never returns, so this ends the process.
fn finish_native(number: Option<i32>) {
    finish(number.and_then(signal::QuitSignal::from_number));
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The prefix of the variables draw once read, split so the source guard
    /// `tests/no_draw_env.rs` does not match it.
    const OLD_PREFIX: &str = concat!("DRAW", "_");

    /// AC-1 of T167: `main` picks the interpreters with a `match` and runs
    /// the generic `app::run`.
    #[test]
    fn main_runs_generic_run_through_a_match() {
        let source = include_str!("main.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        for needed in [
            "match backend",
            "app::run(",
            "X11Platform::open(",
            "X11Gpu::new(",
            "ClosingX11::new(",
            "WaylandPlatform::connect_with(",
            "WaylandGpu::new(",
            "ClosingWayland::new(",
        ] {
            assert!(production.contains(needed), "{needed} missing");
        }
        for removed in ["run_on", "MqPlatform", "MqGpu", "macroquad", "miniquad"] {
            assert!(!production.contains(removed), "{removed} is still used");
        }
    }

    /// T185 AC-4: the Wayland window asks for the size of the X11 window,
    /// tall enough for the whole toolbar.
    #[test]
    fn main_opens_wayland_at_the_native_size() {
        let source = include_str!("main.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        let Some((_, body)) = production.split_once("fn run_wayland(") else {
            panic!("run_wayland missing");
        };
        let body = body.split("\nfn ").next().unwrap_or(body);

        assert!(body.contains("NATIVE_SIZE"), "{body}");
        assert!(!body.contains("WaylandPlatform::connect("), "{body}");
    }

    #[test]
    fn help_lists_the_flags() {
        let text = help_text();

        assert!(text.contains("Debugging"), "{text}");
        for flag in [
            "--log",
            "--frame-times",
            "--skip-preflight",
            "--backend",
            "--msaa",
            "--tablet",
            "--recover",
        ] {
            assert!(text.contains(flag), "{flag} missing: {text}");
        }
        assert!(!text.contains("--platform"), "{text}");
        assert!(!text.contains(OLD_PREFIX), "{text}");
        assert!(!text.contains("X11 only"), "{text}");
    }

    #[test]
    fn help_hides_the_test_flags() {
        let text = help_text();

        for hidden in ["--snapshot", "--e2e-dump", "--e2e-idle", "--e2e-clipboard"] {
            assert!(!text.contains(hidden), "{hidden} shown: {text}");
        }
    }

    #[test]
    fn version_prints_package_version() {
        assert!(version_text().contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn readme_names_no_draw_variable() {
        let readme = include_str!("../README.md");
        for line in readme.lines() {
            assert!(
                !line.contains(OLD_PREFIX),
                "an old variable name is left in the README: {line}"
            );
        }
        assert!(readme.contains("\n### Debugging\n"));
    }
}
