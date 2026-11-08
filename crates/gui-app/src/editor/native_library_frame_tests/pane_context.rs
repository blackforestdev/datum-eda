//! Read-only pane/source context through ordinary navigation and cold reopening.
use super::*;
use datum_gui_protocol::{PaneContent, PaneId, inspector_context::InspectorContext};

pub(super) enum Action {
    Navigate(bool),
    Expand,
    Focus(PaneContent, bool),
    Fit,
    Check(usize),
}

impl Fixture {
    pub(super) fn pane_context_action(&mut self, events: &ActiveEventLoop, action: Action) {
        match action {
            Action::Navigate(child) | Action::Focus(PaneContent::Schematic, child) => {
                let r = self.app.runtime.as_ref().unwrap();
                let key = r
                    .workspace()
                    .schematic_documents
                    .rows
                    .iter()
                    .find(|row| row.parent.is_some() == child)
                    .unwrap()
                    .key
                    .clone();
                if matches!(action, Action::Navigate(_)) {
                    let target = r
                        .presented_hits
                        .regions()
                        .iter()
                        .find_map(|hit| match &hit.target {
                            HitTarget::SchematicDocumentRow { document, .. }
                                if document == &key =>
                            {
                                Some(hit.target.clone())
                            }
                            _ => None,
                        })
                        .expect("qualified Navigator row must be presented");
                    self.control(events, target.clone());
                    self.control(events, target);
                } else {
                    let pane = *r
                        .workspace()
                        .schematic_documents
                        .panes
                        .iter()
                        .find(|(_, document)| **document == key)
                        .unwrap()
                        .0;
                    self.context_focus(events, pane);
                }
            }
            Action::Focus(content, _) => {
                let layout = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .ui
                    .layout
                    .clone();
                let pane = layout
                    .leaves()
                    .into_iter()
                    .find(|id| layout.content_for(*id) == Some(content))
                    .unwrap();
                self.context_focus(events, pane);
            }
            Action::Expand => {
                let r = self.app.runtime.as_ref().unwrap();
                let root = r
                    .workspace()
                    .schematic_documents
                    .rows
                    .iter()
                    .find(|row| row.parent.is_none())
                    .unwrap();
                if !r
                    .workspace()
                    .schematic_documents
                    .expanded
                    .contains(&root.key)
                {
                    let target = r
                        .presented_hits
                        .regions()
                        .iter()
                        .find_map(|hit| match &hit.target {
                            HitTarget::SchematicDocumentExpand { document, .. }
                                if document == &root.key =>
                            {
                                Some(hit.target.clone())
                            }
                            _ => None,
                        })
                        .unwrap();
                    self.control(events, target);
                }
            }
            Action::Fit => {
                assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "View",
                    "Fit to Board",
                    EntrySurface::MenuPointer
                ));
                self.app.request_workspace_redraw();
            }
            Action::Check(count) => self.context_check(count),
        }
    }

    fn context_focus(&mut self, events: &ActiveEventLoop, pane: PaneId) {
        let r = self.app.runtime.as_mut().unwrap();
        let layout = r.workspace().ui.layout.clone();
        let panes = r.prepared_scene().unwrap().layout.viewport_panes(&layout);
        let header = panes
            .panes
            .iter()
            .find(|leaf| leaf.id == pane)
            .unwrap()
            .rect
            .header;
        let (x, y) = (header.x + 15., header.y + header.height / 2.);
        assert_eq!(panes.leaf_at(x, y), Some(pane));
        let id = self.app.window.as_ref().unwrap().id();
        self.app.window_event(
            events,
            id,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(x as f64, y as f64),
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
        assert_eq!(
            self.app
                .runtime
                .as_ref()
                .unwrap()
                .workspace()
                .ui
                .layout
                .focused,
            pane
        );
    }

    fn context_check(&mut self, count: usize) {
        let r = self.app.runtime.as_mut().unwrap();
        let state = r.workspace();
        let layout = state.ui.layout.clone();
        let bindings = state.schematic_documents.panes.clone();
        if let Some(key) = bindings.get(&layout.focused)
            && let Some(row) = state.schematic_documents.row(key)
            && let Some(parent) = &row.parent
        {
            let hit = r
                .presented_hits
                .regions()
                .iter()
                .find(|hit| {
                    matches!(&hit.target,
                HitTarget::SchematicBreadcrumb {pane,from,document,..}
                if *pane==layout.focused && from==key && document==parent)
                })
                .expect("actual nested child presents its qualified ancestor breadcrumb");
            assert!(hit.rect.width > 0.);
        }
        let inspector = InspectorContext::capture(state);
        assert_eq!(inspector.document, bindings.get(&layout.focused));
        let prepared = r.prepared_scene().unwrap();
        let panes = prepared.layout.viewport_panes(&layout);
        assert_eq!(panes.panes.len(), count);
        assert_eq!(prepared.surface_passes().len(), count);
        let mut observed = Vec::new();
        for leaf in &panes.panes {
            let pass = prepared
                .surface_passes()
                .iter()
                .find(|pass| pass.pane_id == leaf.id)
                .unwrap();
            let surface = if leaf.content == PaneContent::Board {
                datum_gui_render::SceneSurface::Board
            } else {
                datum_gui_render::SceneSurface::Schematic
            };
            assert_eq!(pass.surface, surface);
            assert_eq!(pass.schematic_document.as_ref(), bindings.get(&leaf.id));
            assert_eq!(pass.scene_viewport, leaf.rect.scene);
            let (x, y) = (
                leaf.rect.scene.x + leaf.rect.scene.width / 2.,
                leaf.rect.scene.y + leaf.rect.scene.height / 2.,
            );
            assert_eq!(panes.leaf_at(x, y), Some(leaf.id));
            let point = prepared
                .world_point_for_pane_unclipped(leaf.id, x, y)
                .unwrap();
            assert_eq!(prepared.world_point_at_screen(x, y), Some((point, surface)));
            observed.push(serde_json::json!({"pane":leaf.id.0,"content":format!("{:?}",leaf.content),"document":pass.schematic_document,"frame":[leaf.rect.frame.x,leaf.rect.frame.width],"focused":leaf.id==layout.focused}));
        }
        std::fs::write(
            self.output
                .join(format!("pane-context-{}.json", self.stage)),
            serde_json::to_vec_pretty(&observed).unwrap(),
        )
        .unwrap();
    }
}

#[test]
#[ignore = "requires saved ordinary Root/Child project and isolated native Wayland"]
fn ordinary_board_root_child_context_focus_fit_cold_reopen_without_writes() {
    use Action::*;
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut steps = vec![Step::PaneContext(Check(2)), Step::Snapshot];
    steps.extend([
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Check(2)),
        Step::Invariant,
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::PaneContext(Check(3)),
        Step::Invariant,
    ]);
    for (content, child, label) in [
        (PaneContent::Schematic, true, "context-child"),
        (PaneContent::Board, false, "context-board"),
        (PaneContent::Schematic, false, "context-root"),
    ] {
        steps.extend([
            Step::PaneContext(Focus(content, child)),
            Step::PaneContext(Fit),
            Step::PaneContext(Check(3)),
            Step::Capture(label),
            Step::Invariant,
        ]);
    }
    steps.extend([
        Step::Reopen,
        Step::PaneContext(Check(2)),
        Step::Invariant,
        Step::PaneContext(Navigate(false)),
        Step::PaneContext(Expand),
        Step::PaneContext(Navigate(true)),
        Step::PaneContext(Fit),
        Step::PaneContext(Check(3)),
        Step::Capture("context-cold-child"),
        Step::Invariant,
    ]);
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
