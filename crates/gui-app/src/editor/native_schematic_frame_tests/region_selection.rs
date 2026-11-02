//! Ordinary pointer routing for schematic regions after an idle Move tool.
//! Called by the existing isolated native workflow before its Symbol Move.
use super::*;
use datum_gui_protocol::{SelectionTarget, SessionCommand};
use datum_gui_viewport::selection_gesture::RegionKind;
use eda_engine::schematic::PlacedSymbol;
use winit::keyboard::ModifiersState;

fn primary(host: &mut Fixture, events: &ActiveEventLoop, state: ElementState) {
    let window = host.app.window.as_ref().unwrap().id();
    host.app.window_event(
        events,
        window,
        WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state,
            button: MouseButton::Left,
        },
    );
}

pub(super) fn exercise(host: &mut Fixture, events: &ActiveEventLoop, symbol: &PlacedSymbol) {
    // Every pin is enclosed: the governed strict-majority oracle is independent
    // of rendering bounds and does not assume a selected Symbol body is a box.
    let mut anchors: Vec<_> = symbol.pins.iter().map(|pin| pin.position).collect();
    anchors.push(symbol.position);
    let margin = 2_000_000;
    let left = anchors.iter().map(|p| p.x).min().unwrap() - margin;
    let right = anchors.iter().map(|p| p.x).max().unwrap() + margin;
    let top = anchors.iter().map(|p| p.y).min().unwrap() - margin;
    let bottom = anchors.iter().map(|p| p.y).max().unwrap() + margin;
    let p = |x, y| PointNm { x, y };
    let paths = [
        (RegionKind::Rectangle, vec![p(left, top), p(right, bottom)]),
        (
            RegionKind::Lasso,
            vec![
                p(right, top),
                p(left, top),
                p(left, bottom),
                p(right, bottom),
                p(right, top),
            ],
        ),
    ];
    let r = host.app.runtime.as_mut().unwrap();
    r.dispatch_session_command(SessionCommand::ClearSelection);
    r.set_workspace_tool(crate::WorkspaceTool::Move);
    assert!(r.native_edit.preview.is_none());
    for (kind, path) in paths {
        for modifiers in [ModifiersState::SHIFT, ModifiersState::CONTROL] {
            host.app.runtime.as_mut().unwrap().modifiers = modifiers;
            host.world(events, path[0], false);
            primary(host, events, ElementState::Pressed);
            assert!(
                host.app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .selection_gesture
                    .is_some()
            );
            for point in &path[1..] {
                host.world(events, *point, false);
            }
            let r = host.app.runtime.as_ref().unwrap();
            assert_eq!(r.selection_gesture.as_ref().unwrap().kind, Some(kind));
            let feedback = r.workspace().ui.selection_region.as_ref().unwrap();
            assert!(!feedback.path.is_empty());
            assert_eq!(feedback.subtractive, modifiers.control_key());
            assert!(
                r.native_edit.preview.is_none(),
                "modified drag must never move"
            );
            primary(host, events, ElementState::Released);
            let r = host.app.runtime.as_mut().unwrap();
            while r.selection_evaluation.is_some() {
                r.selection_tick = Some(Instant::now());
                r.poll_native_selection_tick();
            }
            assert!(r.workspace().ui.selection_region.is_none());
            if modifiers.shift_key() {
                let SelectionTarget::Native(view) = &r.workspace().selection else {
                    panic!("schematic region must acquire its enclosed symbol");
                };
                assert!(view.members.iter().any(|id| id.id == symbol.uuid));
            } else {
                assert!(matches!(r.workspace().selection, SelectionTarget::None));
            }
        }
        // Cancellation before release preserves the entire pre-gesture selection
        // and consumes release instead of turning the gesture into point selection.
        let before = host
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .selection
            .clone();
        host.app.runtime.as_mut().unwrap().modifiers = ModifiersState::SHIFT;
        host.world(events, path[0], false);
        primary(host, events, ElementState::Pressed);
        host.world(events, path[1], false);
        assert!(
            host.app
                .runtime
                .as_mut()
                .unwrap()
                .cancel_native_selection_gesture()
        );
        primary(host, events, ElementState::Released);
        assert_eq!(
            host.app.runtime.as_ref().unwrap().workspace().selection,
            before
        );
    }
    let r = host.app.runtime.as_mut().unwrap();
    r.modifiers = ModifiersState::empty();
    r.set_workspace_tool(crate::WorkspaceTool::Select);
}
