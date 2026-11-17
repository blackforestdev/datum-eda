//! Recorded pool source/object projection and exact same-birth pin rebasing.
use super::*;

#[derive(Default)]
pub(super) struct RecordedPool {
    pub(super) objects: BTreeMap<Uuid, DomainObject>,
    pub(super) values: BTreeMap<String, Value>,
    pub(super) births: BTreeMap<Uuid, Option<Uuid>>,
}
impl RecordedPool {
    pub(super) fn apply(&mut self, op: &Operation, birth: Option<Uuid>) -> Result<(), EngineError> {
        let (id, path, previous, next, create, delete) = match op {
            Operation::CreatePoolLibraryObject {
                object_id,
                relative_path,
                object,
                ..
            } => (*object_id, relative_path, None, Some(object), true, false),
            Operation::SetPoolLibraryObject {
                object_id,
                relative_path,
                previous_object,
                object,
                ..
            } => (
                *object_id,
                relative_path,
                Some(previous_object),
                Some(object),
                false,
                false,
            ),
            Operation::DeletePoolLibraryObject {
                object_id,
                relative_path,
                object,
                ..
            } => (*object_id, relative_path, Some(object), None, false, true),
            _ => return Ok(()),
        };
        if !self.values.contains_key(path) && !create {
            // An actual recorded native preimage witnesses an initially
            // file-owned source. Calling the canonical registration primitive
            // seeds its object map; it does not invent a creation transaction.
            let raw =
                previous.ok_or_else(|| invalid("physical source has no recorded preimage"))?;
            let kind = path
                .split('/')
                .nth(1)
                .ok_or_else(|| invalid("invalid physical source path"))?;
            crate::substrate::operation_application_pool_objects::create(
                &mut self.objects,
                None,
                id,
                path,
                raw,
                kind,
            )?;
            self.values.insert(path.clone(), raw.clone());
            for object in self
                .objects
                .values()
                .filter(|o| o.source_shard_id == self.objects[&id].source_shard_id)
            {
                self.births.entry(object.object_id).or_insert(None);
            }
        }
        if let Some(previous) = previous {
            if self.values.get(path) != Some(previous) {
                return Err(invalid("recorded physical pool preimage differs"));
            }
        }
        let old: BTreeSet<_> = self.objects.keys().copied().collect();
        apply_operation_to_objects(&mut self.objects, op, None)?;
        for id in old.difference(&self.objects.keys().copied().collect()) {
            self.births.remove(id);
        }
        for id in self.objects.keys().filter(|id| !old.contains(id)) {
            self.births.insert(*id, birth);
        }
        if create {
            for id in self
                .objects
                .values()
                .filter(|o| o.source_shard_id == self.objects[&id].source_shard_id)
                .map(|o| o.object_id)
            {
                self.births.insert(id, birth);
            }
        }
        if delete {
            self.values.remove(path);
        } else {
            self.values.insert(path.clone(), next.unwrap().clone());
        }
        Ok(())
    }
    pub(super) fn history(prior: &[TransactionRecord]) -> Result<Self, EngineError> {
        let mut pool = Self::default();
        for tx in prior {
            for op in &tx.operations {
                pool.apply(op, Some(tx.transaction_id))?;
            }
        }
        Ok(pool)
    }
    pub(super) fn pin(
        &self,
        pin: &mut PinnedPadLibrarySource,
        current: &PinnedPadLibrarySource,
        tip: Uuid,
    ) -> Result<(), EngineError> {
        if let Some(value) = self.values.get(&pin.relative_path) {
            let object = self
                .objects
                .get(&pin.object_id)
                .ok_or_else(|| invalid("compensated library source is absent"))?;
            if object.source_shard_id != pin.source_shard_id
                || object.domain != "pool"
                || value["uuid"] != json!(pin.owner_id)
                || self.births.get(&pin.object_id).copied().flatten() != pin.incarnation
            {
                return Err(invalid(
                    "compensation cannot restore a retired or reborn library source",
                ));
            }
            let record = if pin.object_id == pin.owner_id {
                value
            } else {
                value
                    .get("pads")
                    .and_then(|pads| pads.get(pin.object_id.to_string()))
                    .ok_or_else(|| invalid("compensated library Pad is absent"))?
            };
            pin.object_revision = object.object_revision;
            pin.owner_value_hash = crate::substrate::component_pad_source::hash_value(value)?;
            pin.record_hash = crate::substrate::component_pad_source::hash_value(record)?;
        } else {
            let mut old = pin.clone();
            let mut now = current.clone();
            old.journal_tip = None;
            now.journal_tip = None;
            if old != now {
                return Err(invalid(
                    "unchanged source pin has no exact recorded current witness",
                ));
            }
        }
        pin.journal_tip = Some(tip);
        Ok(())
    }
}
