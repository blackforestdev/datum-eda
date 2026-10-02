//! Immutable public selection queries. Members come from canonical engine owners,
//! never pane visibility, display names, hit primitives or a context envelope.
#[path = "electrical_selection_query/bus_run.rs"]
mod bus_run;
use super::*;
use crate::substrate::{
    DesignModel, ElectricalIdentity, ElectricalOccurrence, ModelRevision, NetCorrespondenceStatus,
    SourceShardKind,
};
pub use bus_run::{BusRunMembership, CrossBusContact};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElectricalQueryFailure {
    Physical(crate::connectivity::PhysicalQueryFailure),
    StaleRevision,
    AbsentIdentity {
        id: Uuid,
    },
    UnavailableBasis {
        sources: Vec<Uuid>,
        reason: String,
    },
    UnavailableBinding {
        id: Uuid,
        status: NetCorrespondenceStatus,
    },
    InvalidOccurrence {
        origin: ElectricalOccurrence,
    },
    UnavailableGeometry {
        origin: ElectricalOccurrence,
        reason: String,
    },
    UnavailableAssignment {
        origin: ElectricalOccurrence,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionSourceBasis {
    pub shard_id: Uuid,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectricalMembership {
    pub project_id: Uuid,
    pub revision: ModelRevision,
    pub source_basis: Vec<SelectionSourceBasis>,
    pub subject_id: Uuid,
    pub owned: BTreeSet<ElectricalOccurrence>,
    pub related: BTreeSet<ElectricalOccurrence>,
    pub related_subjects: BTreeSet<Uuid>,
}

/// A captured immutable model and source values: subsequent disk writes cannot
/// mix geometry/bindings in an existing query result. Capture itself writes nothing.
pub struct ElectricalSelectionSnapshot {
    model: DesignModel,
    basis: super::super::electrical_basis::ElectricalBasis,
    logical: BTreeMap<Uuid, BTreeSet<ElectricalOccurrence>>,
    physical: Vec<BTreeSet<ElectricalOccurrence>>,
    schematic: Option<crate::schematic::Schematic>,
    representations: BTreeSet<ElectricalOccurrence>,
    values: BTreeMap<Uuid, serde_json::Value>,
    source_basis: Vec<SelectionSourceBasis>,
}

impl ElectricalSelectionSnapshot {
    pub fn capture(model: &DesignModel) -> Result<Self, ElectricalQueryFailure> {
        let mut values = BTreeMap::new();
        let mut source_basis = vec![];
        for shard in &model.source_shards {
            if !matches!(
                shard.kind,
                SourceShardKind::SchematicRoot
                    | SourceShardKind::SchematicSheet
                    | SourceShardKind::SchematicDefinition
                    | SourceShardKind::BoardRoot
                    | SourceShardKind::Pool
                    | SourceShardKind::ElectricalIdentity
                    | SourceShardKind::ComponentInstance
                    | SourceShardKind::ZoneFill
            ) {
                continue;
            }
            let value = super::super::journal::capture_shard_value(model, shard).map_err(|e| {
                ElectricalQueryFailure::UnavailableBasis {
                    sources: vec![shard.shard_id],
                    reason: e.to_string(),
                }
            })?;
            values.insert(shard.shard_id, value);
            source_basis.push(SelectionSourceBasis {
                shard_id: shard.shard_id,
                content_hash: shard.content_hash.clone(),
            });
        }
        source_basis.sort_by_key(|v| v.shard_id);
        let read = |shard: &super::super::SourceShardRef| {
            values
                .get(&shard.shard_id)
                .cloned()
                .ok_or_else(|| EngineError::Validation("missing captured electrical source".into()))
        };
        let failure = |e: EngineError| ElectricalQueryFailure::UnavailableBasis {
            sources: source_basis.iter().map(|s| s.shard_id).collect(),
            reason: e.to_string(),
        };
        let basis = super::super::electrical_basis::ElectricalBasis::with_reader(model, &[], &read)
            .map_err(failure)?;
        let (logical, physical, representations, schematic) = if model
            .source_shards
            .iter()
            .any(|s| s.kind == SourceShardKind::SchematicRoot)
        {
            let schematic =
                super::super::electrical_topology_source::schematic_with_reader(model, &[], &read)
                    .map_err(failure)?;
            let groups = crate::connectivity::partitions(&schematic).map_err(failure)?;
            let logical = super::super::electrical_transaction::previous(model, &groups)
                .map_err(failure)?
                .into_iter()
                .map(|g| (g.net_id, g.members))
                .collect();
            (
                logical,
                crate::connectivity::physical_partitions(&schematic).map_err(failure)?,
                crate::connectivity::representation_occurrences(&schematic).map_err(failure)?,
                Some(schematic),
            )
        } else {
            (BTreeMap::new(), vec![], BTreeSet::new(), None)
        };
        Ok(Self {
            model: model.clone(),
            basis,
            logical,
            physical,
            schematic,
            representations,
            values,
            source_basis,
        })
    }

    pub fn board_run(
        &self,
        expected: &ModelRevision,
        origin: &ElectricalOccurrence,
        hit: Option<crate::ir::geometry::Point>,
    ) -> Result<crate::connectivity::BoardRunMembership, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        self.basis.occurrence(&self.model, origin).map_err(|_| {
            ElectricalQueryFailure::InvalidOccurrence {
                origin: origin.clone(),
            }
        })?;
        let source = self.board_source()?;
        crate::connectivity::board_run(&source, &self.model, origin, hit, &self.source_basis)
            .map_err(ElectricalQueryFailure::Physical)
    }

    pub fn zone_region_successor(
        &self,
        expected: &ModelRevision,
        previous: &Self,
        qualifier: &crate::connectivity::ZoneRegionQualifier,
    ) -> Result<crate::connectivity::ZoneRegionSuccessor, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        crate::connectivity::board_physical_successor(
            &previous.board_source()?,
            &previous.model,
            &previous.source_basis,
            &self.board_source()?,
            &self.model,
            &self.source_basis,
            qualifier,
        )
        .map_err(ElectricalQueryFailure::Physical)
    }

    pub(crate) fn board_source(
        &self,
    ) -> Result<crate::connectivity::BoardCopperSource, ElectricalQueryFailure> {
        let roots: Vec<_> = self
            .model
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::BoardRoot)
            .collect();
        if roots.len() != 1 {
            return Err(ElectricalQueryFailure::UnavailableBasis {
                sources: roots.iter().map(|r| r.shard_id).collect(),
                reason: "board root is not unique".into(),
            });
        }
        let source: crate::connectivity::BoardCopperSource =
            serde_json::from_value(self.values[&roots[0].shard_id].clone()).map_err(|e| {
                ElectricalQueryFailure::UnavailableBasis {
                    sources: vec![roots[0].shard_id],
                    reason: e.to_string(),
                }
            })?;
        if !self.model.objects.get(&source.uuid).is_some_and(|o| {
            o.domain == "board" && o.kind == "$" && o.source_shard_id == roots[0].shard_id
        }) {
            return Err(ElectricalQueryFailure::UnavailableBasis {
                sources: vec![roots[0].shard_id],
                reason: "board root identity does not match the captured model".into(),
            });
        }
        Ok(source)
    }

    pub(crate) fn current_zone_fill(&self, id: Uuid) -> Option<&crate::substrate::ZoneFill> {
        self.model.current_zone_fill(id)
    }

    pub fn revision(&self) -> &ModelRevision {
        &self.model.model_revision
    }

    pub fn net_ids(&self) -> BTreeSet<Uuid> {
        self.model
            .electrical_identities
            .values()
            .filter_map(|r| {
                matches!(r.identity, ElectricalIdentity::Net { retired: false, .. }).then_some(r.id)
            })
            .collect()
    }

    pub fn bus_ids(&self) -> BTreeSet<Uuid> {
        self.model
            .electrical_identities
            .values()
            .filter_map(|r| {
                matches!(r.identity, ElectricalIdentity::Bus { retired: false, .. }).then_some(r.id)
            })
            .collect()
    }

    fn require_revision(&self, expected: &ModelRevision) -> Result<(), ElectricalQueryFailure> {
        if expected != self.revision() {
            Err(ElectricalQueryFailure::StaleRevision)
        } else {
            Ok(())
        }
    }

    fn result(&self, id: Uuid, owned: BTreeSet<ElectricalOccurrence>) -> ElectricalMembership {
        ElectricalMembership {
            project_id: self.model.project.project_id,
            revision: self.revision().clone(),
            source_basis: self.source_basis.clone(),
            subject_id: id,
            owned,
            related: BTreeSet::new(),
            related_subjects: BTreeSet::new(),
        }
    }

    pub fn global_net(
        &self,
        expected: &ModelRevision,
        id: Uuid,
    ) -> Result<ElectricalMembership, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        let record = self
            .model
            .electrical_identities
            .get(&id)
            .filter(|r| matches!(r.identity, ElectricalIdentity::Net { retired: false, .. }))
            .ok_or(ElectricalQueryFailure::AbsentIdentity { id })?;
        let status = super::super::electrical_identity_validation::correspondence_with_basis(
            &self.model,
            id,
            Some(&self.basis),
            Some(&self.logical),
        );
        if status != NetCorrespondenceStatus::Complete {
            return Err(ElectricalQueryFailure::UnavailableBinding { id, status });
        }
        let mut members = self.logical.get(&id).cloned().unwrap_or_default();
        let mut found_board = false;
        let mut schematic_only = false;
        for relation in self.model.electrical_identities.values() {
            if let ElectricalIdentity::NetRelationship {
                logical_net,
                board_net,
                intent,
                ..
            } = &relation.identity
            {
                if *logical_net != id {
                    continue;
                }
                schematic_only |= *intent == super::super::NetRelationshipIntent::SchematicOnly;
                if let Some(board) = board_net {
                    found_board = true;
                    members.extend(self.board_net_members(board.object_id)?);
                }
            }
        }
        if let ElectricalIdentity::Net { anchor, .. } = &record.identity {
            self.basis.occurrence(&self.model, anchor).map_err(|_| {
                ElectricalQueryFailure::InvalidOccurrence {
                    origin: anchor.clone(),
                }
            })?;
            if !super::super::electrical_transaction::schematic_class(&anchor.class) && !found_board
            {
                return Err(ElectricalQueryFailure::UnavailableBinding {
                    id,
                    status: NetCorrespondenceStatus::Unverified,
                });
            }
        }
        if !found_board && !schematic_only {
            return Err(ElectricalQueryFailure::UnavailableBinding {
                id,
                status: NetCorrespondenceStatus::Unverified,
            });
        }
        Ok(self.result(id, members))
    }

    fn board_net_members(
        &self,
        net: Uuid,
    ) -> Result<BTreeSet<ElectricalOccurrence>, ElectricalQueryFailure> {
        let mut members = BTreeSet::new();
        let boards: Vec<_> = self
            .model
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::BoardRoot)
            .collect();
        if boards.len() != 1 {
            return Err(ElectricalQueryFailure::UnavailableBasis {
                sources: boards.iter().map(|s| s.shard_id).collect(),
                reason: "board root is not unique".into(),
            });
        }
        let value = &self.values[&boards[0].shard_id];
        if value
            .get("nets")
            .and_then(|v| v.get(net.to_string()))
            .is_none()
        {
            return Err(ElectricalQueryFailure::UnavailableBinding {
                id: net,
                status: NetCorrespondenceStatus::Unverified,
            });
        }
        for class in ["tracks", "vias", "pads", "zones"] {
            let objects = value
                .get(class)
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| ElectricalQueryFailure::UnavailableBasis {
                    sources: vec![boards[0].shard_id],
                    reason: format!("missing {class} source map"),
                })?;
            for (key, object) in objects {
                if object.get("net").and_then(serde_json::Value::as_str)
                    != Some(net.to_string().as_str())
                {
                    continue;
                }
                let source_id =
                    Uuid::parse_str(key).map_err(|_| ElectricalQueryFailure::UnavailableBasis {
                        sources: vec![boards[0].shard_id],
                        reason: "malformed source UUID".into(),
                    })?;
                let origin = ElectricalOccurrence {
                    class: class.into(),
                    source_id,
                    instance_path: vec![],
                };
                self.basis.occurrence(&self.model, &origin).map_err(|_| {
                    ElectricalQueryFailure::InvalidOccurrence {
                        origin: origin.clone(),
                    }
                })?;
                members.insert(origin);
            }
        }
        Ok(members)
    }

    /// Local physical membership is deliberately independent of name/interface
    /// unions. The returned identity is the existing logical Net, never a graph ID.
    pub fn schematic_run(
        &self,
        expected: &ModelRevision,
        origin: &ElectricalOccurrence,
    ) -> Result<ElectricalMembership, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        self.basis.occurrence(&self.model, origin).map_err(|_| {
            ElectricalQueryFailure::InvalidOccurrence {
                origin: origin.clone(),
            }
        })?;
        let physical = self
            .physical
            .iter()
            .find(|g| g.contains(origin))
            .ok_or_else(|| ElectricalQueryFailure::UnavailableAssignment {
                origin: origin.clone(),
            })?;
        let owners: Vec<_> = self
            .logical
            .iter()
            .filter(|(_, g)| g.contains(origin))
            .collect();
        if owners.len() != 1 {
            return Err(ElectricalQueryFailure::UnavailableAssignment {
                origin: origin.clone(),
            });
        }
        let (id, net) = owners[0];
        if !physical.is_subset(net) {
            return Err(ElectricalQueryFailure::UnavailableAssignment {
                origin: origin.clone(),
            });
        }
        Ok(self.result(*id, physical.clone()))
    }

    pub fn bus(
        &self,
        expected: &ModelRevision,
        id: Uuid,
    ) -> Result<ElectricalMembership, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        let record = self
            .model
            .electrical_identities
            .get(&id)
            .ok_or(ElectricalQueryFailure::AbsentIdentity { id })?;
        let ElectricalIdentity::Bus {
            scalar_nets,
            retired: false,
            ..
        } = &record.identity
        else {
            return Err(ElectricalQueryFailure::AbsentIdentity { id });
        };
        let owned = self.bus_representations(id)?;
        let mut result = self.result(id, owned);
        for net in scalar_nets {
            let members = self.logical.get(net).ok_or({
                ElectricalQueryFailure::UnavailableBinding {
                    id: *net,
                    status: NetCorrespondenceStatus::Unverified,
                }
            })?;
            result.related.extend(members.clone());
            result.related_subjects.insert(*net);
        }
        for relation in self.model.electrical_identities.values() {
            if let ElectricalIdentity::BusInterface {
                left_bus,
                right_bus,
                ..
            } = &relation.identity
            {
                if *left_bus == id && *right_bus != id {
                    result.related_subjects.insert(*right_bus);
                    result.related.extend(self.bus_representations(*right_bus)?);
                }
                if *right_bus == id && *left_bus != id {
                    result.related_subjects.insert(*left_bus);
                    result.related.extend(self.bus_representations(*left_bus)?);
                }
            }
        }
        Ok(result)
    }
    fn bus_representations(
        &self,
        id: Uuid,
    ) -> Result<BTreeSet<ElectricalOccurrence>, ElectricalQueryFailure> {
        let Some(ElectricalIdentity::Bus {
            representations,
            retired: false,
            ..
        }) = self
            .model
            .electrical_identities
            .get(&id)
            .map(|r| &r.identity)
        else {
            return Err(ElectricalQueryFailure::AbsentIdentity { id });
        };
        for origin in representations {
            if !self.representations.contains(origin)
                || self.basis.occurrence(&self.model, origin).is_err()
            {
                return Err(ElectricalQueryFailure::InvalidOccurrence {
                    origin: origin.clone(),
                });
            }
        }
        self.bus_owned_entries(id, representations.clone())
    }
}
