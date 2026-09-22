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
        let surface = match host {
            OwnedHost::Global => &mut self.global_preferences_surface,
            OwnedHost::Project => &mut self.project_preferences_surface,
            OwnedHost::New => &mut self.new_project_surface,
        };
        let target = match event {
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(surface) = surface {
                    surface.set_cursor_position(
                        Some((position.x as f32, position.y as f32)),
                        &mut self.frames,
                    );
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
                if surface
                    .as_mut()
                    .is_some_and(|surface| surface.scrollbar_input(*state, &mut self.frames))
                    || *state != ElementState::Released
                {
                    return true;
                }
                surface
                    .as_ref()
                    .and_then(GlobalPreferencesWindowSurface::hit_target)
            }
            _ => return false,
        };
        match host {
            OwnedHost::Global => self.activate_preferences_target(target.as_ref(), false),
            OwnedHost::Project => self.activate_preferences_target(target.as_ref(), true),
            OwnedHost::New => {
                if let (Some(runtime), Some(target)) = (&mut self.runtime, target.as_ref()) {
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
