use super::*;

pub(super) fn append_release_actor(snapshot: &mut AuthoritySnapshot) {
    append_actor_assignment(snapshot, 90, 91);
}

fn append_actor_assignment(snapshot: &mut AuthoritySnapshot, actor: u128, assignment: u128) {
    let project_id = snapshot.project_id;
    transaction::append(
        snapshot,
        AuthorityRecord::ActorIdentity(body(
            ActorIdentityId(id(actor)),
            project_id,
            vec![],
            ActorIdentityData {
                kind: ActorKind::Person,
                stable_name: format!("release-authority-{actor}"),
            },
        )),
    )
    .unwrap();
    transaction::append(
        snapshot,
        AuthorityRecord::RoleAssignment(body(
            RoleAssignmentId(id(assignment)),
            project_id,
            vec![AuthorityRef::ActorIdentity(ActorIdentityId(id(actor)))],
            RoleAssignmentData {
                actor_id: ActorIdentityId(id(actor)),
                capabilities: BTreeSet::from([Capability::ReleaseAuthority]),
                scope: RoleScope::Project(project_id),
                effective: EffectiveInterval {
                    from_inclusive: 0,
                    until_exclusive: Some(101),
                },
                source: "release test fixture".into(),
                rationale: "bounded release authority".into(),
                supersedes: None,
                revokes: None,
            },
        )),
    )
    .unwrap();
}

#[derive(Clone, Copy, Debug)]
enum InvalidSelected {
    Missing,
    Unrelated,
    Digest,
    Revoked,
    Superseded,
    WrongIntent,
    Inactive,
    Policy,
    Version,
    Method,
    Actor,
    Assignment,
    Scope,
    OtherProject,
    Capability,
    NotYetEffective,
    Expired,
    NotEffectiveAtAssertion,
    FutureAssertion,
    RevokedAssignment,
    SupersededAssignment,
}

const CASES: &[InvalidSelected] = &[
    InvalidSelected::Missing,
    InvalidSelected::Unrelated,
    InvalidSelected::Digest,
    InvalidSelected::Revoked,
    InvalidSelected::Superseded,
    InvalidSelected::WrongIntent,
    InvalidSelected::Inactive,
    InvalidSelected::Policy,
    InvalidSelected::Version,
    InvalidSelected::Method,
    InvalidSelected::Actor,
    InvalidSelected::Assignment,
    InvalidSelected::Scope,
    InvalidSelected::OtherProject,
    InvalidSelected::Capability,
    InvalidSelected::NotYetEffective,
    InvalidSelected::Expired,
    InvalidSelected::NotEffectiveAtAssertion,
    InvalidSelected::FutureAssertion,
    InvalidSelected::RevokedAssignment,
    InvalidSelected::SupersededAssignment,
];

// Events must hash the exact record bodies retained by final canonicalization.
fn fixture_snapshot(mut state: AuthoritySnapshot) -> AuthoritySnapshot {
    for record in &mut state.records {
        record.references_mut().sort();
    }
    snapshot(state.project_id, state.records).canonicalized()
}

fn fixture(case: Option<InvalidSelected>) -> (AuthoritySnapshot, ReleaseConfigurationRequest) {
    let (mut state, mut request) =
        candidate_fixture(base_snapshot(id(180)), vec![allocation(10, 40, "A", true)]);
    append_actor_assignment(&mut state, 92, 93);
    // This additional assignment is valid even when B's named assignment is not.
    let AuthorityRecord::RoleAssignment(mut rescue) = state
        .records
        .iter()
        .find(|record| {
            record.record_ref() == AuthorityRef::RoleAssignment(RoleAssignmentId(id(93)))
        })
        .unwrap()
        .clone()
    else {
        panic!("assignment")
    };
    rescue.id = RoleAssignmentId(id(94));
    transaction::append(&mut state, AuthorityRecord::RoleAssignment(rescue)).unwrap();
    let AuthorityRecord::ApprovalAttestation(mut selected) = state
        .records
        .iter()
        .find(|record| {
            record.record_ref() == AuthorityRef::ApprovalAttestation(ApprovalAttestationId(id(31)))
        })
        .unwrap()
        .clone()
    else {
        panic!("approval A")
    };
    selected.id = ApprovalAttestationId(id(181));
    selected.semantics.actor_id = ActorIdentityId(id(92));
    selected.semantics.role_assignment_id = RoleAssignmentId(id(93));
    selected.references.extend([
        AuthorityRef::ActorIdentity(ActorIdentityId(id(92))),
        AuthorityRef::RoleAssignment(RoleAssignmentId(id(93))),
    ]);
    if matches!(case, Some(InvalidSelected::Policy)) {
        let AuthorityRecord::ProjectRevisionPolicy(mut other) = state
            .records
            .iter()
            .find(|record| {
                record.record_ref()
                    == AuthorityRef::ProjectRevisionPolicy(ProjectRevisionPolicyId(id(13)))
            })
            .unwrap()
            .clone()
        else {
            panic!("policy")
        };
        other.id = ProjectRevisionPolicyId(id(999));
        state
            .records
            .push(AuthorityRecord::ProjectRevisionPolicy(other));
    }
    for record in &mut state.records {
        if let AuthorityRecord::ProjectRevisionPolicy(policy) = record {
            if matches!(case, Some(InvalidSelected::Policy))
                && policy.id == ProjectRevisionPolicyId(id(13))
            {
                policy.semantics.supersedes = Some(ProjectRevisionPolicyId(id(999)));
            }
            policy.semantics.required_method_classes.by_intent.insert(
                ApprovalIntent::Release,
                BTreeSet::from(["test-only".into()]),
            );
        }
    }
    if let Some(case) = case {
        match case {
            InvalidSelected::Unrelated => {
                selected.semantics.target = AuthorityRef::BuildIdentity(BuildIdentityId(id(14)))
            }
            InvalidSelected::Digest => selected.semantics.target_digest = digest('x'),
            InvalidSelected::WrongIntent => selected.semantics.intent = ApprovalIntent::Approve,
            InvalidSelected::Policy => {
                selected.semantics.policy_id = ProjectRevisionPolicyId(id(999))
            }
            InvalidSelected::Version => selected.semantics.policy_version = 2,
            InvalidSelected::Method => selected.semantics.method_class = "unapproved-method".into(),
            InvalidSelected::Actor => selected.semantics.actor_id = ActorIdentityId(id(90)),
            InvalidSelected::Assignment => {
                selected.semantics.role_assignment_id = RoleAssignmentId(id(91))
            }
            InvalidSelected::FutureAssertion => selected.semantics.asserted_at = 101,
            InvalidSelected::Inactive => {
                selected.semantics.disposition = AttestationDisposition::Revoked;
                selected.semantics.disposes = Some(ApprovalAttestationId(id(182)));
            }
            _ => {}
        }
        for record in &mut state.records {
            if let AuthorityRecord::RoleAssignment(assignment) = record
                && assignment.id == RoleAssignmentId(id(93))
            {
                match case {
                    InvalidSelected::Scope => {
                        assignment.semantics.scope = RoleScope::Authority(
                            AuthorityRef::BuildIdentity(BuildIdentityId(id(14))),
                        )
                    }
                    InvalidSelected::OtherProject => {
                        assignment.semantics.scope = RoleScope::Project(id(999))
                    }
                    InvalidSelected::Capability => {
                        assignment.semantics.capabilities = BTreeSet::from([Capability::Reviewer])
                    }
                    InvalidSelected::NotYetEffective => {
                        assignment.semantics.effective.from_inclusive = 101
                    }
                    InvalidSelected::Expired => {
                        assignment.semantics.effective.until_exclusive = Some(100)
                    }
                    InvalidSelected::NotEffectiveAtAssertion => {
                        assignment.semantics.effective.from_inclusive = 100
                    }
                    _ => {}
                }
                if matches!(case, InvalidSelected::NotYetEffective) {
                    assignment.semantics.effective.until_exclusive = Some(102);
                }
            }
        }
    }
    request.release_authority_attestation = selected.id;
    if matches!(
        case,
        Some(InvalidSelected::Revoked | InvalidSelected::Superseded | InvalidSelected::Inactive)
    ) {
        let mut disposition = selected.clone();
        disposition.id = ApprovalAttestationId(id(182));
        disposition.semantics.disposes = Some(selected.id);
        disposition.semantics.disposition = if matches!(case, Some(InvalidSelected::Superseded)) {
            AttestationDisposition::Superseded
        } else {
            AttestationDisposition::Revoked
        };
        state
            .records
            .push(AuthorityRecord::ApprovalAttestation(disposition));
    }
    if matches!(
        case,
        Some(InvalidSelected::RevokedAssignment | InvalidSelected::SupersededAssignment)
    ) {
        let AuthorityRecord::RoleAssignment(mut disposition) = state
            .records
            .iter()
            .find(|record| {
                record.record_ref() == AuthorityRef::RoleAssignment(RoleAssignmentId(id(93)))
            })
            .unwrap()
            .clone()
        else {
            panic!("assignment")
        };
        disposition.id = RoleAssignmentId(id(95));
        if matches!(case, Some(InvalidSelected::RevokedAssignment)) {
            disposition.semantics.revokes = Some(RoleAssignmentId(id(93)));
        } else {
            disposition.semantics.supersedes = Some(RoleAssignmentId(id(93)));
        }
        state
            .records
            .push(AuthorityRecord::RoleAssignment(disposition));
    }
    if !matches!(case, Some(InvalidSelected::Missing)) {
        state
            .records
            .push(AuthorityRecord::ApprovalAttestation(selected));
    }
    // Rebuild structural event digests after constructing the independent fixture.
    let state = fixture_snapshot(state);
    assert!(
        state.validate().is_empty(),
        "{case:?}: {:?}",
        state.validate()
    );
    (state, request)
}

// Separate preparations mint fresh event UUIDs; compare immutable intent, then
// bind cold reopening to the exact event chain actually committed by apply.
fn assert_applied_and_cold(
    store: &RevisionAuthorityStore,
    root: &std::path::Path,
    before: &AuthoritySnapshot,
    expected: &AuthoritySnapshot,
) {
    let AuthorityResolution::Resolved { snapshot: actual } =
        store.resolve_authority(before.project_id)
    else {
        panic!("applied release authority must resolve");
    };
    assert!(actual.validate().is_empty(), "{:?}", actual.validate());
    let canonical = actual.canonicalized();
    let expected = expected.canonicalized();
    assert_eq!(actual.schema_version, expected.schema_version);
    assert_eq!(actual.project_id, expected.project_id);
    assert_eq!(canonical.records, expected.records);
    assert_eq!(canonical.opaque_records, expected.opaque_records);
    assert_eq!(canonical.opaque_events, expected.opaque_events);
    assert_eq!(actual.events.len(), expected.events.len());
    let original_count = before.events.len();
    assert!(actual.events.len() > original_count);
    assert_eq!(&actual.events[..original_count], before.events.as_slice());
    assert_eq!(&expected.events[..original_count], before.events.as_slice());
    for (actual, expected) in actual.events[original_count..]
        .iter()
        .zip(&expected.events[original_count..])
    {
        assert_eq!(
            (
                actual.schema_version,
                actual.project_id,
                actual.sequence,
                &actual.kind,
                actual.record,
                &actual.record_digest,
            ),
            (
                expected.schema_version,
                expected.project_id,
                expected.sequence,
                &expected.kind,
                expected.record,
                &expected.record_digest,
            )
        );
    }
    let committed_head = store.read_head().transpose().unwrap();
    assert!(committed_head.is_some());
    let cold = RevisionAuthorityStore::new(root);
    assert_eq!(cold.read_head().transpose().unwrap(), committed_head);
    assert_eq!(
        cold.resolve_authority(before.project_id),
        AuthorityResolution::Resolved { snapshot: actual }
    );
}

#[test]
fn selected_b_authorizes_exact_candidate_without_borrowing_approval_a() {
    let (state, request) = fixture(None);
    let result = prepare_release_configuration(&state, &request, None).unwrap();
    let selected = result
        .snapshot
        .records
        .iter()
        .find_map(|record| match record {
            AuthorityRecord::Release(body) => Some(body.semantics.release_authority_attestation),
            _ => None,
        })
        .unwrap();
    assert_eq!(selected, request.release_authority_attestation);
    assert_ne!(selected, ApprovalAttestationId(id(31)));
}

#[test]
fn selected_b_does_not_need_the_separate_reviewers_capability() {
    let (mut state, request) = fixture(None);
    for record in &mut state.records {
        match record {
            AuthorityRecord::RoleAssignment(assignment)
                if assignment.id == RoleAssignmentId(id(91)) =>
            {
                assignment.semantics.capabilities = BTreeSet::from([Capability::Reviewer]);
            }
            AuthorityRecord::ProjectRevisionPolicy(policy) => {
                policy.semantics.approval_policy.requirements =
                    [Capability::Reviewer, Capability::ReleaseAuthority]
                        .into_iter()
                        .enumerate()
                        .map(|(order, required_capability)| ApprovalRequirement {
                            intent: ApprovalIntent::Release,
                            required_capability,
                            quorum: 1,
                            order: order as u32,
                            independent_from_intents: BTreeSet::new(),
                        })
                        .collect();
            }
            _ => {}
        }
    }
    let state = fixture_snapshot(state);
    assert!(state.validate().is_empty(), "{:?}", state.validate());
    let scope = RoleScope::Authority(AuthorityRef::ReleaseCandidate(request.candidate_id));
    for (actor, granted, absent) in [
        (90, Capability::Reviewer, Capability::ReleaseAuthority),
        (92, Capability::ReleaseAuthority, Capability::Reviewer),
    ] {
        assert!(matches!(
            evaluate_capability(
                &state,
                ActorIdentityId(id(actor)),
                granted,
                scope,
                request.released_at
            ),
            CapabilityEvaluation::Authorized(_)
        ));
        assert!(matches!(
            evaluate_capability(
                &state,
                ActorIdentityId(id(actor)),
                absent,
                scope,
                request.released_at
            ),
            CapabilityEvaluation::Refused(_)
        ));
    }
    // Separate ordered policy requirements may be supplied by separate actors.
    // This control checks selected authorization, without asserting signature validity.
    let expected = prepare_release_configuration(&state, &request, None)
        .unwrap()
        .snapshot;
    let (root, store) = persisted_fixture(&state);
    apply_release_configuration(&store, state.project_id, &request).unwrap();
    assert_applied_and_cold(&store, root.path(), &state, &expected);
}

#[test]
fn invalid_selected_b_refuses_while_separate_approval_a_remains_valid() {
    for case in CASES {
        let (state, request) = fixture(Some(*case));
        let before = state.canonical_bytes().unwrap();
        let mut a_request = request.clone();
        a_request.release_authority_attestation = ApprovalAttestationId(id(31));
        prepare_release_configuration(&state, &a_request, None)
            .expect("A still authorizes release");
        let error = prepare_release_configuration(&state, &request, None).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("revision_selected_release_authority_invalid"),
            "{case:?}: {error}"
        );
        assert_eq!(state.canonical_bytes().unwrap(), before, "{case:?}");
    }
}

struct TestProject(std::path::PathBuf);

impl TestProject {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn persisted_fixture(state: &AuthoritySnapshot) -> (TestProject, RevisionAuthorityStore) {
    use crate::substrate::{
        CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
    };
    let root = TestProject(std::env::temp_dir().join(format!("datum_ca18_{}", Uuid::new_v4())));
    std::fs::create_dir_all(root.path()).unwrap();
    let project_id = state.project_id;
    let files = [
        (
            "project.json",
            serde_json::json!({"schema_version":1,"uuid":project_id,"name":"CA18",
            "pools":[],"schematic":"schematic/schematic.json","board":"board/board.json","rules":"rules/rules.json"}),
        ),
        (
            "schematic/schematic.json",
            serde_json::json!({"schema_version":1,"uuid":id(190),
            "sheets":{},"definitions":{},"instances":[],"variants":{},"waivers":[]}),
        ),
        (
            "board/board.json",
            serde_json::json!({"schema_version":1,"uuid":id(191),"name":"Board",
            "packages":{},"tracks":{},"vias":{},"zones":{},"nets":{},"net_classes":{}}),
        ),
        (
            "rules/rules.json",
            serde_json::json!({"schema_version":1,"rules":[]}),
        ),
    ];
    for (path, value) in files {
        let path = root.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }
    let mut model = ProjectResolver::new(root.path()).resolve().unwrap();
    model
        .commit_journaled(
            root.path(),
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "release-test".into(),
                    source: CommitSource::Test,
                    reason: "establish accepted project transaction before selected authority test"
                        .into(),
                },
                operations: vec![Operation::SetProjectName {
                    project_id,
                    name: "CA18 persisted".into(),
                }],
            },
        )
        .unwrap();
    let store = RevisionAuthorityStore::new(root.path());
    let head = store.read_head().transpose().unwrap().unwrap();
    store
        .commit_authority_snapshot(project_id, &head.integrity_root, state)
        .unwrap();
    assert_eq!(
        store.resolve_authority(project_id),
        AuthorityResolution::Resolved {
            snapshot: state.clone()
        }
    );
    (root, store)
}

#[test]
fn apply_invalid_selected_b_preserves_persisted_head_snapshot_and_cold_state() {
    use crate::substrate::ProjectResolver;
    for case in CASES {
        let (state, request) = fixture(Some(*case));
        let (root, store) = persisted_fixture(&state);
        let head = store.read_head().transpose().unwrap();
        let authority = store.resolve_authority(state.project_id);
        let cold = ProjectResolver::new(root.path()).resolve().unwrap();
        let mut a_request = request.clone();
        a_request.release_authority_attestation = ApprovalAttestationId(id(31));
        prepare_release_configuration(&state, &a_request, None)
            .expect("valid A survives persisted B fixture");
        let error = apply_release_configuration(&store, state.project_id, &request).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("revision_selected_release_authority_invalid"),
            "{case:?}: {error}"
        );
        assert_eq!(store.read_head().transpose().unwrap(), head, "{case:?}");
        assert_eq!(
            store.resolve_authority(state.project_id),
            authority,
            "{case:?}"
        );
        let reopened = RevisionAuthorityStore::new(root.path());
        assert_eq!(reopened.read_head().transpose().unwrap(), head, "{case:?}");
        assert_eq!(
            reopened.resolve_authority(state.project_id),
            authority,
            "{case:?}"
        );
        let after = ProjectResolver::new(root.path()).resolve().unwrap();
        assert_eq!(after.model_revision, cold.model_revision, "{case:?}");
        assert_eq!(after.journal, cold.journal, "{case:?}");
        assert_eq!(after.journal_cursor, cold.journal_cursor, "{case:?}");
        assert_eq!(
            serde_json::to_value(&after).unwrap(),
            serde_json::to_value(&cold).unwrap(),
            "{case:?}"
        );
        assert!(
            matches!(
                reopened.inspect_and_recover(state.project_id, &after.journal),
                RevisionStoreState::Complete { .. }
            ),
            "{case:?}"
        );
    }
}

#[test]
fn apply_valid_selected_b_issues_and_cold_reopens_exact_authority() {
    let (state, request) = fixture(None);
    let (root, store) = persisted_fixture(&state);
    let expected = prepare_release_configuration(&state, &request, None)
        .unwrap()
        .snapshot;
    let before = store.read_head().transpose().unwrap().unwrap();
    let after = apply_release_configuration(&store, state.project_id, &request).unwrap();
    assert_ne!(before.integrity_root, after.integrity_root);
    let reopened = RevisionAuthorityStore::new(root.path());
    assert_eq!(reopened.read_head().transpose().unwrap(), Some(after));
    assert_applied_and_cold(&store, root.path(), &state, &expected);
}
