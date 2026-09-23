//! History metadata is charged alongside unique shared geometry.
use super::*;

#[test]
fn history_keys_scene_metadata_and_entry_capacity_match_allocator() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let scene = RetainedScene::from_workspace(&state, 960, 720);
    let geometry = scene.geometry_observer().heap_bytes_excluding([]);
    let scope = datum_gui_render::cpu_alloc::Scope::new("history-owned-heap-proof");
    let mut history = RetainedSceneHistory::default();
    scope.with(|| {
        for n in 0..6 {
            history.insert(super::tests::key(n), scene.clone());
        }
    });
    let usage = scope.usage();
    assert_eq!(history.entries.len(), 6);
    assert_eq!(
        history.accounted_bytes() - geometry,
        (usage.payload_bytes + usage.tracking_bytes) as usize,
        "shared geometry is charged once, cloned commands/hits/keys and entry capacity remain distinct"
    );
    drop(history);
    let usage = scope.usage();
    assert_eq!(usage.payload_bytes + usage.tracking_bytes, 0);
}
