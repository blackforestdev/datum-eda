use super::super::{GpuMeasurements, SLOTS};
use super::*;

#[test]
#[ignore = "requires local timestamp GPU; shared measurement resource admission and retirement"]
fn query_resources_rollback_admission_and_survive_frame_and_submission_owners() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TIMESTAMP_QUERY,
        ..Default::default()
    }))
    .unwrap();
    let budget = crate::text_gpu::budget::gpu_process();
    let baseline = budget.used();
    let records = || {
        crate::Renderer::gpu_process_allocations()
            .into_iter()
            .filter(|r| matches!(r.kind, Kind::Query | Kind::QueryResolve | Kind::Readback))
            .collect::<Vec<_>>()
    };
    assert!(records().is_empty());
    // Admit one complete slot, then fail the next slot before API allocation.
    let filler = budget
        .reserve(512 * 1024 * 1024 - baseline - BYTES * 5)
        .unwrap();
    assert!(GpuMeasurements::new(&device, &queue, 1, 1, Box::new(|_| {})).is_err());
    assert_eq!(budget.used(), 512 * 1024 * 1024 - BYTES * 5);
    assert!(
        records().is_empty(),
        "failed ring construction must retire every partial slot"
    );
    drop(filler);
    let mut measurements = GpuMeasurements::new(&device, &queue, 1, 2, Box::new(|_| {})).unwrap();
    let live = records();
    assert_eq!(live.len(), SLOTS * 3);
    assert!(
        live.iter()
            .all(|r| r.bytes == BYTES && r.generation == 2 && !r.retiring)
    );
    assert_eq!(budget.used(), baseline + SLOTS as u64 * BYTES * 3);
    let frame = measurements.begin().unwrap();
    drop(measurements);
    assert_eq!(
        budget.used(),
        baseline + BYTES * 3,
        "encoding frame holds its slot through owner close"
    );
    assert!(records().iter().all(|r| r.retiring));
    drop(frame);
    assert_eq!(budget.used(), baseline);
    assert!(records().is_empty());

    let mut measurements = GpuMeasurements::new(&device, &queue, 1, 3, Box::new(|_| {})).unwrap();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("query-resource-proof"),
        size: wgpu::Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let mut frame = measurements.begin().unwrap();
    // Deterministically keep this submitted slot alive after asynchronous map
    // cancellation, even if the tiny real GPU submission has already completed.
    let held = measurements.slots[frame.slot].submission_refs();
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: Some(frame.pass("resource-proof").unwrap()),
            ..Default::default()
        });
    }
    measurements.resolve(&mut frame, &mut encoder).unwrap();
    queue.submit([encoder.finish()]);
    measurements.submitted(&queue, frame).unwrap();
    drop(measurements);
    assert_eq!(budget.used(), baseline + BYTES * 3);
    assert_eq!(records().len(), 3);
    assert!(records().iter().all(|r| r.retiring));
    drop(held);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(budget.used(), baseline);
    assert!(records().is_empty());
}
