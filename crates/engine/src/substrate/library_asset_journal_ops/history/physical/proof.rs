//! Successful history proofs live only for one borrowed, immutable journal.
//! Identity checks precede reuse; prospective operations and altered copies
//! always take the complete qualification path.
use super::*;
use crate::substrate::RevisionedRef;

pub(in crate::substrate) struct Proof<'a> {
    journal: &'a [TransactionRecord],
    transactions: BTreeSet<usize>,
    receipts: BTreeSet<usize>,
    packages: BTreeMap<(usize, Uuid, u64), (Value, ObjectRevision)>,
    #[cfg(test)]
    receipt_evaluations: BTreeMap<usize, usize>,
    #[cfg(test)]
    package_evaluations: BTreeMap<(usize, Uuid, u64), usize>,
}

impl<'a> Proof<'a> {
    pub(in crate::substrate) fn new(journal: &'a [TransactionRecord]) -> Self {
        Self {
            journal,
            transactions: BTreeSet::new(),
            receipts: BTreeSet::new(),
            packages: BTreeMap::new(),
            #[cfg(test)]
            receipt_evaluations: BTreeMap::new(),
            #[cfg(test)]
            package_evaluations: BTreeMap::new(),
        }
    }

    fn prefix(&self, prior: &[TransactionRecord]) -> bool {
        prior.len() <= self.journal.len() && std::ptr::eq(prior.as_ptr(), self.journal.as_ptr())
    }

    fn transaction(&self, tx: &TransactionRecord, prior: &[TransactionRecord]) -> Option<usize> {
        (self.prefix(prior)
            && self
                .journal
                .get(prior.len())
                .is_some_and(|actual| std::ptr::eq(actual, tx)))
        .then_some(prior.len())
    }

    pub(super) fn receipt(
        &self,
        operations: &[Operation],
        inverse: &[Operation],
        prior: &[TransactionRecord],
        before: &ModelRevision,
    ) -> Option<usize> {
        let tx = self.journal.get(prior.len())?;
        (self.prefix(prior)
            && std::ptr::eq(tx.operations.as_slice(), operations)
            && std::ptr::eq(tx.inverse_operations.as_slice(), inverse)
            && &tx.before_model_revision == before)
            .then_some(prior.len())
    }

    pub(super) fn has_receipt(&mut self, index: usize) -> bool {
        let retained = self.receipts.contains(&index);
        #[cfg(test)]
        if !retained {
            *self.receipt_evaluations.entry(index).or_default() += 1;
        }
        retained
    }

    pub(super) fn retain_receipt(&mut self, index: usize) {
        self.receipts.insert(index);
    }

    pub(in crate::substrate) fn package(
        &mut self,
        prior: &[TransactionRecord],
        reference: &RevisionedRef,
    ) -> Result<(Value, ObjectRevision), EngineError> {
        let key = (
            prior.len(),
            reference.object_id,
            reference.object_revision.0,
        );
        let eligible = self.prefix(prior);
        if eligible && let Some(value) = self.packages.get(&key) {
            return Ok(value.clone());
        }
        #[cfg(test)]
        if eligible {
            *self.package_evaluations.entry(key).or_default() += 1;
        }
        let result = crate::substrate::component_instance::placed_package_history::recorded_package_with_proof(prior, reference, self)?;
        if eligible {
            self.packages.insert(key, result.clone());
        }
        Ok(result)
    }
}

pub(in crate::substrate) fn qualified_package_transaction(
    tx: &TransactionRecord,
    prior: &[TransactionRecord],
    package: Uuid,
) -> Result<bool, EngineError> {
    qualified_package_transaction_with_proof(tx, prior, package, &mut Proof::new(prior))
}

pub(in crate::substrate) fn qualified_package_transaction_with_proof(
    tx: &TransactionRecord,
    prior: &[TransactionRecord],
    package: Uuid,
    proof: &mut Proof<'_>,
) -> Result<bool, EngineError> {
    if !family(tx)
        || !tx.operations.iter().any(|op| matches!(op, Operation::SetBoardPackagePackage { package_id, .. } if *package_id == package))
    {
        return Ok(false);
    }
    if prior
        .iter()
        .any(|record| record.transaction_id == tx.transaction_id)
    {
        return Err(invalid(
            "physical receipt proof requires a strict earlier journal prefix",
        ));
    }
    let index = proof.transaction(tx, prior);
    if index.is_some_and(|index| proof.transactions.contains(&index)) {
        return Ok(true);
    }
    if tx.transaction_kind == TransactionKind::Normal {
        if tx.undo_of.is_some() || tx.redo_of.is_some() {
            return Err(invalid(
                "normal physical receipt carries a compensation link",
            ));
        }
    } else {
        let id = match tx.transaction_kind {
            TransactionKind::Undo => tx.undo_of,
            TransactionKind::Redo => tx.redo_of,
            _ => None,
        };
        let target = id
            .and_then(|id| prior.iter().find(|record| record.transaction_id == id))
            .ok_or_else(|| invalid("physical receipt compensation target is absent"))?;
        if !matches_with_proof(tx, target, prior, proof) {
            return Err(invalid(
                "physical receipt compensation is not the qualified selected inverse",
            ));
        }
    }
    receipt::verify(
        &tx.operations,
        &tx.inverse_operations,
        prior,
        &tx.before_model_revision,
        proof,
    )?;
    if let Some(index) = index {
        proof.transactions.insert(index);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a> Proof<'a> {
        // Called by the public sequential geometry workflow with its actual
        // committed journal, including both users and all six history families.
        pub(in crate::substrate) fn assert_cache_identity(journal: &'a [TransactionRecord]) {
            let mut proof = Self::new(journal);
            let physical: Vec<_> = journal
                .iter()
                .enumerate()
                .filter(|(_, tx)| family(tx))
                .collect();
            assert_eq!(physical.len(), 6);
            let mut packages = BTreeSet::new();
            for (index, tx) in &physical {
                for op in &tx.operations {
                    if let Operation::SetBoardPackagePackage { package_id, .. } = op {
                        packages.insert(*package_id);
                        assert!(
                            qualified_package_transaction_with_proof(
                                tx,
                                &journal[..*index],
                                *package_id,
                                &mut proof
                            )
                            .unwrap()
                        );
                    }
                }
            }
            assert_eq!(packages.len(), 2);
            for package in &packages {
                package_basis(journal, *package, &mut proof).unwrap();
            }
            assert_eq!(proof.transactions.len(), 6);
            assert_eq!(proof.receipts.len(), 6);
            assert!(proof.receipt_evaluations.values().all(|count| *count == 1));
            assert!(!proof.package_evaluations.is_empty());
            assert!(proof.package_evaluations.values().all(|count| *count == 1));
            let receipts = proof.receipt_evaluations.clone();
            let folds = proof.package_evaluations.clone();
            // Both package membership requests share whole-transaction proof;
            // repeated history requests perform no additional receipt or fold.
            for (index, tx) in &physical {
                for package in &packages {
                    assert!(
                        qualified_package_transaction_with_proof(
                            tx,
                            &journal[..*index],
                            *package,
                            &mut proof
                        )
                        .unwrap()
                    );
                }
            }
            for package in &packages {
                package_basis(journal, *package, &mut proof).unwrap();
            }
            assert_eq!(proof.receipt_evaluations, receipts);
            assert_eq!(proof.package_evaluations, folds);

            let (index, tx) = physical.last().unwrap();
            let prior = &journal[..*index];
            let package = *packages.first().unwrap();
            assert!(
                !qualified_package_transaction_with_proof(tx, prior, Uuid::new_v4(), &mut proof)
                    .unwrap()
            );
            let mut altered = (*tx).clone();
            altered.inverse_operations.pop();
            assert!(
                qualified_package_transaction_with_proof(&altered, prior, package, &mut proof)
                    .is_err()
            );
            let mut altered = (*tx).clone();
            altered.redo_of = Some(Uuid::new_v4());
            assert!(
                qualified_package_transaction_with_proof(&altered, prior, package, &mut proof)
                    .is_err()
            );
            let mut foreign = prior.to_vec();
            foreign.last_mut().unwrap().after_model_revision = tx.after_model_revision.clone();
            assert_ne!(
                foreign.last().unwrap().after_model_revision,
                tx.before_model_revision
            );
            assert!(
                qualified_package_transaction_with_proof(tx, &foreign, package, &mut proof)
                    .is_err()
            );

            let reference = RevisionedRef {
                object_id: package,
                object_revision: ObjectRevision(u64::MAX),
            };
            assert!(proof.package(journal, &reference).is_err());
            let reference = RevisionedRef {
                object_id: package,
                object_revision: package_basis(journal, package, &mut proof).unwrap().1,
            };
            let expected = proof.package(journal, &reference).unwrap();
            assert!(proof.package(&journal[..0], &reference).is_err());
            // Same length and IDs, but a different immutable journal lacking
            // the actual package birth must not borrow this root's fold proof.
            let mut foreign = journal.to_vec();
            for record in &mut foreign {
                record.operations.retain(|op| !matches!(op, Operation::CreateBoardPackage { package_id, .. } if *package_id == package));
            }
            assert!(proof.package(&foreign, &reference).is_err());
            assert_eq!(proof.package(journal, &reference).unwrap(), expected);
            assert!(
                qualified_package_transaction_with_proof(tx, prior, package, &mut proof).unwrap()
            );
        }
    }
}
