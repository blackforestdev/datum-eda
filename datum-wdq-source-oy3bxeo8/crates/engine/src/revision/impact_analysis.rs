use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    error::EngineError,
    substrate::{ComponentInstance, ObjectRevision, Operation},
};

use super::*;

pub fn capture_dependency_snapshot(mut data: DependencySnapshotData) -> DependencySnapshotData {
    data.nodes.sort_by_key(|node| node.authority_ref);
    for edge in &mut data.edges {
        edge.sensitivity.sort();
        edge.sensitivity.dedup();
    }
    data.edges
        .sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
    data.unresolved_inputs.sort();
    data.unresolved_inputs.dedup();
    data
}

pub fn evaluate_subject_impacts(
    snapshot: &DependencySnapshotData,
    deltas: &[SemanticDeltaData],
    subjects: &[AuthorityRef],
) -> Vec<SubjectImpact> {
    let mut changed: BTreeMap<AuthorityRef, BTreeSet<String>> = BTreeMap::new();
    for delta in deltas {
        changed
            .entry(delta.subject)
            .or_default()
            .extend(delta.changed_observations.iter().cloned());
    }
    let mut impacts: Vec<_> = subjects
        .iter()
        .copied()
        .map(|subject| evaluate_subject(snapshot, &changed, subject))
        .collect();
    impacts.sort_by_key(|impact| impact.subject);
    impacts
}

fn evaluate_subject(
    snapshot: &DependencySnapshotData,
    changed: &BTreeMap<AuthorityRef, BTreeSet<String>>,
    subject: AuthorityRef,
) -> SubjectImpact {
    if !snapshot.graph_complete || !snapshot.unresolved_inputs.is_empty() {
        return unknown_impact(subject, "graph_incomplete");
    }
    let mut queue: VecDeque<_> = changed
        .iter()
        .map(|(source, observations)| (*source, observations.clone(), Vec::<String>::new()))
        .collect();
    let mut visited = BTreeSet::new();
    let mut disjoint_witness = None;
    while let Some((node, observations, path)) = queue.pop_front() {
        if !visited.insert(node) {
            continue;
        }
        for edge in snapshot.edges.iter().filter(|edge| edge.source == node) {
            let mut next_path = path.clone();
            next_path.push(edge.edge_id.clone());
            if edge.evaluator_id.is_empty() || edge.evaluator_revision.is_empty() {
                if edge.target == subject {
                    return unknown_impact(subject, "evaluator_missing");
                }
                continue;
            }
            let edge_sensitivity: BTreeSet<_> = edge.sensitivity.iter().cloned().collect();
            let intersection: BTreeSet<_> = observations
                .intersection(&edge_sensitivity)
                .cloned()
                .collect();
            if intersection.is_empty() {
                if edge.target == subject {
                    disjoint_witness = Some(next_path);
                }
                continue;
            }
            if edge.target == subject {
                return SubjectImpact {
                    subject,
                    result: ImpactResult::Affected,
                    changed_observations: intersection.into_iter().collect(),
                    witness_paths: vec![next_path],
                    reason_code: "sensitivity_intersection".to_string(),
                    required_actions: vec!["review_affected_subject".to_string()],
                    reviewer_disposition: None,
                };
            }
            queue.push_back((edge.target, intersection, next_path));
        }
    }
    SubjectImpact {
        subject,
        result: ImpactResult::Unaffected,
        changed_observations: Vec::new(),
        witness_paths: vec![
            disjoint_witness.unwrap_or_else(|| vec!["complete_graph:no_path".to_string()]),
        ],
        reason_code: "sensitivity_disjoint".to_string(),
        required_actions: Vec::new(),
        reviewer_disposition: None,
    }
}

fn unknown_impact(subject: AuthorityRef, reason: &str) -> SubjectImpact {
    SubjectImpact {
        subject,
        result: ImpactResult::ImpactUnknown,
        changed_observations: Vec::new(),
        witness_paths: Vec::new(),
        reason_code: reason.to_string(),
        required_actions: vec!["resolve_impact_unknown".to_string()],
        reviewer_disposition: None,
    }
}

pub fn evaluate_evidence_freshness(
    input_context: EvidenceInputContextId,
    target_configuration: ConfigurationTarget,
    recorded: &[DigestQualifiedInput],
    current: &BTreeMap<AuthorityRef, Option<AlgorithmQualifiedDigest>>,
    unsupported: &BTreeSet<AuthorityRef>,
) -> EvidenceFreshnessData {
    let mut differing_inputs = Vec::new();
    let mut unresolved_inputs = Vec::new();
    let mut unsupported_inputs = Vec::new();
    for input in recorded {
        if unsupported.contains(&input.input_ref) {
            unsupported_inputs.push(input.input_ref);
        } else {
            match current.get(&input.input_ref) {
                Some(Some(digest)) if digest == &input.digest => {}
                Some(Some(_)) => differing_inputs.push(input.input_ref),
                _ => unresolved_inputs.push(input.input_ref),
            }
        }
    }
    differing_inputs.sort();
    differing_inputs.dedup();
    unresolved_inputs.sort();
    unresolved_inputs.dedup();
    unsupported_inputs.sort();
    unsupported_inputs.dedup();
    let is_current = differing_inputs.is_empty()
        && unresolved_inputs.is_empty()
        && unsupported_inputs.is_empty();
    let mut reasons = Vec::new();
    if !differing_inputs.is_empty() {
        reasons.push("declared_input_differs".to_string());
    }
    if !unresolved_inputs.is_empty() {
        reasons.push("required_input_or_producer_unresolved".to_string());
    }
    if !unsupported_inputs.is_empty() {
        reasons.push("input_evaluator_unsupported".to_string());
    }
    EvidenceFreshnessData {
        input_context,
        target_configuration,
        current: is_current,
        differing_inputs,
        unresolved_inputs,
        unsupported_inputs,
        reasons,
    }
}

pub fn prepare_library_uptake_operation(
    candidate: &LibraryUptakeCandidateData,
    component: &ComponentInstance,
) -> Result<Operation, EngineError> {
    if component.id != candidate.component_instance_id {
        return Err(EngineError::Validation(
            "library uptake candidate names another component instance".to_string(),
        ));
    }
    if !candidate.pinned_source_resolves {
        return Err(EngineError::Validation(
            "pinned library source is missing; uptake cannot float to latest".to_string(),
        ));
    }
    if !candidate.proposed_source_resolves {
        return Err(EngineError::Validation(
            "proposed library source does not resolve".to_string(),
        ));
    }
    let mut updated = component.clone();
    let binding = updated
        .library_bindings
        .get_mut(&candidate.binding_id)
        .ok_or_else(|| EngineError::Validation("library binding does not resolve".to_string()))?;
    if binding.target_object_id != candidate.pinned_library_ref.object_id
        || binding.pinned_object_revision != candidate.pinned_library_ref.object_revision
    {
        return Err(EngineError::Validation(
            "library uptake candidate is stale against the exact pinned binding".to_string(),
        ));
    }
    binding.target_object_id = candidate.proposed_library_ref.object_id;
    binding.pinned_object_revision = candidate.proposed_library_ref.object_revision;
    updated.object_revision = ObjectRevision(updated.object_revision.0 + 1);
    Ok(Operation::SetComponentInstance {
        component_instance_id: candidate.component_instance_id,
        previous_component_instance: serde_json::to_value(component)?,
        component_instance: serde_json::to_value(updated)?,
    })
}

pub fn compare_baselines(
    left_id: ConfigurationBaselineId,
    left: &ConfigurationBaselineData,
    right_id: ConfigurationBaselineId,
    right: &ConfigurationBaselineData,
    semantic_deltas: &BTreeMap<AuthorityRef, SemanticDeltaId>,
) -> BaselineComparisonData {
    let left_members: BTreeMap<_, _> = left
        .members
        .iter()
        .map(|member| (member.stable_authority_ref, member))
        .collect();
    let right_members: BTreeMap<_, _> = right
        .members
        .iter()
        .map(|member| (member.stable_authority_ref, member))
        .collect();
    let keys: BTreeSet<_> = left_members
        .keys()
        .chain(right_members.keys())
        .copied()
        .collect();
    let mut unresolved = Vec::new();
    let aligned_members =
        keys.into_iter()
            .map(|key| {
                let (result, technical_revision_delta) =
                    match (left_members.get(&key), right_members.get(&key)) {
                        (None, Some(_)) => (BaselineComparisonResult::Added, None),
                        (Some(_), None) => (BaselineComparisonResult::Removed, None),
                        (Some(left), Some(right)) if left.target_ref != right.target_ref => {
                            (BaselineComparisonResult::Retargeted, None)
                        }
                        (Some(left), Some(right))
                            if left.exact_technical_revision != right.exact_technical_revision
                                || left.semantic_digest != right.semantic_digest
                                || left.display_name != right.display_name =>
                        {
                            (
                                BaselineComparisonResult::Modified,
                                (left.exact_technical_revision != right.exact_technical_revision)
                                    .then(|| {
                                        (
                                            left.exact_technical_revision.clone(),
                                            right.exact_technical_revision.clone(),
                                        )
                                    }),
                            )
                        }
                        (Some(_), Some(_)) => (BaselineComparisonResult::Unchanged, None),
                        (None, None) => unreachable!(),
                    };
                let related_changes = right.governing_changes.clone();
                if result != BaselineComparisonResult::Unchanged
                    && related_changes.is_empty()
                    && right.departures.is_empty()
                {
                    unresolved.push(key);
                }
                MemberComparison {
                    stable_authority_ref: key,
                    result,
                    technical_revision_delta,
                    semantic_delta: semantic_deltas.get(&key).copied(),
                    related_changes,
                    impact_dispositions: Vec::new(),
                }
            })
            .collect();
    BaselineComparisonData {
        left_baseline: left_id,
        right_baseline: right_id,
        aligned_members,
        dependency_delta: Vec::new(),
        evidence_delta: Vec::new(),
        governing_change_coverage: right.governing_changes.clone(),
        unresolved_differences: unresolved,
    }
}

pub fn require_accounted_baseline_difference(
    comparison: &BaselineComparisonData,
) -> Result<(), EngineError> {
    if comparison.unresolved_differences.is_empty() {
        Ok(())
    } else {
        Err(EngineError::Validation(
            "unaccounted_controlled_difference: release requires a governing Change, departure, or accepted administrative disposition"
                .to_string(),
        ))
    }
}

pub fn create_regeneration_plan(
    target_configuration: ConfigurationTarget,
    basis_impact_evaluation: ImpactEvaluationId,
    requests: Vec<RegenerationRequest>,
    mut unchanged_evidence_reused: Vec<AuthorityRef>,
) -> RegenerationPlanData {
    let mut by_contract = BTreeMap::new();
    let mut blockers = Vec::new();
    for request in requests {
        let contract = request.output_contract.clone();
        if by_contract.insert(contract.clone(), request).is_some() {
            blockers.push(format!("duplicate_output_contract:{contract}"));
        }
    }
    if !blockers.is_empty() {
        blockers.sort();
        blockers.dedup();
        return RegenerationPlanData {
            target_configuration,
            basis_impact_evaluation,
            steps: Vec::new(),
            unchanged_evidence_reused: Vec::new(),
            blockers,
        };
    }
    let mut ready: BTreeSet<_> = by_contract
        .iter()
        .filter(|(_, request)| request.prerequisites.is_empty())
        .map(|(name, _)| name.clone())
        .collect();
    let mut emitted = BTreeSet::new();
    let mut steps = Vec::new();
    while let Some(name) = ready.pop_first() {
        if !emitted.insert(name.clone()) {
            continue;
        }
        if let Some(mut step) = by_contract.remove(&name) {
            step.prerequisites.sort();
            steps.push(step);
        }
        for (candidate, request) in &by_contract {
            if request
                .prerequisites
                .iter()
                .all(|prerequisite| emitted.contains(prerequisite))
            {
                ready.insert(candidate.clone());
            }
        }
    }
    blockers = by_contract
        .keys()
        .map(|contract| format!("dependency_cycle_or_missing:{contract}"))
        .collect();
    unchanged_evidence_reused.sort();
    unchanged_evidence_reused.dedup();
    RegenerationPlanData {
        target_configuration,
        basis_impact_evaluation,
        steps,
        unchanged_evidence_reused,
        blockers,
    }
}

pub(crate) fn validate_rev_i05_record(record: &AuthorityRecord) -> Vec<AuthorityDiagnostic> {
    match record {
        AuthorityRecord::ConfigurationBaseline(body) => validate_baseline(&body.semantics),
        AuthorityRecord::DependencySnapshot(body) => validate_dependency_snapshot(&body.semantics),
        AuthorityRecord::SemanticDelta(body) => validate_semantic_delta(&body.semantics),
        AuthorityRecord::ImpactEvaluation(body) => validate_impact_evaluation(&body.semantics),
        AuthorityRecord::EvidenceInputContext(body) => validate_evidence_context(&body.semantics),
        AuthorityRecord::EvidenceFreshness(body) => validate_freshness(&body.semantics),
        AuthorityRecord::LibraryUptakeCandidate(body) => validate_library_uptake(&body.semantics),
        AuthorityRecord::BaselineComparison(body) => validate_baseline_comparison(&body.semantics),
        AuthorityRecord::RegenerationPlan(body) => validate_regeneration_plan(&body.semantics),
        _ => Vec::new(),
    }
}

pub(crate) fn validate_dependency_snapshot(
    data: &DependencySnapshotData,
) -> Vec<AuthorityDiagnostic> {
    let mut diagnostics = Vec::new();
    if data.evaluator_registry_revision.is_empty()
        || !is_strictly_sorted_by(&data.nodes, |node| node.authority_ref)
        || !is_strictly_sorted_by(&data.edges, |edge| edge.edge_id.clone())
    {
        diagnostics.push(diag(
            "revision_dependency_snapshot_not_canonical",
            "dependency nodes, edges, and evaluator revision must be deterministic",
        ));
    }
    let nodes: BTreeSet<_> = data.nodes.iter().map(|node| node.authority_ref).collect();
    if data.edges.iter().any(|edge| {
        edge.edge_id.is_empty()
            || edge.origin.is_empty()
            || edge.evaluator_id.is_empty()
            || edge.evaluator_revision.is_empty()
            || !nodes.contains(&edge.source)
            || !nodes.contains(&edge.target)
            || !is_sorted_unique(&edge.sensitivity)
    }) {
        diagnostics.push(diag(
            "revision_dependency_edge_incomplete",
            "every edge must resolve exact nodes, evaluator, origin, and sorted sensitivity",
        ));
    }
    let represented: BTreeSet<_> = data.nodes.iter().map(|node| node.node_kind).collect();
    let required: BTreeSet<_> = DependencyNodeKind::ALL.iter().copied().collect();
    if data.graph_complete && (represented != required || !data.unresolved_inputs.is_empty()) {
        diagnostics.push(diag(
            "revision_dependency_false_completeness",
            "complete dependency graphs cover every required domain and no unresolved input",
        ));
    }
    diagnostics
}

fn validate_semantic_delta(data: &SemanticDeltaData) -> Vec<AuthorityDiagnostic> {
    if data.evaluator_id.is_empty()
        || data.evaluator_revision.is_empty()
        || !is_sorted_unique(&data.changed_observations)
        || !is_sorted_unique(&data.administrative_observations)
    {
        vec![diag(
            "revision_semantic_delta_not_canonical",
            "semantic and administrative observations require typed evaluator identity and sorted uniqueness",
        )]
    } else {
        Vec::new()
    }
}

fn validate_impact_evaluation(data: &ImpactEvaluationData) -> Vec<AuthorityDiagnostic> {
    let mut diagnostics = Vec::new();
    if !is_strictly_sorted_by(&data.subjects, |subject| subject.subject) {
        diagnostics.push(diag(
            "revision_impact_subjects_not_canonical",
            "impact subjects must be unique and stable-ID sorted",
        ));
    }
    for subject in &data.subjects {
        if matches!(
            subject.result,
            ImpactResult::Affected | ImpactResult::Unaffected
        ) && subject.witness_paths.is_empty()
        {
            diagnostics.push(diag(
                "revision_impact_witness_missing",
                "Affected and Unaffected require machine-readable witness proof",
            ));
        }
        if !data.graph_complete && subject.result != ImpactResult::ImpactUnknown {
            diagnostics.push(diag(
                "revision_impact_unknown_required",
                "an incomplete graph must preserve ImpactUnknown",
            ));
        }
    }
    diagnostics
}

fn validate_evidence_context(data: &EvidenceInputContextData) -> Vec<AuthorityDiagnostic> {
    if data.producer_revision.is_empty()
        || !is_strictly_sorted_by(&data.inputs, |input| input.input_ref)
        || !is_sorted_unique(&data.policy_refs)
    {
        vec![diag(
            "revision_evidence_context_not_canonical",
            "evidence inputs and policy references must be exact, sorted, and producer-qualified",
        )]
    } else {
        Vec::new()
    }
}

fn validate_freshness(data: &EvidenceFreshnessData) -> Vec<AuthorityDiagnostic> {
    let no_difference = data.differing_inputs.is_empty()
        && data.unresolved_inputs.is_empty()
        && data.unsupported_inputs.is_empty();
    if data.current != no_difference {
        vec![diag(
            "revision_evidence_freshness_contradiction",
            "current evidence cannot carry differing, unresolved, or unsupported inputs",
        )]
    } else {
        Vec::new()
    }
}

fn validate_library_uptake(data: &LibraryUptakeCandidateData) -> Vec<AuthorityDiagnostic> {
    if data.pinned_library_ref == data.proposed_library_ref
        || !is_sorted_unique(&data.required_checks)
        || !is_sorted_unique(&data.required_regeneration)
    {
        vec![diag(
            "revision_library_uptake_no_change",
            "library uptake must preview a distinct exact library revision",
        )]
    } else {
        Vec::new()
    }
}

fn validate_baseline(data: &ConfigurationBaselineData) -> Vec<AuthorityDiagnostic> {
    if data.baseline_type.is_empty()
        || data.source_model_revision.is_empty()
        || data.accepted_transaction_tip.is_empty()
        || data.members.is_empty()
        || !is_strictly_sorted_by(&data.members, |member| member.stable_authority_ref)
        || !is_sorted_unique(&data.scope)
        || !is_sorted_unique(&data.governing_changes)
        || !is_sorted_unique(&data.departures)
        || !is_sorted_unique(&data.profile_refs)
        || !is_sorted_unique(&data.establishment_attestations)
        || !is_sorted_unique(&data.predecessor_baselines)
        || data.members.iter().any(|member| {
            member.exact_technical_revision.is_empty()
                || member.role.is_empty()
                || member.inclusion_reason.is_empty()
        })
    {
        vec![diag(
            "revision_baseline_floating_or_not_canonical",
            "baselines require exact sorted members, revisions, journal tip, roles, and inclusion reasons",
        )]
    } else {
        Vec::new()
    }
}

fn validate_baseline_comparison(data: &BaselineComparisonData) -> Vec<AuthorityDiagnostic> {
    if !is_strictly_sorted_by(&data.aligned_members, |member| member.stable_authority_ref) {
        vec![diag(
            "revision_baseline_comparison_not_canonical",
            "baseline comparison must align unique stable identities",
        )]
    } else {
        Vec::new()
    }
}

pub(crate) fn validate_regeneration_plan(data: &RegenerationPlanData) -> Vec<AuthorityDiagnostic> {
    let mut seen = BTreeSet::new();
    for step in &data.steps {
        let prerequisites_ready = step
            .prerequisites
            .iter()
            .all(|prerequisite| seen.contains(prerequisite));
        if step.output_contract.is_empty()
            || step.producer_revision.is_empty()
            || !prerequisites_ready
            || !seen.insert(step.output_contract.clone())
        {
            return vec![diag(
                "revision_regeneration_plan_not_topological",
                "regeneration steps must be uniquely and topologically ordered",
            )];
        }
    }
    Vec::new()
}

fn is_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn is_strictly_sorted_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}

pub(crate) fn append_rev_i05_records(
    snapshot: &mut super::AuthoritySnapshot,
    records: Vec<AuthorityRecord>,
) -> Result<(), EngineError> {
    if records.is_empty() {
        return Err(EngineError::Validation(
            "REV-I05 record batch cannot be empty".to_string(),
        ));
    }
    for record in records {
        if !REV_I05_RECORD_FAMILIES.contains(&record.kind()) {
            return Err(EngineError::Validation(format!(
                "record family {} is outside the closed REV-I05 inventory",
                record.kind().wire_tag()
            )));
        }
        super::transaction::append(snapshot, record)?;
    }
    let diagnostics = snapshot.validate();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(EngineError::Validation(format!(
            "REV-I05 authority record refused: {}",
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )))
    }
}

pub fn apply_rev_i05_records(
    store: &RevisionAuthorityStore,
    project_id: uuid::Uuid,
    records: Vec<AuthorityRecord>,
) -> Result<super::IntegrityHead, EngineError> {
    let head = store.read_head().transpose()?.ok_or_else(|| {
        EngineError::Validation(
            "REV-I05 authority mutation requires an accepted Project transaction".to_string(),
        )
    })?;
    let mut snapshot = match store.resolve_authority(project_id) {
        AuthorityResolution::Unconfigured => AuthoritySnapshot {
            schema_version: super::AUTHORITY_SCHEMA_VERSION,
            project_id,
            records: Vec::new(),
            events: Vec::new(),
            opaque_records: Vec::new(),
            opaque_events: Vec::new(),
        },
        AuthorityResolution::Resolved { snapshot } => snapshot,
        AuthorityResolution::ReadOnlyDiagnostic { .. } => {
            return Err(EngineError::Validation(
                "revision authority is read-only; preserved unknown data cannot be rewritten"
                    .to_string(),
            ));
        }
    };
    append_rev_i05_records(&mut snapshot, records)?;
    store.commit_authority_snapshot(project_id, &head.integrity_root, &snapshot)
}

pub fn rev_i05_record_id(record: &AuthorityRecord) -> Option<String> {
    match record {
        AuthorityRecord::ConfigurationBaseline(body) => Some(body.id.0.to_string()),
        AuthorityRecord::DependencySnapshot(body) => Some(body.id.0.to_string()),
        AuthorityRecord::SemanticDelta(body) => Some(body.id.0.to_string()),
        AuthorityRecord::ImpactEvaluation(body) => Some(body.id.0.to_string()),
        AuthorityRecord::EvidenceInputContext(body) => Some(body.id.0.to_string()),
        AuthorityRecord::EvidenceFreshness(body) => Some(body.id.0.to_string()),
        AuthorityRecord::LibraryUptakeCandidate(body) => Some(body.id.0.to_string()),
        AuthorityRecord::BaselineComparison(body) => Some(body.id.0.to_string()),
        AuthorityRecord::RegenerationPlan(body) => Some(body.id.0.to_string()),
        _ => None,
    }
}
