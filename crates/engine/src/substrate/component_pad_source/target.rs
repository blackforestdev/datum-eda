//! Read-only, verified planned library inputs. This is expressly not an accepted
//! DesignModel or a commit-certified pending frame. Final admission is separate.
use super::{PinnedPadLibrarySource, invalid, pins};
use crate::{
    error::EngineError,
    library_assets::{AssetRevisionRef, lock::ProjectLibraryLock},
    substrate::{
        DesignModel, DomainObject, ObjectRevision, Operation, SourceShardKind, SourceShardTaxon,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PhysicalPadContinuity {
    pub(crate) placed_pad: Uuid,
    pub(crate) library_pad: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PhysicalPlacementIntent {
    pub(crate) component_instance: Uuid,
    pub(crate) placed_package: Uuid,
    pub(crate) target_assets: Vec<AssetRevisionRef>,
    pub(crate) pads: Vec<PhysicalPadContinuity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PhysicalGeometryIntent {
    pub(crate) placements: Vec<PhysicalPlacementIntent>,
}

pub(crate) struct PlannedPadSources<'a> {
    accepted: &'a DesignModel,
    objects: BTreeMap<Uuid, DomainObject>,
    pool: BTreeMap<String, Value>,
    operations: Vec<Operation>,
    lock: ProjectLibraryLock,
    components: BTreeMap<Uuid, Value>,
    impacts: Vec<
        crate::substrate::library_asset_journal_ops::physical_impact::UnsupportedPhysicalAdoption,
    >,
}

impl<'a> PlannedPadSources<'a> {
    /// Only the library admission owner constructs this after applying exact
    /// canonical operations to qualified source/object maps. No history is made.
    pub(in crate::substrate) fn new(
        accepted: &'a DesignModel,
        objects: BTreeMap<Uuid, DomainObject>,
        pool: BTreeMap<String, Value>,
        operations: Vec<Operation>,
        lock: ProjectLibraryLock,
        components: BTreeMap<Uuid, Value>,
        impacts: Vec<crate::substrate::library_asset_journal_ops::physical_impact::UnsupportedPhysicalAdoption>,
    ) -> Self {
        Self {
            accepted,
            objects,
            pool,
            operations,
            lock,
            components,
            impacts,
        }
    }

    pub(crate) fn accepted(&self) -> &DesignModel {
        self.accepted
    }
    pub(crate) fn recorded_package(
        &self,
        reference: &crate::substrate::RevisionedRef,
    ) -> Result<(Value, ObjectRevision), EngineError> {
        crate::substrate::component_instance::placed_package_history::recorded_package(
            &self.accepted.journal,
            reference,
        )
    }
    pub(crate) fn object_revision(&self, id: Uuid) -> Result<ObjectRevision, EngineError> {
        self.objects
            .get(&id)
            .map(|object| object.object_revision)
            .ok_or_else(|| invalid("planned source object is unavailable"))
    }
    pub(in crate::substrate) fn objects(&self) -> &BTreeMap<Uuid, DomainObject> {
        &self.objects
    }
    pub(in crate::substrate) fn value(&self, path: &str) -> Result<Value, EngineError> {
        self.pool
            .get(path)
            .cloned()
            .ok_or_else(|| invalid("planned pool source is unavailable"))
    }
    pub(crate) fn component(&self, id: Uuid) -> Option<&Value> {
        self.components.get(&id)
    }
    pub(crate) fn impacts(&self)->&[crate::substrate::library_asset_journal_ops::physical_impact::UnsupportedPhysicalAdoption]{
        &self.impacts
    }
    pub(crate) fn reference(&self, id: Uuid) -> Result<AssetRevisionRef, EngineError> {
        let entries: Vec<_> = self
            .lock
            .assets
            .iter()
            .filter(|entry| entry.local_object_id == id)
            .collect();
        match entries.as_slice() {
            [entry] => Ok(entry.asset.clone()),
            _ => Err(invalid("planned source lacks one exact locked revision")),
        }
    }
    pub(crate) fn capture_root(
        &self,
        id: Uuid,
        taxon: SourceShardTaxon,
    ) -> Result<PinnedPadLibrarySource, EngineError> {
        let paths: Vec<_> = self
            .accepted
            .source_shards
            .iter()
            .filter(|head| {
                head.kind == SourceShardKind::Pool
                    && head.taxon == Some(taxon)
                    && self
                        .pool
                        .get(&head.relative_path)
                        .is_some_and(|value| value["uuid"] == serde_json::json!(id))
            })
            .map(|head| head.relative_path.as_str())
            .collect();
        match paths.as_slice() {
            [path] => pins::capture_in(&pins::PinFrame::Planned(self), path, id),
            _ => Err(invalid(
                "planned root ownership is unavailable or ambiguous",
            )),
        }
    }
    pub(crate) fn capture_pad(
        &self,
        owner: &PinnedPadLibrarySource,
        id: Uuid,
    ) -> Result<PinnedPadLibrarySource, EngineError> {
        pins::capture_in(&pins::PinFrame::Planned(self), &owner.relative_path, id)
    }
    pub(crate) fn source_value(&self, pin: &PinnedPadLibrarySource) -> Result<Value, EngineError> {
        pins::check_pin_in(&pins::PinFrame::Planned(self), pin)?;
        pins::source_value_in(&pins::PinFrame::Planned(self), pin)
    }
    pub(crate) fn continuity(
        &self,
        old: &PinnedPadLibrarySource,
        new: &PinnedPadLibrarySource,
    ) -> Result<(), EngineError> {
        pins::continuity_in(&pins::PinFrame::Planned(self), old, new)
    }
    pub(in crate::substrate) fn definition(
        &self,
        id: Uuid,
        kind: &str,
        revision: ObjectRevision,
    ) -> Result<Value, EngineError> {
        crate::substrate::component_instance::pool_history::PoolHistoryBasis {
            model: self.accepted,
            operations: &self.operations,
            applied_operations: &self.operations,
        }
        .projected_definition_value_with_reader(&self.objects, id, revision, kind, &|head| {
            self.accepted
                .materialized_source_shard_value_by_relative_path(&head.relative_path)
        })?
        .ok_or_else(|| invalid("planned definition revision is unavailable"))
    }
}
