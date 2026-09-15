use crate::error::EngineError;
use serde::Serialize;

use super::{
    AUTHORITY_SCHEMA_VERSION, AlgorithmQualifiedDigest, AuthorityRecord, AuthorityRecordBody,
    AuthorityRef, AuthoritySnapshot, ReleaseConfigurationRequest, ReleaseId,
    TitleBlockRevisionProjection, TransmittalData, TransmittalId,
    canonical::{canonical_bytes, digest_bytes, validate_digest},
};

pub fn prepare_transmittal(
    snapshot: &AuthoritySnapshot,
    id: TransmittalId,
    data: TransmittalData,
) -> Result<AuthoritySnapshot, EngineError> {
    let package = snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::ReleasePackage(body) if body.id == data.release_package => Some(body),
        _ => None,
    });
    require(
        package.is_some_and(|body| body.semantics.manifest_digest == data.exact_package_digest),
        "floating_release_input: transmittal must bind the exact package manifest digest",
    )?;
    let mut prepared = snapshot.clone();
    super::transaction::append(
        &mut prepared,
        AuthorityRecord::Transmittal(AuthorityRecordBody {
            schema_version: AUTHORITY_SCHEMA_VERSION,
            id,
            project_id: snapshot.project_id,
            display_name: data.transmittal_number.clone(),
            physical_locator: None,
            references: vec![
                AuthorityRef::ReleasePackage(data.release_package),
                data.delivery_policy_ref,
            ],
            semantics: data,
        }),
    )?;
    require(
        prepared.validate().is_empty(),
        "transmittal_failed_validation",
    )?;
    Ok(prepared)
}

pub fn project_title_block_revision(
    snapshot: &AuthoritySnapshot,
    issue_id: super::DocumentIssueId,
) -> Result<TitleBlockRevisionProjection, EngineError> {
    let issue = snapshot
        .records
        .iter()
        .find_map(|record| match record {
            AuthorityRecord::DocumentIssue(body) if body.id == issue_id => Some(body),
            _ => None,
        })
        .ok_or_else(|| refusal("document_issue_missing", "DocumentIssue does not resolve"))?;
    let document = snapshot
        .records
        .iter()
        .find_map(|record| match record {
            AuthorityRecord::ControlledDocument(body)
                if body.id == issue.semantics.controlled_document =>
            {
                Some(body)
            }
            _ => None,
        })
        .ok_or_else(|| {
            refusal(
                "controlled_document_missing",
                "ControlledDocument does not resolve",
            )
        })?;
    let revision = snapshot
        .records
        .iter()
        .find_map(|record| match record {
            AuthorityRecord::EngineeringRevision(body)
                if body.id == issue.semantics.engineering_revision =>
            {
                Some(body)
            }
            _ => None,
        })
        .ok_or_else(|| {
            refusal(
                "engineering_revision_missing",
                "EngineeringRevision does not resolve",
            )
        })?;
    require(
        revision.semantics.configuration_item == document.semantics.owning_configuration_item,
        "document_revision_owner_mismatch",
    )?;
    Ok(TitleBlockRevisionProjection {
        controlled_document: issue.semantics.controlled_document,
        document_number: document.semantics.document_number.clone(),
        configuration_item: revision.semantics.configuration_item,
        revision_label: revision.semantics.revision_label.clone(),
        suitability_or_status: revision.semantics.suitability_or_status.clone(),
        baseline: issue.semantics.baseline,
        release: issue.semantics.issued_by_release,
    })
}

pub fn require_publish_source_unbound(
    snapshot: &AuthoritySnapshot,
    source: AuthorityRef,
) -> Result<(), EngineError> {
    if snapshot.records.iter().any(|record| {
        matches!(record, AuthorityRecord::ControlledDocument(body) if body.semantics.source_ref == source)
    }) {
        Err(refusal(
            "bound_publish_source",
            "retarget or remove every ControlledDocument dependency before deleting its Publish source",
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn document_issue_references(
    issue: &super::PreparedDocumentIssue,
    request: &ReleaseConfigurationRequest,
) -> Vec<AuthorityRef> {
    let mut refs = vec![
        AuthorityRef::ControlledDocument(issue.controlled_document),
        AuthorityRef::EngineeringRevision(issue.engineering_revision),
        AuthorityRef::ConfigurationBaseline(request.baseline_id),
        AuthorityRef::Release(request.release_id),
    ];
    refs.extend(
        issue
            .approvals
            .iter()
            .copied()
            .map(AuthorityRef::ApprovalAttestation),
    );
    refs.extend(
        issue
            .source_revisions
            .iter()
            .map(|source| source.authority_ref),
    );
    refs.extend(issue.outputs.iter().map(|output| output.artifact_ref));
    refs
}

pub(crate) fn package_references(
    package: &super::PreparedReleasePackage,
    request: &ReleaseConfigurationRequest,
) -> Vec<AuthorityRef> {
    let mut refs = vec![AuthorityRef::Release(request.release_id)];
    refs.extend(package.ordered_member_refs.iter().copied());
    refs.extend(
        package
            .exact_artifacts
            .iter()
            .map(|artifact| artifact.artifact_ref),
    );
    refs
}

#[derive(Serialize)]
struct ReleasePackageManifestMaterial<'a> {
    release: ReleaseId,
    ordered_member_refs: &'a [AuthorityRef],
    exact_artifacts: &'a [super::ExactOutput],
    package_metadata_digest: &'a AlgorithmQualifiedDigest,
}

pub(crate) fn release_package_manifest_digest(
    release: ReleaseId,
    ordered_member_refs: &[AuthorityRef],
    exact_artifacts: &[super::ExactOutput],
    package_metadata_digest: &AlgorithmQualifiedDigest,
) -> Result<AlgorithmQualifiedDigest, EngineError> {
    validate_digest(package_metadata_digest)?;
    for artifact in exact_artifacts {
        validate_digest(&artifact.byte_digest)?;
    }
    Ok(digest_bytes(&canonical_bytes(
        &ReleasePackageManifestMaterial {
            release,
            ordered_member_refs,
            exact_artifacts,
            package_metadata_digest,
        },
    )?))
}

fn refusal(code: &str, message: &str) -> EngineError {
    EngineError::Validation(format!("{code}: {message}"))
}

fn require(condition: bool, message: &str) -> Result<(), EngineError> {
    if condition {
        Ok(())
    } else {
        Err(EngineError::Validation(message.to_string()))
    }
}
