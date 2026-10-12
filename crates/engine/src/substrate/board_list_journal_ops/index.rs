//! One canonical board-application pass owns the lazy array lookup state.
use super::super::{EngineError, Operation};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(in crate::substrate) struct BoardPass<'a> {
    board: &'a mut Value,
    arrays: BTreeMap<&'static str, BTreeMap<Uuid, BTreeSet<usize>>>,
}
impl<'a> BoardPass<'a> {
    pub(in crate::substrate) fn new(board: &'a mut Value) -> Self {
        Self {
            board,
            arrays: BTreeMap::new(),
        }
    }
    pub(in crate::substrate) fn board(&self) -> &Value {
        self.board
    }
    pub(in crate::substrate) fn board_mut(&mut self) -> &mut Value {
        self.arrays.clear();
        self.board
    }
    pub(in crate::substrate) fn replay_noncore(
        &mut self,
        operation: &Operation,
    ) -> Result<bool, EngineError> {
        let changed = super::super::journal_operation_hooks::apply_non_core_shard_operation(
            &super::super::SourceShardKind::BoardRoot,
            self.board,
            operation,
        )?;
        if changed {
            self.arrays.clear();
        }
        Ok(changed)
    }
    pub(in crate::substrate) fn apply(
        &mut self,
        operation: &Operation,
    ) -> Result<bool, EngineError> {
        if let Some(applied) = super::apply_indexed(self, operation)? {
            return Ok(applied);
        }
        self.invalidate_nonlist(operation);
        super::super::board_journal_ops::apply_board_operation(self.board, operation)
    }
    pub(in crate::substrate) fn inverse(
        &mut self,
        operation: &Operation,
        inverse: &mut Vec<Operation>,
    ) -> Result<(), EngineError> {
        if super::inverse_indexed(self, operation, inverse)? {
            return Ok(());
        }
        self.invalidate_nonlist(operation);
        super::super::board_journal_ops::inverse_board_operation(self.board, operation, inverse)
    }
    fn invalidate_nonlist(&mut self, operation: &Operation) {
        // Revision guards/bumps never alter payload arrays. Other owners may
        // adopt or replace a source representation: do not retain stale offsets.
        if !matches!(
            operation,
            Operation::GuardObjectRevision { .. } | Operation::BumpObjectRevision { .. }
        ) {
            self.arrays.clear();
        }
    }
    fn array(&self, name: &str) -> Result<&Vec<Value>, EngineError> {
        self.board
            .get(name)
            .and_then(Value::as_array)
            .ok_or_else(|| EngineError::Validation(format!("board shard missing {name} array")))
    }
    fn array_mut(&mut self, name: &str) -> Result<&mut Vec<Value>, EngineError> {
        self.board
            .get_mut(name)
            .and_then(Value::as_array_mut)
            .ok_or_else(|| EngineError::Validation(format!("board shard missing {name} array")))
    }
    fn index(&mut self, name: &'static str, id: Uuid) -> Result<usize, EngineError> {
        if !self.arrays.contains_key(name) {
            let mut occurrences = BTreeMap::<Uuid, BTreeSet<usize>>::new();
            for (position, value) in self.array(name)?.iter().enumerate() {
                if let Some(id) = value_uuid(value) {
                    occurrences.entry(id).or_default().insert(position);
                }
            }
            self.arrays.insert(name, occurrences);
        }
        self.arrays[name]
            .get(&id)
            .and_then(|positions| positions.first().copied())
            .ok_or(EngineError::NotFound {
                object_type: "board_array_object",
                uuid: id,
            })
    }
    pub(super) fn value(&mut self, name: &'static str, id: Uuid) -> Result<&Value, EngineError> {
        let position = self.index(name, id)?;
        Ok(&self.array(name)?[position])
    }
    fn forget(&mut self, name: &'static str, id: Option<Uuid>, position: usize) {
        if let Some(id) = id {
            let entries = self.arrays.get_mut(name).expect("array index built");
            if let Some(positions) = entries.get_mut(&id) {
                positions.remove(&position);
                if positions.is_empty() {
                    entries.remove(&id);
                }
            }
        }
    }
    fn remember(&mut self, name: &'static str, id: Option<Uuid>, position: usize) {
        if let Some(id) = id {
            self.arrays
                .get_mut(name)
                .expect("array index built")
                .entry(id)
                .or_default()
                .insert(position);
        }
    }
    pub(super) fn insert(
        &mut self,
        name: &'static str,
        id: Uuid,
        value: Value,
    ) -> Result<(), EngineError> {
        if self.index(name, id).is_ok() {
            return Ok(());
        }
        let payload_id = value_uuid(&value);
        let array = self.array_mut(name)?;
        let position = array.len();
        array.push(value);
        self.remember(name, payload_id, position);
        Ok(())
    }
    pub(super) fn replace(
        &mut self,
        name: &'static str,
        id: Uuid,
        value: Value,
    ) -> Result<(), EngineError> {
        let position = self.index(name, id)?;
        let old_id = value_uuid(&self.array(name)?[position]);
        let new_id = value_uuid(&value);
        self.array_mut(name)?[position] = value;
        self.forget(name, old_id, position);
        self.remember(name, new_id, position);
        Ok(())
    }
    pub(super) fn remove(&mut self, name: &'static str, id: Uuid) -> Result<Value, EngineError> {
        let position = self.index(name, id)?;
        let array = self.array_mut(name)?;
        let tail = position + 1 == array.len();
        let value = array.remove(position);
        if tail {
            self.forget(name, value_uuid(&value), position);
        } else {
            self.arrays.remove(name);
        }
        Ok(value)
    }
}
fn value_uuid(value: &Value) -> Option<Uuid> {
    value
        .get("uuid")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
}
#[cfg(test)]
mod tests;
