//! Scenes shared by the EGL and GLX tests: they only need a `Gpu`.

#![allow(clippy::unwrap_used, dead_code)]

use draw_gfx::gpu::{
    Blend, Color, Command, CommandList, Filter, Geometry, Gpu, PipelineDesc, TargetDesc, Vertex,
    View,
};

/// Side of the square targets of the scenes.
pub const SIDE: u32 = 16;

/// A vertex at `(x, y)` with `color`.
pub fn vertex(x: f32, y: f32, color: [u8; 4]) -> Vertex {
    Vertex {
        pos: [x, y],
        uv: [0.0, 0.0],
        color,
    }
}

/// The rectangle `[x0, x1) × [y0, y1)` as four vertices and six indices.
pub fn rect(x0: f32, y0: f32, x1: f32, y1: f32, color: [u8; 4]) -> (Vec<Vertex>, Vec<u32>) {
    (
        vec![
            vertex(x0, y0, color),
            vertex(x1, y0, color),
            vertex(x1, y1, color),
            vertex(x0, y1, color),
        ],
        vec![0, 1, 2, 0, 2, 3],
    )
}

/// The RGBA bytes of the pixel `(x, y)` (origin top left) in a read-back of a
/// target, whose rows are stored bottom first (ADR-T159-1).
pub fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let row = SIDE - 1 - y;
    let at = ((row * SIDE + x) * 4) as usize;
    [bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]
}

/// Clears a `SIDE × SIDE` target to `clear`, draws `geometry` into it with
/// `blend` and returns the read-back bytes.
pub fn draw_to_target<G: Gpu>(
    gpu: &mut G,
    clear: Color,
    blend: Blend,
    geometry: Geometry<G::Buffer>,
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
    let mut list = CommandList::<G>::new();
    list.push(Command::BeginPass {
        target: Some(target.clone()),
        view: View {
            width: SIDE as f32,
            height: SIDE as f32,
        },
        clear: Some(clear),
    });
    list.push(Command::Draw {
        pipeline,
        texture: None,
        geometry,
    });
    list.push(Command::EndPass);
    gpu.submit(&list).unwrap();
    let texture = gpu.target_texture(&target);
    gpu.read_texture(&texture).unwrap()
}

/// Draws the known triangle set (a red square from 4 to 12 on a black
/// target) and checks every pixel.
pub fn assert_triangle_set_reads_back_exactly<G: Gpu>(gpu: &mut G) {
    let (vertices, indices) = rect(4.0, 4.0, 12.0, 12.0, [255, 0, 0, 255]);

    let bytes = draw_to_target(
        gpu,
        Color::rgba(0.0, 0.0, 0.0, 1.0),
        Blend::Opaque,
        Geometry::Inline { vertices, indices },
    );

    assert_eq!(bytes.len(), (SIDE * SIDE * 4) as usize);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let inside = (4..12).contains(&x) && (4..12).contains(&y);
            let want = if inside {
                [255, 0, 0, 255]
            } else {
                [0, 0, 0, 255]
            };
            assert_eq!(pixel(&bytes, x, y), want, "pixel ({x}, {y})");
        }
    }
}
