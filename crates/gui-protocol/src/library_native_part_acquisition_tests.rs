use super::*;
use crate::library_browser::acquisition::{AcquisitionSource, NativePartAcquisitionPreview};
use serde_json::json;

fn ready() -> LibraryAcquisition {
    let id = Uuid::new_v4();
    let mut a = LibraryAcquisition::begin("target revision".into(), None);
    a.use_native_part_source();
    a.selected = Some(id.to_string());
    a.selected_source = Some("/source/native-project".into());
    a.preview = Some(LibraryAcquisitionPreview {
        name: "Maker reusable 10k".into(),
        source_path: "/source/native-project".into(),
        source_digest: "sha256:snapshot".into(),
        unit: Value::Null,
        symbol: json!({"uuid":Uuid::new_v4()}),
        footprint: None,
        native_part: Some(NativePartAcquisitionPreview {
            part: json!({"uuid":id}),
            entity: Value::Null,
            package: Value::Null,
            footprint: Value::Null,
            pin_pad_map: Value::Null,
            gate_count: 2,
            object_count: 10,
            padstacks: vec![],
            gate_symbols: vec![],
        }),
        notices: vec![],
    });
    a
}
#[test]
fn native_part_request_uses_identity_not_display_or_gate_symbol() {
    let mut a = ready();
    let id = a.selected_native_part().unwrap();
    let r = a.request().unwrap();
    assert_eq!(r.source_format, "datum_native_part");
    assert_eq!(r.symbol_name, id.to_string());
    a.preview
        .as_mut()
        .unwrap()
        .native_part
        .as_mut()
        .unwrap()
        .part["uuid"] = json!(id.to_string().to_uppercase());
    assert_eq!(a.request().unwrap().symbol_name, id.to_string());
    a.preview.as_mut().unwrap().name = "Same Part with another display label".into();
    assert_eq!(a.request().unwrap().symbol_name, id.to_string());
    a.placement_pane = Some(crate::PaneId(1));
    assert_eq!(a.selected_native_part(), Some(id));
    assert_eq!(a.request().unwrap().source_format, "datum_native_part");
    let [gate, unit, pin, pad] = std::array::from_fn(|_| Uuid::new_v4());
    let physical = a.preview.as_mut().unwrap().native_part.as_mut().unwrap();
    physical.entity = json!({"gates":{gate.to_string():{"name":"A","unit":unit}}});
    physical.footprint = json!({"pads":{pad.to_string():{"name":"1"}}});
    physical.pin_pad_map = json!({"mappings":{pad.to_string():{"gate":gate,"pin":pin}}});
    physical.gate_symbols = vec![(
        "A".into(),
        json!({"uuid":unit,"pins":{pin.to_string():{"name":"Signal"}}}),
        Value::Null,
    )];
    assert_eq!(
        physical.mapping_labels().unwrap(),
        vec!["Pad 1 → A / Signal"]
    );
    physical.pin_pad_map["mappings"][pad.to_string()]["pin"] = json!(Uuid::new_v4());
    assert!(physical.mapping_labels().is_err());
}
#[test]
fn native_part_source_mode_selection_and_loading_retire_stale_request() {
    let mut a = ready();
    a.loading = true;
    assert!(a.request().is_err());
    assert!(a.selected_native_part().is_none());
    a.loading = false;
    a.selected = Some(Uuid::new_v4().to_string());
    assert!(a.request().is_err());
    assert!(a.selected_native_part().is_none());
    a = ready();
    a.source = AcquisitionSource::KiCadSymbols;
    assert!(a.request().is_err());
    assert!(a.selected_native_part().is_none());
    a = ready();
    a.selected_source = Some("/another/project".into());
    assert!(a.request().is_err());
    a = ready();
    a.source_changed();
    assert!(a.request().is_err());
    assert!(a.selected_native_part().is_none());
}
