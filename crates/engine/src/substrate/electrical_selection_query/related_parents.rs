//! PM026 parent context is source ownership, never electrical membership.
use super::*;
impl ElectricalSelectionSnapshot {
    /// Global workspace acquisition retains valid standalone Pads while owned
    /// Pads collapse to their actual package. Read captured ownership source
    /// once for the complete requested Pad cohort, never require complete
    /// copper geometry merely to establish source ownership.
    pub fn standalone_pads(
        &self,
        expected: &ModelRevision,
        pads: &BTreeSet<ElectricalOccurrence>,
    ) -> Result<BTreeSet<ElectricalOccurrence>, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        if pads.is_empty() {
            return Ok(BTreeSet::new());
        }
        let ids = pads.iter().map(|origin| origin.source_id).collect();
        let parents = self.basis.pad_owners(&self.model, &ids);
        let mut result = BTreeSet::new();
        for origin in pads {
            let invalid = || ElectricalQueryFailure::InvalidOccurrence {
                origin: origin.clone(),
            };
            if origin.class != "pads" {
                return Err(invalid());
            }
            self.basis
                .occurrence(&self.model, origin)
                .map_err(|_| invalid())?;
            let parent = *parents.get(&origin.source_id).ok_or_else(invalid)?;
            if parent.is_nil() {
                result.insert(origin.clone());
            } else if !self.basis.class(&self.model, parent, "packages", "board")
                || self.model.objects[&parent].source_shard_id
                    != self.model.objects[&origin.source_id].source_shard_id
            {
                return Err(invalid());
            }
        }
        Ok(result)
    }
    pub fn related_parents(
        &self,
        expected: &ModelRevision,
        owned: &BTreeSet<ElectricalOccurrence>,
    ) -> Result<BTreeSet<ElectricalOccurrence>, ElectricalQueryFailure> {
        self.require_revision(expected)?;
        let mut result = BTreeSet::new();
        let mut pins = BTreeMap::new();
        if owned.iter().any(|o| o.class == "pins")
            && let Some(schematic) = &self.schematic
        {
            for symbol in schematic.sheets.values().flat_map(|s| s.symbols.values()) {
                for pin in &symbol.pins {
                    if pins
                        .insert(pin.uuid, symbol.uuid)
                        .is_some_and(|prior| prior != symbol.uuid)
                    {
                        return Err(ElectricalQueryFailure::UnavailableBasis {
                            sources: self.source_basis.iter().map(|s| s.shard_id).collect(),
                            reason: "Pin has multiple source parents".into(),
                        });
                    }
                }
            }
        }
        let board = owned
            .iter()
            .any(|o| o.class == "pads")
            .then(|| self.board_source())
            .transpose()?;
        for origin in owned
            .iter()
            .filter(|o| matches!(o.class.as_str(), "pins" | "pads"))
        {
            let invalid = || ElectricalQueryFailure::InvalidOccurrence {
                origin: origin.clone(),
            };
            self.basis
                .occurrence(&self.model, origin)
                .map_err(|_| invalid())?;
            let (class, parent, domain) = if origin.class == "pins" {
                (
                    "symbols",
                    *pins.get(&origin.source_id).ok_or_else(invalid)?,
                    "schematic",
                )
            } else {
                let board = board.as_ref().expect("requested pad source captured");
                let pad = board.pad_terminal(origin.source_id).ok_or_else(invalid)?;
                // Canonical standalone pads explicitly carry no package.
                // Absence of related context cannot invalidate their membership.
                if pad.package.is_nil() {
                    continue;
                }
                ("packages", pad.package, "board")
            };
            if !self.basis.class(&self.model, parent, class, domain)
                || self.model.objects[&parent].source_shard_id
                    != self.model.objects[&origin.source_id].source_shard_id
            {
                return Err(invalid());
            }
            let parent = ElectricalOccurrence {
                class: class.into(),
                source_id: parent,
                instance_path: origin.instance_path.clone(),
            };
            if !owned.contains(&parent) {
                result.insert(parent);
            }
        }
        Ok(result)
    }
}
