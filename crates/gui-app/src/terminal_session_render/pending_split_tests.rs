use super::*;
use datum_gui_protocol::{TerminalSplitDirection, TerminalSplitNode};
use std::time::{Duration, Instant};

#[test]
fn pending_split_preserves_source_render_then_publishes_owned_leaf() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/open-project-pending-split-tests")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let source = registry.active().session_id().to_owned();
    let mut lane = TerminalLaneState::default();
    registry
        .active_core_mut()
        .apply_output(&mut lane, b"source remains visible")
        .unwrap();
    let pending = registry
        .begin_split_and_activate(&context, &mut lane, TerminalSplitDirection::SideBySide)
        .unwrap();
    assert!(registry.active_pending_id.is_none());
    assert!(registry.terminal_tabs[0].root.contains_session(&pending));
    let panes = registry.take_active_tab_render_states(&lane).unwrap();
    assert_eq!(panes.len(), 1);
    assert_eq!(panes[0].session_id, source);
    let text = panes[0]
        .snapshot
        .rows()
        .flat_map(|row| row.cells())
        .filter_map(|cell| match &cell.content {
            datum_terminal_core::CellContent::Cluster(cluster) => Some(cluster.text()),
            _ => None,
        })
        .collect::<String>();
    assert!(text.contains("source remains visible"));
    drop(panes);

    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    while !registry.pending_spawns.is_empty() && Instant::now() < deadline {
        registry.complete_pending_spawns(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(registry.pending_spawns.is_empty());
    let destination = registry.active().session_id().to_owned();
    assert_ne!(destination, source);
    assert!(!registry.terminal_tabs[0].root.contains_session(&pending));
    let panes = registry.take_active_tab_render_states(&lane).unwrap();
    assert_eq!(panes.len(), 2);
    assert!(panes.iter().any(|pane| pane.session_id == source));
    assert!(
        panes
            .iter()
            .any(|pane| pane.session_id == destination && pane.focused)
    );
    drop(panes);

    let committed = registry.terminal_tabs[0].root.clone();
    registry.terminal_tabs[0].root = TerminalSplitNode::Split {
        direction: TerminalSplitDirection::SideBySide,
        ratio_millis: 500,
        first: Box::new(TerminalSplitNode::session(&destination)),
        second: Box::new(TerminalSplitNode::session("not-reserved-or-owned")),
    };
    let refusal = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        drop(registry.take_active_tab_render_states(&lane).unwrap());
    }));
    assert!(refusal.is_err(), "unknown split leaf was silently omitted");
    registry.terminal_tabs[0].root = committed;

    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    registry.terminate_all_by(deadline);
    while !registry.all_sessions_closed() && Instant::now() < deadline {
        registry.drain_all(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(registry.all_sessions_closed());
}
