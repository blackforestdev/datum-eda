use super::*;
use crate::library_browser::acquisition::{AcquisitionSource, NativePartAcquisitionPreview};
use serde_json::json;

#[test]
fn native_component_search_uses_shared_declared_metadata_and_preserves_availability_and_identity() {
    use eda_engine::api::native_write::library_acquisition::native_part::NativePartChoice;
    let [available, refused] = std::array::from_fn(|_| Uuid::new_v4());
    let path = std::path::Path::new("/saved/native/library");
    let catalog = LibraryAcquisition::native_part_catalog(
        path,
        vec![
            NativePartChoice {
                id: available,
                label: "Maker RX10 · 10k".into(),
                search_text: "Precision resistor\n0603\npassive".into(),
                refusal: None,
            },
            NativePartChoice {
                id: refused,
                label: "Maker RX20 · 20k".into(),
                search_text: "Precision resistor\n0805\npassive".into(),
                refusal: Some("Incomplete physical mapping".into()),
            },
        ],
    );
    let mut a = LibraryAcquisition::begin("rev".into(), None);
    a.use_native_part_source();
    a.names = catalog.symbols.iter().map(|s| s.label.clone()).collect();
    a.sources = catalog.symbols;
    for query in ["RESISTOR", "0603", "passive", "Maker", "RX10", "10k"] {
        a.search = query.into();
        assert_eq!(a.results(), vec![0], "Query {query}");
        let source = &a.sources[a.results()[0]];
        assert_eq!(source.name, available.to_string());
        assert_eq!(source.path, path);
    }
    a.search = "0805".into();
    assert!(a.results().is_empty(), "Unavailable components stay hidden");
    a.include_unavailable = true;
    assert_eq!(a.results(), vec![1]);
    assert_eq!(a.sources[1].name, refused.to_string());
    assert!(a.sources[1].is_unavailable());
    assert!(
        a.sources[1]
            .description
            .contains("Incomplete physical mapping")
    );
}

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
