use anyhow::Result;
use eda_engine::substrate::{ProjectResolver, SourceShardDirtyState};
use serde::Serialize;

use crate::LiveReviewRequest;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct SourceShardStatusSummary {
    pub total: usize,
    pub clean: usize,
    pub dirty: usize,
    pub missing: usize,
    pub unknown: usize,
    pub attention: Vec<SourceShardAttentionItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceShardAttentionItem {
    pub relative_path: String,
    pub kind: String,
    pub authority: String,
    pub taxon: Option<String>,
    pub dirty_state: String,
}

impl SourceShardStatusSummary {
    pub fn attention_count(&self) -> usize {
        self.dirty + self.missing + self.unknown
    }
}

pub fn load_source_shard_status(request: &LiveReviewRequest) -> Result<SourceShardStatusSummary> {
    if request.board_file.is_some() {
        return Ok(SourceShardStatusSummary::default());
    }
    let model = ProjectResolver::new(&request.project_root).resolve()?;
    let mut summary = SourceShardStatusSummary {
        total: model.source_shards.len(),
        ..SourceShardStatusSummary::default()
    };
    for shard in &model.source_shards {
        match shard.dirty_state {
            SourceShardDirtyState::Clean => summary.clean += 1,
            SourceShardDirtyState::Dirty => summary.dirty += 1,
            SourceShardDirtyState::Missing => summary.missing += 1,
            SourceShardDirtyState::Unknown => summary.unknown += 1,
        }
        if shard.dirty_state != SourceShardDirtyState::Clean {
            summary.attention.push(SourceShardAttentionItem {
                relative_path: shard.relative_path.clone(),
                kind: stable_json_name(&shard.kind),
                authority: stable_json_name(&shard.authority),
                taxon: shard.taxon.as_ref().map(stable_json_name),
                dirty_state: stable_json_name(&shard.dirty_state),
            });
        }
    }
    Ok(summary)
}

pub fn load_accepted_transaction_tip(request: &LiveReviewRequest) -> Result<Option<String>> {
    if request.board_file.is_some() {
        return Ok(None);
    }
    let model = ProjectResolver::new(&request.project_root).resolve()?;
    Ok(model
        .journal_cursor
        .applied_transaction_count
        .checked_sub(1)
        .and_then(|index| model.journal.get(index))
        .map(|transaction| transaction.transaction_id.to_string()))
}

fn stable_json_name<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
#[path = "source_shard_status_accepted_tip_tests.rs"]
mod source_shard_status_accepted_tip_tests;

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use eda_engine::substrate::{CommitProvenance, CommitSource, Operation, OperationBatch};
    use uuid::Uuid;

    use super::*;

    fn unique_project_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("fixture directory should create");
        }
        std::fs::write(path, format!("{value}\n")).expect("fixture JSON should write");
    }

    fn write_minimal_native_project(root: &Path) {
        let project_id = Uuid::new_v4();
        write_json(
            &root.join("project.json"),
            serde_json::json!({
                "schema_version": 1,
                "uuid": project_id,
                "name": "GUI Source Shard Status Demo",
                "pools": [],
                "schematic": "schematic/schematic.json",
                "board": "board/board.json",
                "rules": "rules/rules.json",
                "forward_annotation_review": {}
            }),
        );
        write_json(
            &root.join("schematic/schematic.json"),
            serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "sheets": {},
                "definitions": {},
                "instances": [],
                "variants": {},
                "waivers": [],
                "deviations": []
            }),
        );
        write_json(
            &root.join("board/board.json"),
            serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "GUI Source Shard Status Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "component_silkscreen": {},
                "component_pads": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "dimensions": {},
                "texts": {},
                "keepouts": {}
            }),
        );
        write_json(
            &root.join("rules/rules.json"),
            serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "object_revision": 0,
                "rules": []
            }),
        );
    }

    #[test]
    fn source_shard_status_counts_missing_forward_annotation_review_sidecar() {
        let root = unique_project_root("datum-gui-source-shard-status-fa-review");
        write_minimal_native_project(&root);
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should resolve before sidecar commit");
        model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v4(),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "gui-protocol-test".to_string(),
                        source: CommitSource::Cli,
                        reason: "record review sidecar".to_string(),
                    },
                    operations: vec![Operation::SetForwardAnnotationReview {
                        relative_path: ".datum/forward_annotation_review/review.json".to_string(),
                        previous_review: None,
                        review: serde_json::json!({
                            "schema_version": 1,
                            "reviews": {
                                "action-1": {
                                    "action_id": "action-1",
                                    "status": "deferred"
                                }
                            }
                        }),
                    }],
                },
            )
            .expect("review sidecar should commit");
        std::fs::remove_file(root.join(".datum/forward_annotation_review/review.json"))
            .expect("promoted review sidecar should remove");

        let summary = load_source_shard_status(&LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        })
        .expect("source-shard status should load");

        assert_eq!(summary.missing, 1);
        assert_eq!(summary.attention_count(), 1);
        assert!(summary.total >= 5);
        assert_eq!(summary.attention.len(), 1);
        assert_eq!(
            summary.attention[0].relative_path,
            ".datum/forward_annotation_review/review.json"
        );
        assert_eq!(summary.attention[0].dirty_state, "missing");
        assert_eq!(
            summary.attention[0].taxon.as_deref(),
            Some("forward_annotation_review")
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn source_shard_status_counts_missing_identity_relationship_sidecars() {
        let root = unique_project_root("datum-gui-source-shard-status-identity");
        write_minimal_native_project(&root);
        let project: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
        let project_id = Uuid::parse_str(project["uuid"].as_str().unwrap()).unwrap();
        let mut schematic: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
                .unwrap();
        let sheet_id = Uuid::new_v4();
        let sheet_path = format!("sheets/{sheet_id}.json");
        schematic["sheets"][sheet_id.to_string()] = serde_json::Value::String(sheet_path.clone());
        write_json(&root.join("schematic/schematic.json"), schematic);
        let symbol_id = Uuid::new_v4();
        let part_id = Uuid::new_v4();
        write_json(
            &root.join("schematic").join(&sheet_path),
            serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_id,
                "name": "Main",
                "symbols": {
                    symbol_id.to_string(): {
                        "uuid": symbol_id,
                        "part": part_id,
                        "entity": Uuid::new_v5(&project_id, b"entity"),
                        "gate": Uuid::new_v5(&project_id, b"gate"),
                        "lib_id": "test:R",
                        "reference": "U1",
                        "value": "OLD",
                        "fields": [],
                        "pins": [],
                        "position": { "x": 0, "y": 0 },
                        "rotation": 0,
                        "mirrored": false,
                        "unit_selection": null,
                        "display_mode": "LibraryDefault",
                        "pin_overrides": [],
                        "hidden_power_behavior": "SourceDefinedImplicit"
                    }
                },
                "wires": {},
                "junctions": {},
                "labels": {},
                "buses": {},
                "bus_entries": {},
                "ports": {},
                "noconnects": {},
                "texts": {},
                "drawings": {}
            }),
        );
        let package_id = Uuid::new_v4();
        let mut board: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
        board["packages"][package_id.to_string()] = serde_json::json!({
            "uuid": package_id,
            "part": part_id,
            "package": Uuid::new_v5(&project_id, b"package"),
            "reference": "U1",
            "value": "OLD",
            "position": { "x": 0, "y": 0 },
            "rotation": 0,
            "layer": 0,
            "locked": false
        });
        write_json(&root.join("board/board.json"), board);

        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should resolve before identity sidecar commit");
        let component_instance_id = Uuid::new_v4();
        let relationship_id = Uuid::new_v4();
        let variant_id = Uuid::new_v4();
        model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v4(),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "gui-protocol-test".to_string(),
                        source: CommitSource::Cli,
                        reason: "record identity relationship sidecars".to_string(),
                    },
                    operations: vec![
                        Operation::CreateComponentInstance {
                            component_instance_id,
                            component_instance: serde_json::json!({
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
                            }),
                        },
                        Operation::CreateRelationship {
                            relationship_id,
                            relationship: serde_json::json!({
                                "id": relationship_id,
                                "kind": "implemented_by",
                                "from": [{
                                    "object_id": symbol_id,
                                    "object_revision": 0
                                }],
                                "to": [{
                                    "object_id": package_id,
                                    "object_revision": 0
                                }],
                                "authored_intent": [],
                                "object_revision": 0
                            }),
                        },
                        Operation::CreateVariantOverlay {
                            variant_id,
                            variant: serde_json::json!({
                                "id": variant_id,
                                "name": "No U1",
                                "base_model_revision": model.model_revision,
                                "variant_revision": 0,
                                "fitted": {
                                    package_id.to_string(): "unfitted"
                                },
                                "relationship_overrides": {},
                                "property_overrides": {}
                            }),
                        },
                    ],
                },
            )
            .expect("identity relationship sidecars should commit");
        std::fs::remove_file(root.join(format!(
            ".datum/component_instances/{component_instance_id}.json"
        )))
        .expect("promoted component instance sidecar should remove");
        std::fs::remove_file(root.join(format!(".datum/relationships/{relationship_id}.json")))
            .expect("promoted relationship sidecar should remove");
        std::fs::remove_file(root.join(format!(".datum/variants/{variant_id}.json")))
            .expect("promoted variant sidecar should remove");

        let summary = load_source_shard_status(&LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        })
        .expect("source-shard status should load");

        assert_eq!(summary.missing, 3);
        assert_eq!(summary.attention_count(), 3);
        for (relative_path, kind, taxon) in [
            (
                format!(".datum/component_instances/{component_instance_id}.json"),
                "component_instance",
                "component_instance",
            ),
            (
                format!(".datum/relationships/{relationship_id}.json"),
                "relationship",
                "relationship",
            ),
            (
                format!(".datum/variants/{variant_id}.json"),
                "variant_overlay",
                "variant_overlay",
            ),
        ] {
            assert!(
                summary.attention.iter().any(|item| {
                    item.relative_path == relative_path
                        && item.kind == kind
                        && item.authority == "authored_design"
                        && item.taxon.as_deref() == Some(taxon)
                        && item.dirty_state == "missing"
                }),
                "source-shard status should expose missing {kind} sidecar"
            );
        }

        let _ = std::fs::remove_dir_all(&root);
    }
}
