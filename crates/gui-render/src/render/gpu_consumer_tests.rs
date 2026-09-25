//! Producer incidence at actual renderer upload and allocation boundaries.
use super::*;
use crate::resource_consumers::{Consumer, Consumers};

#[test]
#[ignore = "requires local GPU; production consumer incidence"]
fn workspace_uploads_preserve_shared_stream_consumers_and_warm_zero_work() {
    let state = crate::gpu_surface_pass::board_fixture_state();
    let retained = RetainedScene::from_workspace(&state, 960, 720);
    let prepared = PreparedScene::from_workspace_for_surface(
        &state,
        960,
        720,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let mut host = hardware_renderer(960, 720);
    let cold = capture_retained(&mut host, &prepared, &retained);
    let panel_id = host
        .renderer
        .panel_gpu
        .submission_ref()
        .unwrap()
        .allocation_id;
    let records = Renderer::gpu_process_allocations();
    let panel = records.iter().find(|r| r.id == panel_id).unwrap();
    for consumer in [
        Consumer::Main,
        Consumer::Navigator,
        Consumer::Layers,
        Consumer::Inspector,
        Consumer::Terminal,
    ] {
        assert!(
            panel.consumers.contains(consumer),
            "missing panel producer {consumer:?}"
        );
    }
    assert_eq!(panel.last_upload.unwrap().consumers, panel.consumers);
    assert!(
        !panel.consumers.contains(Consumer::Menu),
        "closed menu adds labels but no panel geometry"
    );
    let text_id = host
        .renderer
        .text_renderer
        .submission_ref()
        .unwrap()
        .allocation_id;
    assert!(
        records
            .iter()
            .find(|r| r.id == text_id)
            .unwrap()
            .consumers
            .contains(Consumer::Menu)
    );
    let board_id = host
        .renderer
        .world_vertices_gpu
        .submission_ref()
        .unwrap()
        .allocation_id;
    let board = records.iter().find(|r| r.id == board_id).unwrap();
    assert_eq!(board.consumers, Consumer::Board.into());
    assert_eq!(board.last_upload.unwrap().consumers, Consumer::Board.into());
    assert_eq!(board.renderer_id, Some(host.renderer.resource_owner_id()));
    assert!(cold == capture_retained(&mut host, &prepared, &retained));
    assert_eq!(
        host.renderer.last_upload_frame().unwrap().totals,
        Default::default()
    );
    let warm = Renderer::gpu_process_allocations();
    assert_eq!(
        warm.iter().find(|r| r.id == panel_id).unwrap().consumers,
        panel.consumers
    );
}

#[test]
#[ignore = "requires local GPU; default native dialog renderer incidence"]
fn dialog_producers_reach_upload_receipts_without_workspace_consumer_leakage() {
    let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    state.ui.project_preferences.open = true;
    state.ui.new_project.open = true;
    let mut host = hardware_renderer(960, 720);
    for consumer in [Consumer::Global, Consumer::Project, Consumer::New] {
        let mut prepared = match consumer {
            Consumer::New => host
                .renderer
                .prepare_native_new_project(&state.ui.new_project, 960, 720, 1.0)
                .unwrap(),
            _ => PreparedScene::from_native_preferences(
                if consumer == Consumer::Project {
                    &state.ui.project_preferences
                } else {
                    &state.ui.global_preferences
                },
                960,
                720,
                1.0,
            )
            .unwrap(),
        };
        // Same selection used by the real shared native-window adapter.
        prepared.set_native_consumer(consumer);
        let _ = capture(&mut host, &prepared);
        let frame = host.renderer.last_upload_frame().unwrap();
        assert_eq!(frame.consumers, Consumers::from(consumer));
        let id = host
            .renderer
            .menu_overlay_gpu
            .submission_ref()
            .unwrap()
            .allocation_id;
        let records = Renderer::gpu_process_allocations();
        let menu = records.iter().find(|r| r.id == id).unwrap();
        assert_eq!(menu.consumers, consumer.into());
        assert_eq!(menu.last_upload.unwrap().consumers, consumer.into());
        assert!(!menu.consumers.contains(Consumer::Main));
        assert!(!menu.consumers.contains(Consumer::Board));
    }
}

#[test]
#[ignore = "requires local GPU and serial process-wide text admission"]
fn renderer_creation_refuses_text_pressure_and_recovers_after_release() {
    let host = hardware_renderer(64, 64);
    let filler = crate::text_buffer_cache::budget::Owner::new(0);
    let owners = Renderer::text_cache_process_usage();
    let existing: usize = owners.iter().map(|o| o.bytes + o.constructing_bytes).sum();
    filler.publish(32 * 1024 * 1024 - Renderer::text_cache_registry_bytes() - existing);
    assert!(
        Renderer::new(
            &host.device,
            &host.queue,
            OUTPUT_FORMAT,
            DEFAULT_MSAA_SAMPLES
        )
        .is_err()
    );
    assert_eq!(Renderer::text_cache_process_usage().len(), owners.len());
    drop(filler);
    let replacement = Renderer::new(
        &host.device,
        &host.queue,
        OUTPUT_FORMAT,
        DEFAULT_MSAA_SAMPLES,
    )
    .unwrap();
    assert_eq!(Renderer::text_cache_process_usage().len(), owners.len());
    drop(replacement);
    assert_eq!(Renderer::text_cache_process_usage().len(), owners.len() - 1);
}
