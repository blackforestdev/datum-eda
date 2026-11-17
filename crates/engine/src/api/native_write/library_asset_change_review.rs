//! Readable review of the exact states in a prepared canonical library change.
//! Resolving staged content is read-only; this never selects newer revisions.
use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use uuid::Uuid;

use super::{EngineError, PreparedWrite, refusal};
use crate::{
    api::native_write::registry::NativeWriteContext,
    library_assets::{
        AssetRevisionRef, ClosureLimits, VerifiedAsset,
        change_review::{self, ReviewInput},
        inspect_closure,
    },
    substrate::{
        Operation,
        library_asset_operations::LibraryAssetOperation,
        library_asset_resolver::{LibraryAssetLimits, LibraryAssetSnapshot, resolve_state},
    },
};

pub(super) fn attach(
    context: &NativeWriteContext<'_>,
    prepared: &PreparedWrite,
    summary: &mut Value,
    limits: LibraryAssetLimits,
) -> Result<(), EngineError> {
    if summary["action"] == "adopt_profile" {
        return Ok(());
    }
    let mut operations = prepared.batch.operations.iter().filter_map(|op| match op {
        Operation::LibraryAssets { operation } => Some(operation),
        _ => None,
    });
    let Some(
        operation @ LibraryAssetOperation::SetState {
            previous, state, ..
        },
    ) = operations.next()
    else {
        return Err(refusal("library review requires an exact state transition"));
    };
    if operations.next().is_some() {
        return Err(refusal("library review state transition is ambiguous"));
    }
    let pending = operation.content_map(
        limits.metadata_bytes,
        limits.blob_bytes,
        limits.files,
        limits.total_bytes,
    )?;
    let before = resolve_state(context.project_root, previous, &BTreeMap::new(), limits)?;
    let after = resolve_state(context.project_root, state, &pending, limits)?;
    let candidate = summary["action"] == "save_candidate";
    let old = selected(&before, candidate, limits)?;
    let new = selected(&after, candidate, limits)?;
    let ids: BTreeSet<_> = old.keys().chain(new.keys()).copied().collect();
    let mut reviews = Vec::new();
    let mut native_read_bytes = 0usize;
    for id in ids {
        let old_ref = old.get(&id);
        let new_ref = new.get(&id);
        if old_ref == new_ref {
            continue;
        }
        let old_asset = old_ref.and_then(|r| before.assets().get(r));
        let new_asset = new_ref.and_then(|r| after.assets().get(r));
        let review = change_review::compare(old_asset.map(input), new_asset.map(input))?;
        let asset = new_asset
            .or(old_asset)
            .ok_or_else(|| refusal("review asset is unavailable"))?;
        let mut row = json!({"asset_id":id,
            "name":asset.payload().value().get("name"),"review":review});
        // A missing managed preimage says nothing about existing native content.
        // Read the captured model's real preimage, never fabricate an asset record.
        if old_asset.is_none() {
            let kind = asset
                .revision()
                .kind
                .pool_kind()
                .ok_or_else(|| refusal("native review kind is unsupported"))?;
            let native = context
                .model
                .objects
                .get(&id)
                .map(|owner| {
                    if owner.domain != "pool" || owner.kind != kind {
                        return Err(refusal("native review identity has a different owner"));
                    }
                    let remaining = limits
                        .total_bytes
                        .checked_sub(native_read_bytes)
                        .ok_or_else(|| refusal("native review read budget exhausted"))?;
                    let (value, observed) = context.model.capture_source_shard_value_bounded(
                        context.project_root,
                        owner.source_shard_id,
                        limits.blob_bytes.min(remaining),
                    )?;
                    native_read_bytes = native_read_bytes
                        .checked_add(observed)
                        .ok_or_else(|| refusal("native review read size overflow"))?;
                    Ok(value)
                })
                .transpose()?;
            row["native_baseline"] = json!({"exists":native.is_some(),
                "value":native,"comparison":"not_a_prior_managed_revision"});
        }
        reviews.push(row);
    }
    summary["asset_changes"] = Value::Array(reviews);
    summary["native_materialization_changes"] = Value::Array(
        prepared
            .batch
            .operations
            .iter()
            .filter_map(|op| match op {
                Operation::SetPoolLibraryObject {
                    object_id,
                    previous_object,
                    object,
                    ..
                } => Some(
                    json!({"asset_id":object_id,"before":previous_object,"after":object,
                    "created":false}),
                ),
                Operation::CreatePoolLibraryObject {
                    object_id, object, ..
                } => Some(json!({"asset_id":object_id,"before":null,"after":object,
                    "created":true})),
                _ => None,
            })
            .collect(),
    );
    Ok(())
}

fn input(asset: &VerifiedAsset) -> ReviewInput<'_> {
    ReviewInput {
        asset,
        raw_payload: asset.payload().raw_bytes(),
    }
}

fn selected(
    snapshot: &LibraryAssetSnapshot,
    candidate: bool,
    limits: LibraryAssetLimits,
) -> Result<BTreeMap<Uuid, AssetRevisionRef>, EngineError> {
    if !candidate {
        return Ok(snapshot
            .lock()
            .assets
            .iter()
            .map(|locked| (locked.asset.asset_id, locked.asset.clone()))
            .collect());
    }
    let closure = inspect_closure(
        &snapshot.manifest().roots,
        snapshot.assets(),
        ClosureLimits {
            max_nodes: limits.files,
            max_depth: limits.depth,
        },
    )?;
    if !closure.graph_report().valid {
        return Err(refusal("candidate review graph is invalid"));
    }
    let mut result = BTreeMap::new();
    for reference in closure.assets().keys() {
        if result
            .insert(reference.asset_id, (*reference).clone())
            .is_some()
        {
            return Err(refusal("candidate review has conflicting exact revisions"));
        }
    }
    Ok(result)
}
