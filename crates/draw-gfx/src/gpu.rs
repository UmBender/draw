//! The `Gpu` trait: what the shell needs from a graphics backend, written
//! against Vulkan's model (ADR-T140-1).
//!
//! Resources are typed handles (associated types), nothing is drawn outside
//! one recorded [`CommandList`] per frame, and every parameter (view, render
//! target, clear colour, blend mode) is in the list or in a description: an
//! interpreter keeps no state between lists. All 2D primitives are
//! tessellated by the caller into [`Vertex`] triangles.

use std::fmt;

/// A colour with channels in `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red.
    pub r: f32,
    /// Green.
    pub g: f32,
    /// Blue.
    pub b: f32,
    /// Opacity.
    pub a: f32,
}

impl Color {
    /// A colour from its channels.
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

/// One mesh vertex: position in logical pixels, texture coordinate and
/// straight (non-premultiplied) colour bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// Position in the view's pixels, origin at the top left.
    pub pos: [f32; 2],
    /// Texture coordinate in `0..=1`.
    pub uv: [f32; 2],
    /// Red, green, blue and alpha bytes.
    pub color: [u8; 4],
}

/// How a texture is sampled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// Nearest texel.
    Nearest,
    /// Bilinear.
    Linear,
}

/// How a draw blends into its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blend {
    /// Straight alpha over what is there.
    Alpha,
    /// Replaces what is there.
    Opaque,
}

/// A texture to create: RGBA8, `width × height` texels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureDesc {
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
    /// Sampling filter.
    pub filter: Filter,
}

impl TextureDesc {
    /// Bytes an RGBA8 image of this size takes, `None` on overflow.
    #[must_use]
    pub fn byte_len(&self) -> Option<usize> {
        let texels = usize::try_from(self.width)
            .ok()?
            .checked_mul(usize::try_from(self.height).ok()?)?;
        texels.checked_mul(4)
    }
}

/// A render target to create: an RGBA8 image that can be drawn to and then
/// sampled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetDesc {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Samples per pixel; `1` is no multisampling.
    pub samples: u8,
}

/// A pipeline to create.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineDesc {
    /// Blend mode of every draw with this pipeline.
    pub blend: Blend,
}

/// The area a pass draws to: `width × height` logical pixels map to the
/// whole target (origin top left, y down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
}

/// Where the triangles of a draw come from.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry<B> {
    /// Vertices and indices carried by the command.
    Inline {
        /// The vertices.
        vertices: Vec<Vertex>,
        /// Triangle list indices into `vertices`.
        indices: Vec<u32>,
    },
    /// A range of the indices of a buffer.
    Buffer {
        /// The buffer.
        buffer: B,
        /// First index.
        first: u32,
        /// Number of indices.
        count: u32,
    },
}

/// One step of a frame. `B`, `T`, `R` and `P` are the buffer, texture,
/// target and pipeline handles of a [`Gpu`].
#[derive(Debug, Clone, PartialEq)]
pub enum Command<B, T, R, P> {
    /// Starts drawing to `target` (`None`: the window), optionally clearing it.
    BeginPass {
        /// The render target, or the window.
        target: Option<R>,
        /// The pixel area the pass draws in.
        view: View,
        /// Clear colour, `None` keeps the contents.
        clear: Option<Color>,
    },
    /// Draws triangles with `pipeline`, sampling `texture` if given.
    Draw {
        /// How to blend.
        pipeline: P,
        /// The sampled texture, `None` for untextured.
        texture: Option<T>,
        /// The triangles.
        geometry: Geometry<B>,
    },
    /// Ends the current pass.
    EndPass,
}

/// The command type of a [`Gpu`].
pub type CommandOf<G> =
    Command<<G as Gpu>::Buffer, <G as Gpu>::Texture, <G as Gpu>::Target, <G as Gpu>::Pipeline>;

/// The commands of one frame, in order.
pub struct CommandList<G: Gpu> {
    commands: Vec<CommandOf<G>>,
}

impl<G: Gpu> fmt::Debug for CommandList<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandList")
            .field("commands", &self.commands.len())
            .finish()
    }
}

impl<G: Gpu> Default for CommandList<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Gpu> CommandList<G> {
    /// An empty list.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Appends `command`.
    pub fn push(&mut self, command: CommandOf<G>) {
        self.commands.push(command);
    }

    /// The commands, oldest first.
    #[must_use]
    pub fn commands(&self) -> &[CommandOf<G>] {
        &self.commands
    }

    /// Number of commands.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether the list has no command.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Room for commands without reallocating.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.commands.capacity()
    }

    /// Removes every command, keeping the allocation for the next frame.
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

/// Why a [`Gpu`] call failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuError {
    /// The pixel data does not match the size in the description.
    BadPixelData {
        /// Bytes the description asks for.
        expected: usize,
        /// Bytes given.
        got: usize,
    },
    /// A size is zero or too large.
    BadSize,
    /// A command named a handle the interpreter does not know.
    UnknownHandle,
    /// The backend failed; the text is for the log.
    Backend(String),
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadPixelData { expected, got } => {
                write!(f, "pixel data has {got} bytes, expected {expected}")
            }
            Self::BadSize => f.write_str("size is zero or too large"),
            Self::UnknownHandle => f.write_str("unknown resource handle"),
            Self::Backend(text) => write!(f, "gpu backend: {text}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// A graphics backend (ADR-T140-1). The shell is generic over it
/// (`fn run<P: Platform, G: Gpu>`): no trait objects.
pub trait Gpu {
    /// Vertices and indices that live on the GPU.
    type Buffer: Clone;
    /// A sampled RGBA8 image.
    type Texture: Clone;
    /// An image that can be drawn to, then sampled.
    type Target: Clone;
    /// Blend state and shaders.
    type Pipeline: Copy;

    /// Uploads `vertices` and `indices`.
    ///
    /// # Errors
    /// [`GpuError`] if the backend cannot allocate.
    fn create_buffer(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Result<Self::Buffer, GpuError>;

    /// Creates a texture from `rgba` (`desc.byte_len()` bytes).
    ///
    /// # Errors
    /// [`GpuError::BadPixelData`] if `rgba` has the wrong length,
    /// [`GpuError::BadSize`] for a zero size.
    fn create_texture(&mut self, desc: TextureDesc, rgba: &[u8])
    -> Result<Self::Texture, GpuError>;

    /// Overwrites the texels of `texture` in `rect`
    /// (`[x, y, width, height]`) with `rgba`.
    ///
    /// # Errors
    /// [`GpuError`] if `rect` does not fit or `rgba` has the wrong length.
    fn write_texture(
        &mut self,
        texture: &Self::Texture,
        rect: [u32; 4],
        rgba: &[u8],
    ) -> Result<(), GpuError>;

    /// Creates a render target.
    ///
    /// # Errors
    /// [`GpuError::BadSize`] for a zero size.
    fn create_target(&mut self, desc: TargetDesc) -> Result<Self::Target, GpuError>;

    /// The texture a later pass samples to read what `target` holds.
    fn target_texture(&self, target: &Self::Target) -> Self::Texture;

    /// Creates a pipeline.
    fn pipeline(&mut self, desc: PipelineDesc) -> Self::Pipeline;

    /// Executes `list` in order. Nothing of the list survives the call.
    ///
    /// # Errors
    /// [`GpuError`] for an unknown handle or a backend failure.
    fn submit(&mut self, list: &CommandList<Self>) -> Result<(), GpuError>
    where
        Self: Sized;
}
