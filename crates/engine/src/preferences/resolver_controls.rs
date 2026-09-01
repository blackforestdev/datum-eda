use super::*;

pub(super) fn descriptor_provenance(
    descriptor: &crate::preferences::PreferenceDescriptor,
    reason: &str,
) -> FactProvenance {
    FactProvenance {
        origin: descriptor.key.as_str().to_owned(),
        provider: Some(descriptor.owner.clone()),
        package: None,
        generation: Some(descriptor.schema_version.to_string()),
        actor: "descriptor".to_owned(),
        role: Some("subsystem owner".to_owned()),
        observed_at: "registry snapshot".to_owned(),
        effective_from: None,
        effective_until: None,
        offline_valid_until: None,
        last_successful_contact: None,
        reason: reason.to_owned(),
    }
}

pub(super) fn constraint_matches_descriptor(
    constraint: &ValueConstraint,
    descriptor: &crate::preferences::PreferenceDescriptor,
) -> bool {
    match constraint {
        ValueConstraint::IntegerRange { min, max } => match &descriptor.value_schema {
            crate::preferences::ValueSchema::IntegerRange {
                min: descriptor_min,
                max: descriptor_max,
                ..
            } => min <= max && min >= descriptor_min && max <= descriptor_max,
            _ => false,
        },
        ValueConstraint::AllowedValues(values) | ValueConstraint::DeniedValues(values) => {
            !values.is_empty()
                && values.iter().all(|value| descriptor.validates(value))
                && values
                    .iter()
                    .enumerate()
                    .all(|(index, value)| !values[..index].contains(value))
        }
    }
}

pub(super) fn source_rank(source: ResolutionSource) -> Option<u8> {
    match source {
        ResolutionSource::Session => Some(4),
        ResolutionSource::User => Some(3),
        ResolutionSource::Organization => Some(2),
        ResolutionSource::Installation => Some(1),
        ResolutionSource::DescriptorDefault => Some(0),
        ResolutionSource::Context | ResolutionSource::ProjectPolicy => None,
    }
}

pub(super) fn control_conflict(
    controls: &[ActiveControl],
    descriptor: &crate::preferences::PreferenceDescriptor,
) -> Option<(Vec<String>, String)> {
    let locks: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Lock(scopes) => Some((control.id.clone(), scopes)),
            _ => None,
        })
        .collect();
    if let Some((_, first)) = locks.first()
        && locks.iter().any(|(_, scopes)| *scopes != *first)
    {
        let mut ids: Vec<_> = locks.into_iter().map(|(id, _)| id).collect();
        ids.sort();
        return Some((
            ids,
            "equal-authority Locks specify incompatible scope sets".to_owned(),
        ));
    }
    let pins: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Pin(value) => Some((control.id.clone(), value)),
            _ => None,
        })
        .collect();
    let pins_conflict = pins
        .first()
        .is_some_and(|(_, first)| pins.iter().any(|(_, value)| *value != *first));
    if pins_conflict {
        let mut ids: Vec<_> = pins.into_iter().map(|(id, _)| id).collect();
        ids.sort();
        return Some((
            ids,
            "equal-authority Pins specify incompatible values".to_owned(),
        ));
    }
    let ranges: Vec<_> = controls
        .iter()
        .filter_map(|control| match control.directive {
            OrganizationDirective::Constrain(ValueConstraint::IntegerRange { min, max }) => {
                Some((control.id.clone(), min, max))
            }
            _ => None,
        })
        .collect();
    if !ranges.is_empty() {
        let lower = ranges.iter().map(|(_, min, _)| *min).max().unwrap();
        let upper = ranges.iter().map(|(_, _, max)| *max).min().unwrap();
        if lower > upper {
            let mut ids: Vec<_> = ranges.into_iter().map(|(id, _, _)| id).collect();
            ids.sort();
            return Some((
                ids,
                "equal-authority constraints have an empty intersection".to_owned(),
            ));
        }
    }
    let constraints: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Constrain(constraint) => Some((control.id.clone(), constraint)),
            _ => None,
        })
        .collect();
    let candidate_domain = constraints
        .iter()
        .find_map(|(_, constraint)| match constraint {
            ValueConstraint::AllowedValues(values) => Some(values.clone()),
            ValueConstraint::IntegerRange { .. } | ValueConstraint::DeniedValues(_) => None,
        })
        .or_else(|| descriptor.value_schema.finite_domain());
    if let Some(domain) = candidate_domain
        && !domain.iter().any(|value| {
            constraints
                .iter()
                .all(|(_, constraint)| constraint.allows(value))
        })
    {
        let mut ids: Vec<_> = constraints.into_iter().map(|(id, _)| id).collect();
        ids.sort();
        return Some((
            ids,
            "equal-authority constraints have an empty descriptor-domain intersection".to_owned(),
        ));
    }
    None
}

pub(super) fn mark_conflicting(considered: &mut [ConsideredFact], ids: &[String]) {
    for fact in considered {
        if ids.contains(&fact.id) {
            fact.disposition = ContributionDisposition::Conflicting;
        }
    }
}

pub(super) fn resolve_with_controls(
    considered: &mut [ConsideredFact],
    candidates: &mut Vec<Candidate>,
    controls: &[ActiveControl],
    actions: &mut BTreeSet<AvailableAction>,
) -> ResolutionOutcome {
    for control in controls {
        if matches!(
            control.directive,
            OrganizationDirective::Constrain(_) | OrganizationDirective::Lock(_)
        ) {
            considered[control.considered_index].disposition = ContributionDisposition::Effective;
            considered[control.considered_index].reason =
                "active control applies before ordinary value ranking".to_owned();
        }
    }
    let constraints: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Constrain(constraint) => Some(constraint),
            _ => None,
        })
        .collect();
    for candidate in candidates.iter() {
        if constraints
            .iter()
            .any(|constraint| !constraint.allows(&candidate.value))
        {
            considered[candidate.considered_index].disposition = ContributionDisposition::Inert;
            considered[candidate.considered_index].reason =
                "retained value is outside an active organization constraint".to_owned();
        }
    }
    candidates.retain(|candidate| {
        constraints
            .iter()
            .all(|constraint| constraint.allows(&candidate.value))
    });

    let pins: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Pin(value) => Some((control, value)),
            _ => None,
        })
        .collect();
    if let Some((_, value)) = pins.first() {
        let pinned_value = (*value).clone();
        if constraints
            .iter()
            .any(|constraint| !constraint.allows(value))
        {
            let mut ids: Vec<_> = pins.iter().map(|(control, _)| control.id.clone()).collect();
            ids.extend(
                controls
                    .iter()
                    .filter(|control| {
                        matches!(control.directive, OrganizationDirective::Constrain(_))
                    })
                    .map(|control| control.id.clone()),
            );
            ids.sort();
            mark_conflicting(considered, &ids);
            return ResolutionOutcome::UnresolvedConflict {
                fact_ids: ids,
                reason: "Pin violates an applicable constraint".to_owned(),
            };
        }
        for candidate in candidates.iter() {
            considered[candidate.considered_index].disposition = ContributionDisposition::Retained;
            considered[candidate.considered_index].reason =
                "retained value resumes when the active Pin lifts".to_owned();
        }
        let mut ids = Vec::new();
        for (control, _) in &pins {
            considered[control.considered_index].disposition = ContributionDisposition::Effective;
            ids.push(control.id.clone());
        }
        ids.sort();
        actions.insert(AvailableAction::AppealOrRevokeManagedPin);
        return ResolutionOutcome::Effective {
            value: pinned_value,
            winning_fact_ids: ids,
        };
    }

    let Some(highest_rank) = candidates.iter().map(|candidate| candidate.rank).max() else {
        return ResolutionOutcome::NoValue;
    };
    let winners: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.rank == highest_rank)
        .collect();
    let first_value = &winners[0].value;
    if winners
        .iter()
        .any(|candidate| candidate.value != *first_value)
    {
        let mut ids: Vec<_> = winners
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect();
        ids.sort();
        mark_conflicting(considered, &ids);
        return ResolutionOutcome::UnresolvedConflict {
            fact_ids: ids,
            reason: "equal-rank values differ; arrival order cannot choose".to_owned(),
        };
    }
    let winner_ids: BTreeSet<_> = winners
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect();
    let winner_source = winners[0].source;
    for candidate in candidates.iter() {
        if winner_ids.contains(&candidate.id) {
            considered[candidate.considered_index].disposition = ContributionDisposition::Effective;
            considered[candidate.considered_index].reason =
                "wins the descriptor-declared ordinary ranking".to_owned();
        } else {
            considered[candidate.considered_index].disposition = ContributionDisposition::Losing;
            considered[candidate.considered_index].reason =
                format!("loses to higher-ranked {:?} contribution", winner_source);
        }
    }
    ResolutionOutcome::Effective {
        value: first_value.clone(),
        winning_fact_ids: winner_ids.into_iter().collect(),
    }
}

pub(super) fn consider_runtime_default(
    descriptor: &crate::preferences::PreferenceDescriptor,
    fact: &RuntimeDefaultFact,
    key: &PreferenceKey,
    present_sources: &mut BTreeSet<ResolutionSource>,
    considered: &mut Vec<ConsideredFact>,
    candidates: &mut Vec<Candidate>,
) {
    present_sources.insert(ResolutionSource::DescriptorDefault);
    let index = considered.len();
    let expected_recipe = match &descriptor.default_value {
        crate::preferences::DescriptorDefault::Runtime { recipe } => Some(recipe.as_str()),
        crate::preferences::DescriptorDefault::Literal(_)
        | crate::preferences::DescriptorDefault::Absent => None,
    };
    let (disposition, reason) = if fact.key != *key || expected_recipe != Some(&fact.recipe) {
        (
            ContributionDisposition::Refused,
            "runtime default does not match the descriptor key and registered recipe".to_owned(),
        )
    } else if !descriptor.validates(&fact.value) {
        (
            ContributionDisposition::Refused,
            "runtime default violates the authoritative descriptor schema".to_owned(),
        )
    } else {
        candidates.push(Candidate {
            id: fact.id.clone(),
            source: ResolutionSource::DescriptorDefault,
            value: fact.value.clone(),
            considered_index: index,
            rank: 0,
        });
        (
            ContributionDisposition::Losing,
            "validated registered runtime descriptor default admitted before ranking".to_owned(),
        )
    };
    considered.push(ConsideredFact {
        id: fact.id.clone(),
        source: ResolutionSource::DescriptorDefault,
        value: (fact.disclosure == ValueDisclosure::Disclosed).then(|| fact.value.clone()),
        directive: None,
        control: None,
        disposition,
        reason,
        provenance: fact.provenance.clone(),
        disclosure: fact.disclosure,
        provider_state: None,
        authority_release: None,
        remaining_freedom: None,
        appeal_path: None,
        reactivation_condition: None,
        context: None,
    });
}
