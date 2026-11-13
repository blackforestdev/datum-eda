use super::*;
use crate::{LiveReviewRequest, WorkspaceBacking};
use eda_engine::substrate::{ProjectResolver, SourceShardKind};
use std::{path::PathBuf, sync::Arc};
struct Fixture {
    root: PathBuf,
    state: ReviewWorkspaceState,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("datum-board-controls-{}", Uuid::new_v4()));
        eda_engine::api::native_write::genesis::bootstrap_native_project(
            &root,
            eda_engine::api::native_write::genesis::GenesisSpec {
                project_name: "Native PCB controls fixture".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        if let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
            eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
        {
            let prepared =
                eda_engine::api::native_write::project::build_initialize_project_display_units(
                    &model,
                    eda_engine::api::native_write::WriteProvenance::new(
                        "board-controls-fixture",
                        eda_engine::substrate::CommitSource::Test,
                        "Explicit fixture working units",
                    ),
                    profile,
                    &receipt,
                )
                .unwrap();
            eda_engine::api::native_write::commit_prepared(&mut model, &root, prepared).unwrap();
        }
        for (id, name) in [
            ("00000000-0000-0000-0000-00000000c200", "GND"),
            ("00000000-0000-0000-0000-00000000c201", "SIGNAL"),
        ] {
            let net = eda_engine::board::Net::new(id.parse().unwrap(), name, Uuid::nil());
            let prepared = eda_engine::api::native_write::board_routing::build_place_board_net(
                &model,
                eda_engine::api::native_write::WriteProvenance::new(
                    "board-controls-fixture",
                    eda_engine::substrate::CommitSource::Test,
                    "Explicit native fixture Net",
                ),
                &net,
            )
            .unwrap();
            eda_engine::api::native_write::commit_prepared(&mut model, &root, prepared).unwrap();
        }
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let board = model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap();
        let mut state = crate::load_fixture_workspace_state();
        state.ui.layout = crate::WorkspaceLayout::single();
        state.scene.project_uuid = model.project.project_id.to_string();
        state.scene.source_revision = model.model_revision.0.clone();
        state.scene.board_uuid = board["uuid"].as_str().unwrap().into();
        state.scene.layers = crate::scene_layers_from_native_stackup_value(&board).unwrap();
        state.ui.filters.active_layer_id = Some("L1".into());
        state.library = crate::library_browser::LibraryBrowser::capture(Arc::new(model));
        state.backing = Some(WorkspaceBacking {
            request: LiveReviewRequest {
                project_root: root.clone(),
                board_file: None,
                artifact_path: None,
                net_uuid: None,
                from_anchor_pad_uuid: None,
                to_anchor_pad_uuid: None,
                profile: None,
                kicad_board_source: None,
            },
            board_path: root.join("board/board.json"),
            native_write_verbs: ["datum.pcb.draw_track".into(), "datum.pcb.place_zone".into()]
                .into_iter()
                .collect(),
        });
        Self { root, state }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
#[test]
fn board_form_requires_explicit_intent_and_uses_exact_project_units() {
    let f = Fixture::new();
    let mut form = BoardAuthoringControls::capture(&f.state, BoardToolKind::Zone).unwrap();
    assert!(form.fields.iter().all(|v| v.text.is_empty()));
    assert_eq!(form.thermal, None);
    assert!(form.zone_settings().is_err());
    form.field_mut(BoardField::Priority).text = "7".into();
    form.thermal = Some(false);
    assert!(
        form.zone_settings().is_err(),
        "dimensions are never silently defaulted"
    );
    form.field_mut(BoardField::ThermalGap).text = "0.125 mm".into();
    form.field_mut(BoardField::SpokeWidth).text = "10 mil".into();
    let settings = form.zone_settings().unwrap();
    assert_eq!(
        (
            settings.priority,
            settings.thermal_gap_nm,
            settings.thermal_spoke_width_nm
        ),
        (7, 125000, 254000)
    );
    form.field_mut(BoardField::ThermalGap).text = "-1 nm".into();
    assert!(form.zone_settings().is_err());
    form.field_mut(BoardField::RulePriority).text = u64::MAX.to_string();
    assert_eq!(form.rule_priority().unwrap(), u64::MAX);
    form.field_mut(BoardField::RulePriority).text = "18446744073709551616".into();
    assert!(form.rule_priority().is_err());
}
#[test]
fn pcb_configuration_rejects_changed_view_layer_and_cancel_retires_it() {
    let mut f = Fixture::new();
    let form = BoardAuthoringControls::capture(&f.state, BoardToolKind::Route).unwrap();
    assert!(form.current(&f.state));
    f.state.ui.filters.active_layer_id = Some("L2".into());
    assert!(!form.current(&f.state));
    f.state.ui.filters.active_layer_id = Some("L1".into());
    f.state.scene.source_revision.push_str("changed");
    assert!(!form.current(&f.state));
    f.state.authoring.board_controls = Some(form);
    assert!(f.state.cancel_authoring_gesture());
    assert!(f.state.authoring.board_controls.is_none());
}
#[test]
fn net_setup_uses_existing_identity_and_refuses_unknown_net_without_writes() {
    let f = Fixture::new();
    let model = f.state.library.basis.as_ref().unwrap();
    let before = std::fs::read(f.root.join("rules/rules.json")).unwrap();
    let net = "00000000-0000-0000-0000-00000000c200".parse().unwrap();
    let request = crate::native_edit_client::board_rule::BoardClearanceRule::capture(
        model,
        net,
        200000,
        u64::MAX,
    )
    .unwrap();
    assert_eq!(
        request.value().unwrap()["priority"].as_u64(),
        Some(u64::MAX)
    );
    assert!(
        crate::native_edit_client::board_rule::BoardClearanceRule::capture(
            model,
            Uuid::new_v4(),
            200000,
            0
        )
        .is_err()
    );
    assert!(
        crate::native_edit_client::board_rule::BoardClearanceRule::capture(model, net, 0, 0)
            .is_err()
    );
    assert_eq!(
        std::fs::read(f.root.join("rules/rules.json")).unwrap(),
        before
    );
}

#[test]
fn zone_finish_refuses_changed_displayed_net_and_preserves_boundary_and_disk() {
    let mut f = Fixture::new();
    f.state.set_tool(crate::WorkspaceTool::DrawBoardZone);
    let mut setup = BoardAuthoringControls::capture(&f.state, BoardToolKind::Zone).unwrap();
    setup.field_mut(BoardField::Net).text = "00000000-0000-0000-0000-00000000c200".into();
    setup.field_mut(BoardField::Priority).text = "0".into();
    setup.field_mut(BoardField::ThermalGap).text = "0 nm".into();
    setup.field_mut(BoardField::SpokeWidth).text = "0 nm".into();
    setup.thermal = Some(false);
    let net = setup.zone_net(&f.state).unwrap();
    f.state.authoring.board_controls = Some(setup);
    f.state.begin_board_zone(net, 1).unwrap();
    for (x, y) in [(0, 0), (1000000, 0), (1000000, 1000000), (0, 1000000)] {
        f.state
            .append_board_zone_vertex(crate::PointNm { x, y })
            .unwrap();
    }
    let old = f.state.authoring.board_zone.clone();
    let bytes = std::fs::read(f.root.join("board/board.json")).unwrap();
    f.state
        .authoring
        .board_controls
        .as_mut()
        .unwrap()
        .field_mut(BoardField::Net)
        .text = "00000000-0000-0000-0000-00000000c201".into();
    assert!(
        f.state
            .prepare_configured_board_zone()
            .unwrap_err()
            .contains("changing its Net")
    );
    assert_eq!(f.state.authoring.board_zone, old);
    assert_eq!(
        std::fs::read(f.root.join("board/board.json")).unwrap(),
        bytes
    );
    f.state
        .authoring
        .board_controls
        .as_mut()
        .unwrap()
        .field_mut(BoardField::Net)
        .text = net.to_string();
    assert!(f.state.prepare_configured_board_zone().is_ok());
    assert_eq!(
        std::fs::read(f.root.join("board/board.json")).unwrap(),
        bytes
    );
}

#[test]
fn board_form_input_ownership_stays_with_its_original_leaf() {
    let mut f = Fixture::new();
    let setup = BoardAuthoringControls::capture(&f.state, BoardToolKind::Route).unwrap();
    assert!(setup.owns_pane(&f.state));
    let other = f.state.ui.layout.split_focused_with(
        PaneContent::Board,
        crate::SplitOrientation::Vertical,
        true,
    );
    assert_ne!(other, setup.basis.pane);
    assert!(!setup.owns_pane(&f.state));
    f.state.ui.layout.focused = setup.basis.pane;
    assert!(setup.owns_pane(&f.state));
    f.state.inspector_context.owner = crate::inspector_context::InspectorOwner::Navigator;
    assert!(!setup.owns_pane(&f.state));
}

#[test]
fn zone_cached_projection_tracks_net_edits_and_invalidated_source_without_defaults() {
    let mut f = Fixture::new();
    let mut setup = BoardAuthoringControls::capture(&f.state, BoardToolKind::Zone).unwrap();
    setup.field_mut(BoardField::Net).text = "00000000-0000-0000-0000-00000000c200".into();
    setup.refresh_zone_projection(&f.state);
    assert_eq!(
        setup.resolved_zone_net.as_ref().unwrap().to_string(),
        "00000000-0000-0000-0000-00000000c200"
    );
    setup.field_mut(BoardField::Net).text = "not a saved Net".into();
    setup.refresh_zone_projection(&f.state);
    assert!(setup.resolved_zone_net.is_err());
    assert!(setup.zone_clearance.is_err());
    assert!(!setup.needs_clearance_rule);
    setup.field_mut(BoardField::Net).text = "00000000-0000-0000-0000-00000000c201".into();
    setup.refresh_zone_projection(&f.state);
    assert_eq!(
        setup.resolved_zone_net.as_ref().unwrap().to_string(),
        "00000000-0000-0000-0000-00000000c201"
    );
    f.state.scene.source_revision.push_str("changed");
    setup.refresh_zone_projection(&f.state);
    assert!(setup.resolved_zone_net.is_err());
    assert!(setup.zone_clearance_label().is_err());
}

#[test]
fn outline_uses_project_units_without_a_copper_layer_and_retains_invalid_draft() {
    let mut f = Fixture::new();
    f.state
        .backing
        .as_mut()
        .unwrap()
        .native_write_verbs
        .insert("datum.pcb.set_outline".into());
    f.state.ui.filters.active_layer_id = None;
    let form = BoardAuthoringControls::capture(&f.state, BoardToolKind::Outline).unwrap();
    assert!(form.fields.iter().all(|v| v.text.is_empty()));
    f.state.begin_board_outline().unwrap();
    f.state.authoring.board_controls = Some(form);
    assert!(
        f.state
            .authoring
            .board_controls
            .as_ref()
            .unwrap()
            .current(&f.state)
    );
    let before = std::fs::read(f.root.join("board/board.json")).unwrap();
    let tip =
        eda_engine::revision::transaction_tip(&f.state.library.basis.as_ref().unwrap().journal);
    for (x, y) in [(0, 0), (2000000, 2000000), (0, 2000000), (2000000, 0)] {
        f.state
            .append_board_outline_vertex(crate::PointNm { x, y })
            .unwrap();
    }
    assert!(f.state.prepare_board_outline().is_err());
    assert_eq!(
        f.state
            .authoring
            .board_outline
            .as_ref()
            .unwrap()
            .vertices()
            .len(),
        4
    );
    assert!(!f.state.authoring.board_outline.as_ref().unwrap().pending());
    assert_eq!(
        std::fs::read(f.root.join("board/board.json")).unwrap(),
        before
    );
    assert_eq!(
        eda_engine::revision::transaction_tip(&f.state.library.basis.as_ref().unwrap().journal),
        tip
    );
    f.state.remove_board_outline_vertex().unwrap();
    assert!(
        f.state
            .authoring
            .board_outline
            .as_ref()
            .unwrap()
            .geometry_error()
            .is_none()
    );
    // A copper-layer switch does not stale a board-wide Outline draft.
    f.state.ui.filters.active_layer_id = Some("L999".into());
    assert!(
        f.state
            .authoring
            .board_controls
            .as_ref()
            .unwrap()
            .current(&f.state)
    );
    let intent = f.state.prepare_board_outline().unwrap();
    assert!(intent.validate_outline_payload().is_ok());
    assert!(f.state.authoring.board_outline.as_ref().unwrap().pending());
    assert!(
        f.state
            .append_board_outline_vertex(crate::PointNm { x: 3000000, y: 0 })
            .is_err()
    );
    assert!(f.state.remove_board_outline_vertex().is_err());
    let mut bad = intent.clone();
    bad.expected_revision.push_str("stale");
    assert!(bad.validate_outline_payload().is_err());
    bad = intent.clone();
    bad.action = crate::native_edit_client::NativeEditAction::BoardZone;
    assert!(bad.validate_outline_payload().is_err());
    f.state
        .authoring
        .board_outline
        .as_mut()
        .unwrap()
        .release_pending();
    assert!(f.state.cancel_authoring_gesture());
    assert!(f.state.authoring.board_outline.is_none());
    assert_eq!(
        std::fs::read(f.root.join("board/board.json")).unwrap(),
        before
    );
}

#[test]
fn outline_source_change_retains_vertices_and_refuses_finish_without_writing() {
    let mut f = Fixture::new();
    f.state
        .backing
        .as_mut()
        .unwrap()
        .native_write_verbs
        .insert("datum.pcb.set_outline".into());
    f.state.begin_board_outline().unwrap();
    for (x, y) in [(0, 0), (2000000, 0), (0, 2000000)] {
        f.state
            .append_board_outline_vertex(crate::PointNm { x, y })
            .unwrap();
    }
    let before = std::fs::read(f.root.join("board/board.json")).unwrap();
    f.state.scene.source_revision.push_str("changed");
    assert!(
        !f.state
            .authoring
            .board_outline
            .as_ref()
            .unwrap()
            .is_current(&f.state)
    );
    assert!(f.state.prepare_board_outline().is_err());
    assert_eq!(
        f.state
            .authoring
            .board_outline
            .as_ref()
            .unwrap()
            .vertices()
            .len(),
        3
    );
    assert_eq!(
        std::fs::read(f.root.join("board/board.json")).unwrap(),
        before
    );
    f.state.cancel_authoring_gesture();
    assert!(f.state.authoring.board_outline.is_none());
}

#[test]
fn outline_range_refusal_is_distinct_and_cache_tracks_backspace_and_new_vertices() {
    let mut f = Fixture::new();
    f.state
        .backing
        .as_mut()
        .unwrap()
        .native_write_verbs
        .insert("datum.pcb.set_outline".into());
    f.state.authoring.snap.enabled = false;
    f.state.begin_board_outline().unwrap();
    let before = std::fs::read(f.root.join("board/board.json")).unwrap();
    for (x, y) in [
        (i64::MIN, i64::MIN),
        (i64::MAX, i64::MIN),
        (i64::MAX, i64::MAX),
    ] {
        f.state
            .append_board_outline_vertex(crate::PointNm { x, y })
            .unwrap();
    }
    let draft = f.state.authoring.board_outline.as_ref().unwrap();
    assert_eq!(
        eda_engine::board::occupied_copper::validate_polygon(&draft.polygon()),
        Err(eda_engine::board::nominal_geometry::GeometryError::ArithmeticRange)
    );
    let reason = draft.geometry_error().unwrap();
    assert!(reason.contains("ArithmeticRange"));
    assert!(!reason.contains("three vertices"));
    // Finish independently refuses the same unavailable arithmetic without a write.
    assert!(
        f.state
            .prepare_board_outline()
            .unwrap_err()
            .contains("ArithmeticRange")
    );
    assert!(!f.state.authoring.board_outline.as_ref().unwrap().pending());
    f.state.remove_board_outline_vertex().unwrap();
    assert!(
        f.state
            .authoring
            .board_outline
            .as_ref()
            .unwrap()
            .geometry_error()
            .unwrap()
            .contains("simple")
    );
    f.state.remove_board_outline_vertex().unwrap();
    f.state.remove_board_outline_vertex().unwrap();
    for (x, y) in [(0, 0), (1000000, 0), (0, 1000000)] {
        f.state
            .append_board_outline_vertex(crate::PointNm { x, y })
            .unwrap();
    }
    assert!(
        f.state
            .authoring
            .board_outline
            .as_ref()
            .unwrap()
            .geometry_error()
            .is_none()
    );
    assert_eq!(
        std::fs::read(f.root.join("board/board.json")).unwrap(),
        before
    );
    f.state.cancel_authoring_gesture();
}
