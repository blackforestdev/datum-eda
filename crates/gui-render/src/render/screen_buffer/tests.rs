use super::*;

fn read(device: &wgpu::Device, queue: &wgpu::Queue, source: &wgpu::Buffer, len: u64) -> Vec<u8> {
    let target = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("screen-upload-readback"),
        size: len,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(source, 0, &target, 0, len);
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    target
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let bytes = target.slice(..).get_mapped_range().to_vec();
    target.unmap();
    bytes
}

#[test]
#[ignore = "requires local GPU; run serially with visual feature"]
fn cancelled_uploads_rebuild_and_retired_vertices_remain_observable() {
    use crate::text_gpu::lifetime::Kind;
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut owner = ScreenBuffer::default();
    let values = [17_u32; 4];
    owner.sync(&device, &queue, "cancelled", &values).unwrap();
    let buffer = owner.buffer().unwrap().clone();
    assert_eq!(
        read(&device, &queue, &buffer, 16),
        vec![0; 16],
        "preparation must not queue even the initial upload"
    );
    owner.cancel_uploads();
    assert_eq!(owner.sync(&device, &queue, "retry", &values).unwrap(), 16);
    owner.flush_uploads(&device, &queue);
    let held = owner.submission_ref().unwrap();
    queue.submit([]);
    // A bundle/submission hold pins exactly the allocation, after its stream retires.
    let record = crate::Renderer::gpu_process_allocations()
        .into_iter()
        .find(|r| r.kind == Kind::Vertex && !r.retiring)
        .unwrap();
    let large = [23_u32; 64];
    owner.sync(&device, &queue, "replacement", &large).unwrap();
    assert!(
        crate::Renderer::gpu_process_allocations()
            .iter()
            .any(|r| r.id == record.id && r.retiring && r.bytes == 16)
    );
    assert!(
        !crate::Renderer::text_gpu_process_allocations()
            .iter()
            .any(|r| r.id == record.id)
    );
    assert_eq!(
        read(&device, &queue, &buffer, 16),
        bytemuck::cast_slice::<u32, u8>(&values)
    );
    drop(buffer);
    drop(held);
    assert!(
        !crate::Renderer::gpu_process_allocations()
            .iter()
            .any(|r| r.id == record.id)
    );
    owner.cancel_uploads();
    assert_eq!(
        owner
            .sync(&device, &queue, "replacement-retry", &large)
            .unwrap(),
        256
    );
    owner.flush_uploads(&device, &queue);
    assert_eq!(
        read(&device, &queue, owner.buffer().unwrap(), 256),
        bytemuck::cast_slice::<u32, u8>(&large)
    );
}

#[test]
#[ignore = "requires local GPU; run serially with visual feature"]
fn screen_upload_reuses_exact_content_and_bounds_retention() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut owner = ScreenBuffer::default();
    let mut values = [1_u32, 2, 3, 4];
    assert_eq!(owner.sync(&device, &queue, "proof", &values).unwrap(), 16);
    let first = owner.buffer().unwrap().clone();
    let relocated = values.to_vec();
    assert_ne!(relocated.as_ptr(), values.as_ptr());
    assert_eq!(owner.sync(&device, &queue, "proof", &relocated).unwrap(), 0);
    values[1] = 17; // same address AND length, different content
    assert_eq!(owner.sync(&device, &queue, "proof", &values).unwrap(), 4);
    assert_eq!(owner.buffer(), Some(&first));
    owner.flush_uploads(&device, &queue);
    assert_eq!(
        read(&device, &queue, &first, 16),
        bytemuck::cast_slice::<u32, u8>(&values)
    );
    values[0] = 18;
    values[3] = 19;
    assert_eq!(owner.sync(&device, &queue, "proof", &values).unwrap(), 8);
    owner.flush_uploads(&device, &queue);
    assert_eq!(
        read(&device, &queue, &first, 16),
        bytemuck::cast_slice::<u32, u8>(&values)
    );
    // Multi-word vertices: changing one byte transfers only its aligned
    // words. Internal clean words never enter the staging payload.
    let mut vertices = [[0_u32; 5]; 3];
    owner
        .sync(&device, &queue, "vertex-fields", &vertices)
        .unwrap();
    vertices[0][1] = 0x0100;
    vertices[0][4] = 0x0200;
    vertices[2][4] = 0x0300;
    assert_eq!(
        owner
            .sync(&device, &queue, "vertex-fields", &vertices)
            .unwrap(),
        12
    );
    owner.flush_uploads(&device, &queue);
    assert_eq!(
        read(&device, &queue, owner.buffer().unwrap(), 60),
        bytemuck::cast_slice::<[u32; 5], u8>(&vertices)
    );
    assert_eq!(
        owner
            .sync(&device, &queue, "vertex-fields", &vertices)
            .unwrap(),
        0
    );
    // A retained allocation can grow its live prefix without reallocating.
    owner
        .sync(&device, &queue, "short-prefix", &vertices[..2])
        .unwrap();
    vertices[0][0] = 7;
    assert_eq!(
        owner
            .sync(&device, &queue, "grow-prefix", &vertices)
            .unwrap(),
        24
    );
    owner.flush_uploads(&device, &queue);
    assert_eq!(
        read(&device, &queue, owner.buffer().unwrap(), 60),
        bytemuck::cast_slice::<[u32; 5], u8>(&vertices)
    );
    let at_cap = vec![42_u32; MAX_SNAPSHOT_BYTES / 4];
    assert_eq!(
        owner.sync(&device, &queue, "proof", &at_cap).unwrap(),
        MAX_SNAPSHOT_BYTES
    );
    assert_eq!(owner.snapshot.len(), MAX_SNAPSHOT_BYTES);
    assert_eq!(owner.sync(&device, &queue, "proof", &at_cap).unwrap(), 0);
    let oversized = vec![43_u32; MAX_SNAPSHOT_BYTES / 4 + 1];
    for _ in 0..2 {
        assert_eq!(
            owner.sync(&device, &queue, "proof", &oversized).unwrap(),
            MAX_SNAPSHOT_BYTES + 4
        );
        assert!(
            owner.snapshot.is_empty(),
            "overlarge content bypasses retention"
        );
    }
    owner.flush_uploads(&device, &queue);
    assert_eq!(
        read(
            &device,
            &queue,
            owner.buffer().unwrap(),
            oversized.len() as u64 * 4
        ),
        bytemuck::cast_slice::<u32, u8>(&oversized)
    );
    assert_eq!(owner.sync(&device, &queue, "proof", &values).unwrap(), 16);
    assert_eq!(
        owner.buffer().unwrap().size(),
        16,
        "obsolete peak capacity released"
    );
    assert_ne!(owner.buffer(), Some(&first));
    // Clearing releases both owners; identical content must upload again.
    assert_eq!(owner.sync::<u32>(&device, &queue, "proof", &[]).unwrap(), 0);
    assert!(owner.buffer().is_none());
    assert!(owner.snapshot.is_empty());
    assert_eq!(owner.sync(&device, &queue, "proof", &values).unwrap(), 16);
    owner = ScreenBuffer::default(); // renderer/device replacement
    assert_eq!(owner.sync(&device, &queue, "proof", &values).unwrap(), 16);
}

#[test]
#[ignore = "requires local GPU; screen subcap preservation across empty streams"]
fn empty_stream_preserves_its_admission_budget() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let budget = crate::text_gpu::budget::Budget::new(16);
    let mut stream = ScreenBuffer::with_budget(budget.clone());
    stream.sync(&device, &queue, "subcap", &[1_u32; 4]).unwrap();
    let held = stream.submission_ref().unwrap();
    stream.sync::<u32>(&device, &queue, "empty", &[]).unwrap();
    assert_eq!(budget.used(), 16);
    assert!(stream.sync(&device, &queue, "held", &[1_u32; 4]).is_err());
    assert!(stream.buffer().is_none());
    drop(held);
    assert!(
        stream
            .sync(&device, &queue, "oversize", &[1_u32; 5])
            .is_err()
    );
    stream.sync(&device, &queue, "retry", &[1_u32; 4]).unwrap();
    assert_eq!(budget.used(), 16);
    drop(stream);
    assert_eq!(budget.used(), 0);
}

#[test]
#[ignore = "requires local GPU; pending upload supersession and cancellation"]
fn repeated_preparation_queues_each_final_range_once() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut stream = ScreenBuffer::default();
    let mut values = [0_u32; 16];
    stream.sync(&device, &queue, "initial", &values).unwrap();
    for i in 1..=1000 {
        values[1] = i;
        stream.sync(&device, &queue, "superseded", &values).unwrap();
        assert_eq!(stream.pending, vec![0..64]);
    }
    stream.flush_uploads(&device, &queue);
    assert_eq!(
        read(&device, &queue, stream.buffer().unwrap(), 64),
        bytemuck::cast_slice::<u32, u8>(&values)
    );
    // Separated edits stay separated, even across repeated preparation.
    for i in 1001..=2000 {
        values[1] = i;
        values[14] = i;
        stream.sync(&device, &queue, "separate", &values).unwrap();
        assert_eq!(stream.pending, vec![4..8, 56..60]);
    }
    // Shrink within the existing capacity: an obsolete tail must not upload.
    stream
        .sync(&device, &queue, "shrink", &values[..8])
        .unwrap();
    assert_eq!(stream.pending, vec![4..8]);
    stream.flush_uploads(&device, &queue);
    assert_eq!(
        read(&device, &queue, stream.buffer().unwrap(), 32),
        bytemuck::cast_slice::<u32, u8>(&values[..8])
    );
    // Regrowth restores the discarded tail even though the API buffer survived.
    stream.sync(&device, &queue, "regrow", &values).unwrap();
    assert_eq!(stream.pending, vec![32..64]);
    stream.cancel_uploads();
    stream.sync(&device, &queue, "retry", &values).unwrap();
    assert_eq!(stream.pending, vec![0..64]);
    stream.flush_uploads(&device, &queue);
    assert_eq!(
        read(&device, &queue, stream.buffer().unwrap(), 64),
        bytemuck::cast_slice::<u32, u8>(&values)
    );
}

#[test]
fn changed_ranges_match_independent_word_oracle() {
    let old = [0_u32; 12];
    for mask in 0_u32..(1 << old.len()) {
        let new: Vec<u32> = (0..old.len())
            .map(|i| {
                if mask & (1 << i) != 0 {
                    i as u32 + 1
                } else {
                    0
                }
            })
            .collect();
        let mut reconstructed = bytemuck::cast_slice(&old).to_vec();
        let mut copied = [false; 12];
        let mut previous_end = None;
        let bytes = dirty_ranges(
            bytemuck::cast_slice(&old),
            bytemuck::cast_slice(&new),
            16,
            |offset, data| {
                let start = offset as usize;
                assert!(previous_end.is_none_or(|end| end < start));
                previous_end = Some(start + data.len());
                for i in start / 4..(start + data.len()) / 4 {
                    assert_ne!(old[i], new[i], "clean word transferred");
                    assert!(!copied[i], "word transferred twice");
                    copied[i] = true;
                }
                reconstructed[start..start + data.len()].copy_from_slice(data);
            },
        );
        assert_eq!(bytes, mask.count_ones() as usize * 4);
        assert_eq!(reconstructed, bytemuck::cast_slice::<u32, u8>(&new));
    }
}
