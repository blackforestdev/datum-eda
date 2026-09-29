//! Proposed single-case validation of actual shared-session preparation changes.
//! This component case is not native startup or performance qualification.
use super::prefix_graph_tests::{prepare, reference_capture8, target};
use super::prefix_qualification_tests::doa;
use crate::render_input::SourceEpoch;

#[test]
#[ignore = "requires explicit stopped-r3 lineage corrective GPU validation approval"]
fn shared_repreparation_retains_complete_submitted_upload_lineage() {
    let mut c = reference_capture8();
    c.width = 1280;
    c.height = 800;
    let mut state = doa();
    let source = SourceEpoch::default();
    c.renderer
        .enable_gpu_measurements(
            &c.device,
            &c.queue,
            1,
            1,
            Box::new(|r| panic!("unexpected cancellation: {r:?}")),
        )
        .unwrap();
    c.renderer
        .render_session_mut()
        .set_measurement_workload([7, 3, 1])
        .unwrap();
    prepare(&mut c, &state, &source);
    let target = target(&c);
    let view = target.create_view(&Default::default());
    let mut submissions = 0;
    let mut original = None;
    let mut complete = false;
    for turn in 1..=6 {
        let plan = c
            .renderer
            .render_session_mut()
            .prepare_frame(1, 1, 1, true, c.width, c.height)
            .unwrap();
        let frame = c
            .renderer
            .encode_frame(
                plan,
                &c.device,
                &c.queue,
                &mut submissions,
                &mut |_| Ok(Some(view.clone())),
                &mut |count, _| *count += 1,
            )
            .unwrap();
        target.hold_submission(&c.queue);
        c.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(2)),
            })
            .unwrap();
        if turn == 1 {
            assert!(
                frame.is_none(),
                "pinned F-DOA must exercise a cold continuation"
            );
            assert_eq!(submissions, 1);
            assert_eq!(c.renderer.world_upload_chunk_count(), 1);
            original = Some(c.renderer.measurement_attempt);
            let session = c.renderer.render_session_mut();
            session.end_measurement_workload(true);
            session.set_measurement_workload([7, 5, 2]).unwrap();
            session.composition_changed();
            state.ui.active_menu = Some("File".into());
            prepare(&mut c, &state, &source);
        } else if let Some(frame) = frame {
            assert!(
                c.renderer
                    .render_session_mut()
                    .complete_submitted(frame, 1, 1, 1, true)
            );
            complete = true;
            break;
        }
    }
    assert!(
        complete,
        "fixed six-submission capacity must cover this case"
    );
    let samples = c.renderer.poll_gpu_measurements(&c.device).unwrap();
    assert_eq!(samples.len(), 1);
    let sample = &samples[0];
    assert_eq!(sample.submission_manifest.len(), submissions);
    let first = sample.submission_manifest.first().unwrap();
    let last = sample.submission_manifest.last().unwrap();
    assert_eq!(first.kind, "world");
    assert_eq!(first.attempt, original.unwrap());
    assert_eq!(last.kind, "final");
    assert!(
        last.attempt[3] > first.attempt[3],
        "actual preparation must advance"
    );
    assert!(
        sample
            .submission_manifest
            .iter()
            .skip(1)
            .all(|r| r.attempt[3] == last.attempt[3])
    );
    assert_eq!(first.workload, [7, 4, 0, 0, 1, 0, 0, 0]);
    assert_eq!(
        last.workload[1] & 4,
        4,
        "earlier active demand remains pending"
    );
    assert!(sample.raw_ticks.windows(2).all(|p| p[0] <= p[1]));
    assert_eq!(
        sample.frame_span_ns,
        (last.last_tick - first.first_tick) as f64 * sample.period_ns
    );
    assert_eq!(
        c.renderer.gpu_measurement_drained_manifest().unwrap(),
        Some([1, 1, 1])
    );
    eprintln!(
        "shared re-preparation preserved {submissions} actual submissions and old/new preparation identities in one complete span"
    );
}
