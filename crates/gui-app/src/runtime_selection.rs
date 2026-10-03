//! Native selection entry and revision integration; editors configure one owner.
use super::*;
use datum_gui_protocol::{
    SelectionSubject, SelectionTarget, selection_resolution::NativeSelectionResolution,
    selection_view::build_selection_view,
};
use datum_gui_viewport::selection_acquisition::{
    AcquisitionEntry, AcquisitionMode, acquire_authored,
};
use eda_engine::substrate::{DesignModel, ProjectResolver};
use std::collections::BTreeSet;

pub(super) struct NativeSelectionSession {
    pub model: DesignModel,
    pub authority: NativeSelectionResolution,
    last_click: Option<(
        datum_gui_protocol::AuthoredSelectionIdentity,
        (f32, f32),
        std::time::Instant,
        u8,
    )>,
    lineage: Option<NativeSelectionResolution>,
}
impl NativeSelectionSession {
    pub(super) fn reset_click_depth(&mut self) {
        self.last_click = None;
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
        let model = ProjectResolver::new(&backing.request.project_root).resolve()?;
        if self
            .native_selection
            .as_ref()
            .is_some_and(|s| s.model.model_revision == model.model_revision)
        {
            return Ok(true);
        }
        let authority = NativeSelectionResolution::capture(&model)
            .map_err(|e| anyhow::anyhow!("Selection source unavailable: {e:?}"))?;
        let reconcile = if let (Some(previous), SelectionTarget::Native(view)) =
            (&self.native_selection, &self.workspace().selection)
        {
            Some(
                datum_gui_viewport::selection_lifetime::reconcile_native_selection(
                    &view.subject,
                    view.project,
                    previous.lineage.as_ref().unwrap_or(&previous.authority),
                    &authority,
                )
                .map_err(|e| anyhow::anyhow!("Selection revision unavailable: {e:?}"))?,
            )
        } else {
            None
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
                    authority,
                    last_click: None,
                    lineage: Some(previous.lineage.unwrap_or(previous.authority)),
                });
                return Ok(true);
            }
            let state = self.workspace();
            let mut view = build_selection_view(
                &model,
                &authority,
                reconcile.subject,
                &state.scene,
                state.schematic_scene.as_ref(),
                |layer| {
                    state.ui.filters.show_authored
                        && state
                            .ui
                            .filters
                            .layer_visibility
                            .get(layer)
                            .copied()
                            .unwrap_or(true)
                },
            )
            .map_err(|e| anyhow::anyhow!("Selection projection unavailable: {e:?}"))?;
            if let SelectionTarget::Native(previous) = &state.selection {
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
        }
        self.native_selection = Some(NativeSelectionSession {
            model,
            authority,
            last_click: None,
            lineage: None,
        });
        Ok(true)
    }
    pub(super) fn selection_notice(&mut self, message: String) {
        if let SelectionTarget::Native(view) = &mut self.session.workspace_mut().selection {
            view.notice = Some(message.clone());
        }
        self.log_console_refusal(ConsoleFeedbackSource::Selection, message);
        self.invalidate_frame();
    }
    pub(super) fn acquire_native_key(
        &mut self,
        key: &str,
        surface: SceneSurface,
        point: Option<PointNm>,
    ) -> Option<bool> {
        match self.ensure_selection_authority() {
            Ok(false) => return None,
            Err(e) => {
                self.selection_notice(e.to_string());
                return Some(true);
            }
            _ => {}
        }
        if self.modifiers.shift_key() && self.modifiers.control_key() {
            return Some(true);
        }
        let state = self.workspace();
        let scene = match surface {
            SceneSurface::Board => &state.scene,
            SceneSurface::Schematic => state.schematic_scene.as_ref()?,
        };
        let authority = &self.native_selection.as_ref()?.authority;
        if !state.ui.filters.show_authored
            || !datum_gui_protocol::selection_scene::candidate_visible(scene, key, |l| {
                state
                    .ui
                    .filters
                    .layer_visibility
                    .get(l)
                    .copied()
                    .unwrap_or(true)
            })
        {
            return Some(true);
        }
        let Some(id) =
            datum_gui_protocol::selection_scene::authored_candidate(scene, key, authority)
        else {
            self.selection_notice(
                "This source has no authoritative workspace selection identity".into(),
            );
            return Some(true);
        };
        if state
            .ui
            .filters
            .excluded_selection_classes
            .contains(&id.class)
        {
            return Some(true);
        }
        let mode = if self.modifiers.control_key() {
            AcquisitionMode::Remove
        } else if self.modifiers.shift_key() {
            AcquisitionMode::Add
        } else {
            AcquisitionMode::Replace
        };
        let mut candidate = None;
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
                match native
                    .authority
                    .acquire_run(
                        id.clone(),
                        point.map(|p| eda_engine::ir::geometry::Point::new(p.x, p.y)),
                    )
                    .and_then(|run| {
                        if depth >= 3 {
                            let SelectionSubject::Run(run) = run else {
                                unreachable!()
                            };
                            native.authority.widen_run(&run)
                        } else {
                            Ok(run)
                        }
                    }) {
                    Ok(subject) => candidate = Some(subject),
                    Err(e) => {
                        self.selection_notice(format!("Electrical selection refused: {e:?}"));
                        return Some(true);
                    }
                }
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
        let Some(native) = &self.native_selection else {
            return false;
        };
        let prior = match &self.workspace().selection {
            SelectionTarget::Native(v) => v.subject.clone(),
            _ => SelectionSubject::None,
        };
        let outcome = match acquire_authored(&prior, candidates, mode, entry, &native.authority) {
            Ok(o) => o,
            Err(e) => {
                self.selection_notice(format!("Selection refused atomically: {e:?}"));
                return true;
            }
        };
        let announce = outcome.refusal.is_some()
            || outcome.changed
            || semantic.as_ref().is_some_and(|s| *s != prior);
        let subject = semantic.unwrap_or(outcome.subject);
        let state = self.workspace();
        let mut view = match build_selection_view(
            &native.model,
            &native.authority,
            subject,
            &state.scene,
            state.schematic_scene.as_ref(),
            |layer| {
                state.ui.filters.show_authored
                    && state
                        .ui
                        .filters
                        .layer_visibility
                        .get(layer)
                        .copied()
                        .unwrap_or(true)
            },
        ) {
            Ok(v) => v,
            Err(e) => {
                self.selection_notice(format!("Selection projection refused: {e:?}"));
                return true;
            }
        };
        if let SelectionTarget::Native(previous) = &state.selection
            && view.subject == previous.subject
        {
            view.scope = previous.scope;
        }
        if let Some(refusal) = outcome.refusal {
            view.notice = Some(refusal.explanation());
        }
        let summary = view.summary();
        self.session.workspace_mut().selection = if matches!(view.subject, SelectionSubject::None) {
            SelectionTarget::None
        } else {
            SelectionTarget::Native(Box::new(view))
        };
        self.native_selection.as_mut().unwrap().lineage = None;
        if announce {
            self.log_console_echo(ConsoleFeedbackSource::Selection, summary);
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn native_select_all(&mut self) -> bool {
        match self.ensure_selection_authority() {
            Ok(true) => {}
            Ok(false) => return false,
            Err(e) => {
                self.selection_notice(e.to_string());
                return true;
            }
        }
        let state = self.workspace();
        let schematic =
            state.ui.layout.focused_content() == datum_gui_protocol::PaneContent::Schematic;
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
        true
    }

    pub(super) fn dispatch_session_command(&mut self, command: SessionCommand) -> bool {
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
        match build_selection_view(
            &native.model,
            &native.authority,
            previous.subject.clone(),
            &state.scene,
            state.schematic_scene.as_ref(),
            |layer| {
                state.ui.filters.show_authored
                    && state
                        .ui
                        .filters
                        .layer_visibility
                        .get(layer)
                        .copied()
                        .unwrap_or(true)
            },
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
    }
}
