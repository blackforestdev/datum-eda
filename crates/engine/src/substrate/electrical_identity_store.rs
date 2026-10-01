//! Electrical identity persistence through the canonical shard/journal hooks.
use std::collections::BTreeMap;
use std::path::Path;
use uuid::Uuid;

use super::{
    DesignModel, DomainObject, ELECTRICAL_IDENTITY_SCHEMA_VERSION, ElectricalIdentityRecord,
    ElectricalIdentityShard, EngineError, Operation, OperationBatch, SourceShardDirtyState,
    SourceShardKind, SourceShardRef, SourceShardTaxon,
    journal::{StagedShardWrite, stage_new_shard_write},
    source_shard_ref_builders::source_shard_ref_for_bytes,
};

pub(super) fn relative_path(id: Uuid) -> String {
    format!(".datum/electrical_identities/{id}.json")
}

pub(super) fn write(operation: &Operation) -> Option<(Uuid, &ElectricalIdentityRecord, bool)> {
    match operation {
        Operation::CreateElectricalIdentity { record } => Some((record.id, record, false)),
        Operation::SetElectricalIdentity { record, .. } => Some((record.id, record, false)),
        Operation::DeleteElectricalIdentity { record } => Some((record.id, record, true)),
        _ => None,
    }
}

pub(super) fn wrapper(record: &ElectricalIdentityRecord) -> Result<serde_json::Value, EngineError> {
    Ok(serde_json::to_value(ElectricalIdentityShard {
        schema_version: ELECTRICAL_IDENTITY_SCHEMA_VERSION,
        record: record.clone(),
    })?)
}

pub(super) fn insert_object(model: &mut DesignModel, record: &ElectricalIdentityRecord) {
    let path = relative_path(record.id);
    let shard_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:source-shard:{path}").as_bytes(),
    );
    model.objects.insert(record.id, object(record, shard_id));
    if !model.source_shards.iter().any(|s| s.relative_path == path) {
        model.source_shards.push(SourceShardRef {
            shard_id,
            kind: SourceShardKind::ElectricalIdentity,
            taxon: Some(SourceShardTaxon::ElectricalIdentity),
            path: path.clone().into(),
            relative_path: path,
            authority: super::SourceShardAuthority::AuthoredDesign,
            dirty_state: SourceShardDirtyState::Clean,
            schema_version: Some(ELECTRICAL_IDENTITY_SCHEMA_VERSION),
            content_hash: String::new(),
        });
    }
}

pub(super) fn object(record: &ElectricalIdentityRecord, shard_id: Uuid) -> DomainObject {
    DomainObject {
        object_id: record.id,
        object_revision: record.object_revision,
        source_shard_id: shard_id,
        domain: "electrical_identity".into(),
        kind: match record.identity {
            super::ElectricalIdentity::Net { .. } => "net_identity",
            super::ElectricalIdentity::Bus { .. } => "bus_identity",
            super::ElectricalIdentity::NetRelationship { .. } => "net_relationship",
            super::ElectricalIdentity::BusInterface { .. } => "bus_interface",
        }
        .into(),
    }
}

pub(super) fn stage(
    root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
) -> Result<Option<StagedShardWrite>, EngineError> {
    let Some((id, record, delete)) = write(operation) else {
        return Ok(None);
    };
    let path = relative_path(id);
    if delete {
        return Ok(Some(StagedShardWrite {
            destination: root.join(&path),
            staged: None,
            kind: SourceShardKind::ElectricalIdentity,
            relative_path: path,
            content_hash: String::new(),
            schema_version: None,
            delete: true,
        }));
    }
    stage_new_shard_write(
        root,
        batch,
        SourceShardKind::ElectricalIdentity,
        &path,
        &wrapper(record)?,
    )
    .map(Some)
}

pub(super) fn inverse(operation: &Operation, inverse: &mut Vec<Operation>) {
    match operation {
        Operation::CreateElectricalIdentity { record } => {
            inverse.push(Operation::DeleteElectricalIdentity {
                record: record.clone(),
            })
        }
        Operation::DeleteElectricalIdentity { record } => {
            inverse.push(Operation::CreateElectricalIdentity {
                record: record.clone(),
            })
        }
        Operation::SetElectricalIdentity { previous, record } => {
            inverse.push(Operation::SetElectricalIdentity {
                previous: Box::new(record.clone()),
                record: previous.as_ref().clone(),
            })
        }
        _ => {}
    }
}

pub(super) fn apply_shard(
    kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    if kind != &SourceShardKind::ElectricalIdentity {
        return Ok(false);
    }
    let Some((id, record, delete)) = write(operation) else {
        return Ok(false);
    };
    if value
        .get("record")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        != Some(id.to_string().as_str())
    {
        return Ok(false);
    }
    *value = if delete {
        serde_json::Value::Null
    } else {
        wrapper(record)?
    };
    Ok(true)
}

pub(super) fn read(
    root: &Path,
    objects: &mut BTreeMap<Uuid, DomainObject>,
) -> Result<
    (
        Vec<SourceShardRef>,
        BTreeMap<Uuid, ElectricalIdentityRecord>,
    ),
    EngineError,
> {
    let directory = root.join(".datum/electrical_identities");
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok((Vec::new(), BTreeMap::new()));
        }
        Err(e) => return Err(e.into()),
    };
    let mut paths = entries
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    let mut shards = Vec::new();
    let mut records = BTreeMap::new();
    for path in paths {
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let bytes = std::fs::read(&path)?;
        let shard: ElectricalIdentityShard = serde_json::from_slice(&bytes)?;
        if shard.schema_version != ELECTRICAL_IDENTITY_SCHEMA_VERSION
            || path.file_stem().and_then(|s| s.to_str())
                != Some(shard.record.id.to_string().as_str())
            || shard.record.id.is_nil()
        {
            return Err(EngineError::Validation(
                "invalid electrical identity schema/id".into(),
            ));
        }
        if objects.contains_key(&shard.record.id) {
            return Err(EngineError::Validation(
                "electrical identity collides with source object".into(),
            ));
        }
        let source = source_shard_ref_for_bytes(
            SourceShardKind::ElectricalIdentity,
            path,
            relative_path(shard.record.id),
            Some(shard.schema_version),
            &bytes,
            "invalid_electrical_identity",
        )
        .map_err(|e| EngineError::Validation(e.message))?;
        objects.insert(shard.record.id, object(&shard.record, source.shard_id));
        records.insert(shard.record.id, shard.record);
        shards.push(source);
    }
    Ok((shards, records))
}
