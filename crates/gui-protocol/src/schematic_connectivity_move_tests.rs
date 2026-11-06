//! Rigid Wire/Junction Move previews over the immutable qualified source.
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
fn wire_and_junction_move_previews_translate_rigidly_without_writes() {
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
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"instance_path":[],"points":[{"x":70_000_000,"y":80_000_000},{"x":80_000_000,"y":80_000_000}]}),
        ),
        (
            "datum.schematic.place_junction",
            json!({"sheet":sheet,"instance_path":[],"x_nm":70_000_000,"y_nm":80_000_000,"symbol":null,"pin":null}),
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
                "connectivity Move preview",
            ),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
    }
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let source = authoring.source.as_ref().unwrap();
    let wire = source.sheets[&sheet]
        .wires
        .values()
        .find(|w| w.from.x == 70_000_000 && w.from.y == 80_000_000)
        .unwrap()
        .clone();
    let junction = source.sheets[&sheet]
        .junctions
        .values()
        .find(|j| j.position.x == 70_000_000)
        .unwrap()
        .clone();
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let snap = crate::AuthoringSnapState {
        enabled: false,
        grid_nm: 100_000,
    };
    let cursor = PointNm {
        x: 71_000_000,
        y: 81_000_000,
    };
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let document_path = base
        .schematic_document
        .as_ref()
        .unwrap()
        .instance_path
        .clone();
    for (class, id, anchor) in [
        (AuthoredSelectionClass::Wire, wire.uuid, wire.from),
        (
            AuthoredSelectionClass::Junction,
            junction.uuid,
            junction.position,
        ),
    ] {
        let identity = AuthoredSelectionIdentity {
            class,
            id,
            instance_path: document_path.clone(),
        };
        let preview = NativeMovePreview::connectivity(
            &base,
            &authoring,
            &identity,
            cursor,
            snap,
            root.clone(),
        )
        .unwrap();
        assert!(preview.is_schematic());
        assert_eq!(preview.identity().unwrap(), identity);
        assert_eq!(preview.instance_path(), document_path);
        // Translation is cursor-delta applied to the object's anchor.
        let at = preview
            .destination(
                PointNm {
                    x: 73_000_000,
                    y: 84_000_000,
                },
                snap,
            )
            .unwrap();
        assert_eq!(
            (at.x, at.y),
            (anchor.x + 2_000_000, anchor.y + 3_000_000),
            "anchor-relative rigid translation"
        );
        let action = preview.action(at).unwrap();
        // The produced action must carry the shared runtime Move permission:
        // this is the preview→action→gate contract the run94 commit-gate
        // omission broke, asserted across the real layers rather than a
        // mirrored enum list.
        assert!(action.is_move());
        match action {
            NativeEditAction::MoveConnectivity { object, x_nm, y_nm } => {
                assert_eq!(object, id);
                assert_eq!((x_nm, y_nm), (at.x, at.y));
            }
            other => panic!("unexpected action {other:?}"),
        }
        // Preview ink projects a scene; the authored source bytes never change.
        preview.scene_at(at).unwrap();
    }
    // A mismatched occurrence path and a non-connectivity class both refuse.
    assert!(
        NativeMovePreview::connectivity(
            &base,
            &authoring,
            &AuthoredSelectionIdentity {
                class: AuthoredSelectionClass::Wire,
                id: wire.uuid,
                instance_path: vec![uuid::Uuid::new_v4()],
            },
            cursor,
            snap,
            root.clone(),
        )
        .is_err()
    );
    assert!(
        NativeMovePreview::connectivity(
            &base,
            &authoring,
            &AuthoredSelectionIdentity {
                class: AuthoredSelectionClass::Port,
                id: wire.uuid,
                instance_path: document_path.clone(),
            },
            cursor,
            snap,
            root.clone(),
        )
        .is_err()
    );
    assert_eq!(
        authoring.source,
        SchematicAuthoring::from_model(&model).unwrap().source
    );
    for (path, before) in bytes {
        assert_eq!(
            std::fs::read(path).unwrap(),
            before,
            "preview writes no source bytes"
        );
    }
}

#[test]
fn label_and_text_move_previews_accept_the_captured_nonprimary_occurrence() {
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
            json!({"sheet":sheet,"name":"NET1","kind":"local","x_nm":60_000_000,"y_nm":60_000_000}),
        ),
        (
            "datum.schematic.place_text",
            json!({"sheet":sheet,"instance_path":[],"value":"note","x_nm":50_000_000,"y_nm":50_000_000,"rotation_deg":0}),
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
                "nonprimary move gate",
            ),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
    }
    let mut authoring = SchematicAuthoring::from_model(&model).unwrap();
    let source = authoring.source.as_ref().unwrap();
    let label = source.sheets[&sheet]
        .labels
        .values()
        .find(|l| l.name == "NET1")
        .unwrap()
        .clone();
    let text = source.sheets[&sheet]
        .texts
        .values()
        .find(|t| t.text == "note")
        .unwrap()
        .clone();
    let mut base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    // Present the same source sheet as a captured NONPRIMARY occurrence: the
    // constructors must trust the captured document, not the primary field.
    let occurrence = uuid::Uuid::new_v4();
    let document = base.schematic_document.clone().unwrap();
    base.schematic_document = Some(crate::schematic_documents::SchematicDocumentKey {
        instance_path: vec![occurrence],
        ..document
    });
    authoring.move_paths.insert(label.uuid, vec![occurrence]);
    authoring.move_paths.insert(text.uuid, vec![occurrence]);
    let snap = crate::AuthoringSnapState {
        enabled: false,
        grid_nm: 100_000,
    };
    let cursor = PointNm {
        x: 61_000_000,
        y: 61_000_000,
    };
    for (class, id) in [
        (AuthoredSelectionClass::Label, label.uuid),
        (AuthoredSelectionClass::SchematicText, text.uuid),
    ] {
        let begin = match class {
            AuthoredSelectionClass::Label => NativeMovePreview::label,
            _ => NativeMovePreview::text,
        };
        let identity = AuthoredSelectionIdentity {
            class,
            id,
            instance_path: vec![occurrence],
        };
        let preview = begin(&base, &authoring, &identity, cursor, snap, root.clone()).unwrap();
        assert_eq!(preview.identity().unwrap(), identity);
        assert_eq!(preview.instance_path(), vec![occurrence]);
        // A path that does not match the captured document refuses.
        assert!(
            begin(
                &base,
                &authoring,
                &AuthoredSelectionIdentity {
                    class,
                    id,
                    instance_path: vec![],
                },
                cursor,
                snap,
                root.clone(),
            )
            .is_err()
        );
    }
}

#[test]
fn wire_point_returns_exact_contact_only_for_captured_source_wires() {
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
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"instance_path":[],
                "points":[{"x":0,"y":2_110_000},{"x":10_000_000,"y":2_110_000}]}),
        ),
        (
            "datum.schematic.place_label",
            json!({"sheet":sheet,"name":"DATA0","kind":"local","x_nm":0,"y_nm":0}),
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
                "wire point service",
            ),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
    }
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let source = authoring.source.as_ref().unwrap();
    let wire = source.sheets[&sheet]
        .wires
        .values()
        .find(|w| w.from.y == 2_110_000)
        .unwrap();
    let label = source.sheets[&sheet]
        .labels
        .values()
        .find(|l| l.name == "DATA0")
        .unwrap();
    // The run91 defect shape: a grid-snapped desired point one grid line off
    // the wire must land exactly ON the wire.
    let desired = PointNm {
        x: 5_000_000,
        y: 2_100_000,
    };
    assert_eq!(
        authoring
            .wire_point(&wire.uuid.to_string(), desired)
            .unwrap(),
        Some(PointNm {
            x: 5_000_000,
            y: 2_110_000
        })
    );
    // Non-wire sources and unparseable ids keep their existing behavior.
    assert_eq!(
        authoring
            .wire_point(&label.uuid.to_string(), desired)
            .unwrap(),
        None
    );
    assert_eq!(authoring.wire_point("not-a-uuid", desired).unwrap(), None);
}
