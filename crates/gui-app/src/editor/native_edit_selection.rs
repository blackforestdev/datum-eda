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

/// Acquire selects its actual refreshed definition, never a dependency that
/// happened to be created first. An unrelated refresh has no acquire intent.
fn refreshed_library(
    previous: &datum_gui_protocol::library_browser::LibraryBrowser,
    previous_selection: &SelectionTarget,
    mut next: datum_gui_protocol::library_browser::LibraryBrowser,
    acquire_succeeded: bool,
) -> (
    datum_gui_protocol::library_browser::LibraryBrowser,
    Option<String>,
) {
    let acquired = acquire_succeeded
        .then(|| {
            let preview = previous.acquisition.as_ref()?.preview.as_ref()?;
            let (kind, raw) = match &preview.footprint {
                Some(physical) => ("footprints", &physical.footprint),
                None => ("symbols", &preview.symbol),
            };
            let id = raw
                .get("uuid")?
                .as_str()?
                .parse::<eda_engine::substrate::ObjectId>()
                .ok()?;
            let key = format!("library:pool/{kind}/{id}.json");
            let mut matches = next
                .entries
                .iter()
                .filter(|entry| entry.id == id && entry.kind == kind && entry.key() == key);
            if matches.next()?.warning.is_some() || matches.next().is_some() {
                return None;
            }
            Some((kind, key))
        })
        .flatten();
    let created = previous
        .draft
        .as_ref()
        .map(|draft| format!("library:pool/{}/{}.json", draft.kind, draft.id));
    let prior = previous
        .selected(previous_selection)
        .map(|entry| entry.key());
    next.kind = previous.kind;
    next.search = previous.search.clone();
    next.result_offset = previous.result_offset;
    if let Some((kind, _)) = &acquired {
        next.kind = datum_gui_protocol::library_browser::KINDS
            .iter()
            .position(|(candidate, _)| candidate == kind)
            .unwrap_or(previous.kind);
        next.search.clear();
        next.result_offset = 0;
    }
    let selected = acquired
        .map(|(_, key)| key)
        .or(created)
        .or(prior)
        .filter(|key| next.entries.iter().any(|entry| entry.key() == *key));
    (next, selected)
}

impl Runtime {
    /// One atomic source/selection publication path for authored writes and
    /// generated check evidence. View state remains owned by the live session.
    pub(super) fn publish_prepared_workspace(
        &mut self,
        next: datum_gui_protocol::ReviewWorkspaceState,
        prepared: Prepared,
    ) {
        self.publish_edit_refresh(next, prepared, false);
    }

    pub(super) fn publish_edit_refresh(
        &mut self,
        mut next: datum_gui_protocol::ReviewWorkspaceState,
        prepared: Prepared,
        acquire_succeeded: bool,
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
        if !same_world {
            self.preserve_cameras_for_refresh(&next);
        }
        // Keep the new projection's one typed Workspace refusal, not its UI bag
        // or a copied Console history. Other consumer state stays locally owned.
        let refusal = next
            .ui
            .console
            .latest()
            .filter(|record| {
                record.source == datum_gui_protocol::ConsoleFeedbackSource::Workspace
                    && record.category == datum_gui_protocol::ConsoleFeedbackCategory::ActionRefusal
            })
            .map(|record| {
                datum_gui_protocol::ConsoleFeedbackDraft::action_refusal(
                    record.source,
                    record.occurred_unix_ms,
                    record.message.clone(),
                )
            });
        let workspace = self.session.workspace_mut();
        let (library, library_selection) = refreshed_library(
            &workspace.library,
            &workspace.selection,
            next.library,
            acquire_succeeded,
        );
        workspace.library = library;
        if !same_world {
            workspace.scene = next.scene;
            workspace.schematic_scene = next.schematic_scene;
            workspace.schematic_documents = next.schematic_documents;
        }
        let wire_route = workspace.schematic_authoring.wire_route;
        workspace.schematic_authoring = next.schematic_authoring;
        workspace.schematic_authoring.wire_route = wire_route;
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
        // Preserve the engine's native-write verb catalog (a build constant) across this
        // rebuild so truthful native-edit menu enablement is not reset to empty when the
        // new workspace was built without a fresh native.describe (e.g. a check-run refresh).
        if let Some(rebuilt) = next.backing.as_mut()
            && rebuilt.native_write_verbs.is_empty()
            && let Some(current) = workspace.backing.as_ref()
        {
            rebuilt.native_write_verbs = current.native_write_verbs.clone();
        }
        workspace.backing = next.backing;
        if let Some(refusal) = refusal {
            let already_visible = workspace.ui.console.latest().is_some_and(|record| {
                record.source == refusal.source
                    && record.category == refusal.category
                    && record.message == refusal.message
            });
            if !already_visible {
                workspace.ui.publish_console_feedback(refusal);
            }
        }
        workspace.selection = prepared.selected;
        if let Some(key) = library_selection {
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
