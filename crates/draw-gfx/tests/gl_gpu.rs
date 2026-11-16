//! The GL `Gpu` on a headless EGL context (T145 AC-3). Pixel reads are exact
//! where the geometry is aligned to pixel edges.

#![allow(clippy::unwrap_used)]

mod common {
    pub mod scene;
}

use common::scene::{SIDE, draw_to_target, pixel, rect, vertex};
use draw_gfx::gpu::gl::GlGpu;
use draw_gfx::gpu::{
    Blend, Camera, Color, Command, CommandList, Filter, Geometry, Gpu, GpuError, PipelineDesc,
    TargetDesc, TextureDesc, Vertex, View,
};
use draw_sys::egl::EglContext;

fn gpu() -> GlGpu<EglContext> {
    let ctx = EglContext::headless(64, 64).unwrap();
    GlGpu::new(ctx, (64, 64)).unwrap()
}

#[test]
fn triangle_set_reads_back_exactly() {
    let mut gpu = gpu();

    common::scene::assert_triangle_set_reads_back_exactly(&mut gpu);
}

#[test]
fn texture_round_trips_through_write_and_read() {
    let mut gpu = gpu();
    let desc = TextureDesc {
        width: 2,
        height: 2,
        filter: Filter::Nearest,
    };
    let texture = gpu.create_texture(desc, &[1; 16]).unwrap();

    gpu.write_texture(&texture, [1, 0, 1, 1], &[9, 8, 7, 6])
        .unwrap();
    let bytes = gpu.read_texture(&texture).unwrap();

    let mut want = vec![1u8; 16];
    want[4..8].copy_from_slice(&[9, 8, 7, 6]);
    assert_eq!(bytes, want);
}

#[test]
fn alpha_blend_matches_straight_alpha_over() {
    let mut gpu = gpu();
    let (vertices, indices) = rect(0.0, 0.0, 16.0, 16.0, [0, 0, 255, 128]);

    let bytes = draw_to_target(
        &mut gpu,
        Color::rgba(1.0, 1.0, 1.0, 1.0),
        Blend::Alpha,
        Geometry::Inline { vertices, indices },
    );

    let got = pixel(&bytes, 8, 8);
    assert!(got[0].abs_diff(127) <= 1, "{got:?}");
    assert!(got[1].abs_diff(127) <= 1, "{got:?}");
    assert_eq!(got[2], 255);
    assert_eq!(got[3], 255);
}

#[test]
fn buffer_geometry_matches_inline_geometry() {
    let mut gpu = gpu();
    let (vertices, indices) = rect(4.0, 4.0, 12.0, 12.0, [255, 0, 0, 255]);
    let buffer = gpu.create_buffer(&vertices, &indices).unwrap();
    let black = Color::rgba(0.0, 0.0, 0.0, 1.0);

    let inline = draw_to_target(
        &mut gpu,
        black,
        Blend::Opaque,
        Geometry::Inline { vertices, indices },
    );
    let buffered = draw_to_target(
        &mut gpu,
        black,
        Blend::Opaque,
        Geometry::Buffer {
            buffer: buffer.clone(),
            first: 0,
            count: 6,
        },
    );
    let second_triangle = draw_to_target(
        &mut gpu,
        black,
        Blend::Opaque,
        Geometry::Buffer {
            buffer,
            first: 3,
            count: 3,
        },
    );

    assert_eq!(inline, buffered);
    assert_ne!(inline, second_triangle);
    assert_eq!(pixel(&second_triangle, 5, 10), [255, 0, 0, 255]);
    assert_eq!(pixel(&second_triangle, 10, 5), [0, 0, 0, 255]);
}

#[test]
fn set_filter_changes_sampling() {
    let mut gpu = gpu();
    let texture = gpu
        .create_texture(
            TextureDesc {
                width: 2,
                height: 1,
                filter: Filter::Nearest,
            },
            &[0, 0, 0, 255, 255, 255, 255, 255],
        )
        .unwrap();
    let target = gpu
        .create_target(TargetDesc {
            width: 8,
            height: 1,
            samples: 1,
            filter: Filter::Nearest,
        })
        .unwrap();
    let pipeline = gpu.pipeline(PipelineDesc {
        blend: Blend::Opaque,
    });
    let white = [255; 4];
    let quad = |uv_right: f32| {
        let mut vertices = vec![
            vertex(0.0, 0.0, white),
            vertex(8.0, 0.0, white),
            vertex(8.0, 1.0, white),
            vertex(0.0, 1.0, white),
        ];
        vertices[1].uv = [uv_right, 0.0];
        vertices[2].uv = [uv_right, 1.0];
        vertices[3].uv = [0.0, 1.0];
        Geometry::Inline {
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    };
    let render = |gpu: &mut GlGpu<EglContext>| {
        let mut list = CommandList::<GlGpu<EglContext>>::new();
        list.push(Command::BeginPass {
            target: Some(target.clone()),
            view: View {
                width: 8.0,
                height: 1.0,
            },
            clear: Some(Color::rgba(0.0, 0.0, 0.0, 1.0)),
        });
        list.push(Command::Draw {
            pipeline,
            texture: Some(texture.clone()),
            geometry: quad(1.0),
        });
        list.push(Command::EndPass);
        gpu.submit(&list).unwrap();
        let out = gpu.target_texture(&target);
        gpu.read_texture(&out).unwrap()
    };

    let nearest = render(&mut gpu);
    gpu.set_filter(&texture, Filter::Linear).unwrap();
    let linear = render(&mut gpu);

    assert!(nearest.chunks(4).all(|p| p[0] == 0 || p[0] == 255));
    assert!(linear.chunks(4).any(|p| p[0] > 0 && p[0] < 255));
}

/// Renders `passes` (each a camera and a draw) into one `SIDE` target and
/// reads it back.
fn render_with_cameras(
    gpu: &mut GlGpu<EglContext>,
    blend: Blend,
    passes: &[(Option<Camera>, Vec<Vertex>, Vec<u32>)],
) -> Vec<u8> {
    let target = gpu
        .create_target(TargetDesc {
            width: SIDE,
            height: SIDE,
            samples: 1,
            filter: Filter::Nearest,
        })
        .unwrap();
    let pipeline = gpu.pipeline(PipelineDesc { blend });
    let mut list = CommandList::<GlGpu<EglContext>>::new();
    for (n, (camera, vertices, indices)) in passes.iter().enumerate() {
        list.push(Command::BeginPass {
            target: Some(target.clone()),
            view: View {
                width: SIDE as f32,
                height: SIDE as f32,
            },
            clear: (n == 0).then(|| Color::rgba(0.0, 0.0, 0.0, 1.0)),
        });
        if let Some(camera) = camera {
            list.push(Command::SetCamera(*camera));
        }
        list.push(Command::Draw {
            pipeline,
            texture: None,
            geometry: Geometry::Inline {
                vertices: vertices.clone(),
                indices: indices.clone(),
            },
        });
        list.push(Command::EndPass);
    }
    gpu.submit(&list).unwrap();
    let texture = gpu.target_texture(&target);
    gpu.read_texture(&texture).unwrap()
}

#[test]
fn gl_camera_pan_moves_the_triangle() {
    let mut gpu = gpu();
    let (vertices, indices) = rect(4.0, 4.0, 12.0, 12.0, [255, 0, 0, 255]);
    let pan = Camera {
        origin: [2.0, 0.0],
        scale: 1.0,
    };

    let bytes = render_with_cameras(&mut gpu, Blend::Opaque, &[(Some(pan), vertices, indices)]);

    assert_eq!(pixel(&bytes, 3, 8), [255, 0, 0, 255]);
    assert_eq!(pixel(&bytes, 11, 8), [0, 0, 0, 255]);
}

#[test]
fn gl_camera_zoom_scales_the_triangle() {
    let mut gpu = gpu();
    let (vertices, indices) = rect(2.0, 2.0, 6.0, 6.0, [255, 0, 0, 255]);
    let zoom = Camera {
        origin: [0.0, 0.0],
        scale: 2.0,
    };

    let bytes = render_with_cameras(&mut gpu, Blend::Opaque, &[(Some(zoom), vertices, indices)]);

    assert_eq!(pixel(&bytes, 4, 4), [255, 0, 0, 255]);
    assert_eq!(pixel(&bytes, 11, 11), [255, 0, 0, 255]);
    assert_eq!(pixel(&bytes, 12, 12), [0, 0, 0, 255]);
    assert_eq!(pixel(&bytes, 3, 3), [0, 0, 0, 255]);
}

#[test]
fn gl_camera_resets_at_begin_pass() {
    let mut gpu = gpu();
    let (vertices, indices) = rect(4.0, 4.0, 12.0, 12.0, [255, 0, 0, 255]);
    let away = Camera {
        origin: [100.0, 0.0],
        scale: 1.0,
    };

    let bytes = render_with_cameras(
        &mut gpu,
        Blend::Opaque,
        &[
            (Some(away), vertices.clone(), indices.clone()),
            (None, vertices, indices),
        ],
    );

    assert_eq!(pixel(&bytes, 8, 8), [255, 0, 0, 255]);
}

/// A white strip whose left edge is at world `x = 10`, feathered through
/// `uv[0]` (world distance inside the edge), seen with `scale` so that the
/// edge falls at pixel `8.5`.
fn feather_strip(scale: f32) -> (Camera, Vec<Vertex>, Vec<u32>) {
    let camera = Camera {
        origin: [10.0 - 8.5 / scale, 0.0],
        scale,
    };
    let reach = 40.0 / scale;
    let white = [255; 4];
    let mut vertices = vec![
        vertex(10.0, -1.0, white),
        vertex(10.0 + reach, -1.0, white),
        vertex(10.0 + reach, 20.0 / scale, white),
        vertex(10.0, 20.0 / scale, white),
    ];
    for (v, d) in vertices.iter_mut().zip([0.0, reach, reach, 0.0]) {
        v.uv = [d, 0.0];
    }
    (camera, vertices, vec![0, 1, 2, 0, 2, 3])
}

fn feather_row(scale: f32) -> Vec<[u8; 4]> {
    let mut gpu = gpu();
    let (camera, vertices, indices) = feather_strip(scale);
    let bytes = render_with_cameras(
        &mut gpu,
        Blend::AlphaFeathered,
        &[(Some(camera), vertices, indices)],
    );
    (0..SIDE).map(|x| pixel(&bytes, x, 8)).collect()
}

fn assert_one_pixel_feather(row: &[[u8; 4]]) {
    assert_eq!(row[6], [0, 0, 0, 255]);
    assert_eq!(row[7], [0, 0, 0, 255]);
    let edge = i32::from(row[8][0]);
    assert!((edge - 128).abs() <= 12, "edge pixel was {edge}");
    assert_eq!(row[9], [255, 255, 255, 255]);
    assert_eq!(row[13], [255, 255, 255, 255]);
}

#[test]
fn feather_edge_is_one_pixel_wide_at_zoom_quarter() {
    assert_one_pixel_feather(&feather_row(0.25));
}

#[test]
fn feather_edge_is_one_pixel_wide_at_zoom_one() {
    assert_one_pixel_feather(&feather_row(1.0));
}

#[test]
fn feather_edge_is_one_pixel_wide_at_zoom_four() {
    assert_one_pixel_feather(&feather_row(4.0));
}

#[test]
fn feather_interior_is_opaque() {
    let row = feather_row(1.0);

    assert!(row[10..].iter().all(|p| *p == [255, 255, 255, 255]));
}

#[test]
fn multisampled_target_resolves() {
    let mut gpu = gpu();
    let (vertices, indices) = rect(4.0, 4.0, 12.0, 12.0, [255, 0, 0, 255]);
    let target = gpu
        .create_target(TargetDesc {
            width: SIDE,
            height: SIDE,
            samples: 4,
            filter: Filter::Nearest,
        })
        .unwrap();
    let pipeline = gpu.pipeline(PipelineDesc {
        blend: Blend::Opaque,
    });
    let mut list = CommandList::<GlGpu<EglContext>>::new();
    list.push(Command::BeginPass {
        target: Some(target.clone()),
        view: View {
            width: SIDE as f32,
            height: SIDE as f32,
        },
        clear: Some(Color::rgba(0.0, 0.0, 0.0, 1.0)),
    });
    list.push(Command::Draw {
        pipeline,
        texture: None,
        geometry: Geometry::Inline { vertices, indices },
    });
    list.push(Command::EndPass);

    gpu.submit(&list).unwrap();
    let texture = gpu.target_texture(&target);
    let bytes = gpu.read_texture(&texture).unwrap();

    assert_eq!(pixel(&bytes, 8, 8), [255, 0, 0, 255]);
    assert_eq!(pixel(&bytes, 1, 1), [0, 0, 0, 255]);
}

#[test]
fn foreign_handle_is_unknown() {
    let mut first = gpu();
    let mut second = gpu();
    let desc = TextureDesc {
        width: 1,
        height: 1,
        filter: Filter::Nearest,
    };
    let texture = first.create_texture(desc, &[0; 4]).unwrap();

    let written = second.write_texture(&texture, [0, 0, 1, 1], &[0; 4]);
    let read = second.read_texture(&texture);

    assert_eq!(written, Err(GpuError::UnknownHandle));
    assert_eq!(read, Err(GpuError::UnknownHandle));
}

#[test]
fn bad_pixel_data_is_reported() {
    let mut gpu = gpu();
    let desc = TextureDesc {
        width: 2,
        height: 2,
        filter: Filter::Nearest,
    };

    let err = gpu.create_texture(desc, &[0; 15]).err();

    assert_eq!(
        err,
        Some(GpuError::BadPixelData {
            expected: 16,
            got: 15
        })
    );
}

#[test]
fn zero_size_target_is_bad_size() {
    let mut gpu = gpu();

    let err = gpu
        .create_target(TargetDesc {
            width: 0,
            height: 4,
            samples: 1,
            filter: Filter::Nearest,
        })
        .err();

    assert_eq!(err, Some(GpuError::BadSize));
}

#[test]
fn texture_larger_than_the_limit_is_bad_size() {
    let mut gpu = gpu();
    let desc = TextureDesc {
        width: 1 << 24,
        height: 1,
        filter: Filter::Nearest,
    };

    let err = gpu.create_texture(desc, &vec![0; 4 << 24]).err();

    assert_eq!(err, Some(GpuError::BadSize));
}

#[test]
fn write_texture_outside_the_texture_is_bad_size() {
    let mut gpu = gpu();
    let desc = TextureDesc {
        width: 2,
        height: 2,
        filter: Filter::Nearest,
    };
    let texture = gpu.create_texture(desc, &[0; 16]).unwrap();

    let err = gpu.write_texture(&texture, [1, 1, 2, 2], &[0; 16]);

    assert_eq!(err, Err(GpuError::BadSize));
}

#[test]
fn index_out_of_the_vertices_is_an_error_value() {
    let mut gpu = gpu();
    let (vertices, _) = rect(0.0, 0.0, 4.0, 4.0, [0; 4]);

    let err = gpu.create_buffer(&vertices, &[0, 1, 9]).err();

    assert_eq!(err, Some(GpuError::BadSize));
}

#[cfg(debug_assertions)]
#[test]
fn bad_gl_call_is_reported_with_its_name() {
    let gpu = gpu();
    let gl = gpu.gl();

    // SAFETY: an invalid texture target makes GL set an error and nothing
    // else; the context of `gpu` is current on this thread.
    unsafe { gl.bind_texture(0xFFFF, 0) };

    let error = gl.take_error();
    assert!(
        error
            .as_deref()
            .is_some_and(|e| e.contains("glBindTexture")),
        "{error:?}"
    );
    assert_eq!(gl.take_error(), None);
}

#[test]
fn dropped_handles_are_released_without_a_gl_error() {
    let mut gpu = gpu();
    let desc = TextureDesc {
        width: 4,
        height: 4,
        filter: Filter::Nearest,
    };
    for _ in 0..8 {
        let texture = gpu.create_texture(desc, &[0; 64]).unwrap();
        drop(texture);
        let (vertices, indices) = rect(0.0, 0.0, 1.0, 1.0, [0; 4]);
        let _ = gpu.create_buffer(&vertices, &indices).unwrap();
    }

    let list = CommandList::<GlGpu<EglContext>>::new();

    assert_eq!(gpu.submit(&list), Ok(()));
}
