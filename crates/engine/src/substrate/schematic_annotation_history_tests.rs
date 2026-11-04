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
