//! Whole immutable physical receipt proof, shared by read-only preparation and
//! selected-target compensation. No filesystem, frame or accepted-model fiction.
use super::*;
use crate::{
    library_assets::{AssetDigest, AssetRevision, VerifiedAsset, decode_metadata},
    substrate::{
        library_asset_operations::{LibraryContent, LibraryRecordType},
        library_asset_resolver::{LibraryAssetLimits, counter_policy, server_policy},
    },
};

struct Records<'a> {
    prior: &'a [TransactionRecord],
    limits: LibraryAssetLimits,
    charged: BTreeMap<String, usize>,
}
impl<'a> Records<'a> {
    fn read(
        &mut self,
        digest: &AssetDigest,
        kind: Option<LibraryRecordType>,
    ) -> Result<&'a [u8], EngineError> {
        let mut found = None;
        for op in self.prior.iter().flat_map(|tx| &tx.operations) {
            let Operation::LibraryAssets { operation } = op else {
                continue;
            };
            let content = match operation {
                LibraryAssetOperation::SetState { content, .. }
                | LibraryAssetOperation::AdoptProfile { content, .. } => content,
                _ => continue,
            };
            for item in content {
                if item.fingerprint()? != *digest {
                    continue;
                }
                item.validate(self.limits.metadata_bytes, self.limits.blob_bytes)?;
                match (item, kind) {
                    (LibraryContent::Record { record_type, .. }, Some(expected))
                        if *record_type == expected => {}
                    (LibraryContent::Blob { .. }, None) => {}
                    _ => {
                        return Err(invalid(
                            "physical receipt content type differs from exact reference",
                        ));
                    }
                }
                if found.is_some_and(|bytes: &[u8]| bytes != item.bytes()) {
                    return Err(invalid("immutable physical receipt content changed"));
                }
                found = Some(item.bytes());
            }
        }
        let bytes = found
            .ok_or_else(|| invalid("physical source lacks a retained immutable journal witness"))?;
        self.charged.insert(digest.as_str().to_owned(), bytes.len());
        let used = self
            .charged
            .values()
            .try_fold(0usize, |sum, size| sum.checked_add(*size))
            .ok_or_else(|| invalid("physical receipt byte budget overflow"))?;
        if used > self.limits.total_bytes || self.charged.len() > self.limits.files {
            return Err(invalid("physical receipt exceeds library policy"));
        }
        Ok(bytes)
    }
    fn asset(&mut self, reference: &AssetRevisionRef) -> Result<VerifiedAsset, EngineError> {
        let record = self.read(
            &reference.revision_id,
            Some(LibraryRecordType::AssetRevision),
        )?;
        let revision: AssetRevision = decode_metadata(record, self.limits.metadata_bytes)?;
        let provenance = self.read(
            &revision.provenance_ref,
            Some(LibraryRecordType::Provenance),
        )?;
        let payload = self.read(&revision.payload_ref, None)?;
        // Each raw input already passed its actual metadata/blob cap above.
        VerifiedAsset::read(
            reference,
            record,
            provenance,
            payload,
            self.limits.metadata_bytes.max(self.limits.blob_bytes),
            counter_policy(revision.kind, revision.payload_schema_version)?,
        )
    }
}

pub(super) fn verify(
    operations: &[Operation],
    inverse_operations: &[Operation],
    prior: &[TransactionRecord],
    before: &ModelRevision,
    proof: &mut Proof<'_>,
) -> Result<(), EngineError> {
    let memo = proof.receipt(operations, inverse_operations, prior, before);
    if memo.is_some_and(|index| proof.has_receipt(index)) {
        return Ok(());
    }
    if super::inverse(operations)? != inverse_operations
        || prior
            .last()
            .is_none_or(|last| &last.after_model_revision != before)
    {
        return Err(invalid(
            "physical receipt is not the whole current-source inverse",
        ));
    }
    let states: Vec<_> = operations
        .iter()
        .filter_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation),
            _ => None,
        })
        .collect();
    let [
        LibraryAssetOperation::SetState {
            project_id,
            previous,
            state,
            content,
        },
    ] = states.as_slice()
    else {
        return Err(invalid(
            "physical receipt needs one exact library transition",
        ));
    };
    if !content.is_empty() || previous.catalog != state.catalog {
        return Err(invalid(
            "physical receipt changes the local discovery catalog",
        ));
    }
    let current = prior
        .iter()
        .rev()
        .flat_map(|tx| tx.operations.iter().rev())
        .find_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation.state()),
            _ => None,
        })
        .transpose()?
        .ok_or_else(|| invalid("physical receipt has no recorded library preimage"))?;
    if previous != current {
        return Err(invalid("physical receipt library preimage differs"));
    }
    let limits = server_policy()?;
    let (old_lock, _) = previous.decode(limits.metadata_bytes)?;
    let (new_lock, _) = state.decode(limits.metadata_bytes)?;
    if old_lock
        .assets
        .iter()
        .map(|asset| asset.local_object_id)
        .collect::<BTreeSet<_>>()
        != new_lock
            .assets
            .iter()
            .map(|asset| asset.local_object_id)
            .collect()
    {
        return Err(invalid("physical receipt changes managed inventory"));
    }
    let mut records = Records {
        prior,
        limits,
        charged: BTreeMap::new(),
    };
    let references: BTreeSet<_> = old_lock
        .assets
        .iter()
        .chain(&new_lock.assets)
        .map(|entry| entry.asset.clone())
        .collect();
    let mut available = BTreeMap::new();
    for reference in references {
        available.insert(reference.clone(), records.asset(&reference)?);
    }
    for lock in [&old_lock, &new_lock] {
        let closure = crate::library_assets::inspect_closure(
            &lock
                .assets
                .iter()
                .map(|entry| entry.asset.clone())
                .collect::<Vec<_>>(),
            &available,
            crate::library_assets::ClosureLimits {
                max_nodes: limits.files,
                max_depth: limits.depth,
            },
        )?;
        lock.verify_closure(&closure)?;
        if !closure.graph_report().valid {
            return Err(invalid(
                "physical receipt has an invalid complete component graph",
            ));
        }
        if lock.bindings.iter().any(|binding| {
            available
                .get(&binding.component)
                .is_none_or(|asset| asset.revision().kind != AssetKind::Part)
        }) {
            return Err(invalid(
                "physical receipt occurrence is not bound to a Part",
            ));
        }
    }
    for old in &old_lock.assets {
        let new = new_lock
            .assets
            .iter()
            .find(|entry| entry.local_object_id == old.local_object_id)
            .unwrap();
        if !matches!(
            available[&old.asset].revision().kind,
            AssetKind::Footprint | AssetKind::Padstack
        ) && available[&old.asset].revision().payload_ref
            != available[&new.asset].revision().payload_ref
        {
            return Err(invalid(
                "physical receipt changes other component definitions",
            ));
        }
    }
    let binding_key = |lock: &ProjectLibraryLock| {
        lock.bindings
            .iter()
            .map(|binding| {
                ((
                    binding.source_path.clone(),
                    binding.instance_path.clone(),
                    binding.component_instance_id,
                    binding.component.asset_id,
                ))
            })
            .collect::<BTreeSet<_>>()
    };
    if binding_key(&old_lock) != binding_key(&new_lock) {
        return Err(invalid(
            "physical receipt changes qualified component occurrences",
        ));
    }
    let mut pool = Pool::history(prior)?;
    let mut pool_changes = BTreeSet::new();
    for op in operations {
        if let Operation::SetPoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            previous_object,
            object,
        } = op
        {
            if !matches!(object_kind.as_str(), "footprints" | "padstacks")
                || !pool_changes.insert(*object_id)
            {
                return Err(invalid(
                    "physical receipt has unsupported definition changes",
                ));
            }
            crate::substrate::library_asset_journal_ops::admission::qualify_geometry_delta(
                previous_object,
                object,
                object_kind,
            )?;
            for (lock, value) in [(&old_lock, previous_object), (&new_lock, object)] {
                let entries: Vec<_> = lock
                    .assets
                    .iter()
                    .filter(|entry| entry.local_object_id == *object_id)
                    .collect();
                let [entry] = entries.as_slice() else {
                    return Err(invalid("physical receipt root is not uniquely locked"));
                };
                let asset = records.asset(&entry.asset)?;
                if asset.revision().kind.pool_kind() != Some(object_kind.as_str())
                    || entry.source_path != *relative_path
                    || entry.payload_ref != asset.revision().payload_ref
                {
                    return Err(invalid(
                        "physical receipt source differs from its exact immutable lock",
                    ));
                }
                asset.payload().verify_materialization(
                    value,
                    counter_policy(
                        asset.revision().kind,
                        asset.revision().payload_schema_version,
                    )?,
                )?;
            }
        }
    }
    for op in operations {
        pool.apply(op, None)?;
    }
    let mut users = BTreeSet::new();
    let mut ci_new = BTreeMap::new();
    let mut packages = BTreeSet::new();
    let mut pads = BTreeMap::new();
    for op in operations {
        if let Operation::SetComponentInstance {
            component_instance_id,
            previous_component_instance,
            component_instance,
        } = op
        {
            let current = latest_ci(prior, *component_instance_id)?;
            if current != *previous_component_instance
                || ci_intent(&current) != ci_intent(component_instance)
                || component_instance["object_revision"]
                    != json!(increment(
                        current["object_revision"]
                            .as_u64()
                            .ok_or_else(|| invalid("missing CI epoch"))?
                    )?)
                || current["placed_symbol_refs"] != component_instance["placed_symbol_refs"]
                || current["part_ref"] != component_instance["part_ref"]
                || !users.insert(*component_instance_id)
            {
                return Err(invalid(
                    "physical receipt changes component meaning or preimage",
                ));
            }
            if let Some(bindings) = current["library_bindings"].as_object() {
                for (key, old) in bindings {
                    let id: Uuid = serde_json::from_value(old["target_object_id"].clone())?;
                    let mut expected = old.clone();
                    if pool_changes.contains(&id) {
                        expected["pinned_object_revision"] =
                            json!(pool.objects[&id].object_revision);
                    }
                    if component_instance["library_bindings"][key] != expected {
                        return Err(invalid(
                            "physical receipt changes declared native pins beyond the exact source increment",
                        ));
                    }
                }
            }
            ci_new.insert(*component_instance_id, component_instance);
        }
    }
    let tip = prior.last().unwrap().transaction_id;
    for op in operations {
        let Operation::SetBoardPackagePackage {
            package_id,
            package_ref_id,
            previous_materialized,
            materialized,
        } = op
        else {
            continue;
        };
        if !packages.insert(*package_id) {
            return Err(invalid("physical receipt repeats a package"));
        }
        let (reference, current) = latest_package(prior, *package_id)
            .ok_or_else(|| invalid("physical receipt has no original package producer"))?;
        let old = canonical_pads(&current["component_pads"])?;
        canonical_pads(&previous_materialized["component_pads"])?;
        let new = canonical_pads(&materialized["component_pads"])?;
        let mut previous = previous_materialized.clone();
        previous
            .as_object_mut()
            .ok_or_else(|| invalid("invalid physical preimage"))?
            .remove("pad_source_transition");
        if reference != *package_ref_id || current != previous {
            return Err(invalid(
                "physical receipt package preimage is not its actual recorded source",
            ));
        }
        let mut retained = current.clone();
        let mut proposed = materialized.clone();
        for value in [&mut retained, &mut proposed] {
            let fields = value
                .as_object_mut()
                .ok_or_else(|| invalid("invalid retained physical materialization"))?;
            fields.remove("component_pads");
            fields.remove("pad_source_transition");
        }
        if retained != proposed {
            return Err(invalid(
                "physical receipt changes retained graphics or other materialization",
            ));
        }
        let old: BTreeMap<_, _> = old.into_iter().map(|pad| (pad.uuid, pad)).collect();
        let (pose, current_epoch) = package_basis(prior, *package_id, proof)?;
        let pose: crate::board::PlacedPackage = serde_json::from_value(pose)?;
        if old.len() != new.len() || new.is_empty() {
            return Err(invalid("physical receipt changes Pad topology"));
        }
        for pad in new {
            pad.validate_frozen()?;
            let old = old
                .get(&pad.uuid)
                .ok_or_else(|| invalid("physical receipt replaces a Pad"))?;
            old.validate_frozen()?;
            let (Some(binding), Some(old_binding)) = (&pad.source_binding, &old.source_binding)
            else {
                return Err(invalid("physical receipt changes source authority"));
            };
            match (&old_binding.process_snapshot, &binding.process_snapshot) {
                (None, None) => {}
                (Some(old), Some(new))
                    if old.profile == new.profile
                        && old.capability == new.capability
                        && old.stackup == new.stackup
                        && new.transform_basis == pose => {}
                _ => {
                    return Err(invalid(
                        "physical receipt changes retained process/stackup/profile or package pose",
                    ));
                }
            }
            if binding.project_id != *project_id
                || binding.package_id != *package_id
                || binding.materialized_against != *before
                || binding.board_id != old_binding.board_id
                || binding.frozen_geometry.name != old_binding.frozen_geometry.name
                || binding.frozen_geometry.layer != old_binding.frozen_geometry.layer
                || binding.frozen_geometry.layer_span != old_binding.frozen_geometry.layer_span
                || binding.frozen_geometry.plated != old_binding.frozen_geometry.plated
            {
                return Err(invalid(
                    "physical receipt changes source ownership or layer meaning",
                ));
            }
            let mut expected = binding.clone();
            for (pin, now) in [
                (&mut expected.part, &old_binding.part),
                (&mut expected.package, &old_binding.package),
                (&mut expected.land_pattern, &old_binding.land_pattern),
                (&mut expected.library_pad, &old_binding.library_pad),
                (&mut expected.padstack, &old_binding.padstack),
            ] {
                if pin.object_id != now.object_id || pin.owner_id != now.owner_id {
                    return Err(invalid("physical receipt changes source identity"));
                }
                pool.pin(pin, now, tip)?;
            }
            let (Some(map), Some(old_map)) = (&mut expected.pin_pad_map, &old_binding.pin_pad_map)
            else {
                return Err(invalid("physical receipt changes mapping source"));
            };
            if map.object_id != old_map.object_id {
                return Err(invalid("physical receipt remaps pins"));
            }
            pool.pin(map, old_map, tip)?;
            if expected.part != binding.part
                || expected.package != binding.package
                || expected.land_pattern != binding.land_pattern
                || expected.library_pad != binding.library_pad
                || expected.padstack != binding.padstack
                || expected.pin_pad_map != binding.pin_pad_map
            {
                return Err(invalid(
                    "physical receipt source pins differ from canonical pool epochs",
                ));
            }
            let ci = binding
                .component_instance
                .as_ref()
                .ok_or_else(|| invalid("physical receipt has no CI"))?;
            let component = ci_new
                .get(&ci.object_id)
                .ok_or_else(|| invalid("physical receipt omitted component update"))?;
            if component["object_revision"] != json!(ci.object_revision)
                || old_binding
                    .component_instance
                    .as_ref()
                    .is_none_or(|old| old.object_id != ci.object_id)
                || component["placed_package_refs"]
                    != json!([{"object_id":package_id,"object_revision":increment(current_epoch.0)?}])
            {
                return Err(invalid(
                    "physical receipt component/package counter is not its actual producer epoch",
                ));
            }
            if let Some(value) = pool.values.get(&binding.land_pattern.relative_path) {
                let raw = &value["pads"][binding.library_pad.object_id.to_string()];
                if raw["position"]["x"] != json!(binding.frozen_geometry.position.x)
                    || raw["position"]["y"] != json!(binding.frozen_geometry.position.y)
                    || raw["name"] != json!(binding.frozen_geometry.name)
                    || raw["layer"] != json!(binding.frozen_geometry.layer)
                    || raw["padstack"] != json!(binding.padstack.object_id)
                {
                    return Err(invalid(
                        "physical receipt geometry differs from its Footprint",
                    ));
                }
            } else if binding.frozen_geometry.position != old_binding.frozen_geometry.position {
                return Err(invalid(
                    "physical receipt moves a Pad without an authenticated Footprint source",
                ));
            }
            if let Some(value) = pool.values.get(&binding.padstack.relative_path) {
                let stack: crate::pool::Padstack = serde_json::from_value(value.clone())?;
                if stack.aperture.as_ref() != Some(&binding.frozen_geometry.aperture)
                    || stack.drill_nm != binding.frozen_geometry.drill_nm
                {
                    return Err(invalid(
                        "physical receipt geometry differs from its Padstack",
                    ));
                }
            } else if binding.frozen_geometry.aperture != old_binding.frozen_geometry.aperture
                || binding.frozen_geometry.drill_nm != old_binding.frozen_geometry.drill_nm
            {
                return Err(invalid(
                    "physical receipt changes dimensions without an authenticated Padstack",
                ));
            }
            if pads.insert(pad.uuid, binding.clone()).is_some() {
                return Err(invalid("physical receipt duplicates a placed Pad"));
            }
        }
    }
    let physical_users: BTreeSet<_> = pads
        .values()
        .filter_map(|binding| binding.component_instance.as_ref().map(|ci| ci.object_id))
        .collect();
    if packages.is_empty() || physical_users.len() != packages.len() {
        return Err(invalid(
            "physical receipt lacks complete package/component users",
        ));
    }
    let users = physical_users;
    let epochs = pad_epochs(prior, &pads.keys().copied().collect())
        .ok_or_else(|| invalid("physical receipt Pad epochs missing"))?;
    let prior_records = latest_electrical(prior);
    let mut changed_records = BTreeSet::new();
    for op in operations {
        if let Operation::SetElectricalIdentity { previous, record } = op {
            let old = prior_records
                .get(&record.id)
                .ok_or_else(|| invalid("physical electrical preimage missing"))?;
            if old != previous.as_ref()
                || net_semantics(old, &users)? != net_semantics(record, &users)?
                || record.object_revision.0 != increment(old.object_revision.0)?
                || !changed_records.insert(record.id)
            {
                return Err(invalid(
                    "physical receipt changes electrical meaning or preimage",
                ));
            }
            let ElectricalIdentity::NetRelationship { evidence, .. } = &record.identity else {
                return Err(invalid("unsupported physical electrical record"));
            };
            for row in evidence
                .iter()
                .filter(|row| users.contains(&row.component_instance.object_id))
            {
                let binding = pads
                    .get(&row.board_pad.object_id)
                    .ok_or_else(|| invalid("physical receipt electrical Pad missing"))?;
                let map = binding.pin_pad_map.as_ref().unwrap();
                if row.component_instance != *binding.component_instance.as_ref().unwrap()
                    || row.board_pad.object_revision.0
                        != increment(epochs[&row.board_pad.object_id])?
                    || row.library_pad.as_ref().is_none_or(|pin| {
                        pin.object_id != binding.library_pad.object_id
                            || pin.object_revision != binding.library_pad.object_revision
                    })
                    || row.pin_pad_map.object_id != map.object_id
                    || row.pin_pad_map.object_revision != map.object_revision
                {
                    return Err(invalid(
                        "physical receipt electrical pins are not actual source epochs",
                    ));
                }
            }
        }
    }
    for record in prior_records.values() {
        if matches!(&record.identity,ElectricalIdentity::NetRelationship {evidence,..} if evidence.iter().any(|row|users.contains(&row.component_instance.object_id)))
            && !changed_records.contains(&record.id)
        {
            return Err(invalid(
                "physical receipt omitted a shared electrical relationship",
            ));
        }
    }
    if let Some(index) = memo {
        proof.retain_receipt(index);
    }
    Ok(())
}
