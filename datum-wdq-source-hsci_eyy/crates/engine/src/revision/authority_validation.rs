use std::collections::BTreeSet;

use super::{AuthorityDiagnostic, AuthorityRecord, AuthoritySnapshot};

pub(crate) fn validate_record(
    snapshot: &AuthoritySnapshot,
    record: &AuthorityRecord,
) -> Vec<AuthorityDiagnostic> {
    match record {
        AuthorityRecord::ProjectRevisionPolicy(body) => {
            super::policy::validate_policy_data(&body.semantics)
        }
        AuthorityRecord::RevisionScheme(body) => {
            super::scheme::validate_scheme_data(&body.semantics)
        }
        AuthorityRecord::ActorIdentity(body) if body.semantics.stable_name.is_empty() => {
            vec![diag(
                "revision_actor_identity_empty",
                "actor stable name cannot be empty",
            )]
        }
        AuthorityRecord::Effectivity(body) => {
            super::effectivity::validate_effectivity(&body.semantics)
        }
        AuthorityRecord::ProjectSeedReceipt(body) => validate_seed_receipt(body),
        AuthorityRecord::EngineeringChange(body) => {
            super::change::validate_engineering_change(&body.semantics)
        }
        AuthorityRecord::RevisionReservation(body) => {
            super::reservation::validate_revision_reservation(&body.semantics)
        }
        AuthorityRecord::WaiverDeparture(body) => {
            super::departure::validate_waiver_departure(&body.semantics)
        }
        AuthorityRecord::DeviationDeparture(body) => {
            super::departure::validate_deviation_departure(&body.semantics)
        }
        AuthorityRecord::LegacyRevisionFactMapping(body) => {
            super::departure::validate_legacy_mapping(snapshot, &body.semantics)
        }
        record if super::impact::REV_I05_RECORD_FAMILIES.contains(&record.kind()) => {
            super::impact_analysis::validate_rev_i05_record(record)
        }
        record if super::release::REV_I06_RECORD_FAMILIES.contains(&record.kind()) => {
            super::release_transaction::validate_rev_i06_record(record)
        }
        record if super::reproduction::REV_I07_RECORD_FAMILIES.contains(&record.kind()) => {
            super::reproduction::validate_rev_i07_record(record)
        }
        record if super::exchange::REV_I08_RECORD_FAMILIES.contains(&record.kind()) => {
            super::exchange::validate_rev_i08_record(record)
        }
        _ => Vec::new(),
    }
}

fn validate_seed_receipt(
    body: &super::AuthorityRecordBody<super::ProjectSeedReceiptId, super::ProjectSeedReceiptData>,
) -> Vec<AuthorityDiagnostic> {
    let actual: BTreeSet<_> = body
        .semantics
        .items
        .iter()
        .map(|item| item.key.as_str())
        .collect();
    let expected: BTreeSet<_> = super::REVISION_SEED_KEYS.iter().copied().collect();
    if actual == expected && body.semantics.items.len() == expected.len() {
        Vec::new()
    } else {
        vec![diag(
            "revision_seed_receipt_inventory_mismatch",
            "seed receipt must itemize exactly the registered revision keys",
        )]
    }
}

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}
