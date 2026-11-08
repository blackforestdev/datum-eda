//! Canonical operation projection shared by restored history and pending-batch
//! admission. It never writes sources or manufactures a transaction record.
use super::*;

pub(super) struct RevisionProjection<'a> {
    id: ObjectId,
    path: &'a str,
    kind: &'a str,
    requested: ObjectRevision,
    pub state: Option<Value>,
    pub counter: u64,
    pub captured: Option<Value>,
}
impl<'a> RevisionProjection<'a> {
    pub fn new(
        id: ObjectId,
        path: &'a str,
        kind: &'a str,
        requested: ObjectRevision,
        state: Option<Value>,
        counter: u64,
    ) -> Result<Self, EngineError> {
        let mut projection = Self {
            id,
            path,
            kind,
            requested,
            state,
            counter,
            captured: None,
        };
        if let Some(value) = &projection.state {
            projection.qualify(path, kind, value)?;
        }
        projection.capture()?;
        Ok(projection)
    }
    fn qualify(&self, path: &str, kind: &str, value: &Value) -> Result<(), EngineError> {
        if path != self.path || kind != self.kind {
            return Err(invalid("restored source identity differs"));
        }
        super::super::super::pool_journal_ops::validate_pool_library_object(
            self.id, path, kind, value,
        )
    }
    fn capture(&mut self) -> Result<(), EngineError> {
        if self.counter == self.requested.0
            && let Some(value) = &self.state
        {
            capture(&mut self.captured, value)?;
        }
        Ok(())
    }
    pub fn apply(&mut self, operation: &Operation) -> Result<(), EngineError> {
        match operation {
            Operation::CreatePoolLibraryObject {
                object_id,
                relative_path,
                object_kind,
                object,
            } if *object_id == self.id => {
                self.qualify(relative_path, object_kind, object)?;
                if self.state.is_some() {
                    return Err(invalid("creation already has a live definition"));
                }
                self.counter = super::super::super::object_revision_for(object).0;
                self.state = Some(object.clone());
            }
            Operation::SetPoolLibraryObject {
                object_id,
                relative_path,
                object_kind,
                previous_object,
                object,
            } if *object_id == self.id => {
                self.qualify(relative_path, object_kind, previous_object)?;
                self.qualify(relative_path, object_kind, object)?;
                if self.state.as_ref() != Some(previous_object) {
                    return Err(invalid("edit preimage differs from its qualified basis"));
                }
                self.counter = self
                    .counter
                    .checked_add(1)
                    .ok_or_else(|| invalid("revision overflow"))?;
                self.state = Some(object.clone());
            }
            Operation::BumpObjectRevision { object_id } if *object_id == self.id => {
                if self.state.is_none() {
                    return Err(invalid("bump has no live definition"));
                }
                self.counter = self
                    .counter
                    .checked_add(1)
                    .ok_or_else(|| invalid("revision overflow"))?;
            }
            Operation::DeletePoolLibraryObject {
                object_id,
                relative_path,
                object_kind,
                object,
            } if *object_id == self.id => {
                self.qualify(relative_path, object_kind, object)?;
                if self.state.as_ref() != Some(object) {
                    return Err(invalid(
                        "deletion preimage differs from its qualified basis",
                    ));
                }
                self.state = None;
            }
            _ => return Ok(()),
        }
        self.capture()
    }
}
impl PoolHistory<'_> {
    pub(super) fn restored_payload(
        &self,
        object: &DomainObject,
        requested: ObjectRevision,
        path: &str,
    ) -> Result<Option<Value>, EngineError> {
        let mut projection =
            RevisionProjection::new(object.object_id, path, &object.kind, requested, None, 0)?;
        for operation in self.journal.iter().flat_map(|tx| &tx.operations) {
            projection.apply(operation)?;
        }
        if projection.state.is_none() || projection.counter != object.object_revision.0 {
            return Err(invalid(
                "restored history differs from the resolved revision",
            ));
        }
        Ok(projection.captured)
    }
}
