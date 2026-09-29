use super::super::super::frame_revision::Target;
use super::super::{Key, Prefix};
use super::*;
use crate::gpu_surface::PairIdentity;

fn contains(pixels: &Pixels, x: u32, y: u32) -> bool {
    pixels
        .rectangles()
        .iter()
        .any(|r| x >= r[0] && y >= r[1] && x < r[2] && y < r[3])
}
fn support(x: u32) -> Pixels {
    let mut pixels = Pixels::default();
    pixels.push([x, 3, x + 2, 20]).unwrap();
    pixels
}

#[test]
fn presented_support_survives_coalescing_and_uncertain_frames_fail_closed() {
    let pair = PairIdentity::test_identity(1);
    let key = Key {
        preparation: 1,
        strong_revision: 1,
        target: Target {
            host: 1,
            device: 1,
            configuration: 1,
        },
    };
    let mut prefix = Prefix::default();
    prefix.begin(Some(key), Some(support(10)), 1);
    assert!(prefix.damage(pair).is_none());
    prefix.encoded(pair, false, 100);
    let encoded = prefix.take_encoded();
    prefix.complete(encoded, key, true);
    // Revision2's unpresented pointer is never a replacement for revision1's
    // old pixels. Only newest desired support and last completed support matter.
    prefix.begin(Some(key), Some(support(40)), 3);
    let damage = prefix.damage(pair).unwrap();
    assert!(contains(&damage, 10, 5) && contains(&damage, 40, 5));
    assert!(!contains(&damage, 25, 5));
    assert!(prefix.damage(PairIdentity::test_identity(2)).is_none());
    prefix.encoded(pair, true, 0);
    let encoded = prefix.take_encoded();
    prefix.complete(encoded, key, false);
    prefix.begin(Some(key), Some(support(45)), 4);
    assert!(prefix.damage(pair).is_none());
    prefix.encoded(pair, false, 100);
    let encoded = prefix.take_encoded();
    prefix.complete(encoded, key, true);
    prefix.begin(Some(key), Some(support(45)), 4);
    assert!(
        prefix.damage(pair).unwrap().rectangles().is_empty(),
        "exposure only"
    );
    let changed = Key {
        strong_revision: 5,
        ..key
    };
    prefix.begin(Some(changed), Some(support(45)), 5);
    assert!(prefix.damage(pair).is_none(), "strong change wins");
    prefix.begin(Some(key), None, 5);
    assert!(
        prefix.damage(pair).is_none(),
        "unknown support cannot reuse"
    );
}

#[test]
fn rectangles_reject_unknown_geometry_and_bounded_union_overflow() {
    let vertex = |x, y| Vertex {
        pos: [x, y],
        color: [1.; 3],
    };
    let mut p = Pixels::default();
    p.triangles(
        &[vertex(-3.2, 5.3), vertex(7.2, 5.3), vertex(7.2, 6.8)],
        [0, 0, 8, 8],
    )
    .unwrap();
    assert!(contains(&p, 0, 4) && contains(&p, 7, 7));
    assert!(p.rectangles().iter().all(|r| r[2] <= 8 && r[3] <= 8));
    assert!(
        p.triangles(&[vertex(f32::NAN, 0.); 3], [0, 0, 8, 8])
            .is_none()
    );
    assert!(p.triangles(&[vertex(0., 0.); 2], [0, 0, 8, 8]).is_none());
    let mut full = Pixels::default();
    for x in 0..MAX_RECTS as u32 {
        full.push([x * 3, 0, x * 3 + 1, 1]).unwrap();
    }
    assert!(full.union(support(200)).is_none());
    assert_eq!(
        full.union(full).unwrap(),
        full,
        "duplicate bounds need no slots"
    );
}

#[test]
fn real_pointer_triangles_keep_thin_support_and_cover_enter_move_leave() {
    use datum_gui_protocol::{CrosshairStyle, ScreenPointPx};
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = crate::RetainedScene::from_workspace(&state, 1280, 800);
    let mut scene = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        crate::CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let viewport = scene
        .surface_passes()
        .iter()
        .find(|p| p.surface == SceneSurface::Board)
        .unwrap()
        .scene_viewport;
    let mut previous = Pixels::default();
    for style in [
        CrosshairStyle::Local,
        CrosshairStyle::FullViewport,
        CrosshairStyle::None,
    ] {
        for cursor in [
            Some(ScreenPointPx {
                x: viewport.x + viewport.width * 0.4 + 0.25,
                y: viewport.y + viewport.height * 0.5 + 0.75,
            }),
            Some(ScreenPointPx {
                x: viewport.x + viewport.width * 0.6 + 0.25,
                y: viewport.y + viewport.height * 0.7 + 0.75,
            }),
            None,
        ] {
            scene.refresh_pointer(cursor, style, None, None);
            let pixels = Pixels::capture(&scene, [1280, 800]).unwrap();
            let damage = previous.union(pixels).unwrap();
            for r in previous.rectangles().iter().chain(pixels.rectangles()) {
                assert!(contains(&damage, r[0], r[1]));
                assert!(contains(&damage, r[2] - 1, r[3] - 1));
            }
            let area: u32 = pixels
                .rectangles()
                .iter()
                .map(|r| (r[2] - r[0]) * (r[3] - r[1]))
                .sum();
            assert!(
                area < (viewport.width * viewport.height * 0.1) as u32,
                "crosshair must not become a full-pane bounding rectangle"
            );
            for triangle in scene.board_interaction_vertices().as_chunks::<3>().0 {
                let center = [
                    triangle.iter().map(|v| v.pos[0]).sum::<f32>() / 3.,
                    triangle.iter().map(|v| v.pos[1]).sum::<f32>() / 3.,
                ];
                assert!(contains(
                    &pixels,
                    center[0].floor() as u32,
                    center[1].floor() as u32
                ));
            }
            previous = pixels;
        }
        assert!(previous.rectangles().is_empty(), "leave has no new pixels");
    }
}

#[test]
fn direct_full_render_invalidates_presented_composition() {
    let pair = PairIdentity::test_identity(1);
    let key = Key {
        preparation: 1,
        strong_revision: 1,
        target: Target {
            host: 1,
            device: 1,
            configuration: 1,
        },
    };
    let mut prefix = Prefix::default();
    prefix.begin(Some(key), Some(support(10)), 1);
    prefix.encoded(pair, false, 100);
    let encoded = prefix.take_encoded();
    prefix.complete(encoded, key, true);
    prefix.reset_work();
    prefix.begin(Some(key), Some(support(20)), 2);
    assert!(
        prefix.damage(pair).is_none(),
        "raw painter discarded B without a session receipt"
    );
}
