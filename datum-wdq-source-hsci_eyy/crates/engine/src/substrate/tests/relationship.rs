use super::*;

#[test]
fn resolver_discovers_authored_relationship_shards_and_derives_status() {
    let root = temp_project_root("relationship_discovery");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let relationship_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before relationships");
    write_json(
        &root.join(".datum/relationships/component-links.json"),
        serde_json::json!({
            "schema_version": 1,
            "relationships": [{
                "id": relationship_id,
                "kind": "implemented_by",
                "from": [{ "object_id": symbol_id, "object_revision": 0 }],
                "to": [{ "object_id": package_id, "object_revision": 0 }],
                "authored_intent": [],
                "object_revision": 0
            }]
        }),
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with relationships");
    assert_ne!(after.model_revision, before.model_revision);
    assert_eq!(after.relationships.len(), 1);
    assert_eq!(
        after.relationship_statuses.get(&relationship_id),
        Some(&DerivedRelationshipStatus::Implemented)
    );
    assert!(after.objects.contains_key(&relationship_id));
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::Relationship
            && shard.authority == SourceShardAuthority::AuthoredDesign
            && shard.relative_path == ".datum/relationships/component-links.json"
    }));
}

#[test]
fn resolver_derives_unresolved_relationship_status_for_missing_refs() {
    let root = temp_project_root("relationship_missing_ref");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let relationship_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    write_json(
        &root.join(".datum/relationships/component-links.json"),
        serde_json::json!({
            "schema_version": 1,
            "relationships": [{
                "id": relationship_id,
                "kind": "implemented_by",
                "from": [{ "object_id": Uuid::new_v4(), "object_revision": 0 }],
                "to": [{ "object_id": board_id, "object_revision": 0 }],
                "authored_intent": [],
                "object_revision": 0
            }]
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with unresolved relationship");
    assert_eq!(
        model.relationship_statuses.get(&relationship_id),
        Some(&DerivedRelationshipStatus::UnresolvedMismatch)
    );
}

#[test]
fn resolver_rejects_persisted_derived_relationship_status() {
    let root = temp_project_root("relationship_derived_status_rejected");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    write_json(
        &root.join(".datum/relationships/component-links.json"),
        serde_json::json!({
            "schema_version": 1,
            "relationships": [{
                "id": Uuid::new_v4(),
                "kind": "pending",
                "from": [],
                "to": [],
                "authored_intent": [],
                "object_revision": 0,
                "derived_status": "pending_implementation"
            }]
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with relationship diagnostic");
    assert!(model.relationships.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_relationship_shard"
            && diagnostic.message.contains("derived_status")
    }));
}

#[test]
fn resolver_discovers_sparse_variant_overlay_and_derives_population() {
    let root = temp_project_root("variant_overlay_discovery");
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
            "schema_version": 1,
            "variants": [{
                "id": variant_id,
                "name": "No U1",
                "base_model_revision": before.model_revision,
                "variant_revision": 0,
                "fitted": {
                    package_id.to_string(): "unfitted"
                },
                "relationship_overrides": {},
                "property_overrides": {}
            }]
        }),
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with variant");
    assert_ne!(after.model_revision, before.model_revision);
    assert_eq!(after.variants.len(), 1);
    assert_eq!(
        after
            .variant_populations
            .get(&variant_id)
            .and_then(|population| population.get(&package_id)),
        Some(&DerivedVariantPopulation::NotApplicableForVariant)
    );
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::VariantOverlay
            && shard.authority == SourceShardAuthority::AuthoredDesign
            && shard.relative_path == ".datum/variants/assembly.json"
    }));
}

#[test]
fn resolver_keeps_package_variant_population_without_derived_component_instance() {
    let root = temp_project_root("variant_overlay_component_instance_population");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before variant");
    assert!(before.component_instances.is_empty());
    let variant_id = Uuid::new_v4();
    write_json(
        &root.join(".datum/variants/assembly.json"),
        serde_json::json!({
            "schema_version": 1,
            "variants": [{
                "id": variant_id,
                "name": "No U1",
                "base_model_revision": before.model_revision,
                "variant_revision": 0,
                "fitted": {
                    package_id.to_string(): "unfitted"
                },
                "relationship_overrides": {},
                "property_overrides": {}
            }]
        }),
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with propagated variant population");
    let population = after
        .variant_populations
        .get(&variant_id)
        .expect("variant population should derive");
    assert_eq!(
        population.get(&package_id),
        Some(&DerivedVariantPopulation::NotApplicableForVariant)
    );
}

#[test]
fn resolver_does_not_propagate_variant_population_from_legacy_derived_component_instance_id() {
    let root = temp_project_root("variant_overlay_legacy_derived_component_instance_ignored");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before variant");
    assert!(before.component_instances.is_empty());
    let legacy_component_instance_id = Uuid::new_v5(
        &project_id,
        format!("datum-eda:component-instance:{symbol_id}:{package_id}").as_bytes(),
    );
    let variant_id = Uuid::new_v4();
    write_json(
        &root.join(".datum/variants/assembly.json"),
        serde_json::json!({
            "schema_version": 1,
            "variants": [{
                "id": variant_id,
                "name": "No U1",
                "base_model_revision": before.model_revision,
                "variant_revision": 0,
                "fitted": {
                    legacy_component_instance_id.to_string(): "unfitted"
                },
                "relationship_overrides": {},
                "property_overrides": {}
            }]
        }),
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with propagated variant population");
    let population = after
        .variant_populations
        .get(&variant_id)
        .expect("variant population should derive");
    assert_eq!(
        population.get(&legacy_component_instance_id),
        Some(&DerivedVariantPopulation::NotApplicableForVariant)
    );
    assert_eq!(population.get(&symbol_id), None);
    assert_eq!(population.get(&package_id), None);
    assert!(!after.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "variant_component_instance_compatibility_derived_ignored"
    }));
}

#[test]
fn resolver_propagates_variant_population_from_authored_component_instance_to_members() {
    let root = temp_project_root("variant_overlay_authored_component_instance_member_population");
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
            "schema_version": 1,
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
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before variant");
    let variant_id = Uuid::new_v4();
    write_json(
        &root.join(".datum/variants/assembly.json"),
        serde_json::json!({
            "schema_version": 1,
            "variants": [{
                "id": variant_id,
                "name": "No U1",
                "base_model_revision": before.model_revision,
                "variant_revision": 0,
                "fitted": {
                    component_instance_id.to_string(): "unfitted"
                },
                "relationship_overrides": {},
                "property_overrides": {}
            }]
        }),
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with propagated variant population");
    let population = after
        .variant_populations
        .get(&variant_id)
        .expect("variant population should derive");
    assert_eq!(
        population.get(&component_instance_id),
        Some(&DerivedVariantPopulation::NotApplicableForVariant)
    );
    assert_eq!(
        population.get(&symbol_id),
        Some(&DerivedVariantPopulation::NotApplicableForVariant)
    );
    assert_eq!(
        population.get(&package_id),
        Some(&DerivedVariantPopulation::NotApplicableForVariant)
    );
    assert!(!after.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "variant_component_instance_compatibility_derived_ignored"
    }));
}

#[test]
fn resolver_rejects_persisted_derived_variant_population() {
    let root = temp_project_root("variant_overlay_rejects_derived_population");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before variant");
    write_json(
        &root.join(".datum/variants/assembly.json"),
        serde_json::json!({
            "schema_version": 1,
            "variants": [{
                "id": Uuid::new_v4(),
                "name": "Invalid",
                "base_model_revision": before.model_revision,
                "variant_revision": 0,
                "fitted": {},
                "relationship_overrides": {},
                "property_overrides": {},
                "derived_population": {}
            }]
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with variant diagnostic");
    assert!(model.variants.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_variant_overlay_shard"
            && diagnostic.message.contains("derived_population")
    }));
}

#[test]
fn journaled_relationship_and_variant_create_undo_redo_and_replay() {
    let root = temp_project_root("journaled_relationship_variant");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let relationship_id = Uuid::new_v4();
    let variant_id = Uuid::new_v4();
    write_project_with_symbol_and_package(
        &root, project_id, board_id, symbol_id, package_id, part_id,
    );
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before journaled create");
    let relationship = Relationship {
        id: relationship_id,
        kind: RelationshipKind::ImplementedBy,
        from: vec![RevisionedRef {
            object_id: symbol_id,
            object_revision: ObjectRevision(0),
        }],
        to: vec![RevisionedRef {
            object_id: package_id,
            object_revision: ObjectRevision(0),
        }],
        authored_intent: Vec::new(),
        object_revision: ObjectRevision(0),
    };
    let variant = VariantOverlay {
        id: variant_id,
        name: "No U1".to_string(),
        base_model_revision: model.model_revision.clone(),
        variant_revision: ObjectRevision(0),
        fitted: [(package_id, FittedState::Unfitted)].into(),
        relationship_overrides: Default::default(),
        property_overrides: Default::default(),
    };
    let relationship_path = root.join(format!(".datum/relationships/{relationship_id}.json"));
    let variant_path = root.join(format!(".datum/variants/{variant_id}.json"));

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-relationship-variant"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create relationship and variant through substrate".to_string(),
                },
                operations: vec![
                    Operation::CreateRelationship {
                        relationship_id,
                        relationship: serde_json::to_value(&relationship)
                            .expect("relationship serializes"),
                    },
                    Operation::CreateVariantOverlay {
                        variant_id,
                        variant: serde_json::to_value(&variant).expect("variant serializes"),
                    },
                ],
            },
        )
        .expect("journaled create should succeed");
    assert!(relationship_path.exists());
    assert!(variant_path.exists());
    let created = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after create");
    assert_eq!(
        created.relationship_statuses.get(&relationship_id),
        Some(&DerivedRelationshipStatus::Implemented)
    );
    assert_eq!(
        created
            .variant_populations
            .get(&variant_id)
            .and_then(|population| population.get(&package_id)),
        Some(&DerivedVariantPopulation::NotApplicableForVariant)
    );
    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo relationship and variant create".to_string(),
            },
        )
        .expect("undo create should remove relationship and variant");
    assert!(!relationship_path.exists());
    assert!(!variant_path.exists());
    let undone = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after create undo");
    assert!(!undone.relationships.contains_key(&relationship_id));
    assert!(!undone.variants.contains_key(&variant_id));
    model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo relationship and variant create".to_string(),
            },
        )
        .expect("redo create should restore relationship and variant");

    let mut updated_relationship = relationship.clone();
    updated_relationship.kind = RelationshipKind::Pending;
    updated_relationship.object_revision = ObjectRevision(1);
    let mut updated_variant = variant.clone();
    updated_variant.name = "Fit U1".to_string();
    updated_variant.variant_revision = ObjectRevision(1);
    updated_variant
        .fitted
        .insert(package_id, FittedState::Fitted);
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-relationship-variant"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "update relationship and variant through substrate".to_string(),
                },
                operations: vec![
                    Operation::SetRelationship {
                        relationship_id,
                        previous_relationship: serde_json::to_value(&relationship)
                            .expect("previous relationship serializes"),
                        relationship: serde_json::to_value(&updated_relationship)
                            .expect("updated relationship serializes"),
                    },
                    Operation::SetVariantOverlay {
                        variant_id,
                        previous_variant: serde_json::to_value(&variant)
                            .expect("previous variant serializes"),
                        variant: serde_json::to_value(&updated_variant)
                            .expect("updated variant serializes"),
                    },
                ],
            },
        )
        .expect("journaled set should succeed");
    let updated = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after set");
    assert_eq!(
        updated.relationship_statuses.get(&relationship_id),
        Some(&DerivedRelationshipStatus::PendingImplementation)
    );
    assert_eq!(
        updated
            .variant_populations
            .get(&variant_id)
            .and_then(|population| population.get(&package_id)),
        Some(&DerivedVariantPopulation::Applicable)
    );
    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo relationship and variant update".to_string(),
            },
        )
        .expect("undo set should restore previous relationship and variant");
    let restored = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after set undo");
    assert_eq!(
        restored.relationship_statuses.get(&relationship_id),
        Some(&DerivedRelationshipStatus::Implemented)
    );
    assert_eq!(
        restored
            .variant_populations
            .get(&variant_id)
            .and_then(|population| population.get(&package_id)),
        Some(&DerivedVariantPopulation::NotApplicableForVariant)
    );
    model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo relationship and variant update".to_string(),
            },
        )
        .expect("redo set should restore updated relationship and variant");
    let redone_set = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after set redo");
    assert_eq!(
        redone_set.relationship_statuses.get(&relationship_id),
        Some(&DerivedRelationshipStatus::PendingImplementation)
    );

    std::fs::remove_file(&relationship_path).expect("relationship file should delete");
    std::fs::remove_file(&variant_path).expect("variant file should delete");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves from journal replay");
    assert!(replayed.relationships.contains_key(&relationship_id));
    assert!(replayed.variants.contains_key(&variant_id));
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::Relationship
            && shard.relative_path == format!(".datum/relationships/{relationship_id}.json")
            && shard.dirty_state == SourceShardDirtyState::Missing
    }));
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::VariantOverlay
            && shard.relative_path == format!(".datum/variants/{variant_id}.json")
            && shard.dirty_state == SourceShardDirtyState::Missing
    }));
}
