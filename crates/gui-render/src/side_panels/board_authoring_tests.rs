use super::*;
#[test]
fn stale_board_form_keeps_explanation_and_cancel_but_no_write_actions() {
    use datum_gui_protocol::board_authoring_controls::{
        BoardAuthoringControls, BoardFieldBuffer, BoardToolKind,
    };
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.authoring.board_controls = Some(BoardAuthoringControls {
        root: std::path::PathBuf::from("/stale/native/project"),
        tip: None,
        basis: datum_gui_protocol::AuthoringViewBasis {
            project: uuid::Uuid::new_v4(),
            revision: "obsolete".into(),
            pane: state.ui.layout.focused,
            content: datum_gui_protocol::PaneContent::Board,
            source: uuid::Uuid::new_v4(),
            document: None,
        },
        units: eda_engine::ir::units::UnitsProfile::default()
            .resolve()
            .unwrap(),
        layer: 1,
        layer_name: "F.Cu".into(),
        kind: BoardToolKind::Route,
        fields: std::array::from_fn(|_| BoardFieldBuffer::default()),
        focus: None,
        thermal: None,
        resolved_zone_net: Err("Unavailable".into()),
        zone_clearance: Err("Unavailable".into()),
        needs_clearance_rule: false,
        refusal: None,
        pending: false,
    });
    let mut quads = Vec::new();
    let mut text = Vec::new();
    let mut hits = Vec::new();
    assert!(
        render(
            &state,
            RectPx {
                x: 0.,
                y: 0.,
                width: 320.,
                height: 900.
            },
            &mut quads,
            &mut text,
            &mut hits
        )
        .unwrap()
    );
    assert!(
        text.iter()
            .any(|t| t.text.contains("Source, pane or layer changed"))
    );
    assert!(
        hits.iter()
            .any(|h| h.target == HitTarget::BoardAuthoring(Control::Cancel))
    );
    assert!(!hits.iter().any(|h| matches!(
        h.target,
        HitTarget::BoardAuthoring(Control::Finish | Control::ApplyRule | Control::Fill)
    )));
}

struct NativeForm {
    root: std::path::PathBuf,
    state: ReviewWorkspaceState,
}
impl NativeForm {
    fn zone() -> Self {
        use datum_gui_protocol::board_authoring_controls::{BoardAuthoringControls, BoardToolKind};
        use eda_engine::api::native_write::{self as native, WriteProvenance, commit_prepared};
        use eda_engine::substrate::{CommitSource, ProjectResolver, SourceShardKind};
        let root = std::env::temp_dir().join(format!("datum-board-form-{}", uuid::Uuid::new_v4()));
        native::genesis::bootstrap_native_project(
            &root,
            native::genesis::GenesisSpec {
                project_name: "PCB form controls".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        if let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
            eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
        {
            let prepared = native::project::build_initialize_project_display_units(
                &model,
                WriteProvenance::new(
                    "form-test",
                    CommitSource::Test,
                    "Explicit fixture working units",
                ),
                profile,
                &receipt,
            )
            .unwrap();
            commit_prepared(&mut model, &root, prepared).unwrap();
        }
        let net = eda_engine::board::Net::new(uuid::Uuid::new_v4(), "FORM_NET", uuid::Uuid::nil());
        let p = native::board_routing::build_place_board_net(
            &model,
            WriteProvenance::new("form-test", CommitSource::Test, "Explicit form fixture Net"),
            &net,
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let board = model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap();
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.layout = datum_gui_protocol::WorkspaceLayout::single();
        state.scene.project_uuid = model.project.project_id.to_string();
        state.scene.source_revision = model.model_revision.0.clone();
        state.scene.board_uuid = board["uuid"].as_str().unwrap().into();
        state.scene.outline.clear();
        // This is a native single-copper-layer view, not the unrelated demo's
        // layer catalog. Copy the actual selected stackup entry as fixture data.
        let layer = board["stackup"]["layers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|layer| layer["id"].as_i64() == Some(1))
            .unwrap();
        assert_eq!(layer["layer_type"].as_str(), Some("Copper"));
        state.scene.layers = vec![datum_gui_protocol::SceneLayer {
            layer_id: "L1".into(),
            name: layer["name"].as_str().unwrap().into(),
            kind: "copper".into(),
            render_order: 0,
            visible_by_default: true,
        }];
        state.ui.filters.active_layer_id = Some("L1".into());
        state.library = datum_gui_protocol::library_browser::LibraryBrowser::capture(
            std::sync::Arc::new(model),
        );
        state.backing = Some(datum_gui_protocol::WorkspaceBacking {
            request: datum_gui_protocol::LiveReviewRequest {
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
            native_write_verbs: ["datum.pcb.place_zone".into()].into_iter().collect(),
        });
        state.set_tool(datum_gui_protocol::WorkspaceTool::DrawBoardZone);
        let mut setup = BoardAuthoringControls::capture(&state, BoardToolKind::Zone).unwrap();
        for (field, text) in [
            (Field::Net, net.uuid.to_string()),
            (Field::Priority, "0".into()),
            (Field::ThermalGap, "0 nm".into()),
            (Field::SpokeWidth, "0 nm".into()),
            (Field::Clearance, "0.2 mm".into()),
            (Field::RulePriority, "0".into()),
        ] {
            setup.field_mut(field).text = text;
        }
        setup.thermal = Some(false);
        setup.refresh_zone_projection(&state);
        setup.refusal = Some(
            "Long explicit diagnostic requiring scroll access to retained actions. ".repeat(10),
        );
        state.authoring.board_controls = Some(setup);
        state.begin_board_zone(net.uuid, 1).unwrap();
        for (x, y) in [(0, 0), (1000000, 0), (1000000, 1000000), (0, 1000000)] {
            state
                .append_board_zone_vertex(datum_gui_protocol::PointNm { x, y })
                .unwrap();
        }
        Self { root, state }
    }
}
impl Drop for NativeForm {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
#[test]
fn short_narrow_zone_form_scroll_reveals_every_enabled_field_and_action() {
    let mut fixture = NativeForm::zone();
    let panel = RectPx {
        x: 100.,
        y: 40.,
        width: 180.,
        height: 180.,
    };
    let mut visited = Vec::new();
    let mut total = 0;
    for offset in 0..400 {
        fixture.state.ui.selection_inspector.scroll_offset = offset;
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        assert!(render(&fixture.state, panel, &mut quads, &mut text, &mut hits).unwrap());
        for hit in &hits {
            if let HitTarget::BoardAuthoringNavigation {
                total: t, offsets, ..
            } = &hit.target
            {
                total = *t;
                assert!(offsets.iter().any(|(c, _, _)| *c == Control::Thermal));
                assert!(offsets.iter().any(|(c, _, _)| *c == Control::ApplyRule));
            }
            if let HitTarget::BoardAuthoring(control) = hit.target {
                assert!(
                    hit.rect.x >= panel.x && hit.rect.x + hit.rect.width <= panel.x + panel.width
                );
                assert!(
                    hit.rect.y >= panel.y + 68.
                        && hit.rect.y + hit.rect.height <= panel.y + panel.height
                );
                if hit.rect.height >= 29. && !visited.contains(&control) {
                    visited.push(control);
                }
            }
        }
        if total > 0 && offset >= total {
            break;
        }
    }
    for required in [
        Control::Field(Field::Net),
        Control::Field(Field::Priority),
        Control::Field(Field::ThermalGap),
        Control::Field(Field::SpokeWidth),
        Control::Field(Field::Clearance),
        Control::Field(Field::RulePriority),
        Control::Thermal,
        Control::ApplyRule,
        Control::Finish,
        Control::Cancel,
    ] {
        assert!(
            visited.contains(&required),
            "scroll must fully reveal {required:?}"
        );
    }
}
#[test]
fn paused_board_draft_preserves_library_and_navigator_inspector_ownership() {
    let mut fixture = NativeForm::zone();
    let original = fixture.state.authoring.clone();
    for case in 0..3 {
        if case == 0 {
            fixture
                .state
                .ui
                .layout
                .set_focused_content(datum_gui_protocol::PaneContent::Library);
        } else if case == 1 {
            fixture
                .state
                .ui
                .layout
                .set_focused_content(datum_gui_protocol::PaneContent::Board);
            fixture.state.inspector_context.owner =
                datum_gui_protocol::inspector_context::InspectorOwner::Navigator;
        } else {
            fixture.state.inspector_context.owner =
                datum_gui_protocol::inspector_context::InspectorOwner::Selection;
            let prior = fixture.state.ui.layout.focused;
            let other = fixture.state.ui.layout.split_focused_with(
                datum_gui_protocol::PaneContent::Board,
                datum_gui_protocol::SplitOrientation::Vertical,
                true,
            );
            assert_ne!(other, prior);
        }
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        assert!(
            !render(
                &fixture.state,
                RectPx {
                    x: 0.,
                    y: 0.,
                    width: 300.,
                    height: 400.
                },
                &mut quads,
                &mut text,
                &mut hits
            )
            .unwrap()
        );
        assert!(quads.is_empty() && text.is_empty() && hits.is_empty());
        assert_eq!(fixture.state.authoring, original);
    }
}

#[test]
fn authored_zone_inspector_reads_captured_identity_policy_units_and_stale_state() {
    use datum_gui_protocol::{
        AuthoredSelectionClass, AuthoredSelectionIdentity, SelectionSubject, SelectionTarget,
        selection_resolution::NativeSelectionResolution,
    };
    use eda_engine::api::native_write::{self as native, WriteProvenance, commit_prepared};
    use eda_engine::ir::geometry::{Point, Polygon};
    use eda_engine::substrate::{CommitSource, ProjectResolver, SourceShardKind};
    let mut fixture = NativeForm::zone();
    let net = fixture
        .state
        .authoring
        .board_zone
        .as_ref()
        .unwrap()
        .basis()
        .net;
    let proof = || {
        WriteProvenance::new(
            "zone-inspection-test",
            CommitSource::Test,
            "Canonical Zone inspection fixture",
        )
    };
    let mut model = ProjectResolver::new(&fixture.root).resolve().unwrap();
    let request = datum_gui_protocol::native_edit_client::board_rule::BoardClearanceRule::capture(
        &model, net, 200000, 0,
    )
    .unwrap();
    let mut rule = request.value().unwrap();
    rule["name"] = "Persisted inspection clearance".into();
    let (write, _) = native::project::build_create_project_rule(&model, proof(), rule).unwrap();
    commit_prepared(&mut model, &fixture.root, write).unwrap();
    let mut model = ProjectResolver::new(&fixture.root).resolve().unwrap();
    let request = native::board_routing::zone::ZoneRequest {
        basis: native::board_routing::zone::capture_zone_basis(&model, net, 1).unwrap(),
        polygon: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1000000, 0),
            Point::new(1000000, 1000000),
            Point::new(0, 1000000),
        ]),
        priority: 7,
        thermal_relief: false,
        thermal_gap_nm: 120000,
        thermal_spoke_width_nm: 180000,
        edit_seed: uuid::Uuid::new_v4(),
    };
    let write = native::board_routing::zone::prepare_zone(&model, proof(), &request).unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &fixture.root, write).unwrap();
    let model = ProjectResolver::new(&fixture.root).resolve().unwrap();
    fixture.state.cancel_authoring_gesture();
    fixture
        .state
        .set_tool(datum_gui_protocol::WorkspaceTool::Select);
    fixture.state.scene.source_revision = model.model_revision.0.clone();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let identity = AuthoredSelectionIdentity {
        class: AuthoredSelectionClass::Zone,
        id,
        instance_path: vec![],
    };
    let mut view = datum_gui_protocol::selection_view::build_selection_view(
        &model,
        &authority,
        SelectionSubject::Object(identity.clone()),
        &fixture.state.scene,
        None,
        |_| true,
    )
    .unwrap();
    assert_eq!(view.values[&identity]["priority"].as_u64(), Some(7));
    assert_eq!(
        view.values[&identity]["thermal_gap_nm"].as_i64(),
        Some(120000)
    );
    assert_eq!(
        view.values[&identity]["thermal_spoke_width_nm"].as_i64(),
        Some(180000)
    );
    assert_eq!(
        view.values[&identity]["zone_boundary"]["vertices"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        datum_gui_protocol::board_zones::captured_zone_clearance(&view)
            .unwrap()
            .minimum_nm,
        200000
    );
    let board = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
        .unwrap()
        .path
        .clone();
    fixture.state.library =
        datum_gui_protocol::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model));
    fixture.state.selection = SelectionTarget::Native(Box::new(view.clone()));
    // Painting consumes captured values. A missing backing file must not
    // cause a new source read/policy evaluation inside this presentation path.
    let parked = board.with_extension("parked");
    std::fs::rename(&board, &parked).unwrap();
    let mut quads = Vec::new();
    let mut text = Vec::new();
    let mut hits = Vec::new();
    assert!(
        render(
            &fixture.state,
            RectPx {
                x: 0.,
                y: 0.,
                width: 360.,
                height: 1000.
            },
            &mut quads,
            &mut text,
            &mut hits
        )
        .unwrap()
    );
    let words = text
        .iter()
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        words.contains("Net: FORM_NET")
            && words.contains("Layer: Top Copper")
            && words.contains("Vertices: 4")
            && words.contains("Spoke gap: 0.120mm")
            && words.contains("Spoke width: 0.180mm"),
        "{words}"
    );
    assert!(
        words.contains("Conservative clearance:")
            && words.contains("Persisted inspection clearance")
            && words.contains("mm")
    );
    std::fs::rename(parked, board).unwrap();
    view.stale = true;
    assert!(datum_gui_protocol::board_zones::captured_zone_clearance(&view).is_err());
    fixture.state.selection = SelectionTarget::Native(Box::new(view));
    let mut quads = Vec::new();
    let mut text = Vec::new();
    let mut hits = Vec::new();
    assert!(
        !render(
            &fixture.state,
            RectPx {
                x: 0.,
                y: 0.,
                width: 360.,
                height: 1000.
            },
            &mut quads,
            &mut text,
            &mut hits
        )
        .unwrap()
    );
    assert!(
        !hits
            .iter()
            .any(|h| h.target == HitTarget::BoardAuthoring(Control::Fill))
    );
}
