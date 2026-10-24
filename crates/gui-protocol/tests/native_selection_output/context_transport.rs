//! Complete query values and bounded transport share one source-owned state.
use super::*;
use datum_gui_protocol::selection_view::{InspectionDetailKind, InspectionState, SelectionView};

#[test]
fn hidden_projection_does_not_prune_members_and_uncapped_context_is_not_authority() {
    let (f, scene, ids) = setup();
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = authority.acquire_run(ids[0].clone(), None).unwrap();
    let view = build_selection_view(&f.model, &authority, subject.clone(), &scene, None, |_| {
        false
    })
    .unwrap();
    assert_eq!(view.members.len(), 2);
    assert_eq!(view.hidden.len(), 2);
    assert_eq!(view.unprojected.len(), 0);
    assert_eq!(view.board_keys.len(), 2);
    let mut dense = view;
    for n in 3..=256 {
        dense.members.insert(Id {
            class: Class::Track,
            id: id(n),
            instance_path: vec![],
        });
    }
    let at_cap =
        DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(dense.clone())));
    let at_cap = at_cap.details.unwrap();
    assert_eq!(at_cap.total, 256);
    assert_eq!(at_cap.members.unwrap().len(), 256);
    assert!(!at_cap.truncated);
    dense.members.insert(Id {
        class: Class::Track,
        id: id(257),
        instance_path: vec![],
    });
    let original = dense.members.clone();
    let complete_evidence = serde_json::json!(original);
    dense
        .inspection
        .iter_mut()
        .find(|s| s.scope.is_none())
        .unwrap()
        .fields
        .insert(
            "non_electrical_members".into(),
            InspectionValue::Common(complete_evidence.clone()),
        );
    let retained = dense.clone();
    let context = DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(dense)));
    assert_eq!(context.kind, "run");
    let details = context.details.unwrap();
    assert_eq!(details.total, 257);
    assert!(details.members.is_none());
    assert!(details.truncated);
    assert!(
        details
            .omitted_inspection_fields
            .contains(&"non_electrical_members".into())
    );
    assert_eq!(
        details.omitted_inspection_field_states["non_electrical_members"],
        InspectionState::Common
    );
    assert!(!details.inspection.contains_key("non_electrical_members"));
    assert!(
        !serde_json::to_string(&details)
            .unwrap()
            .contains(&id(257).to_string())
    );
    assert_eq!(
        retained.field("non_electrical_members"),
        InspectionValue::Common(complete_evidence)
    );
    assert_eq!(details.subject, Some(subject));
    assert_eq!(original.len(), 257);
    Fixture::assert_bytes(&before);
}

fn complete_track_scope(count: u128) -> (Fixture, SelectionView) {
    let mut f = Fixture::new("context-complete-track-scope");
    let mut identities = Vec::new();
    let mut operations = Vec::new();
    for n in 1..=count {
        let x = n as i64 * 100_000;
        let mut track = f.line(id(n), Point::new(x, 0), Point::new(x + 10_000, 0));
        track.width = if n % 2 == 0 { 100_000 } else { 200_000 };
        identities.push(Id {
            class: Class::Track,
            id: track.uuid,
            instance_path: vec![],
        });
        operations.push(Operation::SetBoardTrack {
            track_id: track.uuid,
            track: serde_json::to_value(track).unwrap(),
        });
    }
    f.commit(operations);
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = SelectionSubject::Compound(CompoundSelection::new(identities, None).unwrap());
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.source_revision = f.model.model_revision.0.clone();
    scene.board_uuid = f.board.to_string();
    scene.components.clear();
    scene.pads.clear();
    scene.tracks.clear();
    scene.vias.clear();
    scene.zones.clear();
    scene.board_graphics.clear();
    scene.board_texts.clear();
    scene.outline.clear();
    let mut view =
        build_selection_view(&f.model, &authority, subject, &scene, None, |_| true).unwrap();
    view.scope = Some(Class::Track);
    (f, view)
}

#[test]
fn current_256_and_257_transport_retains_ordinary_reasons_and_all_value_states() {
    for count in [256, 257] {
        let (f, view) = complete_track_scope(count);
        let before = f.source_bytes();
        let ordinary = InspectionValue::Unavailable(
            "Property is not typed or available for every member in this complete scope".into(),
        );
        assert_eq!(view.field("section_midpoint_nm"), ordinary);
        assert_eq!(view.field("copper_width_nm"), InspectionValue::Mixed);
        assert_eq!(
            view.field("board_layer"),
            InspectionValue::Common(serde_json::json!(1))
        );
        let scope = view
            .inspection
            .iter()
            .find(|s| s.scope == view.scope)
            .unwrap();
        assert_eq!(
            scope.field_details["section_midpoint_nm"],
            InspectionDetailKind::Ordinary
        );
        assert_eq!(
            scope.field_details["net_coverage"],
            InspectionDetailKind::Members
        );
        assert_eq!(
            scope.field_details["combined_bounds_nm"],
            InspectionDetailKind::Members
        );
        let full: std::collections::BTreeMap<_, _> = view
            .fields()
            .into_iter()
            .map(|name| (name.to_string(), view.field(name)))
            .collect();
        let context =
            DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(view.clone())))
                .details
                .unwrap();
        // The new Inspector context refers to the same governed detail owner;
        // it must not re-export a dense field through a second context array.
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.schematic_documents.project = Some(f.model.project.project_id);
        state.selection = SelectionTarget::Native(Box::new(view.clone()));
        let shared = DatumSelectionContext::from_workspace(&state);
        assert_eq!(shared.details.as_ref(), Some(&context));
        let inspector = shared.inspector.as_ref().unwrap();
        for field in inspector
            .sections
            .iter()
            .flat_map(|section| &section.fields)
        {
            if let Some(name) = &field.native_field {
                assert!(field.value.is_none());
                assert_eq!(
                    field.native_value_state,
                    view.field_ref(name).map(InspectionValue::state)
                );
                assert_eq!(
                    field.detail_omitted,
                    context.omitted_inspection_fields.contains(name)
                );
            }
        }
        if count > 256 {
            assert!(
                !serde_json::to_string(&shared)
                    .unwrap()
                    .contains(&id(257).to_string())
            );
        }
        state.inspector_context.owner =
            datum_gui_protocol::inspector_context::InspectorOwner::Navigator;
        let navigation = DatumSelectionContext::from_workspace(&state);
        assert_eq!(
            navigation.details, shared.details,
            "Navigator inspection preserves the global subject and transport cap"
        );
        assert!(
            navigation
                .inspector
                .unwrap()
                .sections
                .iter()
                .flat_map(|section| &section.fields)
                .all(|field| field.native_field.is_none())
        );
        assert_eq!(context.total, count as usize);
        assert_eq!(context.truncated, count > 256);
        assert_eq!(context.inspection["section_midpoint_nm"], ordinary);
        assert_eq!(
            context.inspection["copper_width_nm"],
            InspectionValue::Mixed
        );
        assert_eq!(context.inspection["board_layer"], full["board_layer"]);
        for name in [
            "resolved_net_coverage",
            "non_electrical_members",
            "projected_layer_coverage",
        ] {
            assert_eq!(
                context.inspection[name], full[name],
                "empty coverage has no member detail to omit"
            );
        }
        if count == 256 {
            assert_eq!(context.members.as_ref().unwrap().len(), 256);
            assert_eq!(context.inspection, full);
            assert!(context.omitted_inspection_fields.is_empty());
            assert!(context.omitted_inspection_field_states.is_empty());
        } else {
            assert!(context.members.is_none());
            assert_eq!(
                context.omitted_inspection_fields.len(),
                context.omitted_inspection_field_states.len()
            );
            for name in &context.omitted_inspection_fields {
                assert!(!context.inspection.contains_key(name));
                assert_eq!(
                    context.omitted_inspection_field_states[name],
                    full[name].state()
                );
            }
            for name in ["net_coverage", "combined_bounds_nm", "layer_coverage"] {
                assert_eq!(
                    context.omitted_inspection_field_states[name],
                    InspectionState::Unavailable
                );
                assert!(
                    matches!(&full[name], InspectionValue::Unavailable(reason) if reason.contains(&id(257).to_string()))
                );
            }
            assert!(
                !serde_json::to_string(&context)
                    .unwrap()
                    .contains(&id(257).to_string())
            );
        }
        for (name, value) in &full {
            assert_eq!(view.field(name), *value);
        }
        let roundtrip: Vec<datum_gui_protocol::selection_view::ScopeInspection> =
            serde_json::from_value(serde_json::to_value(&view.inspection).unwrap()).unwrap();
        assert_eq!(view.inspection, roundtrip);
        Fixture::assert_bytes(&before);
    }
}

#[test]
fn legacy_unknown_reason_provenance_is_conservative_and_full_queries_remain_exact() {
    let (f, view) = complete_track_scope(257);
    let before = f.source_bytes();
    let original = view.field("section_midpoint_nm");
    let mut legacy_scopes = serde_json::to_value(&view.inspection).unwrap();
    for scope in legacy_scopes.as_array_mut().unwrap() {
        scope.as_object_mut().unwrap().remove("field_details");
    }
    let mut legacy = view.clone();
    legacy.inspection = serde_json::from_value(legacy_scopes).unwrap();
    assert_eq!(legacy.field("section_midpoint_nm"), original);
    let context =
        DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(legacy.clone())))
            .details
            .unwrap();
    assert!(!context.inspection.contains_key("section_midpoint_nm"));
    assert_eq!(
        context.omitted_inspection_field_states["section_midpoint_nm"],
        InspectionState::Unavailable
    );
    assert_eq!(
        context.inspection["copper_width_nm"],
        InspectionValue::Mixed
    );
    assert_eq!(legacy.field("section_midpoint_nm"), original);
    let mut prior_context = serde_json::to_value(&context).unwrap();
    prior_context
        .as_object_mut()
        .unwrap()
        .remove("omitted_inspection_field_states");
    let prior_context: datum_gui_protocol::NativeSelectionContext =
        serde_json::from_value(prior_context).unwrap();
    assert!(prior_context.omitted_inspection_field_states.is_empty());
    assert_eq!(prior_context.total, 257);
    Fixture::assert_bytes(&before);
}
