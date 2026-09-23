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
        }]
        .into(),
        world_hit_index: datum_gui_viewport::SpatialHitIndex::new(vec![WorldHitRegion {
            target: HitTarget::AuthoredObject(String::with_capacity(64)),
            layer_id: Some(String::with_capacity(32)),
            shape: WorldHitShape::Polyline {
                path: vec![PointNm { x: 0, y: 0 }, PointNm { x: 10, y: 20 }],
                half_width_nm: 2.0,
            },
        }])
        .into(),
    });
    let live = || {
        let usage = scope.usage();
        (usage.payload_bytes + usage.tracking_bytes) as usize
    };
    assert_eq!(scene.heap_payload_bytes(), Some(live()));
    let cloned = scope.with(|| scene.clone());
    assert_eq!(
        scene.heap_payload_bytes(),
        Some(live()),
        "scene clone allocates no metadata"
    );
    assert!(std::sync::Arc::ptr_eq(
        &scene.draw_commands,
        &cloned.draw_commands
    ));
    assert!(std::sync::Arc::ptr_eq(
        &scene.world_hit_index,
        &cloned.world_hit_index
    ));
    let observer = scene.geometry_observer();
    let second = observer.clone();
    assert_eq!(observer.heap_bytes_excluding([&second]), 0);
    drop(scene);
    assert!(observer.is_live());
    assert_eq!(observer.heap_bytes_excluding([]), live());
    drop(cloned);
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

#[test]
fn document_keeps_weak_only_allocations_charged_until_last_observer_releases() {
    use super::retained_scene_owner::document_cpu;
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "weak-document-lifetime".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let observer = scene.geometry_observer();
    let second = scene.geometry_observer();
    let before = observer.document_cpu_payload_bytes();
    drop(scene);
    assert!(!observer.is_live());
    let weak_bytes = observer.document_cpu_payload_bytes();
    assert!(weak_bytes > observer.heap_bytes_excluding([]));
    assert!(weak_bytes < before);
    // Reopening this document must share the still-live accounting identity.
    let reopened = RetainedScene::from_workspace(&state, 960, 720);
    assert!(second.shares_document_with(&reopened.geometry_observer()));
    drop(reopened);
    drop(observer);
    assert!(second.document_cpu_payload_bytes() >= weak_bytes);
    assert!(
        document_cpu::gpu_usage()
            .iter()
            .any(|usage| usage.scene_id == state.scene.scene_id)
    );
    drop(second);
    assert!(
        !document_cpu::gpu_usage()
            .iter()
            .any(|usage| usage.scene_id == state.scene.scene_id)
    );
}

#[test]
fn history_key_and_replacement_storage_share_atomic_admission_and_separate_release() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "history-storage-reservation".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let observer = scene.geometry_observer();
    let baseline = observer.document_cpu_payload_bytes();
    let remaining = retained_scene_owner::document_cpu::DOCUMENT_LIMIT - baseline;
    let (entry, storage) = observer
        .try_charge_document_history_storage(32, remaining - 32)
        .unwrap();
    assert!(observer.try_charge_document_history_storage(1, 1).is_none());
    assert!(
        observer
            .try_charge_document_history_storage(usize::MAX, 1)
            .is_none()
    );
    drop(entry);
    assert_eq!(observer.document_history_entries(), 0);
    assert_eq!(
        observer.document_cpu_payload_bytes(),
        baseline + remaining - 32
    );
    let (entry, next_storage) = observer
        .try_charge_document_history_storage(16, 16)
        .unwrap();
    drop(storage);
    assert_eq!(observer.document_cpu_payload_bytes(), baseline + 32);
    drop(entry);
    drop(next_storage);
    assert_eq!(observer.document_cpu_payload_bytes(), baseline);
}
