//! Origin-to-subject binding uses canonical declarations, never names or graph IDs.
use super::*;

impl ElectricalSelectionSnapshot {
    /// Resolve the origin's canonical Global Net and its complete declared domain.
    /// A board-local Net UUID is an assignment, not a logical identity. Pending or
    /// contradictory correspondence cannot become a local-only complete result.
    pub fn net_for_origin(
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
        let owners: BTreeSet<_> = match origin.class.as_str() {
            "wires" | "labels" | "ports" | "junctions" | "pins" => self
                .logical
                .iter()
                .filter_map(|(id, members)| members.contains(origin).then_some(*id))
                .collect(),
            "tracks" | "vias" | "pads" | "zones" => {
                let board_net = self
                    .basis
                    .object(&self.model, origin.source_id, &origin.class)
                    .and_then(|value| value.get("net"))
                    .and_then(serde_json::Value::as_str)
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .ok_or_else(|| ElectricalQueryFailure::UnavailableAssignment {
                        origin: origin.clone(),
                    })?;
                self.model
                    .electrical_identities
                    .values()
                    .filter_map(|record| match &record.identity {
                        ElectricalIdentity::NetRelationship {
                            logical_net,
                            board_net: Some(reference),
                            ..
                        } if reference.object_id == board_net => Some(*logical_net),
                        _ => None,
                    })
                    .collect()
            }
            _ => {
                return Err(ElectricalQueryFailure::InvalidOccurrence {
                    origin: origin.clone(),
                });
            }
        };
        if owners.len() != 1 {
            return Err(ElectricalQueryFailure::UnavailableAssignment {
                origin: origin.clone(),
            });
        }
        // Reuse correspondence and full membership validation, including the
        // complete board inventory. No read-time adoption or partial fallback.
        self.global_net(expected, *owners.first().unwrap())
    }

    /// Semantic widening includes disconnected representations of the origin's
    /// declared Bus, while foreign declarations and scalar Nets remain related.
    pub fn bus_for_origin(
        &self,
        expected: &ModelRevision,
        origin: &ElectricalOccurrence,
    ) -> Result<ElectricalMembership, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        if origin.class != "buses"
            || !self.representations.contains(origin)
            || self.basis.occurrence(&self.model, origin).is_err()
        {
            return Err(ElectricalQueryFailure::InvalidOccurrence {
                origin: origin.clone(),
            });
        }
        self.bus(expected, self.bus_owner(origin)?)
    }
}
