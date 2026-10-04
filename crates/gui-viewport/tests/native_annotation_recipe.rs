//! Frozen source-backed annotations exercise existing producers and shared
//! qualification/acquisition. This is not a native input acceptance substitute.
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, LiveReviewRequest, PointNm,
    SelectionSubject, SelectionSubjectError,
    selection_geometry_catalog::{QualificationGeometry, qualification_catalog},
    selection_resolution::{NativeSelectionResolution, SelectionResolutionError},
    selection_scene::{authored_candidate, projection_candidate},
    selection_view::{InspectionValue, build_selection_view},
};
use datum_gui_viewport::{
    selection_acquisition::{AcquisitionEntry, AcquisitionMode, acquire_authored},
    selection_region::SelectionRegion,
};
use eda_engine::substrate::{ProjectResolver, transaction_journal_path};
use serde_json::Value;
use std::{collections::BTreeSet, path::PathBuf};
fn rectangle(x1: i64, y1: i64, x2: i64, y2: i64) -> SelectionRegion {
    SelectionRegion::new(&[
        PointNm { x: x1, y: y1 },
        PointNm { x: x2, y: y1 },
        PointNm { x: x2, y: y2 },
        PointNm { x: x1, y: y2 },
    ])
    .unwrap()
}
#[test]
#[ignore = "requires frozen annotation-scope recipe and pinned EDA_CLI_BIN; no absent-fixture pass"]
fn actual_no_connect_and_board_text_keep_independent_sources_and_complete_inspection() {
    let root = PathBuf::from(
        std::env::var_os("S5A_NATIVE_ANNOTATION_FIXTURE").expect("explicit frozen source recipe"),
    );
    assert!(
        std::env::var_os("EDA_CLI_BIN").is_some(),
        "pin actual CLI; no Cargo fallback"
    );
    let report: Value =
        serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
    let annotations = &report["annotations"];
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        model.model_revision.0,
        annotations["model_revision"].as_str().unwrap()
    );
    let mut before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = transaction_journal_path(&root);
    before.push((journal.clone(), std::fs::read(journal).unwrap()));
    let state = datum_gui_protocol::load_board_editor_workspace_state(&LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let identity = |key: &str, class| Id {
        class,
        id: annotations[key]["occurrence"]["source_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
        instance_path: vec![],
    };
    let marker = identity("marker", Class::NoConnect);
    let text = identity("board_text", Class::BoardText);
    let schematic = state.schematic_scene.as_ref().unwrap();
    for (scene, id) in [(schematic, &marker), (&state.scene, &text)] {
        let keys: Vec<_> = if id.class == Class::NoConnect {
            scene
                .board_graphics
                .iter()
                .map(|g| g.object_id.as_str())
                .collect()
        } else {
            scene
                .board_texts
                .iter()
                .map(|t| t.object_id.as_str())
                .collect()
        };
        let key = keys
            .iter()
            .find(|key| projection_candidate(scene, key, &authority).as_ref() == Some(id))
            .expect("actual independent annotation projection");
        assert_eq!(authored_candidate(scene, key, &authority), Some(id.clone()));
    }
    assert!(
        matches!(
            authority.acquire_run(marker.clone(), None),
            Err(SelectionResolutionError::InvalidSubject(
                SelectionSubjectError::InvalidRunOrigin
            ))
        ),
        "no-connect remains object-only"
    );
    let markers = qualification_catalog(&model, &authority, schematic);
    let QualificationGeometry::Anchors(anchors) = markers[&marker].as_ref().unwrap() else {
        panic!("actual connection anchor")
    };
    let actual = &annotations["actual_marker"]["position"];
    assert_eq!(
        anchors,
        &[PointNm {
            x: actual["x"].as_i64().unwrap(),
            y: actual["y"].as_i64().unwrap()
        }]
    );
    let anchor = anchors[0];
    let small = rectangle(anchor.x - 1, anchor.y - 1, anchor.x + 1, anchor.y + 1);
    assert!(small.anchors(anchors).unwrap());
    let parent = Id {
        class: Class::Symbol,
        id: annotations["actual_marker"]["symbol"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
        instance_path: vec![],
    };
    let QualificationGeometry::Anchors(pins) = markers[&parent].as_ref().unwrap() else {
        panic!("complete parent Pin anchors")
    };
    assert!(
        !small.anchors(pins).unwrap(),
        "marker alone does not acquire a five-Pin parent"
    );
    let texts = qualification_catalog(&model, &authority, &state.scene);
    let QualificationGeometry::Layout(layout) = texts[&text].as_ref().unwrap() else {
        panic!("real oriented engine text layout")
    };
    let axis = |i: usize| {
        let values: Vec<_> = layout.iter().map(|p| p[i]).collect();
        (
            values
                .iter()
                .map(|r| r.numerator.div_euclid(r.denominator))
                .min()
                .unwrap() as i64
                - 1,
            values
                .iter()
                .map(|r| r.numerator.div_euclid(r.denominator))
                .max()
                .unwrap() as i64
                + 2,
        )
    };
    let (x1, x2) = axis(0);
    let (y1, y2) = axis(1);
    assert!(rectangle(x1, y1, x2, y2).majority_layout(layout).unwrap());
    assert!(
        !rectangle(x2 + 10, y1, x2 + 20, y2)
            .majority_layout(layout)
            .unwrap()
    );
    let selected = acquire_authored(
        &SelectionSubject::None,
        BTreeSet::from([marker.clone()]),
        AcquisitionMode::Replace,
        AcquisitionEntry::Individual,
        &authority,
    )
    .unwrap();
    assert_eq!(selected.subject, SelectionSubject::Object(marker.clone()));
    let compound = acquire_authored(
        &selected.subject,
        BTreeSet::from([text.clone()]),
        AcquisitionMode::Add,
        AcquisitionEntry::Individual,
        &authority,
    )
    .unwrap();
    assert_eq!(
        compound.members,
        BTreeSet::from([marker.clone(), text.clone()])
    );
    let mut view = build_selection_view(
        &model,
        &authority,
        compound.subject,
        &state.scene,
        Some(schematic),
        |_| true,
    )
    .unwrap();
    let members = view.members.clone();
    view.scope = Some(Class::BoardText);
    assert_eq!(
        view.field("text"),
        InspectionValue::Common(serde_json::json!("Native board note"))
    );
    assert_eq!(
        view.field("board_layer"),
        InspectionValue::Common(serde_json::json!(3))
    );
    view.scope = Some(Class::NoConnect);
    assert_eq!(
        view.field("position_nm"),
        InspectionValue::Common(actual.clone())
    );
    assert_eq!(view.members, members);
    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(path).unwrap(),
            bytes,
            "source/journal unchanged"
        );
    }
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        model.model_revision
    );
}
