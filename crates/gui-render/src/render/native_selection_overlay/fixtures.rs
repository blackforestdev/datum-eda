//! Canonical native source recipes shared by selection presentation proofs.
use super::*;
pub(crate) struct Fixture {
    pub(super) root: std::path::PathBuf,
    pub(super) model: eda_engine::substrate::DesignModel,
    pub(super) board: uuid::Uuid,
    pub(super) schematic: uuid::Uuid,
}
impl Fixture {
    pub(crate) fn select(&self, state: &mut ReviewWorkspaceState, subject: SelectionSubject) {
        let authority = NativeSelectionResolution::capture(&self.model).unwrap();
        let view =
            build_selection_view(&self.model, &authority, subject, &state.scene, None, |_| {
                true
            })
            .unwrap();
        state.selection = SelectionTarget::Native(Box::new(view));
    }

    pub(crate) fn root(&self) -> &std::path::Path {
        &self.root
    }
    pub(crate) fn source_bytes(&self) -> Vec<(std::path::PathBuf, Vec<u8>)> {
        let mut files: Vec<_> = self
            .model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        let journal = eda_engine::substrate::transaction_journal_path(&self.root);
        files.push((journal.clone(), std::fs::read(journal).unwrap()));
        files
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
pub(crate) fn selected_state() -> (Fixture, ReviewWorkspaceState) {
    selected_track_state(10_000_000)
}
pub(crate) fn selected_track_state(length_nm: i64) -> (Fixture, ReviewWorkspaceState) {
    selected_track_state_with_layer(length_nm, "F.Cu")
}
pub(crate) fn selected_track_state_with_layer(
    length_nm: i64,
    layer_name: &str,
) -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::{
        api::native_write::{self, WriteProvenance, genesis},
        board::{Net, Track},
        substrate::{CommitSource, ProjectResolver},
    };
    let root = std::env::temp_dir().join(format!("s5a-renderer-{}", uuid::Uuid::new_v4()));
    let report = genesis::bootstrap_native_project(
        &root,
        genesis::GenesisSpec {
            project_name: "selection-renderer".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut f = Fixture {
        model: ProjectResolver::new(&root).resolve().unwrap(),
        root,
        board: report.board_uuid,
        schematic: report.schematic_uuid,
    };
    let provenance = || {
        WriteProvenance::new(
            "selection-renderer-fixture",
            CommitSource::Test,
            "source-bound immediate proof",
        )
    };
    // Supported F.Cu and actual unmodified genesis Top Copper are separate
    // fixture cases. Neither renderer nor selection invents the layer source.
    if layer_name != "Top Copper" {
        let layer = eda_engine::board::StackupLayer::new(
            1,
            layer_name,
            eda_engine::board::StackupLayerType::Copper,
            35_000,
        );
        let write = native_write::board_layout::build_set_board_stackup(
            &f.model,
            provenance(),
            f.board,
            &[layer],
        )
        .unwrap();
        native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    }
    let source = f
        .model
        .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(
        source["stackup"]["layers"][0]["name"].as_str(),
        Some(layer_name)
    );
    assert_eq!(
        source["stackup"]["layers"][0]["layer_type"].as_str(),
        Some("Copper")
    );
    let net = Net::new(uuid::Uuid::new_v4(), "N", uuid::Uuid::nil());
    let write =
        native_write::board_routing::build_place_board_net(&f.model, provenance(), &net).unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let track = Track::straight(
        uuid::Uuid::new_v4(),
        net.uuid,
        Point::new(0, 0),
        Point::new(length_nm, 0),
        200_000,
        1,
    );
    let write =
        native_write::board_routing::build_place_board_track(&f.model, provenance(), &track)
            .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.backing = Some(datum_gui_protocol::WorkspaceBacking {
        request: datum_gui_protocol::LiveReviewRequest {
            project_root: f.root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        board_path: f
            .model
            .source_shards
            .iter()
            .find(|shard| shard.kind == eda_engine::substrate::SourceShardKind::BoardRoot)
            .unwrap()
            .path
            .clone(),
    });
    state.scene.project_uuid = f.model.project.project_id.to_string();
    state.scene.board_uuid = f.board.to_string();
    state.scene.layers = vec![datum_gui_protocol::SceneLayer {
        layer_id: "layer-1".into(),
        name: layer_name.into(),
        kind: "copper".into(),
        render_order: 0,
        visible_by_default: true,
    }];
    state.scene.components.clear();
    state.scene.pads.clear();
    state.scene.vias.clear();
    state.scene.zones.clear();
    state.scene.board_texts.clear();
    state.scene.board_text_geometries.clear();
    state.scene.board_graphics.clear();
    state.scene.outline.clear();
    state.scene.component_graphics.clear();
    state.scene.component_texts.clear();
    state.scene.unrouted_primitives.clear();
    let mut primitive = state.scene.tracks[0].clone();
    primitive.object_id = "stable-track-display".into();
    primitive.source_object_uuid = track.uuid.to_string();
    primitive.track_uuid = track.uuid.to_string();
    primitive.net_uuid = Some(track.net.to_string());
    primitive.width_nm = track.width;
    primitive.path = vec![PointNm { x: 0, y: 0 }, PointNm { x: length_nm, y: 0 }];
    state.scene.tracks = vec![primitive];
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    state.scene.source_revision = f.model.model_revision.0.clone();
    let subject = SelectionSubject::Object(Id {
        class: Class::Track,
        id: track.uuid,
        instance_path: vec![],
    });
    let view =
        build_selection_view(&f.model, &authority, subject, &state.scene, None, |_| true).unwrap();
    state.selection = SelectionTarget::Native(Box::new(view));
    (f, state)
}
pub(crate) fn selected_occluded_track_state() -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::api::native_write::{self, WriteProvenance};
    let (mut fixture, mut state) = selected_track_state(2_000_000);
    let net = uuid::Uuid::parse_str(state.scene.tracks[0].net_uuid.as_ref().unwrap()).unwrap();
    let track = eda_engine::board::Track::straight(
        uuid::Uuid::new_v4(),
        net,
        Point::new(1_000_000, -500_000),
        Point::new(1_000_000, 500_000),
        200_000,
        1,
    );
    let write = native_write::board_routing::build_place_board_track(
        &fixture.model,
        WriteProvenance::new(
            "selection-renderer-fixture",
            eda_engine::substrate::CommitSource::Test,
            "source-bound occlusion proof",
        ),
        &track,
    )
    .unwrap();
    native_write::commit_prepared(&mut fixture.model, &fixture.root, write).unwrap();
    let mut primitive = state.scene.tracks[0].clone();
    primitive.object_id = "occluding-source-track".into();
    primitive.source_object_uuid = track.uuid.to_string();
    primitive.track_uuid = track.uuid.to_string();
    primitive.path = vec![
        PointNm {
            x: 1_000_000,
            y: -500_000,
        },
        PointNm {
            x: 1_000_000,
            y: 500_000,
        },
    ];
    state.scene.tracks.push(primitive);
    state.scene.source_revision = fixture.model.model_revision.0.clone();
    let subject = match &state.selection {
        SelectionTarget::Native(view) => view.subject.clone(),
        _ => unreachable!(),
    };
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let view = build_selection_view(
        &fixture.model,
        &authority,
        subject,
        &state.scene,
        None,
        |_| true,
    )
    .unwrap();
    state.selection = SelectionTarget::Native(Box::new(view));
    (fixture, state)
}

pub(crate) fn selected_via_state() -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::{
        api::native_write::{self, WriteProvenance},
        board::{StackupLayer, StackupLayerType, Via},
        substrate::CommitSource,
    };
    let (mut fixture, mut state) = selected_track_state(2_000_000);
    let provenance = || {
        WriteProvenance::new(
            "selection-renderer-fixture",
            CommitSource::Test,
            "source-bound drill-void proof",
        )
    };
    let layers = [
        StackupLayer::new(1, "F.Cu", StackupLayerType::Copper, 35_000),
        StackupLayer::new(2, "B.Cu", StackupLayerType::Copper, 35_000),
    ];
    let write = native_write::board_layout::build_set_board_stackup(
        &fixture.model,
        provenance(),
        fixture.board,
        &layers,
    )
    .unwrap();
    native_write::commit_prepared(&mut fixture.model, &fixture.root, write).unwrap();
    let net = uuid::Uuid::parse_str(state.scene.tracks[0].net_uuid.as_ref().unwrap()).unwrap();
    let via = Via {
        uuid: uuid::Uuid::new_v4(),
        net,
        position: Point::new(1_000_000, 0),
        drill: 500_000,
        diameter: 1_000_000,
        from_layer: 1,
        to_layer: 2,
    };
    let write =
        native_write::board_routing::build_place_board_via(&fixture.model, provenance(), &via)
            .unwrap();
    native_write::commit_prepared(&mut fixture.model, &fixture.root, write).unwrap();
    state.scene.layers.push(datum_gui_protocol::SceneLayer {
        layer_id: "layer-2".into(),
        name: "B.Cu".into(),
        kind: "copper".into(),
        render_order: 1,
        visible_by_default: true,
    });
    state.scene.vias = vec![datum_gui_protocol::ViaPrimitive {
        object_id: "selected-source-via".into(),
        object_kind: "via".into(),
        source_object_uuid: via.uuid.to_string(),
        via_uuid: via.uuid.to_string(),
        net_uuid: Some(net.to_string()),
        position: PointNm { x: 1_000_000, y: 0 },
        drill_nm: via.drill,
        diameter_nm: via.diameter,
        start_layer_id: "layer-1".into(),
        end_layer_id: "layer-2".into(),
    }];
    state.scene.source_revision = fixture.model.model_revision.0.clone();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let subject = SelectionSubject::Object(Id {
        class: Class::Via,
        id: via.uuid,
        instance_path: vec![],
    });
    state.selection = SelectionTarget::Native(Box::new(
        build_selection_view(
            &fixture.model,
            &authority,
            subject,
            &state.scene,
            None,
            |_| true,
        )
        .unwrap(),
    ));
    (fixture, state)
}

pub(crate) fn selected_schematic_text_state() -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::{
        api::native_write::{self, WriteProvenance},
        schematic::SchematicText,
        substrate::CommitSource,
    };
    let (mut fixture, mut state) = super::selected_points_state();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let sheet = authority
        .snapshot()
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .next()
        .unwrap()
        .uuid;
    let text = SchematicText {
        uuid: uuid::Uuid::new_v4(),
        text: "8".into(),
        position: Point::new(2_000_000, 0),
        rotation: 0,
    };
    let write = native_write::schematic_sheets::build_create_schematic_text(
        &fixture.model,
        WriteProvenance::new(
            "selection-text-fixture",
            CommitSource::Test,
            "actual native glyph silhouette",
        ),
        sheet,
        &text,
    )
    .unwrap();
    native_write::commit_prepared(&mut fixture.model, &fixture.root, write).unwrap();
    let schematic =
        datum_gui_protocol::project_native_schematic_scene(&fixture.model, &fixture.root)
            .unwrap()
            .unwrap();
    state.scene.source_revision = fixture.model.model_revision.0.clone();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let view = build_selection_view(
        &fixture.model,
        &authority,
        SelectionSubject::Object(Id {
            class: Class::SchematicText,
            id: text.uuid,
            instance_path: vec![],
        }),
        &state.scene,
        Some(&schematic),
        |_| true,
    )
    .unwrap();
    assert_eq!(view.members.len(), 1);
    assert!(view.unprojected.is_empty());
    assert!(!schematic.board_text_geometries.is_empty());
    state.schematic_scene = Some(schematic);
    state.selection = SelectionTarget::Native(Box::new(view));
    (fixture, state)
}
