//! Exact8x full/split output and copy-inclusive measurement conformance.
use super::*;
use crate::render_input::{PointerUpdate, SourceEpoch};

fn capture8() -> OffscreenRenderer {
    capture8_with_preference(wgpu::PowerPreference::HighPerformance)
}
pub(super) fn reference_capture8() -> OffscreenRenderer {
    capture8_with_preference(wgpu::PowerPreference::LowPower)
}
fn capture8_with_preference(preference: wgpu::PowerPreference) -> OffscreenRenderer {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: preference,
        ..Default::default()
    }))
    .unwrap();
    if preference == wgpu::PowerPreference::LowPower {
        assert!(
            adapter.get_info().name.contains("P630"),
            "reference proof requires pinned Intel P630"
        );
    }
    let format = adapter.get_texture_format_features(OUTPUT_FORMAT);
    assert!(
        format.flags.sample_count_supported(8),
        "exact8x test requires supported adapter"
    );
    let features = wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::TIMESTAMP_QUERY
        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES;
    assert!(adapter.features().contains(features));
    eprintln!(
        "exact8x adapter={:?}, format={format:?}",
        adapter.get_info()
    );
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: features,
        ..Default::default()
    }))
    .unwrap();
    let renderer = Renderer::new(&device, &queue, OUTPUT_FORMAT, 8).unwrap();
    OffscreenRenderer {
        device,
        queue,
        renderer,
        width: 640,
        height: 480,
    }
}

pub(super) fn prepare(
    c: &mut OffscreenRenderer,
    state: &ReviewWorkspaceState,
    source: &SourceEpoch,
) {
    c.renderer
        .prepare_session_workspace(
            state,
            WorkspaceView {
                source_revision: Some(source.revision()),
                width: c.width,
                height: c.height,
                scale: 1.0,
                camera: CameraState::fit_to_bounds(&state.scene.bounds),
                schematic_camera: None,
                include_preferences_overlay: false,
                single_terminal_snapshot: true,
                pane_cameras: &[],
            },
            &[],
        )
        .unwrap();
}

pub(super) fn target(c: &OffscreenRenderer) -> crate::capture_resource::CaptureTarget {
    crate::capture_resource::CaptureTarget::new(&c.device, c.extent(), OUTPUT_FORMAT).unwrap()
}

pub(super) fn frame(c: &mut OffscreenRenderer, presented: bool) -> RgbaImage {
    let target = target(c);
    let view = target.create_view(&Default::default());
    let plan = c
        .renderer
        .render_session_mut()
        .prepare_frame(1, 1, 1, true, c.width, c.height)
        .unwrap();
    let frame = c
        .renderer
        .encode_capture(plan, &c.device, &c.queue, &view)
        .unwrap();
    target.hold_submission(&c.queue);
    let image = c.read_texture(&target).unwrap();
    assert_eq!(
        c.renderer
            .render_session_mut()
            .complete_submitted(frame, 1, 1, 1, presented),
        presented
    );
    image
}

pub(super) fn full_reference(c: &mut OffscreenRenderer) -> RgbaImage {
    let prepared = c.renderer.render_session().prepared().unwrap().clone();
    let board = c.renderer.render_session().board().unwrap().clone();
    let schematic = c.renderer.render_session().schematic().cloned();
    let target = target(c);
    let view = target.create_view(&Default::default());
    c.renderer
        .render(
            &c.device,
            &c.queue,
            &view,
            &prepared,
            &board,
            schematic.as_ref(),
            c.width,
            c.height,
        )
        .unwrap();
    target.hold_submission(&c.queue);
    c.read_texture(&target).unwrap()
}

pub(super) fn world_uploads(c: &OffscreenRenderer) -> Vec<(u64, u64, u64)> {
    let ids: Vec<_> = [
        c.renderer.world_vertices_gpu.submission_ref(),
        c.renderer.world_strokes_gpu.submission_ref(),
        c.renderer.schematic_world_vertices_gpu.submission_ref(),
        c.renderer.schematic_world_strokes_gpu.submission_ref(),
    ]
    .into_iter()
    .flatten()
    .map(|hold| hold.allocation_id)
    .collect();
    assert!(!ids.is_empty());
    let records: Vec<_> = Renderer::gpu_process_allocations()
        .into_iter()
        .filter(|record| ids.contains(&record.id))
        .map(|record| {
            (
                record.id,
                record.submitted_source_bytes,
                record.submitted_transfer_bytes,
            )
        })
        .collect();
    assert_eq!(records.len(), ids.len());
    records
}

#[test]
#[ignore = "requires exact8x Vulkan output/readback; correctness and timer conformance, not performance"]
fn exact8_prefix_copy_matches_full_output_and_preserves_failure_recovery() {
    let mut c = capture8();
    let state = crate::gpu_surface_pass::board_fixture_state();
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    let expected = full_reference(&mut c);
    c.renderer.frame_observer = Some(|frame| {
        if frame.submitted_frame == Some(true) && frame.renderer.prefix_copy_work().0 {
            assert_eq!(frame.renderer.world_bundle_execution_count(), 0);
            assert!(
                frame
                    .renderer
                    .grid_geometry_admission()
                    .unwrap()
                    .iter()
                    .all(|g| !g.encoded())
            );
            let screen = frame.renderer.encoded_screen_geometry().unwrap();
            assert!(screen[0].is_none(), "warm frame must not draw panel prefix");
        }
        Ok(())
    });
    let cold = frame(&mut c, true);
    assert!(c.renderer.world_bundle_execution_count() > 0);
    assert_eq!(
        c.renderer.prefix_copy_work(),
        (false, 640 * 480 * 4 * 8 * 2)
    );
    assert!(
        cold.as_raw() == expected.as_raw(),
        "cold split differs from full8x"
    );
    assert_eq!(c.renderer.surface_attachment_reserved_generations(), 1);
    let observer = c.renderer.surface_attachment_allocation_observer();
    assert_eq!(
        observer.allocations().len(),
        2,
        "A/B aliases counted once per image"
    );
    let world_before = world_uploads(&c);
    let viewport = c
        .renderer
        .render_session()
        .prepared()
        .unwrap()
        .surface_passes()
        .iter()
        .find(|pane| pane.surface == crate::SceneSurface::Board)
        .unwrap()
        .scene_viewport;
    for (fx, fy) in [(0.3, 0.4), (0.5, 0.5), (0.7, 0.6)] {
        let (x, y) = (
            viewport.x + viewport.width * fx,
            viewport.y + viewport.height * fy,
        );
        let session = c.renderer.render_session_mut();
        assert!(session.update_pointer(PointerUpdate {
            generation: session.pointer_generation(),
            cursor: Some(datum_gui_protocol::ScreenPointPx { x, y }),
            hover: None,
            style: datum_gui_protocol::CrosshairStyle::FullViewport,
        }));
        assert!(
            !session
                .prepared()
                .unwrap()
                .board_interaction_vertices()
                .is_empty()
        );
        let actual = frame(&mut c, true);
        assert!(
            actual.as_raw() != cold.as_raw(),
            "pointer must visibly change suffix pixels"
        );
        assert_eq!(
            world_uploads(&c),
            world_before,
            "warm world allocation and upload counters must remain unchanged"
        );
        assert!(
            c.renderer.prefix_copy_work().0,
            "pointer failed to reuse prefix"
        );
        let reference = full_reference(&mut c);
        assert!(
            actual.as_raw() == reference.as_raw(),
            "warm split differs at {x},{y}"
        );
    }
    c.renderer
        .enable_gpu_measurements(
            &c.device,
            &c.queue,
            1,
            1,
            Box::new(|event| panic!("unexpected cancellation: {event:?}")),
        )
        .unwrap();
    let measured = frame(&mut c, true);
    assert!(
        c.renderer.prefix_copy_work().0,
        "exposure should reuse successfully completed prefix"
    );
    let samples = c.renderer.poll_gpu_measurements(&c.device).unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["copy-start", "suffix"]
    );
    assert!(samples[0].frame_span_ns >= samples[0].own_pass_sum_ns);
    assert!(
        samples[0].scene_marker_ticks.is_none(),
        "warm frames execute no world/grid markers"
    );
    assert_eq!(
        observer.allocations().len(),
        3,
        "one charged 1x1 marker joins the pair"
    );
    assert_eq!(c.renderer.surface_attachment_reserved_generations(), 1);
    // Deliberately corrupt the retained pixels: the exact comparator must catch it.
    let images = c
        .renderer
        .surface_attachments
        .prefix_images(&c.device, true)
        .unwrap();
    let mut encoder = c.device.create_command_encoder(&Default::default());
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: images.prefix_view(),
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
    c.queue.submit([encoder.finish()]);
    c.renderer.hold_frame_submission(&c.queue, Some(&images));
    let corrupt = frame(&mut c, false);
    assert!(
        corrupt.as_raw() != measured.as_raw(),
        "negative control failed to expose stale/corrupt prefix"
    );
    c.renderer.poll_gpu_measurements(&c.device).unwrap();
    // Failed completion forces a rebuild; no stale validity survives the failure.
    let recovered = frame(&mut c, true);
    assert!(!c.renderer.prefix_copy_work().0);
    assert!(recovered.as_raw() == measured.as_raw());
    let samples = c.renderer.poll_gpu_measurements(&c.device).unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["frame", "suffix"]
    );
    assert!(samples[0].scene_marker_ticks.is_some());
    drop(images);
    c.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    drop(c);
    assert!(
        observer.allocations().is_empty(),
        "observer must not retain closed images"
    );
}

#[test]
#[ignore = "GPU resource-pressure control; requires supported exact8x adapter"]
fn required_other_host_storage_evicts_optional_prefix_without_releasing_live_holds() {
    use crate::text_gpu::budget::{GpuReservation, gpu_process};
    let mut c = capture8();
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    let expected = frame(&mut c, true);
    let observer = c.renderer.surface_attachment_allocation_observer();
    let held = c
        .renderer
        .surface_attachments
        .prefix_images(&c.device, false)
        .unwrap();
    let process = gpu_process();
    let filler = process.reserve(process.available()).unwrap();
    // Registry eviction cannot reclaim a live encode/submission alias early.
    assert!(GpuReservation::new(16, vec![]).is_err());
    assert_eq!(observer.allocations().len(), 2);
    assert!(observer.allocations().iter().any(|a| a.retiring));
    drop(held);
    assert_eq!(observer.allocations().len(), 1);
    // A different host can now admit required resources using the released A.
    let other = Renderer::new(&c.device, &c.queue, OUTPUT_FORMAT, 8).unwrap();
    assert_eq!(c.renderer.surface_attachment_reserved_generations(), 1);
    let actual = frame(&mut c, true);
    assert_eq!(
        c.renderer.prefix_copy_work(),
        (false, 0),
        "suppression must prevent re-admission loops"
    );
    assert!(
        actual.as_raw() == expected.as_raw(),
        "eviction must preserve full-render output"
    );
    assert_eq!(observer.allocations().len(), 1);
    drop(other);
    drop(filler);
    c.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    drop(c);
    assert!(observer.allocations().is_empty());
}

#[test]
#[ignore = "GPU encode-to-submit eviction lifetime control; requires exact8x"]
fn encoded_images_survive_eviction_until_actual_submission_completion() {
    let mut c = capture8();
    let state = datum_gui_protocol::load_fixture_workspace_state();
    prepare(&mut c, &state, &SourceEpoch::default());
    frame(&mut c, true);
    let observer = c.renderer.surface_attachment_allocation_observer();
    let images = c
        .renderer
        .surface_attachments
        .prefix_images(&c.device, true)
        .unwrap();
    let mut encoder = c.device.create_command_encoder(&Default::default());
    images.copy(&mut encoder);
    crate::text_gpu::optional_residency::evict_all();
    assert_eq!(c.renderer.surface_attachment_snapshots().count(), 1);
    assert_eq!(
        observer.allocations().len(),
        3,
        "local encoding references remain charged"
    );
    // A successful required allocation after eviction must not lose submitted A.
    let required = crate::text_gpu::budget::GpuReservation::new(16, vec![]).unwrap();
    c.queue.submit([encoder.finish()]);
    c.renderer.hold_frame_submission(&c.queue, Some(&images));
    let used: Vec<_> = c.renderer.surface_attachment_usage().collect();
    assert_eq!(used.len(), 3);
    assert_eq!(used.iter().filter(|(_, current)| !current).count(), 2);
    drop(images);
    assert_eq!(
        observer.allocations().len(),
        3,
        "queue callback owns evicted images after local handles drop"
    );
    c.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(
        observer.allocations().len(),
        1,
        "completion releases A and marker only"
    );
    drop(required);
    drop(c);
    assert!(observer.allocations().is_empty());
}

#[test]
#[ignore = "GPU coherent pair retirement and recovery control; requires exact8x"]
fn pair_generation_allowance_survives_replacement_and_device_recovery() {
    let mut c = capture8();
    let state = datum_gui_protocol::load_fixture_workspace_state();
    prepare(&mut c, &state, &SourceEpoch::default());
    frame(&mut c, true);
    let observer = c.renderer.surface_attachment_allocation_observer();
    let first = c
        .renderer
        .surface_attachments
        .prefix_images(&c.device, false)
        .unwrap();
    c.renderer
        .prepare_surface_attachment(&c.device, 32, 64, || true)
        .unwrap();
    let second = c
        .renderer
        .surface_attachments
        .prefix_images(&c.device, false)
        .unwrap();
    assert_eq!(observer.allocations().len(), 4);
    assert_eq!(c.renderer.surface_attachment_reserved_generations(), 2);
    let mut replacement = c
        .renderer
        .recreate_for_device(&c.device, &c.queue, OUTPUT_FORMAT, 8)
        .unwrap();
    assert!(
        replacement
            .prepare_surface_attachment(&c.device, 64, 64, || true)
            .is_err()
    );
    assert!(
        c.renderer
            .prepare_surface_attachment(&c.device, 64, 64, || true)
            .is_err()
    );
    drop(c.renderer);
    assert_eq!(
        observer.allocations().len(),
        4,
        "close cannot release outstanding pair references"
    );
    drop(first);
    assert_eq!(observer.allocations().len(), 2);
    replacement
        .prepare_surface_attachment(&c.device, 64, 64, || true)
        .unwrap();
    assert_eq!(replacement.surface_attachment_reserved_generations(), 2);
    drop(second);
    assert!(observer.allocations().is_empty());
    assert_eq!(replacement.surface_attachment_reserved_generations(), 1);
}

#[test]
#[ignore = "GPU bounded optional-refusal fallback control; requires exact8x"]
fn required_refusal_defers_once_then_propagates_without_recreating_prefix() {
    let mut c = capture8();
    let state = datum_gui_protocol::load_fixture_workspace_state();
    prepare(&mut c, &state, &SourceEpoch::default());
    frame(&mut c, true);
    let session = c.renderer.render_session_mut();
    let viewport = session
        .prepared()
        .unwrap()
        .surface_passes()
        .iter()
        .find(|pane| pane.surface == crate::SceneSurface::Board)
        .unwrap()
        .scene_viewport;
    assert!(session.update_pointer(PointerUpdate {
        generation: session.pointer_generation(),
        cursor: Some(datum_gui_protocol::ScreenPointPx {
            x: viewport.x + viewport.width * 0.5,
            y: viewport.y + viewport.height * 0.5
        }),
        hover: None,
        style: datum_gui_protocol::CrosshairStyle::FullViewport,
    }));
    assert!(
        !session
            .prepared()
            .unwrap()
            .board_interaction_vertices()
            .is_empty()
    );
    let target = target(&c);
    let view = target.create_view(&Default::default());
    // A fresh stream forces a required allocation; warm spare CPU/GPU capacity
    // must not accidentally satisfy this deliberate admission-refusal control.
    c.renderer.board_interaction_gpu = c.renderer.board_interaction_gpu.replacement();
    let budget = c.renderer.screen_budget.clone();
    let filler = budget.reserve(budget.available()).unwrap();
    for deferred in [true, false] {
        let plan = c
            .renderer
            .render_session_mut()
            .prepare_frame(1, 1, 1, true, c.width, c.height)
            .unwrap();
        let mut acquisitions = 0;
        let result = c.renderer.encode_frame(
            plan,
            &c.device,
            &c.queue,
            &mut acquisitions,
            &mut |count| {
                *count += 1;
                Ok(Some(view.clone()))
            },
            &mut |_, _| panic!("refusal must not submit"),
        );
        assert_eq!(acquisitions, 0);
        if deferred {
            assert!(result.unwrap().is_none());
        } else {
            assert!(result.is_err());
        }
        assert!(c.renderer.render_session().has_pending_frame());
        assert_eq!(c.renderer.surface_attachment_snapshots().count(), 1);
    }
    drop(filler);
    frame(&mut c, true);
    assert_eq!(c.renderer.prefix_copy_work(), (false, 0));
}
