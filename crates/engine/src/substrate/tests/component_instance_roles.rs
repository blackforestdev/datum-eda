use super::*;

#[test]
fn resolver_preserves_persisted_component_instance_role_metadata() {
    let root = temp_project_root("component_instance_role_metadata");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_component_instance_shard_with_roles(
        &root,
        component_instance_id,
        symbol_id,
        package_id,
        serde_json::json!({
            symbol_id.to_string(): { "role": "logical_unit", "label": "A" }
        }),
        serde_json::json!({
            package_id.to_string(): { "role": "physical_package", "label": "main" }
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .expect("persisted component instance should resolve");
    assert_eq!(
        instance
            .placed_symbol_roles
            .get(&symbol_id)
            .map(|metadata| metadata.role.as_str()),
        Some("logical_unit")
    );
    assert_eq!(
        instance
            .placed_symbol_roles
            .get(&symbol_id)
            .and_then(|metadata| metadata.label.as_deref()),
        Some("A")
    );
    assert_eq!(
        instance
            .placed_package_roles
            .get(&package_id)
            .map(|metadata| metadata.role.as_str()),
        Some("physical_package")
    );
}

#[test]
fn resolver_rejects_invalid_component_instance_role_metadata() {
    let root = temp_project_root("component_instance_invalid_role_metadata");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_component_instance_shard_with_roles(
        &root,
        component_instance_id,
        symbol_id,
        package_id,
        serde_json::json!({
            Uuid::new_v4().to_string(): { "role": "logical_unit" }
        }),
        serde_json::json!({
            package_id.to_string(): { "role": "" }
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    assert!(
        !model
            .component_instances
            .contains_key(&component_instance_id)
    );
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "component_instance_invalid_symbol_roles"
            || diagnostic.code == "component_instance_invalid_package_roles"
    }));
}

fn write_component_instance_shard_with_roles(
    root: &Path,
    component_instance_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
    symbol_roles: serde_json::Value,
    package_roles: serde_json::Value,
) {
    write_json(
        &root.join(format!(
            ".datum/component_instances/{component_instance_id}.json"
        )),
        serde_json::json!({
            "schema_version": 1,
            "component_instance": {
                "uuid": component_instance_id,
                "object_revision": 1,
                "placed_symbol_refs": [{
                    "object_id": symbol_id,
                    "object_revision": 0
                }],
                "placed_package_refs": [{
                    "object_id": package_id,
                    "object_revision": 0
                }],
                "placed_symbol_roles": symbol_roles,
                "placed_package_roles": package_roles
            }
        }),
    );
}
