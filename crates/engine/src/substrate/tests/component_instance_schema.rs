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
