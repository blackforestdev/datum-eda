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
    pub(super) fn placement_mouse(
        &mut self,
        events: &ActiveEventLoop,
        point: PointNm,
        click: bool,
    ) {
        let r = self.app.runtime.as_mut().unwrap();
        let layout = r.workspace().ui.layout.clone();
        let p = r.prepared_scene().unwrap();
        let pane = p
            .layout
            .viewport_panes(&layout)
            .panes
            .into_iter()
            .find(|p| p.content == PaneContent::Schematic && p.id == layout.focused)
            .unwrap();
        let (x, y) = (pane.rect.scene.x + 20., pane.rect.scene.y + 20.);
        let a = p
            .world_point_for_pane_unclipped(pane.id, x, y)
            .expect("focused pane projection");
        let b = p
            .world_point_for_pane_unclipped(pane.id, x + 100., y + 100.)
            .expect("focused pane projection");
        let at = (
            (x + (point.x - a.x) as f32 * 100. / (b.x - a.x) as f32) as f64,
            (y + (point.y - a.y) as f32 * 100. / (b.y - a.y) as f32) as f64,
        );
        assert!(
            at.0 >= f64::from(pane.rect.scene.x)
                && at.0 < f64::from(pane.rect.scene.x + pane.rect.scene.width)
                && at.1 >= f64::from(pane.rect.scene.y)
                && at.1 < f64::from(pane.rect.scene.y + pane.rect.scene.height),
            "fixture placement point must be inside the presented schematic pane: {at:?}"
        );
        let mapped = p.world_point_at_screen(at.0 as f32, at.1 as f32);
        assert!(
            matches!(mapped, Some((_, SceneSurface::Schematic))),
            "qualified schematic pointer mapped={mapped:?}; passes={:?}",
            p.surface_passes()
        );
        let id = self.app.window.as_ref().unwrap().id();
        self.app.window_event(
            events,
            id,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(at.0, at.1),
            },
        );
        let runtime = self.app.runtime.as_ref().unwrap();
        let state = runtime.workspace();
        assert!(
            state.schematic_authoring.position.is_some(),
            "presented pointer must produce captured ghost: at={at:?}, refusal={:?}, picker={}, chosen={:?}, active={}, pending={}, named={}, move_preview={}, divider={}, pan={}, captured={:?}",
            state.schematic_authoring.refusal,
            state.schematic_authoring.picker,
            state.schematic_authoring.chosen,
            state.schematic_authoring.active(),
            runtime.native_edit.pending(),
            state.schematic_authoring.named_edit.is_some(),
            runtime
                .native_edit
                .preview
                .as_ref()
                .is_some_and(|p| p.is_schematic()),
            runtime.divider_drag.is_some(),
            runtime.pan_gesture.is_active(),
            state.authoring.gesture.view_basis
        );
        if click {
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
}
