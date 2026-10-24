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
    layout: datum_gui_protocol::WorkspaceLayout,
    navigation: datum_gui_protocol::schematic_documents::SchematicNavigation,
    filters: WorkspaceFilterState,
    prior: Option<Arc<NativeSelectionResolution>>,
}
impl Inputs {
    pub fn capture(runtime: &Runtime) -> Self {
        Self {
            selected: runtime.workspace().selection.clone(),
            layout: runtime.workspace().ui.layout.clone(),
            navigation: runtime.workspace().schematic_documents.navigation(),
            filters: runtime.workspace().ui.filters.clone(),
            prior: runtime
                .native_selection
                .as_ref()
                .map(|s| s.lifetime_basis()),
        }
    }
    pub fn matches(&self, runtime: &Runtime) -> bool {
        self.matches_workspace(runtime.workspace())
    }
    fn matches_workspace(&self, workspace: &datum_gui_protocol::ReviewWorkspaceState) -> bool {
        self.selected == workspace.selection
            && self.filters == workspace.ui.filters
            && self.layout == workspace.ui.layout
            && self.navigation == workspace.schematic_documents.navigation()
    }
}
pub(super) struct Prepared {
    pub inputs: Inputs,
    pub selected: SelectionTarget,
    pub session: runtime_selection::NativeSelectionSession,
    pub cache: runtime_selection::SelectionReadCache,
}
pub(super) fn prepare_from_model(
    next: &mut datum_gui_protocol::ReviewWorkspaceState,
    inputs: Inputs,
    model: Arc<eda_engine::substrate::DesignModel>,
    cache: runtime_selection::SelectionReadCache,
) -> Result<Prepared> {
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
    next.ui.layout = inputs.layout.clone();
    if inputs.navigation.project().is_some() {
        let root = &next
            .backing
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Document refresh source unavailable"))?
            .request
            .project_root;
        next.schematic_documents.restore_views(
            &inputs.navigation,
            &next.ui.layout,
            root,
            &model,
        )?;
    }
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
                let mut explanation = reconcile.state.explanation();
                if is_artifact(&prior.subject) && reconcile.dissolved {
                    explanation = Some("Producing artifact invalidated its selected identity; selection cleared".into());
                }
                let mut view = build_workspace_selection_view(&model, &authority, reconcile.subject, next)
                    .map_err(|e| anyhow::anyhow!("Selection projection unavailable: {e:?}"))?;
                if datum_gui_protocol::selection_artifacts::artifact_producer_changed(prior, &view) {
                    explanation = Some("Selected action producer changed; selection cleared".into());
                    view = build_workspace_selection_view(&model, &authority, datum_gui_protocol::SelectionSubject::None, next)
                        .map_err(|e| anyhow::anyhow!("Selection projection unavailable: {e:?}"))?;
                }
                view.scope = prior.scope.filter(|class| view.members.iter().any(|id| id.class == *class));
                target_after_reconciliation(view, reconcile.dropped, explanation)
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

/// Keep a current empty projection to disclose actual lifetime results.
/// The report holds no selected identity and is never a resurrection source.
pub(super) fn target_after_reconciliation(
    mut view: datum_gui_protocol::selection_view::SelectionView,
    dropped: std::collections::BTreeSet<datum_gui_protocol::AuthoredSelectionIdentity>,
    explanation: Option<String>,
) -> SelectionTarget {
    view.notice = explanation;
    view.dropped = dropped;
    if !view.dropped.is_empty() {
        view.notice = Some(format!(
            "{} deleted or stale identities dropped; focus was not promoted",
            view.dropped.len()
        ));
    }
    if matches!(view.subject, datum_gui_protocol::SelectionSubject::None)
        && view.dropped.is_empty()
        && view.notice.is_none()
    {
        SelectionTarget::None
    } else {
        SelectionTarget::Native(Box::new(view))
    }
}

impl Runtime {
    /// One atomic source/selection publication path for authored writes and
    /// generated check evidence. View state remains owned by the live session.
    pub(super) fn publish_prepared_workspace(
        &mut self,
        next: datum_gui_protocol::ReviewWorkspaceState,
        prepared: Prepared,
    ) {
        let same_world = {
            let current = self.workspace();
            current.scene == next.scene
                && current.schematic_scene == next.schematic_scene
                && current.review == next.review
                && current.ui.layout == next.ui.layout
                && current
                    .schematic_documents
                    .same_source(&next.schematic_documents)
                && current.schematic_documents.navigation() == next.schematic_documents.navigation()
                && current.ui.layout.leaves().into_iter().all(|pane| {
                    datum_gui_protocol::camera_scene_for_pane(current, pane)
                        == datum_gui_protocol::camera_scene_for_pane(&next, pane)
                })
        };
        let workspace = self.session.workspace_mut();
        let library_selection = workspace
            .library
            .selected(&workspace.selection)
            .map(|e| e.key());
        let created = workspace
            .library
            .draft
            .as_ref()
            .map(|d| format!("library:pool/{}/{}.json", d.kind, d.id));
        let mut library = next.library;
        library.kind = workspace.library.kind;
        library.search = workspace.library.search.clone();
        library.result_offset = workspace.library.result_offset;
        workspace.library = library;
        if !same_world {
            workspace.scene = next.scene;
            workspace.schematic_scene = next.schematic_scene;
            workspace.schematic_documents = next.schematic_documents;
        }
        workspace.schematic_authoring = next.schematic_authoring;
        if !same_world {
            workspace.review = next.review;
        }
        workspace.production = next.production;
        workspace.source_shards = next.source_shards;
        workspace.checks = next.checks;
        workspace.ui.console_journal.reconcile(
            &next.supervision.journal.projection,
            next.supervision.journal.applied_transaction_count,
        );
        workspace.supervision = next.supervision;
        workspace.backing = next.backing;
        workspace.selection = prepared.selected;
        if let Some(key) = created.or(library_selection)
            && workspace.library.entries.iter().any(|e| e.key() == key)
        {
            workspace.selection = datum_gui_protocol::SelectionTarget::AuthoredObject(key);
        }
        self.selection_run.cancel();
        self.native_selection = Some(prepared.session);
        self.selection_read_cache = prepared.cache;
        self.refresh_selection_accessibility();
        if same_world {
            self.render_sources.projections_changed();
            self.invalidate_frame();
        } else {
            self.invalidate_scene();
        }
    }
}

#[cfg(test)]
#[path = "native_edit_selection/lifetime_report_tests.rs"]
mod lifetime_report_tests;

#[cfg(test)]
#[path = "native_edit_selection/artifact_lifetime_tests.rs"]
mod artifact_lifetime_tests;

#[cfg(test)]
#[path = "native_edit_selection/run_lifetime_tests.rs"]
mod run_lifetime_tests;

#[cfg(test)]
#[path = "native_edit_selection/document_navigation_tests.rs"]
mod document_navigation_tests;
