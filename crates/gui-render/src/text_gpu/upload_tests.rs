use super::*;

#[test]
#[ignore = "requires local GPU; explicit staging admission and completion"]
fn padded_texture_staging_is_charged_until_completion_and_preserves_pixels() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("staging-proof"),
        size: wgpu::Extent3d {
            width: 3,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let owner = Owner::new();
    let host = Budget::new(512);
    let upload = [TextureUpload {
        texture: &texture,
        origin: [0, 0],
        size: [3, 2],
        stride: 3,
        pixels: &[1, 2, 3, 4, 5, 6],
    }];
    let mut batch = textures(&device, &owner, 1, &host, &upload)
        .unwrap()
        .unwrap();
    assert_eq!(
        host.used(),
        512,
        "count padded API capacity, not six source bytes"
    );
    assert!(textures(&device, &owner, 2, &host, &upload).is_err());
    assert_eq!(
        owner.records().len(),
        1,
        "refuse before another API allocation"
    );
    drop(batch);
    assert_eq!(host.used(), 0, "unsubmitted cancellation releases staging");
    assert!(owner.records().is_empty());
    // Each later reservation failure rolls back earlier admission atomically.
    let staging = super::super::budget::staging_process();
    let gpu = super::super::budget::gpu_process();
    for budget in [&staging, &gpu] {
        let limit = if std::sync::Arc::ptr_eq(budget, &staging) {
            64
        } else {
            512
        } * 1024
            * 1024;
        let filler = budget.reserve(limit - budget.used()).unwrap();
        assert!(textures(&device, &owner, 2, &host, &upload).is_err());
        assert_eq!(host.used(), 0);
        assert!(owner.records().is_empty());
        drop(filler);
    }
    batch = textures(&device, &owner, 3, &host, &upload)
        .unwrap()
        .unwrap();
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 512,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(2),
            },
        },
        wgpu::Extent3d {
            width: 3,
            height: 2,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([batch.command(), encoder.finish()]);
    // The batch remains charged even when its encoded command has moved to the queue.
    assert_eq!(host.used(), 512);
    batch.hold(&queue);
    let (tx, rx) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let bytes = readback.slice(..).get_mapped_range();
    assert_eq!(&bytes[..3], &[1, 2, 3]);
    assert_eq!(&bytes[256..259], &[4, 5, 6]);
    assert_eq!(host.used(), 0);
    assert!(owner.records().is_empty());
}
