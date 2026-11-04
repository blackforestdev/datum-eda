use super::*;

#[test]
fn native_annotation_stale_complete_set_reject_and_tampered_proposal_refuse() {
    let (root, mut model, sheet, ids, _) = setup("annotation-stale");
    let id = Uuid::new_v4();
    act(
        &mut model,
        &root,
        Action::CreateAnnotation {
            proposal_id: id,
            sheet,
            instance_path: vec![],
        },
    );
    let recorded = model.proposals[&id].clone();
    for tamper in ["omit", "value", "reference"] {
        let mut changed = recorded.clone();
        if tamper == "omit" {
            changed.batch.operations.retain(|op| !matches!(op,Operation::GuardObjectRevision{object_id,..} if *object_id==ids[2]));
        } else {
            for op in &mut changed.batch.operations {
                if let Operation::SetSchematicSymbol { symbol, .. } = op {
                    symbol[tamper] = json!("unreviewed");
                    break;
                }
            }
        }
        assert!(summary(&model, &changed, true).is_err());
    }
    verb(
        &mut model,
        &root,
        "datum.schematic.set_symbol_value",
        json!({"symbol":ids[2],"instance_path":[],"value":"unchanged member edited"}),
    );
    let before = raw(&model, ids[0]);
    let count = model.journal.len();
    for action in [
        Action::Preview { proposal_id: id },
        Action::AcceptApply { proposal_id: id },
    ] {
        let req = request(&model, &root, action);
        assert!(proposal::execute(&mut model, &req).is_err());
    }
    assert_eq!(model.journal.len(), count);
    let rejected = act(&mut model, &root, Action::Reject { proposal_id: id });
    assert_eq!(rejected.status, ProposalStatus::Rejected);
    assert!(rejected.annotation.is_none());
    assert_eq!(raw(&model, ids[0]), before);
    assert!(preview(&model, sheet, &[Uuid::new_v4()]).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_annotation_refuses_unsupported_complete_sheet_and_malformed_references() {
    use crate::api::native_write::board_components::{
        BoardPackagePlacement, build_place_board_package,
    };
    for mode in ["stale", "multiple", "physical", "locked", "malformed"] {
        let (root, mut model, sheet, ids, cis) = setup("annotation-refusal");
        if mode == "multiple" || mode == "physical" {
            let previous = raw(&model, cis[0])["component_instance"].clone();
            let mut next = previous.clone();
            next["object_revision"] = json!(model.objects[&cis[0]].object_revision.0 + 1);
            if mode == "physical" {
                let package = crate::board::PlacedPackage {
                    uuid: Uuid::new_v4(),
                    part: model.component_instances[&cis[0]].part_ref.unwrap(),
                    package: "50000000-0000-4000-8000-000000000001".parse().unwrap(),
                    reference: "U1".into(),
                    value: "".into(),
                    position: crate::ir::geometry::Point::zero(),
                    rotation: 0,
                    layer: 1,
                    locked: false,
                };
                let write = build_place_board_package(
                    &model,
                    provenance(),
                    &BoardPackagePlacement {
                        package: package.clone(),
                        materialized: json!({}),
                    },
                )
                .unwrap();
                commit_prepared(&mut model, &root, write).unwrap();
                next["placed_package_refs"] = json!([{"object_id":package.uuid,"object_revision":model.objects[&package.uuid].object_revision.0}]);
            } else {
                next["placed_symbol_refs"].as_array_mut().unwrap().push(json!({"object_id":ids[2],"object_revision":model.objects[&ids[2]].object_revision.0}));
            }
            let write = BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetComponentInstance {
                    component_instance_id: cis[0],
                    previous_component_instance: previous,
                    component_instance: next,
                })
                .finish()
                .unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
        } else {
            let target = if mode == "stale" { ids[0] } else { ids[2] };
            let mut payload = raw(&model, target)["symbols"][target.to_string()].clone();
            match mode {
                "locked" => payload["locked"] = json!(true),
                "malformed" => payload["reference"] = json!("123"),
                _ => payload["unrepinned_change"] = json!(true),
            }
            let write = BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetSchematicSymbol {
                    sheet_id: sheet,
                    symbol_id: target,
                    symbol: payload,
                })
                .finish()
                .unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        let before = raw(&model, ids[0]);
        let count = model.journal.len();
        assert!(preview(&model, sheet, &[]).is_err(), "{mode}");
        let req = request(
            &model,
            &root,
            Action::CreateAnnotation {
                proposal_id: Uuid::new_v4(),
                sheet,
                instance_path: vec![],
            },
        );
        assert!(proposal::execute(&mut model, &req).is_err(), "{mode}");
        assert_eq!(model.journal.len(), count);
        assert_eq!(raw(&model, ids[0]), before);
        std::fs::remove_dir_all(root).unwrap();
    }
}
