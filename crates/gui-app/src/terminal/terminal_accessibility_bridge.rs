//! Linux accessibility-provider state for the active native terminal.
//!
//! TerminalCore supplies the immutable text model. This bridge computes exact
//! AT-SPI event intent without retaining PTY bytes or inventing a second grid.

use crate::Runtime;
use crate::console_accessibility::AccessibilityAnnouncement;
use crate::terminal_accessibility::{TerminalAccessibilityBounds, TerminalAccessibilitySnapshot};
use crate::terminal_accessibility_platform::PlatformBridge;
use datum_gui_protocol::ApplicationFocus;
use datum_gui_protocol::GlobalPreferencesAccessibleNode;
use datum_gui_protocol::gui_menu_model::accessibility::{
    MenuAccessibleNode, menu_accessibility_nodes,
};
use std::{collections::VecDeque, sync::Arc};

const ANNOUNCEMENT_LOG_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TerminalAccessibilityEvent {
    TextChanged,
    CaretMoved,
    SelectionChanged,
    FocusChanged,
    TitleChanged,
    BoundsChanged,
    Bell,
}

pub(crate) struct LinuxTerminalAccessibilityBridge {
    current: Option<Arc<TerminalAccessibilitySnapshot>>,
    preferences: Vec<GlobalPreferencesAccessibleNode>,
    menus: Vec<MenuAccessibleNode>,
    selection: Option<crate::selection_accessibility::SelectionAccessibilitySnapshot>,
    platform: Option<PlatformBridge>,
    publish_platform: bool,
    announcement_log: VecDeque<AccessibilityAnnouncement>,
}

impl Default for LinuxTerminalAccessibilityBridge {
    fn default() -> Self {
        Self {
            current: None,
            preferences: Vec::new(),
            menus: Vec::new(),
            selection: None,
            platform: None,
            publish_platform: true,
            announcement_log: VecDeque::new(),
        }
    }
}

impl LinuxTerminalAccessibilityBridge {
    pub(crate) fn update_selection(
        &mut self,
        view: Option<&datum_gui_protocol::selection_view::SelectionView>,
        pane: Option<datum_gui_protocol::PaneId>,
        evaluating: Option<(datum_gui_protocol::PaneId, usize)>,
        visual_lod: Option<datum_gui_render::SelectionVisualLod>,
    ) {
        // Only the successful preparation owner supplies a new visual summary.
        // Identical snapshots keep their prior report without noisy no-op events;
        // changed subjects/revisions wait for current preparation.
        let visual_lod = visual_lod.or_else(|| {
            self.selection
                .as_ref()
                .filter(|previous| view.is_some_and(|view| previous.view.as_ref() == view))
                .and_then(|previous| previous.visual_lod)
        });
        let focused = pane.is_some();
        if match (&self.selection, view) {
            (Some(previous), Some(view)) => {
                previous.pane == pane
                    && previous.evaluating == evaluating
                    && previous.visual_lod == visual_lod
                    && previous.view.as_ref() == view
            }
            (None, None) => true,
            _ => false,
        } {
            return;
        }
        let shared_view = view.map(|view| {
            self.selection
                .as_ref()
                .filter(|previous| previous.view.as_ref() == view)
                .map(|previous| previous.view.clone())
                .unwrap_or_else(|| Arc::new(view.clone()))
        });
        self.selection = shared_view.map(|view| {
            crate::selection_accessibility::SelectionAccessibilitySnapshot {
                view,
                evaluating,
                visual_lod,
                focused,
                pane,
            }
        });
        if !self.publish_platform {
            return;
        }
        match &mut self.platform {
            Some(platform) => platform.publish_selection(self.selection.clone()),
            None => self.platform = PlatformBridge::start_selection(self.selection.clone()).ok(),
        }
    }

    pub(crate) fn update_menus(&mut self, nodes: Vec<MenuAccessibleNode>) {
        if self.menus == nodes {
            return;
        }
        self.menus = nodes.clone();
        if !self.publish_platform {
            return;
        }
        match &mut self.platform {
            Some(platform) => platform.publish_menus(nodes),
            None => self.platform = PlatformBridge::start_menus(nodes).ok(),
        }
    }

    pub(crate) fn announce_console(&mut self, announcement: AccessibilityAnnouncement) {
        if self.announcement_log.len() == ANNOUNCEMENT_LOG_CAPACITY {
            self.announcement_log.pop_front();
        }
        self.announcement_log.push_back(announcement.clone());
        if !self.publish_platform {
            return;
        }
        match &mut self.platform {
            Some(platform) => platform.publish_announcement(announcement),
            None => {
                self.platform = PlatformBridge::start_announcement(announcement).ok();
            }
        }
    }

    pub(crate) fn update_preferences(&mut self, nodes: Vec<GlobalPreferencesAccessibleNode>) {
        if self.preferences == nodes {
            return;
        }
        self.preferences = nodes.clone();
        if !self.publish_platform {
            return;
        }
        match &mut self.platform {
            Some(platform) => platform.publish_preferences(nodes),
            None => {
                self.platform = PlatformBridge::start_preferences(nodes).ok();
            }
        }
    }

    pub(crate) fn update(
        &mut self,
        next: TerminalAccessibilitySnapshot,
    ) -> Vec<TerminalAccessibilityEvent> {
        let mut events = Vec::new();
        if let Some(current) = &self.current {
            if current.text != next.text {
                events.push(TerminalAccessibilityEvent::TextChanged);
            }
            if current.caret != next.caret {
                events.push(TerminalAccessibilityEvent::CaretMoved);
            }
            if current.selection != next.selection {
                events.push(TerminalAccessibilityEvent::SelectionChanged);
            }
            if current.focused != next.focused {
                events.push(TerminalAccessibilityEvent::FocusChanged);
            }
            if current.title != next.title {
                events.push(TerminalAccessibilityEvent::TitleChanged);
            }
            if current.bounds != next.bounds {
                events.push(TerminalAccessibilityEvent::BoundsChanged);
            }
            if current.bell_count != next.bell_count {
                events.push(TerminalAccessibilityEvent::Bell);
            }
        } else {
            events.extend([
                TerminalAccessibilityEvent::TextChanged,
                TerminalAccessibilityEvent::CaretMoved,
                TerminalAccessibilityEvent::FocusChanged,
                TerminalAccessibilityEvent::BoundsChanged,
            ]);
        }
        let next = Arc::new(next);
        self.current = Some(Arc::clone(&next));
        if self.publish_platform {
            match &mut self.platform {
                Some(platform) => platform.publish(next, events.clone()),
                None => {
                    self.platform = PlatformBridge::start(next, events.clone()).ok();
                }
            }
        }
        events
    }

    #[cfg(test)]
    pub(crate) fn current(&self) -> Option<&TerminalAccessibilitySnapshot> {
        self.current.as_deref()
    }

    #[cfg(test)]
    fn without_platform() -> Self {
        Self {
            current: None,
            preferences: Vec::new(),
            menus: Vec::new(),
            selection: None,
            platform: None,
            publish_platform: false,
            announcement_log: VecDeque::new(),
        }
    }
}

impl Runtime {
    pub(super) fn refresh_menu_accessibility(&mut self) {
        // Zoom invalidates the prepared frame, but a closed menu has no nodes.
        // Avoid touching its inventory at all in this high-frequency path;
        // publishing the empty projection still clears nodes when a menu closes.
        let nodes = if self.workspace().ui.active_menu.is_none()
            && self
                .workspace()
                .ui
                .marking_menu
                .as_ref()
                .is_none_or(|m| m.selection.is_none())
        {
            Vec::new()
        } else {
            datum_gui_protocol::gui_menu_model::default_gui_menu_model()
                .map(|model| menu_accessibility_nodes(model, self.workspace()))
                .unwrap_or_default()
        };
        self.terminal_accessibility.update_menus(nodes);
    }

    pub(super) fn refresh_global_preferences_accessibility(&mut self) {
        let nodes = if self.workspace().ui.project_open.open {
            self.workspace().ui.project_open.accessibility_nodes()
        } else if self.workspace().ui.new_project.open {
            self.workspace().ui.new_project.accessibility_nodes()
        } else if self.workspace().ui.project_preferences.open {
            self.workspace()
                .ui
                .project_preferences
                .accessibility_nodes()
        } else {
            self.workspace().ui.global_preferences.accessibility_nodes()
        };
        self.terminal_accessibility.update_preferences(nodes);
    }

    pub(super) fn refresh_terminal_accessibility(&mut self) {
        if !self.terminal_sessions.active_attached() {
            return;
        }
        let geometry = self.terminal_screen_geometry();
        let scroll_offset = self.workspace().ui.terminal.scroll_offset;
        let focused = self.application_focus() == ApplicationFocus::Terminal;
        match self.terminal_sessions.active_accessibility_snapshot(
            usize::from(geometry.rows),
            scroll_offset,
            focused,
        ) {
            Ok(mut snapshot) => {
                snapshot.bounds = TerminalAccessibilityBounds {
                    x: geometry.screen.x.round() as i32,
                    y: geometry.screen.y.round() as i32,
                    width: geometry.screen.width.round() as i32,
                    height: geometry.screen.height.round() as i32,
                };
                self.terminal_accessibility.update(snapshot);
            }
            Err(error) => {
                self.log_terminal_event(format!("terminal accessibility refresh failed: {error}"));
            }
        }
    }
}

#[cfg(test)]
#[path = "terminal_accessibility_bridge/region_tests.rs"]
mod region_tests;

#[cfg(test)]
#[path = "terminal_accessibility_bridge/notice_tests.rs"]
mod notice_tests;

#[cfg(test)]
#[path = "terminal_accessibility_bridge/context_tests.rs"]
mod context_tests;

#[cfg(test)]
#[path = "terminal_accessibility_bridge/compound_move_tests.rs"]
mod compound_move_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires X11/Vulkan and pinned CLI; run serially"]
    #[allow(deprecated)] // Hidden integration host; production uses ActiveEventLoop.
    fn native_selection_publication_refreshes_announced_and_unannounced_complete_state() {
        use clap::Parser;
        use datum_gui_protocol::{
            AuthoredSelectionClass as Class, AuthoredSelectionIdentity, SelectionSubject,
            SelectionTarget,
        };
        use winit::platform::x11::EventLoopBuilderExtX11;

        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/golden/text/native/text-transform-repro");
        let root = std::env::temp_dir().join(format!(
            "selection-publication-{}-{}",
            std::process::id(),
            eda_engine::substrate::ObjectId::new_v4()
        ));
        let paths = [
            "project.json",
            "board/board.json",
            "schematic/schematic.json",
            "rules/rules.json",
        ];
        for rel in paths {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::copy(fixture.join(rel), path).unwrap();
        }
        let board: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
        let mut expected = std::collections::BTreeSet::from([AuthoredSelectionIdentity {
            class: Class::BoardOutline,
            id: serde_json::from_value(board["uuid"].clone()).unwrap(),
            instance_path: vec![],
        }]);
        for text in board["texts"].as_array().unwrap() {
            expected.insert(AuthoredSelectionIdentity {
                class: Class::BoardText,
                id: serde_json::from_value(text["uuid"].clone()).unwrap(),
                instance_path: vec![],
            });
        }
        assert_eq!(expected.len(), 9);
        let event_loop = winit::event_loop::EventLoop::<()>::with_user_event()
            .with_x11()
            .with_any_thread(true)
            .build()
            .unwrap();
        let wake = event_loop.create_proxy();
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_visible(false)
                        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
                )
                .unwrap(),
        );
        let args = crate::GuiArgs::try_parse_from([
            "datum-gui",
            "--project-root",
            root.to_str().unwrap(),
            "--focus-pane",
            "board",
        ])
        .unwrap();
        let launch = args.load_launch_state(Some(wake.clone())).unwrap();
        let mut runtime =
            pollster::block_on(crate::Runtime::new(window, launch, Some(1.0), wake)).unwrap();
        runtime.terminal_accessibility = LinuxTerminalAccessibilityBridge::without_platform();
        let canonical: Vec<_> = paths
            .iter()
            .map(|rel| std::fs::read(root.join(rel)).unwrap())
            .collect();
        let journal_path = eda_engine::substrate::transaction_journal_path(&root);
        let journal = std::fs::read(&journal_path).ok();
        let head_path = root.join(".datum/revision/v1/head.json");
        let head = std::fs::read(&head_path).ok();
        let mut previous = None;
        for announced in [true, false] {
            let _ = runtime.prepared_scene();
            assert!(runtime.renderer.render_session().prepared().is_some());
            let revision = runtime.renderer.render_session().content_revision();
            let console = runtime.workspace().ui.console.records().count();
            let announcements = runtime.terminal_accessibility.announcement_log.len();
            assert!(runtime.native_select_all());
            let SelectionTarget::Native(view) = &runtime.workspace().selection else {
                panic!("complete native selection required");
            };
            assert_eq!(view.members, expected);
            let SelectionSubject::Compound(compound) = &view.subject else {
                panic!("ordinary global acquisition must produce a Compound");
            };
            assert_eq!(compound.members, expected);
            assert_eq!(compound.focus, None);
            assert_eq!(view.focus(), None);
            assert!(!view.stale);
            assert_eq!(
                runtime
                    .terminal_accessibility
                    .selection
                    .as_ref()
                    .unwrap()
                    .view
                    .as_ref(),
                view.as_ref()
            );
            if let Some(previous) = &previous {
                assert_eq!(view.as_ref(), previous);
            }
            previous = Some(view.as_ref().clone());
            assert_eq!(
                runtime.workspace().ui.console.records().count(),
                console + usize::from(announced)
            );
            assert_eq!(
                runtime.terminal_accessibility.announcement_log.len(),
                announcements + usize::from(announced)
            );
            assert!(runtime.renderer.render_session().prepared().is_none());
            assert!(runtime.renderer.render_session().has_pending_frame());
            assert!(runtime.renderer.render_session().content_revision() > revision);
            for (rel, bytes) in paths.iter().zip(&canonical) {
                assert_eq!(&std::fs::read(root.join(rel)).unwrap(), bytes);
            }
            assert_eq!(std::fs::read(&journal_path).ok(), journal);
            assert_eq!(std::fs::read(&head_path).ok(), head);
        }
        drop(runtime);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn snapshot(text: &str, caret: usize, focused: bool) -> TerminalAccessibilitySnapshot {
        TerminalAccessibilitySnapshot {
            session_id: "session-a".into(),
            title: "Terminal".into(),
            text: text.into(),
            caret,
            selection: None,
            links: Vec::new(),
            focused,
            bell_count: 0,
            bounds: Default::default(),
        }
    }

    #[test]
    fn bridge_emits_only_changed_terminal_semantics() {
        let mut bridge = LinuxTerminalAccessibilityBridge::without_platform();
        assert_eq!(bridge.update(snapshot("a", 1, true)).len(), 4);
        assert!(bridge.update(snapshot("a", 1, true)).is_empty());
        assert_eq!(
            bridge.update(snapshot("ab", 2, true)),
            vec![
                TerminalAccessibilityEvent::TextChanged,
                TerminalAccessibilityEvent::CaretMoved,
            ]
        );
        assert_eq!(bridge.current().unwrap().text.chars().count(), 2);
    }

    #[test]
    fn console_announces_without_a_terminal_snapshot() {
        let mut bridge = LinuxTerminalAccessibilityBridge::without_platform();
        assert!(bridge.current().is_none());
        bridge.announce_console(AccessibilityAnnouncement {
            text: "Action. Information: fit board".to_string(),
            priority: crate::console_accessibility::AnnouncementPriority::Medium,
        });
        assert_eq!(bridge.announcement_log.len(), 1);
        assert!(bridge.current().is_none());
    }

    #[test]
    fn unchanged_or_initially_empty_preferences_do_not_start_platform_publication() {
        let mut bridge = LinuxTerminalAccessibilityBridge::without_platform();
        bridge.update_preferences(Vec::new());
        assert!(bridge.preferences.is_empty());

        let nodes = vec![GlobalPreferencesAccessibleNode {
            id: "global-preferences".to_owned(),
            name: "Global Preferences".to_owned(),
            role: datum_gui_protocol::GlobalPreferencesAccessibleRole::Dialog,
            value: None,
            description: "Global settings".to_owned(),
            available: true,
            focused: true,
        }];
        bridge.update_preferences(nodes.clone());
        bridge.update_preferences(nodes.clone());
        assert_eq!(bridge.preferences, nodes);
    }
}

#[cfg(test)]
#[path = "terminal_accessibility_bridge/marking_release_tests.rs"]
mod marking_release_tests;
