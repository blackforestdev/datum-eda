use super::*;

#[test]
#[ignore = "requires local GPU and DATUM_RESOURCE_TRACE=1; serial delivery and overflow"]
fn frame_trace_serializes_actual_render_attempts_and_refuses_overflow() {
    assert!(
        std::env::var_os("DATUM_RESOURCE_TRACE").is_some(),
        "run with DATUM_RESOURCE_TRACE=1 to enable text/source observation"
    );
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
    let fixture_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad");
    let board_state = datum_gui_protocol::load_board_editor_workspace_state(
        &datum_gui_protocol::LiveReviewRequest {
            board_file: Some(fixture_root.join("simple-demo.kicad_pcb")),
            project_root: fixture_root,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
    )
    .unwrap();
    let board = datum_gui_render::RetainedScene::from_workspace(&board_state, 960, 720);
    let board_prepared = datum_gui_render::PreparedScene::from_workspace_for_surface(
        &board_state,
        960,
        720,
        1.0,
        datum_gui_render::CameraState::fit_to_bounds(&board_state.scene.bounds),
        &board,
    )
    .unwrap();
    renderer
        .render(
            &device,
            &queue,
            &view,
            &board_prepared,
            &board,
            None,
            960,
            720,
        )
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
        if row["world_panes"].as_array().unwrap().is_empty() || row["submitted_frame"] != true {
            assert!(row["submitted_world_bundles"].is_null());
        }
    }
    for row in &frames {
        let screen = row["prepared_screen_geometry"].as_array().unwrap();
        assert_eq!(screen.len(), 8);
        for group in screen {
            assert_eq!(
                group["payload_bytes"].as_u64().unwrap(),
                group["prepared_vertices"].as_u64().unwrap() * 20
            );
        }
        if row["submitted_frame"] == true {
            assert!(
                row["submitted_terminal_geometry"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        } else {
            assert!(row["submitted_terminal_geometry"].is_null());
        }
    }
    let world_frame = frames
        .iter()
        .find(|r| r["submitted_world_bundles"].is_array())
        .unwrap();
    let text = &world_frame["text_admission"];
    let origins = world_frame["text_origins"].as_array().unwrap();
    assert!(!origins.is_empty());
    assert!(origins.iter().any(|o| o["origin"]["kind"] == "viewport"));
    for (overlay, group) in [(false, "workspace"), (true, "overlay")] {
        let runs: u64 = origins
            .iter()
            .filter(|o| o["overlay"] == overlay)
            .map(|o| o["runs"].as_u64().unwrap())
            .sum();
        let glyphs: u64 = origins
            .iter()
            .filter(|o| o["overlay"] == overlay)
            .map(|o| o["shaped_instances"].as_u64().unwrap())
            .sum();
        assert_eq!(text[group]["runs"], runs);
        assert_eq!(text[group]["shaped_instances"], glyphs);
    }
    let grids = world_frame["prepared_surface_grids"].as_array().unwrap();
    assert!(!grids.is_empty());
    for grid in grids {
        let range = grid["range"].as_array().unwrap();
        let start = range[0].as_u64().unwrap();
        let end = range[1].as_u64().unwrap();
        assert!(start < end && end <= grid["generated_vertices"].as_u64().unwrap());
        assert_eq!(grid["prepared_vertices"], end - start);
        assert!(
            world_frame["world_panes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["pane_id"] == grid["pane_id"])
        );
    }
    assert!(
        renderer.grid_geometry_admission().is_none(),
        "observer metadata retires after callback"
    );
    let panes = world_frame["submitted_world_bundles"].as_array().unwrap();
    assert!(!panes.is_empty());
    for pane in panes {
        let prepared_pane = world_frame["world_panes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["pane_id"] == pane["pane_id"])
            .unwrap();
        assert_eq!(
            pane["submitted_commands"],
            pane["ranges"].as_array().unwrap().len()
        );
        if prepared_pane["counts"].is_null() {
            assert_eq!(pane["surface"], "Schematic");
            assert_eq!(
                prepared_pane["error"],
                "missing retained scene for prepared pane"
            );
            assert_eq!(pane["submitted_commands"], 0);
            assert!(pane["ranges"].as_array().unwrap().is_empty());
            continue;
        }
        assert_eq!(
            pane["submitted_vertices"],
            prepared_pane["counts"]["prepared_vertices"]
        );
        assert_eq!(
            pane["submitted_stroke_instances"],
            prepared_pane["counts"]["prepared_stroke_instances"]
        );
        assert_eq!(
            pane["submitted_triangles"],
            prepared_pane["counts"]["prepared_triangles"]
        );
        assert!(pane["submitted_triangles"].as_u64().unwrap() > 0);
    }
    let end = rows.last().unwrap();
    assert_eq!(end["phase"], "end");
    assert_eq!(end["frames"], frames.len());
    assert_eq!(end["frame_delivery_failed"], true);
    assert_eq!(end["complete_delivery"], false);
    eprintln!("preserved frame writer trace: {}", path.display());
}
