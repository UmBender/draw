//! The `Gpu` of the native X11 run (T178, ADR-T178-1): the OpenGL ES `GlGpu`
//! of `draw-gfx` over the shared GL handle of `X11Platform`.
//!
//! `GlGpu` fixes the size of the default framebuffer when it is made, and
//! the platform swaps the buffers; so [`X11Gpu`] reads the window's current
//! size from the handle before each `submit` and never swaps.

use draw_gfx::gpu::gl::GlGpu;
use draw_gfx::gpu::{
    CommandList, Filter, Gpu, GpuError, PipelineDesc, TargetDesc, TextureDesc, Vertex,
};
use draw_sys::x11::SharedGl;

/// A [`GlGpu`] that follows the size of the X11 window.
#[derive(Debug)]
pub struct X11Gpu {
    inner: GlGpu<SharedGl>,
    shared: SharedGl,
}

impl X11Gpu {
    /// Makes the context current and builds the programs.
    ///
    /// # Errors
    /// [`GpuError::Backend`] if the context or a shader fails.
    pub fn new(shared: SharedGl) -> Result<Self, GpuError> {
        let inner = GlGpu::new(shared.clone(), shared.window_size())?;
        Ok(Self { inner, shared })
    }
}

impl Gpu for X11Gpu {
    type Buffer = <GlGpu<SharedGl> as Gpu>::Buffer;
    type Texture = <GlGpu<SharedGl> as Gpu>::Texture;
    type Target = <GlGpu<SharedGl> as Gpu>::Target;
    type Pipeline = <GlGpu<SharedGl> as Gpu>::Pipeline;

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
        let mut inner_list: CommandList<GlGpu<SharedGl>> = CommandList::new();
        for command in list.commands() {
            inner_list.push(command.clone());
        }
        self.inner.submit(&inner_list)
    }
}
