//! Shared pointer routing for native Preferences and New Project surfaces.
use super::*;
use crate::native_frame_adapters::OwnedHost;

impl App {
    /// Route scroll capture before hits; dialog-specific actions retain their
    /// existing mutation and damage policy. Non-pointer events stay with hosts.
    pub(crate) fn handle_owned_dialog_pointer(
        &mut self,
        host: OwnedHost,
        event: &WindowEvent,
    ) -> bool {
        if matches!(host, OwnedHost::New)
            && let WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } = event
        {
            if *state == ElementState::Released {
                // An existing gesture owns its release even if the old pixels
                // no longer describe the current browser state or target.
                let popup = self
                    .runtime
                    .as_mut()
                    .is_some_and(Runtime::finish_folder_popup_dismissal);
                let editor = self
                    .runtime
                    .as_mut()
                    .is_some_and(Runtime::finish_dialog_editor_pointer);
                let scrollbar = self
                    .new_project_surface
                    .as_mut()
                    .is_some_and(|s| s.scrollbar_input(*state, &mut self.frames));
                if popup || editor || scrollbar {
                    if let Some(surface) = &mut self.new_project_surface {
                        surface.browser_pointer_press = None;
                    }
                    self.request_dialog_key_redraw(DialogInputOutcome::Dialog, OwnedHost::New);
                    return true;
                }
            } else {
                let target = self
                    .new_project_surface
                    .as_ref()
                    .and_then(GlobalPreferencesWindowSurface::hit_target);
                if self.runtime.as_ref().is_some_and(|r| {
                    r.folder_browser
                        .state
                        .as_ref()
                        .is_some_and(|s| s.menu.is_some())
                }) && !matches!(target, Some(HitTarget::FolderBrowserMenuItem(_)))
                {
                    self.runtime
                        .as_mut()
                        .unwrap()
                        .folder_popup_pointer(target.as_ref(), true);
                    if let Some(surface) = &mut self.new_project_surface {
                        surface.browser_pointer_press = None;
                    }
                    self.request_dialog_key_redraw(DialogInputOutcome::Dialog, OwnedHost::New);
                    return true;
                }
            }
        }
        let surface = match host {
            OwnedHost::Global => &mut self.global_preferences_surface,
            OwnedHost::Project => &mut self.project_preferences_surface,
            OwnedHost::New => &mut self.new_project_surface,
        };
        if matches!(host, OwnedHost::New)
            && let WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } = event
            && let (Some(surface), Some(runtime)) = (surface.as_mut(), self.runtime.as_ref())
        {
            if *state == ElementState::Pressed {
                surface.browser_pointer_press = None;
            }
            if let Some(target) = surface.hit_target()
                && matches!(
                    target,
                    HitTarget::FolderBrowser(_)
                        | HitTarget::FolderBrowserRow(_)
                        | HitTarget::FolderBrowserMenuItem(_)
                )
                && !surface.admit_browser_pointer(runtime, &target, *state == ElementState::Pressed)
            {
                self.request_dialog_key_redraw(DialogInputOutcome::Dialog, OwnedHost::New);
                return true;
            }
        }
        let target = match event {
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(surface) = surface {
                    surface.set_cursor_position(
                        Some((position.x as f32, position.y as f32)),
                        &mut self.frames,
                    );
                    if matches!(host, OwnedHost::New)
                        && let Some(runtime) = &mut self.runtime
                    {
                        let hover = surface.hit_target();
                        if runtime.drag_dialog_editor_pointer(position.x as f32)
                            || runtime.hover_folder_popup(hover.as_ref())
                        {
                            self.request_dialog_key_redraw(
                                DialogInputOutcome::Dialog,
                                OwnedHost::New,
                            );
                        }
                    }
                }
                return true;
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(surface) = surface {
                    surface.set_cursor_position(None, &mut self.frames);
                }
                return true;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if matches!(host, OwnedHost::New)
                    && self.runtime.as_ref().is_some_and(|r| {
                        r.folder_browser
                            .state
                            .as_ref()
                            .is_some_and(|s| s.menu.is_some())
                    })
                {
                    return true;
                }
                if let Some(surface) = surface {
                    surface.scroll_wheel(*delta, &mut self.frames);
                }
                return true;
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if matches!(host, OwnedHost::New) {
                    let hit = surface
                        .as_ref()
                        .and_then(GlobalPreferencesWindowSurface::hit_target);
                    if let Some(runtime) = &mut self.runtime {
                        if runtime
                            .folder_popup_pointer(hit.as_ref(), *state == ElementState::Pressed)
                        {
                            self.request_dialog_key_redraw(
                                DialogInputOutcome::Dialog,
                                OwnedHost::New,
                            );
                            return true;
                        }
                    }
                }
                if surface
                    .as_mut()
                    .is_some_and(|surface| surface.scrollbar_input(*state, &mut self.frames))
                {
                    return true;
                }
                if *state == ElementState::Pressed {
                    if matches!(host, OwnedHost::New)
                        && let Some((region, point, scale)) = surface
                            .as_ref()
                            .and_then(GlobalPreferencesWindowSurface::pointer_region)
                        && let Some(runtime) = &mut self.runtime
                        && runtime.start_dialog_editor_pointer(region, point.0, scale)
                    {
                        self.request_dialog_key_redraw(DialogInputOutcome::Dialog, OwnedHost::New);
                    }
                    return true;
                }
                surface
                    .as_ref()
                    .and_then(GlobalPreferencesWindowSurface::hit_target)
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } if matches!(host, OwnedHost::New) => {
                if *state == ElementState::Pressed
                    && let (Some(surface), Some(runtime)) =
                        (surface.as_ref(), self.runtime.as_ref())
                    && !surface.browser_hits_current(runtime)
                {
                    self.request_dialog_key_redraw(DialogInputOutcome::Dialog, OwnedHost::New);
                    return true;
                }
                if *state == ElementState::Pressed
                    && let Some((region, point, scale)) = surface
                        .as_ref()
                        .and_then(GlobalPreferencesWindowSurface::pointer_region)
                    && let Some(runtime) = &mut self.runtime
                {
                    let index = match region.target {
                        HitTarget::FolderBrowserRow(index) => Some(index),
                        HitTarget::FolderBrowser(
                            datum_gui_protocol::FolderBrowserFocus::Folders,
                        ) => None,
                        _ => return true,
                    };
                    runtime.open_folder_actions_menu(index, (point.0 / scale, point.1 / scale));
                    self.request_dialog_key_redraw(DialogInputOutcome::Dialog, OwnedHost::New);
                }
                return true;
            }
            _ => return false,
        };
        match host {
            OwnedHost::Global => self.activate_preferences_target(target.as_ref(), false),
            OwnedHost::Project => self.activate_preferences_target(target.as_ref(), true),
            OwnedHost::New => {
                if let (Some(runtime), Some(target)) = (&mut self.runtime, target.as_ref()) {
                    if runtime.activate_folder_browser_hit(target) {
                        self.request_dialog_key_redraw(DialogInputOutcome::Dialog, OwnedHost::New);
                        return true;
                    }
                    if runtime.workspace().ui.project_open.open {
                        let handled = runtime.activate_project_open_hit(target);
                        self.request_dialog_key_redraw(
                            DialogInputOutcome::from_handled(
                                handled,
                                DialogInputOutcome::Dependents,
                            ),
                            OwnedHost::New,
                        );
                        return true;
                    }
                    let changed = match target {
                        HitTarget::NewProjectCancel | HitTarget::NewProjectCreate => {
                            DialogInputOutcome::Dependents
                        }
                        HitTarget::NewProjectModal => DialogInputOutcome::Consumed,
                        _ => DialogInputOutcome::Dialog,
                    };
                    let outcome = DialogInputOutcome::from_handled(
                        runtime.activate_new_project_hit(target),
                        changed,
                    );
                    self.request_dialog_key_redraw(outcome, OwnedHost::New);
                }
            }
        }
        true
    }
}

#[cfg(test)]
#[path = "owned_dialog_pointer_tests.rs"]
mod tests;
