//! Existing native-scene fixture extended by real canonical source producers.
use super::*;
use crate::selection_resolution::NativeSelectionResolution;
use crate::selection_subject::{AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id};
use eda_engine::api::native_write::{
    WriteProvenance,
    board_components::{
        BoardPackageEdit, BoardPackagePlacement, build_edit_board_package,
        build_place_board_package, pad_materialization::*,
    },
    board_layout::source_profile::build_adopt_board_source_profile,
    commit_prepared,
    library::{
        PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
        build_pool_library_write, pool_part_payload,
    },
};
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;
use eda_engine::substrate::{CommitSource, ProjectResolver, SourceShardKind};
use serde_json::json;
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "native-pad-projection-test",
        CommitSource::Test,
        "PM057 native projection and Inspector",
    )
}
fn scene(root: &Path) -> BoardReviewSceneV1 {
    load_scene_from_engine(&LiveReviewRequest {
        project_root: root.into(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap()
    .0
}
#[test]
fn native_placed_pad_scene_majority_and_stale_inspector_use_same_journaled_sources() {
    let root = crate::tests::unique_project_root("native_placed_pad_projection");
    let board_id = Uuid::new_v4();
    crate::tests::write_minimal_native_project(&root, Uuid::new_v4(), board_id);
    let path = root.join("board/board.json");
    let mut initial: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    initial["stackup"] = json!({"layers":[{"id":1,"name":"F.Cu","layer_type":"Copper","thickness_nm":35000},{"id":2,"name":"B.Cu","layer_type":"Copper","thickness_nm":35000}]});
    crate::tests::write_json(&path, initial);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let part = Uuid::new_v4();
    let package = Uuid::new_v4();
    let library_pad = Uuid::new_v4();
    let stack = Uuid::new_v4();
    let objects = vec![
        (
            "packages",
            package,
            json!({"schema_version":1,"uuid":package,"name":"PKG","pads":{library_pad.to_string():{"uuid":library_pad,"name":"1","position":{"x":100000,"y":200000},"padstack":stack,"layer":1}}}),
        ),
        (
            "padstacks",
            stack,
            json!({"schema_version":1,"uuid":stack,"name":"PAD","aperture":{"rect":{"width_nm":400000,"height_nm":600000}},"drill_nm":null,"plated":false}),
        ),
        (
            "parts",
            part,
            pool_part_payload(part, Uuid::new_v4(), package, "", "", "", "", "", "Active"),
        ),
    ];
    let write = build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        objects
            .into_iter()
            .map(|(kind, id, mut object)| {
                object["schema_version"] = 1.into();
                Spec::Create {
                    target: Target::new("pool", kind, id),
                    object,
                }
            })
            .collect(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let write = build_adopt_board_source_profile(&model, provenance(), board_id).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut placements = Vec::new();
    for n in 0..2 {
        let placement = PlacedPackage {
            uuid: Uuid::new_v4(),
            part,
            package,
            reference: format!("U{n}"),
            value: "".into(),
            position: Point::new(3000000 + 2000000 * n, 4000000),
            rotation: 90,
            layer: 1,
            locked: false,
        };
        let request = ComponentPadMaterializationRequest {
            placement: placement.clone(),
            land_pattern: PadLandPatternSource::LegacyPackage,
            component_instance: None,
            pin_pad_map: None,
            correspondences: vec![],
            adopt_existing: false,
        };
        let payload = prepare_component_pad_materialization(&model, &request, &json!({})).unwrap();
        let write = build_place_board_package(
            &model,
            provenance(),
            &BoardPackagePlacement {
                package: placement.clone(),
                materialized: payload,
            },
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        placements.push(placement.uuid);
    }
    let raw = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let first: Uuid =
        serde_json::from_value(raw["component_pads"][placements[0].to_string()][0]["uuid"].clone())
            .unwrap();
    let second: Uuid =
        serde_json::from_value(raw["component_pads"][placements[1].to_string()][0]["uuid"].clone())
            .unwrap();
    assert_ne!(first, second);
    assert_ne!(first, library_pad);
    let bytes = std::fs::read(&path).unwrap();
    let journal = std::fs::read(eda_engine::substrate::transaction_journal_path(&root)).unwrap();
    let board = scene(&root);
    assert_eq!(board.pads.len(), 2);
    assert!(board.pads.iter().any(|p| p.pad_uuid == first.to_string()
        && p.center
            == PointNm {
                x: 2800000,
                y: 4100000
            }));
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let id = Id {
        class: Class::Footprint,
        id: placements[0],
        instance_path: vec![],
    };
    let catalog =
        crate::selection_geometry_catalog::qualification_catalog(&model, &authority, &board);
    let crate::selection_geometry_catalog::QualificationGeometry::Anchors(anchors) =
        catalog[&id].as_ref().unwrap()
    else {
        panic!("strict majority requires actual transformed Pad anchors")
    };
    assert_eq!(
        anchors,
        &vec![PointNm {
            x: 2800000,
            y: 4100000
        }]
    );
    let view = crate::selection_view::build_selection_view(
        &model,
        &authority,
        SelectionSubject::Object(id.clone()),
        &board,
        None,
        |_| true,
    )
    .unwrap();
    assert_eq!(
        view.field("pad_materialization_status"),
        crate::selection_view::InspectionValue::Common(json!("current"))
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        std::fs::read(eda_engine::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let write = build_edit_board_package(
        &model,
        provenance(),
        placements[0],
        BoardPackageEdit::Part {
            part_id: Uuid::new_v4(),
        },
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let board = scene(&root);
    let view = crate::selection_view::build_selection_view(
        &model,
        &authority,
        SelectionSubject::Object(id),
        &board,
        None,
        |_| true,
    )
    .unwrap();
    assert_eq!(
        view.field("pad_materialization_status"),
        crate::selection_view::InspectionValue::Common(json!("stale"))
    );
    assert!(board.pads.iter().any(|p| p.pad_uuid == first.to_string()
        && p.center
            == PointNm {
                x: 2800000,
                y: 4100000
            }));
    std::fs::remove_dir_all(root).unwrap();
}
