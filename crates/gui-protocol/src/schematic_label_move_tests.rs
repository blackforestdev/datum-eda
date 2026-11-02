//! Label Move uses the canonical source basis for glyphs and attached wire preview.
use super::*;
use crate::{
    AuthoredSelectionClass, AuthoredSelectionIdentity,
    footprint_move_preview::native::NativeMovePreview, native_edit_client::NativeEditAction,
};
use eda_engine::api::native_write::{
    WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
};
use serde_json::json;

#[test]
fn power_label_move_preview_translates_glyph_and_wire_without_writes() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let sheet = SchematicAuthoring::from_model(&model)
        .unwrap()
        .sheet
        .unwrap();
    for (verb, params) in [
        (
            "datum.schematic.place_label",
            json!({"sheet":sheet,"name":"GND","kind":"power","x_nm":80_000_000,"y_nm":80_000_000}),
        ),
        (
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"points":[{"x":70_000_000,"y":80_000_000},{"x":80_000_000,"y":80_000_000}]}),
        ),
    ] {
        let prepared = (find_native_write_verb(verb).unwrap().build)(
            &NativeWriteContext {
                model: &model,
                project_root: root,
            },
            WriteProvenance::new(
                "test",
                eda_engine::substrate::CommitSource::Test,
                "Power Move preview",
            ),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
    }
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let source = authoring.source.as_ref().unwrap();
    let label = source.sheets[&sheet]
        .labels
        .values()
        .find(|l| l.position.x == 80_000_000 && l.position.y == 80_000_000)
        .unwrap();
    let wire = source.sheets[&sheet]
        .wires
        .values()
        .find(|w| w.to.x == 80_000_000 && w.to.y == 80_000_000)
        .unwrap();
    let id = AuthoredSelectionIdentity {
        class: AuthoredSelectionClass::Label,
        id: label.uuid,
        instance_path: authoring.move_paths[&label.uuid].clone(),
    };
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let cursor = PointNm {
        x: 81_000_000,
        y: 82_000_000,
    };
    let snap = crate::AuthoringSnapState {
        enabled: false,
        grid_nm: 100_000,
    };
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let mut preview =
        NativeMovePreview::label(&base, &authoring, &id, cursor, snap, root.clone()).unwrap();
    assert_eq!(preview.identity().unwrap(), id);
    assert!(preview.is_schematic());
    let at = preview
        .destination(
            PointNm {
                x: 83_000_000,
                y: 85_000_000,
            },
            snap,
        )
        .unwrap();
    assert_eq!(
        at,
        PointNm {
            x: 82_000_000,
            y: 83_000_000
        },
        "grabbing glyph off-anchor preserves cursor offset"
    );
    assert!(
        !preview.transform_symbol(Rotate, at).unwrap(),
        "Label has no Symbol pose"
    );
    let NativeEditAction::MoveLabel {
        label: moved_id,
        x_nm,
        y_nm,
    } = preview.action(at).unwrap()
    else {
        panic!("wrong Move action")
    };
    assert_eq!((moved_id, x_nm, y_nm), (id.id, at.x, at.y));
    let scene = preview.scene_at(at).unwrap();
    let before: Vec<_> = base
        .board_graphics
        .iter()
        .filter(|g| g.source_object_uuid == label.uuid.to_string())
        .collect();
    assert!(!before.is_empty());
    for graphic in before {
        assert_eq!(
            authoring.pin_point(&graphic.source_object_uuid),
            Some(PointNm {
                x: label.position.x,
                y: label.position.y
            }),
            "picking any visible Power glyph stroke resolves its exact electrical origin"
        );
        let moved = scene
            .board_graphics
            .iter()
            .find(|g| g.object_id == graphic.object_id)
            .unwrap();
        assert_eq!(
            moved.path,
            graphic
                .path
                .iter()
                .map(|p| PointNm {
                    x: p.x + 2_000_000,
                    y: p.y + 3_000_000
                })
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(
        authoring.pin_point(&wire.uuid.to_string()),
        None,
        "wire ink must not become a Power terminal"
    );
    let moved_wire = scene
        .board_graphics
        .iter()
        .find(|g| g.source_object_uuid == wire.uuid.to_string() && g.path.len() == 2)
        .unwrap();
    assert_eq!(
        moved_wire.path,
        vec![
            PointNm {
                x: 70_000_000,
                y: 80_000_000
            },
            at
        ]
    );
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
    let mut wrong = id.clone();
    wrong.instance_path.push(Uuid::new_v4());
    assert!(
        NativeMovePreview::label(&base, &authoring, &wrong, cursor, snap, root.clone()).is_err()
    );
}
