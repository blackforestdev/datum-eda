use std::collections::{BTreeMap, BTreeSet};

use super::*;

fn id(value: u128) -> uuid::Uuid {
    uuid::Uuid::from_u128(value)
}

fn digest(bytes: &[u8]) -> AlgorithmQualifiedDigest {
    super::canonical::digest_bytes(bytes)
}

fn producer(class: ProducerClass) -> ProducerIdentity {
    ProducerIdentity {
        authority_ref: AuthorityRef::BuildIdentity(BuildIdentityId(id(1))),
        exact_revision: "datum-generator-1".into(),
        executable_digest: digest(b"generator"),
        class,
    }
}

fn environment(influential: &[(&str, &str)], excluded: &[&str]) -> EnvironmentIdentity {
    let influential = influential
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect::<BTreeMap<_, _>>();
    let explicit_exclusions = excluded
        .iter()
        .map(|value| (*value).to_string())
        .collect::<BTreeSet<_>>();
    let environment_digest = digest(
        &super::canonical::canonical_bytes(&(influential.clone(), explicit_exclusions.clone()))
            .unwrap(),
    );
    EnvironmentIdentity {
        influential,
        explicit_exclusions,
        environment_digest,
    }
}

fn specified(name: &str, kind: &str, bytes: &[u8]) -> SpecifiedOutput {
    SpecifiedOutput {
        logical_name: name.into(),
        kind: kind.into(),
        byte_count: bytes.len() as u64,
        digest: digest(bytes),
    }
}

fn produced(name: &str, kind: &str, bytes: &[u8]) -> ProducedOutput {
    ProducedOutput {
        logical_name: name.into(),
        kind: kind.into(),
        byte_count: bytes.len() as u64,
        digest: digest(bytes),
    }
}

fn manifest(outputs: Vec<SpecifiedOutput>) -> ReproductionManifestData {
    let source = AuthorityRef::EngineeringRevision(EngineeringRevisionId(id(2)));
    let mut data = ReproductionManifestData {
        release: ReleaseId(id(3)),
        source_subjects: vec![ExactAuthorityRevision {
            authority_ref: source,
            exact_technical_revision: "A".into(),
            digest: digest(b"source"),
        }],
        dependency_snapshot: DependencySnapshotId(id(4)),
        producer: producer(ProducerClass::DatumOwned),
        invocation: InvocationIdentity {
            effective_settings: BTreeMap::from([("units".into(), "mm".into())]),
            ordered_instructions: vec!["render".into(), "serialize".into()],
            invocation_digest: digest(b"invocation"),
        },
        environment: environment(
            &[
                ("timestamp_source", "source-date-epoch:0"),
                ("random_source", "seed:42"),
                ("ordering", "stable-id"),
            ],
            &["locale", "timezone", "working_directory"],
        ),
        specified_outputs: outputs,
        variance_policy: VariancePolicy {
            explicitly_irrelevant_environment: BTreeSet::from([
                "locale".into(),
                "timezone".into(),
                "working_directory".into(),
            ]),
            byte_identity_required: true,
            network_access_allowed: false,
            undeclared_local_files_allowed: false,
            clock_is_controlled: true,
            randomness_is_controlled: true,
        },
        canonicalization_policy_refs: Vec::new(),
        created_at: 10,
        manifest_digest: AlgorithmQualifiedDigest(String::new()),
    };
    data.manifest_digest = reproduction_manifest_digest(&data).unwrap();
    data
}

fn observation(
    manifest: &ReproductionManifestData,
    outputs: Vec<ProducedOutput>,
) -> ReproductionObservation {
    ReproductionObservation {
        executor_identity: manifest.producer.clone(),
        independently_resolved_environment: manifest.environment.clone(),
        produced_outputs: outputs,
        execution_findings: Vec::new(),
        unavailable_requirements: Vec::new(),
        comparison_evidence: vec!["algorithm-qualified byte digest comparison".into()],
        authenticity_evidence: Vec::new(),
    }
}

#[test]
fn rev_i07_closed_inventories_are_bidirectionally_exact() {
    assert_eq!(
        REV_I07_RECORD_FAMILIES,
        &[
            AuthorityRecordKind::ReproductionManifest,
            AuthorityRecordKind::ReproductionAttempt,
        ]
    );
    assert_eq!(
        ReproductionResult::ALL,
        &[
            "byte_identical",
            "output_mismatch",
            "unavailable",
            "execution_failed",
        ]
    );
    let variants = [
        ReproductionResult::ByteIdentical,
        ReproductionResult::OutputMismatch {
            differences: vec!["bytes".into()],
        },
        ReproductionResult::Unavailable {
            missing_requirements: vec!["producer".into()],
        },
        ReproductionResult::ExecutionFailed {
            findings: vec!["generator".into()],
        },
    ];
    assert_eq!(
        variants
            .iter()
            .map(ReproductionResult::wire_tag)
            .collect::<Vec<_>>(),
        ReproductionResult::ALL
    );
}

#[test]
fn independent_directories_and_irrelevant_locale_timezone_path_are_byte_identical() {
    let bytes = b"deterministic released bytes\n";
    let manifest = manifest(vec![specified("assembly", "pdf", bytes)]);
    let root_a = std::env::temp_dir().join(format!("datum-repro-a-{}", id(100)));
    let root_b = std::env::temp_dir().join(format!("datum-repro-b-{}", id(101)));
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    std::fs::write(root_a.join("assembly.pdf"), bytes).unwrap();
    std::fs::write(root_b.join("assembly.pdf"), bytes).unwrap();
    let mut first = observation(
        &manifest,
        vec![produced(
            "assembly",
            "pdf",
            &std::fs::read(root_a.join("assembly.pdf")).unwrap(),
        )],
    );
    first
        .independently_resolved_environment
        .explicit_exclusions
        .extend([
            "locale=C".into(),
            "timezone=UTC".into(),
            root_a.display().to_string(),
        ]);
    let mut second = observation(
        &manifest,
        vec![produced(
            "assembly",
            "pdf",
            &std::fs::read(root_b.join("assembly.pdf")).unwrap(),
        )],
    );
    second
        .independently_resolved_environment
        .explicit_exclusions
        .extend([
            "locale=fr_FR".into(),
            "timezone=Pacific/Auckland".into(),
            root_b.display().to_string(),
        ]);
    for attempt in [first, second] {
        assert_eq!(
            create_reproduction_attempt(ReproductionManifestId(id(5)), &manifest, 20, attempt,)
                .unwrap()
                .result,
            ReproductionResult::ByteIdentical
        );
    }
    std::fs::remove_dir_all(root_a).unwrap();
    std::fs::remove_dir_all(root_b).unwrap();
}

#[test]
fn controlled_timestamp_random_order_locale_and_path_leaks_fail_byte_identity() {
    let expected = b"stable\n";
    let manifest = manifest(vec![specified("board", "gerber", expected)]);
    for influence in ["timestamp", "random", "ordering", "locale", "path"] {
        let bytes = format!("stable\nleaked-{influence}\n");
        let attempt = create_reproduction_attempt(
            ReproductionManifestId(id(5)),
            &manifest,
            30,
            observation(
                &manifest,
                vec![produced("board", "gerber", bytes.as_bytes())],
            ),
        )
        .unwrap();
        assert!(matches!(
            attempt.result,
            ReproductionResult::OutputMismatch { ref differences }
                if differences == &["output_bytes_differ:board:gerber"]
        ));
    }
}

#[test]
fn missing_producer_environment_execution_failure_and_external_generator_are_distinct() {
    let manifest = manifest(vec![specified("bom", "csv", b"bom")]);
    let mut missing_producer = observation(&manifest, vec![produced("bom", "csv", b"bom")]);
    missing_producer.executor_identity.exact_revision = "missing".into();
    assert!(matches!(
        create_reproduction_attempt(
            ReproductionManifestId(id(5)),
            &manifest,
            40,
            missing_producer
        )
        .unwrap()
        .result,
        ReproductionResult::Unavailable { .. }
    ));
    let mut missing_environment = observation(&manifest, vec![produced("bom", "csv", b"bom")]);
    missing_environment
        .independently_resolved_environment
        .influential
        .remove("random_source");
    assert!(matches!(
        create_reproduction_attempt(
            ReproductionManifestId(id(5)),
            &manifest,
            41,
            missing_environment,
        )
        .unwrap()
        .result,
        ReproductionResult::Unavailable { .. }
    ));
    let mut failed = observation(&manifest, Vec::new());
    failed.execution_findings.push("generator_exit:2".into());
    assert!(matches!(
        create_reproduction_attempt(ReproductionManifestId(id(5)), &manifest, 42, failed)
            .unwrap()
            .result,
        ReproductionResult::ExecutionFailed { .. }
    ));
    let mut external = manifest.clone();
    external.producer = producer(ProducerClass::External);
    external.manifest_digest = reproduction_manifest_digest(&external).unwrap();
    assert!(
        create_reproduction_attempt(
            ReproductionManifestId(id(5)),
            &external,
            43,
            observation(&external, Vec::new()),
        )
        .unwrap_err()
        .to_string()
        .contains("external_generator_refused")
    );
}

#[test]
fn regeneration_creates_successors_and_reuses_only_current_evidence() {
    let context = EvidenceInputContextId(id(10));
    let predecessor = AuthorityRef::DocumentIssue(DocumentIssueId(id(11)));
    let successor = AuthorityRef::DocumentIssue(DocumentIssueId(id(12)));
    let plan = RegenerationPlanData {
        target_configuration: ConfigurationTarget::Working {
            model_revision: "next".into(),
            accepted_transaction_tip: "tip".into(),
        },
        basis_impact_evaluation: ImpactEvaluationId(id(13)),
        steps: vec![RegenerationRequest {
            output_contract: "drawing".into(),
            predecessor_evidence: Some(predecessor),
            reason_paths: vec![vec!["design-changed".into()]],
            prerequisites: Vec::new(),
            producer_ref: AuthorityRef::BuildIdentity(BuildIdentityId(id(1))),
            producer_revision: "1".into(),
            expected_input_context: context,
        }],
        unchanged_evidence_reused: vec![AuthorityRef::DocumentIssue(DocumentIssueId(id(14)))],
        blockers: Vec::new(),
    };
    let current_contexts = BTreeSet::from([context]);
    let current_evidence = BTreeSet::from([plan.unchanged_evidence_reused[0]]);
    let results = execute_regeneration_plan(&plan, &current_contexts, &current_evidence, |step| {
        Ok(RegeneratedEvidence {
            output_contract: step.output_contract.clone(),
            predecessor_evidence: step.predecessor_evidence,
            successor_evidence: successor,
            input_context: step.expected_input_context,
            digest: digest(b"successor"),
        })
    })
    .unwrap();
    assert_eq!(results[0].successor_evidence, successor);
    assert!(
        execute_regeneration_plan(
            &plan,
            &current_contexts,
            &BTreeSet::new(),
            |_| unreachable!()
        )
        .unwrap_err()
        .to_string()
        .contains("stale_evidence_reuse_refused")
    );
}

#[test]
fn later_failure_retains_original_release_bytes_and_authenticity_is_separate() {
    let release_bytes = b"issued";
    let manifest = manifest(vec![specified("release", "zip", release_bytes)]);
    let release_digest = manifest.specified_outputs[0].digest.clone();
    let success = create_reproduction_attempt(
        ReproductionManifestId(id(5)),
        &manifest,
        50,
        observation(&manifest, vec![produced("release", "zip", release_bytes)]),
    )
    .unwrap();
    let mut failed_observation = observation(&manifest, Vec::new());
    failed_observation.execution_findings = vec!["tool_crash".into()];
    failed_observation.authenticity_evidence = vec![AuthorityRef::ApprovalAttestation(
        ApprovalAttestationId(id(20)),
    )];
    let failed = create_reproduction_attempt(
        ReproductionManifestId(id(5)),
        &manifest,
        51,
        failed_observation,
    )
    .unwrap();
    assert_eq!(success.result, ReproductionResult::ByteIdentical);
    assert!(matches!(
        failed.result,
        ReproductionResult::ExecutionFailed { .. }
    ));
    assert_eq!(manifest.specified_outputs[0].digest, release_digest);
    assert_eq!(failed.authenticity_evidence.len(), 1);
    assert_ne!(success.attempt_digest, failed.attempt_digest);
}
