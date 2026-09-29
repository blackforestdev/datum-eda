//! Additional r3 graph controls, called only by the single bounded P630 batch.
use super::*;

pub(super) fn pointer(c: &mut OffscreenRenderer, fraction: Option<(f32, f32)>) {
    let viewport = c
        .renderer
        .render_session()
        .prepared()
        .unwrap()
        .surface_passes()
        .iter()
        .find(|p| p.surface == crate::SceneSurface::Board)
        .unwrap()
        .scene_viewport;
    let cursor = fraction.map(|(x, y)| ScreenPointPx {
        x: viewport.x + viewport.width * x + 0.375,
        y: viewport.y + viewport.height * y + 0.625,
    });
    let session = c.renderer.render_session_mut();
    assert!(session.update_pointer(PointerUpdate {
        generation: session.pointer_generation(),
        cursor,
        hover: None,
        style: CrosshairStyle::FullViewport,
    }));
}

pub(super) fn motion(c: &mut OffscreenRenderer, label: &str) {
    for position in [Some((0.25, 0.3)), Some((0.72, 0.6)), None] {
        pointer(c, position);
        exact(c, label, true);
    }
}

pub(super) fn prove() {
    use crate::gpu_surface::prefix_negative_control::{Fault, set};
    let mut c = reference_capture8();
    c.width = 1280;
    c.height = 800;
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    let source = SourceEpoch::default();
    prepare(&mut c, &state, &source);
    exact(&mut c, "r3 cold baseline", false);
    pointer(&mut c, Some((0.2, 0.3)));
    exact(&mut c, "r3 pointer entered", true);
    // The unpresented middle position must never replace old presented support.
    pointer(&mut c, Some((0.4, 0.5)));
    pointer(&mut c, Some((0.7, 0.6)));
    exact(&mut c, "coalesced newest pointer", true);
    state.ui.active_menu = Some("File".to_owned());
    prepare(&mut c, &state, &source);
    exact(&mut c, "mixed composition invalidation", false);
    motion(&mut c, "menu overlap masked suffix");
    state.ui.active_menu = None;
    prepare(&mut c, &state, &source);
    exact(&mut c, "menu removed", false);

    for (fault, label) in [
        (Fault::OldDamageMissing, "omitted old damage"),
        (Fault::MissingSuffixMask, "omitted shared suffix predicate"),
        (Fault::StaleWorking, "unrestored atlas"),
    ] {
        pointer(&mut c, Some((0.2, 0.3)));
        exact(&mut c, "negative old presented support", true);
        if fault == Fault::StaleWorking {
            pointer(&mut c, Some((0.45, 0.4)));
            let _ = frame(&mut c, false);
            pointer(&mut c, Some((0.7, 0.6)));
        } else if fault == Fault::OldDamageMissing {
            pointer(&mut c, None);
        } else {
            pointer(&mut c, Some((0.7, 0.6)));
        }
        let expected = full_reference(&mut c);
        set(fault);
        let wrong = frame(&mut c, false);
        assert!(
            wrong.as_raw() != expected.as_raw(),
            "oracle failed to reject {label}"
        );
        assert!(
            exact(&mut c, "fault recovery reconstructs C", false).as_raw() == expected.as_raw()
        );
        eprintln!("r3 negative rejected: {label}");
    }

    // R4 has no per-frame mapping/mask allocation. Its source IDs are immutable
    // draw arguments; admission and retirement concern the coherent five images.
    assert!(c.renderer.surface_attachment_reserved_generations() <= 2);
    let images = c
        .renderer
        .surface_attachment_snapshots()
        .collect::<Vec<_>>();
    assert_eq!(images.len(), 5, "UNORM aliases must not add allocations");
    assert_eq!(images.iter().filter(|image| image.samples == 8).count(), 3);
    assert_eq!(images.iter().filter(|image| image.samples == 1).count(), 2);
    assert_eq!(
        images
            .iter()
            .map(|image| image.payload_bytes.unwrap())
            .sum::<u64>(),
        79_069_184
    );
    assert!(
        Renderer::gpu_process_allocations()
            .iter()
            .map(|r| r.bytes)
            .sum::<u64>()
            <= 512 * 1024 * 1024
    );
    image_lifetime(&mut c);
}

/// Eviction between encode and submit cannot release A while submitted work owns it.
pub(super) fn image_lifetime(c: &mut OffscreenRenderer) {
    let expected = full_reference(c);
    let observer = c.renderer.surface_attachment_allocation_observer();
    let images = c
        .renderer
        .surface_attachments
        .prefix_images(&c.device)
        .unwrap();
    let held: Vec<_> = images.submission_refs().collect();
    let mut encoder = c.device.create_command_encoder(&Default::default());
    images.copy(&mut encoder);
    crate::text_gpu::optional_residency::evict_all();
    assert_eq!(c.renderer.surface_attachment_snapshots().count(), 1);
    assert_eq!(observer.allocations().len(), 5);
    c.queue.submit([encoder.finish()]);
    c.renderer.hold_frame_submission(&c.queue, Some(&images));
    drop(images);
    assert_eq!(observer.allocations().len(), 5);
    assert_eq!(
        observer.allocations().iter().filter(|a| a.retiring).count(),
        4
    );
    c.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    drop(held);
    assert_eq!(observer.allocations().len(), 1);
    let actual = frame(c, true);
    assert_eq!(c.renderer.prefix_copy_work(), (false, 0));
    assert!(
        actual.as_raw() == expected.as_raw(),
        "eviction fallback must preserve exact output"
    );
    eprintln!("r4 five-image aliases, encoded eviction, queue holds and full fallback passed");
}
