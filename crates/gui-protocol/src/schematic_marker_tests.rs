//! Marker ghosts and requests use one immutable native source sheet.
use super::*;
use crate::schematic_authoring::MarkerKind;
use eda_engine::schematic::{LabelKind, NetLabel};

fn marker_model(root: &std::path::Path) -> eda_engine::substrate::DesignModel {
    use eda_engine::api::native_write::{
        commit_prepared,
        context::WriteProvenance,
        registry::{NativeWriteContext, find_native_write_verb},
    };
    let mut model = eda_engine::substrate::ProjectResolver::new(root)
        .resolve()
        .unwrap();
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let library = authoring
        .choices
        .iter()
        .find(|choice| choice.name == "Datum single op amp symbol")
        .unwrap()
        .id;
    // The retained U1/U2 fixture intentionally shares pin coordinates. Place a
    // real library symbol separately so the positive NC case has one exact pin.
    let write = (find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(
        &NativeWriteContext { model: &model, project_root: root },
        WriteProvenance::new("test", eda_engine::substrate::CommitSource::Test, "marker pin target"),
        serde_json::json!({"sheet":authoring.sheet.unwrap(),"library_symbol":library,"reference":"U3","value":"Marker target","x_nm":40_000_000,"y_nm":40_000_000}),
    ).unwrap();
    commit_prepared(&mut model, root, write).unwrap();
    model
}

#[test]
fn marker_preview_request_cancel_preserves_source_and_exact_pin_identity() {
    let fixture = isolated_fixture();
    let root = &fixture.0;
    let model = marker_model(root);
    let original = SchematicAuthoring::from_model(&model).unwrap();
    let sheet = original.sheet.unwrap();
    let source = original.source.as_ref().unwrap();
    let (symbol, pin) = source.sheets[&sheet]
        .symbols
        .values()
        .flat_map(|s| s.pins.iter().map(move |pin| (s.uuid, pin)))
        .find(|(_, pin)| {
            source.sheets[&sheet]
                .symbols
                .values()
                .flat_map(|s| &s.pins)
                .filter(|other| other.position == pin.position)
                .count()
                == 1
        })
        .unwrap();
    let at = PointNm {
        x: pin.position.x,
        y: pin.position.y,
    };
    let base = crate::native_schematic_scene::project_native_schematic_scene(&model, root)
        .unwrap()
        .unwrap();
    let bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    for kind in [MarkerKind::Junction, MarkerKind::NoConnect] {
        let mut authoring = original.clone();
        authoring.marker = Some(kind);
        authoring.position = Some(at);
        authoring.base = Some(base.clone());
        assert!(authoring.active());
        assert!(!authoring.transform_placement(Rotate));
        let preview = authoring.preview_scene(&base, root).unwrap();
        let graphics: Vec<_> = preview
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == Uuid::nil().to_string())
            .collect();
        assert!(!graphics.is_empty(), "marker has real projected ink");
        let points: Vec<_> = graphics.iter().flat_map(|g| &g.path).collect();
        assert_eq!(
            points.iter().map(|p| p.x).min().unwrap() + points.iter().map(|p| p.x).max().unwrap(),
            2 * at.x
        );
        assert_eq!(
            points.iter().map(|p| p.y).min().unwrap() + points.iter().map(|p| p.y).max().unwrap(),
            2 * at.y
        );
        assert_eq!(preview.source_revision, base.source_revision);
        assert_eq!(preview.schematic_document, base.schematic_document);
        assert_eq!(authoring.source, original.source);
        let (verb, params) = authoring.marker_request(at).unwrap().verb_params();
        assert_eq!(verb, "datum.schematic.place_marker");
        assert_eq!(params["kind"], kind.token());
        assert_eq!(params["sheet"], sheet.to_string());
        assert_eq!(
            (params["x_nm"].as_i64(), params["y_nm"].as_i64()),
            (Some(at.x), Some(at.y))
        );
        if kind == MarkerKind::NoConnect {
            assert_eq!(params["symbol"], symbol.to_string());
            assert_eq!(params["pin"], pin.uuid.to_string());
            assert_eq!(authoring.pin_point(&pin.uuid.to_string()), Some(at));
        } else {
            assert!(params["symbol"].is_null() && params["pin"].is_null());
        }
        authoring.pending_request = Some(authoring.marker_request(at).unwrap());
        assert!(authoring.cancel().is_some());
        assert!(!authoring.active());
        assert!(
            authoring.marker.is_none()
                && authoring.position.is_none()
                && authoring.pending_request.is_none()
        );
        assert_eq!(authoring.source, original.source);
    }
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}

#[test]
fn noconnect_rejects_off_pin_and_coincident_pins_and_never_picks_power_labels() {
    let fixture = isolated_fixture();
    let model = marker_model(&fixture.0);
    let mut authoring = SchematicAuthoring::from_model(&model).unwrap();
    authoring.marker = Some(MarkerKind::NoConnect);
    let sheet_id = authoring.sheet.unwrap();
    let background = PointNm {
        x: i64::MAX / 2,
        y: i64::MAX / 2,
    };
    assert!(authoring.marker_request(background).is_err());
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        kind: LabelKind::Power,
        name: "GND".into(),
        position: eda_engine::ir::geometry::Point::new(background.x, background.y),
    };
    authoring
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&sheet_id)
        .unwrap()
        .labels
        .insert(label.uuid, label.clone());
    assert!(authoring.pin_point(&label.uuid.to_string()).is_none());
    assert!(authoring.marker_request(background).is_err());
    let sheet = authoring
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&sheet_id)
        .unwrap();
    let symbol = sheet
        .symbols
        .values_mut()
        .find(|s| s.reference == "U3")
        .unwrap();
    let mut coincident = symbol.pins[0].clone();
    let at = PointNm {
        x: coincident.position.x,
        y: coincident.position.y,
    };
    let original_pin = coincident.uuid;
    coincident.uuid = Uuid::new_v4();
    let duplicate = coincident.clone();
    let symbol_id = symbol.uuid;
    assert!(
        authoring.marker_request(at).is_ok(),
        "negative control begins with one valid exact pin"
    );
    authoring
        .source
        .as_mut()
        .unwrap()
        .sheets
        .get_mut(&sheet_id)
        .unwrap()
        .symbols
        .get_mut(&symbol_id)
        .unwrap()
        .pins
        .push(duplicate);
    assert!(
        authoring
            .marker_pin_point(&original_pin.to_string())
            .is_some()
    );
    assert!(
        authoring
            .marker_request(at)
            .unwrap_err()
            .contains("unambiguous")
    );
    authoring.marker = Some(MarkerKind::Junction);
    assert!(authoring.marker_request(at).is_ok());
    authoring.marker = None;
    assert!(authoring.marker_request(at).is_err());
}

#[test]
fn marker_request_binds_to_focused_child_occurrence() {
    let fixture = isolated_fixture();
    let model = marker_model(&fixture.0);
    let mut authoring = SchematicAuthoring::from_model(&model).unwrap();
    let root_sheet = authoring.sheet.unwrap();
    let child_sheet = Uuid::new_v4();
    let instance = Uuid::new_v4();
    {
        let source = authoring.source.as_mut().unwrap();
        let mut sheet = source.sheets[&root_sheet].clone();
        sheet.uuid = child_sheet;
        source.sheets.insert(child_sheet, sheet);
    }
    authoring.marker = Some(crate::schematic_authoring::MarkerKind::Junction);
    let at = PointNm { x: 0, y: 0 };
    // Without a circuit basis the request still binds to the primary sheet
    // with the empty root occurrence path.
    let (_, params) = authoring.marker_request(at).unwrap().verb_params();
    assert_eq!(params["sheet"], root_sheet.to_string());
    assert_eq!(params["instance_path"], serde_json::json!([]));
    // With the focused child basis, sheet and exact occurrence path follow it.
    authoring.circuit_basis = Some(crate::AuthoringViewBasis {
        project: Uuid::new_v4(),
        revision: "test-source".into(),
        pane: crate::PaneId(1),
        content: crate::PaneContent::Schematic,
        source: child_sheet,
        document: Some(crate::schematic_documents::SchematicDocumentKey {
            project: Uuid::new_v4(),
            schematic: Uuid::new_v4(),
            source_sheet: child_sheet,
            instance_path: vec![instance],
        }),
    });
    let (verb, params) = authoring.marker_request(at).unwrap().verb_params();
    assert_eq!(verb, "datum.schematic.place_marker");
    assert_eq!(params["sheet"], child_sheet.to_string());
    assert_eq!(params["instance_path"], serde_json::json!([instance]));
    // No-connect pin identity is resolved on the focused sheet, and pin
    // pickup follows the same basis rather than the primary sheet.
    authoring.marker = Some(crate::schematic_authoring::MarkerKind::NoConnect);
    let source = authoring.source.as_ref().unwrap();
    let (pin_at, pin_id) = source.sheets[&child_sheet]
        .symbols
        .values()
        .flat_map(|s| s.pins.iter())
        .map(|pin| {
            (
                PointNm {
                    x: pin.position.x,
                    y: pin.position.y,
                },
                pin.uuid,
            )
        })
        .find(|(at, _)| {
            source.sheets[&child_sheet]
                .symbols
                .values()
                .flat_map(|s| &s.pins)
                .filter(|p| p.position.x == at.x && p.position.y == at.y)
                .count()
                == 1
        })
        .unwrap();
    let (_, params) = authoring.marker_request(pin_at).unwrap().verb_params();
    assert_eq!(params["sheet"], child_sheet.to_string());
    assert_eq!(params["instance_path"], serde_json::json!([instance]));
    assert_eq!(params["pin"], pin_id.to_string());
    assert_eq!(
        authoring.marker_pin_point(&pin_id.to_string()),
        Some(pin_at)
    );
}

#[path = "schematic_connectivity_move_tests.rs"]
mod connectivity_move_tests;
