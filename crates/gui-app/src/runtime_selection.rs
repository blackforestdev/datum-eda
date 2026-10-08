//! Native selection entry and revision integration; editors configure one owner.
use super::*;
use datum_gui_protocol::{
    SelectionSubject, SelectionTarget,
    selection_artifacts::{
        WorkspaceSelectionAuthority, build_workspace_selection_view, is_artifact,
    },
    selection_resolution::NativeSelectionResolution,
};
use datum_gui_viewport::selection_acquisition::{
    AcquisitionEntry, AcquisitionMode, acquire_authored,
};
use eda_engine::substrate::{DesignModel, ProjectReadCache};
use std::collections::BTreeSet;
use std::sync::Arc;
mod publication;

// The session owns its required model. Optional reuse holds only a Weak model
// reference and the engine's existing bounded, byte-compared input evidence.
pub(super) struct SelectionReadCache {
    cache: ProjectReadCache,
    owner: datum_gui_render::cpu_alloc::Scope,
}
impl Default for SelectionReadCache {
    fn default() -> Self {
        Self {
            cache: ProjectReadCache::nonretaining(),
            owner: datum_gui_render::cpu_alloc::Scope::new("selection-resolver"),
        }
    }
}
impl SelectionReadCache {
    pub(super) fn resolve(&mut self, root: &std::path::Path) -> anyhow::Result<Arc<DesignModel>> {
        Ok(self.owner.with(|| self.cache.resolve(root))?)
    }
}

pub(super) struct NativeSelectionSession {
    pub model: Arc<DesignModel>,
    pub authority: Arc<NativeSelectionResolution>,
    last_click: Option<(
        datum_gui_protocol::AuthoredSelectionIdentity,
        (f32, f32),
        std::time::Instant,
        u8,
    )>,
    lineage: Option<Arc<NativeSelectionResolution>>,
    pub(super) binding_bases:
        std::collections::BTreeMap<String, std::sync::Arc<runtime_selection_hit::BindingBasis>>,
    pub(super) region_bases: std::collections::BTreeMap<
        String,
        std::sync::Arc<selection_region_evaluation::RegionBasis>,
    >,
}
impl NativeSelectionSession {
    pub(super) fn prepared(
        model: Arc<DesignModel>,
        authority: Arc<NativeSelectionResolution>,
        lineage: Option<Arc<NativeSelectionResolution>>,
    ) -> Self {
        Self {
            model,
            authority,
            lineage,
            last_click: None,
            region_bases: Default::default(),
            binding_bases: Default::default(),
        }
    }
    pub(super) fn lifetime_basis(&self) -> Arc<NativeSelectionResolution> {
        self.lineage
            .clone()
            .unwrap_or_else(|| self.authority.clone())
    }
    pub(super) fn reset_click_depth(&mut self) {
        self.last_click = None;
    }
    /// Proposal metadata is outside the source-model revision. Refresh its
    /// canonical lifetime while retaining the unchanged native geometry basis.
    fn refresh_same_revision(&mut self, model: Arc<DesignModel>) -> Option<Arc<DesignModel>> {
        if self.model.project.project_id != model.project.project_id
            || self.model.model_revision != model.model_revision
        {
            return Some(model);
        }
        self.model = model;
        None
    }
}
impl Runtime {
    pub(super) fn ensure_selection_authority(&mut self) -> anyhow::Result<bool> {
        let Some(backing) = &self.workspace().backing else {
            return Ok(false);
        };
        if backing.request.board_file.is_some() {
            return Ok(false);
        }
        let project_root = backing.request.project_root.clone();
        crate::append_gui_verbose_diagnostic_line(|| "selection resolve begin");
        let model = self.selection_read_cache.resolve(&project_root)?;
        crate::append_gui_verbose_diagnostic_line(|| "selection resolve end");
        let model = if let Some(session) = &mut self.native_selection {
            match session.refresh_same_revision(model) {
                Some(model) => model,
                None => {
                    self.reconcile_native_artifact_selection();
                    return Ok(true);
                }
            }
        } else {
            model
        };
        let authority = NativeSelectionResolution::capture(&model)
            .map_err(|e| anyhow::anyhow!("Selection source unavailable: {e:?}"))?;
        let reconcile = if let (Some(previous), SelectionTarget::Native(view)) =
            (&self.native_selection, &self.workspace().selection)
        {
            Some(
                if is_artifact(&view.subject) {
                    datum_gui_viewport::selection_lifetime::reconcile_selection(
                        &view.subject,
                        view.project,
                        &WorkspaceSelectionAuthority {
                            native: &authority,
                            model: &model,
                            workspace: self.workspace(),
                        },
                    )
                } else {
                    datum_gui_viewport::selection_lifetime::reconcile_native_selection(
                        &view.subject,
                        view.project,
                        previous.lineage.as_ref().unwrap_or(&previous.authority),
                        &authority,
                    )
                }
                .map_err(|e| anyhow::anyhow!("Selection revision unavailable: {e:?}")),
            )
        } else {
            None
        };
        let reconcile = match reconcile.transpose() {
            Ok(value) => value,
            Err(error) => {
                if let SelectionTarget::Native(view) = &mut self.session.workspace_mut().selection
                    && is_artifact(&view.subject)
                {
                    view.stale = true;
                }
                return Err(error);
            }
        };
        // Capture/refuse atomically before replacing the previous complete basis.
        if let Some(reconcile) = reconcile {
            if matches!(
                reconcile.state,
                datum_gui_viewport::selection_lifetime::SelectionReconciliationState::Suspended { .. }
            ) {
                self.selection_notice(reconcile.state.explanation().unwrap());
                if let SelectionTarget::Native(view) = &mut self.session.workspace_mut().selection {
                    view.stale = true;
                    view.board_keys.clear();
                    view.schematic_keys.clear();
                }
                let previous = self.native_selection.take().unwrap();
                self.native_selection = Some(NativeSelectionSession {
                    model,
                    authority: Arc::new(authority),
                    last_click: None,
                    region_bases: Default::default(),
                    binding_bases: Default::default(),
                    lineage: Some(previous.lineage.unwrap_or(previous.authority)),
                });
                self.refresh_selection_accessibility();
                return Ok(true);
            }
            let mut artifact_dissolved = reconcile.dissolved
                && matches!(&self.workspace().selection, SelectionTarget::Native(prior) if is_artifact(&prior.subject));
            let state = self.workspace();
            let mut view =
                build_workspace_selection_view(&model, &authority, reconcile.subject, state)
                    .map_err(|e| anyhow::anyhow!("Selection projection unavailable: {e:?}"))?;
            if let SelectionTarget::Native(previous) = &state.selection {
                if datum_gui_protocol::selection_artifacts::artifact_producer_changed(
                    previous, &view,
                ) {
                    view.subject = SelectionSubject::None;
                    artifact_dissolved = true;
                }
                view.scope = previous
                    .scope
                    .filter(|class| view.members.iter().any(|id| id.class == *class));
            }
            view.notice = reconcile.state.explanation();
            view.dropped = reconcile.dropped;
            if !view.dropped.is_empty() {
                view.notice = Some(format!(
                    "{} deleted or stale identities dropped; focus was not promoted",
                    view.dropped.len()
                ));
            }
            self.session.workspace_mut().selection =
                if matches!(view.subject, SelectionSubject::None) {
                    SelectionTarget::None
                } else {
                    SelectionTarget::Native(Box::new(view))
                };
            if artifact_dissolved {
                self.log_console_echo(
                    ConsoleFeedbackSource::Selection,
                    "Producing artifact invalidated its selected identity; selection cleared",
                );
            }
        }
        self.native_selection = Some(NativeSelectionSession {
            model,
            authority: Arc::new(authority),
            last_click: None,
            region_bases: Default::default(),
            binding_bases: Default::default(),
            lineage: None,
        });
        self.refresh_selection_accessibility();
        Ok(true)
    }
    pub(super) fn selection_notice(&mut self, message: String) {
        if let SelectionTarget::Native(view) = &mut self.session.workspace_mut().selection {
            view.notice = Some(message.clone());
        }
        self.refresh_selection_accessibility();
        self.log_console_refusal(ConsoleFeedbackSource::Selection, message);
        self.invalidate_frame();
    }
    pub(super) fn acquire_native_key(
        &mut self,
        key: &str,
        surface: SceneSurface,
        point: Option<PointNm>,
    ) -> Option<bool> {
        let basis = match self.native_binding_basis(surface) {
            Ok(Some(basis)) => basis,
            Ok(None) => return None,
            Err(error) => {
                self.selection_notice(error.to_string());
                return Some(true);
            }
        };
        self.acquire_native_key_from_basis(key, point, &basis)
    }
    /// Hit qualification and acquisition within one synchronous action use the
    /// same freshly resolved source. Direct-key entry resolves before calling.
    pub(super) fn acquire_native_key_from_basis(
        &mut self,
        key: &str,
        point: Option<PointNm>,
        basis: &runtime_selection_hit::BindingBasis,
    ) -> Option<bool> {
        if self.modifiers.shift_key() && self.modifiers.control_key() {
            return Some(true);
        }
        let Some(binding) = basis.bindings.get(key) else {
            self.selection_notice(
                "This source has no authoritative workspace selection identity".into(),
            );
            return Some(true);
        };
        if !binding.eligible(&self.workspace().ui.filters) {
            return Some(true);
        }
        let id = binding.acquired.clone();
        let mode = if self.modifiers.control_key() {
            AcquisitionMode::Remove
        } else if self.modifiers.shift_key() {
            AcquisitionMode::Add
        } else {
            AcquisitionMode::Replace
        };
        let candidate = None;
        if mode == AcquisitionMode::Replace {
            let screen = self.last_cursor_pos.unwrap_or((0.0, 0.0));
            let native = self.native_selection.as_mut().unwrap();
            let now = std::time::Instant::now();
            let depth = native
                .last_click
                .as_ref()
                .filter(|(previous, old_screen, at, _)| {
                    previous == &id
                        && (screen.0 - old_screen.0).hypot(screen.1 - old_screen.1) < 4.0
                        && now.duration_since(*at) < std::time::Duration::from_millis(500)
                })
                .map(|(_, _, _, d)| (*d + 1).min(3))
                .unwrap_or(1);
            native.last_click = Some((id.clone(), screen, now, depth));
            if depth >= 2 && id.can_originate_run() {
                let request = crate::runtime_selection_run::Request {
                    model: native.model.clone(),
                    origin: id.clone(),
                    point: point.map(|p| eda_engine::ir::geometry::Point::new(p.x, p.y)),
                    depth,
                    inputs: Arc::new(crate::runtime_selection_run::ProjectionInputs::capture(
                        self.workspace(),
                    )),
                };
                if let Err(error) = self.selection_run.schedule(request) {
                    self.selection_notice(error);
                }
                return Some(true);
            }
        } else {
            self.native_selection.as_mut().unwrap().last_click = None;
        }
        Some(self.publish_native_acquisition(
            [id].into_iter().collect(),
            mode,
            AcquisitionEntry::Individual,
            candidate,
        ))
    }
    pub(super) fn publish_native_acquisition(
        &mut self,
        candidates: BTreeSet<datum_gui_protocol::AuthoredSelectionIdentity>,
        mode: AcquisitionMode,
        entry: AcquisitionEntry,
        semantic: Option<SelectionSubject>,
    ) -> bool {
        self.publish_native_acquisition_prepared(candidates, mode, entry, semantic, None)
    }
    pub(super) fn native_select_all(&mut self) -> bool {
        self.native_select_all_on(
            self.workspace().ui.layout.focused_content()
                == datum_gui_protocol::PaneContent::Schematic,
        )
    }
    pub(super) fn native_select_all_on(&mut self, schematic: bool) -> bool {
        self.cancel_native_selection_gesture();
        match self.ensure_selection_authority() {
            Ok(true) => {}
            Ok(false) => return false,
            Err(e) => {
                self.selection_notice(e.to_string());
                return true;
            }
        }
        let state = self.workspace();
        let authority = &self.native_selection.as_ref().unwrap().authority;
        let root_sheet = state.schematic_scene.as_ref().and_then(|s| {
            s.board_uuid
                .parse::<datum_gui_protocol::SelectionProjectId>()
                .ok()
        });
        let source = authority.snapshot().schematic_source();
        let candidates = authority
            .authored_identities()
            .iter()
            .filter(|id| {
                use datum_gui_protocol::AuthoredSelectionClass as C;
                if matches!(id.class, C::Pad | C::Pin) {
                    return false;
                }
                let is_schematic = matches!(
                    id.class,
                    C::Symbol
                        | C::Wire
                        | C::BusSection
                        | C::Label
                        | C::Port
                        | C::Junction
                        | C::NoConnect
                        | C::SchematicText
                        | C::SchematicGraphic
                );
                if schematic != is_schematic {
                    return false;
                }
                if !schematic {
                    return true;
                }
                if !id.instance_path.is_empty() {
                    return false;
                }
                source
                    .and_then(|s| root_sheet.and_then(|sheet| s.sheets.get(&sheet)))
                    .is_some_and(|sheet| match id.class {
                        C::Symbol => sheet.symbols.contains_key(&id.id),
                        C::Wire => sheet.wires.contains_key(&id.id),
                        C::BusSection => sheet.buses.contains_key(&id.id),
                        C::Label => sheet.labels.contains_key(&id.id),
                        C::Port => sheet.ports.contains_key(&id.id),
                        C::Junction => sheet.junctions.contains_key(&id.id),
                        C::NoConnect => sheet.noconnects.contains_key(&id.id),
                        C::SchematicText => sheet.texts.contains_key(&id.id),
                        C::SchematicGraphic => sheet.drawings.contains_key(&id.id),
                        _ => false,
                    })
            })
            .cloned()
            .collect();
        self.native_selection.as_mut().unwrap().last_click = None;
        self.publish_native_acquisition(
            candidates,
            AcquisitionMode::Replace,
            AcquisitionEntry::Global,
            None,
        )
    }
    pub(super) fn set_selection_scope(
        &mut self,
        scope: Option<datum_gui_protocol::AuthoredSelectionClass>,
    ) -> bool {
        let SelectionTarget::Native(view) = &mut self.session.workspace_mut().selection else {
            return false;
        };
        if scope.is_some_and(|class| !view.members.iter().any(|id| id.class == class)) {
            return false;
        }
        view.scope = scope;
        self.refresh_selection_accessibility();
        self.session
            .workspace_mut()
            .ui
            .selection_inspector
            .reset_scroll();
        self.invalidate_frame();
        true
    }
}

impl Runtime {
    pub(super) fn apply_session_result(
        &mut self,
        result: datum_gui_protocol::SessionCommandResult,
    ) -> bool {
        if !result.handled {
            return false;
        }
        let selection_changed = result
            .events
            .iter()
            .any(|event| matches!(event, SessionEvent::SelectionChanged(_)));
        for event in result.events {
            match event {
                // Shared source dependency validation owns retained reuse/history.
                SessionEvent::SceneChanged | SessionEvent::SelectionChanged(_) => {
                    self.invalidate_frame()
                }
                SessionEvent::FrameChanged => self.invalidate_frame(),
                SessionEvent::ToolChanged(_) => self.invalidate_frame(),
            }
        }
        if selection_changed {
            self.refresh_selection_accessibility();
        }
        true
    }

    pub(super) fn dispatch_session_command(&mut self, command: SessionCommand) -> bool {
        if matches!(command, SessionCommand::CancelAuthoringGesture) && self.cancel_native_move() {
            return true;
        }

        if let Some(handled) = self.dispatch_native_artifact_command(&command) {
            return handled;
        }
        let refresh = matches!(
            command,
            SessionCommand::ToggleLayerVisibility(_) | SessionCommand::ToggleShowAuthored
        );
        let result = self.session.apply(command);
        if refresh {
            self.refresh_native_selection_projection();
        }
        self.apply_session_result(result)
    }
}

impl Runtime {
    pub(super) fn refresh_native_selection_projection(&mut self) {
        let (Some(native), SelectionTarget::Native(previous)) =
            (&self.native_selection, &self.workspace().selection)
        else {
            return;
        };
        if previous.stale {
            return;
        }
        let state = self.workspace();
        match build_workspace_selection_view(
            &native.model,
            &native.authority,
            previous.subject.clone(),
            state,
        ) {
            Ok(mut view) => {
                view.scope = previous.scope;
                view.notice = previous.notice.clone();
                view.dropped = previous.dropped.clone();
                self.session.workspace_mut().selection = SelectionTarget::Native(Box::new(view));
            }
            Err(error) => {
                self.selection_notice(format!("Selection projection unavailable: {error:?}"))
            }
        }
        self.refresh_selection_accessibility();
    }
}

#[cfg(test)]
#[path = "runtime_selection_metadata_tests.rs"]
mod metadata_tests;
