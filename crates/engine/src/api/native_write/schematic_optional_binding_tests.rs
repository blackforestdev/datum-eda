//! Native GUI placement shares the CLI's optional Part binding semantics.
use super::*;
use crate::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
};

fn library_objects(kinds: &[&str], second_part: bool) -> Vec<PoolLibraryOperationSpec> {
    let baseline = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/library/native_authored_baseline_v1/pool");
    let mut objects = Vec::new();
    for kind in kinds {
        for entry in std::fs::read_dir(baseline.join(kind)).unwrap() {
            let object: serde_json::Value =
                serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
            let id = object["uuid"].as_str().unwrap().parse().unwrap();
            if *kind == "parts" && second_part {
                let mut second = object.clone();
                let id = Uuid::new_v4();
                second["uuid"] = serde_json::json!(id);
                second["default_pin_pad_map"] = serde_json::Value::Null;
                objects.push(PoolLibraryOperationSpec::Create {
                    target: PoolLibraryObjectTarget::new("pool", kind, id),
                    object: second,
                });
            }
            objects.push(PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", kind, id),
                object,
            });
        }
    }
    objects
}

#[test]
fn optional_binding_native_symbol_preserves_missing_and_ambiguous_part_without_inventing_instances()
{
    let symbol: Uuid = "30000000-0000-4000-8000-000000000001".parse().unwrap();
    let full = [
        "units",
        "symbols",
        "entities",
        "packages",
        "padstacks",
        "footprints",
        "parts",
        "pin_pad_maps",
    ];
    for (kinds, duplicate, status, entity_expected) in [
        (
            &["units", "symbols"][..],
            false,
            "unresolved_entity_gate",
            false,
        ),
        (
            &["units", "symbols", "entities"][..],
            false,
            "bound_without_part",
            true,
        ),
        (&full[..], true, "ambiguous_part", true),
    ] {
        let mut f = fixture("native_optional_binding");
        let prepared = build_pool_library_write(
            &f.model,
            provenance(),
            Some("pool"),
            library_objects(kinds, duplicate),
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        let resolution =
            resolve_pool_symbol_component_binding(&f.model, Some(&symbol.to_string())).unwrap();
        assert_eq!(resolution.status, status);
        assert!(!resolution.diagnostics.is_empty());
        let instances_before = f
            .model
            .objects
            .values()
            .filter(|o| o.kind == "component_instance")
            .count();
        let params = serde_json::json!({"sheet":f.sheet,"library_symbol":symbol,"reference":"U2","value":"Unbound test","x_nm":10_000_000,"y_nm":20_000_000});
        let prepared = place_symbol(
            &NativeWriteContext {
                model: &f.model,
                project_root: &f.root,
            },
            WriteProvenance::new("test", CommitSource::Manual, "optional binding"),
            params.clone(),
        )
        .unwrap();
        assert!(
            !prepared
                .batch
                .operations
                .iter()
                .any(|o| matches!(o, Operation::CreateComponentInstance { .. }))
        );
        let id = prepared.primary_object_id.unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        let reopened = ProjectResolver::new(&f.root).resolve().unwrap();
        let source = schematic(&reopened).unwrap();
        let placed = &source.sheets[&f.sheet].symbols[&id];
        assert_eq!(placed.part, None);
        assert_eq!(placed.entity.is_some(), entity_expected);
        assert_eq!(placed.gate.is_some(), entity_expected);
        assert_eq!(placed.pins.len(), 5);
        assert_eq!(placed.lib_id.as_deref(), Some(symbol.to_string().as_str()));
        assert_eq!(
            reopened
                .objects
                .values()
                .filter(|o| o.kind == "component_instance")
                .count(),
            instances_before
        );
        assert!(
            place_symbol(
                &NativeWriteContext {
                    model: &reopened,
                    project_root: &f.root
                },
                provenance(),
                params
            )
            .unwrap_err()
            .to_string()
            .contains("unique")
        );
    }
}

#[test]
fn optional_binding_does_not_admit_unknown_symbol_or_invalid_unit_or_pin_reference() {
    for missing_unit in [false, true] {
        let mut f = fixture("native_optional_binding_invalid");
        let mut objects = library_objects(&["units", "symbols"], false);
        for object in &mut objects {
            if let PoolLibraryOperationSpec::Create { target: _, object } = object
                && object.get("pin_anchors").is_some()
            {
                if missing_unit {
                    object["unit"] = serde_json::json!(Uuid::new_v4());
                } else {
                    object["pin_anchors"][0]["pin"] = serde_json::json!(Uuid::new_v4());
                }
            }
        }
        let prepared =
            build_pool_library_write(&f.model, provenance(), Some("pool"), objects).unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        let before = f.model.model_revision.clone();
        for symbol in [
            Uuid::new_v4(),
            "30000000-0000-4000-8000-000000000001".parse().unwrap(),
        ] {
            let result = place_symbol(
                &NativeWriteContext {
                    model: &f.model,
                    project_root: &f.root,
                },
                provenance(),
                serde_json::json!({"sheet":f.sheet,"library_symbol":symbol,"reference":"U2","value":"Invalid","x_nm":0,"y_nm":0}),
            );
            assert!(result.is_err());
            assert_eq!(
                ProjectResolver::new(&f.root)
                    .resolve()
                    .unwrap()
                    .model_revision,
                before
            );
        }
    }
}
