use super::*;

#[test]
fn grid_cache_reuses_exact_dependencies_and_refuses_partial_replacement() {
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
    let prepared = PreparedScene::from_workspace_for_surface(
        &state,
        1600,
        1000,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    let passes = prepared.surface_passes();
    let budget = Budget::new(16 * 1024 * 1024);
    let mut cache = GridCache::default();
    assert!(cache.prepare(passes, &budget).unwrap());
    let (expected, batches) = build_surface_grids(&prepared);
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&expected),
        bytemuck::cast_slice::<_, u8>(&cache.vertices)
    );
    assert_eq!(batches.len(), cache.batches.len());
    for (a, b) in batches.iter().zip(cache.batches.iter()) {
        assert_eq!(
            (a.pane_id, a.viewport, a.vertices.clone()),
            (b.pane_id, b.viewport, b.vertices.clone())
        );
    }
    let ptr = cache.vertices.as_ptr();
    let bytes = cache.allocated_bytes();
    assert_eq!(budget.used(), bytes);
    assert!(!cache.prepare(passes, &budget).unwrap());
    assert_eq!(ptr, cache.vertices.as_ptr());
    assert_eq!(budget.used(), bytes);
    for change in 0..7 {
        let mut changed = passes.to_vec();
        match change {
            0 => changed[0].camera.center_x_nm += 500_000.0,
            1 => changed[0].scene_viewport.width += 1.0,
            2 => changed[0].bounds.min_x -= 1,
            3 => changed[0].pane_id = datum_gui_protocol::PaneId(999),
            4 => changed[0].surface = SceneSurface::Schematic,
            5 => {
                changed[0].grid_lod_resolved.tier = if changed[0].grid_lod_resolved.tier == Some(2)
                {
                    Some(0)
                } else {
                    Some(2)
                };
            }
            _ => changed.reverse(),
        }
        if changed == passes {
            continue;
        }
        assert!(
            cache.prepare(&changed, &budget).unwrap(),
            "dependency {change}"
        );
        assert!(!cache.prepare(&changed, &budget).unwrap());
        cache.prepare(passes, &budget).unwrap();
    }
    let mut changed = passes.to_vec();
    changed[0].camera.center_x_nm += 500_000.0;
    assert_ne!(changed, passes);
    let before = cache.vertices.as_ptr();
    let used = budget.used();
    let keys_bytes = StagingVec::<PreparedSurfacePass>::capacity_bytes(passes.len()).unwrap();
    // The second limit admits keys, then refuses vertices: rollback must release
    // the partial replacement without altering the published old entry.
    for limit in [0, keys_bytes] {
        let refused_budget = Budget::new(limit);
        assert!(cache.prepare(&changed, &refused_budget).is_err());
        assert_eq!(refused_budget.used(), 0);
    }
    assert_eq!(before, cache.vertices.as_ptr());
    assert_eq!(used, budget.used());
    assert!(!cache.prepare(passes, &budget).unwrap());
    assert!(cache.prepare(&[], &budget).unwrap());
    assert_eq!(budget.used(), 0);
    assert_eq!(cache.allocated_bytes(), 0);
}
