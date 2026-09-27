use super::*;

#[test]
#[ignore = "requires local GPU; serial resource writer delivery and overflow"]
fn frame_trace_serializes_actual_render_attempts_and_refuses_overflow() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ENABLED.store(false, Ordering::Release);
            WRITER.lock().unwrap_or_else(|e| e.into_inner()).take();
        }
    }
    let _reset = Reset;
    let path =
        std::env::temp_dir().join(format!("pm045-frame-writer-{}.jsonl", uuid::Uuid::new_v4()));
    let now = Instant::now();
    *WRITER.lock().unwrap() = Some(Writer {
        file: File::options()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap(),
        hosts: Vec::new(),
        started: now,
        last: now,
        interval: Duration::from_secs(1),
        sequence: 0,
        limit: 100,
        frames: 0,
        frame_limit: 100,
        frame_failed: false,
        frame_scope: cpu_alloc::Scope::new("frame-admission-writer-test"),
    });
    resource_observation::register_frame_observer(frames::record).unwrap();
    resource_observation::register_frame_observer(frames::record).unwrap();
    ENABLED.store(true, Ordering::Release);
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    assert_ne!(adapter.get_info().device_type, wgpu::DeviceType::Cpu);
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = Renderer::new(&device, &queue, format, 4).unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.global_preferences.open = true;
    let prepared = datum_gui_render::PreparedScene::from_native_preferences(
        &state.ui.global_preferences,
        960,
        720,
        1.0,
    )
    .unwrap();
    let retained = datum_gui_render::RetainedScene::empty();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("frame-writer-test"),
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
    let view = target.create_view(&Default::default());
    renderer
        .render(&device, &queue, &view, &prepared, &retained, None, 960, 720)
        .unwrap();
    {
        let mut lock = WRITER.lock().unwrap();
        let writer = lock.as_mut().unwrap();
        assert!(writer.frames > 0);
        writer.frame_limit = writer.frames;
    }
    let error = renderer
        .render(&device, &queue, &view, &prepared, &retained, None, 960, 720)
        .unwrap_err();
    assert!(error.to_string().contains("frame capacity exhausted"));
    assert!(WRITER.lock().unwrap().as_ref().unwrap().frame_failed);
    assert!(
        finish(true).is_err(),
        "truncated delivery cannot finish successfully"
    );
    let rows: Vec<Value> = std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let frames: Vec<_> = rows.iter().filter(|r| r["phase"] == "frame").collect();
    assert!(!frames.is_empty());
    assert!(frames.iter().any(|r| r["submitted_frame"] == true));
    for (index, row) in frames.iter().enumerate() {
        assert_eq!(row["sequence"], index as u64 + 1);
        assert_eq!(row["renderer_id"], renderer.resource_owner_id());
        assert_eq!(row["extent"], json!([960, 720]));
        assert!(row["world_panes"].as_array().unwrap().is_empty());
    }
    let end = rows.last().unwrap();
    assert_eq!(end["phase"], "end");
    assert_eq!(end["frames"], frames.len());
    assert_eq!(end["frame_delivery_failed"], true);
    assert_eq!(end["complete_delivery"], false);
    eprintln!("preserved frame writer trace: {}", path.display());
}
