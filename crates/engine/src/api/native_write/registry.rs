//! Verb registry over the native write facade.
//!
//! Maps stable `datum.*` verb ids to family builders so verb-addressed
//! surfaces (the daemon's `native.write`, and later MCP) can dispatch JSON
//! params onto the exact same build-only builders every other caller uses.
//! Entries live as small `VERBS` tables colocated with their family modules
//! ([`super::project`], [`super::waivers`]); this module only aggregates,
//! sorts, and looks them up. Verb ids follow the public `datum.*` tool
//! taxonomy (`mcp-server/tools_catalog_datum.py`) where a tool already
//! exists, and extend it consistently where none does.

use std::path::Path;
use std::sync::OnceLock;

use serde::de::DeserializeOwned;

use crate::error::EngineError;
use crate::substrate::DesignModel;

use super::context::{PreparedWrite, WriteProvenance};

/// Read-only context a verb adapter builds against: the resolved model and
/// the project root it was resolved from.
pub struct NativeWriteContext<'a> {
    pub model: &'a DesignModel,
    pub project_root: &'a Path,
}

/// A verb adapter: parse JSON params, call the family builder, return the
/// prepared (uncommitted) write.
pub type BuildFn = fn(
    &NativeWriteContext<'_>,
    WriteProvenance,
    serde_json::Value,
) -> Result<PreparedWrite, EngineError>;

/// One registered native write verb.
#[derive(Clone, Copy)]
pub struct NativeWriteVerb {
    /// Stable public id in the `datum.*` taxonomy, e.g.
    /// `datum.project.set_name`.
    pub id: &'static str,
    pub build: BuildFn,
}

/// All registered verbs, sorted by id. Duplicate ids are a construction
/// defect and panic at first use (also locked by a unit test).
pub fn native_write_verbs() -> &'static [NativeWriteVerb] {
    static VERBS: OnceLock<Vec<NativeWriteVerb>> = OnceLock::new();
    VERBS.get_or_init(|| {
        let mut verbs: Vec<NativeWriteVerb> = Vec::new();
        verbs.extend_from_slice(super::project::VERBS);
        verbs.extend_from_slice(super::waivers::VERBS);
        verbs.sort_by(|a, b| a.id.cmp(b.id));
        for pair in verbs.windows(2) {
            assert_ne!(
                pair[0].id, pair[1].id,
                "duplicate native write verb id: {}",
                pair[0].id
            );
        }
        verbs
    })
}

/// Look up one verb by its stable id.
pub fn find_native_write_verb(id: &str) -> Option<&'static NativeWriteVerb> {
    let verbs = native_write_verbs();
    verbs
        .binary_search_by(|verb| verb.id.cmp(id))
        .ok()
        .map(|index| &verbs[index])
}

/// Shared params parser for verb adapters: deserialize the JSON params of
/// `verb_id`, mapping malformed input to a validation error naming the verb.
pub(super) fn parse_verb_params<T: DeserializeOwned>(
    verb_id: &str,
    params: serde_json::Value,
) -> Result<T, EngineError> {
    serde_json::from_value(params)
        .map_err(|error| EngineError::Validation(format!("invalid params for {verb_id}: {error}")))
}

#[cfg(test)]
mod tests {
    use super::super::context::commit_prepared;
    use super::super::genesis::{GenesisSpec, bootstrap_native_project};
    use super::super::test_support::temp_project_root;
    use super::*;
    use crate::substrate::{CommitSource, ProjectResolver, SourceShardKind};
    use std::path::PathBuf;

    fn test_provenance(reason: &str) -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, reason)
    }

    fn resolved_genesis_project(label: &str) -> (PathBuf, DesignModel) {
        let root = temp_project_root(label);
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Registry Fixture".to_string(),
                existing_ids: None,
            },
        )
        .expect("genesis should succeed");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("genesis project should resolve");
        (root, model)
    }

    #[test]
    fn verb_ids_are_sorted_and_unique() {
        let verbs = native_write_verbs();
        assert!(!verbs.is_empty());
        for pair in verbs.windows(2) {
            assert!(
                pair[0].id < pair[1].id,
                "verb ids must be strictly ascending: {} !< {}",
                pair[0].id,
                pair[1].id
            );
        }
    }

    #[test]
    fn registry_covers_the_two_pilot_families() {
        let ids: Vec<&str> = native_write_verbs().iter().map(|verb| verb.id).collect();
        assert_eq!(
            ids,
            vec![
                "datum.check.accept_deviation",
                "datum.check.waive",
                "datum.project.create_rule",
                "datum.project.delete_rule",
                "datum.project.set_name",
                "datum.project.set_rule",
                "datum.project.set_rules",
            ]
        );
    }

    #[test]
    fn find_native_write_verb_hits_and_misses() {
        assert!(find_native_write_verb("datum.project.set_name").is_some());
        assert!(find_native_write_verb("datum.check.waive").is_some());
        assert!(find_native_write_verb("datum.project.explode").is_none());
        assert!(find_native_write_verb("").is_none());
    }

    #[test]
    fn set_name_verb_round_trips_json_params_to_committed_project() {
        let (root, mut model) = resolved_genesis_project("registry_set_name");
        let verb = find_native_write_verb("datum.project.set_name").expect("verb should exist");
        let context = NativeWriteContext {
            model: &model,
            project_root: &root,
        };

        let prepared = (verb.build)(
            &context,
            test_provenance("registry set-name round trip"),
            serde_json::json!({ "name": "  Registry Renamed  " }),
        )
        .expect("verb build should succeed");
        assert_eq!(prepared.primary_object_id, Some(model.project.project_id));

        commit_prepared(&mut model, &root, prepared).expect("commit should succeed");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("renamed project should resolve");
        assert_eq!(model.project.name, "Registry Renamed");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn rule_verbs_round_trip_create_set_delete() {
        let (root, mut model) = resolved_genesis_project("registry_rules");
        let rule_id = uuid::Uuid::new_v4();
        let rule = |name: &str, min_nm: i64| {
            serde_json::json!({
                "uuid": rule_id,
                "name": name,
                "scope": "All",
                "priority": 10,
                "enabled": true,
                "rule_type": "clearance_copper",
                "params": { "min_nm": min_nm }
            })
        };
        let rules_len = |model: &DesignModel| {
            model
                .materialized_source_shard_value(SourceShardKind::RulesRoot)
                .expect("rules root should materialize")["rules"]
                .as_array()
                .expect("rules array")
                .len()
        };

        let create = find_native_write_verb("datum.project.create_rule").expect("verb");
        let prepared = (create.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            test_provenance("registry create rule"),
            serde_json::json!({ "rule": rule("Registry Clearance", 150_000) }),
        )
        .expect("create rule should build");
        commit_prepared(&mut model, &root, prepared).expect("create rule should commit");
        let mut model = ProjectResolver::new(&root).resolve().expect("resolve");
        assert_eq!(rules_len(&model), 1);

        let set = find_native_write_verb("datum.project.set_rule").expect("verb");
        let prepared = (set.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            test_provenance("registry set rule"),
            serde_json::json!({ "rule": rule("Registry Clearance Updated", 225_000) }),
        )
        .expect("set rule should build");
        commit_prepared(&mut model, &root, prepared).expect("set rule should commit");
        model = ProjectResolver::new(&root).resolve().expect("resolve");
        let rules = model
            .materialized_source_shard_value(SourceShardKind::RulesRoot)
            .expect("rules root should materialize");
        assert_eq!(rules["rules"][0]["name"], "Registry Clearance Updated");

        let delete = find_native_write_verb("datum.project.delete_rule").expect("verb");
        let prepared = (delete.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            test_provenance("registry delete rule"),
            serde_json::json!({ "rule_id": rule_id }),
        )
        .expect("delete rule should build");
        commit_prepared(&mut model, &root, prepared).expect("delete rule should commit");
        let model = ProjectResolver::new(&root).resolve().expect("resolve");
        assert_eq!(rules_len(&model), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn set_rules_verb_replaces_rule_list() {
        let (root, mut model) = resolved_genesis_project("registry_set_rules");
        let verb = find_native_write_verb("datum.project.set_rules").expect("verb");
        let prepared = (verb.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            test_provenance("registry set rules"),
            serde_json::json!({
                "rules": [{
                    "uuid": uuid::Uuid::new_v4(),
                    "name": "Wholesale",
                    "scope": "All",
                    "priority": 5,
                    "enabled": true,
                    "rule_type": "clearance_copper",
                    "params": { "min_nm": 100_000 }
                }]
            }),
        )
        .expect("set rules should build");
        commit_prepared(&mut model, &root, prepared).expect("set rules should commit");
        let model = ProjectResolver::new(&root).resolve().expect("resolve");
        let rules = model
            .materialized_source_shard_value(SourceShardKind::RulesRoot)
            .expect("rules root should materialize");
        assert_eq!(rules["rules"].as_array().expect("rules array").len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn waive_verb_defaults_schematic_root_and_commits() {
        let (root, mut model) = resolved_genesis_project("registry_waive");
        let verb = find_native_write_verb("datum.check.waive").expect("verb should exist");

        let prepared = (verb.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            test_provenance("waive check finding erc:unconnected:reg123"),
            serde_json::json!({
                "domain": "erc",
                "fingerprint": "erc:unconnected:reg123",
                "rationale": "reviewed via registry",
                "created_by": "reviewer"
            }),
        )
        .expect("waive build should succeed");
        let waiver_id = prepared
            .primary_object_id
            .expect("waiver id should be the primary object");

        commit_prepared(&mut model, &root, prepared).expect("waiver should commit");
        assert!(
            model.objects.contains_key(&waiver_id),
            "committed waiver should be a resolved domain object"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn accept_deviation_verb_commits_through_journaled_path() {
        let (root, mut model) = resolved_genesis_project("registry_deviation");
        let verb =
            find_native_write_verb("datum.check.accept_deviation").expect("verb should exist");

        let prepared = (verb.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            test_provenance("accept check finding deviation drc:clearance:reg456"),
            serde_json::json!({
                "domain": "drc",
                "fingerprint": "drc:clearance:reg456",
                "rationale": "accepted via registry",
                "accepted_by": "approver"
            }),
        )
        .expect("deviation build should succeed");
        let deviation_id = prepared
            .primary_object_id
            .expect("deviation id should be the primary object");

        commit_prepared(&mut model, &root, prepared).expect("deviation should commit");
        assert!(
            model.objects.contains_key(&deviation_id),
            "committed deviation should be a resolved domain object"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn verb_adapters_reject_malformed_params_naming_the_verb() {
        let (root, model) = resolved_genesis_project("registry_bad_params");
        let verb = find_native_write_verb("datum.project.set_name").expect("verb should exist");
        let error = (verb.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            test_provenance("bad params"),
            serde_json::json!({ "nom": "typo" }),
        )
        .expect_err("missing name should fail");
        assert!(
            error
                .to_string()
                .contains("invalid params for datum.project.set_name"),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
