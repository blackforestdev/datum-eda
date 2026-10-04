//! Frozen actual semantic sources exercise the production adapter and projection.
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, LiveReviewRequest,
    SelectionSubject,
    selection_resolution::{NativeSelectionResolution, SelectionAuthority},
    selection_scene::{authored_candidate, projection_candidate},
    selection_view::build_selection_view,
};
use eda_engine::substrate::{ProjectResolver, transaction_journal_path};
use serde_json::Value;
use std::{collections::BTreeSet, path::PathBuf};
fn identity(o: &Value) -> Option<Id> {
    let class = match o["class"].as_str().unwrap() {
        "wires" => Class::Wire,
        "labels" => Class::Label,
        "junctions" => Class::Junction,
        "ports" => Class::Port,
        "buses" => Class::BusSection,
        "bus_entries" => return None,
        _ => panic!("declared fixture class"),
    };
    Some(Id {
        class,
        id: o["source_id"].as_str().unwrap().parse().unwrap(),
        instance_path: o["instance_path"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().parse().unwrap())
            .collect(),
    })
}
fn members(report: &Value, key: &str) -> BTreeSet<Id> {
    report[key]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(identity)
        .collect()
}
#[test]
#[ignore = "requires frozen semantic-scope native recipe and pinned EDA_CLI_BIN; no absent-fixture pass"]
fn native_semantic_recipe_keeps_complete_identity_and_visible_root_projection() {
    let fixture = PathBuf::from(
        std::env::var_os("S5A_NATIVE_SEMANTIC_FIXTURE").expect("explicit frozen semantic recipe"),
    );
    assert!(
        std::env::var_os("EDA_CLI_BIN").is_some(),
        "pin CLI; no Cargo subprocess fallback"
    );
    let recipe: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("s5a-fixture.json")).unwrap()).unwrap();
    let report = &recipe["semantic_variant"];
    let root = fixture.join("variants/semantic-scope");
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let mut before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = transaction_journal_path(&root);
    before.push((journal.clone(), std::fs::read(journal).unwrap()));
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
    assert_eq!(
        schematic.board_uuid, "20000000-0000-4000-8000-000000000005",
        "native root projection must not prefer instantiated definition template over actual top-level Sheet"
    );
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let expected_run = members(report, "expected_bus_run_members");
    let origin = expected_run.iter().min_by_key(|i| i.id).unwrap().clone();
    let object = schematic
        .board_graphics
        .iter()
        .find(|g| {
            projection_candidate(schematic, &g.object_id, &authority).as_ref() == Some(&origin)
        })
        .unwrap();
    assert_eq!(
        authored_candidate(schematic, &object.object_id, &authority),
        Some(origin.clone())
    );
    let run = authority.acquire_run(origin, None).unwrap();
    assert_eq!(authority.members(&run).unwrap(), expected_run);
    let resolved = authority.resolve(&run).unwrap();
    assert_eq!(resolved.bus_run.unwrap().contacts.len(), 1);
    let SelectionSubject::Run(run_id) = &run else {
        unreachable!()
    };
    let bus = authority.widen_run(run_id).unwrap();
    assert_eq!(
        bus,
        SelectionSubject::Bus(report["primary_bus"].as_str().unwrap().parse().unwrap())
    );
    let expected_bus = members(report, "expected_bus_members");
    assert_eq!(expected_bus.len(), 6);
    assert_eq!(authority.members(&bus).unwrap(), expected_bus);
    let view = build_selection_view(
        &model,
        &authority,
        bus.clone(),
        &state.scene,
        Some(schematic),
        |_| true,
    )
    .unwrap();
    assert_eq!(view.members, expected_bus);
    assert_eq!(view.unprojected.len(), 3);
    assert!(!view.schematic_keys.is_empty());
    assert!(view.related.iter().any(|id| id.class == Class::Wire));
    let hidden = build_selection_view(
        &model,
        &authority,
        bus,
        &state.scene,
        Some(schematic),
        |_| false,
    )
    .unwrap();
    assert_eq!(hidden.members, view.members);
    assert_eq!(hidden.hidden.len(), 3);
    assert_eq!(
        hidden.schematic_keys, view.schematic_keys,
        "projection keys retain complete identity inventory; canvas visibility is applied by the renderer"
    );
    assert_eq!(hidden.unprojected.len(), 3);
    let net = SelectionSubject::GlobalNet(report["logical_net"].as_str().unwrap().parse().unwrap());
    let net_view = build_selection_view(
        &model,
        &authority,
        net,
        &state.scene,
        Some(schematic),
        |_| true,
    )
    .unwrap();
    assert_eq!(
        net_view.members,
        members(report, "expected_global_net_members")
    );
    assert_eq!(net_view.members.len(), 21);
    assert_eq!(net_view.unprojected.len(), 14);
    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(&path).unwrap(),
            bytes,
            "source/journal unchanged: {}",
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
