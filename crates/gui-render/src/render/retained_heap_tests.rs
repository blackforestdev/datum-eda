//! Compare scene accounting with real allocator ownership, including weak retirement.
use super::*;

#[test]
fn complete_scene_owned_capacities_match_allocator_and_weak_release() {
    let scope = crate::cpu_alloc::Scope::new("retained-scene-heap-proof");
    let scene = scope.with(|| RetainedScene {
        surface_size_independent: true,
        world_vertices: vec![
            Vertex {
                pos: [1.0; 2],
                color: [0.5; 3]
            };
            100
        ]
        .into(),
        world_strokes: Vec::new().into(),
        draw_commands: vec![RetainedDrawCommand::Quads {
            layer_id: Some(String::with_capacity(80)),
            range: 0..100,
        }],
        world_hit_index: datum_gui_viewport::SpatialHitIndex::new(vec![WorldHitRegion {
            target: HitTarget::AuthoredObject(String::with_capacity(64)),
            layer_id: Some(String::with_capacity(32)),
            shape: WorldHitShape::Polyline {
                path: vec![PointNm { x: 0, y: 0 }, PointNm { x: 10, y: 20 }],
                half_width_nm: 2.0,
            },
        }]),
    });
    let live = || {
        let usage = scope.usage();
        (usage.payload_bytes + usage.tracking_bytes) as usize
    };
    assert_eq!(scene.heap_payload_bytes(), Some(live()));
    let observer = scene.geometry_observer();
    let second = observer.clone();
    assert_eq!(observer.heap_bytes_excluding([&second]), 0);
    drop(scene);
    assert!(!observer.is_live());
    assert_eq!(
        observer.heap_bytes_excluding([]),
        live(),
        "weak handles retain charged Arc containers only"
    );
    drop(observer);
    assert_eq!(second.heap_bytes_excluding([]), live());
    drop(second);
    assert_eq!(live(), 0);
}
