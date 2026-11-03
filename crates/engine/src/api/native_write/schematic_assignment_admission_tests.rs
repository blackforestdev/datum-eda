//! Source graph admission boundaries on canonical authored fixtures.
use super::*;

#[test]
fn native_assignment_excludes_invalid_mapping_and_repeated_source() {
    use super::super::super::schematic_sheets::{
        build_create_schematic_definition, build_create_schematic_sheet_instance,
    };
    use crate::{
        ir::geometry::Point,
        schematic::{SheetDefinition, SheetInstance},
    };
    let (root, mut model, symbol, part) = fixture("assignment-map-occurrence");
    let mut payload = materialized_pool_object(&model, part, "parts").unwrap();
    payload["pad_map"] =
        json!({"80000000-0000-4000-8000-000000000001":{"gate":ids().3,"pin":Uuid::new_v4()}});
    let write = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Set {
            target: Target::new("pool", "parts", part),
            object: payload,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(
        !options(&model, symbol, &[])
            .unwrap()
            .choices
            .iter()
            .any(|c| c.part == part)
    );
    assert!(build(&model, provenance(), symbol, &[], part, ids().3).is_err());
    let sheet = options(&model, symbol, &[]).unwrap().sheet;
    let schematic = id(
        &model
            .materialized_source_shard_value(SourceShardKind::SchematicRoot)
            .unwrap(),
        "uuid",
    )
    .unwrap();
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: sheet,
        name: "Repeated source".into(),
    };
    let write = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/repeated.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut path = vec![];
    for index in 0..2 {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            position: Point::new(index, 0),
            name: format!("I{index}"),
            ports: vec![],
        };
        path = vec![instance.uuid];
        let write = build_create_schematic_sheet_instance(
            &model,
            provenance(),
            schematic,
            instance.uuid,
            serde_json::to_value(instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let len = model.journal.len();
    assert!(options(&model, symbol, &path).is_err());
    assert_eq!(model.journal.len(), len);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_assignment_refuses_physical_or_multiple_symbol_ownership() {
    use super::super::super::board_components::{BoardPackagePlacement, build_place_board_package};
    use crate::{board::PlacedPackage, ir::geometry::Point};
    for physical in [false, true] {
        let (root, mut model, symbol, part) = fixture("assignment-owned");
        let draft = act(&mut model, &root, create(symbol, part));
        let ci = draft.assignment.unwrap().component_instance;
        act(
            &mut model,
            &root,
            Action::AcceptApply {
                proposal_id: draft.proposal_id,
            },
        );
        let other = if physical {
            let placement = BoardPackagePlacement {
                package: PlacedPackage {
                    uuid: Uuid::new_v4(),
                    part,
                    package: "50000000-0000-4000-8000-000000000001".parse().unwrap(),
                    reference: "U1".into(),
                    value: "OPAMP".into(),
                    position: Point::zero(),
                    rotation: 0,
                    layer: 1,
                    locked: false,
                },
                materialized: json!({}),
            };
            let write = build_place_board_package(&model, provenance(), &placement).unwrap();
            let id = placement.package.uuid;
            commit_prepared(&mut model, &root, write).unwrap();
            id
        } else {
            let sheet = options(&model, symbol, &[]).unwrap().sheet;
            let write=(find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},provenance(),json!({"sheet":sheet,"library_symbol":ids().0,"reference":"U2","value":"OPAMP","x_nm":50_000_000,"y_nm":20_000_000})).unwrap();
            let id = write.primary_object_id.unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
            id
        };
        let (_, previous) = component::current(&model, symbol).unwrap().unwrap();
        let mut next = previous.clone();
        next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
        let field = if physical {
            "placed_package_refs"
        } else {
            "placed_symbol_refs"
        };
        next[field].as_array_mut().unwrap().push(
            json!({"object_id":other,"object_revision":model.objects[&other].object_revision.0}),
        );
        let write = PreparedWrite {
            primary_object_id: Some(ci),
            batch: super::super::super::context::build_batch(
                &model,
                provenance(),
                vec![Operation::SetComponentInstance {
                    component_instance_id: ci,
                    previous_component_instance: previous,
                    component_instance: next,
                }],
            )
            .unwrap(),
        };
        commit_prepared(&mut model, &root, write).unwrap();
        let len = model.journal.len();
        assert!(options(&model, symbol, &[]).is_err());
        assert_eq!(model.journal.len(), len);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn native_assignment_stale_draft_rejects_after_source_deletion_without_design_write() {
    let (root, mut model, symbol, part) = fixture("assignment-stale-dismissal");
    let draft = act(&mut model, &root, create(symbol, part));
    let write = (find_native_write_verb("datum.schematic.delete_symbol")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"symbol":symbol,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(!model.objects.contains_key(&symbol));
    let before = model
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .unwrap();
    let revision = model.model_revision.clone();
    let identities = model.electrical_identities.clone();
    let rejected = act(
        &mut model,
        &root,
        Action::Reject {
            proposal_id: draft.proposal_id,
        },
    );
    assert_eq!(rejected.status, ProposalStatus::Rejected);
    assert!(rejected.assignment.is_none());
    assert_eq!(model.model_revision, revision);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(
        model
            .materialized_source_shard_value(SourceShardKind::SchematicSheet)
            .unwrap(),
        before
    );
    let len = model.journal.len();
    let recovered = act(
        &mut model,
        &root,
        Action::Preview {
            proposal_id: draft.proposal_id,
        },
    );
    assert_eq!(recovered.status, ProposalStatus::Rejected);
    assert!(recovered.assignment.is_none());
    assert_eq!(model.journal.len(), len);
    std::fs::remove_dir_all(root).unwrap();
}
