use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    DerivedRelationshipStatus, DomainObject, ObjectId, Relationship, RelationshipKind,
    ResolveDiagnostic, RevisionedRef, SourceShardKind, SourceShardRef, read_json_value,
    source_shard::validate_source_shard_schema_version,
    source_shard_ref_builders::source_shard_ref_for_bytes,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipShard {
    #[serde(default = "default_relationship_shard_schema_version")]
    pub schema_version: u64,
    pub relationships: Vec<Relationship>,
}

pub const RELATIONSHIP_SHARD_SCHEMA_VERSION: u64 = 1;

fn default_relationship_shard_schema_version() -> u64 {
    RELATIONSHIP_SHARD_SCHEMA_VERSION
}

// Established multi-value signature; a tuple type alias would not improve clarity.
#[allow(clippy::type_complexity)]
pub(super) fn read_relationship_shards(
    project_root: &Path,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<ObjectId, Relationship>,
    BTreeMap<ObjectId, DerivedRelationshipStatus>,
    Vec<ResolveDiagnostic>,
) {
    let relationship_dir = project_root.join(".datum/relationships");
    let mut shards = Vec::new();
    let mut relationships = BTreeMap::new();
    let mut relationship_statuses = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&relationship_dir) else {
        return (shards, relationships, relationship_statuses, diagnostics);
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let relative_path = format!(".datum/relationships/{filename}");
        let path = project_root.join(&relative_path);
        match read_relationship_shard(path, relative_path) {
            Ok((shard, relationship_shard)) => {
                insert_relationships(
                    &shard,
                    relationship_shard.relationships,
                    objects,
                    &mut relationships,
                    &mut relationship_statuses,
                    &mut diagnostics,
                );
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, relationships, relationship_statuses, diagnostics)
}

fn read_relationship_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, RelationshipShard), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_relationship_shard".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_relationship_shard".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    validate_source_shard_schema_version(
        &SourceShardKind::Relationship,
        &relative_path,
        schema_version,
    )
    .map_err(|error| ResolveDiagnostic {
        code: "invalid_relationship_shard".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let shard = source_shard_ref_for_bytes(
        SourceShardKind::Relationship,
        path,
        relative_path,
        schema_version,
        &bytes,
        "invalid_relationship_shard",
    )?;
    let relationship_shard =
        serde_json::from_value::<RelationshipShard>(value).map_err(|error| ResolveDiagnostic {
            code: "invalid_relationship_shard".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        })?;
    if relationship_shard.schema_version != RELATIONSHIP_SHARD_SCHEMA_VERSION {
        return Err(ResolveDiagnostic {
            code: "invalid_relationship_shard".to_string(),
            message: format!(
                "unsupported RelationshipShard schema_version {}",
                relationship_shard.schema_version
            ),
            path: Some(shard.path.clone()),
        });
    }
    Ok((shard, relationship_shard))
}

fn insert_relationships(
    shard: &SourceShardRef,
    input: Vec<Relationship>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    relationships: &mut BTreeMap<ObjectId, Relationship>,
    relationship_statuses: &mut BTreeMap<ObjectId, DerivedRelationshipStatus>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) {
    let mut ids = BTreeSet::new();
    for relationship in input {
        if !ids.insert(relationship.id) || relationships.contains_key(&relationship.id) {
            diagnostics.push(ResolveDiagnostic {
                code: "relationship_duplicate_id".to_string(),
                message: format!("duplicate relationship id {}", relationship.id),
                path: Some(shard.path.clone()),
            });
            continue;
        }
        let status = derive_relationship_status(&relationship, objects);
        objects.insert(
            relationship.id,
            DomainObject {
                object_id: relationship.id,
                object_revision: relationship.object_revision,
                source_shard_id: shard.shard_id,
                domain: "relationship".to_string(),
                kind: "relationship".to_string(),
            },
        );
        relationship_statuses.insert(relationship.id, status);
        relationships.insert(relationship.id, relationship);
    }
}

fn derive_relationship_status(
    relationship: &Relationship,
    objects: &BTreeMap<ObjectId, DomainObject>,
) -> DerivedRelationshipStatus {
    match relationship.kind {
        RelationshipKind::Pending => DerivedRelationshipStatus::PendingImplementation,
        RelationshipKind::Mismatch => DerivedRelationshipStatus::UnresolvedMismatch,
        RelationshipKind::ImplementedBy
        | RelationshipKind::BoardOnly
        | RelationshipKind::SchematicOnly
        | RelationshipKind::ReverseEngineered => {
            if refs_are_resolved(&relationship.from, objects)
                && refs_are_resolved(&relationship.to, objects)
            {
                DerivedRelationshipStatus::Implemented
            } else {
                DerivedRelationshipStatus::UnresolvedMismatch
            }
        }
    }
}

fn refs_are_resolved(refs: &[RevisionedRef], objects: &BTreeMap<ObjectId, DomainObject>) -> bool {
    refs.iter().all(|reference| {
        objects
            .get(&reference.object_id)
            .map(|object| object.object_revision == reference.object_revision)
            .unwrap_or(false)
    })
}
