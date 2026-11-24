---
tags: [proposal, architecture]
task: ""
status: decided
date: 2026-11-24
---

# Windows port proposal

Written after the owner's request of 2026-11-24: port draw to Windows,
using the **fastest and safest** solutions available, even where that
breaks decisions taken when only Linux was planned. The owner's answers
are in §8a; the tasks are T186–T193 (§7), and the spec step of
[[T187 Windows ADRs and portable seams]] turns §3 into ADRs. Line counts are *measured* on `main` at 2.0.0 + T185; sizes
of new code are estimates and say so.

## 1. Goals and limits

- **Same app, same files.** Every feature of 2.0 and the `.draw` format
  of [[ADR-T108-1 The draw file format and its promise]], on Windows 10
  (1809 or later) and Windows 11, `x86_64` first.
- **Same budgets as Linux**: the redraw budgets of
  [[ADR-T24-5 Redraw budgets]], idle at 0 % CPU
  ([[T147 Frame pacing by the compositor and idle at zero]]), the 20 MiB
  binary limit of [[ADR-T82-1 Shipped binary limit of 20 MiB]]. New for
  the pen: at most **one frame queued** between input and screen.
- **Linux does not change.** No new crate in a Linux build, no change of
  behaviour on Wayland or X11, except the shared cleanups that §3.8 and
  §3.11 name.
- **The 2.0 safety rules hold** for every Windows primitive (§5 of the
  [[2.0 Architecture proposal]]): a safe wrapper with a `# Safety`
  section, unit tests, integration tests against the real system, a
  regression test per bug, `// TEST:` markers (ADR-T142-1).

## 2. What ports and what does not

| Part | Today | On Windows |
|------|-------|------------|
| `draw-types`, `draw-core` | Pure Rust; `fontdue`, `png`, `ttf-parser` are pure Rust | Port as is (but see §3.11, CRLF) |
| `draw-gfx` renderer | Generic over `Gpu` ([[ADR-T140-1 The Gpu and Platform traits]]); the trait is API-neutral: buffers, textures, targets, blend pipelines, one command list per frame | Ports; needs a second `Gpu` interpreter |
| `draw-gfx/src/gpu/gl.rs` | 1,479 lines, GLSL ES 3.00 over EGL/GLX | Linux only |
| `app::run<P, G>` | Generic, monomorphised from a `match` in `main` | Ports; one more arm |
| `draw-sys` Wayland | 3,716 lines + 2,838 generated | Linux only |
| `draw-sys` X11 | 2,664 lines | Linux only |
| `draw-sys` `egl`, `glx`, `loader` (`dlopen`), `signal` (`sigaction`) | 1,712 lines | Linux only |
| Shell paths | 7 sites read `HOME` / `XDG_STATE_HOME` (`app.rs` save and exports, `crash.rs`, `diag`) | Need a per-OS answer |
| `autosave.rs` | `std::os::unix::fs` for mode `0600` | Needs a `cfg` |
| `preflight.rs` | Checks `.so` files | Different checks |
| `backend.rs`, `main.rs` | Wayland or X11 from `WAYLAND_DISPLAY` and `--backend` | One Windows arm |
| Tests | Integration on headless sway and Xvfb; e2e with sway, `wtype`, `grim`; `check.sh` in bash with `systemd-run` | New harness and CI job |
| Packaging | `.deb`, `.rpm`, Flatpak, AUR, COPR, Nix flake | New: zip, winget, Scoop |

## 3. Proposed decisions (one ADR each)

### 3.1 GPU: Direct3D 11, a second interpreter of `Gpu`

**Proposed: Direct3D 11 (feature level 10_0 or better; the UHD 620 offers
12_1) with a DXGI flip-model swap chain, and WARP as the fallback
device.**

Why it is the fastest and safest choice on Windows:

- **Driver quality.** D3D11 is the API Windows GPU drivers are most
  tested on, and the one the desktop compositor itself uses. Intel's
  OpenGL driver on Windows is known to be weaker than its D3D driver
  (stutter, presentation bugs), and the target is an Intel iGPU.
- **Latency and pacing.** `DXGI_SWAP_EFFECT_FLIP_DISCARD` presents
  without a copy by the compositor; the frame-latency waitable object
  with `SetMaximumFrameLatency(1)` keeps at most one frame queued, which
  is what a pen needs. The same handle is the idle wait, so idle stays
  at 0 % CPU with no timer.
- **WARP.** Microsoft's software rasteriser ships with every Windows:
  draw starts in a VM, over Remote Desktop or on a machine with a broken
  driver, and CI renders without a GPU, deterministically.
- **Validation.** The D3D11 debug layer reports misuse in tests, the
  same role Vulkan's validation layers would have.
- **Device loss is survivable.** A driver update or a GPU reset returns
  `DXGI_ERROR_DEVICE_REMOVED`; the backend recreates the device and the
  renderer rebuilds its retained buffers from the document instead of
  crashing. This adds a `DeviceLost` case to `GpuError` (amends
  ADR-T140-1); GL on Linux never reports it.

The command list maps one to one: buffers to immutable or dynamic
`ID3D11Buffer`s, textures to `Texture2D` with a shader view, targets to
render targets, `samples > 1` to a multisampled target resolved with
`ResolveSubresource` (flip-model back buffers cannot be multisampled, so
MSAA always renders off screen), pipelines to blend states,
`read_texture` to a staging copy.

**Shaders.** The three GLSL programs (including the feathered edges of
the retained path) are ported to HLSL, compiled **offline** to DXBC with
`fxc` and checked in; a test on the Windows CI job recompiles them and
fails when the checked-in bytes differ (the pattern of
[[ADR-T168-1 Generated Wayland protocol code and surface type states]]).
No shader compiler at run time: no `d3dcompiler_47.dll` to load (it is
not a KnownDLL, so loading it opens a DLL-planting hole), no compile at
start-up.

Alternatives considered:

- **WGL and OpenGL 3.3 core** — the least new code (`gl.rs` reused, the
  shaders only change their `#version` line), but it keeps Intel's
  weaker Windows GL driver, gives no control over flip-model
  presentation or frame latency, and has no WARP: CI would need Mesa's
  `opengl32.dll`.
- **ANGLE** (OpenGL ES over D3D11) — keeps `egl.rs` and the shaders
  unchanged and is fast (Chrome uses it), but ships two C++ DLLs of about
  10 MB (half the binary budget), adds a translation layer we do not
  control, and loads DLLs from the app's folder.
- **Vulkan** ([[T150 Optional Vulkan backend for the Gpu trait]]) — in
  theory one GPU backend for both systems, but the most code and
  `unsafe` (allocator, synchronisation, swap chain), and Linux keeps GL
  anyway, so it would be a third backend, not a replacement. T150 stays
  optional.
- **Direct3D 12** — explicit like Vulkan, much more code, no gain at a
  handful of draw calls per frame.

Estimated size: about 1.5k lines, like `gl.rs`.

### 3.2 Bindings: Microsoft's `windows` crate, Windows target only

**Proposed: add the `windows` crate (pinned, only the features used) as a
`[target.'cfg(windows)'.dependencies]` of `draw-sys` and `draw-gfx`.**

- Signatures are generated from Microsoft's metadata: no hand-written
  FFI declaration can be wrong.
- COM interfaces are smart pointers: `Release` on `Drop`, `AddRef` on
  `Clone`. No leaked or double-released D3D object.
- `HRESULT` becomes `Result`; wide strings have checked helpers.
- No run-time cost; a Linux build never compiles it.

Alternatives: `windows-sys` (raw functions only) or hand-written
declarations, as `dlopen` is today. Both leave D3D11's COM vtables and
reference counts to hand-written `unsafe`, the most bug-prone part of
the port. Cost of the choice: compile time on Windows (estimate:
20–40 s more for a clean release build).

Breaks [[ADR-T127-3 The dependency list]] (only `fontdue`, `png`,
`ttf-parser`): amended with a target-gated exception; `tests/deps.rs`
learns it.

### 3.3 System DLLs are linked, never loaded at run time

**Proposed: import `kernel32`, `user32`, `shell32`, `dwmapi`, `d3d11`
and `dxgi` at link time; the first call in `main` is
`SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32)`; the only DLL
loaded at run time is `dbghelp` for a crash dump (§3.9), from System32
only.**

These DLLs are part of every supported Windows, so there is nothing to
fall back from: Linux's `dlopen` exists because Wayland, X11 and EGL may
be missing. [[ADR-T142-1 dlopen through libc symbols and TEST markers]]
becomes Linux-only. On Windows the preflight checks the Windows version
and that a D3D11 device (or WARP) can be created, not library files.

### 3.4 A window thread and an app thread

**Problem.** While the user drags or resizes a window, `DefWindowProc`
runs its own modal message loop. A single-threaded app does not draw
during that time: the canvas freezes or stretches until the mouse button
is released. Some menus and system dialogs do the same.

**Proposed.**

- A **window thread** owns the `HWND` and only pumps messages. It
  translates them into `RawEvent`s, sends them over a channel and sets
  an auto-reset event. It never waits on the app thread, so DXGI's own
  window messages cannot deadlock.
- The **app thread** runs today's `app::run`, owns the D3D11 device and
  the swap chain, and waits with `WaitForMultipleObjects` on the
  frame-latency handle, the input event and the quit event. A resize
  reaches it as an event; it calls `ResizeBuffers` itself.
- Only `Send` data crosses. `Drop` of the platform stops and joins the
  window thread, and releases the swap chain before the window is
  destroyed.

Gains: the canvas keeps drawing during a resize or move, and input is
read while a frame renders (pen points are not delayed by drawing).

Breaks §7 of the [[2.0 Architecture proposal]] ("threads only with a
measured gain"): this thread is for correctness, and its ADR says so.
Linux keeps one thread.

### 3.5 Input

- **Keyboard.** `WM_KEYDOWN` / `WM_KEYUP` and their `WM_SYS*` twins (Alt
  combinations). Bit 30 of `lParam` is `repeat`. Letters come from the
  layout's virtual-key codes (`RawKey::Letter`), text from `WM_CHAR`
  with UTF-16 surrogate pairs joined, dead keys through
  `TranslateMessage`.
- **AltGr.** Windows sends AltGr as a synthetic left Ctrl followed by
  right Alt with the same message time. The decoder recognises that
  pair, so typing `/`, `?`, `\` or `|` with AltGr on Brazilian (ABNT2)
  and other international layouts never fires a Ctrl shortcut. A
  regression test per layout the owner uses.
- **One pointer path.** `EnableMouseInPointer(TRUE)` before the window is
  created: mouse, pen and touch all arrive as `WM_POINTER*`.
- **Pen.** `GetPointerPenInfoHistory` returns every point Windows
  coalesced between two messages (tablets report at 200 Hz or more), so
  strokes keep all their samples. Pressure `0..=1024` becomes `0..=1`;
  the tap rule of [[T34 Pen tablet input]] stays in the shell. The
  tablet driver must have Windows Ink on (Huion's default); the feature
  note says how to check.
- **Wheel.** `WM_MOUSEWHEEL` and `WM_MOUSEHWHEEL`, 120 units per notch,
  fractions from precision touchpads; `backend.rs` gets the Windows unit
  next to the compositor ones.
- **Tested on Linux too.** The decoders (message parameters to
  `RawEvent`, UTF-16 joining, AltGr pairing, pressure, wheel) are pure
  functions in a module compiled on every OS, like
  `tests/x11_input_pure.rs`, so Linux CI runs them.

### 3.6 Window, DPI and pacing

- **Per-Monitor V2 DPI awareness** in an embedded application manifest,
  with `supportedOS` Windows 10, the UTF-8 code page and long paths. The
  manifest goes in through MSVC linker arguments from `build.rs`, with
  no crate. Fractional scales (125 %, 150 %) go straight into
  `WindowMetrics::scale` (already an `f32`); `WM_DPICHANGED` applies the
  rectangle Windows suggests. Text is rasterised at the real scale,
  sharper than Wayland's integer scale today.
- **Size** 1280×800 logical pixels, the `NATIVE_SIZE` of
  [[T185 Wayland title bar and window size]].
- **Pacing** with `Present(1, 0)` on the flip-model chain and the
  waitable object (§3.1).
- **Dark title bar.** `DWMWA_USE_IMMERSIVE_DARK_MODE` matches the dark
  theme; on Windows 11 `DWMWA_CAPTION_COLOR` can take the canvas colour.

### 3.7 Clipboard

- Text: `CF_UNICODETEXT` both ways.
- Image ([[T115 Copy the selection as an image]]): the registered `PNG`
  format (keeps alpha; browsers, Office, GIMP and chat apps read it) and
  `CF_DIBV5` for Paint and older programs.
- `OpenClipboard` is retried briefly when another program holds the
  clipboard, instead of failing the copy.

### 3.8 One `paths` module for every OS

Today seven sites read `HOME` or `XDG_STATE_HOME`. Proposed:
`shell::paths` answers `state_dir()` and `files_dir()` once per OS.
Windows uses Known Folders (`SHGetKnownFolderPath`), not environment
variables:

- state (autosave, diagnostics log, crash report): `%LOCALAPPDATA%\draw`;
- saves and exports (`draw-<date>-<time>.draw`, PNG, SVG):
  `Documents\draw` (owner decision, §8).

Writes stay "temporary file, then rename" (`std::fs::rename` is
`MoveFileExW` with replace on Windows). A sharing violation from an
antivirus or the search indexer is retried a few times with a short
back-off instead of failing the save. The `0600` mode of the autosave
becomes Linux-only: `%LOCALAPPDATA%` is already private to the user.
On Linux the change is a cleanup: one place instead of seven, same
paths.

### 3.9 Process: subsystem, quitting and crashes

- **No console window**: `windows_subsystem = "windows"`. `--help`,
  `--version` and `--from-md` attach to the parent console
  (`AttachConsole(ATTACH_PARENT_PROCESS)`) so they still print in a
  terminal.
- **Clean exit**, the promise of [[T48 Clean exit on Ctrl-C]]:
  `WM_CLOSE`, `WM_QUERYENDSESSION` and `WM_ENDSESSION` (log off,
  shutdown, Windows Update restarts) flush the autosave as `SIGTERM`
  does; Ctrl+C from a console goes through `SetConsoleCtrlHandler`.
- **Crashes.** The panic hook stays
  ([[ADR-T27-3 Crash report from a panic hook and split debug symbols]]).
  Native faults, the role of `on_fatal` and `sigaction` on Linux, go to
  `SetUnhandledExceptionFilter`, which writes the diagnostics line and a
  minidump to `%LOCALAPPDATA%\draw\crash.dmp`. The PDB stays out of the
  package, like `draw.debug`.
- [[ADR-T48-1 Signal handling in the vendored miniquad]] (signals)
  becomes Linux-only.

### 3.10 Build and hardening

- Target `x86_64-pc-windows-msvc` with a **static CRT**
  (`+crt-static`): one `draw.exe`, no Visual C++ redistributable.
- Hardening: Control Flow Guard (`-C control-flow-guard`), CET
  shadow-stack compatibility (`/CETCOMPAT`), and the defaults kept on
  (ASLR, high-entropy VA, DEP); `SetDefaultDllDirectories` of §3.3.
- The release profile does not change (fat LTO, `panic = "abort"`).
- Windows on ARM (`aarch64-pc-windows-msvc`) is the same code and cheap
  to add later (§8).

### 3.11 Line endings in `.draw` files (fix on every OS)

`snapshot::Reader::new` splits lines on `'\n'` only and tokens on
spaces, so in a `.draw` file that went through Windows with CRLF line
endings (Git's `autocrlf`, an editor) the last token of every line keeps
a `\r`. The file does not open: already the header's version, `1\r`, is
not a number (found by reading the code; the regression test will show
it). Proposed:
the reader accepts `\r\n` and writers keep writing `\n`. This is a
regression test and a small fix in `draw-core`, worth doing even if the
port waits.

## 4. Tests and gates

- **Windows code linted on Linux.** `check.sh` adds
  `cargo clippy --target x86_64-pc-windows-msvc --all-targets -D warnings`:
  the `windows` crate type-checks without Windows, so every local check
  catches Windows compile errors and lints. It needs
  `rustup target add x86_64-pc-windows-msvc`.
- **A Windows CI job** on `windows-latest`: fmt, clippy, the whole
  `cargo test --all-targets`, D3D11 tests on WARP with the debug layer,
  the shader check of §3.1, and the `unsafe_refs`, `deps` and `no_dyn`
  guards. `scripts/check.ps1` runs the same gates as `check.sh`, and a
  test fails if the two lists differ. The capped scopes of
  [[ADR-T77-1 Test gates run in capped low-priority scopes]] stay
  Linux-only.
- **Integration tests on Windows** (the 2.0 §5 set): create and close a
  real window, inject input with `SendInput`, change DPI, round-trip the
  clipboard, resize the swap chain, map device removal to recovery.
- **The pen without a tablet.** `CreateSyntheticPointerDevice(PT_PEN)`
  and `InjectSyntheticPointerInput` (Windows 10 1809 and later) inject
  pen samples with pressure, so CI proves the pen path; nobody on the
  project has a tablet on Windows (§8a).
- **Where Windows tests run.** Linux cannot run them: the red and green
  states of a Windows task are shown by the Windows CI job on its
  branch, so task branches are pushed to GitHub (§8a, answer 7). The
  Windows clippy of `check.sh` catches compile errors before that.
- **Manual checks by a tester.** The owner's friend runs CI builds on
  their own Windows machine (no tablet): window, DPI, keyboard layout,
  clipboard, installer, file association, SmartScreen. Each task lists
  the manual checks to send.
- **Render parity.** The same command lists rendered by D3D11 on WARP
  and by GL on llvmpipe, compared with a tolerance, next to
  `tests/tessellator_parity.rs` and `tests/text_parity.rs`. Every
  renderer feature lands in both backends or the parity test fails.
- **End-to-end on Windows.** A harness starts `draw.exe`, drives it with
  `SendInput`, and reads the app's own frame and snapshot through the
  existing `--e2e-dump` and `--e2e-idle` hooks (there is no compositor
  screenshot). Same case folders as [[T75 End-to-end test suite]], with
  per-backend pixel goldens where D3D11 and GL differ.
- **Release gate** ([[T129 Release gate with end-to-end on X11 and Wayland]]):
  Windows e2e joins Wayland and X11; `release.sh` checks the Windows CI
  result of the tagged commit.
- The coverage ratchet stays as it is: it counts `src/` modules, which
  are shared.

## 5. Packaging and distribution

- **Installer from the first release** (§8a): a **per-user MSI** built
  with WiX. It installs to `%LOCALAPPDATA%\Programs\draw` with no
  administrator prompt, adds the Start menu entry and associates `.draw`
  files (a ProgId under `HKCU\Software\Classes`, icon, `open` verb
  `"draw.exe" "%1"`), and uninstalls cleanly. MSIX was not chosen: it
  redirects `%LOCALAPPDATA%` writes into the package, which moves the
  autosave and crash dump away from the paths the docs give, and it is
  harder to debug. WiX's licence terms are checked in the spec step of
  [[T193 Windows installer, signing and packages]].
- **Portable zip** next to it: `draw.exe`, licences, README and
  CHANGELOG (fonts are embedded), for Scoop and for people who do not
  install.
- **winget** (the MSI) and **Scoop** (the zip) manifests pointing at the
  GitHub release.
- **Icon** in `draw.exe` from the existing artwork: an `.ico` and a
  `.rc` compiled by `rc.exe` from `build.rs` (skipped when not building
  on Windows, so the Linux clippy still runs).
- **Signed from the first release** (§8a): `draw.exe` and the MSI, in
  the release workflow, never on a developer machine. The route, checked
  on 2026-11-24:
  - Microsoft's Azure Artifact Signing (formerly Trusted Signing) takes
    individual developers only in the USA and Canada, so it is not open
    to the owner;
  - **SignPath Foundation** (proposed) signs open-source projects for
    free from GitHub Actions, with the key in an HSM; the certificate is
    issued to SignPath Foundation, so Windows shows it as the publisher,
    and it asks for MFA on GitHub and SignPath and an OSI licence (draw
    is MIT);
  - the fallback is an OV certificate from a commercial CA with cloud
    signing, which shows the owner's name, for a yearly fee.
  A signature does not remove the SmartScreen warning at once:
  reputation builds with downloads.

## 6. Decisions this breaks or amends

| Decision | Change |
|----------|--------|
| [[ADR-T127-3 The dependency list]] | Amended: `windows` crate, Windows target only (§3.2) |
| [[ADR-T132-1 Workspace of five crates with unsafe levels]] | Amended: `unsafe` allowed in `draw-gfx/src/gpu/d3d11.rs` and in `draw-sys`'s Windows module |
| [[ADR-T140-1 The Gpu and Platform traits]] | Amended: `GpuError::DeviceLost` and the renderer's rebuild (§3.1) |
| [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]] | Scoped to Linux; D3D11 on Windows (§3.1) |
| [[ADR-T142-1 dlopen through libc symbols and TEST markers]] | `dlopen` scoped to Linux; `TEST` markers everywhere (§3.3) |
| [[ADR-T48-1 Signal handling in the vendored miniquad]] | Scoped to Linux (§3.9) |
| [[ADR-T27-1 Build in an AlmaLinux 8 container for a glibc 2.28 floor]], [[ADR-T27-2 Package formats from one staging tree and one library list]] | Scoped to Linux; Windows builds on the MSVC runner (§5) |
| [[ADR-T77-1 Test gates run in capped low-priority scopes]] | Scoped to Linux (§4) |
| [[ADR-T75-1 Headless sway session for end-to-end tests]] | Joined by a Windows harness (§4) |
| [[ADR-T47-1 Flight recorder and capped session log]], [[ADR-T94-1 Autosave captured on input and written off the frame]] | Locations through `shell::paths` (§3.8) |
| [[2.0 Architecture proposal]] §7 (threads) | Exception for the window thread (§3.4) |

Unchanged: [[ADR-T131-1 No dyn outside an allow-list]] (`main`
monomorphises `run::<Win32Platform, D3d11Gpu>`), the automaton, the
file format (apart from §3.11), every Linux backend.

## 7. Order of work

Each task ships green. Linux stays releasable throughout.

| Task | Wave | Depends on | Done when |
|------|------|------------|-----------|
| [[T186 CRLF line endings in draw files]] (§3.11) | 77 | — | Regression test green on Linux; independent of the rest |
| [[T187 Windows ADRs and portable seams]]: ADRs, Linux modules behind `cfg`, `shell::paths`, per-OS `Backend`, `.gitattributes`, Windows clippy in `check.sh`, Windows CI job | 77 | — | The portable tests pass on the Windows runner; Linux unchanged |
| [[T188 Win32 window thread and message loop]]: window thread, DPI, close, end of session (`Platform` window part) | 78 | T187 | A window opens, resizes and closes in integration tests |
| [[T189 Direct3D 11 Gpu]]: device, WARP fallback, flip-model waitable swap chain, resources, MSAA resolve, read-back, offline HLSL, device loss | 79 | T188 | Parity with the GL goldens on WARP |
| [[T190 Windows keyboard and pointer]]: decoders, AltGr, repeat, wheel | 79 | T188 | Pure tests on Linux, injection tests on Windows |
| [[T191 Windows pen and clipboard]]: `WM_POINTER` pen with history and pressure, text, PNG and DIBV5 | 80 | T190 | Synthetic pen stroke with pressure; clipboard round trip |
| [[T192 Windows platform in main and the e2e suite]]: `main`, subsystem and console, crash filter, manifest, preflight, e2e harness, release gate | 81 | T189, T191 | The e2e suite passes on Windows in CI |
| [[T193 Windows installer, signing and packages]]: hardened build, per-user MSI with the association, zip, signing, winget, Scoop | 82 | T192 | A tagged release publishes signed packages |

Windows on ARM has no task yet (§8a).

Size, as engineering judgement: the Windows platform about 3k lines, the
D3D11 `Gpu` about 1.5k, shell and process changes under 1k, plus the
harness, CI and packaging scripts. That is roughly the Wayland backend
plus the GL `Gpu` again.

## 8. Decisions for the owner

1. **Hardware.** Is there a Windows machine (ideally with the Huion) for
   manual checks? CI covers WARP; drivers, the pen, DPI changes across
   monitors and AltGr need real hardware.
2. **Where saves and exports go**: `Documents\draw` (proposed) or the
   profile folder, as `$HOME` on Linux.
3. **Installer**: portable zip with winget and Scoop first (proposed), or
   an MSI with the `.draw` association from the first release.
4. **Signing**: unsigned at first (proposed), or a paid signing service
   from the first release.
5. **Windows on ARM**: later (proposed), or with the first release.
6. **Oldest Windows**: 10 version 1809 (proposed; costs nothing), or
   Windows 11 only.

## 8a. Owner's decisions (2026-11-24)

1. **Hardware**: the port is for a friend of the owner, who tests on
   their own Windows machine and has no tablet: the pen is proven by injected
   synthetic pen input in CI (§4), not by hand.
2. **Saves and exports** go to `Documents\draw`, as proposed.
3. **Installer with the `.draw` association from the first release**:
   the per-user MSI of §5.
4. **Signed from the first release**: route of §5, SignPath Foundation
   proposed.
5. **Windows on ARM later**: no task now.
6. **Oldest Windows: 10 version 1809**, as proposed.

7. **Task branches are pushed to `origin`** (answered the same day):
   Windows tests run only on the Windows CI job, so each Windows task
   pushes its branch for its red and green states, the batch runner
   included.

Still open:

8. **Signing account.** SignPath Foundation needs an application by the
   owner (or the owner picks the paid OV certificate) before
   [[T193 Windows installer, signing and packages]] can finish.

## 9. Risks, stated plainly

- **Two GPU backends to keep equal.** The parity tests are the guard; a
  renderer change that skips one backend fails CI.
- **Without a Windows machine**, some bugs only CI can see, and WARP
  hides driver bugs.
- **Win32 edge cases**: modal loops, a DPI change while the window moves
  between monitors, AltGr, the Windows Ink setting of tablet drivers,
  GPU resets. Each one gets a test listed in its task.
- **Antivirus false positives** are common for unsigned Rust binaries
  (§5).
- **The `windows` crate** has frequent releases: it stays pinned and is
  updated on purpose, like every dependency.
