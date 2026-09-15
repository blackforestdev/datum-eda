use super::*;

fn schematic_definition(definition_id: Uuid, root_sheet_id: Uuid, name: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": definition_id,
        "root_sheet": root_sheet_id,
        "name": name
    })
}

#[test]
fn journaled_schematic_definition_replays_create_and_suppresses_deleted_stale_file() {
    let root = temp_project_root("schematic_definition_replay");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let schematic_id = Uuid::new_v5(&project_id, b"schematic");
    let root_sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let definition_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let relative_path = format!("definitions/{definition_id}.json");
    let materialized_path = format!("schematic/{relative_path}");
    let definition = schematic_definition(definition_id, root_sheet_id, "Amplifier");

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "create schematic definition".to_string(),
                },
                operations: vec![Operation::CreateSchematicDefinition {
                    schematic_id,
                    definition_id,
                    relative_path: relative_path.clone(),
                    definition: definition.clone(),
                }],
            },
        )
        .expect("schematic definition create should commit");
    assert!(model.objects.contains_key(&definition_id));

    let definition_path = root.join(&materialized_path);
    std::fs::remove_file(&definition_path).expect("promoted definition should remove");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay missing journal definition");
    assert!(replayed.objects.contains_key(&definition_id));
    let replayed_definition = replayed
        .materialized_source_shard_value_by_relative_path(&materialized_path)
        .expect("materialized created definition should read");
    assert_eq!(replayed_definition["name"], "Amplifier");

    let mut replayed = replayed;
    replayed
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(replayed.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete schematic definition".to_string(),
                },
                operations: vec![Operation::DeleteSchematicDefinition {
                    schematic_id,
                    definition_id,
                    relative_path: relative_path.clone(),
                    definition: definition.clone(),
                }],
            },
        )
        .expect("schematic definition delete should commit");
    assert!(!replayed.objects.contains_key(&definition_id));

    write_json(
        &definition_path,
        schematic_definition(definition_id, root_sheet_id, "Stale"),
    );
    let deleted = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should suppress stale deleted definition");
    assert!(!deleted.objects.contains_key(&definition_id));
    assert!(
        deleted
            .source_shards
            .iter()
            .all(|shard| shard.relative_path != materialized_path)
    );
    assert!(
        deleted
            .materialized_source_shard_value_by_relative_path(&materialized_path)
            .is_err()
    );
}
