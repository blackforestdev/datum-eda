use super::*;

#[test]
#[ignore = "requires local GPU; explicit staging admission and completion"]
fn mixed_staging_is_charged_until_completion_and_preserves_texture_and_buffer_gaps() {
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
    let observer = owner.observer();
    let metadata = StagingVec::<Tracked<wgpu::Buffer>>::capacity_bytes(1).unwrap()
        + destination_metadata_bytes(2).unwrap();
    let host = Budget::new(520 + metadata);
    let target = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mixed-buffer-proof"),
        size: 16,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let destinations = Owner::new();
    let texture = destinations.track(texture, 6, 7, Kind::Texture);
    let target = destinations.track(target, 16, 9, Kind::Vertex);
    let buffers = [
        BufferUpload {
            target: Some(target.upload_target()),
            buffer: &target,
            offset: 4,
            bytes: &[9, 0, 0, 0],
        },
        BufferUpload {
            target: Some(target.upload_target()),
            buffer: &target,
            offset: 12,
            bytes: &[10, 0, 0, 0],
        },
    ];
    let upload = [TextureUpload {
        target: Some(texture.upload_target()),
        texture: &texture,
        origin: [0, 0],
        size: [3, 2],
        stride: 3,
        pixels: &[1, 2, 3, 4, 5, 6],
    }];
    let mut pending = batch(&device, &owner, 1, &host, &upload, &buffers)
        .unwrap()
        .unwrap();
    assert_eq!(
        host.used(),
        520 + metadata,
        "count padded texture rows plus exact buffer range bytes"
    );
    assert!(batch(&device, &owner, 2, &host, &upload, &buffers).is_err());
    assert_eq!(
        owner.records().len(),
        1,
        "refuse before another API allocation"
    );
    drop(pending);
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
        assert!(batch(&device, &owner, 2, &host, &upload, &buffers).is_err());
        assert_eq!(host.used(), 0);
        assert!(owner.records().is_empty());
        drop(filler);
    }
    assert_eq!(observer.submitted_upload_totals(), Default::default());
    assert!(
        destinations
            .records()
            .iter()
            .all(|r| r.submitted_source_bytes == 0 && r.submitted_transfer_bytes == 0)
    );
    owner.begin_upload_frame();
    pending = batch(&device, &owner, 3, &host, &upload, &buffers)
        .unwrap()
        .unwrap();
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 528,
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
    encoder.copy_buffer_to_buffer(&target, 0, &readback, 512, 16);
    queue.submit([pending.command(), encoder.finish()]);
    // The batch remains charged even when its encoded command has moved to the queue.
    assert_eq!(host.used(), 520 + metadata);
    pending.hold(&queue);
    let expected = crate::UploadTotals {
        batches: 1,
        buffer_payload_bytes: 8,
        texture_source_bytes: 6,
        texture_padding_bytes: 506,
        staging_capacity_bytes: 520,
        buffer_copy_bytes: 8,
        buffer_copies: 2,
        texture_copies: 1,
        ..Default::default()
    };
    assert_eq!(observer.submitted_upload_totals(), expected);
    let records = destinations.records();
    let texture_record = records.iter().find(|r| r.id == texture.id()).unwrap();
    assert_eq!(
        (
            texture_record.generation,
            texture_record.submitted_source_bytes,
            texture_record.submitted_transfer_bytes
        ),
        (7, 6, 512)
    );
    let buffer_record = records.iter().find(|r| r.id == target.id()).unwrap();
    assert_eq!(
        (
            buffer_record.generation,
            buffer_record.submitted_source_bytes,
            buffer_record.submitted_transfer_bytes
        ),
        (9, 8, 8)
    );
    let frame = owner.last_upload_frame().unwrap();
    let destination_frame = texture_record.last_upload.unwrap();
    assert_eq!(
        (destination_frame.owner, destination_frame.attempt),
        (frame.owner, frame.attempt)
    );
    assert_eq!(
        (
            destination_frame.source_bytes,
            destination_frame.transfer_bytes
        ),
        (6, 512)
    );
    owner.finish_upload_frame(Some(true));
    owner.begin_upload_frame();
    assert_ne!(
        owner.last_upload_frame().unwrap().attempt,
        destination_frame.attempt
    );
    assert_eq!(
        destinations
            .records()
            .iter()
            .find(|r| r.id == texture.id())
            .unwrap()
            .last_upload,
        Some(destination_frame)
    );
    owner.finish_upload_frame(Some(true));
    let (tx, rx) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let bytes = readback.slice(..).get_mapped_range();
    assert_eq!(&bytes[..3], &[1, 2, 3]);
    assert_eq!(&bytes[256..259], &[4, 5, 6]);
    assert_eq!(
        &bytes[512..528],
        &[0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0]
    );
    assert_eq!(host.used(), 0);
    assert!(owner.records().is_empty());
}
