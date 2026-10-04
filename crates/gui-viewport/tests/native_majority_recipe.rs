//! Canonical source-backed strict-majority and pinless-anchor oracles.
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, LiveReviewRequest, PointNm,
    SelectionSubject,
    selection_geometry_catalog::{QualificationGeometry, qualification_catalog},
    selection_resolution::{NativeSelectionResolution, SelectionAuthority},
    selection_view::build_selection_view,
};
use datum_gui_viewport::{
    selection_acquisition::{AcquisitionEntry, AcquisitionMode, acquire_authored},
    selection_region::SelectionRegion,
};
use eda_engine::substrate::{ProjectResolver, transaction_journal_path};
use serde_json::Value;
use std::{collections::BTreeSet, path::PathBuf};
fn rectangle(x1: i64, x2: i64) -> SelectionRegion {
    SelectionRegion::new(&[
        PointNm { x: x1, y: -1000000 },
        PointNm { x: x2, y: -1000000 },
        PointNm { x: x2, y: 1000000 },
        PointNm { x: x1, y: 1000000 },
    ])
    .unwrap()
}
#[test]
#[ignore = "requires frozen symbol-majority recipe and pinned EDA_CLI_BIN; no absent-fixture pass"]
fn fourteen_pin_majority_and_pinless_anchor_preserve_complete_parent_identity() {
    let root = PathBuf::from(
        std::env::var_os("S5A_NATIVE_MAJORITY_FIXTURE").expect("explicit frozen recipe"),
    );
    assert!(std::env::var_os("EDA_CLI_BIN").is_some());
    let report: Value =
        serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        model.model_revision.0,
        report["model_revision"].as_str().unwrap()
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
    let scene = state.schematic_scene.as_ref().unwrap();
    let catalog = qualification_catalog(&model, &authority, scene);
    let bindings =
        datum_gui_protocol::selection_scene::SceneSelectionBindings::capture(scene, &authority);
    let ids: Vec<_> = report["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| Id {
            class: Class::Symbol,
            id: s["source_id"].as_str().unwrap().parse().unwrap(),
            instance_path: vec![],
        })
        .collect();
    let QualificationGeometry::Anchors(anchors) = catalog[&ids[0]].as_ref().unwrap() else {
        panic!("complete fourteen-pin geometry")
    };
    assert_eq!(anchors.len(), 14);
    assert_eq!(
        anchors.iter().map(|p| (p.x, p.y)).collect::<BTreeSet<_>>(),
        (0..14).map(|n| (n * 1000000, 0)).collect()
    );
    assert!(
        !rectangle(-1000000, 6500000).anchors(anchors).unwrap(),
        "seven of fourteen is exactly half and fails"
    );
    assert!(
        rectangle(-1000000, 7500000).anchors(anchors).unwrap(),
        "eight of fourteen passes"
    );
    let selected = acquire_authored(
        &SelectionSubject::None,
        BTreeSet::from([ids[0].clone()]),
        AcquisitionMode::Replace,
        AcquisitionEntry::Region,
        &authority,
    )
    .unwrap();
    assert_eq!(selected.members, BTreeSet::from([ids[0].clone()]));
    assert!(
        matches!(selected.subject, SelectionSubject::Compound(_)),
        "region has no manufactured individual focus"
    );
    let view = build_selection_view(
        &model,
        &authority,
        selected.subject,
        &state.scene,
        Some(scene),
        |_| true,
    )
    .unwrap();
    assert_eq!(view.members, selected.members);
    assert!(view.focus().is_none());
    for p in report["subjects"][0]["anchors"].as_array().unwrap() {
        let pin = Id {
            class: Class::Pin,
            id: p["source_id"].as_str().unwrap().parse().unwrap(),
            instance_path: vec![],
        };
        assert!(authority.authored_contains(&pin));
        assert!(
            bindings
                .iter()
                .any(|(_, b)| b.projected == pin && b.acquired == ids[0]),
            "actual Pin hit collapses to Symbol"
        );
        assert!(
            !view.members.contains(&pin),
            "complete parent acquisition never creates child compound members"
        );
    }
    let QualificationGeometry::Anchors(pinless) = catalog[&ids[1]].as_ref().unwrap() else {
        panic!("pinless source placement anchor")
    };
    assert_eq!(pinless, &vec![PointNm { x: 30000000, y: 0 }]);
    assert!(rectangle(29000000, 31000000).anchors(pinless).unwrap());
    assert!(!rectangle(31000000, 32000000).anchors(pinless).unwrap());
    for (path, bytes) in before {
        assert!(
            std::fs::read(&path).unwrap() == bytes,
            "source/journal unchanged: {}",
            path.display()
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
