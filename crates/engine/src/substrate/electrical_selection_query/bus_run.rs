//! PM055 origin-declaration-constrained physical Bus query.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CrossBusContact {
    pub origin_bus: Uuid,
    pub foreign_bus: Uuid,
    pub origin_member: ElectricalOccurrence,
    pub foreign_member: ElectricalOccurrence,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusRunMembership {
    pub membership: ElectricalMembership,
    pub origin: ElectricalOccurrence,
    pub contacts: BTreeSet<CrossBusContact>,
}

impl ElectricalSelectionSnapshot {
    pub fn bus_run(
        &self,
        expected: &ModelRevision,
        origin: &ElectricalOccurrence,
    ) -> Result<BusRunMembership, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        if origin.class != "buses"
            || !self.representations.contains(origin)
            || self.basis.occurrence(&self.model, origin).is_err()
        {
            return Err(ElectricalQueryFailure::InvalidOccurrence {
                origin: origin.clone(),
            });
        }
        let id = self.bus_owner(origin)?;
        let declared = self.bus(expected, id)?;
        let schematic =
            self.schematic
                .as_ref()
                .ok_or_else(|| ElectricalQueryFailure::InvalidOccurrence {
                    origin: origin.clone(),
                })?;
        let physical = crate::connectivity::bus_physical(schematic, origin, &declared.owned)?;
        let mut contacts = BTreeSet::new();
        for (origin_member, foreign_member) in physical.foreign {
            let foreign_bus = self.bus_owner(&foreign_member)?;
            // Validate the declaration and all bound source occurrences before disclosure.
            self.bus_representations(foreign_bus)?;
            contacts.insert(CrossBusContact {
                origin_bus: id,
                foreign_bus,
                origin_member,
                foreign_member,
            });
        }
        let mut membership = declared;
        membership.owned = physical.owned;
        Ok(BusRunMembership {
            membership,
            origin: origin.clone(),
            contacts,
        })
    }

    /// BusEntry.bus is an explicit source association to an owned spine, not
    /// an inferred scalar relationship or a selection-triggered binding write.
    pub(super) fn bus_owned_entries(
        &self,
        id: Uuid,
        mut owned: BTreeSet<ElectricalOccurrence>,
    ) -> Result<BTreeSet<ElectricalOccurrence>, ElectricalQueryFailure> {
        let Some(schematic) = &self.schematic else {
            return Ok(owned);
        };
        let spines: Vec<_> = owned
            .iter()
            .filter(|o| o.class == "buses")
            .cloned()
            .collect();
        for spine in spines {
            let sheet = crate::connectivity::occurrence_sheet(schematic, &spine).map_err(|_| {
                ElectricalQueryFailure::InvalidOccurrence {
                    origin: spine.clone(),
                }
            })?;
            for entry in sheet
                .bus_entries
                .values()
                .filter(|e| e.bus == spine.source_id)
            {
                let occurrence = ElectricalOccurrence {
                    class: "bus_entries".into(),
                    source_id: entry.uuid,
                    instance_path: spine.instance_path.clone(),
                };
                let owners: Vec<_> = self
                    .model
                    .electrical_identities
                    .values()
                    .filter_map(|r| match &r.identity {
                        ElectricalIdentity::Bus {
                            representations,
                            retired: false,
                            ..
                        } if representations.contains(&occurrence) => Some(r.id),
                        _ => None,
                    })
                    .collect();
                if owners.len() > 1 || owners.first().is_some_and(|owner| *owner != id) {
                    return Err(ElectricalQueryFailure::UnavailableAssignment {
                        origin: occurrence,
                    });
                }
                if !self.representations.contains(&occurrence)
                    || self.basis.occurrence(&self.model, &occurrence).is_err()
                {
                    return Err(ElectricalQueryFailure::InvalidOccurrence { origin: occurrence });
                }
                owned.insert(occurrence);
            }
        }
        Ok(owned)
    }

    fn bus_owner(&self, origin: &ElectricalOccurrence) -> Result<Uuid, ElectricalQueryFailure> {
        let owners: Vec<_> = self
            .model
            .electrical_identities
            .values()
            .filter_map(|r| match &r.identity {
                ElectricalIdentity::Bus {
                    representations,
                    retired: false,
                    ..
                } if representations.contains(origin) => Some(r.id),
                _ => None,
            })
            .collect();
        if owners.len() == 1 {
            Ok(owners[0])
        } else {
            Err(ElectricalQueryFailure::UnavailableAssignment {
                origin: origin.clone(),
            })
        }
    }
}
