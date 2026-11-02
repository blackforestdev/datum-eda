//! Edit replacement is preview-only until the canonical name/kind request is submitted.
use super::*;
use crate::{
    AuthoredSelectionClass, AuthoredSelectionIdentity, AuthoringViewBasis, PaneContent, PaneId,
};
use eda_engine::api::native_write::{
    commit_prepared, context::WriteProvenance, schematic_connectivity::build_create_schematic_label,
};
use eda_engine::schematic::{LabelKind, NetLabel};
use eda_engine::substrate::CommitSource;

#[test]
fn label_edit_preview_replaces_same_identity_at_fixed_position_and_cancel_has_no_writes() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let sheet = SchematicAuthoring::from_model(&model)
        .unwrap()
        .sheet
        .unwrap();
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "OLD".into(),
        kind: LabelKind::Local,
        position: eda_engine::ir::geometry::Point::new(80_000_000, 80_000_000),
    };
    let write = build_create_schematic_label(
        &model,
        WriteProvenance::new("test", CommitSource::Test, "label edit preview"),
        sheet,
        &label,
    )
    .unwrap();
    commit_prepared(&mut model, root, write).unwrap();
    let original = SchematicAuthoring::from_model(&model).unwrap();
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let draft = selected::LabelEditDraft {
        target: AuthoredSelectionIdentity {
            class: AuthoredSelectionClass::Label,
            id: label.uuid,
            instance_path: original.move_paths[&label.uuid].clone(),
        },
        basis: AuthoringViewBasis {
            project: model.project.project_id,
            revision: model.model_revision.0.clone(),
            pane: PaneId(1),
            content: PaneContent::Schematic,
            source: sheet,
            document: base.schematic_document.clone(),
        },
        tip: None,
        original: label.clone(),
    };
    assert!(draft.request("OLD", LabelKind::Local).unwrap().is_none());
    for name in ["", " ", "two\nlines"] {
        assert!(draft.request(name, LabelKind::Local).is_err());
    }
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    for (kind, token) in [
        (LabelKind::Local, "local"),
        (LabelKind::Global, "global"),
        (LabelKind::Hierarchical, "hierarchical"),
        (LabelKind::Power, "power"),
    ] {
        let mut edit = original.clone();
        edit.label = Some(LabelDraft::new(kind.clone()));
        edit.label.as_mut().unwrap().name = "GND".into();
        edit.label_edit = Some(draft.clone());
        edit.picker = true;
        edit.position = Some(PointNm {
            x: -25_000_000,
            y: -25_000_000,
        });
        edit.base = Some(base.clone());
        assert!(
            edit.active(),
            "edit has live preview while form remains open"
        );
        let replacement = draft.replacement(" GND ", kind.clone()).unwrap();
        assert_eq!(
            (replacement.uuid, replacement.position),
            (label.uuid, label.position)
        );
        let scene = edit.preview_scene(&base, root).unwrap();
        let graphics: Vec<_> = scene
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == label.uuid.to_string())
            .collect();
        assert!(!graphics.is_empty());
        assert!(
            graphics
                .iter()
                .flat_map(|g| &g.path)
                .all(|point| point.x > 70_000_000 && point.y > 70_000_000),
            "form edit ignores cursor placement position"
        );
        assert!(
            !scene
                .board_graphics
                .iter()
                .any(|g| g.source_object_uuid == Uuid::nil().to_string()),
            "replacement never births a placement ghost"
        );
        assert_eq!(scene.source_revision, base.source_revision);
        assert_eq!(scene.schematic_document, base.schematic_document);
        assert_eq!(edit.source, original.source);
        let (verb, params) = draft.request(" GND ", kind).unwrap().unwrap().verb_params();
        assert_eq!(verb, "datum.schematic.rename_label");
        assert_eq!(
            params,
            serde_json::json!({"label":label.uuid,"instance_path":draft.target.instance_path,"name":"GND","kind":token})
        );
        edit.label.as_mut().unwrap().name.clear();
        let invalid = edit.preview_scene(&base, root).unwrap();
        let old_graphics: Vec<_> = base
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == label.uuid.to_string())
            .map(|g| (&g.object_id, &g.path))
            .collect();
        let unchanged: Vec<_> = invalid
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == label.uuid.to_string())
            .map(|g| (&g.object_id, &g.path))
            .collect();
        assert_eq!(
            unchanged, old_graphics,
            "invalid intermediate name retains source ink and form"
        );
        assert!(edit.picker && edit.label_edit.is_some());
        assert!(edit.cancel().is_some());
        assert!(edit.label_edit.is_none() && edit.label.is_none() && !edit.active());
        assert_eq!(edit.source, original.source);
    }
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}
