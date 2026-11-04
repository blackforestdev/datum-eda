//! Real canonical source drives configured Gate choice and immutable Proposal review.
use super::*;
use crate::schematic_authoring::gate_placement::GatePlacementDraft;
use eda_engine::{
    api::native_write::{
        commit_prepared,
        context::WriteProvenance,
        genesis::{GenesisSpec, bootstrap_native_project},
        library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_delete_proposal::{self as proposal, NativeProposalRequest},
        schematic_sheets::build_create_schematic_sheet,
    },
    substrate::{CommitSource, DesignModel, ProjectResolver},
};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "Gate picker integration")
}
fn action(
    model: &mut DesignModel,
    root: &std::path::Path,
    action: Action,
) -> NativeProposalResponse {
    let request = NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: eda_engine::revision::transaction_tip(&model.journal),
        action,
    };
    proposal::execute(model, &request).unwrap()
}
fn setup() -> (FixtureRoot, DesignModel, Uuid) {
    let root =
        FixtureRoot(std::env::temp_dir().join(format!("datum-gate-picker-{}", Uuid::new_v4())));
    let ids = bootstrap_native_project(
        &root.0,
        GenesisSpec {
            project_name: "Gate picker".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root.0).resolve().unwrap();
    let sheet = Uuid::new_v4();
    let write=build_create_schematic_sheet(&model,provenance(),ids.schematic_uuid,sheet,"sheets/main.json",serde_json::json!({"schema_version":1,"uuid":sheet,"name":"Main","symbols":{},"wires":{},"labels":{},"ports":{},"junctions":{},"buses":{},"bus_entries":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap();
    commit_prepared(&mut model, &root.0, write).unwrap();
    let baseline = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/library/native_authored_baseline_v1/pool");
    let mut specs = vec![];
    let mut entity_raw = serde_json::Value::Null;
    for kind in [
        "units",
        "symbols",
        "entities",
        "packages",
        "padstacks",
        "footprints",
        "parts",
        "pin_pad_maps",
    ] {
        for entry in std::fs::read_dir(baseline.join(kind)).unwrap() {
            let object: serde_json::Value =
                serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
            if kind == "entities" {
                entity_raw = object.clone();
            }
            specs.push(Spec::Create {
                target: Target::new(
                    "pool",
                    kind,
                    serde_json::from_value(object["uuid"].clone()).unwrap(),
                ),
                object,
            });
        }
    }
    let entity: Uuid = serde_json::from_value(entity_raw["uuid"].clone()).unwrap();
    let first_gate: Uuid = entity_raw["gates"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let symbol: Uuid =
        serde_json::from_value(entity_raw["gates"][first_gate.to_string()]["symbol"].clone())
            .unwrap();
    let part = Uuid::new_v4();
    specs.push(Spec::Create {
        target: Target::new("pool", "parts", part),
        object: library::pool_part_payload(
            part,
            entity,
            "50000000-0000-4000-8000-000000000001".parse().unwrap(),
            "Gate test Part",
            "Datum",
            "OPAMP",
            "Explicit Gate test",
            "",
            "Active",
        ),
    });
    let write =
        library::build_pool_library_write(&model, provenance(), Some("pool"), specs).unwrap();
    commit_prepared(&mut model, &root.0, write).unwrap();
    let mut first = None;
    for (reference, x) in [("U_FIRST", 10_000_000), ("U_OTHER", 70_000_000)] {
        let write=(find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root.0},provenance(),serde_json::json!({"sheet":sheet,"library_symbol":symbol,"reference":reference,"value":"OPAMP","x_nm":x,"y_nm":20_000_000})).unwrap();
        first = first.or(write.primary_object_id);
        commit_prepared(&mut model, &root.0, write).unwrap();
    }
    let gate = Uuid::new_v4();
    let mut second = entity_raw["gates"][first_gate.to_string()].clone();
    second["uuid"] = serde_json::json!(gate);
    second["name"] = serde_json::json!("Second gate");
    entity_raw["gates"][gate.to_string()] = second;
    let write = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Set {
            target: Target::new("pool", "entities", entity),
            object: entity_raw,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root.0, write).unwrap();
    let first = first.unwrap();
    let draft = action(
        &mut model,
        &root.0,
        Action::CreateAssignment {
            proposal_id: Uuid::new_v4(),
            symbol: first,
            instance_path: vec![],
            part,
            gate: first_gate,
        },
    );
    action(
        &mut model,
        &root.0,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    (root, model, first)
}
fn selected(model: &DesignModel, root: &std::path::Path, id: Uuid) -> crate::ReviewWorkspaceState {
    let mut state = workspace(model, root);
    let authority = crate::selection_resolution::NativeSelectionResolution::capture(model).unwrap();
    state.selection = SelectionTarget::Native(Box::new(
        crate::selection_view::build_selection_view(
            model,
            &authority,
            SelectionSubject::Object(Id {
                class: Class::Symbol,
                id,
                instance_path: vec![],
            }),
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap(),
    ));
    state.supervision.journal.accepted_transaction_tip =
        eda_engine::revision::transaction_tip(&model.journal).map(|id| id.to_string());
    state.backing = Some(crate::WorkspaceBacking {
        request: crate::LiveReviewRequest {
            project_root: root.into(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        board_path: root.join("board/board.json"),
        native_write_verbs: Default::default(),
    });
    state
}
#[test]
fn gate_placement_picker_freezes_exact_ids_reference_and_source_through_cancelled_review() {
    let (fixture, mut model, first) = setup();
    let mut state = selected(&model, &fixture.0, first);
    let mut draft = GatePlacementDraft::capture(&state).unwrap();
    assert_eq!(draft.options.choices.len(), 1);
    assert!(draft.selected.is_none() && draft.material.is_none());
    assert!(
        draft.review.pending.is_none(),
        "opening the form creates no Proposal"
    );
    let source = state.schematic_authoring.source.clone();
    let catalogue = state.schematic_authoring.choices.clone();
    let journal = model.journal.len();
    draft.choose(&state, 0).unwrap();
    let e = &mut state.schematic_authoring;
    e.gate_draft = Some(draft.clone());
    e.chosen = Some(0);
    e.picker = true;
    e.reference.clear();
    e.value.clear();
    assert!(
        !e.select(0),
        "ordinary auto-reference selection cannot replace explicit Gate input"
    );
    assert!(e.reference.is_empty());
    assert!(e.start_gate_preview().unwrap().is_err());
    assert!(
        e.picker && e.reference.is_empty(),
        "Preview cannot invent the required Reference"
    );
    assert!(
        !e.transform_placement(PlacementTransform::Rotate),
        "picker text owns R, not pose"
    );
    assert_eq!(
        e.choices, catalogue,
        "fixed-owner choice must not remove global library bodies"
    );
    let at = PointNm {
        x: 50_000_000,
        y: 40_000_000,
    };
    assert!(
        draft.clone().stage(&state, at).is_err(),
        "Reference must be supplied explicitly"
    );
    let e = &mut state.schematic_authoring;
    e.reference_editor
        .replace(&mut e.reference, "U_SECOND")
        .unwrap();
    e.value_editor
        .replace(&mut e.value, "Literal value")
        .unwrap();
    e.chosen = Some(usize::MAX);
    assert!(
        e.start_gate_preview().unwrap().is_err(),
        "stale picker index cannot enter preview"
    );
    e.chosen = Some(0);
    e.start_gate_preview().unwrap().unwrap();
    assert!(!e.picker && e.preview_dirty);
    assert!(e.transform_placement(PlacementTransform::Rotate));
    assert!(e.transform_placement(PlacementTransform::FlipX));
    assert!(e.transform_placement(PlacementTransform::FlipX));
    assert!(e.transform_placement(PlacementTransform::FlipY));
    assert_eq!((e.rotation_deg, e.mirrored), (90, true));
    assert!(
        e.gate_draft.as_ref().unwrap().review.pending.is_none(),
        "typing and R/X/Y only change the local preview"
    );
    e.position = Some(at);
    let posed = draft
        .placed("U_SECOND", "Literal value", at, 90, true)
        .unwrap();
    let preview = state
        .schematic_authoring
        .preview_scene(state.schematic_scene.as_ref().unwrap(), &fixture.0)
        .unwrap();
    assert!(
        preview
            .board_graphics
            .iter()
            .any(|g| g.source_object_uuid == posed.uuid.to_string())
    );
    for pin in &posed.pins {
        assert!(
            preview
                .board_graphics
                .iter()
                .any(|g| g.source_object_uuid == pin.uuid.to_string())
        );
    }
    let mut review = draft.stage(&state, at).unwrap();
    assert_eq!(
        review.gate_placement.as_ref().unwrap().summary.symbol,
        posed.uuid
    );
    assert_eq!(state.schematic_authoring.source, source);
    assert_eq!(model.journal.len(), journal);
    let receipt = action(&mut model, &fixture.0, review.pending.clone().unwrap());
    review.observe(receipt.clone()).unwrap();
    assert!(review.ready());
    assert!(
        !review
            .preview_scene()
            .board_graphics
            .iter()
            .any(|g| g.source_object_uuid == posed.uuid.to_string()),
        "uncommitted Gate stays in proposed overlay, never authored hit geometry"
    );
    assert!(
        review
            .gate_placement
            .as_ref()
            .unwrap()
            .preview
            .board_graphics
            .iter()
            .any(|g| g.source_object_uuid == posed.uuid.to_string())
    );
    assert!(review.summary().contains("U_SECOND"));
    let mut wrong = receipt.clone();
    wrong.gate_placement.as_mut().unwrap().reference = "U_WRONG".into();
    assert!(review.observe(wrong).is_err());
    for status in [
        ProposalStatus::Draft,
        ProposalStatus::Accepted,
        ProposalStatus::Applied,
    ] {
        let mut missing = receipt.clone();
        missing.status = status;
        missing.gate_placement = None;
        assert!(review.observe(missing).is_err());
    }
    review.stage(false).unwrap();
    let mut cancelled = action(&mut model, &fixture.0, review.pending.clone().unwrap());
    cancelled.gate_placement = None; // A confirmed stale-source terminal cancellation retains local evidence.
    review.observe(cancelled).unwrap();
    assert!(review.finished() && review.gate_placement.is_some());
    assert_eq!(
        SchematicAuthoring::from_model(&model).unwrap().source,
        source
    );
    let original = state.schematic_authoring.base.clone();
    state.schematic_authoring.cancel();
    assert!(state.schematic_authoring.gate_draft.is_none());
    assert_eq!(state.schematic_authoring.source, source);
    assert!(original.is_none());
}
#[test]
fn gate_placement_stale_source_refuses_before_materialization_or_proposal() {
    let (fixture, model, first) = setup();
    let mut state = selected(&model, &fixture.0, first);
    let mut draft = GatePlacementDraft::capture(&state).unwrap();
    state.scene.source_revision = "stale".into();
    assert!(draft.choose(&state, 0).is_err());
    assert!(draft.clone().stage(&state, PointNm { x: 0, y: 0 }).is_err());
    assert!(draft.review.pending.is_none() && draft.material.is_none());
}
