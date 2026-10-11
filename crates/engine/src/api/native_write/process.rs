//! Build-only PM061 authoring through the shared guarded journal facade.
use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use uuid::Uuid;

use crate::error::EngineError;
use crate::pool::process::{ProcessBasisSelector, ProcessProfile, ProcessSourceCapability};
use crate::substrate::{
    DesignModel, Operation, SourceShardKind,
    process_operations::{ProcessOperation, capability_from_board},
};

use super::{BatchComposer, PreparedWrite, WriteProvenance};

#[cfg(test)]
mod tests;

pub fn build_set_process_profile(
    model: &DesignModel,
    provenance: WriteProvenance,
    profile: ProcessProfile,
) -> Result<PreparedWrite, EngineError> {
    profile.basis.validate()?;
    if profile.name.trim().is_empty() {
        return Err(refusal("profile name is empty"));
    }
    let previous = profiles(model)?;
    let mut next = previous.clone().unwrap_or_default();
    next.insert(profile.uuid, profile);
    build_profiles(model, provenance, previous, Some(next))
}

pub fn build_delete_process_profile(
    model: &DesignModel,
    provenance: WriteProvenance,
    profile_id: Uuid,
) -> Result<PreparedWrite, EngineError> {
    let previous = profiles(model)?;
    let mut next = previous
        .clone()
        .ok_or_else(|| refusal("missing process profile collection"))?;
    if next.remove(&profile_id).is_none() {
        return Err(refusal("missing process profile"));
    }
    build_profiles(model, provenance, previous, Some(next))
}

pub fn profiles(
    model: &DesignModel,
) -> Result<Option<BTreeMap<Uuid, ProcessProfile>>, EngineError> {
    let value = model.materialized_source_shard_value(SourceShardKind::ProjectManifest)?;
    value
        .get("process_profiles")
        .map(|v| serde_json::from_value(v.clone()).map_err(EngineError::from))
        .transpose()
}

fn build_profiles(
    model: &DesignModel,
    provenance: WriteProvenance,
    previous: Option<BTreeMap<Uuid, ProcessProfile>>,
    profiles: Option<BTreeMap<Uuid, ProcessProfile>>,
) -> Result<PreparedWrite, EngineError> {
    let project_id = model.project.project_id;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::Process {
            operation: ProcessOperation::SetProfiles {
                project_id,
                previous,
                profiles,
            },
        })
        .primary_object(project_id)
        .finish()
}

pub fn pinned_profile_selector(
    model: &DesignModel,
    profile_id: Uuid,
) -> Result<ProcessBasisSelector, EngineError> {
    let profiles = profiles(model)?.ok_or_else(|| refusal("missing process profile collection"))?;
    let profile = profiles
        .get(&profile_id)
        .ok_or_else(|| refusal("missing process profile"))?;
    let object = model
        .objects
        .get(&profile_id)
        .ok_or_else(|| refusal("missing canonical process profile identity"))?;
    let value_hash = crate::substrate::sha256_hex(
        crate::ir::serialization::to_json_deterministic(&serde_json::to_value(profile)?)?
            .as_bytes(),
    );
    Ok(ProcessBasisSelector::ProjectProfile {
        profile_id,
        expected_object_revision: object.object_revision.0,
        expected_value_hash: value_hash,
    })
}

pub fn build_adopt_process_capability(
    model: &DesignModel,
    root: &Path,
    provenance: WriteProvenance,
    capability: ProcessSourceCapability,
) -> Result<PreparedWrite, EngineError> {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
        .ok_or_else(|| refusal("missing BoardRoot"))?;
    let bytes = std::fs::read(root.join(&shard.relative_path))?;
    if crate::substrate::sha256_hex(&bytes) != shard.content_hash {
        return Err(refusal("stale BoardRoot bytes"));
    }
    let value: Value = serde_json::from_slice(&bytes)?;
    if capability_from_board(&value)?.is_some() {
        return Err(refusal("process capability already adopted"));
    }
    capability.validate(&serde_json::from_value(value["stackup"].clone())?)?;
    let board_id = serde_json::from_value(value["uuid"].clone())?;
    let preimage =
        String::from_utf8(bytes).map_err(|_| refusal("BoardRoot byte preimage is not UTF-8"))?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::Process {
            operation: ProcessOperation::AdoptCapability {
                board_id,
                preimage,
                capability,
            },
        })
        .primary_object(board_id)
        .finish()
}

pub fn build_set_process_associations(
    model: &DesignModel,
    provenance: WriteProvenance,
    capability: ProcessSourceCapability,
) -> Result<PreparedWrite, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let previous = capability_from_board(&board)?
        .ok_or_else(|| refusal("process capability has not been adopted"))?;
    capability.validate(&serde_json::from_value(board["stackup"].clone())?)?;
    let board_id = serde_json::from_value(board["uuid"].clone())?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::Process {
            operation: ProcessOperation::SetAssociations {
                board_id,
                previous,
                capability,
            },
        })
        .primary_object(board_id)
        .finish()
}

fn refusal(reason: &str) -> EngineError {
    EngineError::Validation(format!("process authoring: {reason}"))
}

pub fn build_select_process_basis(
    model: &DesignModel,
    provenance: WriteProvenance,
    footprint_id: Uuid,
    selector: ProcessBasisSelector,
) -> Result<PreparedWrite, EngineError> {
    let object = model
        .objects
        .get(&footprint_id)
        .filter(|o| o.domain == "pool" && o.kind == "footprints")
        .ok_or_else(|| {
            refusal(&format!(
                "object {footprint_id} is not an authoritative Footprint identity"
            ))
        })?;
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id && s.kind == SourceShardKind::Pool)
        .ok_or_else(|| refusal("Footprint is not in a native pool"))?;
    let mut footprint =
        super::library::pool_library_object_payload(model, &shard.relative_path, footprint_id)?;
    // The typed Footprint parser establishes object family without discarding raw siblings.
    let _: crate::pool::Footprint = serde_json::from_value(footprint.clone())?;
    match &selector {
        ProcessBasisSelector::Inline { basis } => basis.validate()?,
        ProcessBasisSelector::ProjectProfile { profile_id, .. } => {
            if pinned_profile_selector(model, *profile_id)? != selector {
                return Err(refusal("selected profile pin is stale"));
            }
        }
        ProcessBasisSelector::IpcBasis => {
            let ipc: crate::pool::IpcFootprintBasis = serde_json::from_value(
                footprint
                    .get("ipc_basis")
                    .ok_or_else(|| refusal("IPC selector requires structured Footprint IPC basis"))?
                    .clone(),
            )?;
            if ipc.mask_expansion_nm < 0 || ipc.paste_reduction_nm < 0 {
                return Err(refusal("IPC process signs must be nonnegative"));
            }
        }
    }
    footprint["process_basis"] = serde_json::to_value(selector)?;
    super::library::build_pool_library_write(
        model,
        provenance,
        None,
        vec![super::library::PoolLibraryOperationSpec::Set {
            target: super::library::PoolLibraryObjectTarget::at_relative_path(
                footprint_id,
                "footprints",
                shard.relative_path.clone(),
            ),
            object: footprint,
        }],
    )
}

pub fn inspect_effective_process(
    model: &DesignModel,
    footprint_id: Uuid,
    padstack_id: Uuid,
) -> Result<crate::pool::process::EffectivePadstackProcess, EngineError> {
    let read = |id, expected_kind: &str| {
        let object = model
            .objects
            .get(&id)
            .filter(|o| o.domain == "pool" && o.kind == expected_kind)
            .ok_or_else(|| {
                refusal(&format!(
                    "object {id} is not an authoritative {expected_kind} source"
                ))
            })?;
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id && s.kind == SourceShardKind::Pool)
            .ok_or_else(|| {
                refusal(&format!(
                    "object {id} has no authoritative native pool shard"
                ))
            })?;
        super::library::pool_library_object_payload(model, &shard.relative_path, id)
    };
    let footprint = read(footprint_id, "footprints")?;
    let padstack = read(padstack_id, "padstacks")?;
    inspect_materialized_process(model, &footprint, &padstack).map_err(|e| {
        refusal(&format!(
            "Footprint {footprint_id} Padstack {padstack_id}: {e}"
        ))
    })
}

/// Resolve a disposable preview with the same canonical profile-pin evidence.
/// Neither materialized inputs nor this inspection can mutate the model.
pub fn inspect_materialized_process(
    model: &DesignModel,
    footprint: &Value,
    padstack: &Value,
) -> Result<crate::pool::process::EffectivePadstackProcess, EngineError> {
    let selector: Option<ProcessBasisSelector> = footprint
        .get("process_basis")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()?;
    let profiles = profiles(model)?;
    let pin = match &selector {
        Some(ProcessBasisSelector::ProjectProfile { profile_id, .. }) => {
            let profile = profiles
                .as_ref()
                .and_then(|p| p.get(profile_id))
                .ok_or_else(|| refusal("selected project profile is absent"))?;
            let object = model
                .objects
                .get(profile_id)
                .ok_or_else(|| refusal("missing profile canonical identity"))?;
            Some((
                profile,
                object.object_revision.0,
                crate::substrate::sha256_hex(
                    crate::ir::serialization::to_json_deterministic(&serde_json::to_value(
                        profile,
                    )?)?
                    .as_bytes(),
                ),
            ))
        }
        _ => None,
    };
    crate::pool::process::resolve_padstack_process(
        padstack,
        footprint,
        pin.as_ref().map(
            |(profile, revision, hash)| crate::pool::process::PinnedProcessProfile {
                profile,
                object_revision: *revision,
                value_hash: hash,
            },
        ),
    )
}
