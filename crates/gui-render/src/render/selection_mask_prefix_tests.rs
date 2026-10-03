//! Selection suffix pixels remain coherent with the existing PM049/052 image graph.
use super::*;
#[test]
#[ignore = "requires pinned Intel GPU: exact8x native mask and prefix replay"]
fn native_mask_exact8x_suffix_reuses_world_and_matches_cold_reference() {
    let (fixture, mut state) = crate::native_selection_overlay::tests::selected_state();
    let source = fixture.source_bytes();
    state.ui.global_preferences.high_contrast_noncolor = true;
    let mut host = reference_capture8();
    let epoch = SourceEpoch::default();
    prepare(&mut host, &state, &epoch);
    let first = frame(&mut host, true);
    assert!(host.renderer.selection_mask_attachment_snapshot().is_some());
    assert_eq!(first, full_reference(&mut host));
    let world = world_uploads(&host);
    let mask = host.renderer.selection_mask_attachment_snapshot().unwrap();
    assert_eq!(mask.samples, 8);
    assert_eq!(mask.payload_bytes, Some(640 * 100 * 4 * 8));
    let viewport = host
        .renderer
        .render_session()
        .prepared()
        .unwrap()
        .surface_passes()[0]
        .scene_viewport;
    for (fx, fy) in [(0.3, 0.4), (0.6, 0.6)] {
        let session = host.renderer.render_session_mut();
        assert!(session.update_pointer(PointerUpdate {
            generation: session.pointer_generation(),
            cursor: Some(datum_gui_protocol::ScreenPointPx {
                x: viewport.x + viewport.width * fx,
                y: viewport.y + viewport.height * fy
            }),
            hover: None,
            style: datum_gui_protocol::CrosshairStyle::FullViewport,
        }));
        let warm = frame(&mut host, true);
        assert!(
            host.renderer.prefix_copy_work().0,
            "native mask preserves warm prefix reuse"
        );
        assert_eq!(world_uploads(&host), world);
        assert_eq!(warm, full_reference(&mut host));
        assert_eq!(
            host.renderer.selection_mask_attachment_snapshot(),
            Some(mask)
        );
    }
    host.renderer
        .enable_gpu_measurements(
            &host.device,
            &host.queue,
            1,
            1,
            Box::new(|event| panic!("unexpected cancellation: {event:?}")),
        )
        .unwrap();
    let session = host.renderer.render_session_mut();
    assert!(session.update_pointer(PointerUpdate {
        generation: session.pointer_generation(),
        cursor: Some(datum_gui_protocol::ScreenPointPx {
            x: viewport.x + viewport.width * 0.4,
            y: viewport.y + viewport.height * 0.5
        }),
        hover: None,
        style: datum_gui_protocol::CrosshairStyle::FullViewport,
    }));
    let measured = frame(&mut host, true);
    let samples = host.renderer.poll_gpu_measurements(&host.device).unwrap();
    assert_eq!(samples.len(), 1);
    assert!(samples[0].passes_ns.iter().any(|p| p.0 == "suffix"));
    assert!(samples[0].frame_span_ns >= samples[0].own_pass_sum_ns);
    assert!(samples[0].scene_marker_ticks.is_none());
    assert_eq!(measured, full_reference(&mut host));
    let cold_samples = host.renderer.poll_gpu_measurements(&host.device).unwrap();
    assert_eq!(cold_samples.len(), 1);
    assert!(cold_samples[0].passes_ns.iter().any(|p| p.0 == "frame"));
    assert!(cold_samples[0].passes_ns.iter().any(|p| p.0 == "suffix"));
    assert!(cold_samples[0].scene_marker_ticks.is_some());
    assert_eq!(fixture.source_bytes(), source);
}
