//! The `Gpu` and the window wrapper of the native runs (T178, T171,
//! ADR-T178-1, ADR-T171-1): the OpenGL ES `GlGpu` of `draw-gfx` over the
//! shared GL handle of `X11Platform` or `WaylandPlatform`.
//!
//! `GlGpu` fixes the size of the default framebuffer when it is made, and
//! the platform swaps the buffers; so [`SizedGpu`] reads the window's current
//! size from the handle before each `submit` and never swaps.

use draw_gfx::gpu::gl::GlGpu;
use draw_gfx::gpu::{
    CommandList, Filter, Gpu, GpuError, PipelineDesc, TargetDesc, TextureDesc, Vertex,
};
use draw_sys::egl::GlContext;
use draw_sys::platform::Platform;

/// A shared GL handle that knows the current size of its window in pixels.
pub trait WindowSize: GlContext + Clone {
    /// The size of the window in physical pixels.
    fn window_size(&self) -> (u32, u32);
}

impl WindowSize for draw_sys::x11::SharedGl {
    fn window_size(&self) -> (u32, u32) {
        Self::window_size(self)
    }
}

impl WindowSize for draw_sys::wayland::SharedGl {
    fn window_size(&self) -> (u32, u32) {
        Self::window_size(self)
    }
}

/// The OpenGL ES Gpu over the X11 window.
pub type X11Gpu = SizedGpu<draw_sys::x11::SharedGl>;
/// The OpenGL ES Gpu over the Wayland window.
pub type WaylandGpu = SizedGpu<draw_sys::wayland::SharedGl>;

/// A [`GlGpu`] that follows the size of its window.
#[derive(Debug)]
pub struct SizedGpu<C: WindowSize> {
    inner: GlGpu<C>,
    shared: C,
}

impl<C: WindowSize> SizedGpu<C> {
    /// Makes the context current and builds the programs.
    ///
    /// # Errors
    /// [`GpuError::Backend`] if the context or a shader fails.
    pub fn new(shared: C) -> Result<Self, GpuError> {
        let inner = GlGpu::new(shared.clone(), shared.window_size())?;
        Ok(Self { inner, shared })
    }
}

impl<C: WindowSize> Gpu for SizedGpu<C> {
    type Buffer = <GlGpu<C> as Gpu>::Buffer;
    type Texture = <GlGpu<C> as Gpu>::Texture;
    type Target = <GlGpu<C> as Gpu>::Target;
    type Pipeline = <GlGpu<C> as Gpu>::Pipeline;

    fn create_buffer(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<Self::Buffer, GpuError> {
        self.inner.create_buffer(vertices, indices)
    }

    fn create_texture(
        &mut self,
        desc: TextureDesc,
        rgba: &[u8],
    ) -> Result<Self::Texture, GpuError> {
        self.inner.create_texture(desc, rgba)
    }

    fn write_texture(
        &mut self,
        texture: &Self::Texture,
        rect: [u32; 4],
        rgba: &[u8],
    ) -> Result<(), GpuError> {
        self.inner.write_texture(texture, rect, rgba)
    }

    fn create_target(&mut self, desc: TargetDesc) -> Result<Self::Target, GpuError> {
        self.inner.create_target(desc)
    }

    fn target_texture(&self, target: &Self::Target) -> Self::Texture {
        self.inner.target_texture(target)
    }

    fn set_filter(&mut self, texture: &Self::Texture, filter: Filter) -> Result<(), GpuError> {
        self.inner.set_filter(texture, filter)
    }

    fn read_texture(&mut self, texture: &Self::Texture) -> Result<Vec<u8>, GpuError> {
        self.inner.read_texture(texture)
    }

    fn pipeline(&mut self, desc: PipelineDesc) -> Self::Pipeline {
        self.inner.pipeline(desc)
    }

    fn submit(&mut self, list: &CommandList<Self>) -> Result<(), GpuError> {
        self.inner.set_window_size(self.shared.window_size());
        // The list is typed by this wrapper and the inner Gpu takes its own
        // type with the same handles: the commands are copied over.
        let mut inner_list: CommandList<GlGpu<C>> = CommandList::new();
        for command in list.commands() {
            inner_list.push(command.clone());
        }
        self.inner.submit(&inner_list)
    }
}

/// A native platform that can say its window was asked to close.
pub trait CloseRequest: Platform {
    /// Whether the window manager or compositor asked the window to close.
    fn close_requested(&self) -> bool;
}

impl CloseRequest for draw_sys::x11::X11Platform {
    fn close_requested(&self) -> bool {
        Self::close_requested(self)
    }
}

impl CloseRequest for draw_sys::wayland::WaylandPlatform {
    fn close_requested(&self) -> bool {
        Self::close_requested(self)
    }
}

/// The `X11Platform` of the native run, ending the process in order.
pub type ClosingX11 = Closing<draw_sys::x11::X11Platform>;
/// The `WaylandPlatform` of the native run, ending the process in order.
pub type ClosingWayland = Closing<draw_sys::wayland::WaylandPlatform>;

/// A native platform ending the process in order when the window is closed
/// or a quit signal arrives: the frame loop of the app has no exit of its
/// own (the macroquad window ends it). The signal flag is the one of
/// `draw_sys::x11::signal`, which does not depend on X11.
pub struct Closing<P: CloseRequest> {
    /// The wrapped platform.
    inner: P,
    /// Called once with the quit signal's number (if any) after the frame
    /// that saw the close; it ends the process.
    on_close: fn(Option<i32>),
}

impl<P: CloseRequest> Closing<P> {
    /// Wraps `inner`; `on_close` ends the process.
    #[must_use]
    pub fn new(inner: P, on_close: fn(Option<i32>)) -> Self {
        Self { inner, on_close }
    }
}

impl<P: CloseRequest> draw_sys::platform::Platform for Closing<P> {
    fn poll(&mut self, out: &mut Vec<draw_sys::platform::RawEvent>) {
        self.inner.poll(out);
    }

    fn window(&mut self) -> draw_sys::platform::WindowMetrics {
        self.inner.window()
    }

    fn clipboard_text(&mut self) -> Option<String> {
        self.inner.clipboard_text()
    }

    fn set_clipboard_text(&mut self, text: &str) {
        self.inner.set_clipboard_text(text);
    }

    fn set_clipboard_png(&mut self, png: &[u8]) {
        self.inner.set_clipboard_png(png);
    }

    fn schedule_update(&mut self) {
        self.inner.schedule_update();
    }

    async fn next_frame(&mut self) {
        self.inner.next_frame().await;
        if self.inner.close_requested() || draw_sys::x11::signal::peek().is_some() {
            (self.on_close)(draw_sys::x11::signal::take());
        }
    }
}
