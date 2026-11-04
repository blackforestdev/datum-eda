//! Multi-Symbol reference history uses exact recorded semantic and revision pairs.
use super::*;
use crate::api::native_write::{
    WriteProvenance,
    schematic_annotation::tests::{annotate, setup},
};
use crate::substrate::{CommitSource, ProjectResolver};
#[test]
fn native_annotation_history_rejects_unrecorded_reference_pose_and_binding_changes() {
    let (root, mut model, sheet, _, _) = setup("annotation-history-tamper");
    annotate(&mut model, &root, sheet);
    model
        .commit_journal_undo(
            &root,
            WriteProvenance::new("test", CommitSource::Manual, "undo annotation").into(),
        )
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let tx = model.journal.last().unwrap();
    let prior = &model.journal[..model.journal.len() - 1];
    let target = prior.last().unwrap();
    assert!(matches(tx, target, prior));
    for property in ["reference", "value", "position", "fields", "epoch"] {
        let mut changed = tx.clone();
        for op in &mut changed.operations {
            match op {
                Operation::SetSchematicSymbol { symbol, .. } if property != "epoch" => {
                    match property {
                        "position" => symbol["position"]["x"] = json!(88),
                        "fields" => {
                            symbol["fields"] = json!([{"uuid":Uuid::new_v4(),"key":"unrecorded","value":"","visible":false,"position":null}])
                        }
                        key => symbol[key] = json!("unrecorded"),
                    }
                    break;
                }
                Operation::SetComponentInstance {
                    component_instance, ..
                } if property == "epoch" => {
                    component_instance["placed_symbol_refs"][0]["object_revision"] = json!(999);
                    break;
                }
                _ => {}
            }
        }
        assert!(!matches(&changed, target, prior), "{property}");
    }
    for property in ["value", "position", "part"] {
        let mut changed = target.clone();
        for op in &mut changed.inverse_operations {
            if let Operation::SetSchematicSymbol { symbol, .. } = op {
                match property {
                    "position" => symbol["position"]["x"] = json!(88),
                    "part" => symbol["part"] = json!(Uuid::new_v4()),
                    _ => symbol["value"] = json!("outside reference-only family"),
                }
                break;
            }
        }
        assert!(normalized(&changed, prior).is_none(), "{property}");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn annotation_history_refuses_paired_unchanged_sibling_epoch_tamper() {
    use crate::api::native_write::{
        schematic_annotation::tests::annotate,
        schematic_delete_proposal::NativeProposalAction as Action,
        schematic_gate_placement::tests::{action, setup},
    };
    let (root, mut model, first, mut request) = setup("annotation-history-multigate-tamper");
    request.reference = "U9".into();
    action(
        &mut model,
        &root,
        Action::CreateGatePlacement(request.clone()),
    );
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: request.proposal_id,
        },
    );
    annotate(&mut model, &root, request.sheet);
    let original = model.journal.last().unwrap();
    assert!(family::capture(original).is_some());
    let mut tampered = original.clone();
    let mut replaced = None;
    for op in &mut tampered.operations {
        if let Operation::SetComponentInstance {
            component_instance, ..
        } = op
        {
            for member in component_instance["placed_symbol_refs"]
                .as_array_mut()
                .unwrap()
            {
                if member["object_id"] == json!(first) {
                    member["object_revision"] =
                        json!(member["object_revision"].as_u64().unwrap() + 1);
                }
            }
            replaced = Some(component_instance.clone());
        }
    }
    for op in &mut tampered.inverse_operations {
        if let Operation::SetComponentInstance {
            previous_component_instance,
            ..
        } = op
        {
            *previous_component_instance = replaced.clone().unwrap();
        }
    }
    assert!(
        family::capture(&tampered).is_none(),
        "pair equality cannot authorize an untouched sibling epoch"
    );
    std::fs::remove_dir_all(root).unwrap();
}
