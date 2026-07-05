use super::*;

#[test]
fn resolver_preserves_persisted_component_instance_part_ref() {
    let root = temp_project_root("component_instance_part_ref");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_pool_part_object(&root, part_id, 0);
    write_component_instance_shard_with_part_ref(
        &root,
        component_instance_id,
        symbol_id,
        package_id,
        part_id,
        0,
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with component instance part ref");
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .expect("persisted component instance should resolve");
    assert_eq!(instance.part_ref, Some(part_id));
    assert_eq!(instance.placed_symbol_refs, vec![symbol_id]);
    assert_eq!(instance.placed_package_refs, vec![package_id]);
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.starts_with("component_instance_"))
    );
}

#[test]
fn resolver_projects_part_ref_from_library_binding() {
    let root = temp_project_root("component_instance_library_binding_part");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let binding_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_pool_part_object(&root, part_id, 0);
    write_component_instance_shard_with_library_binding(
        &root,
        component_instance_id,
        symbol_id,
        package_id,
        binding_id,
        part_id,
        0,
        None,
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with component instance LibraryBinding");
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .expect("persisted component instance should resolve");
    assert_eq!(instance.part_ref, Some(part_id));
    assert_eq!(instance.library_bindings.len(), 1);
    let binding = instance
        .library_bindings
        .get(&binding_id)
        .expect("LibraryBinding should be retained");
    assert_eq!(binding.target_object_id, part_id);
    assert_eq!(binding.pinned_object_revision.0, 0);
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.starts_with("component_instance_"))
    );
}

#[test]
fn resolver_rejects_invalid_component_instance_part_ref() {
    let root = temp_project_root("component_instance_invalid_part_ref");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_pool_part_object(&root, part_id, 0);
    write_component_instance_shard_with_part_ref(
        &root,
        component_instance_id,
        symbol_id,
        package_id,
        part_id,
        7,
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with diagnostics");
    assert!(
        !model
            .component_instances
            .contains_key(&component_instance_id)
    );
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "component_instance_invalid_part_ref"
            && diagnostic.message.contains(&part_id.to_string())
    }));
}

#[test]
fn resolver_rejects_mismatched_part_ref_and_library_binding() {
    let root = temp_project_root("component_instance_library_binding_mismatch");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let component_instance_id = Uuid::new_v4();
    let binding_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let other_part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    write_pool_part_object(&root, part_id, 0);
    write_pool_part_object(&root, other_part_id, 0);
    write_component_instance_shard_with_library_binding(
        &root,
        component_instance_id,
        symbol_id,
        package_id,
        binding_id,
        part_id,
        0,
        Some(other_part_id),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with diagnostics");
    assert!(
        !model
            .component_instances
            .contains_key(&component_instance_id)
    );
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "component_instance_invalid_library_binding"
            && diagnostic
                .message
                .contains("does not match part LibraryBinding")
    }));
}

fn write_component_instance_shard_with_part_ref(
    root: &Path,
    component_instance_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
    part_id: Uuid,
    part_revision: u64,
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
                "part_ref": {
                    "object_id": part_id,
                    "object_revision": part_revision
                },
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
}

fn write_component_instance_shard_with_library_binding(
    root: &Path,
    component_instance_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
    binding_id: Uuid,
    binding_part_id: Uuid,
    binding_part_revision: u64,
    compatibility_part_ref: Option<Uuid>,
) {
    let part_ref = compatibility_part_ref.map(|part_id| {
        serde_json::json!({
            "object_id": part_id,
            "object_revision": 0
        })
    });
    write_json(
        &root.join(format!(
            ".datum/component_instances/{component_instance_id}.json"
        )),
        serde_json::json!({
            "schema_version": 1,
            "component_instance": {
                "uuid": component_instance_id,
                "object_revision": 1,
                "part_ref": part_ref,
                "library_bindings": {
                    (binding_id.to_string()): {
                        "target_object_id": binding_part_id,
                        "pinned_object_revision": binding_part_revision,
                        "binding_role": "part"
                    }
                },
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
}

fn write_pool_part_object(root: &Path, part_id: Uuid, object_revision: u64) {
    let project_path = root.join("project.json");
    let mut project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&project_path).expect("project json should read"))
            .expect("project json should parse");
    project["pools"] = serde_json::json!([{ "path": "pool", "priority": 1 }]);
    write_json(&project_path, project);
    write_json(
        &root.join(format!("pool/parts/{part_id}.json")),
        serde_json::json!({
            "schema_version": 1,
            "uuid": part_id,
            "object_revision": object_revision,
            "entity": Uuid::new_v5(&part_id, b"entity"),
            "package": Uuid::new_v5(&part_id, b"package"),
            "mpn": "TEST-PART",
            "manufacturer": "Datum",
            "value": "TEST",
            "description": "",
            "datasheet": "",
            "lifecycle": "Active"
        }),
    );
}
