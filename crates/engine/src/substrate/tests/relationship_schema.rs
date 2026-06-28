use super::*;

#[test]
fn resolver_rejects_unsupported_relationship_shard_schema_version() {
    let root = temp_project_root("relationship_unsupported_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    write_json(
        &root.join(".datum/relationships/component-links.json"),
        serde_json::json!({
            "schema_version": 2,
            "relationships": [{
                "id": Uuid::new_v4(),
                "kind": "pending",
                "from": [],
                "to": [],
                "authored_intent": [],
                "object_revision": 0
            }]
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with relationship diagnostic");
    assert!(model.relationships.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_relationship_shard"
            && diagnostic
                .message
                .contains("unsupported Relationship schema_version 2")
    }));
}

#[test]
fn resolver_defaults_legacy_relationship_shard_schema_version() {
    let root = temp_project_root("relationship_legacy_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let relationship_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    write_json(
        &root.join(".datum/relationships/component-links.json"),
        serde_json::json!({
            "relationships": [{
                "id": relationship_id,
                "kind": "pending",
                "from": [],
                "to": [],
                "authored_intent": [],
                "object_revision": 0
            }]
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with legacy relationship shard");
    assert!(model.diagnostics.is_empty());
    assert!(model.relationships.contains_key(&relationship_id));
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::Relationship && shard.schema_version.is_none()
    }));
}

#[test]
fn resolver_rejects_unsupported_variant_overlay_schema_version() {
    let root = temp_project_root("variant_overlay_unsupported_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before variant");
    write_json(
        &root.join(".datum/variants/assembly.json"),
        serde_json::json!({
            "schema_version": 2,
            "variants": [{
                "id": Uuid::new_v4(),
                "name": "Future Variant",
                "base_model_revision": before.model_revision,
                "variant_revision": 0,
                "fitted": {},
                "relationship_overrides": {},
                "property_overrides": {}
            }]
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with variant diagnostic");
    assert!(model.variants.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_variant_overlay_shard"
            && diagnostic
                .message
                .contains("unsupported VariantOverlay schema_version 2")
    }));
}

#[test]
fn resolver_defaults_legacy_variant_overlay_shard_schema_version() {
    let root = temp_project_root("variant_overlay_legacy_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before variant");
    let variant_id = Uuid::new_v4();
    write_json(
        &root.join(".datum/variants/assembly.json"),
        serde_json::json!({
            "variants": [{
                "id": variant_id,
                "name": "Legacy Variant",
                "base_model_revision": before.model_revision,
                "variant_revision": 0,
                "fitted": {},
                "relationship_overrides": {},
                "property_overrides": {}
            }]
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with legacy variant shard");
    assert!(model.variants.contains_key(&variant_id));
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_variant_overlay_shard")
    );
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::VariantOverlay && shard.schema_version.is_none()
    }));
}
