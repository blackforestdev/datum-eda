//! Real canonical wiring exercises the shared Move pose lifecycle without writes.
use super::*;
use crate::{
    AuthoredSelectionClass as C, AuthoredSelectionIdentity as Id, CompoundSelection,
    SelectionSubject,
    footprint_move_preview::native::{NativeMovePreview, wiring::WiringGroupPreview},
};
use eda_engine::api::native_write::{
    WriteProvenance, commit_prepared,
    design_clipboard_edit::DesignEditIntent,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_wire_edit::{WireEditAction, WireEndpoint, WireHandle, WireRouting},
    schematic_wire_edit_adapter::WireEditParams,
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
type Scene = crate::BoardReviewSceneV1;
type Bytes = Vec<(PathBuf, Vec<u8>)>;

fn fixture(connected: bool) -> (FixtureRoot, SchematicAuthoring, Scene, Id, Id, Bytes) {
    fixture_with_marker(connected, false)
}
fn fixture_with_marker(
    connected: bool,
    isolated_marker: bool,
) -> (FixtureRoot, SchematicAuthoring, Scene, Id, Id, Bytes) {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let sheet = SchematicAuthoring::from_model(&model)
        .unwrap()
        .sheet
        .unwrap();
    let marker = if isolated_marker {
        nm(90, 90)
    } else {
        nm(70, 80)
    };
    let end_x = if isolated_marker {
        80_100_000
    } else {
        80_000_000
    };
    let mut commands = vec![
        (
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"instance_path":[],"points":[{"x":70_000_000,"y":80_000_000},{"x":end_x,"y":80_000_000}]}),
        ),
        (
            "datum.schematic.place_junction",
            json!({"sheet":sheet,"instance_path":[],"x_nm":marker.x,"y_nm":marker.y,"symbol":null,"pin":null}),
        ),
    ];
    if connected {
        commands.push(("datum.schematic.draw_wire",json!({"sheet":sheet,"instance_path":[],"points":[{"x":end_x,"y":80_000_000},{"x":90_000_000,"y":80_000_000}]})));
    }
    for (verb, params) in commands {
        let edit = (find_native_write_verb(verb).unwrap().build)(
            &NativeWriteContext {
                model: &model,
                project_root: root,
            },
            WriteProvenance::new(
                "test",
                eda_engine::substrate::CommitSource::Test,
                "wiring pose fixture",
            ),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, root, edit).unwrap();
    }
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let source = &authoring.source.as_ref().unwrap().sheets[&sheet];
    let wire = source
        .wires
        .values()
        .find(|w| w.from.x == 70_000_000 && w.from.y == 80_000_000)
        .unwrap()
        .uuid;
    let junction = source
        .junctions
        .values()
        .find(|j| j.position.x == marker.x && j.position.y == marker.y)
        .unwrap()
        .uuid;
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let path = Vec::new();
    assert!(
        base.schematic_document
            .as_ref()
            .unwrap()
            .instance_path
            .is_empty()
    );
    let bytes = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let identity = |class, id| Id {
        class,
        id,
        instance_path: path.clone(),
    };
    (
        fixture,
        authoring,
        base,
        identity(C::Wire, wire),
        identity(C::Junction, junction),
        bytes,
    )
}
fn nm(x: i64, y: i64) -> PointNm {
    PointNm {
        x: x * 1_000_000,
        y: y * 1_000_000,
    }
}
fn connectivity(
    base: &Scene,
    authoring: &SchematicAuthoring,
    id: &Id,
    root: &Path,
) -> NativeMovePreview {
    NativeMovePreview::connectivity_with_wire_edit(
        base,
        authoring,
        id,
        nm(70, 80),
        crate::AuthoringSnapState {
            enabled: false,
            grid_nm: 100_000,
        },
        root.to_path_buf(),
        Some(WireEditParams {
            action: WireEditAction::Move,
            handle: WireHandle::Endpoint {
                end: WireEndpoint::From,
            },
            routing: WireRouting::Free,
            edit_seed: Uuid::new_v4(),
        }),
    )
    .unwrap()
}

fn group(
    base: &Scene,
    authoring: &SchematicAuthoring,
    subject: &SelectionSubject,
    root: &Path,
) -> anyhow::Result<WiringGroupPreview> {
    WiringGroupPreview::begin(
        base,
        authoring,
        subject,
        nm(70, 80),
        root.to_path_buf(),
        &Default::default(),
    )
}
fn path(scene: &Scene, wire: Uuid) -> Vec<PointNm> {
    scene
        .board_graphics
        .iter()
        .find(|g| g.object_kind == "schematic_wire" && g.source_object_uuid == wire.to_string())
        .unwrap()
        .path
        .clone()
}

#[test]
fn wiring_rxy_preview_request_preserve_original_ids_pivot_and_source_bytes() {
    let (fixture, authoring, base, wire, junction, bytes) = fixture(false);
    for (pose, end) in [
        (Rotate, nm(70, 90)),
        (FlipX, nm(60, 80)),
        (FlipY, nm(80, 80)),
    ] {
        let mut single = connectivity(&base, &authoring, &wire, &fixture.0);
        assert!(single.transform_symbol(pose, nm(70, 80)).unwrap());
        assert!(single.uses_electrical_pin_snap());
        assert_eq!(single.identity().unwrap(), wire);
        let exact = PointNm {
            x: 70_001_234,
            y: 80_005_678,
        };
        assert_eq!(
            single
                .destination(
                    exact,
                    crate::AuthoringSnapState {
                        enabled: false,
                        grid_nm: 100_000,
                    }
                )
                .unwrap(),
            exact,
            "upgraded endpoint retains exact off-grid pin destination"
        );
        let scene = single.scene_at(nm(70, 80)).unwrap();
        assert_eq!(path(&scene, wire.id), vec![nm(70, 80), end]);
        let subject = SelectionSubject::Compound(
            CompoundSelection::new(vec![wire.clone(), junction.clone()], Some(wire.clone()))
                .unwrap(),
        );
        let mut group =
            NativeMovePreview::Wiring(group(&base, &authoring, &subject, &fixture.0).unwrap());
        assert!(group.transform_symbol(pose, nm(100, 100)).unwrap());
        assert!(!group.uses_electrical_pin_snap());
        assert!(group.identity().is_err());
        let request = group.design_request(nm(100, 100)).unwrap();
        let (method, params) = request.verb_params();
        assert_eq!(method, "datum.schematic.move");
        assert_eq!(params["sheet"], json!(authoring.sheet.unwrap()));
        let instance_path: Vec<Uuid> =
            serde_json::from_value(params["instance_path"].clone()).unwrap();
        let DesignEditIntent::TransformExisting {
            targets,
            reference,
            placement,
            ..
        } = serde_json::from_value(params["design_edit"].clone()).unwrap()
        else {
            panic!("existing-object transform request")
        };
        assert_eq!(instance_path, wire.instance_path);
        assert_eq!((reference.x, reference.y), (70_000_000, 80_000_000));
        assert_eq!(
            (placement.position.x, placement.position.y),
            (100_000_000, 100_000_000)
        );
        let ids: BTreeSet<_> = targets.iter().map(|target| target.id).collect();
        assert_eq!(ids, BTreeSet::from([wire.id, junction.id]));
        let scene = group.scene_at(nm(100, 100)).unwrap();
        let end = PointNm {
            x: end.x + 30_000_000,
            y: end.y + 20_000_000,
        };
        assert_eq!(path(&scene, wire.id), vec![nm(100, 100), end]);
        assert!(group.action(nm(100, 100)).unwrap().is_move());
    }
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}

#[test]
fn wiring_wrong_path_mixed_run_and_connected_pose_refusal_keep_prior_preview() {
    let (fixture, authoring, base, wire, junction, bytes) = fixture(true);
    let mut wrong = wire.clone();
    wrong.instance_path.push(Uuid::new_v4());
    let mut mixed = junction.clone();
    mixed.class = C::Symbol;
    let subjects = [
        SelectionSubject::Object(wrong),
        SelectionSubject::Run(wire.clone().into()),
        SelectionSubject::Compound(
            CompoundSelection::new(vec![wire.clone(), mixed], None).unwrap(),
        ),
    ];
    for subject in subjects {
        assert!(group(&base, &authoring, &subject, &fixture.0).is_err());
    }
    let mut preview = connectivity(&base, &authoring, &wire, &fixture.0);
    let before = serde_json::to_value(preview.scene_at(nm(70, 80)).unwrap()).unwrap();
    assert!(preview.transform_symbol(Rotate, nm(70, 80)).is_err());
    assert!(
        matches!(preview, NativeMovePreview::Connectivity { .. }),
        "failed upgrade must be atomic"
    );
    assert_eq!(
        serde_json::to_value(preview.scene_at(nm(70, 80)).unwrap()).unwrap(),
        before
    );
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}

#[test]
fn offcenter_segment_pose_upgrade_keeps_pickup_offset_through_pointer_and_rxy() {
    let (fixture, authoring, base, wire, _, bytes) = fixture_with_marker(false, true);
    let pickup = PointNm {
        x: 76_200_000,
        y: 80_000_000,
    };
    let snap = crate::AuthoringSnapState {
        enabled: true,
        grid_nm: 100_000,
    };
    let handle = WireHandle::Segment {
        from: eda_engine::ir::geometry::Point::new(70_000_000, 80_000_000),
        to: eda_engine::ir::geometry::Point::new(80_100_000, 80_000_000),
    };
    let mut preview = NativeMovePreview::connectivity_with_wire_edit(
        &base,
        &authoring,
        &wire,
        pickup,
        snap,
        fixture.0.clone(),
        Some(WireEditParams {
            action: WireEditAction::Move,
            handle,
            routing: WireRouting::Free,
            edit_seed: Uuid::new_v4(),
        }),
    )
    .unwrap();
    let pivot = PointNm {
        x: 75_050_000,
        y: 80_000_000,
    };
    assert_eq!(preview.destination(pickup, snap).unwrap(), pivot);
    assert!(preview.transform_symbol(Rotate, pivot).unwrap());
    assert_eq!(
        preview.destination(pickup, snap).unwrap(),
        pivot,
        "upgrade must not replace the pickup with the midpoint"
    );
    assert_eq!(
        path(&preview.scene_at(pivot).unwrap(), wire.id),
        vec![
            PointNm {
                x: 75_050_000,
                y: 74_950_000
            },
            PointNm {
                x: 75_050_000,
                y: 85_050_000
            }
        ]
    );
    let pointer = PointNm {
        x: 78_260_000,
        y: 83_340_000,
    };
    let destination = PointNm {
        x: 77_150_000,
        y: 83_300_000,
    };
    assert_eq!(preview.destination(pointer, snap).unwrap(), destination);
    for (transform, expected_pose, from_x, to_x) in [
        (Rotate, (180, false), 82_200_000, 72_100_000),
        (FlipX, (180, true), 72_100_000, 82_200_000),
        (FlipY, (0, false), 72_100_000, 82_200_000),
    ] {
        assert!(preview.transform_symbol(transform, destination).unwrap());
        assert_eq!(preview.destination(pointer, snap).unwrap(), destination);
        let SchematicEditRequest::DesignEdit {
            edit:
                DesignEditIntent::TransformExisting {
                    targets,
                    reference,
                    placement,
                    ..
                },
            ..
        } = preview.design_request(destination).unwrap()
        else {
            panic!("existing wiring request")
        };
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].id, wire.id);
        assert_eq!((reference.x, reference.y), (pivot.x, pivot.y));
        assert_eq!(
            (placement.position.x, placement.position.y),
            (destination.x, destination.y)
        );
        assert_eq!((placement.rotation_deg, placement.mirrored), expected_pose);
        let scene = preview.scene_at(destination).unwrap();
        assert_eq!(
            path(&scene, wire.id),
            vec![
                PointNm {
                    x: from_x,
                    y: destination.y
                },
                PointNm {
                    x: to_x,
                    y: destination.y
                }
            ]
        );
    }
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}
