use super::*;
#[path = "../../tests/support/native_selection_fixture.rs"]
mod fixture;
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, LiveReviewRequest,
    selection_resolution::NativeSelectionResolution,
};
use eda_engine::{
    api::native_write::{self, board_annotations::graphics, board_layout},
    board::{StackupLayer, StackupLayerType},
};
use fixture::{Fixture, id, provenance};

#[test]
fn recorded_graphics_reopen_as_source_qualified_native_scene_without_selection_writes() {
    let mut f = Fixture::new("native graphic projection");
    let write = board_layout::build_set_board_stackup(
        &f.model,
        provenance(),
        f.board,
        &[StackupLayer::new(
            37,
            "F.SilkS",
            StackupLayerType::Silkscreen,
            0,
        )],
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let write = board_layout::source_profile::build_adopt_board_source_profile(
        &f.model,
        provenance(),
        f.board,
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let p = Point::new;
    let mut variants = vec![
        G::Line {
            from: p(0, 0),
            to: p(100, 0),
        },
        G::Polyline {
            points: vec![p(0, 0), p(50, 100), p(100, 0)],
        },
        G::Arc {
            from: p(0, 0),
            midpoint: p(50, 50),
            to: p(100, 0),
        },
        G::Rectangle {
            from: p(0, 0),
            to: p(100, 100),
            filled: true,
        },
        G::Circle {
            center: p(50, 50),
            radius: 50,
            filled: true,
        },
        G::Polygon {
            outer: vec![p(0, 0), p(100, 0), p(100, 100), p(0, 100)],
            holes: vec![vec![p(30, 30), p(70, 30), p(70, 70), p(30, 70)]],
            filled: true,
        },
    ];
    let unfilled: Vec<_> = variants[3..]
        .iter()
        .cloned()
        .map(|geometry| match geometry {
            G::Rectangle { from, to, .. } => G::Rectangle {
                from,
                to,
                filled: false,
            },
            G::Circle { center, radius, .. } => G::Circle {
                center,
                radius,
                filled: false,
            },
            G::Polygon { outer, holes, .. } => G::Polygon {
                outer,
                holes,
                filled: false,
            },
            _ => unreachable!(),
        })
        .collect();
    variants.extend(unfilled);
    for (i, geometry) in variants.into_iter().enumerate() {
        let graphic = BoardGraphic {
            uuid: id(100 + i as u128),
            layer: 37,
            width: if geometry.filled() { 0 } else { 2 },
            geometry,
            import_key: None,
        };
        let write = graphics::build_create_board_graphic(&f.model, provenance(), f.board, &graphic)
            .unwrap();
        native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    }
    let before = f.source_bytes();
    let model = f.reopen();
    assert_eq!(model.model_revision, f.model.model_revision);
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let (scene, _) = crate::load_scene_from_engine(&LiveReviewRequest {
        project_root: f.root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap();
    let bindings = crate::selection_scene::SceneSelectionBindings::capture(&scene, &authority);
    let catalog =
        crate::selection_geometry_catalog::qualification_catalog(&model, &authority, &scene);
    for n in 100..109 {
        let identity = Id {
            class: Class::BoardGraphic,
            id: id(n),
            instance_path: vec![],
        };
        let key = format!("board-graphic:{}", id(n));
        assert_eq!(bindings.get(&key).unwrap().acquired, identity);
        assert_eq!(
            crate::selection_scene::projection_candidate(&scene, &key, &authority),
            Some(identity.clone())
        );
        assert!(catalog[&identity].is_ok());
        assert_eq!(
            authority
                .resolve(&crate::SelectionSubject::Object(identity.clone()))
                .unwrap()
                .members,
            std::collections::BTreeSet::from([identity])
        );
    }
    let arc = Id {
        class: Class::BoardGraphic,
        id: id(102),
        instance_path: vec![],
    };
    let Q::Anchors(anchors) = catalog[&arc].as_ref().unwrap() else {
        panic!("authored anchors")
    };
    assert_eq!(anchors, &ring(&[p(0, 0), p(50, 50), p(100, 0)]));
    for n in 103..106 {
        let filled = scene
            .board_graphics
            .iter()
            .find(|g| g.source_object_uuid == id(n).to_string())
            .unwrap();
        let unfilled = scene
            .board_graphics
            .iter()
            .find(|g| g.source_object_uuid == id(n + 3).to_string())
            .unwrap();
        assert_eq!(filled.primitive_kind, "polygon");
        assert_eq!(unfilled.primitive_kind, "polyline");
        assert_eq!(unfilled.width_nm, Some(2));
        assert_eq!(filled.path, unfilled.path);
        assert_eq!(filled.holes, unfilled.holes);
    }
    let poly = scene
        .board_graphics
        .iter()
        .find(|g| g.source_object_uuid == id(105).to_string())
        .unwrap();
    assert_eq!(poly.holes.len(), 1);
    assert_eq!(poly.width_nm, None);
    assert_eq!(poly.primitive_kind, "polygon");
    assert!(crate::selection_scene::candidate_visible(
        &scene,
        &poly.object_id,
        |layer| layer == "L37"
    ));
    assert!(!crate::selection_scene::candidate_visible(
        &scene,
        &poly.object_id,
        |_| false
    ));
    let mut spoof = scene.clone();
    let g = spoof
        .board_graphics
        .iter_mut()
        .find(|g| g.object_id == poly.object_id)
        .unwrap();
    g.source_object_uuid = id(999).to_string();
    assert!(
        crate::selection_scene::projection_candidate(&spoof, &poly.object_id, &authority).is_none()
    );
    let mut broken = model.clone();
    broken.objects.remove(&id(105));
    assert!(
        NativeSelectionResolution::capture(&broken).is_err(),
        "missing registration refuses complete inventory"
    );
    Fixture::assert_bytes(&before);
}
