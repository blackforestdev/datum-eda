use super::*;

#[test]
fn resolver_rejects_unsupported_component_instance_schema_version() {
    let root = temp_project_root("component_instance_unsupported_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let component_instance_id = Uuid::new_v4();
    write_json(
        &root.join(format!(
            ".datum/component_instances/{component_instance_id}.json"
        )),
        serde_json::json!({
            "schema_version": 2,
            "component_instance": {
                "uuid": component_instance_id,
                "object_revision": 0,
                "placed_symbol_refs": [],
                "placed_package_refs": []
            }
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with component-instance diagnostic");

    assert!(model.component_instances.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_component_instance_shard"
            && diagnostic
                .message
                .contains("unsupported ComponentInstance schema_version 2")
    }));
}

#[test]
fn resolver_defaults_legacy_component_instance_shard_schema_version() {
    let root = temp_project_root("component_instance_legacy_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );

    write_json(
        &root.join(format!(
            ".datum/component_instances/{component_instance_id}.json"
        )),
        serde_json::json!({
            "component_instance": {
                "uuid": component_instance_id,
                "object_revision": 0,
                "placed_symbol_refs": [{
                    "object_id": symbol_id,
                    "object_revision": 0
                }],
                "placed_package_refs": [{
                    "object_id": package_id,
                    "object_revision": 0
                }]
            }
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with legacy component-instance shard");
    assert!(
        model
            .component_instances
            .contains_key(&component_instance_id)
    );
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_component_instance_shard")
    );
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ComponentInstance && shard.schema_version.is_none()
    }));
}

#[test]
fn component_instance_inventory_distinguishes_absence_and_unusable_path() {
    let root = temp_project_root("component_inventory_absence");
    write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
    let absent = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        !absent
            .diagnostics
            .iter()
            .any(|d| d.code == "component_instance_inventory_unavailable")
    );
    std::fs::create_dir_all(root.join(".datum")).unwrap();
    let path = root.join(".datum/component_instances");
    std::fs::write(&path, b"not a directory").unwrap();
    let unavailable = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        unavailable
            .diagnostics
            .iter()
            .any(|d| d.code == "component_instance_inventory_unavailable"
                && d.path.as_ref() == Some(&path))
    );
    assert!(unavailable.component_instances.is_empty());
}

#[test]
#[cfg(unix)]
fn component_instance_inventory_reports_non_utf8_json_and_dangling_links() {
    use std::os::unix::{ffi::OsStringExt, fs::symlink};
    let root = temp_project_root("component_inventory_non_utf8");
    write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
    let dir = root.join(".datum/component_instances");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(std::ffi::OsString::from_vec(b"owner-\xff.json".to_vec()));
    std::fs::write(&path, b"{}").unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        model
            .diagnostics
            .iter()
            .any(|d| d.code == "component_instance_inventory_unavailable"
                && d.path.as_ref() == Some(&dir)
                && d.message.contains("not canonical UTF-8"))
    );
    assert!(serde_json::to_value(&model.diagnostics).is_ok());
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&dir).unwrap();
    symlink(root.join("missing-ci-target"), &dir).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        model
            .diagnostics
            .iter()
            .any(|d| d.code == "component_instance_inventory_unavailable"
                && d.path.as_ref() == Some(&dir))
    );
}
