//! Current authored component bindings provide context between distinct subjects.
use super::*;
use crate::substrate::ComponentInstanceAuthority;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ComponentSelectionRelation {
    pub component_instance: Uuid,
    pub source: ElectricalOccurrence,
    pub counterpart: ElectricalOccurrence,
    pub counterpart_reference: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComponentSelectionContext {
    pub relations: BTreeSet<ComponentSelectionRelation>,
    pub unavailable: BTreeMap<ElectricalOccurrence, String>,
}

impl ElectricalSelectionSnapshot {
    /// Related context only: no identity union, name join, read-time adoption or
    /// inference of an occurrence from a source-only ComponentInstance ref.
    pub fn component_selection_context(
        &self,
        expected: &ModelRevision,
        origins: &BTreeSet<ElectricalOccurrence>,
    ) -> Result<ComponentSelectionContext, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        let mut result = ComponentSelectionContext::default();
        if origins.is_empty() {
            return Ok(result);
        }
        let authored: BTreeSet<_> = self
            .authored_sources()?
            .into_iter()
            .map(|s| s.occurrence)
            .collect();
        for origin in origins {
            if !matches!(origin.class.as_str(), "symbols" | "packages")
                || !authored.contains(origin)
            {
                return Err(ElectricalQueryFailure::InvalidOccurrence {
                    origin: origin.clone(),
                });
            }
            if !origin.instance_path.is_empty() {
                result.unavailable.insert(
                    origin.clone(),
                    "Component binding does not qualify this sheet occurrence".into(),
                );
                continue;
            }
            if authored
                .iter()
                .filter(|id| id.class == origin.class && id.source_id == origin.source_id)
                .count()
                != 1
            {
                result.unavailable.insert(
                    origin.clone(),
                    "Component binding does not distinguish repeated source occurrences".into(),
                );
                continue;
            }
            let symbol = origin.class == "symbols";
            let owners: Vec<_> = self
                .model
                .component_instances
                .values()
                .filter(|ci| {
                    ci.authority == ComponentInstanceAuthority::Authored
                        && self
                            .model
                            .objects
                            .get(&ci.id)
                            .is_some_and(|o| o.domain == "component_instance")
                        && if symbol {
                            ci.placed_symbol_refs.contains(&origin.source_id)
                        } else {
                            ci.placed_package_refs.contains(&origin.source_id)
                        }
                })
                .collect();
            let [owner] = owners.as_slice() else {
                let reason = if owners.is_empty() {
                    "No current authored component binding"
                } else {
                    "Multiple authored component bindings are ambiguous"
                };
                result.unavailable.insert(origin.clone(), reason.into());
                continue;
            };
            if !self.current_component_binding(owner) {
                result.unavailable.insert(
                    origin.clone(),
                    "Authored component binding source or reference revision is stale".into(),
                );
                continue;
            }
            let (class, targets) = if symbol {
                ("packages", &owner.placed_package_refs)
            } else {
                ("symbols", &owner.placed_symbol_refs)
            };
            let counterparts: Vec<_> = targets
                .iter()
                .map(|id| ElectricalOccurrence {
                    class: class.into(),
                    source_id: *id,
                    instance_path: vec![],
                })
                .collect();
            // A partially resolved binding is not a complete related result.
            if counterparts.is_empty()
                || counterparts.iter().any(|id| {
                    !authored.contains(id)
                        || authored
                            .iter()
                            .filter(|actual| {
                                actual.class == id.class && actual.source_id == id.source_id
                            })
                            .count()
                            != 1
                })
            {
                result.unavailable.insert(
                    origin.clone(),
                    "Component counterpart is absent, stale or occurrence-unqualified".into(),
                );
                continue;
            }
            for counterpart in counterparts {
                let counterpart_reference = self
                    .basis
                    .object(&self.model, counterpart.source_id, class)
                    .and_then(|v| v.get("reference"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                result.relations.insert(ComponentSelectionRelation {
                    component_instance: owner.id,
                    source: origin.clone(),
                    counterpart,
                    counterpart_reference,
                });
            }
        }
        Ok(result)
    }

    fn current_component_binding(&self, owner: &crate::substrate::ComponentInstance) -> bool {
        let Some(object) = self.model.objects.get(&owner.id) else {
            return false;
        };
        if !self.model.source_shards.iter().any(|s| {
            s.shard_id == object.source_shard_id && s.kind == SourceShardKind::ComponentInstance
        }) {
            return false;
        }
        let Some(value) = self.values.get(&object.source_shard_id) else {
            return false;
        };
        let Ok(shard) =
            serde_json::from_value::<crate::substrate::ComponentInstanceShard>(value.clone())
        else {
            return false;
        };
        let source = shard.component_instance;
        let current = |r: &crate::substrate::RevisionedRef| {
            self.model
                .objects
                .get(&r.object_id)
                .is_some_and(|o| o.object_revision == r.object_revision)
        };
        shard.schema_version == crate::substrate::COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION
            && source.uuid == owner.id
            && source.object_revision == owner.object_revision
            && source.object_revision == object.object_revision
            && source.placed_symbol_refs.iter().all(current)
            && source.placed_package_refs.iter().all(current)
            && source.part_ref.as_ref().is_none_or(current)
            && source
                .placed_symbol_refs
                .iter()
                .map(|r| r.object_id)
                .collect::<Vec<_>>()
                == owner.placed_symbol_refs
            && source
                .placed_package_refs
                .iter()
                .map(|r| r.object_id)
                .collect::<Vec<_>>()
                == owner.placed_package_refs
            && source.library_bindings.values().all(|b| {
                self.model
                    .objects
                    .get(&b.target_object_id)
                    .is_some_and(|o| o.object_revision == b.pinned_object_revision)
                    && b.local_override_refs.iter().all(current)
            })
    }
}
