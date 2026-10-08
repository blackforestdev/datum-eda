//! Exact native scene/provenance integration reuses the existing placed fixture.
use super::tests::{fixture, scene};
use super::*;
use crate::selection_geometry_catalog::{QualificationGeometry, qualification_catalog};
use crate::selection_resolution::NativeSelectionResolution;
use eda_engine::substrate::{SourceShardKind, component_pad_source::component_pad_projection};
#[test]
fn exact30_native_both_faces_complete_scene_anchors_and_source_bytes() {
    let (root, model, placements, _) = fixture(true);
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let before = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect::<Vec<_>>();
    let journal = eda_engine::substrate::transaction_journal_path(&root);
    let journal_bytes = std::fs::read(&journal).unwrap();
    let projection = component_pad_projection(&model, &board).unwrap();
    assert!(
        projection.unavailable.is_empty(),
        "{:?}",
        projection.unavailable
    );
    assert_eq!(projection.query_pads.len(), 16);
    assert!(projection.pads.is_empty());
    let native = scene(&root);
    assert_eq!(native.version, 2);
    assert_eq!(native.pads.len(), 16);
    let encoded = serde_json::to_value(&native).unwrap();
    assert_eq!(
        serde_json::from_value::<BoardReviewSceneV1>(encoded.clone()).unwrap(),
        native
    );
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let catalog = qualification_catalog(&model, &authority, &native);
    for package in &placements {
        let id = crate::AuthoredSelectionIdentity {
            class: crate::AuthoredSelectionClass::Footprint,
            id: *package,
            instance_path: vec![],
        };
        let QualificationGeometry::ExactAnchors(anchors) = catalog[&id].as_ref().unwrap() else {
            panic!("exact anchor authority required")
        };
        assert_eq!(anchors.len(), 8);
        let expected = projection
            .query_pads
            .values()
            .filter(|p| p.terminal().package == *package)
            .map(|p| p.center())
            .collect::<Vec<_>>();
        assert_eq!(*anchors, expected);
        for p in native
            .pads
            .iter()
            .filter(|p| p.component_uuid == package.to_string())
        {
            let query = &projection.query_pads[&p.pad_uuid.parse().unwrap()];
            assert_eq!(p.center.exact_point(), query.center());
            let PadCenter::Exact { back, .. } = p.center else {
                panic!("typed exact pose")
            };
            assert_eq!(back, *package == placements[1]);
        }
    }
    let pad_id = projection.query_pads.keys().next().copied().unwrap();
    let id = crate::AuthoredSelectionIdentity {
        class: crate::AuthoredSelectionClass::Pad,
        id: pad_id,
        instance_path: vec![],
    };
    let view = crate::selection_view::build_selection_view(
        &model,
        &authority,
        crate::SelectionSubject::Object(id),
        &native,
        None,
        |_| true,
    )
    .unwrap();
    let point = projection.query_pads[&pad_id].center();
    assert_eq!(
        view.field("terminal_position_nm"),
        crate::selection_view::InspectionValue::Common(
            serde_json::json!({"x":point[0],"y":point[1]})
        )
    );
    assert!(matches!(
        view.field("local_source_position_nm"),
        crate::selection_view::InspectionValue::Common(_)
    ));
    for (path, bytes) in &before {
        assert_eq!(std::fs::read(path).unwrap(), *bytes);
    }
    assert_eq!(std::fs::read(&journal).unwrap(), journal_bytes);
    let reopened = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let again = component_pad_projection(&reopened, &board).unwrap();
    for (id, p) in &projection.query_pads {
        assert_eq!(again.query_pads[id].center(), p.center());
    }
    if let Ok(destination) = std::env::var("DATUM_EXACT30_PROOF_ROOT") {
        let destination = std::path::Path::new(&destination);
        std::fs::create_dir_all(destination).unwrap();
        fn copy(a: &std::path::Path, b: &std::path::Path) {
            std::fs::create_dir_all(b).unwrap();
            for e in std::fs::read_dir(a).unwrap() {
                let e = e.unwrap();
                if e.file_type().unwrap().is_dir() {
                    copy(&e.path(), &b.join(e.file_name()));
                } else {
                    std::fs::copy(e.path(), b.join(e.file_name())).unwrap();
                }
            }
        }
        copy(&root, destination);
        std::fs::write(
            destination.join("exact30-scene.json"),
            serde_json::to_vec_pretty(&encoded).unwrap(),
        )
        .unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
}
