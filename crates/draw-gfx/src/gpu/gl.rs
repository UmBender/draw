//! The [`Gpu`] over OpenGL ES 3.0 (ADR-T145-1): a checked function table,
//! GLSL ES 3.00 shaders and one program per [`Blend`].
//!
//! Every GL call goes through [`Gl`]; the interpreter reaches it only
//! through a `Ready` token, which proves the context of the [`GlGpu`] is
//! current on this thread.

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::ffi::{CString, c_char, c_void};
use std::fmt;
use std::ptr::{null, without_provenance};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use draw_sys::egl::GlContext;

use super::{
    Blend, Command, CommandList, Filter, Geometry, Gpu, GpuError, PipelineDesc, TargetDesc,
    TextureDesc, Vertex, View,
};

const ARRAY_BUFFER: u32 = 0x8892;
const ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
const DYNAMIC_DRAW: u32 = 0x88E8;
const FLOAT: u32 = 0x1406;
const UNSIGNED_BYTE: u32 = 0x1401;
const UNSIGNED_INT: u32 = 0x1405;
const TRIANGLES: u32 = 4;
const TEXTURE_2D: u32 = 0x0DE1;
const TEXTURE0: u32 = 0x84C0;
const RGBA: u32 = 0x1908;
const RGBA8: u32 = 0x8058;
const TEXTURE_MAG_FILTER: u32 = 0x2800;
const TEXTURE_MIN_FILTER: u32 = 0x2801;
const TEXTURE_WRAP_S: u32 = 0x2802;
const TEXTURE_WRAP_T: u32 = 0x2803;
const NEAREST: u32 = 0x2600;
const LINEAR: u32 = 0x2601;
const CLAMP_TO_EDGE: i32 = 0x812F;
const FRAMEBUFFER: u32 = 0x8D40;
const READ_FRAMEBUFFER: u32 = 0x8CA8;
const DRAW_FRAMEBUFFER: u32 = 0x8CA9;
const RENDERBUFFER: u32 = 0x8D41;
const COLOR_ATTACHMENT0: u32 = 0x8CE0;
const FRAMEBUFFER_COMPLETE: u32 = 0x8CD5;
const COLOR_BUFFER_BIT: u32 = 0x4000;
const BLEND: u32 = 0x0BE2;
const SCISSOR_TEST: u32 = 0x0C11;
const DEPTH_TEST: u32 = 0x0B71;
const CULL_FACE: u32 = 0x0B44;
const SRC_ALPHA: u32 = 0x0302;
const ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
const ONE: u32 = 1;
const VERTEX_SHADER: u32 = 0x8B31;
const FRAGMENT_SHADER: u32 = 0x8B30;
const COMPILE_STATUS: u32 = 0x8B81;
const LINK_STATUS: u32 = 0x8B82;
const INFO_LOG_LENGTH: u32 = 0x8B84;
const MAX_SAMPLES: u32 = 0x8D57;
const MAX_TEXTURE_SIZE: u32 = 0x0D33;
const UNPACK_ALIGNMENT: u32 = 0x0CF5;
const PACK_ALIGNMENT: u32 = 0x0D05;

/// Bytes of one vertex in the vertex buffer: position, uv, colour.
const STRIDE: usize = 20;
/// [`STRIDE`] as the `GLsizei` of the attribute pointers.
const STRIDE_GL: i32 = 20;

const VERTEX_SOURCE: &str = "#version 300 es
layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;
uniform vec2 view;
out vec2 v_uv;
out vec4 v_color;
void main() {
    v_uv = uv;
    v_color = color;
    gl_Position = vec4(pos.x / view.x * 2.0 - 1.0, 1.0 - pos.y / view.y * 2.0, 0.0, 1.0);
}
";

const FRAGMENT_SOURCE: &str = "#version 300 es
precision mediump float;
uniform sampler2D tex;
in vec2 v_uv;
in vec4 v_color;
out vec4 frag;
void main() {
    frag = texture(tex, v_uv) * v_color;
}
";

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

/// The name of a GL error code.
fn error_name(code: u32) -> &'static str {
    match code {
        0x0500 => "GL_INVALID_ENUM",
        0x0501 => "GL_INVALID_VALUE",
        0x0502 => "GL_INVALID_OPERATION",
        0x0505 => "GL_OUT_OF_MEMORY",
        0x0506 => "GL_INVALID_FRAMEBUFFER_OPERATION",
        _ => "unknown GL error",
    }
}

/// Declares the function table: the struct, the loader that checks every
/// entry point, and one wrapper per function that checks `glGetError` in
/// debug builds.
macro_rules! gl_table {
    ($($name:ident = $c:literal ($($arg:ident: $ty:ty),*) $(-> $ret:ty)?;)*) => {
        /// The GL functions draw needs, loaded once; each wrapper checks
        /// `glGetError` after the call in debug builds.
        pub struct Gl {
            // TEST: triangle_set_reads_back_exactly
            $($name: unsafe extern "C" fn($($ty),*) $(-> $ret)?,)*
            first_error: RefCell<Option<String>>,
        }

        impl Gl {
            /// Every entry point that must be present.
            pub const ENTRY_POINTS: &'static [&'static str] = &[$($c),*];

            /// Loads the table with `lookup`, which gives the address of a
            /// GL function by name.
            ///
            /// # Errors
            /// [`GlError::MissingEntryPoints`] listing every absent entry
            /// point.
            pub fn load_with<F>(mut lookup: F) -> Result<Self, GlError>
            where
                F: FnMut(&str) -> Option<*const c_void>,
            {
                let mut missing: Vec<String> = Vec::new();
                $(
                    let $name = lookup($c).unwrap_or_else(|| {
                        missing.push($c.to_owned());
                        null()
                    });
                )*
                if !missing.is_empty() {
                    return Err(GlError::MissingEntryPoints(missing));
                }
                $(
                    // TEST: all_entry_points_present_loads
                    // SAFETY: every address is non-null (checked above) and
                    // is the entry point of the function named in its
                    // lookup; the field type is the C prototype of it.
                    let $name = unsafe {
                        std::mem::transmute::<
                            *const c_void,
                            unsafe extern "C" fn($($ty),*) $(-> $ret)?,
                        >($name)
                    };
                )*
                Ok(Self {
                    $($name,)*
                    first_error: RefCell::new(None),
                })
            }

            $(
                #[doc = concat!("Calls `", $c, "`, then checks `glGetError` in debug builds.")]
                ///
                /// # Safety
                /// A context of this table is current on the calling thread
                /// and the arguments are valid for the GL call.
                #[allow(clippy::too_many_arguments)]
                // TEST: triangle_set_reads_back_exactly
                pub unsafe fn $name(&self, $($arg: $ty),*) $(-> $ret)? {
                    // TEST: triangle_set_reads_back_exactly
                    // SAFETY: the field holds the entry point loaded for the
                    // call; the caller upholds the contract above.
                    let result = unsafe { (self.$name)($($arg),*) };
                    self.check($c);
                    result
                }
            )*
        }
    };
}

gl_table! {
    get_error = "glGetError"() -> u32;
    get_integerv = "glGetIntegerv"(pname: u32, data: *mut i32);
    gen_buffers = "glGenBuffers"(n: i32, buffers: *mut u32);
    delete_buffers = "glDeleteBuffers"(n: i32, buffers: *const u32);
    bind_buffer = "glBindBuffer"(target: u32, buffer: u32);
    buffer_data = "glBufferData"(target: u32, size: isize, data: *const c_void, usage: u32);
    gen_vertex_arrays = "glGenVertexArrays"(n: i32, arrays: *mut u32);
    delete_vertex_arrays = "glDeleteVertexArrays"(n: i32, arrays: *const u32);
    bind_vertex_array = "glBindVertexArray"(array: u32);
    enable_vertex_attrib_array = "glEnableVertexAttribArray"(index: u32);
    vertex_attrib_pointer = "glVertexAttribPointer"(
        index: u32, size: i32, kind: u32, normalized: u8, stride: i32, pointer: *const c_void);
    gen_textures = "glGenTextures"(n: i32, textures: *mut u32);
    delete_textures = "glDeleteTextures"(n: i32, textures: *const u32);
    bind_texture = "glBindTexture"(target: u32, texture: u32);
    active_texture = "glActiveTexture"(texture: u32);
    tex_image_2d = "glTexImage2D"(
        target: u32, level: i32, internal: i32, width: i32, height: i32, border: i32,
        format: u32, kind: u32, data: *const c_void);
    tex_sub_image_2d = "glTexSubImage2D"(
        target: u32, level: i32, x: i32, y: i32, width: i32, height: i32,
        format: u32, kind: u32, data: *const c_void);
    tex_parameteri = "glTexParameteri"(target: u32, pname: u32, param: i32);
    pixel_storei = "glPixelStorei"(pname: u32, param: i32);
    gen_framebuffers = "glGenFramebuffers"(n: i32, framebuffers: *mut u32);
    delete_framebuffers = "glDeleteFramebuffers"(n: i32, framebuffers: *const u32);
    bind_framebuffer = "glBindFramebuffer"(target: u32, framebuffer: u32);
    framebuffer_texture_2d = "glFramebufferTexture2D"(
        target: u32, attachment: u32, textarget: u32, texture: u32, level: i32);
    framebuffer_renderbuffer = "glFramebufferRenderbuffer"(
        target: u32, attachment: u32, renderbuffertarget: u32, renderbuffer: u32);
    check_framebuffer_status = "glCheckFramebufferStatus"(target: u32) -> u32;
    blit_framebuffer = "glBlitFramebuffer"(
        sx0: i32, sy0: i32, sx1: i32, sy1: i32, dx0: i32, dy0: i32, dx1: i32, dy1: i32,
        mask: u32, filter: u32);
    gen_renderbuffers = "glGenRenderbuffers"(n: i32, renderbuffers: *mut u32);
    delete_renderbuffers = "glDeleteRenderbuffers"(n: i32, renderbuffers: *const u32);
    bind_renderbuffer = "glBindRenderbuffer"(target: u32, renderbuffer: u32);
    renderbuffer_storage_multisample = "glRenderbufferStorageMultisample"(
        target: u32, samples: i32, internal: u32, width: i32, height: i32);
    viewport = "glViewport"(x: i32, y: i32, width: i32, height: i32);
    clear_color = "glClearColor"(r: f32, g: f32, b: f32, a: f32);
    clear = "glClear"(mask: u32);
    enable = "glEnable"(cap: u32);
    disable = "glDisable"(cap: u32);
    blend_func_separate = "glBlendFuncSeparate"(
        src_rgb: u32, dst_rgb: u32, src_alpha: u32, dst_alpha: u32);
    create_shader = "glCreateShader"(kind: u32) -> u32;
    shader_source = "glShaderSource"(
        shader: u32, count: i32, string: *const *const c_char, length: *const i32);
    compile_shader = "glCompileShader"(shader: u32);
    get_shaderiv = "glGetShaderiv"(shader: u32, pname: u32, params: *mut i32);
    get_shader_info_log = "glGetShaderInfoLog"(
        shader: u32, size: i32, length: *mut i32, log: *mut c_char);
    delete_shader = "glDeleteShader"(shader: u32);
    create_program = "glCreateProgram"() -> u32;
    attach_shader = "glAttachShader"(program: u32, shader: u32);
    link_program = "glLinkProgram"(program: u32);
    get_programiv = "glGetProgramiv"(program: u32, pname: u32, params: *mut i32);
    get_program_info_log = "glGetProgramInfoLog"(
        program: u32, size: i32, length: *mut i32, log: *mut c_char);
    use_program = "glUseProgram"(program: u32);
    delete_program = "glDeleteProgram"(program: u32);
    get_uniform_location = "glGetUniformLocation"(program: u32, name: *const c_char) -> i32;
    uniform2f = "glUniform2f"(location: i32, x: f32, y: f32);
    uniform1i = "glUniform1i"(location: i32, value: i32);
    draw_elements = "glDrawElements"(mode: u32, count: i32, kind: u32, indices: *const c_void);
    read_pixels = "glReadPixels"(
        x: i32, y: i32, width: i32, height: i32, format: u32, kind: u32, pixels: *mut c_void);
}

impl fmt::Debug for Gl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gl")
            .field("entry_points", &Self::ENTRY_POINTS.len())
            .finish_non_exhaustive()
    }
}

impl Gl {
    /// Drains `glGetError` after `call` and keeps the first error. Debug
    /// builds only: release builds trust the driver.
    fn check(&self, call: &'static str) {
        if !cfg!(debug_assertions) {
            return;
        }
        // A lost context reports an error forever, so the loop is bounded.
        for _ in 0..8 {
            // TEST: bad_gl_call_is_reported_with_its_name
            // SAFETY: `glGetError` takes no arguments; the callers of the
            // wrappers keep a context current.
            let code = unsafe { (self.get_error)() };
            if code == 0 {
                break;
            }
            let mut first = self.first_error.borrow_mut();
            if first.is_none() {
                *first = Some(format!("{call}: {} ({code:#x})", error_name(code)));
            }
        }
    }

    /// The first GL error since the last call, with the name of the call
    /// that raised it. Only recorded in debug builds.
    #[must_use]
    pub fn take_error(&self) -> Option<String> {
        self.first_error.borrow_mut().take()
    }
}

/// What a dropped handle leaves for the next `Gpu` call to delete.
#[derive(Debug)]
enum Release {
    Buffers(u32, u32),
    Texture(u32),
    Target { fbo: u32, msaa: Option<(u32, u32)> },
}

type ReleaseQueue = Rc<RefCell<Vec<Release>>>;

#[derive(Debug)]
struct TextureInner {
    owner: u64,
    name: u32,
    width: u32,
    height: u32,
    filter: Cell<Filter>,
    release: ReleaseQueue,
}

impl Drop for TextureInner {
    fn drop(&mut self) {
        self.release.borrow_mut().push(Release::Texture(self.name));
    }
}

#[derive(Debug)]
struct BufferInner {
    owner: u64,
    vbo: u32,
    ibo: u32,
    indices: u32,
    release: ReleaseQueue,
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        self.release
            .borrow_mut()
            .push(Release::Buffers(self.vbo, self.ibo));
    }
}

#[derive(Debug)]
struct TargetInner {
    owner: u64,
    fbo: u32,
    msaa: Option<(u32, u32)>,
    texture: GlTexture,
    release: ReleaseQueue,
}

impl Drop for TargetInner {
    fn drop(&mut self) {
        self.release.borrow_mut().push(Release::Target {
            fbo: self.fbo,
            msaa: self.msaa,
        });
    }
}

/// Vertices and indices on the GPU.
#[derive(Debug, Clone)]
pub struct GlBuffer(Rc<BufferInner>);

/// A sampled RGBA8 image.
#[derive(Debug, Clone)]
pub struct GlTexture(Rc<TextureInner>);

/// A render target.
#[derive(Debug, Clone)]
pub struct GlTarget(Rc<TargetInner>);

/// A blend state with its program.
#[derive(Debug, Clone, Copy)]
pub struct GlPipeline {
    owner: u64,
    blend: Blend,
}

/// A linked program and its uniform locations.
#[derive(Debug, Clone, Copy)]
struct Program {
    name: u32,
    view: i32,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// The id of the `GlGpu` whose context is current on this thread.
    static CURRENT: Cell<u64> = const { Cell::new(0) };
}

/// Proof that the context of a [`GlGpu`] is current on this thread; only
/// [`GlGpu::ready`] makes one.
#[derive(Clone, Copy)]
struct Ready<'a> {
    gl: &'a Gl,
}

fn backend(text: impl Into<String>) -> GpuError {
    GpuError::Backend(text.into())
}

/// The vertices as the bytes of the vertex buffer: two `f32` position, two
/// `f32` uv, four colour bytes, native endian.
fn encode_vertices(vertices: &[Vertex], out: &mut Vec<u8>) {
    out.clear();
    out.reserve(vertices.len() * STRIDE);
    for v in vertices {
        for value in [v.pos[0], v.pos[1], v.uv[0], v.uv[1]] {
            out.extend_from_slice(&value.to_ne_bytes());
        }
        out.extend_from_slice(&v.color);
    }
}

/// The indices as bytes, native endian.
fn encode_indices(indices: &[u32], out: &mut Vec<u8>) {
    out.clear();
    out.reserve(indices.len() * 4);
    for index in indices {
        out.extend_from_slice(&index.to_ne_bytes());
    }
}

/// Whether every index points into `vertices` vertices.
fn indices_in_range(indices: &[u32], vertices: usize) -> bool {
    indices
        .iter()
        .all(|&i| usize::try_from(i).is_ok_and(|i| i < vertices))
}

fn filter_value(filter: Filter) -> u32 {
    match filter {
        Filter::Nearest => NEAREST,
        Filter::Linear => LINEAR,
    }
}

fn gl_size(value: usize) -> Result<isize, GpuError> {
    isize::try_from(value).map_err(|_| GpuError::BadSize)
}

fn gl_int(value: u32) -> Result<i32, GpuError> {
    i32::try_from(value).map_err(|_| GpuError::BadSize)
}

impl Ready<'_> {
    fn gen_one(&self, gen_call: impl Fn(&Gl, *mut u32)) -> u32 {
        let mut name = 0;
        gen_call(self.gl, &raw mut name);
        name
    }

    fn texture_name(&self) -> u32 {
        self.gen_one(|gl, out| {
            // TEST: triangle_set_reads_back_exactly
            // SAFETY: `out` points to one `u32`; `Ready` proves a current
            // context.
            unsafe { gl.gen_textures(1, out) };
        })
    }

    /// Allocates the storage of texture `name` and sets its sampling.
    fn texture_storage(&self, name: u32, width: i32, height: i32, filter: Filter, data: &[u8]) {
        let pixels = if data.is_empty() {
            null()
        } else {
            data.as_ptr().cast::<c_void>()
        };
        // TEST: texture_round_trips_through_write_and_read
        // SAFETY: `pixels` is null or points to `width * height * 4` bytes
        // (the callers check the length); the context is current.
        unsafe {
            self.gl.bind_texture(TEXTURE_2D, name);
            self.gl.pixel_storei(UNPACK_ALIGNMENT, 1);
            self.gl.tex_image_2d(
                TEXTURE_2D,
                0,
                RGBA8.cast_signed(),
                width,
                height,
                0,
                RGBA,
                UNSIGNED_BYTE,
                pixels,
            );
            self.gl
                .tex_parameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE);
            self.gl
                .tex_parameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE);
        }
        self.sampling(name, filter);
    }

    fn sampling(self, name: u32, filter: Filter) {
        let value = filter_value(filter).cast_signed();
        // TEST: set_filter_changes_sampling
        // SAFETY: plain state calls on a texture name of this context.
        unsafe {
            self.gl.bind_texture(TEXTURE_2D, name);
            self.gl
                .tex_parameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, value);
            self.gl
                .tex_parameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, value);
        }
    }

    fn sub_image(&self, name: u32, rect: [i32; 4], data: &[u8]) {
        // TEST: texture_round_trips_through_write_and_read
        // SAFETY: `data` has `rect[2] * rect[3] * 4` bytes (checked by the
        // caller) and the rectangle is inside the texture.
        unsafe {
            self.gl.bind_texture(TEXTURE_2D, name);
            self.gl.pixel_storei(UNPACK_ALIGNMENT, 1);
            self.gl.tex_sub_image_2d(
                TEXTURE_2D,
                0,
                rect[0],
                rect[1],
                rect[2],
                rect[3],
                RGBA,
                UNSIGNED_BYTE,
                data.as_ptr().cast(),
            );
        }
    }

    fn integer(&self, pname: u32) -> i32 {
        let mut value = 0;
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `value` is a valid out pointer for one integer.
        unsafe { self.gl.get_integerv(pname, &raw mut value) };
        value
    }

    /// A framebuffer with `texture` as colour attachment, complete or not.
    fn texture_framebuffer(&self, texture: u32) -> (u32, bool) {
        let fbo = self.gen_one(|gl, out| {
            // TEST: triangle_set_reads_back_exactly
            // SAFETY: `out` points to one `u32`.
            unsafe { gl.gen_framebuffers(1, out) };
        });
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `fbo` and `texture` are names of this context.
        let complete = unsafe {
            self.gl.bind_framebuffer(FRAMEBUFFER, fbo);
            self.gl
                .framebuffer_texture_2d(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, texture, 0);
            self.gl.check_framebuffer_status(FRAMEBUFFER) == FRAMEBUFFER_COMPLETE
        };
        (fbo, complete)
    }

    /// A multisampled framebuffer: renderbuffer and framebuffer names and
    /// whether it is complete.
    fn msaa_framebuffer(&self, samples: i32, width: i32, height: i32) -> (u32, u32, bool) {
        let rbo = self.gen_one(|gl, out| {
            // TEST: multisampled_target_resolves
            // SAFETY: `out` points to one `u32`.
            unsafe { gl.gen_renderbuffers(1, out) };
        });
        let fbo = self.gen_one(|gl, out| {
            // TEST: multisampled_target_resolves
            // SAFETY: `out` points to one `u32`.
            unsafe { gl.gen_framebuffers(1, out) };
        });
        // TEST: multisampled_target_resolves
        // SAFETY: `rbo` and `fbo` are names of this context.
        let complete = unsafe {
            self.gl.bind_renderbuffer(RENDERBUFFER, rbo);
            self.gl
                .renderbuffer_storage_multisample(RENDERBUFFER, samples, RGBA8, width, height);
            self.gl.bind_framebuffer(FRAMEBUFFER, fbo);
            self.gl
                .framebuffer_renderbuffer(FRAMEBUFFER, COLOR_ATTACHMENT0, RENDERBUFFER, rbo);
            self.gl.check_framebuffer_status(FRAMEBUFFER) == FRAMEBUFFER_COMPLETE
        };
        (fbo, rbo, complete)
    }

    fn delete(&self, release: &Release) {
        // TEST: dropped_handles_are_released_without_a_gl_error
        // SAFETY: the names were created by this context and, being queued
        // by the drop of their last handle, are deleted once.
        unsafe {
            match *release {
                Release::Buffers(vbo, ibo) => {
                    self.gl.delete_buffers(2, [vbo, ibo].as_ptr());
                }
                Release::Texture(name) => self.gl.delete_textures(1, &raw const name),
                Release::Target { fbo, msaa } => {
                    self.gl.delete_framebuffers(1, &raw const fbo);
                    if let Some((msaa_fbo, rbo)) = msaa {
                        self.gl.delete_framebuffers(1, &raw const msaa_fbo);
                        self.gl.delete_renderbuffers(1, &raw const rbo);
                    }
                }
            }
        }
    }

    fn compile(&self, kind: u32, source: &str) -> Result<u32, GpuError> {
        let text = CString::new(source).map_err(|_| backend("shader source has a NUL"))?;
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `text` is NUL-terminated and outlives the calls; a null
        // length array means the string is NUL-terminated.
        let shader = unsafe {
            let shader = self.gl.create_shader(kind);
            let pointer = text.as_ptr();
            self.gl.shader_source(shader, 1, &raw const pointer, null());
            self.gl.compile_shader(shader);
            shader
        };
        let mut status = 0;
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `status` is a valid out pointer.
        unsafe {
            self.gl
                .get_shaderiv(shader, COMPILE_STATUS, &raw mut status);
        }
        if status != 0 {
            return Ok(shader);
        }
        let log = self.log(shader, true);
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `shader` is a name of this context.
        unsafe { self.gl.delete_shader(shader) };
        Err(backend(format!("shader does not compile: {log}")))
    }

    /// The info log of a shader or program.
    fn log(&self, object: u32, shader: bool) -> String {
        let mut length = 0;
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `length` is a valid out pointer.
        unsafe {
            if shader {
                self.gl
                    .get_shaderiv(object, INFO_LOG_LENGTH, &raw mut length);
            } else {
                self.gl
                    .get_programiv(object, INFO_LOG_LENGTH, &raw mut length);
            }
        }
        let mut buffer = vec![0u8; usize::try_from(length).unwrap_or(0).max(1)];
        let mut written = 0;
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `buffer` has `length.max(1)` bytes, the size passed, and
        // `written` is a valid out pointer.
        unsafe {
            let size = i32::try_from(buffer.len()).unwrap_or(i32::MAX);
            let log = buffer.as_mut_ptr().cast::<c_char>();
            if shader {
                self.gl
                    .get_shader_info_log(object, size, &raw mut written, log);
            } else {
                self.gl
                    .get_program_info_log(object, size, &raw mut written, log);
            }
        }
        let used = usize::try_from(written).unwrap_or(0).min(buffer.len());
        String::from_utf8_lossy(&buffer[..used]).trim().to_owned()
    }

    fn link(&self) -> Result<Program, GpuError> {
        let vertex = self.compile(VERTEX_SHADER, VERTEX_SOURCE)?;
        let fragment = self.compile(FRAGMENT_SHADER, FRAGMENT_SOURCE)?;
        let view = c"view";
        let tex = c"tex";
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: the shader names were just created; the uniform names are
        // NUL-terminated literals.
        let (name, view_location) = unsafe {
            let program = self.gl.create_program();
            self.gl.attach_shader(program, vertex);
            self.gl.attach_shader(program, fragment);
            self.gl.link_program(program);
            self.gl.delete_shader(vertex);
            self.gl.delete_shader(fragment);
            (
                program,
                self.gl.get_uniform_location(program, view.as_ptr()),
            )
        };
        let mut status = 0;
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `status` is a valid out pointer.
        unsafe { self.gl.get_programiv(name, LINK_STATUS, &raw mut status) };
        if status == 0 {
            let log = self.log(name, false);
            // TEST: triangle_set_reads_back_exactly
            // SAFETY: `name` is a program of this context.
            unsafe { self.gl.delete_program(name) };
            return Err(backend(format!("program does not link: {log}")));
        }
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `name` is a linked program; the sampler is on unit 0.
        unsafe {
            self.gl.use_program(name);
            let location = self.gl.get_uniform_location(name, tex.as_ptr());
            self.gl.uniform1i(location, 0);
        }
        Ok(Program {
            name,
            view: view_location,
        })
    }
}

/// The interpreter of the [`Gpu`] trait over OpenGL ES 3.0 and a context `C`.
#[derive(Debug)]
pub struct GlGpu<C: GlContext> {
    context: C,
    gl: Gl,
    id: u64,
    window: (u32, u32),
    release: ReleaseQueue,
    /// One program per [`Blend`]: alpha, then opaque.
    programs: [Program; 2],
    vao: u32,
    scratch_vbo: u32,
    scratch_ibo: u32,
    white: u32,
    max_texture: i32,
    max_samples: i32,
    /// Scratch bytes of inline geometry, reused by every draw.
    vertex_bytes: RefCell<Vec<u8>>,
    index_bytes: RefCell<Vec<u8>>,
}

fn program_index(blend: Blend) -> usize {
    match blend {
        Blend::Alpha => 0,
        Blend::Opaque => 1,
    }
}

impl<C: GlContext> GlGpu<C> {
    /// Makes `context` current, loads and checks the function table and
    /// builds the programs. `window_size` is the size in pixels of the
    /// default framebuffer.
    ///
    /// # Errors
    /// [`GpuError::Backend`] if the context, the table or a shader fails.
    pub fn new(context: C, window_size: (u32, u32)) -> Result<Self, GpuError> {
        context.make_current().map_err(|e| backend(e.to_string()))?;
        let gl =
            Gl::load_with(|name| context.proc_address(name)).map_err(|e| backend(e.to_string()))?;
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        CURRENT.with(|current| current.set(id));
        let ready = Ready { gl: &gl };
        let max_texture = ready.integer(MAX_TEXTURE_SIZE);
        let max_samples = ready.integer(MAX_SAMPLES).max(1);
        let alpha = ready.link()?;
        let opaque = ready.link()?;
        let white = ready.texture_name();
        ready.texture_storage(white, 1, 1, Filter::Nearest, &[255; 4]);
        let mut names = [0u32; 3];
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: `names` has room for the names asked for in each call.
        unsafe {
            let mut vao = 0;
            gl.gen_vertex_arrays(1, &raw mut vao);
            gl.gen_buffers(2, names[1..].as_mut_ptr());
            names[0] = vao;
        }
        let gpu = Self {
            context,
            gl,
            id,
            window: window_size,
            release: Rc::new(RefCell::new(Vec::new())),
            programs: [alpha, opaque],
            vao: names[0],
            scratch_vbo: names[1],
            scratch_ibo: names[2],
            white,
            max_texture,
            max_samples,
            vertex_bytes: RefCell::new(Vec::new()),
            index_bytes: RefCell::new(Vec::new()),
        };
        gpu.finish()?;
        Ok(gpu)
    }

    /// The function table, for diagnostics and tests.
    #[must_use]
    pub fn gl(&self) -> &Gl {
        &self.gl
    }

    /// Sets the size in pixels of the default framebuffer, after the window
    /// is resized.
    pub fn set_window_size(&mut self, size: (u32, u32)) {
        self.window = size;
    }

    /// Presents the default framebuffer.
    ///
    /// # Errors
    /// [`GpuError::Backend`] if the context refuses.
    pub fn swap_buffers(&mut self) -> Result<(), GpuError> {
        self.ready()?;
        self.context
            .swap_buffers()
            .map_err(|e| backend(e.to_string()))
    }

    /// Makes the context current if another one took over the thread, and
    /// deletes what dropped handles left.
    fn ready(&self) -> Result<Ready<'_>, GpuError> {
        if CURRENT.with(Cell::get) != self.id {
            self.context
                .make_current()
                .map_err(|e| backend(e.to_string()))?;
            CURRENT.with(|current| current.set(self.id));
        }
        let ready = Ready { gl: &self.gl };
        let queued = std::mem::take(&mut *self.release.borrow_mut());
        for release in &queued {
            ready.delete(release);
        }
        Ok(ready)
    }

    /// Turns the first recorded GL error into an error value.
    fn finish(&self) -> Result<(), GpuError> {
        match self.gl.take_error() {
            Some(text) => Err(backend(text)),
            None => Ok(()),
        }
    }

    fn check_owner(&self, owner: u64) -> Result<(), GpuError> {
        if owner == self.id {
            Ok(())
        } else {
            Err(GpuError::UnknownHandle)
        }
    }

    fn side(&self, value: u32) -> Result<i32, GpuError> {
        let value = gl_int(value)?;
        if value == 0 || value > self.max_texture {
            return Err(GpuError::BadSize);
        }
        Ok(value)
    }

    fn texture_handle(&self, name: u32, width: u32, height: u32, filter: Filter) -> GlTexture {
        GlTexture(Rc::new(TextureInner {
            owner: self.id,
            name,
            width,
            height,
            filter: Cell::new(filter),
            release: Rc::clone(&self.release),
        }))
    }

    /// Binds the vertex data of one draw and returns the first index byte
    /// offset and index count.
    fn bind_geometry(
        &self,
        ready: Ready<'_>,
        geometry: &Geometry<GlBuffer>,
    ) -> Result<(usize, i32), GpuError> {
        let (vbo, ibo, first, count) = match geometry {
            Geometry::Inline { vertices, indices } => {
                if !indices_in_range(indices, vertices.len()) {
                    return Err(GpuError::BadSize);
                }
                let mut vertex_bytes = self.vertex_bytes.borrow_mut();
                let mut index_bytes = self.index_bytes.borrow_mut();
                encode_vertices(vertices, &mut vertex_bytes);
                encode_indices(indices, &mut index_bytes);
                // TEST: triangle_set_reads_back_exactly
                // SAFETY: the byte vectors outlive the calls and the sizes
                // passed are their lengths.
                unsafe {
                    ready.gl.bind_buffer(ARRAY_BUFFER, self.scratch_vbo);
                    ready.gl.buffer_data(
                        ARRAY_BUFFER,
                        gl_size(vertex_bytes.len())?,
                        vertex_bytes.as_ptr().cast(),
                        DYNAMIC_DRAW,
                    );
                    ready.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, self.scratch_ibo);
                    ready.gl.buffer_data(
                        ELEMENT_ARRAY_BUFFER,
                        gl_size(index_bytes.len())?,
                        index_bytes.as_ptr().cast(),
                        DYNAMIC_DRAW,
                    );
                }
                (
                    self.scratch_vbo,
                    self.scratch_ibo,
                    0,
                    u32::try_from(indices.len()).map_err(|_| GpuError::BadSize)?,
                )
            }
            Geometry::Buffer {
                buffer,
                first,
                count,
            } => {
                self.check_owner(buffer.0.owner)?;
                let end = first.checked_add(*count).ok_or(GpuError::BadSize)?;
                if end > buffer.0.indices {
                    return Err(GpuError::BadSize);
                }
                (buffer.0.vbo, buffer.0.ibo, *first, *count)
            }
        };
        // TEST: buffer_geometry_matches_inline_geometry
        // SAFETY: the buffers are of this context; the attribute offsets
        // and the stride describe the layout `encode_vertices` writes.
        unsafe {
            ready.gl.bind_vertex_array(self.vao);
            ready.gl.bind_buffer(ARRAY_BUFFER, vbo);
            ready.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, ibo);
            let stride = STRIDE_GL;
            ready.gl.enable_vertex_attrib_array(0);
            ready
                .gl
                .vertex_attrib_pointer(0, 2, FLOAT, 0, stride, null());
            ready.gl.enable_vertex_attrib_array(1);
            ready
                .gl
                .vertex_attrib_pointer(1, 2, FLOAT, 0, stride, without_provenance(8));
            ready.gl.enable_vertex_attrib_array(2);
            ready
                .gl
                .vertex_attrib_pointer(2, 4, UNSIGNED_BYTE, 1, stride, without_provenance(16));
        }
        Ok((first as usize * 4, gl_int(count)?))
    }

    /// Starts a pass.
    fn begin_pass(
        &self,
        ready: Ready<'_>,
        target: Option<&GlTarget>,
        view: View,
        clear: Option<super::Color>,
    ) -> Result<(), GpuError> {
        let (framebuffer, width, height) = match target {
            Some(target) => {
                self.check_owner(target.0.owner)?;
                let texture = &target.0.texture.0;
                let fbo = target.0.msaa.map_or(target.0.fbo, |(fbo, _)| fbo);
                (fbo, texture.width, texture.height)
            }
            None => (0, self.window.0, self.window.1),
        };
        let (width, height) = (gl_int(width)?, gl_int(height)?);
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: the framebuffer is 0 or a name of this context.
        unsafe {
            ready.gl.bind_framebuffer(FRAMEBUFFER, framebuffer);
            ready.gl.viewport(0, 0, width, height);
            ready.gl.disable(DEPTH_TEST);
            ready.gl.disable(CULL_FACE);
            ready.gl.disable(SCISSOR_TEST);
        }
        match (clear, target.and_then(|t| t.0.msaa.map(|m| (t, m)))) {
            (Some(color), _) => {
                // TEST: triangle_set_reads_back_exactly
                // SAFETY: plain state and clear calls.
                unsafe {
                    ready.gl.clear_color(color.r, color.g, color.b, color.a);
                    ready.gl.clear(COLOR_BUFFER_BIT);
                }
            }
            (None, Some((target, (msaa_fbo, _)))) => {
                // Keeps the contents: copies the resolved texture into the
                // multisampled buffer before drawing on top.
                // TEST: multisampled_target_resolves
                // SAFETY: both framebuffers are of this target.
                unsafe {
                    ready.gl.bind_framebuffer(READ_FRAMEBUFFER, target.0.fbo);
                    ready.gl.bind_framebuffer(DRAW_FRAMEBUFFER, msaa_fbo);
                    ready.gl.blit_framebuffer(
                        0,
                        0,
                        width,
                        height,
                        0,
                        0,
                        width,
                        height,
                        COLOR_BUFFER_BIT,
                        NEAREST,
                    );
                    ready.gl.bind_framebuffer(FRAMEBUFFER, msaa_fbo);
                }
            }
            (None, None) => {}
        }
        let _ = view;
        Ok(())
    }

    /// Ends a pass: resolves a multisampled target into its texture.
    fn end_pass(ready: Ready<'_>, target: Option<&GlTarget>) {
        if let Some(target) = target
            && let Some((msaa_fbo, _)) = target.0.msaa
        {
            let texture = &target.0.texture.0;
            let (width, height) = (
                i32::try_from(texture.width).unwrap_or(i32::MAX),
                i32::try_from(texture.height).unwrap_or(i32::MAX),
            );
            // TEST: multisampled_target_resolves
            // SAFETY: both framebuffers are of this target.
            unsafe {
                ready.gl.bind_framebuffer(READ_FRAMEBUFFER, msaa_fbo);
                ready.gl.bind_framebuffer(DRAW_FRAMEBUFFER, target.0.fbo);
                ready.gl.blit_framebuffer(
                    0,
                    0,
                    width,
                    height,
                    0,
                    0,
                    width,
                    height,
                    COLOR_BUFFER_BIT,
                    NEAREST,
                );
            }
        }
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: unbinding to the default framebuffer is always valid.
        unsafe { ready.gl.bind_framebuffer(FRAMEBUFFER, 0) };
    }

    fn draw(
        &self,
        ready: Ready<'_>,
        view: View,
        pipeline: GlPipeline,
        texture: Option<&GlTexture>,
        geometry: &Geometry<GlBuffer>,
    ) -> Result<(), GpuError> {
        self.check_owner(pipeline.owner)?;
        let texture_name = match texture {
            Some(texture) => {
                self.check_owner(texture.0.owner)?;
                texture.0.name
            }
            None => self.white,
        };
        let program = self.programs[program_index(pipeline.blend)];
        let (offset, count) = self.bind_geometry(ready, geometry)?;
        // TEST: alpha_blend_matches_straight_alpha_over
        // SAFETY: the program and texture are of this context; `offset` is
        // a byte offset into the bound index buffer, checked against the
        // buffer's index count.
        unsafe {
            ready.gl.use_program(program.name);
            ready.gl.uniform2f(program.view, view.width, view.height);
            ready.gl.active_texture(TEXTURE0);
            ready.gl.bind_texture(TEXTURE_2D, texture_name);
            match pipeline.blend {
                Blend::Alpha => {
                    ready.gl.enable(BLEND);
                    ready.gl.blend_func_separate(
                        SRC_ALPHA,
                        ONE_MINUS_SRC_ALPHA,
                        ONE,
                        ONE_MINUS_SRC_ALPHA,
                    );
                }
                Blend::Opaque => ready.gl.disable(BLEND),
            }
            ready
                .gl
                .draw_elements(TRIANGLES, count, UNSIGNED_INT, without_provenance(offset));
        }
        Ok(())
    }
}

impl<C: GlContext> Drop for GlGpu<C> {
    fn drop(&mut self) {
        if let Ok(ready) = self.ready() {
            let [alpha, opaque] = self.programs;
            // TEST: dropped_handles_are_released_without_a_gl_error
            // SAFETY: every name was created by this context and is deleted
            // once; the context is current (`ready`).
            unsafe {
                ready.gl.delete_program(alpha.name);
                ready.gl.delete_program(opaque.name);
                ready.gl.delete_vertex_arrays(1, &raw const self.vao);
                ready.gl.delete_buffers(1, &raw const self.scratch_vbo);
                ready.gl.delete_buffers(1, &raw const self.scratch_ibo);
                ready.gl.delete_textures(1, &raw const self.white);
            }
        }
        CURRENT.with(|current| {
            if current.get() == self.id {
                current.set(0);
            }
        });
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
        if !indices_in_range(indices, vertices.len()) {
            return Err(GpuError::BadSize);
        }
        let ready = self.ready()?;
        let mut vertex_bytes = Vec::new();
        let mut index_bytes = Vec::new();
        encode_vertices(vertices, &mut vertex_bytes);
        encode_indices(indices, &mut index_bytes);
        let mut names = [0u32; 2];
        // TEST: buffer_geometry_matches_inline_geometry
        // SAFETY: `names` has room for two names; the byte vectors outlive
        // the calls and the sizes passed are their lengths.
        unsafe {
            ready.gl.gen_buffers(2, names.as_mut_ptr());
            ready.gl.bind_buffer(ARRAY_BUFFER, names[0]);
            ready.gl.buffer_data(
                ARRAY_BUFFER,
                gl_size(vertex_bytes.len())?,
                vertex_bytes.as_ptr().cast(),
                DYNAMIC_DRAW,
            );
            ready.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, names[1]);
            ready.gl.buffer_data(
                ELEMENT_ARRAY_BUFFER,
                gl_size(index_bytes.len())?,
                index_bytes.as_ptr().cast(),
                DYNAMIC_DRAW,
            );
        }
        let buffer = GlBuffer(Rc::new(BufferInner {
            owner: self.id,
            vbo: names[0],
            ibo: names[1],
            indices: u32::try_from(indices.len()).map_err(|_| GpuError::BadSize)?,
            release: Rc::clone(&self.release),
        }));
        self.finish()?;
        Ok(buffer)
    }

    fn create_texture(
        &mut self,
        desc: TextureDesc,
        rgba: &[u8],
    ) -> Result<Self::Texture, GpuError> {
        let (width, height) = (self.side(desc.width)?, self.side(desc.height)?);
        let expected = desc.byte_len().ok_or(GpuError::BadSize)?;
        if rgba.len() != expected {
            return Err(GpuError::BadPixelData {
                expected,
                got: rgba.len(),
            });
        }
        let ready = self.ready()?;
        let name = ready.texture_name();
        let texture = self.texture_handle(name, desc.width, desc.height, desc.filter);
        ready.texture_storage(name, width, height, desc.filter, rgba);
        self.finish()?;
        Ok(texture)
    }

    fn write_texture(
        &mut self,
        texture: &Self::Texture,
        rect: [u32; 4],
        rgba: &[u8],
    ) -> Result<(), GpuError> {
        self.check_owner(texture.0.owner)?;
        let [x, y, width, height] = rect;
        let fits = x
            .checked_add(width)
            .is_some_and(|end| end <= texture.0.width)
            && y.checked_add(height)
                .is_some_and(|end| end <= texture.0.height);
        if !fits {
            return Err(GpuError::BadSize);
        }
        let expected = TextureDesc {
            width,
            height,
            filter: Filter::Nearest,
        }
        .byte_len()
        .ok_or(GpuError::BadSize)?;
        if rgba.len() != expected {
            return Err(GpuError::BadPixelData {
                expected,
                got: rgba.len(),
            });
        }
        if expected == 0 {
            return Ok(());
        }
        let ready = self.ready()?;
        ready.sub_image(
            texture.0.name,
            [gl_int(x)?, gl_int(y)?, gl_int(width)?, gl_int(height)?],
            rgba,
        );
        self.finish()
    }

    fn create_target(&mut self, desc: TargetDesc) -> Result<Self::Target, GpuError> {
        let (width, height) = (self.side(desc.width)?, self.side(desc.height)?);
        let samples = i32::from(desc.samples).clamp(1, self.max_samples);
        let ready = self.ready()?;
        let name = ready.texture_name();
        let texture = self.texture_handle(name, desc.width, desc.height, desc.filter);
        ready.texture_storage(name, width, height, desc.filter, &[]);
        let (fbo, complete) = ready.texture_framebuffer(name);
        let msaa = if samples > 1 {
            let (msaa_fbo, rbo, msaa_complete) = ready.msaa_framebuffer(samples, width, height);
            Some((msaa_fbo, rbo, msaa_complete))
        } else {
            None
        };
        let target = GlTarget(Rc::new(TargetInner {
            owner: self.id,
            fbo,
            msaa: msaa.map(|(fbo, rbo, _)| (fbo, rbo)),
            texture,
            release: Rc::clone(&self.release),
        }));
        // TEST: triangle_set_reads_back_exactly
        // SAFETY: unbinding to the default framebuffer is always valid.
        unsafe { ready.gl.bind_framebuffer(FRAMEBUFFER, 0) };
        self.finish()?;
        if !complete || msaa.is_some_and(|(_, _, ok)| !ok) {
            return Err(backend("framebuffer is incomplete"));
        }
        Ok(target)
    }

    fn target_texture(&self, target: &Self::Target) -> Self::Texture {
        target.0.texture.clone()
    }

    fn set_filter(&mut self, texture: &Self::Texture, filter: Filter) -> Result<(), GpuError> {
        self.check_owner(texture.0.owner)?;
        let ready = self.ready()?;
        ready.sampling(texture.0.name, filter);
        texture.0.filter.set(filter);
        self.finish()
    }

    fn read_texture(&mut self, texture: &Self::Texture) -> Result<Vec<u8>, GpuError> {
        self.check_owner(texture.0.owner)?;
        let desc = TextureDesc {
            width: texture.0.width,
            height: texture.0.height,
            filter: texture.0.filter.get(),
        };
        let mut bytes = vec![0u8; desc.byte_len().ok_or(GpuError::BadSize)?];
        let (width, height) = (gl_int(desc.width)?, gl_int(desc.height)?);
        let ready = self.ready()?;
        let (fbo, complete) = ready.texture_framebuffer(texture.0.name);
        if complete {
            // TEST: texture_round_trips_through_write_and_read
            // SAFETY: `bytes` has `width * height * 4` bytes, which is what
            // `glReadPixels` writes with a pack alignment of 1.
            unsafe {
                ready.gl.pixel_storei(PACK_ALIGNMENT, 1);
                ready.gl.read_pixels(
                    0,
                    0,
                    width,
                    height,
                    RGBA,
                    UNSIGNED_BYTE,
                    bytes.as_mut_ptr().cast(),
                );
            }
        }
        // TEST: texture_round_trips_through_write_and_read
        // SAFETY: `fbo` is the framebuffer made above, deleted once.
        unsafe {
            ready.gl.bind_framebuffer(FRAMEBUFFER, 0);
            ready.gl.delete_framebuffers(1, &raw const fbo);
        }
        self.finish()?;
        if complete {
            Ok(bytes)
        } else {
            Err(backend(
                "cannot read the texture: framebuffer is incomplete",
            ))
        }
    }

    fn pipeline(&mut self, desc: PipelineDesc) -> Self::Pipeline {
        GlPipeline {
            owner: self.id,
            blend: desc.blend,
        }
    }

    fn submit(&mut self, list: &CommandList<Self>) -> Result<(), GpuError> {
        let ready = self.ready()?;
        let mut pass: Option<(Option<GlTarget>, View)> = None;
        for command in list.commands() {
            match command {
                Command::BeginPass {
                    target,
                    view,
                    clear,
                } => {
                    if pass.is_some() {
                        return Err(backend("BeginPass inside a pass"));
                    }
                    self.begin_pass(ready, target.as_ref(), *view, *clear)?;
                    pass = Some((target.clone(), *view));
                }
                Command::Draw {
                    pipeline,
                    texture,
                    geometry,
                } => {
                    let Some((_, view)) = &pass else {
                        return Err(backend("Draw outside a pass"));
                    };
                    let view = *view;
                    self.draw(ready, view, *pipeline, texture.as_ref(), geometry)?;
                }
                Command::EndPass => {
                    let Some((target, _)) = pass.take() else {
                        return Err(backend("EndPass outside a pass"));
                    };
                    Self::end_pass(ready, target.as_ref());
                }
            }
        }
        if let Some((target, _)) = pass {
            Self::end_pass(ready, target.as_ref());
            return Err(backend("the list ended inside a pass"));
        }
        self.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(x: f32, color: [u8; 4]) -> Vertex {
        Vertex {
            pos: [x, 2.0],
            uv: [0.25, 0.5],
            color,
        }
    }

    #[test]
    fn encode_vertices_writes_twenty_bytes_each() {
        let mut out = Vec::new();
        assert_eq!(usize::try_from(STRIDE_GL), Ok(STRIDE));

        encode_vertices(
            &[vertex(1.0, [1, 2, 3, 4]), vertex(3.0, [5, 6, 7, 8])],
            &mut out,
        );

        assert_eq!(out.len(), 2 * STRIDE);
        assert_eq!(out[0..4], 1.0f32.to_ne_bytes());
        assert_eq!(out[8..12], 0.25f32.to_ne_bytes());
        assert_eq!(out[16..20], [1, 2, 3, 4]);
        assert_eq!(out[STRIDE..STRIDE + 4], 3.0f32.to_ne_bytes());
    }

    #[test]
    fn encode_indices_writes_four_bytes_each_and_clears() {
        let mut out = vec![9; 3];

        encode_indices(&[1, 0x0102_0304], &mut out);

        assert_eq!(out.len(), 8);
        assert_eq!(out[4..], 0x0102_0304u32.to_ne_bytes());
    }

    #[test]
    fn indices_in_range_rejects_the_vertex_count() {
        assert!(indices_in_range(&[0, 1, 2], 3));
        assert!(!indices_in_range(&[0, 3], 3));
        assert!(indices_in_range(&[], 0));
    }

    #[test]
    fn error_names_cover_the_common_codes() {
        assert_eq!(error_name(0x0500), "GL_INVALID_ENUM");
        assert_eq!(error_name(0x1234), "unknown GL error");
    }
}
