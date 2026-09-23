use super::*;
use crate::gpu_data::shared_geometry::SharedGeometry;
use std::sync::{Arc, Mutex};

#[test]
#[ignore = "requires local GPU; bounded world continuation and source replacement"]
fn cold_world_yields_without_presenting_partial_data_and_restarts_changed_sources() {
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
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = Renderer::new(&device, &queue, format, 4).unwrap();
    let incomplete = Arc::new(Mutex::new(Vec::new()));
    let records = incomplete.clone();
    renderer
        .enable_gpu_measurements(
            &device,
            &queue,
            1,
            1,
            Box::new(move |record| records.lock().unwrap().push(record)),
        )
        .unwrap();
    let state = crate::gpu_surface_pass::board_fixture_state();
    let mut retained = RetainedScene::from_workspace(&state, 960, 720);
    let mut vertices = retained.world_vertices.to_vec();
    vertices.resize(220_000, vertices[0]);
    retained.world_vertices = SharedGeometry::for_document(vertices.clone(), "cold-world-proof");
    let prepared = PreparedScene::from_workspace_for_surface(
        &state,
        960,
        720,
        1.0,
        crate::CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("cold-world-proof-target"),
        size: wgpu::Extent3d {
            width: 960,
            height: 720,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let mut submissions = 0;
    for attempt in 0..3 {
        let ready = renderer
            .render_with_submission(
                &device,
                &queue,
                &view,
                &prepared,
                &retained,
                None,
                960,
                720,
                &mut |_| submissions += 1,
            )
            .unwrap();
        assert_eq!(ready, attempt == 2, "only the final attempt can present");
        if !ready {
            assert!(renderer.upload_staging_reserved_bytes() <= CHUNK_BYTES as u64);
        }
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        assert_eq!(renderer.upload_staging_reserved_bytes(), 0);
    }
    assert_eq!(submissions, 3);
    assert_eq!(
        renderer.world_upload_chunk_bytes(),
        (std::mem::size_of_val(vertices.as_slice())
            + std::mem::size_of_val(retained.world_strokes().as_ref())) as u64
    );
    assert_eq!(renderer.world_upload_chunk_count(), 2);
    assert!(
        renderer.poll_gpu_measurements(&device).unwrap().is_empty(),
        "partial cold timings must not become numeric frame samples"
    );
    {
        let records = incomplete.lock().unwrap();
        assert_eq!(records.len(), 3);
        assert!(records.iter().all(|r| r.frame == records[0].frame
            && r.reason == "cold_upload_multisubmission_timestamps_unqualified"));
        assert!(
            records
                .windows(2)
                .all(|p| p[0].submission < p[1].submission)
        );
    }
    assert!(
        renderer
            .render_with_submission(
                &device,
                &queue,
                &view,
                &prepared,
                &retained,
                None,
                960,
                720,
                &mut |_| submissions += 1
            )
            .unwrap()
    );
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(
        renderer.world_upload_chunk_count(),
        2,
        "warm content adds no upload turn"
    );
    assert_eq!(renderer.poll_gpu_measurements(&device).unwrap().len(), 1);

    // Replace the source again after only the first chunk of a new revision.
    vertices.last_mut().unwrap().pos[0] += 1.0;
    retained.world_vertices = SharedGeometry::for_document(vertices.clone(), "cold-world-proof");
    assert!(
        !renderer
            .render_with_submission(
                &device,
                &queue,
                &view,
                &prepared,
                &retained,
                None,
                960,
                720,
                &mut |_| {}
            )
            .unwrap()
    );
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert!(
        !renderer
            .world_vertices_gpu
            .matches_source(&SharedGeometry::from(Vec::<crate::Vertex>::new()))
    );
    vertices[0].pos[0] += 2.0;
    retained.world_vertices = SharedGeometry::for_document(vertices.clone(), "cold-world-proof");
    for attempt in 0..3 {
        assert_eq!(
            renderer
                .render_with_submission(
                    &device,
                    &queue,
                    &view,
                    &prepared,
                    &retained,
                    None,
                    960,
                    720,
                    &mut |_| {}
                )
                .unwrap(),
            attempt == 2
        );
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
    assert_eq!(renderer.world_upload_chunk_count(), 5);
    let expected: &[u8] = bytemuck::cast_slice(&vertices);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: expected.len() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(
        renderer.world_vertices_gpu.buffer().unwrap(),
        0,
        &readback,
        0,
        expected.len() as u64,
    );
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    assert_eq!(&*readback.slice(..).get_mapped_range(), expected);
}
