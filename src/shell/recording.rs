//! Recording and null interpreters of [`Gpu`] and [`Platform`]
//! (ADR-T140-1): tests run the shell with them, with no window.

use std::collections::VecDeque;
use std::future::{Future, Ready, ready};

use std::collections::HashMap;

use draw_gfx::gpu::{
    CommandList, CommandOf, Filter, Gpu, GpuError, PipelineDesc, TargetDesc, TextureDesc, Vertex,
};
use draw_sys::platform::{Platform, RawEvent, WindowMetrics};

/// A handle of the recording and null interpreters: a number unique per
/// interpreter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle(pub u32);

/// One call made to a [`RecordingGpu`].
#[derive(Debug, Clone, PartialEq)]
pub enum GpuCall {
    /// `create_buffer`.
    CreateBuffer {
        /// The new handle.
        handle: Handle,
        /// The vertices.
        vertices: Vec<Vertex>,
        /// The indices.
        indices: Vec<u32>,
    },
    /// `create_texture`.
    CreateTexture {
        /// The new handle.
        handle: Handle,
        /// The description.
        desc: TextureDesc,
    },
    /// `write_texture`.
    WriteTexture {
        /// The texture written.
        texture: Handle,
        /// `[x, y, width, height]`.
        rect: [u32; 4],
    },
    /// `create_target`.
    CreateTarget {
        /// The new handle.
        handle: Handle,
        /// The description.
        desc: TargetDesc,
    },
    /// `pipeline`.
    Pipeline {
        /// The new handle.
        handle: Handle,
        /// The description.
        desc: PipelineDesc,
    },
    /// `set_filter`.
    SetFilter {
        /// The texture changed.
        texture: Handle,
        /// The new filter.
        filter: Filter,
    },
    /// `read_texture`.
    ReadTexture {
        /// The texture read.
        texture: Handle,
    },
    /// `submit`, with a copy of the list.
    Submit {
        /// The commands, in order.
        commands: Vec<CommandOf<RecordingGpu>>,
    },
}

/// Hands out handles `0, 1, 2, …`.
#[derive(Debug, Clone, Default)]
struct Handles(u32);

impl Handles {
    fn next(&mut self) -> Handle {
        let handle = Handle(self.0);
        self.0 = self.0.saturating_add(1);
        handle
    }
}

/// Checks `rgba` against the size of `desc`.
fn check_pixels(desc: &TextureDesc, rgba: &[u8]) -> Result<(), GpuError> {
    let expected = desc.byte_len().ok_or(GpuError::BadSize)?;
    if expected == 0 {
        return Err(GpuError::BadSize);
    }
    if rgba.len() != expected {
        return Err(GpuError::BadPixelData {
            expected,
            got: rgba.len(),
        });
    }
    Ok(())
}

/// Checks that `rect` holds `rgba` exactly.
fn check_rect(rect: [u32; 4], rgba: &[u8]) -> Result<(), GpuError> {
    let [_, _, width, height] = rect;
    check_pixels(
        &TextureDesc {
            width,
            height,
            filter: draw_gfx::gpu::Filter::Nearest,
        },
        rgba,
    )
}

/// A [`Gpu`] that keeps every call, in order, and draws nothing.
#[derive(Debug, Clone, Default)]
pub struct RecordingGpu {
    calls: Vec<GpuCall>,
    handles: Handles,
    /// Size of every texture and target, for `read_texture`.
    sizes: Sizes,
}

/// Texel size of the textures an interpreter made, by handle.
type Sizes = HashMap<Handle, (u32, u32)>;

/// The zeroed pixels of the texture `handle`.
fn blank_pixels(sizes: &Sizes, handle: Handle) -> Result<Vec<u8>, GpuError> {
    let &(width, height) = sizes.get(&handle).ok_or(GpuError::UnknownHandle)?;
    let desc = TextureDesc {
        width,
        height,
        filter: Filter::Nearest,
    };
    Ok(vec![0; desc.byte_len().ok_or(GpuError::BadSize)?])
}

impl RecordingGpu {
    /// An interpreter with no call yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The calls made so far, oldest first.
    #[must_use]
    pub fn calls(&self) -> &[GpuCall] {
        &self.calls
    }

    /// Removes and returns the calls made so far.
    pub fn take_calls(&mut self) -> Vec<GpuCall> {
        std::mem::take(&mut self.calls)
    }
}

impl Gpu for RecordingGpu {
    type Buffer = Handle;
    type Texture = Handle;
    type Target = Handle;
    type Pipeline = Handle;

    fn create_buffer(&mut self, vertices: &[Vertex], indices: &[u32]) -> Result<Handle, GpuError> {
        let handle = self.handles.next();
        self.calls.push(GpuCall::CreateBuffer {
            handle,
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
        });
        Ok(handle)
    }

    fn create_texture(&mut self, desc: TextureDesc, rgba: &[u8]) -> Result<Handle, GpuError> {
        check_pixels(&desc, rgba)?;
        let handle = self.handles.next();
        self.sizes.insert(handle, (desc.width, desc.height));
        self.calls.push(GpuCall::CreateTexture { handle, desc });
        Ok(handle)
    }

    fn write_texture(
        &mut self,
        texture: &Handle,
        rect: [u32; 4],
        rgba: &[u8],
    ) -> Result<(), GpuError> {
        check_rect(rect, rgba)?;
        self.calls.push(GpuCall::WriteTexture {
            texture: *texture,
            rect,
        });
        Ok(())
    }

    fn create_target(&mut self, desc: TargetDesc) -> Result<Handle, GpuError> {
        if desc.width == 0 || desc.height == 0 || desc.samples == 0 {
            return Err(GpuError::BadSize);
        }
        let handle = self.handles.next();
        self.sizes.insert(handle, (desc.width, desc.height));
        self.calls.push(GpuCall::CreateTarget { handle, desc });
        Ok(handle)
    }

    fn set_filter(&mut self, texture: &Handle, filter: Filter) -> Result<(), GpuError> {
        if !self.sizes.contains_key(texture) {
            return Err(GpuError::UnknownHandle);
        }
        self.calls.push(GpuCall::SetFilter {
            texture: *texture,
            filter,
        });
        Ok(())
    }

    fn read_texture(&mut self, texture: &Handle) -> Result<Vec<u8>, GpuError> {
        let pixels = blank_pixels(&self.sizes, *texture)?;
        self.calls.push(GpuCall::ReadTexture { texture: *texture });
        Ok(pixels)
    }

    fn target_texture(&self, target: &Handle) -> Handle {
        *target
    }

    fn pipeline(&mut self, desc: PipelineDesc) -> Handle {
        let handle = self.handles.next();
        self.calls.push(GpuCall::Pipeline { handle, desc });
        handle
    }

    fn submit(&mut self, list: &CommandList<Self>) -> Result<(), GpuError> {
        self.calls.push(GpuCall::Submit {
            commands: list.commands().to_vec(),
        });
        Ok(())
    }
}

/// A [`Gpu`] that checks its arguments and does nothing else.
#[derive(Debug, Clone, Default)]
pub struct NullGpu {
    handles: Handles,
    sizes: Sizes,
}

impl Gpu for NullGpu {
    type Buffer = Handle;
    type Texture = Handle;
    type Target = Handle;
    type Pipeline = Handle;

    fn create_buffer(&mut self, _: &[Vertex], _: &[u32]) -> Result<Handle, GpuError> {
        Ok(self.handles.next())
    }

    fn create_texture(&mut self, desc: TextureDesc, rgba: &[u8]) -> Result<Handle, GpuError> {
        check_pixels(&desc, rgba)?;
        let handle = self.handles.next();
        self.sizes.insert(handle, (desc.width, desc.height));
        Ok(handle)
    }

    fn write_texture(&mut self, _: &Handle, rect: [u32; 4], rgba: &[u8]) -> Result<(), GpuError> {
        check_rect(rect, rgba)
    }

    fn create_target(&mut self, desc: TargetDesc) -> Result<Handle, GpuError> {
        if desc.width == 0 || desc.height == 0 || desc.samples == 0 {
            return Err(GpuError::BadSize);
        }
        let handle = self.handles.next();
        self.sizes.insert(handle, (desc.width, desc.height));
        Ok(handle)
    }

    fn target_texture(&self, target: &Handle) -> Handle {
        *target
    }

    fn set_filter(&mut self, texture: &Handle, _: Filter) -> Result<(), GpuError> {
        if self.sizes.contains_key(texture) {
            Ok(())
        } else {
            Err(GpuError::UnknownHandle)
        }
    }

    fn read_texture(&mut self, texture: &Handle) -> Result<Vec<u8>, GpuError> {
        blank_pixels(&self.sizes, *texture)
    }

    fn pipeline(&mut self, _: PipelineDesc) -> Handle {
        self.handles.next()
    }

    fn submit(&mut self, _: &CommandList<Self>) -> Result<(), GpuError> {
        Ok(())
    }
}

/// One call made to a [`RecordingPlatform`] (polls are not logged).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformCall {
    /// `window`.
    Window,
    /// `clipboard_text`.
    ClipboardText,
    /// `set_clipboard_text`.
    SetClipboardText(String),
    /// `set_clipboard_png`.
    SetClipboardPng {
        /// Size of the image.
        bytes: usize,
    },
    /// `schedule_update`.
    ScheduleUpdate,
    /// `next_frame`.
    NextFrame,
}

/// A [`Platform`] fed by the test: it replays queued events and keeps every
/// other call, in order.
#[derive(Debug, Clone)]
pub struct RecordingPlatform {
    metrics: WindowMetrics,
    events: VecDeque<RawEvent>,
    clipboard: Option<String>,
    calls: Vec<PlatformCall>,
}

impl RecordingPlatform {
    /// A window of `metrics` with no event and an empty clipboard.
    #[must_use]
    pub fn new(metrics: WindowMetrics) -> Self {
        Self {
            metrics,
            events: VecDeque::new(),
            clipboard: None,
            calls: Vec::new(),
        }
    }

    /// Queues `event` for the next poll.
    pub fn queue(&mut self, event: RawEvent) {
        if let RawEvent::Resize(metrics) = event {
            self.metrics = metrics;
        }
        self.events.push_back(event);
    }

    /// The calls made so far, oldest first.
    #[must_use]
    pub fn calls(&self) -> &[PlatformCall] {
        &self.calls
    }

    /// Removes and returns the calls made so far.
    pub fn take_calls(&mut self) -> Vec<PlatformCall> {
        std::mem::take(&mut self.calls)
    }
}

impl Platform for RecordingPlatform {
    fn poll(&mut self, out: &mut Vec<RawEvent>) {
        out.extend(self.events.drain(..));
    }

    fn window(&mut self) -> WindowMetrics {
        self.calls.push(PlatformCall::Window);
        self.metrics
    }

    fn clipboard_text(&mut self) -> Option<String> {
        self.calls.push(PlatformCall::ClipboardText);
        self.clipboard.clone()
    }

    fn set_clipboard_text(&mut self, text: &str) {
        self.calls
            .push(PlatformCall::SetClipboardText(text.to_owned()));
        self.clipboard = Some(text.to_owned());
    }

    fn set_clipboard_png(&mut self, png: &[u8]) {
        self.calls
            .push(PlatformCall::SetClipboardPng { bytes: png.len() });
    }

    fn schedule_update(&mut self) {
        self.calls.push(PlatformCall::ScheduleUpdate);
    }

    fn next_frame(&mut self) -> impl Future<Output = ()> {
        self.calls.push(PlatformCall::NextFrame);
        let done: Ready<()> = ready(());
        done
    }
}
