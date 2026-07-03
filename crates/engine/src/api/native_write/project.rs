//! Project-root mutation builders for the native write facade: project name
//! and rules-root authoring (set rules, create/set/delete individual rules).
//!
//! Family J (part 2) of the native-write migration: all operation authoring
//! for `SetProjectName`, `SetProjectRules`, `CreateProjectRule`,
//! `SetProjectRule`, and `DeleteProjectRule` lives here. The CLI callers in
//! `crates/cli/src/command_project_roots.rs` are thin argument-parsers: they
//! read rule files, call a `build_*` function, and commit the returned
//! [`PreparedWrite`] via [`super::commit_prepared`].
//!
//! Builders are build-only; they never touch disk. Guard insertion and
//! error-message text are byte-for-byte the CLI's historical behavior —
//! journal records and shards must not drift. Project genesis (the initial
//! shard bootstrap) lives in [`super::genesis`].

use uuid::Uuid;

use crate::error::EngineError;
use crate::substrate::{DesignModel, Operation, SourceShardKind};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};

/// Build the batch that renames the project (guards the project root
/// object). The name is trimmed; an empty name is rejected.
pub fn build_set_project_name(
    model: &DesignModel,
    provenance: WriteProvenance,
    name: &str,
) -> Result<PreparedWrite, EngineError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(EngineError::Validation(
            "project name must not be empty".to_string(),
        ));
    }
    let project_id = model.project.project_id;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetProjectName { project_id, name })
        .primary_object(project_id)
        .finish()
}

/// Build the batch that replaces the entire project rules list (guards the
/// rules root object).
pub fn build_set_project_rules(
    model: &DesignModel,
    provenance: WriteProvenance,
    rules: Vec<serde_json::Value>,
) -> Result<PreparedWrite, EngineError> {
    let rules_root_id = rules_root_id(
        model,
        "project rules root is missing uuid; run project new migration first",
    )?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetProjectRules {
            rules_root_id,
            rules,
        })
        .primary_object(rules_root_id)
        .finish()
}

/// Build the batch that appends one project rule. The rule payload must
/// carry its own `uuid`; returns the prepared write and that rule id.
///
/// Creation is not an existing-rule mutation: no revision guard is inserted
/// (matching the historical CLI batch shape).
pub fn build_create_project_rule(
    model: &DesignModel,
    provenance: WriteProvenance,
    rule: serde_json::Value,
) -> Result<(PreparedWrite, Uuid), EngineError> {
    let rule_id = rule_uuid(&rule)?;
    let rules_root_id = rules_root_id(
        model,
        "rules root is missing uuid; run project new migration first",
    )?;
    let prepared = BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateProjectRule {
            rules_root_id,
            rule_id,
            rule,
        })
        .primary_object(rules_root_id)
        .finish()?;
    Ok((prepared, rule_id))
}

/// Build the batch that replaces one existing project rule (identified by
/// the `uuid` inside the rule payload; guards the rules root object).
pub fn build_set_project_rule(
    model: &DesignModel,
    provenance: WriteProvenance,
    rule: serde_json::Value,
) -> Result<(PreparedWrite, Uuid), EngineError> {
    let rule_id = rule_uuid(&rule)?;
    let rules_root_id = rules_root_id(
        model,
        "rules root is missing uuid; run project new migration first",
    )?;
    if !current_rules(model)?
        .iter()
        .any(|existing| rule_uuid(existing).ok() == Some(rule_id))
    {
        return Err(EngineError::Validation(format!(
            "project rule {rule_id} not found"
        )));
    }
    let prepared = BatchComposer::compose(model, provenance)
        .push_op(Operation::SetProjectRule {
            rules_root_id,
            rule_id,
            rule,
        })
        .primary_object(rules_root_id)
        .finish()?;
    Ok((prepared, rule_id))
}

/// Build the batch that deletes one existing project rule (guards the rules
/// root object). Returns the prepared write and the pre-delete rule payload
/// so callers can report on what was removed without re-resolving.
pub fn build_delete_project_rule(
    model: &DesignModel,
    provenance: WriteProvenance,
    rule_id: Uuid,
) -> Result<(PreparedWrite, serde_json::Value), EngineError> {
    let rules_root_id = rules_root_id(
        model,
        "rules root is missing uuid; run project new migration first",
    )?;
    let rule = current_rules(model)?
        .iter()
        .find(|existing| rule_uuid(existing).ok() == Some(rule_id))
        .cloned()
        .ok_or_else(|| EngineError::Validation(format!("project rule {rule_id} not found")))?;
    let prepared = BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteProjectRule {
            rules_root_id,
            rule_id,
            rule: rule.clone(),
        })
        .primary_object(rules_root_id)
        .finish()?;
    Ok((prepared, rule))
}

/// Extract the `uuid` a project rule payload must carry.
pub fn rule_uuid(rule: &serde_json::Value) -> Result<Uuid, EngineError> {
    let uuid = rule
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("project rule missing uuid".to_string()))?;
    Uuid::parse_str(uuid)
        .map_err(|error| EngineError::Validation(format!("invalid project rule uuid: {uuid}: {error}")))
}

fn rules_root_value(model: &DesignModel) -> Result<serde_json::Value, EngineError> {
    model.materialized_source_shard_value(SourceShardKind::RulesRoot)
}

fn rules_root_id(model: &DesignModel, missing_message: &str) -> Result<Uuid, EngineError> {
    rules_root_value(model)?
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .and_then(|uuid| Uuid::parse_str(uuid).ok())
        .ok_or_else(|| EngineError::Validation(missing_message.to_string()))
}

fn current_rules(model: &DesignModel) -> Result<Vec<serde_json::Value>, EngineError> {
    Ok(rules_root_value(model)?
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default())
}

// ---------------------------------------------------------------------------
// Verb registry entries (see `super::registry`): JSON-params adapters over
// the builders above. Verb ids extend the public `datum.project.*` taxonomy.
// ---------------------------------------------------------------------------

use super::registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params};

/// Project-family verbs, registered by [`super::registry::native_write_verbs`].
pub(super) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.project.create_rule",
        build: verb_create_rule,
    },
    NativeWriteVerb {
        id: "datum.project.delete_rule",
        build: verb_delete_rule,
    },
    NativeWriteVerb {
        id: "datum.project.set_name",
        build: verb_set_name,
    },
    NativeWriteVerb {
        id: "datum.project.set_rule",
        build: verb_set_rule,
    },
    NativeWriteVerb {
        id: "datum.project.set_rules",
        build: verb_set_rules,
    },
];

fn verb_set_name(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    #[derive(serde::Deserialize)]
    struct Params {
        name: String,
    }
    let params: Params = parse_verb_params("datum.project.set_name", params)?;
    build_set_project_name(context.model, provenance, &params.name)
}

fn verb_set_rules(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    #[derive(serde::Deserialize)]
    struct Params {
        rules: Vec<serde_json::Value>,
    }
    let params: Params = parse_verb_params("datum.project.set_rules", params)?;
    build_set_project_rules(context.model, provenance, params.rules)
}

fn verb_create_rule(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    #[derive(serde::Deserialize)]
    struct Params {
        rule: serde_json::Value,
    }
    let params: Params = parse_verb_params("datum.project.create_rule", params)?;
    build_create_project_rule(context.model, provenance, params.rule)
        .map(|(prepared, _rule_id)| prepared)
}

fn verb_set_rule(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    #[derive(serde::Deserialize)]
    struct Params {
        rule: serde_json::Value,
    }
    let params: Params = parse_verb_params("datum.project.set_rule", params)?;
    build_set_project_rule(context.model, provenance, params.rule)
        .map(|(prepared, _rule_id)| prepared)
}

fn verb_delete_rule(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    #[derive(serde::Deserialize)]
    struct Params {
        rule_id: Uuid,
    }
    let params: Params = parse_verb_params("datum.project.delete_rule", params)?;
    build_delete_project_rule(context.model, provenance, params.rule_id)
        .map(|(prepared, _rule)| prepared)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::context::commit_prepared;
    use super::super::genesis::{GenesisSpec, bootstrap_native_project};
    use super::super::test_support::temp_project_root;
    use super::*;
    use crate::substrate::{CommitSource, ObjectRevision, ProjectResolver};

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, "project facade test")
    }

    /// Bootstrap a real genesis project and resolve it — the same scaffold
    /// every CLI project mutation starts from.
    fn resolved_genesis_project(label: &str) -> (PathBuf, DesignModel, Uuid) {
        let root = temp_project_root(label);
        let report = bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Project Facade Fixture".to_string(),
                existing_ids: None,
            },
        )
        .expect("genesis should succeed");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("genesis project should resolve");
        (root, model, report.rules_uuid)
    }

    /// Mirrors the CLI regression fixture in
    /// `crates/cli/src/main_tests_project_rules.rs`.
    fn clearance_rule(rule_id: Uuid, name: &str, min_nm: i64) -> serde_json::Value {
        serde_json::json!({
            "uuid": rule_id,
            "name": name,
            "scope": "All",
            "priority": 10,
            "enabled": true,
            "rule_type": "clearance_copper",
            "params": { "min_nm": min_nm }
        })
    }

    #[test]
    fn set_project_name_guards_project_root_and_commits() {
        let (root, mut model, _rules_id) = resolved_genesis_project("project_set_name");
        let project_id = model.project.project_id;

        let prepared =
            build_set_project_name(&model, test_provenance(), "  Renamed Project  ")
                .expect("set name should build");
        assert_eq!(prepared.primary_object_id, Some(project_id));
        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: project_id,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetProjectName {
                    project_id,
                    name: "Renamed Project".to_string(),
                },
            ]
        );

        commit_prepared(&mut model, &root, prepared).expect("set name should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("renamed project should resolve");
        assert_eq!(model.project.name, "Renamed Project");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn set_project_name_rejects_empty_name() {
        let (root, model, _rules_id) = resolved_genesis_project("project_empty_name");
        let error = build_set_project_name(&model, test_provenance(), "   ")
            .expect_err("empty name should fail");
        assert!(error.to_string().contains("project name must not be empty"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn create_project_rule_commits_without_guard_and_returns_rule_id() {
        let (root, mut model, rules_root_id) = resolved_genesis_project("project_create_rule");
        let rule_id = Uuid::new_v4();

        let (prepared, extracted_id) = build_create_project_rule(
            &model,
            test_provenance(),
            clearance_rule(rule_id, "Authored Clearance", 175_000),
        )
        .expect("create rule should build");
        assert_eq!(extracted_id, rule_id);
        assert_eq!(prepared.primary_object_id, Some(rules_root_id));
        // Creation matches the historical CLI batch shape: no revision guard.
        assert_eq!(prepared.batch.operations.len(), 1);
        assert!(matches!(
            &prepared.batch.operations[0],
            Operation::CreateProjectRule { rules_root_id: id, rule_id: op_rule_id, .. }
                if *id == rules_root_id && *op_rule_id == rule_id
        ));

        commit_prepared(&mut model, &root, prepared).expect("create rule should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        let rules = model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::RulesRoot)
            .expect("rules root should materialize");
        assert_eq!(rules["rules"].as_array().expect("rules array").len(), 1);
        assert_eq!(rules["rules"][0]["uuid"], rule_id.to_string());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn set_project_rules_and_set_project_rule_guard_rules_root() {
        let (root, mut model, rules_root_id) = resolved_genesis_project("project_set_rules");
        let rule_id = Uuid::new_v4();
        let (prepared, _) = build_create_project_rule(
            &model,
            test_provenance(),
            clearance_rule(rule_id, "Original Clearance", 150_000),
        )
        .expect("create rule should build");
        commit_prepared(&mut model, &root, prepared).expect("create rule should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");

        let (prepared, extracted_id) = build_set_project_rule(
            &model,
            test_provenance(),
            clearance_rule(rule_id, "Updated Clearance", 225_000),
        )
        .expect("set rule should build");
        assert_eq!(extracted_id, rule_id);
        assert_eq!(prepared.batch.operations.len(), 2);
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == rules_root_id
        ));

        let replacement = build_set_project_rules(
            &model,
            test_provenance(),
            vec![clearance_rule(rule_id, "Replaced Clearance", 200_000)],
        )
        .expect("set rules should build");
        assert_eq!(replacement.primary_object_id, Some(rules_root_id));
        assert!(matches!(
            replacement.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == rules_root_id
        ));
        assert!(matches!(
            &replacement.batch.operations[1],
            Operation::SetProjectRules { rules_root_id: id, rules } if *id == rules_root_id && rules.len() == 1
        ));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn delete_project_rule_returns_previous_payload() {
        let (root, mut model, rules_root_id) = resolved_genesis_project("project_delete_rule");
        let rule_id = Uuid::new_v4();
        let payload = clearance_rule(rule_id, "Doomed Clearance", 125_000);
        let (prepared, _) = build_create_project_rule(&model, test_provenance(), payload.clone())
            .expect("create rule should build");
        commit_prepared(&mut model, &root, prepared).expect("create rule should commit");
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");

        let (prepared, previous) =
            build_delete_project_rule(&model, test_provenance(), rule_id)
                .expect("delete rule should build");
        assert_eq!(previous["uuid"], rule_id.to_string());
        assert_eq!(prepared.primary_object_id, Some(rules_root_id));
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == rules_root_id
        ));
        assert!(matches!(
            &prepared.batch.operations[1],
            Operation::DeleteProjectRule { rule_id: id, .. } if *id == rule_id
        ));

        commit_prepared(&mut model, &root, prepared).expect("delete rule should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        let rules = model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::RulesRoot)
            .expect("rules root should materialize");
        assert_eq!(rules["rules"].as_array().expect("rules array").len(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn rule_builders_reject_missing_rules_and_bad_payloads() {
        let (root, model, _rules_root_id) = resolved_genesis_project("project_rule_errors");

        let missing = Uuid::new_v4();
        let error = build_set_project_rule(
            &model,
            test_provenance(),
            clearance_rule(missing, "Ghost", 100_000),
        )
        .expect_err("set of unknown rule should fail");
        assert!(error
            .to_string()
            .contains(&format!("project rule {missing} not found")));

        let error = build_delete_project_rule(&model, test_provenance(), missing)
            .expect_err("delete of unknown rule should fail");
        assert!(error
            .to_string()
            .contains(&format!("project rule {missing} not found")));

        let error =
            build_create_project_rule(&model, test_provenance(), serde_json::json!({ "name": "x" }))
                .expect_err("rule without uuid should fail");
        assert!(error.to_string().contains("project rule missing uuid"));

        let error = build_create_project_rule(
            &model,
            test_provenance(),
            serde_json::json!({ "uuid": "not-a-uuid" }),
        )
        .expect_err("rule with invalid uuid should fail");
        assert!(error.to_string().contains("invalid project rule uuid: not-a-uuid"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
