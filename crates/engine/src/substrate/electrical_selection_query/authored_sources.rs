//! Complete source inventory for the shared selection adapter, not scene IDs.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SelectionSourceDomain {
    Board,
    Schematic,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AuthoredSelectionSource {
    pub domain: SelectionSourceDomain,
    pub occurrence: ElectricalOccurrence,
}

impl ElectricalSelectionSnapshot {
    pub fn project_id(&self) -> Uuid {
        self.model.project.project_id
    }
    pub fn source_basis(&self) -> &[SelectionSourceBasis] {
        &self.source_basis
    }

    pub fn authored_sources(
        &self,
    ) -> Result<BTreeSet<AuthoredSelectionSource>, ElectricalQueryFailure> {
        let mut result = BTreeSet::new();
        if let Some(schematic) = &self.schematic {
            let occurrences =
                crate::connectivity::authored_occurrences(schematic).map_err(|e| {
                    ElectricalQueryFailure::UnavailableBasis {
                        sources: self.source_basis.iter().map(|s| s.shard_id).collect(),
                        reason: e.to_string(),
                    }
                })?;
            for occurrence in occurrences {
                if !self.basis.class(
                    &self.model,
                    occurrence.source_id,
                    &occurrence.class,
                    "schematic",
                ) {
                    return Err(ElectricalQueryFailure::InvalidOccurrence { origin: occurrence });
                }
                result.insert(AuthoredSelectionSource {
                    domain: SelectionSourceDomain::Schematic,
                    occurrence,
                });
            }
        }
        for object in self.model.objects.values().filter(|o| o.domain == "board") {
            // The resolver index only supplies candidates. Captured class data
            // proves membership in each actual source map/array.
            for class in ["packages", "pads", "tracks", "vias", "zones", "texts"] {
                if self
                    .basis
                    .class(&self.model, object.object_id, class, "board")
                {
                    result.insert(AuthoredSelectionSource {
                        domain: SelectionSourceDomain::Board,
                        occurrence: ElectricalOccurrence {
                            class: class.into(),
                            source_id: object.object_id,
                            instance_path: vec![],
                        },
                    });
                }
            }
        }
        for shard in self
            .model
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::BoardRoot)
        {
            let value = &self.values[&shard.shard_id];
            let id = value
                .get("uuid")
                .and_then(serde_json::Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok());
            if let Some(id) = id.filter(|id| {
                value.get("outline").is_some()
                    && self.model.objects.get(id).is_some_and(|o| {
                        o.kind == "$" && o.domain == "board" && o.source_shard_id == shard.shard_id
                    })
            }) {
                // The outline is an owned Board source slot. Reuse that real
                // owner UUID with its typed slot class; never allocate an ID.
                result.insert(AuthoredSelectionSource {
                    domain: SelectionSourceDomain::Board,
                    occurrence: ElectricalOccurrence {
                        class: "outline".into(),
                        source_id: id,
                        instance_path: vec![],
                    },
                });
            }
        }
        Ok(result)
    }
}
