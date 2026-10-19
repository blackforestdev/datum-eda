//! Shared typed chrome action routing, extracted without changing entry ownership.
use super::*;
impl Runtime {
    pub(super) fn select_hit_target_inner(&mut self, target: &HitTarget) -> bool {
        if self.activate_folder_browser_hit(target) {
            return true;
        }
        if matches!(
            target,
            HitTarget::ProjectOpenModal
                | HitTarget::ProjectOpenCancel
                | HitTarget::ProjectOpenConfirm
        ) {
            return self.activate_project_open_hit(target);
        }
        if matches!(
            target,
            HitTarget::NewProjectModal
                | HitTarget::NewProjectFolderName
                | HitTarget::NewProjectUseName
                | HitTarget::NewProjectName
                | HitTarget::NewProjectRetryGlobal
                | HitTarget::NewProjectCancel
                | HitTarget::NewProjectCreate
        ) {
            return self.activate_new_project_hit(target);
        }
        if let Some(handled) = self.apply_revision_hit(target) {
            return handled;
        }
        if let Some(handled) = self.activate_application_overlay_hit_target(target) {
            return handled;
        }
        if let Some(handled) = self.activate_selection_inspector_target(target) {
            return handled;
        }
        match target {
            HitTarget::ProjectOpenModal
            | HitTarget::ProjectOpenCancel
            | HitTarget::ProjectOpenConfirm => unreachable!("Open targets return above"),
            HitTarget::Library(control) => self.activate_library(control.clone()),
            HitTarget::SchematicPicker
            | HitTarget::SchematicLibraryPage(_)
            | HitTarget::SchematicLibraryChoice(_)
            | HitTarget::SchematicReference
            | HitTarget::SchematicValue
            | HitTarget::SchematicPlace
            | HitTarget::SchematicCancel
            | HitTarget::SchematicFinishWire => self.handle_schematic_chrome(target),
            HitTarget::NewProjectModal
            | HitTarget::NewProjectFolderName
            | HitTarget::NewProjectUseName
            | HitTarget::NewProjectName
            | HitTarget::NewProjectRetryGlobal
            | HitTarget::NewProjectCancel
            | HitTarget::NewProjectCreate => unreachable!("New Project targets return above"),
            HitTarget::CloseRevisionSurface
            | HitTarget::ToggleRevisionIssuanceArm
            | HitTarget::OpenRevisionWitness(_) => unreachable!("revision targets return above"),
            HitTarget::ProposalAction(action_id) => self
                .acquire_native_artifact(datum_gui_protocol::SelectionSubject::Proposal(
                    action_id.clone(),
                ))
                .unwrap_or_else(|| {
                    self.dispatch_session_command(SessionCommand::SelectReviewAction(
                        action_id.clone(),
                    ))
                }),
            HitTarget::ReviewAction(action_id) => {
                let handled = self.dispatch_session_command(SessionCommand::SelectReviewAction(
                    action_id.clone(),
                ));
                if handled {
                    self.log_console_echo_for_action(
                        ConsoleFeedbackSource::Selection,
                        action_id,
                        "Review action selected",
                    );
                }
                handled
            }
            HitTarget::SelectionScope(scope) => self.set_selection_scope(*scope),
            HitTarget::SelectionInventoryToggle
            | HitTarget::SelectionFiltersToggle
            | HitTarget::SelectionClassToggle(_)
            | HitTarget::SelectionInspectorScroll { .. } => {
                unreachable!("selection Inspector targets return above")
            }
            HitTarget::AuthoredObject(object_id) => {
                if let Some(handled) = self.acquire_native_key(object_id, SceneSurface::Board, None)
                {
                    return handled;
                }
                let handled = self.dispatch_session_command(SessionCommand::SelectAuthoredObject(
                    object_id.clone(),
                ));
                if handled {
                    self.session.workspace_mut().ui.hovered_object = None;
                    self.log_console_echo_for_target(
                        ConsoleFeedbackSource::Selection,
                        object_id,
                        "Object selected",
                    );
                }
                handled
            }
            HitTarget::CheckFinding(fingerprint) => {
                if let Some(handled) = self.acquire_native_artifact(
                    datum_gui_protocol::SelectionSubject::Diagnostic(fingerprint.clone()),
                ) {
                    return handled;
                }
                let handled = self.dispatch_session_command(SessionCommand::SelectCheckFinding(
                    fingerprint.clone(),
                ));
                if handled {
                    let target = self
                        .session
                        .workspace()
                        .checks
                        .findings
                        .iter()
                        .find(|finding| finding.fingerprint == *fingerprint)
                        .and_then(|finding| {
                            datum_gui_protocol::check_finding_scene_target_object_id(
                                &self.session.workspace().scene,
                                finding,
                            )
                        });
                    self.session.workspace_mut().ui.hovered_object =
                        target.clone().map(|object_id| HoverTarget {
                            object_id,
                            surface: PaneContent::Board,
                        });
                    if let Some(target) = target {
                        self.fit_scene_object(&target);
                    }
                }
                handled
            }
            HitTarget::FitBoard => {
                self.fit_camera();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "fit board");
                true
            }
            HitTarget::FitReviewTarget => {
                let handled = self.fit_review_target();
                if handled {
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        "fit active review target",
                    );
                }
                handled
            }
            HitTarget::SetWorkspaceTool(tool) => self.set_workspace_tool(*tool),
            HitTarget::ReviewPrev => {
                let handled =
                    self.dispatch_session_command(SessionCommand::SelectPreviousReviewAction);
                if handled {
                    self.log_console_echo(
                        ConsoleFeedbackSource::Selection,
                        "selected previous review action",
                    );
                }
                handled
            }
            HitTarget::ReviewNext => {
                let handled = self.dispatch_session_command(SessionCommand::SelectNextReviewAction);
                if handled {
                    self.log_console_echo(
                        ConsoleFeedbackSource::Selection,
                        "selected next review action",
                    );
                }
                handled
            }
            HitTarget::ToggleShowAuthored => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowAuthored);
                if handled {
                    let state = if self.workspace().ui.filters.show_authored {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        format!("authored visibility {state}"),
                    );
                }
                handled
            }
            HitTarget::ToggleShowProposed => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowProposed);
                if handled {
                    let state = if self.workspace().ui.filters.show_proposed {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        format!("proposal visibility {state}"),
                    );
                }
                handled
            }
            HitTarget::ToggleShowUnrouted => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowUnrouted);
                if handled {
                    let state = if self.workspace().ui.filters.show_unrouted {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        format!("unrouted visibility {state}"),
                    );
                }
                handled
            }
            HitTarget::ToggleDimUnrelated => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleDimUnrelated);
                if handled {
                    let state = if self.workspace().ui.filters.dim_unrelated {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_console_echo(
                        ConsoleFeedbackSource::Viewport,
                        format!("dim unrelated {state}"),
                    );
                }
                handled
            }
            HitTarget::ToggleLayer(layer_id) => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleLayerVisibility(
                    layer_id.clone(),
                ));
                if handled {
                    self.session.workspace_mut().ui.filters.active_layer_id =
                        Some(layer_id.clone());
                    let visible = self
                        .workspace()
                        .ui
                        .filters
                        .layer_visibility
                        .get(layer_id)
                        .copied()
                        .unwrap_or(true);
                    let state = if visible { "visible" } else { "hidden" };
                    self.log_console_echo_for_target(
                        ConsoleFeedbackSource::Viewport,
                        layer_id,
                        format!("Layer visibility {state}"),
                    );
                    self.invalidate_scene();
                }
                handled
            }
            HitTarget::ToggleSelectedBoardTextMirrored => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::Mirrored)
            }
            HitTarget::ToggleSelectedBoardTextKeepUpright => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::KeepUpright)
            }
            HitTarget::ToggleSelectedBoardTextBold => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::Bold)
            }
            HitTarget::CycleSelectedBoardTextRenderIntent => {
                self.cycle_selected_board_text_field(BoardTextCycleField::RenderIntent)
            }
            HitTarget::CycleSelectedBoardTextFamily => {
                self.cycle_selected_board_text_field(BoardTextCycleField::Family)
            }
            HitTarget::CycleSelectedBoardTextHAlign => {
                self.cycle_selected_board_text_alignment(BoardTextAlignmentField::Horizontal)
            }
            HitTarget::CycleSelectedBoardTextVAlign => {
                self.cycle_selected_board_text_alignment(BoardTextAlignmentField::Vertical)
            }
            HitTarget::DecreaseSelectedBoardTextHeight => {
                self.step_selected_board_text_height(BoardTextHeightStep::Decrease)
            }
            HitTarget::IncreaseSelectedBoardTextHeight => {
                self.step_selected_board_text_height(BoardTextHeightStep::Increase)
            }
            HitTarget::RotateSelectedBoardTextCounterClockwise90 => {
                self.step_selected_board_text_rotation(BoardTextRotationStep::CounterClockwise90)
            }
            HitTarget::RotateSelectedBoardTextClockwise90 => {
                self.step_selected_board_text_rotation(BoardTextRotationStep::Clockwise90)
            }
            HitTarget::DecreaseSelectedBoardTextLineSpacing => {
                self.step_selected_board_text_line_spacing(BoardTextLineSpacingStep::Decrease)
            }
            HitTarget::IncreaseSelectedBoardTextLineSpacing => {
                self.step_selected_board_text_line_spacing(BoardTextLineSpacingStep::Increase)
            }
            HitTarget::EditSelectedBoardTextContent => {
                self.begin_selected_board_text_content_edit()
            }
            HitTarget::EditSelectedBoardTextHeight => self.begin_selected_board_text_height_edit(),
            HitTarget::EditSelectedBoardTextRotation => {
                self.begin_selected_board_text_rotation_edit()
            }
            HitTarget::EditSelectedBoardTextLineSpacing => {
                self.begin_selected_board_text_line_spacing_edit()
            }
            HitTarget::EditSelectedBoardTextRenderIntent => {
                self.begin_selected_board_text_render_intent_edit()
            }
            HitTarget::EditSelectedBoardTextFamily => self.begin_selected_board_text_family_edit(),
            HitTarget::EditSelectedBoardTextAlignment => {
                self.begin_selected_board_text_alignment_edit()
            }
            HitTarget::TerminalTab => {
                // Owner decision 2026-08-14 (bead
                // dat-pan-trace-terminal-pollution-0j0): tab-click is
                // deliberate terminal entry, so the click stays handled even
                // when the dock is already showing the terminal —
                // `select_hit_target` then arms focus via
                // `hit_target_is_terminal_entry`.
                self.set_active_dock(DockTab::Terminal);
                true
            }
            HitTarget::TerminalSessionTab(session_id) => self.activate_terminal_session(session_id),
            HitTarget::TerminalPaneScreen(session_id) => self.activate_terminal_session(session_id),
            HitTarget::TerminalSessionClose(session_id) => self.close_terminal_session(session_id),
            HitTarget::TerminalSessionNew => self.spawn_terminal_session_tab(),
            target @ (HitTarget::TerminalSessionTerminateActive
            | HitTarget::TerminalSessionForceKillActive
            | HitTarget::TerminalSessionRetryTermination
            | HitTarget::TerminalShutdownCancel) => self.handle_terminal_lifecycle_target(target),
            HitTarget::TerminalScreen => self.click_terminal_screen(),
            HitTarget::TerminalClipboardCopy => {
                self.dismiss_terminal_clipboard_menu();
                self.copy_terminal_scrollback();
                true
            }
            HitTarget::TerminalClipboardPaste => {
                self.dismiss_terminal_clipboard_menu();
                self.paste_terminal_input();
                true
            }
            HitTarget::TerminalThemeNext => {
                self.dismiss_terminal_clipboard_menu();
                self.cycle_terminal_theme()
            }
            HitTarget::TerminalProfileNext => {
                self.dismiss_terminal_clipboard_menu();
                self.cycle_terminal_profile()
            }
            HitTarget::TerminalLinkCopy => {
                let target = self.terminal_clipboard_link_target();
                self.dismiss_terminal_clipboard_menu();
                if let Some(target) = target {
                    self.copy_terminal_link_target(&target);
                }
                true
            }
            HitTarget::TerminalLinkOpen => {
                let target = self.terminal_clipboard_link_target();
                self.dismiss_terminal_clipboard_menu();
                if let Some(target) = target {
                    self.arm_terminal_link_target(target);
                }
                true
            }
            HitTarget::TerminalLinkConfirmOpen => self.confirm_terminal_link_open(),
            HitTarget::TerminalLinkCancel => {
                self.cancel_terminal_link_confirmation();
                true
            }
            HitTarget::TerminalClipboardConfirmWrite => self.confirm_terminal_clipboard_write(),
            HitTarget::TerminalClipboardCancelWrite => self.cancel_terminal_clipboard_write(),
            HitTarget::ProductionArtifact(artifact_id) => {
                let handled = self.dispatch_session_command(
                    SessionCommand::FocusProductionArtifact(artifact_id.clone()),
                );
                if handled {
                    self.log_console_echo_for_target(
                        ConsoleFeedbackSource::Production,
                        artifact_id,
                        "Production artifact selected",
                    );
                }
                handled
            }
            HitTarget::ProductionArtifactFile(path) => {
                let handled = self.dispatch_session_command(
                    SessionCommand::FocusProductionArtifactFile(path.clone()),
                );
                if handled {
                    self.log_console_echo_for_target(
                        ConsoleFeedbackSource::Production,
                        path,
                        "Production artifact file selected",
                    );
                }
                handled
            }
            HitTarget::ProductionOutputJobRun(handoff) => {
                self.set_active_dock(DockTab::Terminal);
                let command = prepare_terminal_command_execution(
                    self.terminal_sessions.active(),
                    "production_output_job_run",
                    handoff,
                )
                .unwrap_or_else(|err| {
                    self.log_terminal_event(format!("terminal handoff prepare failed: {err}"));
                    handoff.command.clone()
                });
                let mut bytes = command.into_bytes();
                bytes.push(b'\r');
                if self.write_foreign_shell_bytes(&bytes) {
                    self.log_console_echo_for_action(
                        ConsoleFeedbackSource::Production,
                        &handoff.command,
                        "Production output command sent to Terminal",
                    );
                } else {
                    self.log_console_critical_refusal_for_action(
                        ConsoleFeedbackSource::Production,
                        &handoff.command,
                        "Production output command could not be sent to Terminal",
                    );
                }
                true
            }
            HitTarget::ProductionTerminalCommand(handoff) => {
                self.set_active_dock(DockTab::Terminal);
                let command = prepare_terminal_command_execution(
                    self.terminal_sessions.active(),
                    "production_terminal_command",
                    handoff,
                )
                .unwrap_or_else(|err| {
                    self.log_terminal_event(format!("terminal handoff prepare failed: {err}"));
                    handoff.command.clone()
                });
                let mut bytes = command.into_bytes();
                bytes.push(b'\r');
                if self.write_foreign_shell_bytes(&bytes) {
                    self.log_console_echo_for_action(
                        ConsoleFeedbackSource::Production,
                        &handoff.command,
                        "Production command sent to Terminal",
                    );
                } else {
                    self.log_console_critical_refusal_for_action(
                        ConsoleFeedbackSource::Production,
                        &handoff.command,
                        "Production command could not be sent to Terminal",
                    );
                }
                true
            }
            HitTarget::ArtifactPreviewZoomIn
            | HitTarget::ArtifactPreviewZoomOut
            | HitTarget::ArtifactPreviewReset
            | HitTarget::ToggleArtifactPreviewGeometry
            | HitTarget::ToggleArtifactPreviewDrills => self
                .select_artifact_preview_hit_target(target)
                .unwrap_or(false),
            HitTarget::ArtifactPreviewViewport => false,
            HitTarget::ConsoleHistoryToggle => self.toggle_console_history(),
            HitTarget::ConsoleHistoryFilter(filter) => {
                self.session
                    .workspace_mut()
                    .ui
                    .console
                    .set_history_filter(*filter);
                self.invalidate_frame();
                true
            }
            HitTarget::FolderBrowser(_)
            | HitTarget::FolderBrowserMenuItem(_)
            | HitTarget::FolderBrowserRow(_)
            | HitTarget::MenuTitle(_)
            | HitTarget::MenuItem { .. }
            | HitTarget::GlobalPreferencesModal
            | HitTarget::GlobalPreferencesSection(_)
            | HitTarget::GlobalPreferencesSearch
            | HitTarget::GlobalPreferencesSettingName(_)
            | HitTarget::GlobalPreferencesControl(_)
            | HitTarget::GlobalPreferencesChoice { .. }
            | HitTarget::GlobalPreferencesReset(_)
            | HitTarget::GlobalPreferencesExplanationClose
            | HitTarget::MarkingMenuItem { .. } => {
                unreachable!("application overlay targets return above")
            }
            // Divider gestures are handled directly by mouse press/release.
            HitTarget::MarkingMenuSurface { .. }
            | HitTarget::DockResizeHandle
            | HitTarget::TerminalSplitDivider(_)
            | HitTarget::LayerScrollRegion => false,
        }
    }
}
