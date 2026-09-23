use super::*;

#[test]
fn measured_pass_sum_excludes_gap_and_preserves_empty_pass() {
    let (passes, sum, span) =
        decode(&["empty", "draw", "text"], &[10, 10, 20, 60, 80, 90], 2.5).unwrap();
    assert_eq!(passes, [("empty", 0.0), ("draw", 100.0), ("text", 25.0)]);
    assert_eq!(sum, 125.0);
    assert_eq!(span, 200.0);
}

#[test]
fn invalid_or_missing_measurements_are_never_zero_samples() {
    for period in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(decode(&["draw"], &[10, 20], period).is_err());
    }
    for ticks in [
        vec![],
        vec![10],
        vec![10, 20, 30],
        vec![20, 10],
        vec![u64::MAX, 0],
        vec![0, 2_000_000_000],
    ] {
        assert!(decode(&["draw"], &ticks, 1.0).is_err());
    }
    assert!(decode(&[], &[], 1.0).is_err());
}

#[test]
fn active_deadline_pauses_without_resetting_progress() {
    let start = Instant::now();
    let mut clock = ActiveClock {
        previous: start,
        elapsed: Duration::ZERO,
        drawable: true,
        occluded: false,
        suspended: false,
    };
    clock.advance(start + Duration::from_millis(500));
    clock.occluded = true;
    clock.advance(start + Duration::from_secs(10));
    assert_eq!(clock.elapsed, Duration::from_millis(500));
    clock.occluded = false;
    clock.advance(start + Duration::from_millis(10_500));
    assert_eq!(clock.elapsed, Duration::from_secs(1));
    clock.drawable = false;
    clock.advance(start + Duration::from_secs(20));
    clock.drawable = true;
    clock.suspended = true;
    clock.advance(start + Duration::from_secs(30));
    assert_eq!(clock.elapsed, Duration::from_secs(1));
}

#[cfg(feature = "visual")]
#[test]
#[ignore = "requires real timestamp-capable GPU; explicit conformance run"]
fn real_gpu_query_ring_completion_and_failure_controls() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    assert!(adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY));
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TIMESTAMP_QUERY,
        ..Default::default()
    }))
    .unwrap();
    let cancellations = Arc::new(std::sync::Mutex::new(Vec::new()));
    let observer = |receipts: &Arc<std::sync::Mutex<Vec<GpuMeasurementCancellation>>>| -> GpuCancellationObserver {
        let receipts = receipts.clone();
        Box::new(move |receipt| receipts.lock().unwrap().push(receipt))
    };
    let mut measurement =
        GpuMeasurements::new(&device, &queue, 1, 1, observer(&cancellations)).unwrap();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("query-conformance-target"),
        size: wgpu::Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("conformance-triangle"),
        source: wgpu::ShaderSource::Wgsl("@vertex fn vs(@builtin(vertex_index) i:u32)->@builtin(position) vec4f { var p = array<vec2f,3>(vec2f(-1.,-1.),vec2f(1.,-1.),vec2f(0.,1.)); return vec4f(p[i],0.,1.); } @fragment fn fs()->@location(0) vec4f { return vec4f(1.,0.,0.,1.); }".into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("conformance-triangle"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let mut frame = measurement.begin().unwrap();
    let reserved_submission = frame.submission;
    let mut encoder = device.create_command_encoder(&Default::default());
    for name in ["empty", "draw"] {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: if name == "empty" {
                        wgpu::LoadOp::Load
                    } else {
                        wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: Some(frame.pass(name).unwrap()),
            ..Default::default()
        });
        if name == "draw" {
            pass.set_pipeline(&pipeline);
            pass.draw(0..3, 0..1);
        }
    }
    measurement.resolve(&mut frame, &mut encoder).unwrap();
    queue.submit([encoder.finish()]);
    measurement.submitted(&queue, frame).unwrap();
    let limit = Instant::now() + Duration::from_secs(2);
    let samples = loop {
        let samples = measurement.poll(&device).unwrap();
        if !samples.is_empty() {
            break samples;
        }
        assert!(Instant::now() < limit);
        std::thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].passes_ns.len(), 2);
    assert_eq!(samples[0].raw_ticks.len(), 4);
    assert_eq!(
        (
            samples[0].host,
            samples[0].device_epoch,
            samples[0].submission
        ),
        (1, 1, reserved_submission)
    );
    eprintln!("adapter={:?} sample={:?}", adapter.get_info(), samples[0]);
    let mut next_epoch =
        GpuMeasurements::new(&device, &queue, 1, 2, observer(&cancellations)).unwrap();
    let mut missing = next_epoch.begin().unwrap();
    assert_ne!(missing.submission, reserved_submission);
    // Epoch mismatch is checked against a live, uncancelled owner.
    assert!(
        measurement
            .validate(&missing)
            .unwrap_err()
            .to_string()
            .contains("stale device epoch")
    );
    let mut encoder = device.create_command_encoder(&Default::default());
    assert!(
        next_epoch
            .resolve(&mut missing, &mut encoder)
            .unwrap_err()
            .to_string()
            .contains("missing")
    );
    drop(missing);
    drop(next_epoch);
    assert_eq!(cancellations.lock().unwrap().len(), 1);

    let submit_empty = |m: &mut GpuMeasurements| {
        let mut frame = m.begin().unwrap();
        let slot = frame.slot;
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
                timestamp_writes: Some(frame.pass("pending").unwrap()),
                ..Default::default()
            });
        }
        m.resolve(&mut frame, &mut encoder).unwrap();
        queue.submit([encoder.finish()]);
        m.submitted(&queue, frame).unwrap();
        slot
    };
    let mut delayed =
        GpuMeasurements::new(&device, &queue, 1, 3, observer(&cancellations)).unwrap();
    let slot = submit_empty(&mut delayed);
    assert_eq!(
        delayed.slots[slot]
            .pending
            .as_ref()
            .unwrap()
            .signal
            .load(Ordering::Acquire),
        MAPPING
    );
    delayed.clock.elapsed += DEADLINE;
    assert!(
        delayed
            .poll(&device)
            .unwrap_err()
            .to_string()
            .contains("two seconds")
    );
    drop(delayed);
    assert_eq!(cancellations.lock().unwrap().len(), 2);

    let mut map_failed =
        GpuMeasurements::new(&device, &queue, 1, 4, observer(&cancellations)).unwrap();
    let slot = submit_empty(&mut map_failed);
    // A real pending-map cancellation invokes the actual error callback.
    map_failed.slots[slot].readback.unmap();
    assert!(
        map_failed
            .poll(&device)
            .unwrap_err()
            .to_string()
            .contains("map failed")
    );
    drop(map_failed);
    assert_eq!(cancellations.lock().unwrap().len(), 3);

    let held: Vec<_> = (0..SLOTS).map(|_| measurement.begin().unwrap()).collect();
    assert!(
        measurement
            .begin()
            .err()
            .expect("exhausted ring")
            .to_string()
            .contains("ring exhausted")
    );
    drop(held);
    assert!(
        measurement
            .poll(&device)
            .unwrap_err()
            .to_string()
            .contains("aborted")
    );
    measurement.cancel();
    assert!(measurement.begin().is_err());
    assert_eq!(cancellations.lock().unwrap().len(), 6);
}
