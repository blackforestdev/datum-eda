//! Technical integrity and crash-safe local storage for Revision authority.
//!
//! The technical store, typed authority vocabulary, and Project-owned policy
//! evaluation live here. Lifecycle transitions, issuance, and public workflow
//! remain later slices.

mod approval;
mod authority;
mod authority_store;
mod backup;
mod canonical;
mod effectivity;
mod model;
mod policy;
mod resolver;
mod role;
mod scheme;
mod seed;
mod store;
#[cfg(test)]
mod test_support;
#[allow(dead_code)]
mod transaction;

pub use approval::*;
pub use authority::*;
pub use effectivity::*;
pub use policy::*;
pub use role::*;
pub use scheme::*;
pub use seed::*;

pub use backup::{
    BackupEntry, BackupManifest, RestorePreview, RestoreReceipt, export_backup, preview_restore,
    restore_backup, undo_restore, verify_backup,
};
pub use model::{
    AffectedShard, AffectedShardAction, AlgorithmQualifiedDigest, IntegrityDiagnostic,
    IntegrityGeneration, IntegrityHead, REVISION_STORE_SCHEMA_VERSION, RevisionStoreState,
    TechnicalValidationState,
};
pub use resolver::AuthorityResolution;
pub use store::{
    ProjectWriteLease, RevisionAuthorityStore, StagedIntegrityGeneration, transaction_tip,
};

pub(crate) use model::StagedShardPostimage;
pub(crate) use store::IntegrityCommitFaultPoint;
#[cfg(test)]
pub(crate) use transaction::{
    REV_I03_MUTATIONS, REV_I03_QUERIES, RevisionMutation, RevisionMutationKind, RevisionQueryKind,
    apply_revision_mutations, consume_seed_mutation,
};
