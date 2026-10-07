//! Producing-artifact authority composes with native authored selection, never
//! with scene-derived identities or a second selected-state store.
mod production;
pub use crate::check_runs::marker as diagnostic_markers;
use crate::selection_resolution::{
    NativeSelectionResolution, SelectionAuthority, SelectionModelRevision,
    SelectionResolutionError as Error,
};
use crate::{AuthoredSelectionIdentity as Id, ReviewWorkspaceState, SelectionSubject};
pub use production::ProductionProposalActionIdentity;
use std::collections::BTreeSet;

pub struct WorkspaceSelectionAuthority<'a> {
    pub native: &'a NativeSelectionResolution,
    pub model: &'a eda_engine::substrate::DesignModel,
    pub workspace: &'a ReviewWorkspaceState,
}
impl WorkspaceSelectionAuthority<'_> {
    pub fn artifact(&self, subject: &SelectionSubject) -> Result<Option<serde_json::Value>, Error> {
        subject.validate().map_err(Error::InvalidSubject)?;
        match subject {
            SelectionSubject::Proposal(id) | SelectionSubject::Review(id) => {
                if self.model.project.project_id != self.native.project()
                    || self.model.model_revision != *self.native.revision()
                {
                    return Err(Error::MissingArtifactAuthority);
                }
                if let SelectionSubject::Proposal(key) = subject
                    && let Ok(identity) =
                        serde_json::from_str::<ProductionProposalActionIdentity>(key)
                {
                    return self.production_proposal(&identity).map(|p| {
                        p.map(|proposal| serde_json::to_value(proposal).expect("typed proposal"))
                    });
                }
                if let Some(path) = &self.workspace.review.artifact_path {
                    let producer = self
                        .workspace
                        .review
                        .producer_proposal
                        .as_ref()
                        .ok_or(Error::MissingArtifactAuthority)?;
                    if producer.project_id != self.native.project()
                        || producer.schema_version != eda_engine::substrate::PROPOSAL_SCHEMA_VERSION
                        || producer.batch.expected_model_revision.as_ref()
                            != Some(&producer.prepared_against)
                        || !self
                            .workspace
                            .review
                            .source_fingerprint
                            .as_deref()
                            .is_some_and(|s| {
                                s.strip_prefix("sha256:").is_some_and(|digest| {
                                    digest.len() == 64
                                        && digest.bytes().all(|c| c.is_ascii_hexdigit())
                                })
                            })
                    {
                        return Err(Error::MissingArtifactAuthority);
                    }
                    if let Some(current) = self.model.proposals.get(&producer.proposal_id) {
                        if current.project_id != producer.project_id
                            || current.prepared_against != producer.prepared_against
                            || current.batch != producer.batch
                        {
                            return Err(Error::MissingArtifactAuthority);
                        }
                        if matches!(
                            current.status,
                            eda_engine::substrate::ProposalStatus::Applied
                                | eda_engine::substrate::ProposalStatus::Rejected
                        ) {
                            return Ok(None);
                        }
                    }
                    match std::fs::metadata(path) {
                        Ok(meta) if meta.is_file() => {}
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            return Ok(None);
                        }
                        _ => return Err(Error::MissingArtifactAuthority),
                    }
                } else if self.workspace.review.producer_revision.as_deref()
                    != Some(&self.native.revision().0)
                {
                    return Err(Error::MissingArtifactAuthority);
                }
                // The route producer supplies whole action IDs. Production
                // proposal UUIDs alone are not action authority (OPEN-12).
                if self.workspace.review.project_uuid.as_deref()
                    != Some(&self.native.project().to_string())
                {
                    return Err(Error::MissingArtifactAuthority);
                }
                let mut matches = self
                    .workspace
                    .review
                    .proposal_actions
                    .iter()
                    .filter(|a| a.action_id == *id);
                let found = matches.next();
                if matches.next().is_some() {
                    return Err(Error::MissingArtifactAuthority);
                }
                Ok(found.map(|a| serde_json::to_value(a).expect("typed action")))
            }
            SelectionSubject::Diagnostic(id) => {
                if self.workspace.checks.project_id.as_deref()
                    != Some(&self.native.project().to_string())
                {
                    return Err(Error::MissingArtifactAuthority);
                }
                let mut matches = self
                    .workspace
                    .checks
                    .findings
                    .iter()
                    .filter(|f| f.fingerprint == *id);
                let found = matches.next();
                if matches.next().is_some() {
                    return Err(Error::MissingArtifactAuthority);
                }
                Ok(found.map(|f| serde_json::to_value(f).expect("typed finding")))
            }
            _ => Err(Error::MissingArtifactAuthority),
        }
    }
}
impl SelectionAuthority for WorkspaceSelectionAuthority<'_> {
    fn project(&self) -> crate::SelectionProjectId {
        self.native.project()
    }
    fn revision(&self) -> &SelectionModelRevision {
        self.native.revision()
    }
    fn authored_contains(&self, id: &Id) -> bool {
        self.native.authored_contains(id)
    }
    fn derived_exists(&self, subject: &SelectionSubject) -> Result<bool, Error> {
        self.native.derived_exists(subject)
    }
    fn artifact_exists(&self, subject: &SelectionSubject) -> Result<bool, Error> {
        Ok(self.artifact(subject)?.is_some())
    }
    fn members(&self, subject: &SelectionSubject) -> Result<BTreeSet<Id>, Error> {
        if is_artifact(subject) {
            if !self.artifact_exists(subject)? {
                return Err(Error::MissingArtifactAuthority);
            }
            Ok(BTreeSet::new())
        } else {
            self.native.members(subject)
        }
    }
}
pub fn is_artifact(subject: &SelectionSubject) -> bool {
    matches!(
        subject,
        SelectionSubject::Proposal(_)
            | SelectionSubject::Review(_)
            | SelectionSubject::Diagnostic(_)
    )
}

pub fn build_workspace_selection_view(
    model: &eda_engine::substrate::DesignModel,
    native: &NativeSelectionResolution,
    subject: SelectionSubject,
    state: &ReviewWorkspaceState,
) -> Result<crate::selection_view::SelectionView, Error> {
    let visible = |layer: &str| {
        state.ui.filters.show_authored
            && state
                .ui
                .filters
                .layer_visibility
                .get(layer)
                .copied()
                .unwrap_or(true)
    };
    if !is_artifact(&subject) {
        return crate::selection_view::build_selection_view(
            model,
            native,
            subject,
            &state.scene,
            state.schematic_scene.as_ref(),
            visible,
        );
    }
    let authority = WorkspaceSelectionAuthority {
        native,
        model,
        workspace: state,
    };
    let payload = authority
        .artifact(&subject)?
        .ok_or(Error::MissingArtifactAuthority)?;
    let mut view = crate::selection_view::build_selection_view(
        model,
        native,
        SelectionSubject::None,
        &state.scene,
        state.schematic_scene.as_ref(),
        visible,
    )?;
    view.subject = subject;
    let fields = &mut view.inspection[0].fields;
    use crate::selection_view::InspectionValue as V;
    fields.insert(
        "artifact_kind".into(),
        V::Common(serde_json::json!(view.subject.kind())),
    );
    if let SelectionSubject::Diagnostic(id) = &view.subject {
        let finding = state
            .checks
            .findings
            .iter()
            .find(|f| f.fingerprint == *id)
            .expect("validated producer");
        for (name, value) in [
            ("fingerprint", &finding.fingerprint),
            ("severity", &finding.severity),
            ("rule", &finding.rule_id),
            ("message", &finding.message),
            ("explanation", &finding.explanation),
        ] {
            fields.insert(name.into(), V::Common(serde_json::json!(value)));
        }
        for (surface, content, scene) in [
            ("board", crate::PaneContent::Board, Some(&state.scene)),
            (
                "schematic",
                crate::PaneContent::Schematic,
                state.schematic_scene.as_ref(),
            ),
        ] {
            let projection = scene.map(|scene| {
                crate::check_runs::marker::MarkerProjectionIndex::capture(
                    scene,
                    &state.checks,
                    content,
                    visible,
                )
                .anchor(finding)
            });
            fields.insert(
                format!("{surface}_marker"),
                match projection {
                    Some(Ok(point)) => V::Common(
                        serde_json::json!({"navigation_anchor_nm":point,"selected_identity":id}),
                    ),
                    Some(Err(reason)) => V::Unavailable(reason.into()),
                    None => V::Unavailable("Surface is not available".into()),
                },
            );
        }
    }
    if let SelectionSubject::Proposal(key) = &view.subject
        && let Ok(identity) = serde_json::from_str::<ProductionProposalActionIdentity>(key)
    {
        let proposal = model
            .proposals
            .get(&identity.proposal_id)
            .expect("validated producer");
        view.stale = identity.prepared_against != model.model_revision;
        fields.insert(
            "producer_identity".into(),
            V::Common(serde_json::to_value(identity).unwrap()),
        );
        fields.insert(
            "artifact_basis".into(),
            V::Common(serde_json::json!({
                "canonical_status": proposal.status,
                "current_model_revision": model.model_revision,
                "stale": view.stale,
            })),
        );
        fields.insert("artifact_payload".into(), V::Common(payload));
        return Ok(view);
    }
    fields.insert("artifact_payload".into(), V::Common(payload));
    if !matches!(view.subject, SelectionSubject::Diagnostic(_)) {
        fields.insert("producer_identity".into(),V::Common(if let Some(proposal)=&state.review.producer_proposal {
            serde_json::json!({"project_id":proposal.project_id,"proposal_id":proposal.proposal_id,"batch_id":proposal.batch.batch_id,"prepared_against":proposal.prepared_against})
        } else {
            serde_json::json!({"project_id":view.project,"review_source":state.review.review_source,"net_uuid":state.review.net_uuid,"from_anchor_pad_uuid":state.review.from_anchor_pad_uuid,"to_anchor_pad_uuid":state.review.to_anchor_pad_uuid,"selection_profile":state.review.selection_profile,"selected_candidate":state.review.selected_candidate,"selected_policy":state.review.selected_policy})
        }));
    }
    fields.insert("artifact_basis".into(),V::Common(match &view.subject {
        SelectionSubject::Diagnostic(_)=>serde_json::json!({"check_run_id":state.checks.check_run_id,"model_revision":state.checks.model_revision,"status":state.checks.status}),
        _=>serde_json::json!({"review_source":state.review.review_source,"artifact_path":state.review.artifact_path,"producer_revision":state.review.producer_revision,"source_fingerprint":state.review.source_fingerprint,"producer_proposal_ref":state.review.producer_proposal.as_ref().map(|p| serde_json::json!({"proposal_id":p.proposal_id,"prepared_against":p.prepared_against})),"canonical_status":state.review.producer_proposal.as_ref().and_then(|p|model.proposals.get(&p.proposal_id)).map(|p|p.status),"status":state.review.status}),
    }));
    // Related target payloads remain producing evidence/navigation, never
    // acquired authored members or an inferred occurrence binding.
    Ok(view)
}

/// Action IDs are scoped by their actual producing proposal/context. Compare
/// that existing authority; never generate a replacement ID from scene data.
pub fn artifact_producer_changed(
    prior: &crate::selection_view::SelectionView,
    current: &crate::selection_view::SelectionView,
) -> bool {
    if !matches!(
        prior.subject,
        SelectionSubject::Proposal(_) | SelectionSubject::Review(_)
    ) || !is_artifact(&current.subject)
    {
        return false;
    }
    match (
        prior.field("producer_identity"),
        current.field("producer_identity"),
    ) {
        (
            crate::selection_view::InspectionValue::Common(a),
            crate::selection_view::InspectionValue::Common(b),
        ) => a != b,
        _ => false,
    }
}
