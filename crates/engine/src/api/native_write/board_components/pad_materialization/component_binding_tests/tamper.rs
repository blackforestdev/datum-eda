//! Real resolver refusal over isolated corrupt journal copies, in both directions.
use super::*;
use std::path::Path;
fn copy(from: &Path, to: &Path) {
    std::fs::create_dir(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
pub(super) fn refuse(original: &Path, model: &DesignModel) {
    let root =
        std::env::temp_dir().join(format!("datum_binding_history_tamper_{}", Uuid::new_v4()));
    copy(original, &root);
    let path = crate::substrate::transaction_journal_path(&root);
    let raw = std::fs::read_to_string(&path).unwrap();
    let records: Vec<serde_json::Value> = raw
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let before = authored_bytes(&root, model);
    for control in 0..18 {
        let mut records = records.clone();
        let n = records.len() - 1;
        let key = if (6..10).contains(&control) || control == 15 || control == 17 {
            "inverse_operations"
        } else {
            "operations"
        };
        let operations = records[n][key].as_array_mut().unwrap();
        match control {
            0 | 6 => {
                let ci = operations
                    .iter_mut()
                    .find(|o| o["kind"] == "set_component_instance")
                    .unwrap();
                let value = ci["component_instance"]["object_revision"]
                    .as_u64()
                    .unwrap()
                    + 1;
                ci["component_instance"]["object_revision"] = json!(value);
            }
            1 => {
                let ci = operations
                    .iter_mut()
                    .find(|o| o["kind"] == "set_component_instance")
                    .unwrap();
                let value = ci["component_instance"]["placed_package_refs"][0]["object_revision"]
                    .as_u64()
                    .unwrap()
                    + 1;
                ci["component_instance"]["placed_package_refs"][0]["object_revision"] =
                    json!(value);
            }
            2 | 7 => {
                let e = operations
                    .iter_mut()
                    .find(|o| o["kind"] == "set_electrical_identity")
                    .unwrap();
                let value = e["record"]["identity"]["evidence"][0]["board_pad"]["object_revision"]
                    .as_u64()
                    .unwrap()
                    + 1;
                e["record"]["identity"]["evidence"][0]["board_pad"]["object_revision"] =
                    json!(value);
            }
            3 | 8 => {
                let p = operations
                    .iter_mut()
                    .find(|o| o["kind"] == "set_board_package_package")
                    .unwrap();
                let pin = &mut p["materialized"]["component_pads"][0]["source_binding"]["component_instance"]
                    ["object_revision"];
                *pin = json!(pin.as_u64().unwrap() + 1);
            }
            4 | 9 => {
                let ci = operations
                    .iter_mut()
                    .find(|o| o["kind"] == "set_component_instance")
                    .unwrap();
                ci["component_instance"]["part_ref"]["object_id"] = json!(Uuid::new_v4());
            }
            5 => operations.retain(|o| o["kind"] != "set_electrical_identity"),
            10 | 11 => {
                let kind = if control == 10 {
                    "set_board_package_package"
                } else {
                    "set_electrical_identity"
                };
                let prior = records[..n]
                    .iter_mut()
                    .rev()
                    .find(|r| {
                        r["operations"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|o| o["kind"] == kind)
                    })
                    .unwrap();
                let id = prior["operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|o| o["kind"] == kind)
                    .map(|o| {
                        if control == 10 {
                            o["materialized"]["component_pads"][0]["uuid"].clone()
                        } else {
                            o["record"]["id"].clone()
                        }
                    })
                    .unwrap();
                prior["diff"]["modified"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|v| v != &id);
            }
            12 | 13 => {
                let prior = &mut records[n - 1];
                let package = prior["operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|o| o["kind"] == "set_board_package_package")
                    .unwrap()["package_id"]
                    .clone();
                let op = if control == 12 {
                    json!({"kind":"bump_object_revision","object_id":package})
                } else {
                    serde_json::to_value(Operation::SetBoardPackagePosition {
                        package_id: serde_json::from_value(package).unwrap(),
                        x: 1_000_000,
                        y: 0,
                    })
                    .unwrap()
                };
                prior["operations"].as_array_mut().unwrap().push(op);
            }
            14 | 15 => {
                let e = operations
                    .iter_mut()
                    .find(|o| o["kind"] == "set_electrical_identity")
                    .unwrap();
                let value = e["record"]["object_revision"].as_u64().unwrap() + 1;
                e["record"]["object_revision"] = json!(value);
            }
            _ => {
                let e = operations
                    .iter_mut()
                    .find(|o| o["kind"] == "set_electrical_identity")
                    .unwrap();
                let pin = &mut e["record"]["identity"]["evidence"][0]["component_instance"]["object_revision"];
                *pin = json!(pin.as_u64().unwrap() + 1);
            }
        }
        // Fault injection is confined to this disposable test copy. The actual
        // journal and its original prefix are never rewritten by compensation.
        let text = records
            .iter()
            .map(|v| serde_json::to_string(v).unwrap() + "\n")
            .collect::<String>();
        std::fs::write(&path, &text).unwrap();
        let refused = ProjectResolver::new(&root).resolve().unwrap();
        assert!(
            refused
                .diagnostics
                .iter()
                .any(|d| d.code == "journal_link_mismatch"),
            "tamper {control}"
        );
        assert!(refused.journal.len() < records.len(), "tamper {control}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        assert_eq!(authored_bytes(&root, model), before);
    }
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(original)).unwrap(),
        raw.as_bytes()
    );
    std::fs::remove_dir_all(root).unwrap();
}
