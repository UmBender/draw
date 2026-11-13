//! The [`Gpu`] over OpenGL ES 3.0 (ADR-T145-1): a checked function table,
//! GLSL ES 3.00 shaders and one program per [`Blend`](super::Blend).

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::fmt;
use std::marker::PhantomData;

use draw_sys::egl::GlContext;

use super::{
    CommandList, Filter, Gpu, GpuError, PipelineDesc, TargetDesc, TextureDesc, Vertex,
};

/// Why the function table could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlError {
    /// The driver has none of these entry points.
    MissingEntryPoints(Vec<String>),
}

impl fmt::Display for GlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEntryPoints(names) => {
                write!(f, "missing GL entry points: {}", names.join(", "))
            }
        }
    }
}

impl std::error::Error for GlError {}

/// The GL functions draw needs, loaded once.
#[derive(Debug)]
pub struct Gl {
    _private: (),
}

impl Gl {
    /// Every entry point that must be present.
    pub const ENTRY_POINTS: &'static [&'static str] = &[];

    /// Loads the table with `lookup`, which gives the address of a GL
    /// function by name.
    ///
    /// # Errors
    /// [`GlError::MissingEntryPoints`] listing every absent entry point.
    pub fn load_with<F>(lookup: F) -> Result<Self, GlError>
    where
        F: FnMut(&str) -> Option<*const c_void>,
    {
        let _ = lookup;
        todo!("T145: implement the function table")
    }

    /// Binds `texture` to `target` (`glBindTexture`).
    ///
    /// # Safety
    /// A context of this table must be current on the calling thread.
    // TEST: bad_gl_call_is_reported_with_its_name
    pub unsafe fn bind_texture(&self, target: u32, texture: u32) {
        let _ = (target, texture);
        todo!("T145")
    }

    /// The first GL error since the last call, with the name of the call
    /// that raised it. Only recorded in debug builds.
    #[must_use]
    pub fn take_error(&self) -> Option<String> {
        todo!("T145")
    }
}

/// Vertices and indices on the GPU.
#[derive(Debug, Clone)]
pub struct GlBuffer {
    _private: (),
}

/// A sampled RGBA8 image.
#[derive(Debug, Clone)]
pub struct GlTexture {
    _private: (),
}

/// A render target.
#[derive(Debug, Clone)]
pub struct GlTarget {
    _private: (),
}

/// A blend state with its program.
#[derive(Debug, Clone, Copy)]
pub struct GlPipeline {
    _private: (),
}

/// The GL interpreter of the [`Gpu`] trait over a context `C`.
#[derive(Debug)]
pub struct GlGpu<C: GlContext> {
    context: PhantomData<C>,
}

impl<C: GlContext> GlGpu<C> {
    /// Makes `context` current, loads and checks the function table and
    /// builds the programs. `window_size` is the size in pixels of the
    /// default framebuffer.
    ///
    /// # Errors
    /// [`GpuError::Backend`] if the context, the table or a shader fails.
    pub fn new(context: C, window_size: (u32, u32)) -> Result<Self, GpuError> {
        let _ = (context, window_size);
        todo!("T145: implement GlGpu::new")
    }

    /// The function table, for diagnostics and tests.
    #[must_use]
    pub fn gl(&self) -> &Gl {
        todo!("T145")
    }
}

impl<C: GlContext> Gpu for GlGpu<C> {
    type Buffer = GlBuffer;
    type Texture = GlTexture;
    type Target = GlTarget;
    type Pipeline = GlPipeline;

    fn create_buffer(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<Self::Buffer, GpuError> {
        let _ = (vertices, indices);
        todo!("T145")
    }

    fn create_texture(
        &mut self,
        desc: TextureDesc,
        rgba: &[u8],
    ) -> Result<Self::Texture, GpuError> {
        let _ = (desc, rgba);
        todo!("T145")
    }

    fn write_texture(
        &mut self,
        texture: &Self::Texture,
        rect: [u32; 4],
        rgba: &[u8],
    ) -> Result<(), GpuError> {
        let _ = (texture, rect, rgba);
        todo!("T145")
    }

    fn create_target(&mut self, desc: TargetDesc) -> Result<Self::Target, GpuError> {
        let _ = desc;
        todo!("T145")
    }

    fn target_texture(&self, target: &Self::Target) -> Self::Texture {
        let _ = target;
        todo!("T145")
    }

    fn set_filter(&mut self, texture: &Self::Texture, filter: Filter) -> Result<(), GpuError> {
        let _ = (texture, filter);
        todo!("T145")
    }

    fn read_texture(&mut self, texture: &Self::Texture) -> Result<Vec<u8>, GpuError> {
        let _ = texture;
        todo!("T145")
    }

    fn pipeline(&mut self, desc: PipelineDesc) -> Self::Pipeline {
        let _ = desc;
        todo!("T145")
    }

    fn submit(&mut self, list: &CommandList<Self>) -> Result<(), GpuError> {
        let _ = list;
        todo!("T145")
    }
}
