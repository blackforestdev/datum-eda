//! Existing selection lifetime and complete projection prepared off dispatch.
use super::*;
use datum_gui_protocol::{
    SelectionTarget, WorkspaceFilterState,
    selection_artifacts::{
        WorkspaceSelectionAuthority, build_workspace_selection_view, is_artifact,
    },
    selection_resolution::NativeSelectionResolution,
};
use std::sync::Arc;

pub(super) struct Inputs {
    selected: SelectionTarget,
    filters: WorkspaceFilterState,
    prior: Option<Arc<NativeSelectionResolution>>,
}
impl Inputs {
    pub fn capture(runtime: &Runtime) -> Self {
        Self {
            selected: runtime.workspace().selection.clone(),
            filters: runtime.workspace().ui.filters.clone(),
            prior: runtime
                .native_selection
                .as_ref()
                .map(|s| s.lifetime_basis()),
        }
    }
    pub fn matches(&self, runtime: &Runtime) -> bool {
        self.selected == runtime.workspace().selection
            && self.filters == runtime.workspace().ui.filters
    }
}
pub(super) struct Prepared {
    pub inputs: Inputs,
    pub selected: SelectionTarget,
    pub session: runtime_selection::NativeSelectionSession,
    pub cache: runtime_selection::SelectionReadCache,
}
pub(super) fn prepare(
    root: &std::path::Path,
    next: &mut datum_gui_protocol::ReviewWorkspaceState,
    inputs: Inputs,
) -> Result<Prepared> {
    let mut cache = runtime_selection::SelectionReadCache::default();
    let model = cache.resolve(root)?;
    anyhow::ensure!(
        model.model_revision.0 == next.scene.source_revision
            && model.project.project_id.to_string() == next.scene.project_uuid
            && eda_engine::revision::transaction_tip(&model.journal).map(|t| t.to_string())
                == next.supervision.journal.accepted_transaction_tip,
        "Selection source changed during edit refresh"
    );
    let authority =
        Arc::new(NativeSelectionResolution::capture(&model).map_err(|e| anyhow::anyhow!("{e:?}"))?);
    next.ui.filters = inputs.filters.clone();
    let mut lineage = None;
    let selected = if let SelectionTarget::Native(prior) = &inputs.selected {
        let reconcile = if is_artifact(&prior.subject) {
            datum_gui_viewport::selection_lifetime::reconcile_selection(
                &prior.subject,
                prior.project,
                &WorkspaceSelectionAuthority {
                    native: &authority,
                    model: &model,
                    workspace: next,
                },
            )
        } else if let Some(previous) = &inputs.prior {
            datum_gui_viewport::selection_lifetime::reconcile_native_selection(
                &prior.subject,
                prior.project,
                previous,
                &authority,
            )
        } else {
            datum_gui_viewport::selection_lifetime::reconcile_selection(
                &prior.subject,
                prior.project,
                &*authority,
            )
        };
        match reconcile {
            Ok(reconcile)
                if !matches!(
                    reconcile.state,
                    datum_gui_viewport::selection_lifetime::SelectionReconciliationState::Suspended { .. }
                ) =>
            {
                let mut view = build_workspace_selection_view(&model, &authority, reconcile.subject, next)
                    .map_err(|e| anyhow::anyhow!("Selection projection unavailable: {e:?}"))?;
                if datum_gui_protocol::selection_artifacts::artifact_producer_changed(prior, &view) {
                    view.subject = datum_gui_protocol::SelectionSubject::None;
                }
                view.scope = prior.scope.filter(|class| view.members.iter().any(|id| id.class == *class));
                view.notice = reconcile.state.explanation();
                view.dropped = reconcile.dropped;
                if !view.dropped.is_empty() {
                    view.notice = Some(format!("{} deleted or stale identities dropped; focus was not promoted", view.dropped.len()));
                }
                if matches!(view.subject, datum_gui_protocol::SelectionSubject::None) {
                    SelectionTarget::None
                } else {
                    SelectionTarget::Native(Box::new(view))
                }
            }
            outcome => {
                let mut view = prior.clone();
                view.stale = true;
                view.board_keys.clear();
                view.schematic_keys.clear();
                view.notice = Some(match outcome {
                    Ok(r) => r.state.explanation().unwrap_or_else(|| "Selection source unavailable".into()),
                    Err(e) => format!("Selection revision unavailable: {e:?}"),
                });
                lineage = inputs.prior.clone();
                SelectionTarget::Native(view)
            }
        }
    } else {
        inputs.selected.clone()
    };
    Ok(Prepared {
        inputs,
        selected,
        session: runtime_selection::NativeSelectionSession::prepared(model, authority, lineage),
        cache,
    })
}
