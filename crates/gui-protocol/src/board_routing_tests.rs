use super::*;
use crate::{LiveReviewRequest, PaneId, WorkspaceBacking};
use eda_engine::api::native_write::schematic_wire_edit::{WireCornerOrder, WireRouting};
use eda_engine::substrate::ProjectResolver;
use std::sync::Arc;

struct Fixture {
    root: PathBuf,
    state: ReviewWorkspaceState,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("datum-board-draft-{}", Uuid::new_v4()));
        crate::known_good_demo::write_known_good_demo_project(&root).unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let board = model
            .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)
            .unwrap();
        let mut state = crate::load_fixture_workspace_state();
        state.project_unopened = false;
        state.ui.layout = crate::WorkspaceLayout::single();
        state.scene.project_uuid = model.project.project_id.to_string();
        state.scene.source_revision = model.model_revision.0.clone();
        state.scene.board_uuid = board["uuid"].as_str().unwrap().into();
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
            native_write_verbs: ["datum.pcb.draw_track".into()].into_iter().collect(),
        });
        Self { root, state }
    }
    fn begin(&mut self) {
        self.state
            .begin_board_route(
                "00000000-0000-0000-0000-00000000c205".parse().unwrap(),
                1,
                WireRouteConfig::default(),
            )
            .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn board_route_shared_preview_anchors_posture_and_removes_actual_corner() {
    let mut fixture = Fixture::new();
    fixture.begin();
    let state = &mut fixture.state;
    let start = state.authoring.gesture.anchor.unwrap();
    assert_eq!(
        start,
        PointNm {
            x: 6_975_000,
            y: 4_000_000
        }
    );
    assert_ne!(
        state.snap_authoring_point(start),
        start,
        "source contact must bypass grid rounding"
    );
    assert!(
        state
            .finish_draw_board_track_handoff(PointNm {
                x: start.x + 1_000_000,
                y: start.y,
            })
            .is_none(),
        "the legacy terminal handoff cannot consume a native route draft"
    );
    assert_eq!(state.authoring.gesture.anchor, Some(start));
    let draft = state.authoring.board_route.as_mut().unwrap();
    draft
        .set_posture(WireRouteConfig {
            routing: WireRouting::Orthogonal,
            corner: WireCornerOrder::Start,
        })
        .unwrap();
    let next = PointNm {
        x: start.x + 1_000_000,
        y: start.y + 2_000_000,
    };
    let preview = draft.preview_points(Some(next)).unwrap();
    draft.append_vertex(next).unwrap();
    assert_eq!(draft.vertices(), preview);
    assert_eq!(preview.len(), 3);
    draft
        .set_posture(WireRouteConfig {
            routing: WireRouting::Diagonal45,
            corner: WireCornerOrder::End,
        })
        .unwrap();
    let future = draft
        .preview_points(Some(PointNm {
            x: next.x + 2_000_000,
            y: next.y + 1_000_000,
        }))
        .unwrap();
    assert_eq!(&future[..3], preview);
    assert!(draft.remove_last_vertex());
    assert_eq!(draft.vertices(), &preview[..2]);
    assert!(draft.remove_last_vertex());
    assert!(
        !draft.remove_last_vertex(),
        "Backspace cannot remove the source terminal"
    );
    assert!(state.cancel_authoring_gesture());
    assert!(state.authoring.board_route.is_none());
    assert!(!state.authoring.gesture.is_active());
}

#[test]
fn board_route_guard_retires_focus_content_root_revision_and_tip_changes() {
    let mut fixture = Fixture::new();
    fixture.begin();
    let baseline = fixture.state.clone();
    assert!(!fixture.state.retire_stale_board_route());
    fixture.state.ui.layout.toggle_zoom();
    assert!(
        !fixture.state.retire_stale_board_route(),
        "camera/layout maximize preserves basis"
    );
    for mutation in 0..6 {
        let mut state = baseline.clone();
        match mutation {
            0 => state.ui.layout.focused = PaneId(99),
            1 => state.ui.layout.set_focused_content(PaneContent::Schematic),
            2 => state.backing.as_mut().unwrap().request.project_root = fixture.root.join("other"),
            3 => state.scene.source_revision.push_str("-changed"),
            4 => state.scene.project_uuid = Uuid::new_v4().to_string(),
            _ => state.authoring.board_route.as_mut().unwrap().basis.tip = Some(Uuid::new_v4()),
        }
        assert!(state.retire_stale_board_route(), "mutation {mutation}");
        assert!(state.authoring.board_route.is_none());
        assert!(!state.authoring.gesture.is_active());
    }
    let mut state = baseline;
    state.set_tool(WorkspaceTool::Move);
    assert!(state.authoring.board_route.is_none());
}

#[test]
fn board_route_preparation_is_read_only_and_freezes_one_owned_request() {
    let mut fixture = Fixture::new();
    let before = std::fs::read(fixture.root.join("board/board.json")).unwrap();
    fixture.begin();
    let mut draft = fixture.state.authoring.board_route.take().unwrap();
    assert_eq!(
        draft.width_nm(),
        Some(200_000),
        "only the actual native class supplies this suggestion"
    );
    assert!(draft.set_width_nm(0).is_err());
    assert!(draft.set_width_nm(100_000).is_err());
    let to = PointNm {
        x: draft.basis.start.x + 10_000,
        y: draft.basis.start.y,
    };
    let intent = draft
        .prepare_intent(&fixture.state, Some(to), None)
        .unwrap();
    let request = intent.board_route.unwrap();
    assert_eq!(
        request.points,
        vec![draft.basis.start, Point::new(to.x, to.y)]
    );
    assert_eq!(request.edit_seed, draft.edit_seed);
    assert!(draft.append_vertex(to).is_err());
    assert!(draft.set_width_nm(300_000).is_err());
    assert!(!draft.remove_last_vertex());
    assert!(
        draft
            .prepare_intent(&fixture.state, Some(to), None)
            .is_err()
    );
    assert_eq!(
        std::fs::read(fixture.root.join("board/board.json")).unwrap(),
        before
    );
    assert!(!eda_engine::substrate::transaction_journal_path(&fixture.root).exists());
}

#[test]
fn board_route_refuses_non_native_and_wrong_source_without_draft() {
    let fixture = Fixture::new();
    for mutation in 0..4 {
        let mut state = fixture.state.clone();
        match mutation {
            0 => state.backing = None,
            1 => state.backing.as_mut().unwrap().request.board_file = Some("import.json".into()),
            2 => state.scene.board_uuid = Uuid::new_v4().to_string(),
            _ => state.project_unopened = true,
        }
        let reason = state
            .begin_board_route(
                "00000000-0000-0000-0000-00000000c205".parse().unwrap(),
                1,
                WireRouteConfig::default(),
            )
            .unwrap_err();
        let expected = match mutation {
            0 => "Open a native Project before routing",
            2 => "Board source view is stale",
            _ => "Routing requires a native Project",
        };
        assert_eq!(
            reason, expected,
            "context guard {mutation} must refuse a valid source Pad"
        );
        assert!(state.authoring.board_route.is_none());
        assert_eq!(state.tool, WorkspaceTool::Select);
    }
}
