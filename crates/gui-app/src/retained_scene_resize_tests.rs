//! Exercise the same CPU ownership transition used by Runtime surface invalidation.
use super::*;

#[test]
fn resize_keeps_complete_active_charge_while_retiring_obsolete_history() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.unrouted_primitives.clear();
    state
        .scene
        .component_graphics
        .retain(|graphic| !(graphic.closed && graphic.render_role == "component_mechanical"));
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    assert!(scene.can_reuse_for_surface_resize());
    let payload = scene.heap_payload_bytes().unwrap();
    let geometry = scene.geometry_observer();
    assert_eq!(
        payload,
        geometry.heap_bytes_excluding([]),
        "the observer includes command/hit ownership as well as geometry"
    );
    let mut history = RetainedSceneHistory::default();
    history.insert(tests::key(0), tests::scene());
    history.limit_for_active(&scene);
    assert_eq!(history.entries.len(), 1);
    let mut active = Some(scene);
    for _ in 0..1000 {
        history.invalidate_surface_size(&mut active);
        assert!(active.is_some());
        assert_eq!(history.entries.capacity(), 0);
        assert_eq!(history.active_bytes, payload);
        assert_eq!(history.accounted_bytes(), payload);
        assert_eq!(
            history
                .active_geometry
                .as_ref()
                .unwrap()
                .heap_bytes_excluding([&geometry]),
            0,
            "resize must preserve the same geometry owner"
        );
    }
    // Full invalidation still retires the active registration when its owner closes.
    active = None;
    history.invalidate_surface_size(&mut active);
    assert_eq!(history.accounted_bytes(), 0);
    assert!(history.active_geometry.is_none());
}

#[test]
fn dependent_scene_retires_active_and_history_charges_for_reconstruction() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state
        .scene
        .component_graphics
        .push(datum_gui_protocol::ComponentGraphicPrimitive {
            graphic_id: "resize-dependent".into(),
            component_uuid: "resize-component".into(),
            layer_id: None,
            primitive_kind: "polyline".into(),
            render_role: "component_mechanical".into(),
            width_nm: Some(100_000),
            closed: true,
            path: Vec::new(),
            holes: Vec::new(),
        });
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    assert!(!scene.can_reuse_for_surface_resize());
    let mut history = RetainedSceneHistory::default();
    history.insert(tests::key(0), tests::scene());
    history.limit_for_active(&scene);
    let mut active = Some(scene);
    history.invalidate_surface_size(&mut active);
    assert!(active.is_none());
    assert_eq!(history.entries.capacity(), 0);
    assert_eq!(history.active_bytes, 0);
    assert!(history.active_geometry.is_none());
    assert_eq!(history.accounted_bytes(), 0);
}

#[test]
fn companion_schematic_uses_active_and_retiring_cpu_ownership() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let schematic = datum_gui_protocol::load_kicad_schematic_workspace_state(&path).unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.schematic_scene = Some(schematic.scene);
    let scene = RetainedScene::from_workspace_schematic_for_surface(&state, 960, 720, 1.0).unwrap();
    let payload = scene.heap_payload_bytes().unwrap();
    let pinned = scene.clone();
    let pinned_geometry = pinned.geometry_observer().heap_bytes_excluding([]);
    assert_eq!(
        pinned_geometry, payload,
        "external clone retains complete shared metadata"
    );
    let reusable = scene.can_reuse_for_surface_resize();
    let mut owner = RetainedSceneHistory::default();
    owner.limit_for_active(&scene);
    assert_eq!(owner.accounted_bytes(), payload);
    owner.check_render_budget().unwrap();
    let mut active = Some(scene);
    owner.invalidate_surface_size(&mut active);
    assert_eq!(active.is_some(), reusable);
    assert!(owner.accounted_bytes() >= pinned_geometry);
    drop(active.take());
    owner.clear();
    assert_eq!(owner.active_bytes, 0);
    assert!(
        owner.entries.is_empty(),
        "companion has no revision-history cache"
    );
    assert!(owner.accounted_bytes() >= pinned_geometry);
    owner.budget = 1;
    assert!(owner.check_render_budget().is_err());
    drop(pinned);
    owner.clear();
    assert_eq!(owner.accounted_bytes(), 0);
    owner.check_render_budget().unwrap();
}

#[test]
fn document_pressure_retires_only_matching_history_and_keeps_external_pins_charged() {
    let make_scene = |id: &str| {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.scene_id = id.into();
        RetainedScene::from_workspace(&state, 960, 720)
    };
    for pinned in [false, true] {
        let id = if pinned {
            "history-document-pinned"
        } else {
            "history-document-unpinned"
        };
        let mut history = RetainedSceneHistory::default();
        let unrelated = make_scene("history-document-unrelated");
        let mut unrelated_key = tests::key(0);
        unrelated_key.scene_id = "history-document-unrelated".into();
        history.insert(unrelated_key.clone(), unrelated);
        let old = make_scene(id);
        let old_payload = old.heap_payload_bytes().unwrap();
        let external = pinned.then(|| old.clone());
        let mut old_key = tests::key(1);
        old_key.scene_id = id.into();
        history.insert(old_key.clone(), old);
        let active = make_scene(id);
        let observer = active.geometry_observer();
        let before = observer.document_cpu_payload_bytes();
        let limit = before - old_payload / 2;
        history.limit_for_active_document(&active, limit);
        assert!(
            history
                .entries
                .iter()
                .any(|entry| entry.key == unrelated_key)
        );
        assert!(!history.entries.iter().any(|entry| entry.key == old_key));
        if pinned {
            assert!(observer.document_cpu_payload_bytes() > limit);
            drop(external);
        }
        assert!(observer.document_cpu_payload_bytes() <= limit);
        assert_eq!(history.active_bytes, active.heap_payload_bytes().unwrap());
        history.check_render_budget().unwrap();
    }
}

#[test]
fn document_pressure_bypasses_candidate_without_forgiving_external_owner() {
    let mut history = RetainedSceneHistory::default();
    history.insert(tests::key(0), tests::scene());
    let before = history.accounted_bytes();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "history-document-candidate-bypass".into();
    let candidate = RetainedScene::from_workspace(&state, 960, 720);
    let external = candidate.clone();
    let payload = external.heap_payload_bytes().unwrap();
    history.insert_document(tests::key(1), candidate, 1);
    assert_eq!(
        history.entries.len(),
        1,
        "unrelated history survives refusal"
    );
    assert!(history.active_geometry.is_none());
    assert!(history.accounted_bytes() >= before + payload);
    assert_eq!(external.heap_payload_bytes(), Some(payload));
    drop(external);
    history.prune_retired();
    assert_eq!(history.accounted_bytes(), before);
}

#[test]
fn shared_document_history_metadata_is_charged_per_owner_and_released_on_take() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "history-document-metadata-owners".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let observer = scene.geometry_observer();
    let baseline = observer.document_cpu_payload_bytes();
    let mut first = RetainedSceneHistory::default();
    let mut second = RetainedSceneHistory::default();
    let mut key = tests::key(0);
    key.scene_id = state.scene.scene_id.clone();
    key.selection = "selected-object".repeat(128);
    let second_key = key.clone();
    let first_charge = key.heap_bytes().unwrap() + capacity_bytes::<Entry>(1);
    let second_charge = second_key.heap_bytes().unwrap() + capacity_bytes::<Entry>(1);
    first.insert(key.clone(), scene.clone());
    assert_eq!(
        observer.document_cpu_payload_bytes(),
        baseline + first_charge
    );
    // Shared payload fits, but metadata from the other owner leaves no room.
    second.insert_document(second_key.clone(), scene.clone(), baseline + first_charge);
    assert!(second.entries.is_empty());
    assert_eq!(
        observer.document_cpu_payload_bytes(),
        baseline
            + first_charge
            + capacity_bytes::<RetainedGeometryObserver>(second.retired_geometry.capacity())
    );
    second.insert(second_key, scene.clone());
    assert_eq!(
        observer.document_cpu_payload_bytes(),
        baseline + first_charge + second_charge
    );
    let restored = first.take(&key).unwrap();
    assert_eq!(
        observer.document_cpu_payload_bytes(),
        baseline + second_charge
    );
    drop(second);
    assert_eq!(observer.document_cpu_payload_bytes(), baseline);
    assert_eq!(restored, scene);
}

#[test]
fn document_history_container_charges_spare_capacity_and_growth_overlap() {
    for fits_overlap in [false, true] {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.scene_id = format!("history-document-container-{fits_overlap}");
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        let observer = scene.geometry_observer();
        let baseline = observer.document_cpu_payload_bytes();
        let mut history = RetainedSceneHistory::default();
        history.insert(tests::key(0), scene.clone());
        history.insert(tests::key(1), scene.clone());
        let candidate_key = tests::key(2);
        let peak = observer.document_cpu_payload_bytes()
            + candidate_key.heap_bytes().unwrap()
            + capacity_bytes::<Entry>(4);
        history.insert_document(
            candidate_key,
            scene.clone(),
            peak - usize::from(!fits_overlap),
        );
        assert_eq!(history.entries.len(), if fits_overlap { 3 } else { 2 });
        assert_eq!(history.entries.capacity(), if fits_overlap { 4 } else { 2 });
        let expected_keys: usize = history
            .entries
            .iter()
            .map(|entry| entry.key.heap_bytes().unwrap())
            .sum();
        assert_eq!(
            observer.document_cpu_payload_bytes(),
            baseline + expected_keys + capacity_bytes::<Entry>(history.entries.capacity())
        );
        let restored = history.take(&tests::key(2)).unwrap();
        let remaining_keys: usize = history
            .entries
            .iter()
            .map(|entry| entry.key.heap_bytes().unwrap())
            .sum();
        assert_eq!(
            observer.document_cpu_payload_bytes(),
            baseline + remaining_keys + capacity_bytes::<Entry>(history.entries.capacity())
        );
        history.clear();
        assert_eq!(history.entries.capacity(), 0);
        assert!(history.entry_storage.is_none());
        assert_eq!(
            observer.document_cpu_payload_bytes(),
            baseline
                + capacity_bytes::<RetainedGeometryObserver>(history.retired_geometry.capacity())
        );
        drop(history);
        assert_eq!(observer.document_cpu_payload_bytes(), baseline);
        assert_eq!(restored, scene);
    }
}

#[test]
fn history_storage_retains_allocating_document_identity_until_release() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "history-storage-owner-first".into();
    let first = RetainedScene::from_workspace(&state, 960, 720);
    let mut history = RetainedSceneHistory::default();
    history.insert(tests::key(0), first);
    state.scene.scene_id = "history-storage-owner-second".into();
    let second = RetainedScene::from_workspace(&state, 960, 720);
    let observer = second.geometry_observer();
    history.insert(tests::key(1), second);
    drop(history.take(&tests::key(1)).unwrap());
    // The second document caused vector growth. Its scene can disappear while
    // the allocation remains live with another document's entry in it.
    assert!(
        observer.document_cpu_payload_bytes() > capacity_bytes::<Entry>(history.entries.capacity()),
        "metadata-only document retains its owned registry record as well as history storage"
    );
    let reopened = RetainedScene::from_workspace(&state, 960, 720);
    assert!(observer.shares_document_with(&reopened.geometry_observer()));
    let before_clear = observer.document_cpu_payload_bytes();
    let storage = capacity_bytes::<Entry>(history.entries.capacity());
    history.clear();
    assert_eq!(
        observer.document_cpu_payload_bytes(),
        before_clear - storage
    );
}

#[test]
fn retirement_observer_storage_is_charged_through_growth_reuse_and_drop() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "history-retirement-storage".into();
    let mut history = RetainedSceneHistory::default();
    let mut pins = Vec::new();
    for index in 0..5 {
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        pins.push(scene.clone());
        history.insert(tests::key(index), scene);
    }
    let observer = pins[0].geometry_observer();
    let keys: usize = history
        .entries
        .iter()
        .map(|entry| entry.key.heap_bytes().unwrap())
        .sum();
    let payload_registry = observer.document_cpu_payload_bytes()
        - keys
        - capacity_bytes::<Entry>(history.entries.capacity());
    history.clear();
    assert_eq!(history.retired_geometry.len(), 5);
    assert_eq!(
        observer.document_cpu_payload_bytes(),
        payload_registry
            + capacity_bytes::<RetainedGeometryObserver>(history.retired_geometry.capacity())
    );
    let capacity = history.retired_geometry.capacity();
    let pointer = history.retired_geometry.as_ptr();
    pins.truncate(1);
    history.prune_retired();
    assert_eq!(history.retired_geometry.capacity(), capacity);
    assert_eq!(history.retired_geometry.as_ptr(), pointer);
    let before_drop = observer.document_cpu_payload_bytes();
    drop(history);
    assert_eq!(
        observer.document_cpu_payload_bytes(),
        before_drop - capacity_bytes::<RetainedGeometryObserver>(capacity)
    );
    drop(pins);
    assert_eq!(observer.document_cpu_payload_bytes(), 0);
}

#[test]
fn history_entry_limit_is_shared_across_owners_without_limiting_active_references() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.scene_id = "history-document-shared-entry-cap".into();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let observer = scene.geometry_observer();
    let mut first = RetainedSceneHistory::default();
    let mut second = RetainedSceneHistory::default();
    let mut third = RetainedSceneHistory::default();
    for index in 0..3 {
        first.insert(tests::key(index), scene.clone());
    }
    for index in 3..6 {
        second.insert(tests::key(index), scene.clone());
    }
    assert_eq!(observer.document_history_entries(), 6);
    third.insert(tests::key(6), scene.clone());
    assert!(third.entries.is_empty());
    assert_eq!(first.entries.len(), 3);
    assert_eq!(second.entries.len(), 3);
    second.insert(tests::key(7), scene.clone());
    assert_eq!(observer.document_history_entries(), 6);
    assert!(
        second
            .entries
            .iter()
            .any(|entry| entry.key == tests::key(7))
    );
    assert!(
        second
            .entries
            .iter()
            .all(|entry| entry.key != tests::key(3))
    );
    let active = first.take(&tests::key(0)).unwrap();
    assert_eq!(observer.document_history_entries(), 5);
    third.insert(tests::key(6), scene.clone());
    assert_eq!(third.entries.len(), 1);
    assert_eq!(observer.document_history_entries(), 6);
    drop(first);
    drop(second);
    assert_eq!(observer.document_history_entries(), 1);
    drop(third);
    assert_eq!(observer.document_history_entries(), 0);
    assert_eq!(active, scene);
    assert!(observer.document_cpu_payload_bytes() >= scene.heap_payload_bytes().unwrap());
}
