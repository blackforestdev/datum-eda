use super::*;
use crate::text_gpu::{budget::Budget, lifetime::Owner, upload::batch_with_scatter};

#[test]
#[ignore = "requires local GPU; sparse packet admission, cancellation and readback"]
fn sparse_packets_preserve_clean_words_and_retire_complete_capacity() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let target = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sparse-readback-proof"),
        size: 4096 * 20,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: true,
    });
    target.slice(..).get_mapped_range_mut().fill(0x35);
    target.unmap();
    let value = 0x1234_abcd_u32.to_le_bytes();
    let direct = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mixed-direct-proof"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let destinations = Owner::new();
    let target = destinations.track(
        target,
        4096 * 20,
        4,
        crate::text_gpu::lifetime::Kind::Vertex,
    );
    let direct = destinations.track(direct, 4, 5, crate::text_gpu::lifetime::Kind::Vertex);
    let mut uploads: Vec<_> = (0..4096)
        .map(|i| BufferUpload {
            target: Some(target.upload_target()),
            buffer: &target,
            offset: i * 20,
            bytes: &value,
        })
        .collect();
    let plan: Vec<_> = groups(&uploads).collect();
    assert_eq!(plan.len(), 1);
    assert!(plan[0].sparse);
    assert_eq!(plan[0].packet_bytes(), 4096 * 8);
    assert!(!groups(&uploads[..63]).next().unwrap().sparse);
    uploads.swap(1, 2);
    assert!(
        !groups(&uploads).next().unwrap().sparse,
        "out-of-order ranges retain ordered copies"
    );
    uploads.swap(1, 2);
    uploads.insert(
        0,
        BufferUpload {
            target: Some(direct.upload_target()),
            buffer: &direct,
            offset: 0,
            bytes: &value,
        },
    );
    let required = crate::text_gpu::upload::required_bytes(&[], &uploads);
    assert_eq!(required, 4096 * 16 + 4);
    let owner = Owner::new();
    let observer = owner.observer();
    let metadata = crate::text_gpu::staging_vec::StagingVec::<
        crate::text_gpu::lifetime::Tracked<wgpu::Buffer>,
    >::capacity_bytes(2)
    .unwrap()
        + crate::text_gpu::upload::destination_metadata_bytes(2).unwrap();
    let host = Budget::new(required + metadata);
    let scatter = Scatter::default();
    let create = || batch_with_scatter(&device, &owner, 1, &host, &[], &uploads, Some(&scatter));
    let pending = create().unwrap().unwrap();
    assert_eq!(host.used(), required + metadata);
    assert_eq!(
        owner.records().iter().map(|r| r.bytes).sum::<u64>(),
        required
    );
    assert!(create().is_err());
    assert_eq!(owner.records().len(), 2);
    drop(pending);
    assert_eq!(host.used(), 0);
    assert_eq!(observer.submitted_upload_totals(), Default::default());
    let mut pending = create().unwrap().unwrap();
    queue.submit([pending.command()]);
    assert!(
        destinations
            .records()
            .iter()
            .all(|r| r.submitted_source_bytes == 0)
    );
    pending.hold(&queue);
    let records = destinations.records();
    let sparse = records.iter().find(|r| r.id == target.id()).unwrap();
    assert_eq!(
        (
            sparse.submitted_source_bytes,
            sparse.submitted_transfer_bytes
        ),
        (4096 * 4, 4096 * 8)
    );
    let plain = records.iter().find(|r| r.id == direct.id()).unwrap();
    assert_eq!(
        (plain.submitted_source_bytes, plain.submitted_transfer_bytes),
        (4, 4)
    );
    assert_eq!(
        observer.submitted_upload_totals(),
        crate::UploadTotals {
            batches: 1,
            buffer_payload_bytes: 4096 * 4 + 4,
            scatter_index_bytes: 4096 * 4,
            staging_capacity_bytes: required,
            buffer_copy_bytes: 4096 * 8 + 4,
            buffer_copies: 2,
            scatter_dispatches: 1,
            ..Default::default()
        }
    );
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sparse-readback"),
        size: target.size() + 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(&target, 0, &readback, 0, target.size());
    encoder.copy_buffer_to_buffer(&direct, 0, &readback, target.size(), 4);
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let mapped = readback.slice(..).get_mapped_range();
    assert_eq!(&mapped[target.size() as usize..], &value);
    for vertex in mapped[..target.size() as usize].as_chunks::<20>().0 {
        assert_eq!(&vertex[..4], &value);
        assert_eq!(&vertex[4..], &[0x35; 16]);
    }
    drop(mapped);
    readback.unmap();
    assert_eq!(host.used(), 0);
    assert!(owner.records().is_empty());
    // Negative control: deliberately repeat the identical upload. Counters must
    // reflect real submitted work even though the output pixels stay unchanged.
    let first = observer.submitted_upload_totals();
    let mut repeated = create().unwrap().unwrap();
    queue.submit([repeated.command()]);
    repeated.hold(&queue);
    let mut doubled = first;
    doubled.add(first);
    assert_eq!(observer.submitted_upload_totals(), doubled);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    drop(owner);
    assert_eq!(observer.submitted_upload_totals(), doubled);
}

#[test]
#[ignore = "bounded diagnostic only; run optimized explicitly, not qualification"]
fn sparse_encoding_comparison() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let target = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sparse-encoding-diagnostic"),
        size: 4096 * 20,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let bytes = vec![0x35; 4096 * 20];
    let sparse: Vec<_> = (0..4096)
        .map(|i| BufferUpload {
            target: None,
            buffer: &target,
            offset: i * 20,
            bytes: &bytes[..4],
        })
        .collect();
    let span = [BufferUpload {
        target: None,
        buffer: &target,
        offset: 0,
        bytes: &bytes[..bytes.len() - 16],
    }];
    let owner = Owner::new();
    let budget = Budget::new(16 * 1024 * 1024);
    let scatter = Scatter::default();
    let mut samples = [Vec::new(), Vec::new()];
    // Alternate order; exclude first lazy pipeline compilation and GPU waits.
    for iteration in 0..14 {
        for n in 0..2 {
            let mode = (iteration + n) % 2;
            let uploads = if mode == 0 { &span[..] } else { &sparse[..] };
            let start = std::time::Instant::now();
            let mut batch =
                batch_with_scatter(&device, &owner, 1, &budget, &[], uploads, Some(&scatter))
                    .unwrap()
                    .unwrap();
            let elapsed = start.elapsed().as_nanos();
            queue.submit([batch.command()]);
            batch.hold(&queue);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            if iteration >= 2 {
                samples[mode].push(elapsed);
            }
        }
    }
    for (name, mut values) in ["coalesced-span", "exact-scatter"].into_iter().zip(samples) {
        values.sort_unstable();
        eprintln!(
            "{name}: encode_median_ns={} samples={values:?}",
            values[values.len() / 2]
        );
    }
}
