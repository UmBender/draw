//! The GL `Gpu` over a GLX context under a private Xvfb (T145 AC-1, AC-3).

#![allow(clippy::unwrap_used)]

#[path = "../../draw-sys/tests/common/xvfb.rs"]
mod xvfb;

mod common {
    pub mod scene;
}

use draw_gfx::gpu::gl::GlGpu;
use draw_sys::glx::GlxContext;

#[test]
fn glx_gpu_draws_triangle() {
    let server = xvfb::Xvfb::start();
    let ctx = GlxContext::pbuffer(Some(&server.display), 64, 64).unwrap();
    let mut gpu = GlGpu::new(ctx, (64, 64)).unwrap();

    common::scene::assert_triangle_set_reads_back_exactly(&mut gpu);
}
