use super::{EngineError, PROPOSAL_SCHEMA_VERSION, Proposal, ResolveDiagnostic};

pub(super) fn validate_proposal_payload_schema_version(
    proposal: &Proposal,
) -> Result<(), EngineError> {
    if proposal.schema_version == PROPOSAL_SCHEMA_VERSION {
        return Ok(());
    }
    Err(EngineError::Validation(format!(
        "unsupported Proposal schema_version {}",
        proposal.schema_version
    )))
}

pub(super) fn proposal_payload_schema_diagnostic(
    proposal: &Proposal,
    path: Option<std::path::PathBuf>,
) -> Option<ResolveDiagnostic> {
    validate_proposal_payload_schema_version(proposal)
        .err()
        .map(|error| ResolveDiagnostic {
            code: "invalid_proposal_metadata".to_string(),
            message: error.to_string(),
            path,
        })
}

pub(super) fn is_sha256_fingerprint(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}
