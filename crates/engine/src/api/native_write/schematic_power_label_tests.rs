use super::*;
use crate::api::native_write::{commit_prepared, electrical::topology_tests::fixture};
use crate::connectivity::schematic_net_info;
use crate::schematic::{PinElectricalType, SymbolPin};
use crate::substrate::{
    CommitSource, ElectricalOccurrence, ProjectResolver, electrical_test_partitions,
};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "native power label")
}

fn label(
    model: &mut DesignModel,
    root: &Path,
    sheet: Uuid,
    name: &str,
    kind: &str,
    x: i64,
) -> Uuid {
    let verb =
        super::super::registry::find_native_write_verb("datum.schematic.place_label").unwrap();
    let write = (verb.build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        serde_json::json!({"sheet":sheet,"name":name,"kind":kind,"x_nm":x,"y_nm":0}),
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(model, root, write).unwrap();
    id
}

fn second_sheet(model: &mut DesignModel, root: &Path) -> Uuid {
    let sheet = Uuid::new_v4();
    let write = create_sheet(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        serde_json::json!({"sheet":sheet,"name":"Other"}),
    )
    .unwrap();
    commit_prepared(model, root, write).unwrap();
    sheet
}

fn member(id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: "labels".into(),
        source_id: id,
        instance_path: vec![],
    }
}

#[test]
fn native_power_labels_join_across_sheets_without_collapsing_domains_and_survive_undo_reopen() {
    let (root, mut model, _, a) = fixture("native_power_labels");
    let b = second_sheet(&mut model, &root);
    let gnd_a = label(&mut model, &root, a, "GND", "power", 0);
    let gnd_b = label(&mut model, &root, b, "GND", "power", 0);
    let global = label(&mut model, &root, b, "GND", "global", 100);
    let analog = label(&mut model, &root, a, "GNDA", "power", 200);
    let lower = label(&mut model, &root, a, "gnd", "power", 300);
    let rail = label(&mut model, &root, a, "+3V3", "power", 400);
    let local = label(&mut model, &root, b, "GND", "local", 500);
    let bus = label(&mut model, &root, b, "VBUS", "power", 600);
    let expected = vec![
        BTreeSet::from([member(gnd_a), member(gnd_b), member(global)]),
        BTreeSet::from([member(analog)]),
        BTreeSet::from([member(lower)]),
        BTreeSet::from([member(rail)]),
        BTreeSet::from([member(local)]),
        BTreeSet::from([member(bus)]),
    ];
    let partitions = electrical_test_partitions(&model, &[]).unwrap();
    assert_eq!(partitions.len(), expected.len());
    for group in &expected {
        assert!(partitions.contains(group), "missing {group:?}");
    }
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&model)
            .unwrap()
            .net_ids()
            .len(),
        6
    );
    let nets = schematic_net_info(&schematic(&model).unwrap());
    assert_eq!(nets.len(), 6);
    let gnd = nets
        .iter()
        .find(|net| net.name == "GND" && net.sheets.len() == 2)
        .unwrap();
    assert_eq!(gnd.semantic_class.as_deref(), Some("ground"));
    assert_eq!(
        nets.iter()
            .find(|net| net.name == "VBUS")
            .unwrap()
            .semantic_class
            .as_deref(),
        Some("power")
    );
    let identities = model.electrical_identities.clone();
    let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.electrical_identities, identities);
    assert_eq!(schematic(&reopened).unwrap(), schematic(&model).unwrap());
    reopened
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let mut undone = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        !schematic(&undone).unwrap().sheets[&b]
            .labels
            .contains_key(&bus)
    );
    assert_eq!(electrical_test_partitions(&undone, &[]).unwrap().len(), 5);
    undone
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let restored = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(restored.electrical_identities, identities);
    assert_eq!(schematic(&restored).unwrap(), schematic(&model).unwrap());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_label_params_reject_invalid_input_and_preserve_name_and_kind() {
    let (root, mut model, _, sheet) = fixture("native_label_params");
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let base = serde_json::json!({"sheet":sheet,"name":"VCC","x_nm":0,"y_nm":0});
    for (field, value) in [
        ("name", serde_json::json!("  ")),
        ("name", serde_json::json!("VCC\nGND")),
        ("kind", serde_json::json!("driver")),
        ("sheet", serde_json::json!(Uuid::new_v4())),
        ("label", serde_json::json!(Uuid::new_v4())),
    ] {
        let mut params = base.clone();
        params[field] = value;
        assert!(place_label(&context, provenance(), params).is_err());
    }
    for (index, kind) in [
        None,
        Some(serde_json::Value::Null),
        Some(serde_json::json!("hier")),
    ]
    .into_iter()
    .enumerate()
    {
        let mut params = base.clone();
        params["x_nm"] = serde_json::json!(index as i64 * 100);
        params["name"] = serde_json::json!(" Rail ");
        if let Some(kind) = kind {
            params["kind"] = kind;
        }
        let write = place_label(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            params,
        )
        .unwrap();
        let id = write.primary_object_id.unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        let source = schematic(&model).unwrap();
        assert_eq!(source.sheets[&sheet].labels[&id].name, " Rail ");
        assert_eq!(
            source.sheets[&sheet].labels[&id].kind,
            if index == 2 {
                LabelKind::Hierarchical
            } else {
                LabelKind::Local
            }
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

fn pin_symbol(reference: &str, kind: PinElectricalType) -> PlacedSymbol {
    PlacedSymbol {
        uuid: Uuid::new_v4(),
        part: None,
        entity: None,
        gate: None,
        lib_id: Some("Device:IC".into()),
        reference: reference.into(),
        value: "IC".into(),
        fields: vec![],
        pins: vec![SymbolPin {
            library_pin: None,
            uuid: Uuid::new_v4(),
            number: "1".into(),
            name: "VCC".into(),
            electrical_type: kind,
            position: Point::new(0, 0),
        }],
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
    }
}

#[test]
fn native_power_label_does_not_drive_a_net_but_connects_real_power_source_across_sheets() {
    let (root, mut model, _, sink_sheet) = fixture("native_power_source");
    let source_sheet = second_sheet(&mut model, &root);
    for (sheet, reference, kind) in [
        (sink_sheet, "U1", PinElectricalType::PowerIn),
        (source_sheet, "U2", PinElectricalType::PowerOut),
    ] {
        let write = build_place_schematic_symbol(
            &model,
            provenance(),
            sheet,
            &pin_symbol(reference, kind),
            None,
        )
        .unwrap();
        let write = prepare_native_birth(&model, write).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    label(&mut model, &root, sink_sheet, "VCC", "power", 0);
    label(&mut model, &root, source_sheet, "VDD", "power", 0);
    let findings = crate::erc::run_prechecks(&schematic(&model).unwrap());
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "power_in_without_source"
                && finding.net_name.as_deref() == Some("VCC"))
    );
    // A source on a distinct domain is not permission to suppress the finding.
    label(&mut model, &root, source_sheet, "VCC", "power", 0);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    let findings = crate::erc::run_prechecks(&schematic(&reopened).unwrap());
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "power_in_without_source")
    );
    std::fs::remove_dir_all(root).unwrap();
}
