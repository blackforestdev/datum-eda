use super::*;

#[test]
fn tool_component_instance_direct_commit_requires_proposal() {
    let root = temp_project_root("tool_component_instance_direct_commit_requires_proposal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let component_instance_id = Uuid::new_v4();
    let payload = serde_json::Value::Null;
    let cases = [
        Operation::CreateComponentInstance {
            component_instance_id,
            component_instance: payload.clone(),
        },
        Operation::SetComponentInstance {
            component_instance_id,
            previous_component_instance: payload.clone(),
            component_instance: payload.clone(),
        },
        Operation::DeleteComponentInstance {
            component_instance_id,
            component_instance: payload,
        },
    ];

    for operation in cases {
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("resolve should succeed");
        let error = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v5(
                        &project_id,
                        format!("tool-component-instance-{operation:?}").as_bytes(),
                    ),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "tool".to_string(),
                        source: CommitSource::Tool,
                        reason: "attempt direct cross-domain identity write".to_string(),
                    },
                    operations: vec![operation],
                },
            )
            .expect_err("tool-authored ComponentInstance writes must be proposals");
        assert!(
            format!("{error:#}")
                .contains("proposal_required_for_automated_cross_domain_identity_operation"),
            "unexpected error: {error:#}"
        );
    }

    assert!(
        !root
            .join(format!(
                ".datum/component_instances/{component_instance_id}.json"
            ))
            .exists(),
        "blocked ComponentInstance operations must not stage authored identity shards"
    );
}

#[test]
fn tool_relationship_direct_commit_requires_proposal() {
    let root = temp_project_root("tool_relationship_direct_commit_requires_proposal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let relationship_id = Uuid::new_v4();
    let payload = serde_json::Value::Null;
    let cases = [
        Operation::CreateRelationship {
            relationship_id,
            relationship: payload.clone(),
        },
        Operation::SetRelationship {
            relationship_id,
            previous_relationship: payload.clone(),
            relationship: payload.clone(),
        },
        Operation::DeleteRelationship {
            relationship_id,
            relationship: payload,
        },
    ];

    for operation in cases {
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("resolve should succeed");
        let error = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v5(
                        &project_id,
                        format!("tool-relationship-{operation:?}").as_bytes(),
                    ),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "tool".to_string(),
                        source: CommitSource::Tool,
                        reason: "attempt direct cross-domain relationship write".to_string(),
                    },
                    operations: vec![operation],
                },
            )
            .expect_err("tool-authored Relationship writes must be proposals");
        assert!(
            format!("{error:#}")
                .contains("proposal_required_for_automated_cross_domain_identity_operation"),
            "unexpected error: {error:#}"
        );
    }

    assert!(
        !root
            .join(format!(".datum/relationships/{relationship_id}.json"))
            .exists(),
        "blocked Relationship operations must not stage authored relationship shards"
    );
}

#[test]
fn tool_variant_overlay_direct_commit_requires_proposal() {
    let root = temp_project_root("tool_variant_overlay_direct_commit_requires_proposal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let variant_id = Uuid::new_v4();
    let payload = serde_json::Value::Null;
    let cases = [
        Operation::CreateVariantOverlay {
            variant_id,
            variant: payload.clone(),
        },
        Operation::SetVariantOverlay {
            variant_id,
            previous_variant: payload.clone(),
            variant: payload.clone(),
        },
        Operation::DeleteVariantOverlay {
            variant_id,
            variant: payload,
        },
    ];

    for operation in cases {
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("resolve should succeed");
        let error = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v5(
                        &project_id,
                        format!("tool-variant-overlay-{operation:?}").as_bytes(),
                    ),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "tool".to_string(),
                        source: CommitSource::Tool,
                        reason: "attempt direct cross-domain variant write".to_string(),
                    },
                    operations: vec![operation],
                },
            )
            .expect_err("tool-authored VariantOverlay writes must be proposals");
        assert!(
            format!("{error:#}")
                .contains("proposal_required_for_automated_cross_domain_identity_operation"),
            "unexpected error: {error:#}"
        );
    }

    assert!(
        !root
            .join(format!(".datum/variants/{variant_id}.json"))
            .exists(),
        "blocked VariantOverlay operations must not stage authored variant shards"
    );
}

#[test]
fn automated_pool_library_direct_commit_requires_proposal() {
    let root = temp_project_root("automated_pool_library_direct_commit_requires_proposal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let package_id = Uuid::new_v4();
    let padstack_id = Uuid::new_v4();
    let object_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let relative_path = format!("pool/symbols/{object_id}.json");
    let pool_value = serde_json::json!({
        "schema_version": 1,
        "uuid": object_id,
        "name": "AUTO_BLOCKED"
    });
    let cases = vec![
        Operation::CreatePoolPackage {
            package_id,
            relative_path: format!("pool/packages/{package_id}.json"),
            package: pool_value.clone(),
        },
        Operation::DeletePoolPackage {
            package_id,
            relative_path: format!("pool/packages/{package_id}.json"),
            package: pool_value.clone(),
        },
        Operation::CreatePoolPadstack {
            padstack_id,
            relative_path: format!("pool/padstacks/{padstack_id}.json"),
            padstack: pool_value.clone(),
        },
        Operation::DeletePoolPadstack {
            padstack_id,
            relative_path: format!("pool/padstacks/{padstack_id}.json"),
            padstack: pool_value.clone(),
        },
        Operation::CreatePoolLibraryObject {
            object_id,
            relative_path: relative_path.clone(),
            object_kind: "symbols".to_string(),
            object: pool_value.clone(),
        },
        Operation::SetPoolLibraryObject {
            object_id,
            relative_path: relative_path.clone(),
            object_kind: "symbols".to_string(),
            previous_object: pool_value.clone(),
            object: pool_value.clone(),
        },
        Operation::DeletePoolLibraryObject {
            object_id,
            relative_path: relative_path.clone(),
            object_kind: "symbols".to_string(),
            object: pool_value.clone(),
        },
        Operation::AttachPoolPartModel {
            part_id,
            relative_path: format!("pool/parts/{part_id}.json"),
            previous_attachments: vec![],
            attachments: vec![pool_value.clone()],
        },
        Operation::DetachPoolPartModel {
            part_id,
            relative_path: format!("pool/parts/{part_id}.json"),
            previous_attachments: vec![pool_value.clone()],
            attachments: vec![],
        },
    ];

    for source in [CommitSource::Tool, CommitSource::Assistant] {
        for operation in cases.clone() {
            let mut model = ProjectResolver::new(&root)
                .resolve()
                .expect("resolve should succeed");
            let error = model
                .commit_journaled(
                    &root,
                    OperationBatch {
                        batch_id: Uuid::new_v5(
                            &project_id,
                            format!("automated-library-{source:?}-{operation:?}").as_bytes(),
                        ),
                        expected_model_revision: Some(model.model_revision.clone()),
                        provenance: CommitProvenance {
                            actor: "automation".to_string(),
                            source,
                            reason: "attempt direct library write".to_string(),
                        },
                        operations: vec![operation],
                    },
                )
                .expect_err("automated library writes must be proposals");
            assert!(
                format!("{error:#}").contains("proposal_required_for_automated_library_operation"),
                "unexpected error: {error:#}"
            );
        }
    }

    assert!(
        !root.join(relative_path).exists(),
        "blocked automated library operations must not stage pool shards"
    );
}
