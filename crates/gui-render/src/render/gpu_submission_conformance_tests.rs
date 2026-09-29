//! Bounded timer component proof; synthetic pressure amplifies existing fixtures.
//! These explicit local identities do not claim native input-attribution proof.
use super::prefix_graph_tests::{reference_capture8, target};
use super::*;

#[test]
#[ignore = "approved r3 single timer batch: P630 world/glyph continuations and final mixed upload"]
fn world_and_glyph_submissions_include_transfer_and_final_graph() {
    for kind in ["world", "glyph"] {
        let mut c = reference_capture8();
        c.width = 960;
        c.height = 720;
        let (prepared, retained) = if kind == "world" {
            let state = crate::gpu_surface_pass::board_fixture_state();
            let mut retained = RetainedScene::from_workspace(&state, 960, 720);
            let mut vertices = retained.world_vertices.to_vec();
            vertices.resize(220_000, vertices[0]);
            retained.world_vertices =
                crate::gpu_data::shared_geometry::SharedGeometry::for_document(
                    vertices,
                    "r3-timer-world",
                );
            let prepared = PreparedScene::from_workspace_for_surface(
                &state,
                960,
                720,
                1.0,
                CameraState::fit_to_bounds(&state.scene.bounds),
                &retained,
            )
            .unwrap();
            (prepared, retained)
        } else {
            let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
            let mut prepared =
                PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
                    .unwrap();
            let template = prepared.menu_overlay_text_runs[0].clone();
            prepared.menu_overlay_text_runs = (0..6)
                .map(|i| {
                    let mut run = template.clone();
                    run.text = "W".into();
                    run.rich_spans.clear();
                    run.size = 1800.0 + i as f32 * 20.0;
                    run.x = i as f32 * 20.0;
                    run.y = -900.0;
                    run.clip_bounds = None;
                    run.layout_size = Some((4000.0, 4000.0));
                    run
                })
                .collect();
            (prepared, RetainedScene::empty())
        };
        c.renderer
            .enable_gpu_measurements(
                &c.device,
                &c.queue,
                1,
                1,
                Box::new(|r| panic!("unexpected timer cancellation: {r:?}")),
            )
            .unwrap();
        let target = target(&c);
        let view = target.create_view(&Default::default());
        let mut submissions = 0;
        let mut complete = false;
        for attempt in 1..=6 {
            c.renderer.measurement_attempt = [1, attempt, 1, 1, 1, 1, 1];
            c.renderer.measurement_workload = [1, 4, 0, 0, 1, 0, 0, 0];
            complete = c
                .renderer
                .render_with_submission(
                    &c.device,
                    &c.queue,
                    &view,
                    &prepared,
                    &retained,
                    None,
                    960,
                    720,
                    &mut |_| submissions += 1,
                )
                .unwrap();
            target.hold_submission(&c.queue);
            c.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(std::time::Duration::from_secs(2)),
                })
                .unwrap();
            if complete {
                break;
            }
        }
        assert!(
            complete,
            "bounded continuation capacity must cover the declared component"
        );
        assert!(submissions > 1, "fixture must execute continuation work");
        let final_upload = c.renderer.last_upload_frame().unwrap();
        assert_eq!(final_upload.rendered, Some(true));
        assert!(
            final_upload.totals.buffer_copy_bytes > 0,
            "final mixed buffer upload must be exercised"
        );
        let samples = c.renderer.poll_gpu_measurements(&c.device).unwrap();
        assert_eq!(samples.len(), 1);
        let sample = &samples[0];
        assert_eq!(sample.submission_manifest.len(), submissions);
        assert!(sample.raw_ticks.windows(2).all(|p| p[0] <= p[1]));
        assert!(sample.frame_span_ns >= sample.own_pass_sum_ns);
        assert_eq!(
            sample
                .passes_ns
                .iter()
                .filter(|p| p.0 == "upload-leading")
                .count(),
            submissions
        );
        for (index, boundary) in sample.submission_manifest.iter().enumerate() {
            assert_eq!(
                boundary.kind,
                if index + 1 == submissions {
                    "final"
                } else {
                    kind
                }
            );
            assert_eq!(boundary.attempt, [1, index as u64 + 1, 1, 1, 1, 1, 1]);
            assert!(
                boundary.first_tick <= boundary.transfer_first_tick
                    && boundary.transfer_first_tick <= boundary.transfer_last_tick
                    && boundary.transfer_last_tick <= boundary.last_tick
            );
            assert!(
                boundary.transfer_interval_ns > 0.0,
                "real uploaded bytes need a measured interval"
            );
        }
        let first = sample.submission_manifest.first().unwrap();
        let last = sample.submission_manifest.last().unwrap();
        assert_eq!(
            sample.frame_span_ns,
            (last.last_tick - first.first_tick) as f64 * sample.period_ns
        );
        assert_eq!(
            c.renderer.gpu_measurement_drained_manifest().unwrap(),
            Some([1, 1, 1])
        );
        eprintln!(
            "complete timer {kind}: {submissions} submissions; span={}ns, pass_sum={}ns, final_transfer={}ns",
            sample.frame_span_ns, sample.own_pass_sum_ns, last.transfer_interval_ns
        );
    }
    cancelled_continuation_preserves_lineage_and_lifetimes();
}

fn cancelled_continuation_preserves_lineage_and_lifetimes() {
    use crate::text_gpu::lifetime::Kind;
    let mut c = reference_capture8();
    c.width = 960;
    c.height = 720;
    let state = crate::gpu_surface_pass::board_fixture_state();
    let mut retained = RetainedScene::from_workspace(&state, 960, 720);
    let mut vertices = retained.world_vertices.to_vec();
    vertices.resize(220_000, vertices[0]);
    retained.world_vertices =
        crate::gpu_data::shared_geometry::SharedGeometry::for_document(vertices, "r3-timer-cancel");
    let prepared = PreparedScene::from_workspace_for_surface(
        &state,
        960,
        720,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let cancelled = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let observer = cancelled.clone();
    c.renderer
        .enable_gpu_measurements(
            &c.device,
            &c.queue,
            7,
            7,
            Box::new(move |r| observer.lock().unwrap().push(r)),
        )
        .unwrap();
    c.renderer.measurement_attempt = [7; 7];
    c.renderer.measurement_workload = [7, 4, 0, 0, 1, 0, 0, 0];
    let target = target(&c);
    let view = target.create_view(&Default::default());
    let mut submitted = 0;
    assert!(
        !c.renderer
            .render_with_submission(
                &c.device,
                &c.queue,
                &view,
                &prepared,
                &retained,
                None,
                960,
                720,
                &mut |_| submitted += 1
            )
            .unwrap()
    );
    assert_eq!(submitted, 1);
    target.hold_submission(&c.queue);
    // Keep the actual continuation's encoding owner alive through measurement
    // owner close and real queue completion; do not fabricate a resource hold.
    let pending = c.renderer.pending_measurement.take().unwrap();
    let ids: Vec<_> = Renderer::gpu_process_allocations()
        .iter()
        .filter(|r| {
            matches!(r.kind, Kind::Query | Kind::QueryResolve | Kind::Readback)
                || (r.kind == Kind::Attachment && r.bytes == 4)
        })
        .map(|r| r.id)
        .collect();
    drop(c.renderer);
    let records = cancelled.lock().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].submitted_lineage.len(), 1);
    assert_eq!(records[0].submitted_lineage[0].1, "world");
    assert_eq!(records[0].submitted_lineage[0].2, [7; 7]);
    assert_eq!(
        records[0].submission,
        Some(records[0].submitted_lineage[0].0)
    );
    drop(records);
    c.device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(2)),
        })
        .unwrap();
    let live: Vec<_> = Renderer::gpu_process_allocations()
        .into_iter()
        .filter(|r| ids.contains(&r.id))
        .collect();
    assert_eq!(
        live.len(),
        4,
        "continuation retains query/resolve/readback plus marker after close"
    );
    assert!(
        live.iter()
            .all(|r| r.retiring && r.submission_references > 0)
    );
    drop(pending);
    assert!(
        !Renderer::gpu_process_allocations()
            .iter()
            .any(|r| ids.contains(&r.id))
    );
    eprintln!(
        "submitted continuation cancellation preserves lineage and charged resources until final owner release"
    );
}
