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
    owner.flush_uploads(&queue);
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
    owner.flush_uploads(&queue);
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
    owner.flush_uploads(&queue);
    assert_eq!(
        read(&device, &queue, &first, 16),
        bytemuck::cast_slice::<u32, u8>(&values)
    );
    values[0] = 18;
    values[3] = 19;
    assert_eq!(owner.sync(&device, &queue, "proof", &values).unwrap(), 8);
    owner.flush_uploads(&queue);
    assert_eq!(
        read(&device, &queue, &first, 16),
        bytemuck::cast_slice::<u32, u8>(&values)
    );
    // Multi-word vertices: changing one byte transfers only its aligned
    // edge words. Internal gaps stay coalesced to bound queue-call overhead.
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
        20
    );
    owner.flush_uploads(&queue);
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
    owner.flush_uploads(&queue);
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
    owner.flush_uploads(&queue);
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
