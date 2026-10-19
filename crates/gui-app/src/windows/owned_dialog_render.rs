//! Shared owned-dialog frame preparation and presentation.
use super::*;

impl GlobalPreferencesWindowSurface {
    pub(crate) fn browser_hits_current(&self, runtime: &Runtime) -> bool {
        self.presented_browser == runtime.folder_browser_presentation()
    }
    pub(crate) fn admit_browser_pointer(
        &mut self,
        runtime: &Runtime,
        target: &HitTarget,
        pressed: bool,
    ) -> bool {
        if pressed {
            self.browser_pointer_press = self
                .presented_browser
                .clone()
                .filter(|_| self.browser_hits_current(runtime))
                .map(|binding| (target.clone(), binding));
            return self.browser_pointer_press.is_some();
        }
        self.browser_pointer_press
            .take()
            .is_some_and(|(initial, binding)| {
                initial == *target
                    && Some(binding) == runtime.folder_browser_presentation()
                    && self.browser_hits_current(runtime)
            })
    }
    pub(crate) fn presented_browser_rows(&self) -> Vec<usize> {
        self.presented_hits
            .regions()
            .iter()
            .filter_map(|hit| match hit.target {
                HitTarget::FolderBrowserRow(index) => Some(index),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn scroll_browser_preview_key(&mut self, event: &KeyEvent) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }
        let scroll = &mut self.browser_preview;
        let next = match event.logical_key {
            Key::Named(NamedKey::ArrowUp) => scroll.offset() - 20.0,
            Key::Named(NamedKey::ArrowDown) => scroll.offset() + 20.0,
            Key::Named(NamedKey::PageUp) => scroll.offset() - scroll.viewport.height,
            Key::Named(NamedKey::PageDown) => scroll.offset() + scroll.viewport.height,
            Key::Named(NamedKey::Home) => 0.0,
            Key::Named(NamedKey::End) => scroll.maximum(),
            _ => return false,
        };
        if scroll.set_offset(next) {
            self.invalidate();
        }
        true
    }

    pub(crate) fn scroll_project_body_key(&mut self, event: &KeyEvent) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }
        let next = match event.logical_key {
            Key::Named(NamedKey::PageDown) => self.scroll.offset() + self.scroll.viewport.height,
            Key::Named(NamedKey::PageUp) => self.scroll.offset() - self.scroll.viewport.height,
            _ => return false,
        };
        if self.scroll.set_offset(next) {
            self.invalidate();
        }
        true
    }

    pub(crate) fn render(
        &mut self,
        runtime: &Runtime,
        project_preferences: bool,
        new_project: bool,
    ) -> Result<bool> {
        if runtime.device_health.failed() {
            return Ok(false);
        }
        if !self.surface_transaction.begin_frame(
            &self.surface,
            &runtime.device,
            &self.config,
            &runtime.device_health,
        )? {
            return Ok(false);
        }
        self.renderer.prepare_surface_attachment(
            &runtime.device,
            self.config.width,
            self.config.height,
            || !runtime.device_health.failed(),
        )?;
        if self.renderer.render_session().prepared().is_none() {
            let browser_binding = if new_project {
                runtime.folder_browser_presentation()
            } else {
                None
            };
            let view = datum_gui_render::DialogView {
                width: self.config.width,
                height: self.config.height,
                scale: self.scale_factor,
            };
            if new_project {
                let navigation = runtime.folder_browser.state.as_ref();
                let identity = navigation.map(|n| (n.mode, n.path_input.clone(), n.selected));
                let selected_changed = identity != self.browser_view;
                if self.browser_view.as_ref().map(|old| old.0) != identity.as_ref().map(|v| v.0) {
                    self.scroll = Default::default();
                    self.navigation_scroll = Default::default();
                    self.browser_preview = Default::default();
                } else if self.browser_view.as_ref().map(|old| &old.1)
                    != identity.as_ref().map(|v| &v.1)
                {
                    self.navigation_scroll = Default::default();
                }
                self.browser_view = identity;
                let ui = &runtime.workspace().ui;
                let leaf = ui
                    .new_project
                    .open
                    .then(|| ui.new_project.effective_folder_name());
                let preview_identity = (
                    navigation.and_then(|n| n.preview(leaf)),
                    navigation.and_then(|n| n.refusal(leaf)),
                );
                if self.browser_preview_identity.as_ref() != Some(&preview_identity) {
                    self.browser_preview = Default::default();
                }
                self.browser_preview_identity = Some(preview_identity);
                if ui.project_open.open {
                    let reveal =
                        self.project_open_focus != Some(ui.project_open.focus) || selected_changed;
                    self.project_open_focus = Some(ui.project_open.focus);
                    self.new_project_focus = None;
                    self.renderer.prepare_session_dialog(
                        datum_gui_render::DialogInput::OpenProject {
                            dialog: &ui.project_open,
                            navigation,
                            list_scroll: &mut self.navigation_scroll,
                            preview_scroll: &mut self.browser_preview,
                            reveal_focus: reveal,
                        },
                        view,
                        &mut self.scroll,
                    )?;
                } else {
                    let focus = ui.new_project.focus;
                    let reveal = self.new_project_focus != Some(focus) || selected_changed;
                    self.new_project_focus = Some(focus);
                    self.project_open_focus = None;
                    self.renderer.prepare_session_dialog(
                        datum_gui_render::DialogInput::NewProject {
                            dialog: &ui.new_project,
                            navigation,
                            list_scroll: &mut self.navigation_scroll,
                            preview_scroll: &mut self.browser_preview,
                            reveal_focus: reveal,
                        },
                        view,
                        &mut self.scroll,
                    )?;
                }
            } else {
                let dialog = if project_preferences {
                    &runtime.workspace().ui.project_preferences
                } else {
                    &runtime.workspace().ui.global_preferences
                };
                let identity_changed =
                    self.scroll_identity
                        .as_ref()
                        .is_none_or(|(section, query, row)| {
                            section != &dialog.section_id
                                || query != &dialog.search_query
                                || *row != dialog.scroll_row
                        });
                let expanded_changed = self.scroll_expanded.0 != dialog.open_choice_key
                    || self.scroll_expanded.1 != dialog.explanation_key;
                let focus_changed = self.scroll_focus.as_ref() != Some(&dialog.focus);
                let mut reveal_row = identity_changed.then_some(dialog.scroll_row);
                if focus_changed || expanded_changed {
                    use datum_gui_protocol::GlobalPreferencesFocus;
                    let key = match &dialog.focus {
                        GlobalPreferencesFocus::SettingName(key)
                        | GlobalPreferencesFocus::Control(key)
                        | GlobalPreferencesFocus::Reset(key) => Some(key.as_str()),
                        GlobalPreferencesFocus::ExplanationClose => {
                            dialog.explanation_key.as_deref()
                        }
                        _ => None,
                    };
                    if let Some(key) = key {
                        reveal_row = dialog.visible_rows().position(|row| row.key == key);
                    }
                }
                if identity_changed || expanded_changed {
                    self.scroll.release();
                }
                let input = if project_preferences {
                    datum_gui_render::DialogInput::ProjectPreferences { dialog, reveal_row }
                } else {
                    datum_gui_render::DialogInput::GlobalPreferences { dialog, reveal_row }
                };
                self.renderer
                    .prepare_session_dialog(input, view, &mut self.scroll)?;
                // Scrolling rebuilds geometry, but unchanged reveal keys stay owned here.
                if focus_changed {
                    self.scroll_focus = Some(dialog.focus.clone());
                }
                if expanded_changed {
                    self.scroll_expanded.0.clone_from(&dialog.open_choice_key);
                    self.scroll_expanded.1.clone_from(&dialog.explanation_key);
                }
                if identity_changed {
                    self.scroll_identity = Some((
                        dialog.section_id.clone(),
                        dialog.search_query.clone(),
                        dialog.scroll_row,
                    ));
                }
            }
            self.prepared_browser = browser_binding;
            append_gui_verbose_diagnostic_line(|| {
                format!(
                    "native control_meshes window={:?} builds={}",
                    self.window.id(),
                    self.renderer.control_mesh_build_count()
                )
            });
        }
        use gui_runtime_support::native_surface_transaction::NativeRenderTarget;
        let plan = self.surface_transaction.prepare_render_plan(
            &mut self.renderer,
            self.config.width,
            self.config.height,
        )?;
        let mut target = NativeRenderTarget::new(
            &mut self.surface_transaction,
            &self.surface,
            &runtime.device,
            &runtime.queue,
            &self.config,
            &runtime.device_health,
        );
        let rendered = self.renderer.encode_frame(
            plan,
            &runtime.device,
            &runtime.queue,
            &mut target,
            &mut NativeRenderTarget::acquire,
            &mut NativeRenderTarget::submitted,
        );
        let (frame, _) = target.finish(&self.renderer);
        let Some(submitted) = rendered? else {
            return Ok(false);
        };
        let frame = frame.context("rendered owned frame must own an acquisition")?;
        if runtime.device_health.failed() {
            return Ok(false);
        }
        self.surface_transaction.present_rendered(
            frame,
            &self.window,
            &mut self.renderer,
            submitted,
        )?;
        if let Some((hits, _)) = self.renderer.render_session_mut().take_published_frame() {
            self.presented_hits.replace(hits);
            self.presented_browser = self.prepared_browser.clone();
        }
        Ok(true)
    }
}
