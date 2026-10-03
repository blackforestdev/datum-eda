//! Producing-artifact authority composes with native authored selection, never
//! with scene-derived identities or a second selected-state store.
use crate::selection_resolution::{
    NativeSelectionResolution, SelectionAuthority, SelectionModelRevision,
    SelectionResolutionError as Error,
};
use crate::{AuthoredSelectionIdentity as Id, ReviewWorkspaceState, SelectionSubject};
use std::collections::BTreeSet;

pub struct WorkspaceSelectionAuthority<'a> {
    pub native: &'a NativeSelectionResolution,
    pub workspace: &'a ReviewWorkspaceState,
}
impl WorkspaceSelectionAuthority<'_> {
    pub fn artifact(&self, subject: &SelectionSubject) -> Result<Option<serde_json::Value>, Error> {
        subject.validate().map_err(Error::InvalidSubject)?;
        match subject {
            SelectionSubject::Proposal(id) | SelectionSubject::Review(id) => {
                // Saved-route lifecycle metadata is not exposed by this
                // payload yet. Do not select an applied/discarded saved action
                // merely because its old file still contains that action ID.
                if self.workspace.review.artifact_path.is_some() {
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
    fields.insert("artifact_payload".into(), V::Common(payload));
    fields.insert("artifact_basis".into(),V::Common(match &view.subject {
        SelectionSubject::Diagnostic(_)=>serde_json::json!({"check_run_id":state.checks.check_run_id,"model_revision":state.checks.model_revision,"status":state.checks.status}),
        _=>serde_json::json!({"review_source":state.review.review_source,"artifact_path":state.review.artifact_path,"status":state.review.status}),
    }));
    // Related target payloads remain producing evidence/navigation, never
    // acquired authored members or an inferred occurrence binding.
    Ok(view)
}
