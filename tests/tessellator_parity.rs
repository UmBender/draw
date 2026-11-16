//! The tessellated geometry against `draw_shape`, both on headless EGL
//! through the same `Gpu` (T179 AC-4, metric of ADR-T179-2).

#![allow(clippy::unwrap_used, clippy::cast_possible_truncation)]

use draw::core::camera::Camera;
use draw::core::geom::Vec2;
use draw::core::palette::ColorId;
use draw::core::shape::{CellFill, Shape, Style};
use draw::shell::render_ctx::RenderCtx;
use draw_gfx::gpu::gl::GlGpu;
use draw_gfx::gpu::{
    Blend, Camera as GfxCamera, Color, Command, CommandList, Filter, Gpu, PipelineDesc, TargetDesc,
    View,
};
use draw_gfx::tessellate::{tessellate_at, zoom_bucket};
use draw_sys::egl::EglContext;

/// Side of the square targets, in pixels.
const SIDE: u32 = 64;
/// Largest channel difference that does not count (the e2e tolerance).
const CHANNEL: u8 = 8;
/// Largest relative difference of the total ink.
const INK_TOLERANCE: f32 = 0.04;
/// The shapes are 40 px wide on screen, whatever the zoom.
const EXTENT_PX: f32 = 40.0;

type Ctx = RenderCtx<GlGpu<EglContext>>;

fn context() -> Ctx {
    let egl = EglContext::headless(SIDE, SIDE).unwrap();
    RenderCtx::new(GlGpu::new(egl, (SIDE, SIDE)).unwrap())
}

fn view() -> View {
    View {
        width: SIDE as f32,
        height: SIDE as f32,
    }
}

fn white() -> Color {
    Color::rgba(1.0, 1.0, 1.0, 1.0)
}

fn target(ctx: &mut Ctx) -> <GlGpu<EglContext> as Gpu>::Target {
    ctx.gpu_mut()
        .create_target(TargetDesc {
            width: SIDE,
            height: SIDE,
            samples: 1,
            filter: Filter::Nearest,
        })
        .unwrap()
}

fn read(ctx: &mut Ctx, target: &<GlGpu<EglContext> as Gpu>::Target) -> Vec<u8> {
    let gpu = ctx.gpu_mut();
    let texture = gpu.target_texture(target);
    gpu.read_texture(&texture).unwrap()
}

/// `draw_shape` through `RenderCtx`.
fn reference(ctx: &mut Ctx, shape: &Shape, camera: &Camera) -> Vec<u8> {
    let target = target(ctx);
    ctx.set_window(view(), 1.0);
    ctx.render_to(&target, view(), Some(white()), |ctx| {
        ctx.draw_preview(shape, camera);
    });
    read(ctx, &target)
}

/// The tessellated mesh drawn feathered through the camera uniform.
fn tessellated(ctx: &mut Ctx, shape: &Shape, camera: &Camera) -> Vec<u8> {
    let target = target(ctx);
    let mesh = tessellate_at(shape, zoom_bucket(camera.zoom()));
    let gpu = ctx.gpu_mut();
    let pipeline = gpu.pipeline(PipelineDesc {
        blend: Blend::AlphaFeathered,
    });
    let mut list = CommandList::<GlGpu<EglContext>>::new();
    list.push(Command::BeginPass {
        target: Some(target.clone()),
        view: view(),
        clear: Some(white()),
    });
    list.push(Command::SetCamera(GfxCamera {
        origin: [camera.offset().x, camera.offset().y],
        scale: camera.zoom(),
    }));
    list.push(Command::Draw {
        pipeline,
        texture: None,
        geometry: mesh.geometry(),
    });
    list.push(Command::EndPass);
    ctx.gpu_mut().submit(&list).unwrap();
    read(ctx, &target)
}

fn at(bytes: &[u8], x: i32, y: i32) -> [u8; 4] {
    let x = x.clamp(0, SIDE as i32 - 1) as usize;
    let y = y.clamp(0, SIDE as i32 - 1) as usize;
    let i = (y * SIDE as usize + x) * 4;
    [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]
}

/// `255 - green`, summed: how much ink the image holds.
fn ink(bytes: &[u8]) -> f32 {
    bytes
        .chunks_exact(4)
        .map(|p| f32::from(255 - p[1]))
        .sum::<f32>()
}

/// Checks ADR-T179-2 and returns a description of the first failure.
fn compare(reference: &[u8], candidate: &[u8]) -> Result<(), String> {
    for y in 0..SIDE as i32 {
        for x in 0..SIDE as i32 {
            let (r, c) = (at(reference, x, y), at(candidate, x, y));
            if r.iter().zip(&c).all(|(a, b)| a.abs_diff(*b) <= CHANNEL) {
                continue;
            }
            let uniform = (-1..=1)
                .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
                .all(|(dx, dy)| at(reference, x + dx, y + dy) == r);
            if uniform {
                return Err(format!(
                    "pixel ({x}, {y}) differs away from any edge: {r:?} against {c:?}"
                ));
            }
        }
    }
    let (want, got) = (ink(reference), ink(candidate));
    if (want - got).abs() > want.max(1.0) * INK_TOLERANCE {
        return Err(format!("ink {got} against {want}"));
    }
    Ok(())
}

/// A camera at `zoom` that puts world `center` in the middle of the target.
fn camera(center: Vec2, zoom: f32) -> Camera {
    let half = SIDE as f32 / 2.0 / zoom;
    Camera::new(Vec2::new(center.x - half, center.y - half), zoom)
}

/// Runs `make` (given the world size of 40 px) at the three zooms.
fn assert_parity(name: &str, make: impl Fn(f32) -> Shape) {
    let mut ctx = context();
    for zoom in [0.25_f32, 1.0, 4.0] {
        let shape = make(EXTENT_PX / zoom);
        let camera = camera(Vec2::new(100.0, 100.0), zoom);
        let want = reference(&mut ctx, &shape, &camera);
        let got = tessellated(&mut ctx, &shape, &camera);
        if let Err(why) = compare(&want, &got) {
            panic!("{name} at zoom {zoom}: {why}");
        }
    }
}

/// A point `(fx, fy)` of the 40 px box, in world units.
fn p(extent: f32, fx: f32, fy: f32) -> Vec2 {
    Vec2::new(100.0 + (fx - 0.5) * extent, 100.0 + (fy - 0.5) * extent)
}

/// A world width of 2: under a pixel at zoom 0.25, 8 px at zoom 4.
const STYLE: Style = Style {
    color: ColorId::INK,
    width: 2.0,
};

#[test]
fn parity_stroke() {
    assert_parity("stroke", |e| Shape::Stroke {
        points: vec![
            p(e, 0.1, 0.8),
            p(e, 0.3, 0.2),
            p(e, 0.5, 0.7),
            p(e, 0.7, 0.2),
            p(e, 0.9, 0.8),
        ],
        widths: None,
        style: STYLE,
    });
}

#[test]
fn parity_pressure_stroke() {
    assert_parity("pressure stroke", |e| Shape::Stroke {
        points: vec![p(e, 0.1, 0.5), p(e, 0.35, 0.3), p(e, 0.65, 0.7), p(e, 0.9, 0.5)],
        widths: Some(vec![0.5, 1.0, 2.0, 1.0]),
        style: STYLE,
    });
}

#[test]
fn parity_line() {
    assert_parity("line", |e| Shape::Line {
        a: p(e, 0.1, 0.2),
        b: p(e, 0.9, 0.8),
        style: STYLE,
    });
}

#[test]
fn parity_arrow() {
    assert_parity("arrow", |e| Shape::Arrow {
        a: p(e, 0.1, 0.8),
        b: p(e, 0.9, 0.3),
        style: STYLE,
    });
}

#[test]
fn parity_rect() {
    for fill in [None, ColorId::new(2)] {
        assert_parity("rect", |e| Shape::Rect {
            a: p(e, 0.1, 0.2),
            b: p(e, 0.9, 0.8),
            style: STYLE,
            fill,
            label: None,
            text: None,
        });
    }
}

#[test]
fn parity_ellipse() {
    for fill in [None, ColorId::new(3)] {
        assert_parity("ellipse", |e| Shape::Ellipse {
            a: p(e, 0.1, 0.2),
            b: p(e, 0.9, 0.8),
            style: STYLE,
            fill,
            label: None,
            text: None,
        });
    }
}

#[test]
fn parity_grid() {
    assert_parity("grid", |e| Shape::Grid {
        a: p(e, 0.1, 0.1),
        b: p(e, 0.9, 0.9),
        cols: 3,
        rows: 2,
        style: STYLE,
        axes: false,
        fills: vec![
            CellFill {
                col: 0,
                row: 0,
                color: ColorId::new(2).unwrap(),
            },
            CellFill {
                col: 2,
                row: 1,
                color: ColorId::new(4).unwrap(),
            },
        ],
        texts: Vec::new(),
    });
}
