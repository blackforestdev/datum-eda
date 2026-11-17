use std::collections::{BTreeMap, BTreeSet};

use uuid::Uuid;

use super::{
    AssetDigest, AssetProvenance, AssetRevision, AssetRevisionRef, LocalCounters, RecordDomain,
    VerifiedPayload, decode_metadata, encode_metadata, invalid, slots,
};
use crate::{
    error::EngineError,
    pool::{LibraryGraph, LibraryGraphValidationReport},
};

/// Fingerprint-verified records, not a new pool authority or approval state.
pub(crate) struct VerifiedAsset {
    reference: AssetRevisionRef,
    revision: AssetRevision,
    provenance: AssetProvenance,
    payload: VerifiedPayload,
}

impl VerifiedAsset {
    pub(crate) fn read(
        expected: &AssetRevisionRef,
        revision_bytes: &[u8],
        provenance_bytes: &[u8],
        payload_bytes: &[u8],
        byte_limit: usize,
        counters: LocalCounters,
    ) -> Result<Self, EngineError> {
        let revision: AssetRevision = decode_metadata(revision_bytes, byte_limit)?;
        if revision.reference()? != *expected {
            return Err(invalid(
                "revision bytes do not match the requested identity",
            ));
        }
        let provenance: AssetProvenance = decode_metadata(provenance_bytes, byte_limit)?;
        if provenance.fingerprint()? != revision.provenance_ref {
            return Err(invalid(
                "provenance fingerprint differs from the asset record",
            ));
        }
        let payload = VerifiedPayload::read(&revision, payload_bytes, byte_limit, counters)?;
        Ok(Self {
            reference: expected.clone(),
            revision,
            provenance,
            payload,
        })
    }

    pub(crate) fn revision(&self) -> &AssetRevision {
        &self.revision
    }

    pub(crate) fn reference(&self) -> &AssetRevisionRef {
        &self.reference
    }
    pub(crate) fn provenance(&self) -> &AssetProvenance {
        &self.provenance
    }
    pub(crate) fn payload(&self) -> &VerifiedPayload {
        &self.payload
    }
}

pub(crate) struct AssetClosure<'a> {
    assets: BTreeMap<AssetRevisionRef, &'a VerifiedAsset>,
    digest: AssetDigest,
    /// Existing graph checks retain their actual coverage; not a standards PASS.
    graph_report: LibraryGraphValidationReport,
}

impl<'a> AssetClosure<'a> {
    pub(crate) fn assets(&self) -> &BTreeMap<AssetRevisionRef, &'a VerifiedAsset> {
        &self.assets
    }
    pub(crate) fn digest(&self) -> &AssetDigest {
        &self.digest
    }
    pub(crate) fn graph_report(&self) -> &LibraryGraphValidationReport {
        &self.graph_report
    }
}

/// Supplied by the transport/application policy, not a hidden numeric budget.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ClosureLimits {
    pub(crate) max_nodes: usize,
    pub(crate) max_depth: usize,
}

pub(crate) fn inspect_closure<'a>(
    roots: &[AssetRevisionRef],
    available: &'a BTreeMap<AssetRevisionRef, VerifiedAsset>,
    limits: ClosureLimits,
) -> Result<AssetClosure<'a>, EngineError> {
    let mut state = Walker {
        available,
        active: BTreeMap::new(),
        visiting: BTreeSet::new(),
        ids: BTreeMap::new(),
        limits,
    };
    let mut root_ids = BTreeSet::new();
    for root in roots {
        if !root_ids.insert(root) {
            return Err(invalid("duplicate closure root"));
        }
        state.walk(root)?;
    }
    let mut graph = LibraryGraph::default();
    for (reference, asset) in &state.active {
        if let Some(kind) = asset.revision.kind.pool_kind() {
            graph.insert_pool_object(
                kind,
                reference.asset_id,
                reference.asset_id.to_string(),
                asset.payload.value().clone(),
            );
        }
    }
    let references: Vec<_> = state.active.keys().collect();
    let digest = AssetDigest::record(RecordDomain::Closure, &encode_metadata(&references)?);
    Ok(AssetClosure {
        assets: state.active,
        digest,
        graph_report: graph.validation_report(),
    })
}

struct Walker<'a> {
    available: &'a BTreeMap<AssetRevisionRef, VerifiedAsset>,
    active: BTreeMap<AssetRevisionRef, &'a VerifiedAsset>,
    visiting: BTreeSet<AssetRevisionRef>,
    ids: BTreeMap<Uuid, AssetRevisionRef>,
    limits: ClosureLimits,
}

impl<'a> Walker<'a> {
    fn walk(&mut self, reference: &AssetRevisionRef) -> Result<(), EngineError> {
        if self.visiting.contains(reference) {
            return Err(invalid("cyclic forward asset dependency"));
        }
        if self.active.contains_key(reference) {
            return Ok(());
        }
        if self.visiting.len() >= self.limits.max_depth || self.ids.len() >= self.limits.max_nodes {
            return Err(invalid("closure exceeds the declared node/depth limits"));
        }
        let asset = self
            .available
            .get(reference)
            .ok_or_else(|| invalid("required asset revision unavailable"))?;
        if asset.reference() != reference {
            return Err(invalid(
                "lookup identity differs from the sealed verified revision",
            ));
        }
        if self
            .ids
            .get(&reference.asset_id)
            .is_some_and(|prior| prior != reference)
        {
            return Err(invalid("conflicting active revisions of one asset"));
        }
        self.ids.insert(reference.asset_id, reference.clone());
        self.visiting.insert(reference.clone());
        let required = slots::slots(asset.revision.kind, asset.payload.value())?;
        if required.len() != asset.revision.dependencies.len() {
            return Err(invalid(
                "declared dependencies do not cover the domain reference slots",
            ));
        }
        for (slot, dependency) in required.iter().zip(&asset.revision.dependencies) {
            if slot.path != dependency.payload_path
                || slot.target_id != dependency.target.asset_id
                || slot.path.last() != Some(&dependency.role)
            {
                return Err(invalid(
                    "dependency identity/slot differs from the domain payload",
                ));
            }
            let target = self
                .available
                .get(&dependency.target)
                .ok_or_else(|| invalid("dependency revision unavailable"))?;
            if target.reference() != &dependency.target {
                return Err(invalid(
                    "dependency lookup differs from the sealed verified revision",
                ));
            }
            if target.revision.kind != slot.kind {
                return Err(invalid("dependency resolves to the wrong asset kind"));
            }
            self.walk(&dependency.target)?;
        }
        self.visiting.remove(reference);
        self.active.insert(reference.clone(), asset);
        Ok(())
    }
}
