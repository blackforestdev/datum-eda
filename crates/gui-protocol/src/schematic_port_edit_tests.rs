//! Port configuration uses the shared preview while preserving source identity and files.
use super::*;
use eda_engine::api::native_write::schematic_connectivity::build_create_schematic_port;
use eda_engine::schematic::{HierarchicalPort, PortDirection};

#[test]
fn interface_port_preview_and_edit_preserve_identity_and_never_create_labels() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let mut authoring = SchematicAuthoring::from_model(&model).unwrap();
    let sheet = authoring.sheet.unwrap();
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let at = PointNm {
        x: 80_123_456,
        y: 80_654_321,
    };
    let mut form = LabelDraft::port(PortDirection::Input);
    assert!(form.placement_request(sheet, at).is_err());
    form.name = " IO ".into();
    let (verb, params) = form.placement_request(sheet, at).unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.place_port");
    assert_eq!(
        params,
        serde_json::json!({"sheet":sheet,"port":null,"instance_path":[],"name":"IO","direction":"input","x_nm":at.x,"y_nm":at.y})
    );
    authoring.label = Some(form);
    authoring.position = Some(at);
    authoring.base = Some(base.clone());
    let original_source = authoring.source.clone();
    let scene = authoring.preview_scene(&base, root).unwrap();
    let ghost: Vec<_> = scene
        .board_graphics
        .iter()
        .filter(|g| g.source_object_uuid == Uuid::nil().to_string())
        .collect();
    assert!(!ghost.is_empty());
    assert!(
        ghost
            .iter()
            .all(|g| g.object_id.starts_with("schematic-port:"))
    );
    assert_eq!(authoring.source, original_source);
    assert_eq!(authoring.cancel().unwrap(), base);
    assert!(!authoring.active());

    let port = HierarchicalPort {
        uuid: Uuid::new_v4(),
        name: "IO".into(),
        direction: PortDirection::Input,
        position: eda_engine::ir::geometry::Point { x: at.x, y: at.y },
    };
    let write = build_create_schematic_port(
        &model,
        WriteProvenance::new("test", CommitSource::Test, "port shared-form preview"),
        sheet,
        &port,
    )
    .unwrap();
    commit_prepared(&mut model, root, write).unwrap();
    let original = SchematicAuthoring::from_model(&model).unwrap();
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let draft = selected::PortEditDraft {
        target: AuthoredSelectionIdentity {
            class: AuthoredSelectionClass::Port,
            id: port.uuid,
            instance_path: vec![],
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
        original: port.clone(),
    };
    assert!(draft.request("IO", PortDirection::Input).unwrap().is_none());
    for invalid in ["", " ", "two\nlines"] {
        assert!(draft.request(invalid, PortDirection::Input).is_err());
    }
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    for (direction, token) in [
        (PortDirection::Input, "input"),
        (PortDirection::Output, "output"),
        (PortDirection::Bidirectional, "bidirectional"),
        (PortDirection::Passive, "passive"),
    ] {
        let named = selected::NamedPointEditDraft::Port(draft.clone());
        let mut form = named.form();
        form.name = " RENAMED ".into();
        form.port_direction = Some(direction.clone());
        let replacement = draft.replacement(&form.name, direction.clone()).unwrap();
        assert_eq!(
            (replacement.uuid, replacement.position),
            (port.uuid, port.position)
        );
        let (verb, params) = named.request(&form).unwrap().unwrap().verb_params();
        assert_eq!(verb, "datum.schematic.place_port");
        assert_eq!(
            params,
            serde_json::json!({"sheet":sheet,"port":port.uuid,"instance_path":[],"name":"RENAMED","direction":token,"x_nm":at.x,"y_nm":at.y})
        );
        assert!(named.request(&LabelDraft::new(LabelKind::Local)).is_err());
        let mut edit = original.clone();
        edit.label = Some(form);
        edit.named_edit = Some(named.clone());
        edit.picker = true;
        edit.position = Some(PointNm {
            x: -25_000_000,
            y: -25_000_000,
        });
        edit.base = Some(base.clone());
        assert!(edit.active());
        let scene = edit.preview_scene(&base, root).unwrap();
        let graphics: Vec<_> = scene
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == port.uuid.to_string())
            .collect();
        assert!(!graphics.is_empty());
        assert!(
            graphics
                .iter()
                .all(|g| g.object_id.starts_with("schematic-port:"))
        );
        assert!(
            graphics
                .iter()
                .flat_map(|g| &g.path)
                .all(|p| p.x > 70_000_000 && p.y > 70_000_000)
        );
        assert!(
            !scene
                .board_graphics
                .iter()
                .any(|g| g.source_object_uuid == Uuid::nil().to_string())
        );
        assert_eq!(scene.schematic_document, base.schematic_document);
        assert_eq!(scene.source_revision, base.source_revision);
        assert_eq!(edit.source, original.source);
        let mut changed = original.source.as_ref().unwrap().sheets[&sheet].clone();
        changed.ports.get_mut(&port.uuid).unwrap().name = "EXTERNAL".into();
        assert!(!named.source_matches(&changed));
        edit.label.as_mut().unwrap().name.clear();
        let invalid = edit.preview_scene(&base, root).unwrap();
        let old_graphics: Vec<_> = base
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == port.uuid.to_string())
            .collect();
        let invalid_graphics: Vec<_> = invalid
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == port.uuid.to_string())
            .collect();
        assert_eq!(invalid_graphics, old_graphics);
        assert!(edit.cancel().is_some());
        assert!(!edit.active() && edit.named_edit.is_none() && edit.label.is_none());
    }
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}
