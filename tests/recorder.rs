//! The recording and null interpreters of `Gpu` and `Platform` (T140, AC-1).

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use draw::shell::recording::{GpuCall, NullGpu, PlatformCall, RecordingGpu, RecordingPlatform};
use draw_gfx::gpu::{
    Blend, Color, Command, CommandList, Filter, Geometry, Gpu, GpuError, PipelineDesc, TargetDesc,
    TextureDesc, Vertex, View,
};
use draw_sys::platform::{Platform, RawButton, RawEvent, RawKey, RawMods, WindowMetrics};

const METRICS: WindowMetrics = WindowMetrics {
    width: 800.0,
    height: 600.0,
    scale: 1.0,
};

fn vertex(x: f32, y: f32) -> Vertex {
    Vertex {
        pos: [x, y],
        uv: [0.0, 0.0],
        color: [255, 255, 255, 255],
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
    }
}

#[test]
fn recorder_keeps_the_call_order() -> Result<(), GpuError> {
    // Arrange
    let mut gpu = RecordingGpu::new();
    let texture_desc = TextureDesc {
        width: 2,
        height: 2,
        filter: Filter::Nearest,
    };
    // Act
    let buffer = gpu.create_buffer(&[vertex(0.0, 0.0)], &[0])?;
    let texture = gpu.create_texture(texture_desc, &[0; 16])?;
    let target = gpu.create_target(TargetDesc {
        width: 4,
        height: 4,
        samples: 1,
        filter: Filter::Nearest,
    })?;
    let pipeline = gpu.pipeline(PipelineDesc {
        blend: Blend::Alpha,
    });
    let mut list = CommandList::<RecordingGpu>::new();
    list.push(Command::BeginPass {
        target: Some(target),
        view: View {
            width: 4.0,
            height: 4.0,
        },
        clear: Some(Color::rgba(0.0, 0.0, 0.0, 1.0)),
    });
    list.push(Command::Draw {
        pipeline,
        texture: Some(texture),
        geometry: Geometry::Buffer {
            buffer,
            first: 0,
            count: 1,
        },
    });
    list.push(Command::EndPass);
    gpu.submit(&list)?;
    // Assert
    let kinds: Vec<&str> = gpu
        .calls()
        .iter()
        .map(|call| match call {
            GpuCall::CreateBuffer { .. } => "buffer",
            GpuCall::CreateTexture { .. } => "texture",
            GpuCall::WriteTexture { .. } => "write",
            GpuCall::CreateTarget { .. } => "target",
            GpuCall::Pipeline { .. } => "pipeline",
            GpuCall::SetFilter { .. } => "filter",
            GpuCall::ReadTexture { .. } => "read",
            GpuCall::Submit { .. } => "submit",
        })
        .collect();
    assert_eq!(kinds, ["buffer", "texture", "target", "pipeline", "submit"]);
    let Some(GpuCall::Submit { commands }) = gpu.calls().last() else {
        panic!("the last call is the submit");
    };
    assert_eq!(commands.len(), 3);
    assert!(matches!(commands[2], Command::EndPass));
    Ok(())
}

#[test]
fn recorder_gpu_hands_out_distinct_handles() -> Result<(), GpuError> {
    // Arrange
    let mut gpu = RecordingGpu::new();
    let desc = TextureDesc {
        width: 1,
        height: 1,
        filter: Filter::Linear,
    };
    // Act
    let a = gpu.create_texture(desc, &[0; 4])?;
    let b = gpu.create_texture(desc, &[0; 4])?;
    // Assert
    assert_ne!(a, b);
    Ok(())
}

#[test]
fn recorder_gpu_rejects_a_texture_with_the_wrong_byte_count() {
    // Arrange
    let mut gpu = RecordingGpu::new();
    let desc = TextureDesc {
        width: 2,
        height: 2,
        filter: Filter::Linear,
    };
    // Act
    let result = gpu.create_texture(desc, &[0; 3]);
    // Assert
    assert!(result.is_err());
    assert_eq!(gpu.calls().len(), 0);
}

// ---- T159 AC-2: filter and readback ------------------------------------------

#[test]
fn recording_gpu_records_set_filter_and_read_texture() -> Result<(), GpuError> {
    // Arrange
    let mut gpu = RecordingGpu::new();
    let desc = TextureDesc {
        width: 2,
        height: 1,
        filter: Filter::Nearest,
    };
    let texture = gpu.create_texture(desc, &[0; 8])?;
    // Act
    gpu.set_filter(&texture, Filter::Linear)?;
    gpu.read_texture(&texture)?;
    // Assert
    let calls = gpu.calls();
    assert_eq!(
        calls[calls.len() - 2],
        GpuCall::SetFilter {
            texture,
            filter: Filter::Linear
        }
    );
    assert_eq!(calls[calls.len() - 1], GpuCall::ReadTexture { texture });
    Ok(())
}

#[test]
fn recording_gpu_read_texture_returns_one_rgba_image() -> Result<(), GpuError> {
    // Arrange
    let mut gpu = RecordingGpu::new();
    let target = gpu.create_target(TargetDesc {
        width: 3,
        height: 2,
        samples: 1,
        filter: Filter::Linear,
    })?;
    let texture = gpu.target_texture(&target);
    // Act
    let pixels = gpu.read_texture(&texture)?;
    // Assert
    assert_eq!(pixels.len(), 3 * 2 * 4);
    Ok(())
}

#[test]
fn recording_gpu_set_filter_unknown_handle_fails() {
    // Arrange
    let mut gpu = RecordingGpu::new();
    let other = RecordingGpu::new().create_buffer(&[], &[]);
    let Ok(handle) = other else {
        panic!("buffer");
    };
    // Act
    let result = gpu.set_filter(&handle, Filter::Linear);
    // Assert
    assert_eq!(result, Err(GpuError::UnknownHandle));
    assert_eq!(gpu.read_texture(&handle), Err(GpuError::UnknownHandle));
}

#[test]
fn null_gpu_read_texture_returns_zeroed_pixels() -> Result<(), GpuError> {
    // Arrange
    let mut gpu = NullGpu::default();
    let desc = TextureDesc {
        width: 2,
        height: 2,
        filter: Filter::Nearest,
    };
    let texture = gpu.create_texture(desc, &[7; 16])?;
    // Act
    let pixels = gpu.read_texture(&texture)?;
    // Assert
    assert_eq!(pixels, vec![0; 16]);
    Ok(())
}

#[test]
fn recorder_platform_replays_queued_events_in_order() {
    // Arrange
    let mut platform = RecordingPlatform::new(METRICS);
    let first = RawEvent::Key {
        key: RawKey::Letter('A'),
        mods: RawMods::default(),
        down: true,
        repeat: false,
    };
    let second = RawEvent::Button {
        button: RawButton::Left,
        x: 3.0,
        y: 4.0,
        down: true,
    };
    platform.queue(first);
    platform.queue(second);
    let mut out = Vec::new();
    // Act
    platform.poll(&mut out);
    let mut again = Vec::new();
    platform.poll(&mut again);
    // Assert
    assert_eq!(out, [first, second]);
    assert_eq!(again, []);
}

#[test]
fn recorder_platform_logs_clipboard_and_wakeups() {
    // Arrange
    let mut platform = RecordingPlatform::new(METRICS);
    platform.set_clipboard_text("1 2\n");
    // Act
    platform.schedule_update();
    let text = platform.clipboard_text();
    platform.set_clipboard_png(&[1, 2, 3]);
    block_on(platform.next_frame());
    // Assert
    assert_eq!(text.as_deref(), Some("1 2\n"));
    assert_eq!(platform.window(), METRICS);
    assert_eq!(
        platform.calls(),
        [
            PlatformCall::SetClipboardText("1 2\n".to_owned()),
            PlatformCall::ScheduleUpdate,
            PlatformCall::ClipboardText,
            PlatformCall::SetClipboardPng { bytes: 3 },
            PlatformCall::NextFrame,
            PlatformCall::Window,
        ]
    );
}

#[test]
fn null_gpu_accepts_a_command_list() {
    // Arrange
    let mut gpu = NullGpu::default();
    let mut list = CommandList::<NullGpu>::new();
    list.push(Command::BeginPass {
        target: None,
        view: View {
            width: 1.0,
            height: 1.0,
        },
        clear: None,
    });
    list.push(Command::Draw {
        pipeline: gpu.pipeline(PipelineDesc {
            blend: Blend::Opaque,
        }),
        texture: None,
        geometry: Geometry::Inline {
            vertices: vec![vertex(0.0, 0.0), vertex(1.0, 0.0), vertex(0.0, 1.0)],
            indices: vec![0, 1, 2],
        },
    });
    list.push(Command::EndPass);
    // Act
    let result = gpu.submit(&list);
    // Assert
    assert!(result.is_ok());
}

#[test]
fn command_list_clear_keeps_capacity() {
    // Arrange
    let mut list = CommandList::<NullGpu>::new();
    for _ in 0..64 {
        list.push(Command::EndPass);
    }
    let capacity = list.capacity();
    // Act
    list.clear();
    // Assert
    assert!(list.is_empty());
    assert_eq!(list.capacity(), capacity);
}
