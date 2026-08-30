use std::collections::BTreeSet;

use super::{AuthorityRecord, AuthorityRecordKind, AuthorityRef, AuthoritySnapshot};

impl AuthoritySnapshot {
    pub fn record(&self, reference: AuthorityRef) -> Option<&AuthorityRecord> {
        self.records
            .iter()
            .find(|record| record.record_ref() == reference)
    }

    pub fn records_of_kind(&self, kind: AuthorityRecordKind) -> Vec<&AuthorityRecord> {
        self.records
            .iter()
            .filter(|record| record.kind() == kind)
            .collect()
    }

    pub fn as_of(&self, sequence: u64) -> Vec<&AuthorityRecord> {
        let refs: BTreeSet<_> = self
            .events
            .iter()
            .filter(|event| event.sequence <= sequence)
            .map(|event| event.record)
            .collect();
        self.records
            .iter()
            .filter(|record| refs.contains(&record.record_ref()))
            .collect()
    }
}
