//! Presented pointer callbacks and shared logical-key input for ordinary App proof.
use super::*;
impl Fixture {
    pub(super) fn key(&mut self, key: Key) {
        assert!(
            self.app
                .runtime
                .as_mut()
                .unwrap()
                .handle_library_input(&key, ElementState::Pressed)
        );
        self.app.request_workspace_redraw();
    }
    pub(super) fn focus_field(&mut self, key: &str) {
        let r = self.app.runtime.as_ref().unwrap();
        let index = r
            .workspace()
            .library
            .draft
            .as_ref()
            .unwrap()
            .fields
            .iter()
            .position(|f| f.key == key)
            .unwrap();
        for _ in 0..40 {
            if self.app.runtime.as_ref().unwrap().workspace().library.focus == Some(C::Field(index))
            {
                return;
            }
            self.key(Key::Named(NamedKey::Tab));
        }
        panic!("Field {key} must be keyboard reachable");
    }
    pub(super) fn control(&mut self, events: &ActiveEventLoop, target: HitTarget) {
        let rect = self
            .app
            .runtime
            .as_ref()
            .unwrap()
            .presented_hits
            .regions()
            .iter()
            .find(|h| h.target == target)
            .unwrap_or_else(|| panic!("stage{} missing presented {target:?}", self.stage))
            .rect;
        assert_eq!(
            self.app
                .runtime
                .as_ref()
                .unwrap()
                .presented_hits
                .hit_test(rect.x + rect.width / 2., rect.y + rect.height / 2.),
            Some(&target),
            "presented pointer target must be unobscured"
        );
        let id = self.app.window.as_ref().unwrap().id();
        self.app.window_event(
            events,
            id,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(
                    (rect.x + rect.width / 2.) as f64,
                    (rect.y + rect.height / 2.) as f64,
                ),
            },
        );
        for state in [ElementState::Pressed, ElementState::Released] {
            self.app.window_event(
                events,
                id,
                WindowEvent::MouseInput {
                    device_id: winit::event::DeviceId::dummy(),
                    state,
                    button: MouseButton::Left,
                },
            );
        }
    }
}
