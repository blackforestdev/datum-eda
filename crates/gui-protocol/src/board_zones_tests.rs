use super::*;
use crate::{LiveReviewRequest, WorkspaceBacking};
use eda_engine::substrate::ProjectResolver;
use std::{collections::BTreeMap, sync::Arc};

struct Fixture {
    root: PathBuf,
    state: ReviewWorkspaceState,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("datum-zone-draft-{}", Uuid::new_v4()));
        crate::known_good_demo::write_known_good_demo_project(&root).unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let board = capture_board_basis(&model).unwrap();
        let mut state = crate::load_fixture_workspace_state();
        state.project_unopened = false;
        state.ui.layout = crate::WorkspaceLayout::single();
        state.scene.project_uuid = board.project.to_string();
        state.scene.source_revision = board.revision.0;
        state.scene.board_uuid = board.board.to_string();
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
            native_write_verbs: [
                "datum.pcb.place_zone".into(),
                "datum.check.fill_zones".into(),
            ]
            .into_iter()
            .collect(),
        });
        Self { root, state }
    }
    fn begin(&mut self) {
        self.state
            .begin_board_zone("00000000-0000-0000-0000-00000000c200".parse().unwrap(), 1)
            .unwrap();
    }
    fn configured(&mut self) {
        self.begin();
        self.state
            .authoring
            .board_zone
            .as_mut()
            .unwrap()
            .configure(ZoneSettings {
                priority: 3,
                thermal_relief: false,
                thermal_gap_nm: 0,
                thermal_spoke_width_nm: 0,
            })
            .unwrap();
        for (x, y) in [
            (0, 0),
            (1_000_000, 0),
            (1_000_000, 1_000_000),
            (0, 1_000_000),
        ] {
            self.state
                .append_board_zone_vertex(PointNm { x, y })
                .unwrap();
        }
    }
    fn bytes(&self) -> BTreeMap<String, Vec<u8>> {
        self.state
            .library
            .basis
            .as_ref()
            .unwrap()
            .source_shards
            .iter()
            .map(|s| {
                (
                    s.relative_path.clone(),
                    std::fs::read(self.root.join(&s.relative_path)).unwrap(),
                )
            })
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn zone_shared_snap_explicit_configuration_cancel_and_pending_never_write() {
    let mut f = Fixture::new();
    let before = f.bytes();
    f.begin();
    assert!(!crate::check_runs::producer::CheckProducerRequest::editor_preview_idle(&f.state));
    assert!(
        f.state
            .authoring
            .board_zone
            .as_ref()
            .unwrap()
            .settings()
            .is_none()
    );
    assert!(f.state.prepare_board_zone().is_err());
    f.state
        .append_board_zone_vertex(PointNm {
            x: 149999,
            y: 249999,
        })
        .unwrap();
    assert_eq!(
        f.state.authoring.board_zone.as_ref().unwrap().vertices(),
        [Point::new(100000, 200000)]
    );
    assert!(f.state.remove_board_zone_vertex().unwrap());
    assert!(f.state.cancel_authoring_gesture());
    assert_eq!(before, f.bytes());
    f.configured();
    let intent = f.state.prepare_board_zone().unwrap();
    let edit = intent.board_zone.unwrap();
    assert_eq!(edit.verb_params().unwrap().0, "datum.pcb.place_zone");
    assert!(f.state.authoring.board_zone.as_ref().unwrap().pending());
    assert!(f.state.prepare_board_zone().is_err());
    assert!(
        f.state
            .append_board_zone_vertex(PointNm { x: 2_000_000, y: 0 })
            .is_err()
    );
    assert!(f.state.prepare_board_zone_fill(None, None).is_err());
    assert_eq!(before, f.bytes());
    f.state.cancel_authoring_gesture();
    assert_eq!(before, f.bytes());
}

#[test]
fn zone_draft_retires_on_root_document_revision_and_tool_changes() {
    for change in 0..4 {
        let mut f = Fixture::new();
        f.configured();
        let before = f.bytes();
        match change {
            0 => f
                .state
                .backing
                .as_mut()
                .unwrap()
                .request
                .project_root
                .push("other"),
            1 => {
                f.state
                    .ui
                    .layout
                    .set_focused_content(PaneContent::Schematic);
            }
            2 => f.state.scene.source_revision = "stale".into(),
            _ => {
                f.state.set_tool(WorkspaceTool::Select);
            }
        }
        if change != 3 {
            assert!(f.state.prepare_board_zone().is_err());
            assert!(f.state.retire_stale_board_zone());
        }
        assert!(f.state.authoring.board_zone.is_none());
        assert_eq!(before, f.bytes());
    }
}

#[test]
fn explicit_fill_scope_uses_distinct_native_request_and_blocks_during_boundary_draft() {
    let f = Fixture::new();
    let before = f.bytes();
    let intent = f.state.prepare_board_zone_fill(None, None).unwrap();
    let Some(NativeZoneEdit::Fill(request)) = intent.board_zone else {
        panic!("expected explicit Fill")
    };
    assert!(request.zone.is_none() && request.net.is_none());
    assert_eq!(
        request.basis,
        capture_board_basis(f.state.library.basis.as_ref().unwrap()).unwrap()
    );
    assert_eq!(before, f.bytes());
    assert!(
        f.state
            .prepare_board_zone_fill(Some(Uuid::new_v4()), None)
            .is_err()
    );
}
