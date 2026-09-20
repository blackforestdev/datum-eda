//! Explicit diagnostic: allocation contribution, not resize UX acceptance.
use super::Renderer;
use crate::{CameraState, PreparedScene, RetainedScene};
use std::time::{Duration, Instant};

fn cpu_ticks() -> u64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap();
    let fields: Vec<_> = stat
        .rsplit_once(')')
        .unwrap()
        .1
        .split_whitespace()
        .collect();
    fields[11].parse::<u64>().unwrap() + fields[12].parse::<u64>().unwrap()
}

fn wait(device: &wgpu::Device) {
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(10)),
        })
        .unwrap();
}

fn pixels(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
    let extent = texture.size();
    let row = (extent.width * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("attachment-probe-readback"),
        size: u64::from(row) * u64::from(extent.height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(extent.height),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
    wait(device);
    rx.recv_timeout(Duration::from_secs(10)).unwrap().unwrap();
    let mapped = buffer.slice(..).get_mapped_range();
    let result = mapped
        .chunks_exact(row as usize)
        .flat_map(|row| row[..extent.width as usize * 4].iter().copied())
        .collect();
    drop(mapped);
    buffer.unmap();
    result
}

#[test]
#[ignore = "manual optimized GPU cost isolation; not a resource or temporal acceptance test"]
fn fixed_extent_msaa_reuse_vs_replacement() {
    run_probe(false);
}

#[test]
#[ignore = "manual changing-extent allocation isolation; not native resize acceptance"]
fn alternating_extent_msaa_reuse_vs_replacement() {
    run_probe(true);
}

fn run_probe(alternate: bool) {
    let hz: u64 = String::from_utf8(
        std::process::Command::new("getconf")
            .arg("CLK_TCK")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .parse()
    .unwrap();
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let supported = adapter
        .get_texture_format_features(format)
        .flags
        .supported_sample_counts();
    let samples = [8, 4, 1]
        .into_iter()
        .find(|n| supported.contains(n))
        .unwrap();
    assert!(samples > 1, "diagnostic requires supported MSAA");
    println!(
        "adapter={:?} sample_count={samples} extent=1280x800 hz={hz}",
        adapter.get_info()
    );
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: adapter.features()
            & wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
        memory_hints: wgpu::MemoryHints::Performance,
        ..Default::default()
    }))
    .unwrap();
    let mut renderer = Renderer::new(&device, &queue, format, samples);
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let extents = if alternate {
        vec![(1280, 800), (1288, 808)]
    } else {
        vec![(1280, 800)]
    };
    println!("diagnostic_extents={extents:?}");
    let mut cases = Vec::new();
    for (width, height) in extents {
        let retained = RetainedScene::from_workspace(&state, width, height);
        let prepared = PreparedScene::from_workspace(
            &state,
            width,
            height,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("attachment-probe-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        for _ in 0..30 {
            renderer
                .render(
                    &device, &queue, &view, &prepared, &retained, None, width, height,
                )
                .unwrap();
            wait(&device);
        }
        let reference = pixels(&device, &queue, &target);
        cases.push((width, height, target, view, retained, prepared, reference));
    }
    let period = Duration::from_nanos(1_000_000_000 / 60);
    for (phase, replace) in [false, true, true, false].into_iter().enumerate() {
        // At most two exact-size attachments, only in the test reuse condition.
        // This is not a production cache or a proposed unbounded extent pool.
        let cache: Vec<_> = if alternate && !replace {
            cases
                .iter()
                .map(|(w, h, ..)| renderer.ensure_msaa(&device, *w, *h).clone())
                .collect()
        } else {
            Vec::new()
        };
        wait(&device);
        let start_ticks = cpu_ticks();
        let start = Instant::now();
        let mut late = 0;
        for frame in 0..180 {
            let index = frame as usize % cases.len();
            let (width, height, _, view, retained, prepared, _) = &cases[index];
            if !cache.is_empty() {
                renderer.msaa_view = Some(cache[index].clone());
                renderer.msaa_size = (*width, *height);
            }
            if replace {
                // Keep old view alive until allocation, matching production resize.
                renderer.msaa_size = (0, 0);
            }
            renderer
                .render(
                    &device, &queue, view, prepared, retained, None, *width, *height,
                )
                .unwrap();
            wait(&device);
            let deadline = start + period * (frame + 1);
            if let Some(left) = deadline.checked_duration_since(Instant::now()) {
                std::thread::sleep(left);
            } else {
                late += 1;
            }
        }
        let elapsed = start.elapsed().as_secs_f64();
        let cpu_s = (cpu_ticks() - start_ticks) as f64 / hz as f64;
        println!(
            "phase={phase} replace={replace} frames=180 elapsed_s={elapsed:.6} cpu_s={cpu_s:.6} cpu_percent={:.3} late_deadlines={late}",
            cpu_s / elapsed * 100.0
        );
        for (width, height, target, _, _, _, reference) in &cases {
            assert_eq!(
                &pixels(&device, &queue, target),
                reference,
                "pixel difference in phase {phase}, extent {width}x{height}"
            );
        }
    }
}
