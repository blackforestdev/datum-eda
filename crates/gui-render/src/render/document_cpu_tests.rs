//! Document admission, accounting and registry lifetime tests.
use super::*;
#[test]
fn pruning_reuses_charged_capacity_without_allocating_and_releases_empty_storage() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "registry-pruning-reuse".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let observer = scene.geometry_observer();
    let budget = scene.world_vertices.document_budget().unwrap().clone();
    let mut registry = Registry::default();
    let mut scenes = Vec::with_capacity(32);
    let mut local_observer = observer.clone();
    local_observer.lifetime = None;
    scenes.push(local_observer);
    let pointer = scenes.as_ptr();
    registry.insert(
        "registry-pruning-reuse".into(),
        Arc::downgrade(&budget),
        scenes,
    );
    let before = document_bytes(registry.head.as_ref().unwrap());
    let scope = crate::cpu_alloc::Scope::new("registry-pruning-allocation");
    scope.with(|| prune(&mut registry));
    let document = registry.head.as_ref().unwrap();
    assert_eq!(document.scenes.as_ptr(), pointer);
    assert_eq!(document.scenes.capacity(), 32);
    assert_eq!(document_bytes(document), before);
    assert_eq!(scope.usage().peak_payload_bytes, 0);
    drop(scene);
    scope.with(|| prune(&mut registry));
    assert_eq!(registry.head.as_ref().unwrap().scenes.capacity(), 0);
    assert_eq!(scope.usage().peak_payload_bytes, 0);
    drop(budget);
    drop(observer);
    gpu_usage();
    prune(&mut registry);
    assert!(registry.head.is_none());
}

#[test]
fn publication_refusal_preserves_registry_and_retry_registers_unique_metadata() {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.scene.scene_id = "construction-publication-refusal".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let scope = crate::cpu_alloc::Scope::new("publication-candidate");
    let mut candidate = scene.clone();
    candidate.draw_commands = scope.with(|| Arc::new(scene.draw_commands.as_ref().clone()));
    let before = usage(&scene.geometry_observer());
    let candidate_bytes = scope.usage();
    let error = register(&candidate, &scope, 0).unwrap_err();
    assert!(error.to_string().contains("registry publication exceeds"));
    assert_eq!(usage(&scene.geometry_observer()), before);
    assert_eq!(scope.usage().allocations, candidate_bytes.allocations);
    register(&candidate, &scope, DOCUMENT_LIMIT).unwrap();
    assert!(usage(&scene.geometry_observer()) > before);
    drop(candidate);
    drop(scene);
    gpu_usage();
    assert_eq!(scope.usage().allocations, 0);
}

#[test]
fn shared_owner_refusal_allocates_no_owners_and_preserves_staging() {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.scene.scene_id = "construction-owner-refusal".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let budget = scene.world_vertices.document_budget().unwrap();
    let scope = crate::cpu_alloc::Scope::new("shared-owner-staging");
    let mut vertices = scope.with(|| scene.world_vertices().to_vec());
    scope.with(|| vertices.reserve_exact(7));
    let strokes = scope.with(|| scene.world_strokes.to_vec());
    let before = scope.usage();
    let error =
        RetainedScene::admit_shared_owners(&vertices, &strokes, budget, &scope, 0).unwrap_err();
    assert!(error.to_string().contains("shared owners exceeds"));
    RetainedScene::admit_shared_owners(&vertices, &strokes, budget, &scope, DOCUMENT_LIMIT)
        .unwrap();
    assert_eq!(scope.usage().allocations, before.allocations);
    assert_eq!(vertices.as_slice(), scene.world_vertices());
}

#[test]
fn real_hit_index_admission_includes_staging_and_matches_allocator() {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.scene.scene_id = "construction-hit-index-admission".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let budget = scene.world_vertices.document_budget().unwrap();
    let scope = crate::cpu_alloc::Scope::new("hit-index-construction-proof");
    let regions = scope.with(|| scene.world_hit_index.regions().to_vec());
    assert!(!regions.is_empty());
    let layouts = datum_gui_viewport::SpatialHitIndex::<crate::HitTarget>::construction_layouts(
        regions.len(),
    )
    .unwrap();
    let extra: usize = layouts.into_iter().map(allocation_bytes).sum();
    let staging = scope.usage();
    let limit = usage(&scene.geometry_observer())
        + (staging.payload_bytes + staging.tracking_bytes) as usize
        + extra;
    let error = RetainedScene::admitted_hit_index(regions, budget, &scope, limit - 1).unwrap_err();
    assert!(error.to_string().contains("hit index exceeds"));
    assert_eq!(scope.usage().allocations, 0);
    let regions = scope.with(|| scene.world_hit_index.regions().to_vec());
    let index = scope
        .with(|| RetainedScene::admitted_hit_index(regions, budget, &scope, limit))
        .unwrap();
    let bytes = index
        .heap_bytes_with(
            |target| {
                Some(match target {
                    crate::HitTarget::AuthoredObject(id) => capacity_bytes::<u8>(id.capacity()),
                    _ => 0,
                })
            },
            |layout| Some(allocation_bytes(layout)),
        )
        .unwrap();
    let live = scope.usage();
    assert_eq!(bytes as u64, live.payload_bytes + live.tracking_bytes);
    drop(index);
    assert_eq!(scope.usage().allocations, 0);
}

#[test]
fn board_and_companion_constructors_refuse_budget_and_allow_retry() {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.scene.scene_id = "construction-board-refusal".into();
    let error = RetainedScene::from_workspace_bounded(&state, 960, 720, 1.0, 0).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("geometry emission exceeds document CPU budget")
    );
    let board = RetainedScene::try_from_workspace_for_surface(&state, 960, 720, 1.0).unwrap();
    assert!(!board.world_vertices().is_empty() || !board.world_strokes().is_empty());
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let mut schematic = datum_gui_protocol::load_kicad_schematic_workspace_state(&path).unwrap();
    schematic.scene.scene_id = "construction-schematic-refusal".into();
    state.schematic_scene = Some(schematic.scene);
    let error = RetainedScene::schematic_workspace_bounded(&state, 960, 720, 1.0, 0).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("geometry emission exceeds document CPU budget")
    );
    assert!(
        RetainedScene::try_from_workspace_schematic_for_surface(&state, 960, 720, 1.0)
            .unwrap()
            .is_some()
    );
}

#[test]
fn vertex_expansion_admission_includes_live_owners_staging_and_requested_capacity() {
    let budget = for_scene("construction-expansion-boundary");
    let scope = crate::cpu_alloc::Scope::new("construction-boundary-proof");
    let staging = scope.with(|| vec![0u8; 4096]);
    let before = scope.usage();
    let existing = usage_for_identity(&Arc::downgrade(&budget));
    let limit = existing
        + (before.payload_bytes + before.tracking_bytes) as usize
        + capacity_bytes::<crate::Vertex>(6);
    admit_vertex_expansion(&budget, &scope, 1, limit).unwrap();
    assert!(admit_vertex_expansion(&budget, &scope, 1, limit - 1).is_err());
    assert!(admit_vertex_expansion(&budget, &scope, usize::MAX, limit).is_err());
    assert_eq!(scope.usage().allocations, before.allocations);
    assert_eq!(staging.len(), 4096);
}

#[test]
fn constructor_gate_covers_work_and_unwinds_without_poisoning_future_builds() {
    let scope = crate::cpu_alloc::Scope::new("constructor-serialization");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        with_constructor(&scope, || {
            std::thread::spawn(|| assert!(CONSTRUCTION.try_lock().is_err()))
                .join()
                .unwrap();
            panic!("construction failure");
        });
    }));
    assert!(result.is_err());
    assert_eq!(with_constructor(&scope, || 42), 42);
}

#[test]
fn history_byte_admission_is_atomic_and_releases_for_retry() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "cpu-document-history-byte-admission".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let observer = scene.geometry_observer();
    let remaining = DOCUMENT_LIMIT - observer.document_cpu_payload_bytes();
    let charge = observer.try_charge_document_history(remaining).unwrap();
    assert_eq!(observer.document_cpu_payload_bytes(), DOCUMENT_LIMIT);
    assert!(observer.try_charge_document_history(1).is_none());
    assert!(observer.try_charge_document_history(usize::MAX).is_none());
    assert_eq!(observer.document_history_entries(), 1);
    drop(charge);
    assert!(observer.try_charge_document_history(remaining).is_some());
}

#[test]
fn concurrent_history_admission_cannot_exceed_six_or_release_early() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "cpu-document-history-concurrency".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let observer = scene.geometry_observer();
    let barrier = Arc::new(std::sync::Barrier::new(9));
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let observer = observer.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let charge = observer.try_charge_document_history(0);
                barrier.wait();
                barrier.wait();
                charge
            })
        })
        .collect();
    barrier.wait();
    assert_eq!(observer.document_history_entries(), 6);
    assert!(observer.try_charge_document_history(0).is_none());
    barrier.wait();
    let charges: Vec<_> = workers
        .into_iter()
        .filter_map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(charges.len(), 6);
    assert_eq!(observer.document_history_entries(), 6);
    drop(charges);
    assert_eq!(observer.document_history_entries(), 0);
    assert!(observer.try_charge_document_history(0).is_some());
}

#[test]
fn production_document_payload_and_identity_match_live_allocator_bytes() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "cpu-document-allocator-warmup".into();
    drop(RetainedScene::from_workspace(&state, 960, 720));
    gpu_usage();
    state.scene.scene_id = "cpu-document-allocator-owned".into();
    let scope = crate::cpu_alloc::Scope::new("complete-document-owner");
    let scene = scope.with(|| RetainedScene::from_workspace(&state, 960, 720));
    let observer = scene.geometry_observer();
    let construction_owner = crate::cpu_alloc::usage()
        .into_iter()
        .filter(|value| {
            value.label == "retained-board-construction" && value.owner_id > scope.usage().owner_id
        })
        .map(|value| value.owner_id)
        .max()
        .expect("constructor owner");
    let live = || {
        let value = scope.usage();
        let construction = crate::cpu_alloc::usage()
            .into_iter()
            .find(|value| value.owner_id == construction_owner)
            .map_or(0, |value| value.payload_bytes + value.tracking_bytes);
        (value.payload_bytes + value.tracking_bytes + construction) as usize
    };
    assert_eq!(usage(&observer), live());
    let clone = scene.clone();
    drop(scene);
    assert_eq!(usage(&observer), live());
    drop(clone);
    assert_eq!(
        usage(&observer),
        live(),
        "weak-only ownership remains exact"
    );
    drop(observer);
    gpu_usage();
    assert_eq!(live(), 0);
}

#[test]
fn registry_records_match_allocator_and_release_independently() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "cpu-registry-first".into();
    let first = RetainedScene::from_workspace(&state, 960, 720);
    state.scene.scene_id = "cpu-registry-second".into();
    let second = RetainedScene::from_workspace(&state, 960, 720);
    let scope = crate::cpu_alloc::Scope::new("document-registry-storage");
    let mut registry = Registry::default();
    for scene in [&first, &second] {
        let mut observer = scene.geometry_observer();
        observer.lifetime = None; // This local registry tests storage, not global leases.
        scope.with(|| {
            registry.insert(
                "registry-owned-key".into(),
                observer.document.clone().unwrap(),
                vec![observer],
            )
        });
    }
    let record_bytes = allocation_bytes(std::alloc::Layout::new::<Document>())
        + capacity_bytes::<RetainedGeometryObserver>(1)
        + capacity_bytes::<u8>("registry-owned-key".len());
    let live = || {
        let usage = scope.usage();
        (usage.payload_bytes + usage.tracking_bytes) as usize
    };
    assert_eq!(live(), 2 * record_bytes);
    drop(first);
    gpu_usage();
    prune(&mut registry);
    assert_eq!(live(), record_bytes);
    drop(second);
    gpu_usage();
    prune(&mut registry);
    assert_eq!(live(), 0);
    assert!(registry.head.is_none());
}

#[test]
fn document_payload_aggregates_distinct_scenes_deduplicates_clones_and_releases() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "cpu-document-aggregation-proof".into();
    let first = RetainedScene::from_workspace(&state, 960, 720);
    let observer = first.geometry_observer();
    let payload = first.heap_payload_bytes().unwrap();
    let initial = usage(&observer);
    assert!(initial > payload, "observer storage is included");
    let clone = first.clone();
    assert_eq!(usage(&observer), initial);
    let second = RetainedScene::from_workspace(&state, 960, 720);
    assert!(usage(&observer) >= initial + payload);
    let both = usage(&observer);
    check_limit(&observer, both).unwrap();
    assert!(check_limit(&observer, both - 1).is_err());
    state.scene.scene_id = "cpu-document-isolation-proof".into();
    let other = RetainedScene::from_workspace(&state, 960, 720);
    assert_eq!(
        usage(&observer),
        both,
        "another document is not charged here"
    );
    drop(first);
    assert_eq!(
        usage(&observer),
        both,
        "external clone retains complete payload"
    );
    drop(clone);
    assert!(usage(&observer) < both);
    assert!(usage(&observer) >= second.heap_payload_bytes().unwrap());
    drop(second);
    assert!(usage(&observer) >= observer.heap_bytes_excluding([]));
    assert!(!observer.is_live());
    drop(observer);
    assert!(
        !gpu_usage()
            .iter()
            .any(|entry| entry.scene_id == "cpu-document-aggregation-proof")
    );
    assert!(usage(&other.geometry_observer()) > 0);
}
