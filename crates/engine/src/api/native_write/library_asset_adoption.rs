//! Build-only library revision and adoption planning over canonical operations.
//! Candidate saves retain the active project lock. Adoption is a separate plan.
use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use uuid::Uuid;

use super::{
    context::{PreparedWrite, WriteProvenance, build_batch},
    registry::NativeWriteContext,
};
use crate::{
    error::EngineError,
    library_assets::{
        Actor, AssetDigest, AssetKind, AssetProvenance, AssetRevision, AssetRevisionRef,
        ClosureLimits, Dependency, Evidence, OriginKind, PROFILE, RecordDomain, Rights,
        VerifiedAsset, encode_metadata, inspect_closure,
        manifest::{BlobEntry, LibraryDiscovery, LibraryReleaseManifest, ObjectEntry, ObjectKind},
    },
    substrate::{
        Operation, SourceShardKind,
        library_asset_operations::{
            LibraryAssetOperation, LibraryContent, LibraryRecordType, LibraryStateValues, refusal,
        },
        library_asset_resolver::{
            LibraryAssetLimits, LibraryAssetSnapshot, counter_policy, read_state, resolve_state,
            restrict_limits,
        },
    },
};

mod adoption;
mod capture;
mod catalog;
pub(crate) use adoption::{AdoptionPlan, AdoptionRequest, prepare_adoption};

/// Every edit names its exact current catalog basis. Native capture is explicit
/// and cannot silently replace an asset already managed by that catalog.
pub(crate) enum CandidateChange {
    CaptureNative {
        asset_id: Uuid,
    },
    Revise {
        base: AssetRevisionRef,
        payload: Value,
    },
}

pub(crate) struct CandidateRequest {
    pub(crate) expected_catalog: AssetDigest,
    pub(crate) change: CandidateChange,
    /// A recorded claim supplied by the authoring surface, never inferred here.
    pub(crate) changed_at: Option<String>,
    /// Explicit recorded claims for the selected asset only. Captured
    /// dependencies do not inherit the selected asset's author or rights.
    pub(crate) provenance_override: Option<AssetProvenance>,
}

pub(crate) struct CandidatePlan {
    pub(crate) prepared: PreparedWrite,
    pub(crate) candidate: AssetRevisionRef,
    /// Includes dependents whose exact dependency pins changed.
    pub(crate) changed_assets: Vec<AssetRevisionRef>,
}

/// Generate immutable candidate revisions and a local discovery head. No native
/// pool materialization, placed binding or active lock is adopted by this save.
pub(crate) fn prepare_candidate(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: CandidateRequest,
    limits: LibraryAssetLimits,
) -> Result<CandidatePlan, EngineError> {
    let limits = restrict_limits(limits)?;
    if context.model.project.schema_version != Some(2) {
        return Err(refusal(
            "separately adopt the library format before saving revisions",
        ));
    }
    let previous = read_state(context.project_root, limits)?;
    let snapshot = resolve_state(context.project_root, &previous, &BTreeMap::new(), limits)?;
    if snapshot.catalog().manifest_ref != request.expected_catalog {
        return Err(refusal("catalog changed since the revision editor opened"));
    }
    let draft = capture::capture(context, &snapshot, &provenance, request, limits)?;
    if draft.changed.is_empty() {
        return Err(refusal("candidate has no engineering or provenance change"));
    }
    let (state, content) = catalog::candidate_state(&snapshot, &previous, &provenance, &draft)?;
    let operation = LibraryAssetOperation::SetState {
        project_id: context.model.project.project_id,
        previous,
        state,
        content,
    };
    let files = operation.content_map(
        limits.metadata_bytes,
        limits.blob_bytes,
        limits.files,
        limits.total_bytes,
    )?;
    // The shared resolver rechecks fingerprints, schema slots, graph and retained
    // evidence. Draft construction never substitutes for verified closure.
    let checked = resolve_state(context.project_root, operation.state()?, &files, limits)?;
    let closure = inspect_closure(
        &checked.manifest().roots,
        checked.assets(),
        ClosureLimits {
            max_nodes: limits.files,
            max_depth: limits.depth,
        },
    )?;
    if !closure.graph_report().valid {
        return Err(refusal(
            "candidate library graph is invalid; retain the editor draft for repair",
        ));
    }
    let candidate = draft.selected.clone();
    let changed_assets = draft.changed;
    Ok(CandidatePlan {
        prepared: PreparedWrite {
            batch: build_batch(
                context.model,
                provenance,
                vec![Operation::LibraryAssets { operation }],
            )?,
            primary_object_id: Some(candidate.asset_id),
        },
        candidate,
        changed_assets,
    })
}

fn actor(provenance: &WriteProvenance) -> Actor {
    Actor {
        recorded_id: Some(provenance.actor.clone()),
        display_name: None,
        verification_evidence: vec![],
    }
}

#[cfg(test)]
mod tests;
