//! Replay actual materializer output; generate/freeze it with s5a_fixture first.
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection,
    LiveReviewRequest, SelectionSubject,
    selection_geometry_catalog::{QualificationGeometry, qualification_catalog},
    selection_resolution::NativeSelectionResolution,
    selection_scene::SceneSelectionBindings,
    selection_view::{InspectionValue, build_selection_view},
};
use eda_engine::substrate::{ProjectResolver, transaction_journal_path};
use serde_json::Value;
use std::{collections::BTreeSet, path::PathBuf};
fn identity(class: Class, value: &Value) -> Id {
    Id {
        class,
        id: value.as_str().unwrap().parse().unwrap(),
        instance_path: vec![],
    }
}
#[test]
#[ignore = "requires frozen library-parents native recipe and pinned EDA_CLI_BIN; no absent-fixture pass"]
fn actual_placed_parents_pins_inspection_and_unresolved_pad_boundary() {
    let root = PathBuf::from(
        std::env::var_os("S5A_NATIVE_PARENT_FIXTURE")
            .expect("explicit frozen native parent recipe"),
    );
    assert!(
        std::env::var_os("EDA_CLI_BIN").is_some(),
        "pin CLI; no Cargo subprocess fallback"
    );
    let recipe: Value =
        serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let mut before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = transaction_journal_path(&root);
    before.push((journal.clone(), std::fs::read(journal).unwrap()));
    assert_eq!(
        serde_json::to_value(&model.model_revision).unwrap(),
        recipe["model_revision"]
    );
    let state = datum_gui_protocol::load_board_editor_workspace_state(&LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap();
    let schematic = state.schematic_scene.as_ref().unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let mut parents = BTreeSet::new();
    let catalog = qualification_catalog(&model, &authority, &state.scene);
    let schematic_catalog = qualification_catalog(&model, &authority, schematic);
    let bindings = SceneSelectionBindings::capture(schematic, &authority);
    for placement in recipe["placements"].as_array().unwrap() {
        let symbol = identity(Class::Symbol, &placement["symbol"]);
        let package = identity(Class::Footprint, &placement["package"]);
        parents.extend([symbol.clone(), package.clone()]);
        let projected = state
            .scene
            .components
            .iter()
            .find(|c| c.source_object_uuid == package.id.to_string())
            .unwrap();
        assert_eq!(
            projected.locked,
            placement["locked"].as_bool().unwrap(),
            "actual authored Package lock must reach persistent painter independently of selection"
        );
        let symbol_view = build_selection_view(
            &model,
            &authority,
            SelectionSubject::Object(symbol.clone()),
            &state.scene,
            Some(schematic),
            |_| true,
        )
        .unwrap();
        assert_eq!(symbol_view.members, BTreeSet::from([symbol.clone()]));
        assert!(!symbol_view.schematic_keys.is_empty());
        assert!(symbol_view.board_keys.is_empty());
        let QualificationGeometry::Anchors(anchors) = schematic_catalog[&symbol].as_ref().unwrap()
        else {
            panic!("actual instance Pin anchors")
        };
        let expected: BTreeSet<_> = placement["pins"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                (
                    p["position"]["x"].as_i64().unwrap(),
                    p["position"]["y"].as_i64().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            anchors.iter().map(|p| (p.x, p.y)).collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(anchors.len(), 5);
        for pin in placement["pins"].as_array().unwrap() {
            let pin_id = identity(Class::Pin, &pin["uuid"]);
            assert!(
                bindings
                    .iter()
                    .any(|(_, b)| b.projected == pin_id && b.acquired == symbol),
                "ordinary Pin hit acquires its real Symbol"
            );
            let view = build_selection_view(
                &model,
                &authority,
                SelectionSubject::Object(pin_id.clone()),
                &state.scene,
                Some(schematic),
                |_| true,
            )
            .unwrap();
            assert_eq!(view.members, BTreeSet::from([pin_id]));
            assert_eq!(view.related, BTreeSet::from([symbol.clone()]));
            assert_eq!(
                view.field("pin_number"),
                InspectionValue::Common(pin["number"].clone())
            );
            assert!(!view.schematic_keys.is_empty());
            assert!(!view.related_schematic_keys.is_empty());
        }
        // Source contains eight actual component pad payloads. An unresolved
        // bridge must not silently replace their majority with one origin.
        assert!(
            catalog[&package].is_err(),
            "component-Pad authority gap is an explicit blocker, not singleton placement qualification"
        );
    }
    let subject = SelectionSubject::Compound(
        CompoundSelection::new(parents.clone(), parents.first().cloned()).unwrap(),
    );
    let mut view = build_selection_view(
        &model,
        &authority,
        subject,
        &state.scene,
        Some(schematic),
        |_| true,
    )
    .unwrap();
    assert_eq!(view.members, parents);
    assert_eq!(view.locked.len(), 1);
    assert!(!view.board_keys.is_empty() && !view.schematic_keys.is_empty());
    view.scope = Some(Class::Footprint);
    assert_eq!(view.field("locked"), InspectionValue::Mixed);
    assert_eq!(view.field("reference"), InspectionValue::Mixed);
    assert_eq!(
        view.field("value"),
        InspectionValue::Common("native fixture".into())
    );
    view.scope = Some(Class::Symbol);
    assert_eq!(view.field("placement_rotation"), InspectionValue::Mixed);
    assert_eq!(view.field("reference"), InspectionValue::Mixed);
    assert_eq!(
        view.field("value"),
        InspectionValue::Common("native fixture".into())
    );
    assert!(matches!(
        view.field("locked"),
        InspectionValue::Unavailable(_)
    ));
    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(&path).unwrap(),
            bytes,
            "source/journal read-only: {}",
            path.display()
        );
    }
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        model.model_revision
    );
}

#[test]
#[ignore = "requires existing frozen parents recipe and pinned EDA_CLI_BIN"]
fn captured_pad_ownership_preserves_unresolved_parents_and_rejects_forged_nil_scene_parent() {
    let root =
        PathBuf::from(std::env::var_os("S5A_NATIVE_PARENT_FIXTURE").expect("frozen parent recipe"));
    assert!(std::env::var_os("EDA_CLI_BIN").is_some());
    let recipe: Value =
        serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        serde_json::to_value(&model.model_revision).unwrap(),
        recipe["model_revision"]
    );
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|source| (source.path.clone(), std::fs::read(&source.path).unwrap()))
        .chain(std::iter::once((
            transaction_journal_path(&root),
            std::fs::read(transaction_journal_path(&root)).unwrap(),
        )))
        .collect();
    let state = datum_gui_protocol::load_board_editor_workspace_state(&LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let catalog = qualification_catalog(&model, &authority, &state.scene);
    for placement in recipe["placements"].as_array().unwrap() {
        let package = identity(Class::Footprint, &placement["package"]);
        assert!(
            catalog[&package].is_err(),
            "existing unresolved geometry boundary remains explicit"
        );
        let view = build_selection_view(
            &model,
            &authority,
            SelectionSubject::Object(package.clone()),
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap();
        assert!(view.members.contains(&package));
    }
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    let mixed_root = PathBuf::from(
        std::env::var_os("S5A_NATIVE_MIXED_PAD_FIXTURE")
            .expect("retained canonical mixed Pad fixture"),
    );
    let model = ProjectResolver::new(&mixed_root).resolve().unwrap();
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|source| (source.path.clone(), std::fs::read(&source.path).unwrap()))
        .chain(std::iter::once((
            transaction_journal_path(&mixed_root),
            std::fs::read(transaction_journal_path(&mixed_root)).unwrap(),
        )))
        .collect();
    let state = datum_gui_protocol::load_board_editor_workspace_state(&LiveReviewRequest {
        project_root: mixed_root,
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let mut checked = 0;
    let mut standalone_checked = 0;
    let bindings = SceneSelectionBindings::capture(&state.scene, &authority);
    for pad in &state.scene.pads {
        let binding = bindings
            .get(&pad.object_id)
            .expect("actual mixed source Pad projection");
        assert_eq!(binding.projected.class, Class::Pad);
        if authority.is_standalone_pad(&binding.projected) {
            assert_eq!(binding.acquired, binding.projected);
            standalone_checked += 1;
            continue;
        }
        assert_eq!(binding.acquired.class, Class::Footprint);
        assert_eq!(binding.acquired.id.to_string(), pad.component_uuid);
        let mut forged = state.scene.clone();
        forged
            .pads
            .iter_mut()
            .find(|p| p.object_id == pad.object_id)
            .unwrap()
            .component_uuid = "00000000-0000-0000-0000-000000000000".into();
        assert!(
            SceneSelectionBindings::capture(&forged, &authority)
                .get(&pad.object_id)
                .is_none()
        );
        assert!(
            datum_gui_protocol::selection_scene::authored_candidate(
                &forged,
                &pad.object_id,
                &authority
            )
            .is_none()
        );
        checked += 1;
    }
    assert!(checked > 0, "real source-owned Pad projection required");
    assert!(
        standalone_checked > 0,
        "real source-standalone Pad projection required"
    );
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
