use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::EngineError;

use super::{
    AlgorithmQualifiedDigest, AuthorityDiagnostic, AuthorityRecord, AuthorityRecordKind,
    AuthorityRef, DependencySnapshotId, EvidenceInputContextId, ExactAuthorityRevision,
    RegenerationPlanData, ReleaseId, ReproductionManifestId,
    canonical::{canonical_bytes, digest_bytes, validate_digest},
};

pub const REV_I07_RECORD_FAMILIES: &[AuthorityRecordKind] = &[
    AuthorityRecordKind::ReproductionManifest,
    AuthorityRecordKind::ReproductionAttempt,
];

pub const REPRODUCTION_RESULTS: &[&str] = &[
    "byte_identical",
    "output_mismatch",
    "unavailable",
    "execution_failed",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProducerClass {
    DatumOwned,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerIdentity {
    pub authority_ref: AuthorityRef,
    pub exact_revision: String,
    pub executable_digest: AlgorithmQualifiedDigest,
    pub class: ProducerClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationIdentity {
    pub effective_settings: BTreeMap<String, String>,
    pub ordered_instructions: Vec<String>,
    pub invocation_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentIdentity {
    pub influential: BTreeMap<String, String>,
    pub explicit_exclusions: BTreeSet<String>,
    pub environment_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecifiedOutput {
    pub logical_name: String,
    pub kind: String,
    pub byte_count: u64,
    pub digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariancePolicy {
    pub explicitly_irrelevant_environment: BTreeSet<String>,
    pub byte_identity_required: bool,
    pub network_access_allowed: bool,
    pub undeclared_local_files_allowed: bool,
    pub clock_is_controlled: bool,
    pub randomness_is_controlled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReproductionManifestData {
    pub release: ReleaseId,
    pub source_subjects: Vec<ExactAuthorityRevision>,
    pub dependency_snapshot: DependencySnapshotId,
    pub producer: ProducerIdentity,
    pub invocation: InvocationIdentity,
    pub environment: EnvironmentIdentity,
    pub specified_outputs: Vec<SpecifiedOutput>,
    pub variance_policy: VariancePolicy,
    pub canonicalization_policy_refs: Vec<AuthorityRef>,
    pub created_at: i64,
    pub manifest_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducedOutput {
    pub logical_name: String,
    pub kind: String,
    pub byte_count: u64,
    pub digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "result")]
pub enum ReproductionResult {
    ByteIdentical,
    OutputMismatch { differences: Vec<String> },
    Unavailable { missing_requirements: Vec<String> },
    ExecutionFailed { findings: Vec<String> },
}

impl ReproductionResult {
    pub const ALL: &'static [&'static str] = REPRODUCTION_RESULTS;

    pub const fn wire_tag(&self) -> &'static str {
        match self {
            Self::ByteIdentical => REPRODUCTION_RESULTS[0],
            Self::OutputMismatch { .. } => REPRODUCTION_RESULTS[1],
            Self::Unavailable { .. } => REPRODUCTION_RESULTS[2],
            Self::ExecutionFailed { .. } => REPRODUCTION_RESULTS[3],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReproductionAttemptData {
    pub manifest: ReproductionManifestId,
    pub manifest_digest: AlgorithmQualifiedDigest,
    pub executor_identity: ProducerIdentity,
    pub attempted_at: i64,
    pub independently_resolved_environment: EnvironmentIdentity,
    pub produced_outputs: Vec<ProducedOutput>,
    pub result: ReproductionResult,
    pub comparison_evidence: Vec<String>,
    pub authenticity_evidence: Vec<AuthorityRef>,
    pub attempt_digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReproductionObservation {
    pub executor_identity: ProducerIdentity,
    pub independently_resolved_environment: EnvironmentIdentity,
    pub produced_outputs: Vec<ProducedOutput>,
    pub execution_findings: Vec<String>,
    pub unavailable_requirements: Vec<String>,
    pub comparison_evidence: Vec<String>,
    pub authenticity_evidence: Vec<AuthorityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegeneratedEvidence {
    pub output_contract: String,
    pub predecessor_evidence: Option<AuthorityRef>,
    pub successor_evidence: AuthorityRef,
    pub input_context: EvidenceInputContextId,
    pub digest: AlgorithmQualifiedDigest,
}

#[derive(Serialize)]
struct ManifestMaterial<'a> {
    release: ReleaseId,
    source_subjects: &'a [ExactAuthorityRevision],
    dependency_snapshot: DependencySnapshotId,
    producer: &'a ProducerIdentity,
    invocation: &'a InvocationIdentity,
    environment: &'a EnvironmentIdentity,
    specified_outputs: &'a [SpecifiedOutput],
    variance_policy: &'a VariancePolicy,
    canonicalization_policy_refs: &'a [AuthorityRef],
    created_at: i64,
}

pub fn reproduction_manifest_digest(
    data: &ReproductionManifestData,
) -> Result<AlgorithmQualifiedDigest, EngineError> {
    Ok(digest_bytes(&canonical_bytes(&ManifestMaterial {
        release: data.release,
        source_subjects: &data.source_subjects,
        dependency_snapshot: data.dependency_snapshot,
        producer: &data.producer,
        invocation: &data.invocation,
        environment: &data.environment,
        specified_outputs: &data.specified_outputs,
        variance_policy: &data.variance_policy,
        canonicalization_policy_refs: &data.canonicalization_policy_refs,
        created_at: data.created_at,
    })?))
}

#[derive(Serialize)]
struct AttemptMaterial<'a> {
    manifest: ReproductionManifestId,
    manifest_digest: &'a AlgorithmQualifiedDigest,
    executor_identity: &'a ProducerIdentity,
    attempted_at: i64,
    independently_resolved_environment: &'a EnvironmentIdentity,
    produced_outputs: &'a [ProducedOutput],
    result: &'a ReproductionResult,
    comparison_evidence: &'a [String],
    authenticity_evidence: &'a [AuthorityRef],
}

pub fn reproduction_attempt_digest(
    data: &ReproductionAttemptData,
) -> Result<AlgorithmQualifiedDigest, EngineError> {
    Ok(digest_bytes(&canonical_bytes(&AttemptMaterial {
        manifest: data.manifest,
        manifest_digest: &data.manifest_digest,
        executor_identity: &data.executor_identity,
        attempted_at: data.attempted_at,
        independently_resolved_environment: &data.independently_resolved_environment,
        produced_outputs: &data.produced_outputs,
        result: &data.result,
        comparison_evidence: &data.comparison_evidence,
        authenticity_evidence: &data.authenticity_evidence,
    })?))
}

pub fn create_reproduction_attempt(
    manifest_id: ReproductionManifestId,
    manifest: &ReproductionManifestData,
    attempted_at: i64,
    mut observation: ReproductionObservation,
) -> Result<ReproductionAttemptData, EngineError> {
    require(
        manifest.producer.class == ProducerClass::DatumOwned,
        "external_generator_refused: release reproduction requires a Datum-owned producer",
    )?;
    observation.produced_outputs.sort();
    observation.comparison_evidence.sort();
    observation.comparison_evidence.dedup();
    observation.authenticity_evidence.sort();
    observation.authenticity_evidence.dedup();
    let mut unavailable = observation.unavailable_requirements;
    if observation.executor_identity != manifest.producer {
        unavailable.push("producer_unavailable_or_changed".to_string());
    }
    for (name, expected) in &manifest.environment.influential {
        if observation
            .independently_resolved_environment
            .influential
            .get(name)
            != Some(expected)
        {
            unavailable.push(format!("environment_uncaptured_or_changed:{name}"));
        }
    }
    unavailable.sort();
    unavailable.dedup();
    let result = if !unavailable.is_empty() {
        ReproductionResult::Unavailable {
            missing_requirements: unavailable,
        }
    } else if !observation.execution_findings.is_empty() {
        let mut findings = observation.execution_findings;
        findings.sort();
        findings.dedup();
        ReproductionResult::ExecutionFailed { findings }
    } else {
        compare_outputs(&manifest.specified_outputs, &observation.produced_outputs)
    };
    let mut attempt = ReproductionAttemptData {
        manifest: manifest_id,
        manifest_digest: manifest.manifest_digest.clone(),
        executor_identity: observation.executor_identity,
        attempted_at,
        independently_resolved_environment: observation.independently_resolved_environment,
        produced_outputs: observation.produced_outputs,
        result,
        comparison_evidence: observation.comparison_evidence,
        authenticity_evidence: observation.authenticity_evidence,
        attempt_digest: AlgorithmQualifiedDigest(String::new()),
    };
    attempt.attempt_digest = reproduction_attempt_digest(&attempt)?;
    Ok(attempt)
}

fn compare_outputs(expected: &[SpecifiedOutput], actual: &[ProducedOutput]) -> ReproductionResult {
    let expected_by_name: BTreeMap<_, _> = expected
        .iter()
        .map(|output| ((output.logical_name.as_str(), output.kind.as_str()), output))
        .collect();
    let actual_by_name: BTreeMap<_, _> = actual
        .iter()
        .map(|output| ((output.logical_name.as_str(), output.kind.as_str()), output))
        .collect();
    let keys: BTreeSet<_> = expected_by_name
        .keys()
        .chain(actual_by_name.keys())
        .copied()
        .collect();
    let mut differences = Vec::new();
    for key in keys {
        match (expected_by_name.get(&key), actual_by_name.get(&key)) {
            (Some(expected), Some(actual))
                if expected.byte_count == actual.byte_count && expected.digest == actual.digest => {
            }
            (Some(_), Some(_)) => {
                differences.push(format!("output_bytes_differ:{}:{}", key.0, key.1))
            }
            (Some(_), None) => differences.push(format!("output_missing:{}:{}", key.0, key.1)),
            (None, Some(_)) => differences.push(format!("output_unexpected:{}:{}", key.0, key.1)),
            (None, None) => unreachable!(),
        }
    }
    if differences.is_empty() && expected.len() == actual.len() {
        ReproductionResult::ByteIdentical
    } else {
        ReproductionResult::OutputMismatch { differences }
    }
}

pub fn execute_regeneration_plan(
    plan: &RegenerationPlanData,
    current_contexts: &BTreeSet<EvidenceInputContextId>,
    current_reusable_evidence: &BTreeSet<AuthorityRef>,
    mut execute: impl FnMut(&super::RegenerationRequest) -> Result<RegeneratedEvidence, EngineError>,
) -> Result<Vec<RegeneratedEvidence>, EngineError> {
    require(plan.blockers.is_empty(), "regeneration_plan_blocked")?;
    require(
        plan.unchanged_evidence_reused
            .iter()
            .all(|evidence| current_reusable_evidence.contains(evidence)),
        "stale_evidence_reuse_refused",
    )?;
    let mut results = Vec::new();
    for step in &plan.steps {
        require(
            current_contexts.contains(&step.expected_input_context),
            "regeneration_input_context_stale",
        )?;
        let result = execute(step)?;
        require(
            result.output_contract == step.output_contract
                && result.predecessor_evidence == step.predecessor_evidence
                && result.input_context == step.expected_input_context
                && Some(result.successor_evidence) != step.predecessor_evidence,
            "regeneration_must_create_exact_successor_evidence",
        )?;
        validate_digest(&result.digest)?;
        results.push(result);
    }
    Ok(results)
}

pub(crate) fn validate_rev_i07_record(record: &AuthorityRecord) -> Vec<AuthorityDiagnostic> {
    let valid = match record {
        AuthorityRecord::ReproductionManifest(body) => validate_manifest(&body.semantics),
        AuthorityRecord::ReproductionAttempt(body) => validate_attempt(&body.semantics),
        _ => return Vec::new(),
    };
    if valid {
        Vec::new()
    } else {
        vec![diagnostic()]
    }
}

fn validate_manifest(data: &ReproductionManifestData) -> bool {
    data.producer.class == ProducerClass::DatumOwned
        && !data.producer.exact_revision.is_empty()
        && !data.source_subjects.is_empty()
        && !data.specified_outputs.is_empty()
        && is_sorted_unique(&data.source_subjects)
        && is_sorted_unique(&data.specified_outputs)
        && is_sorted_unique(&data.canonicalization_policy_refs)
        && data.variance_policy.byte_identity_required
        && !data.variance_policy.network_access_allowed
        && !data.variance_policy.undeclared_local_files_allowed
        && data.variance_policy.clock_is_controlled
        && data.variance_policy.randomness_is_controlled
        && !data.producer.exact_revision.eq_ignore_ascii_case("latest")
        && data.source_subjects.iter().all(|source| {
            !source
                .exact_technical_revision
                .eq_ignore_ascii_case("latest")
        })
        && reproduction_manifest_digest(data).is_ok_and(|digest| data.manifest_digest == digest)
        && validate_digest(&data.producer.executable_digest).is_ok()
        && validate_digest(&data.invocation.invocation_digest).is_ok()
        && validate_digest(&data.environment.environment_digest).is_ok()
        && data.specified_outputs.iter().all(|output| {
            !output.logical_name.is_empty()
                && !output.kind.is_empty()
                && validate_digest(&output.digest).is_ok()
        })
}

fn validate_attempt(data: &ReproductionAttemptData) -> bool {
    let result_is_valid = match &data.result {
        ReproductionResult::ByteIdentical => true,
        ReproductionResult::OutputMismatch { differences } => !differences.is_empty(),
        ReproductionResult::Unavailable {
            missing_requirements,
        } => !missing_requirements.is_empty(),
        ReproductionResult::ExecutionFailed { findings } => !findings.is_empty(),
    };
    data.executor_identity.class == ProducerClass::DatumOwned
        && is_sorted_unique(&data.produced_outputs)
        && is_sorted_unique(&data.comparison_evidence)
        && is_sorted_unique(&data.authenticity_evidence)
        && reproduction_attempt_digest(data).is_ok_and(|digest| data.attempt_digest == digest)
        && result_is_valid
}

fn is_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn diagnostic() -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: "revision_reproduction_record_invalid".to_string(),
        message: "REV-I07 record violates exact immutable reproduction authority".to_string(),
    }
}

fn require(condition: bool, message: &str) -> Result<(), EngineError> {
    if condition {
        Ok(())
    } else {
        Err(EngineError::Validation(message.to_string()))
    }
}
