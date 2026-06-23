use super::*;
use std::collections::BTreeMap;

fn write_temp_footprint(name: &str, contents: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "datum-kicad-footprint-test-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp dir should exist");
    let path = root.join(name);
    std::fs::write(&path, contents).expect("footprint should write");
    path
}

#[test]
fn imports_kicad_footprint_into_package_and_padstacks() {
    let path = write_temp_footprint(
        "demo.kicad_mod",
        r#"(footprint "R_0805_2012Metric"
  (layer "F.Cu")
  (property "Reference" "R?" (at 0 -1.65 0) (layer "F.SilkS"))
  (fp_line (start -1 -0.8) (end 1 -0.8) (layer "F.SilkS") (width 0.12))
  (fp_rect (start -1.5 -0.9) (end 1.5 0.9) (layer "F.CrtYd") (width 0.05))
  (pad "1" smd roundrect (at -0.9 0) (size 1 1.4) (layers "F.Cu" "F.Paste" "F.Mask"))
  (pad "2" smd roundrect (at 0.9 0) (size 1 1.4) (layers "F.Cu" "F.Paste" "F.Mask"))
)"#,
    );

    let (imported, report) = import_footprint_document(&path).expect("footprint should import");
    assert_eq!(report.kind, ImportKind::KiCadFootprint);
    assert_eq!(imported.package.name, "R_0805_2012Metric");
    assert!(report.metadata["source_hash"].starts_with("sha256:"));
    assert_eq!(imported.package.pads.len(), 2);
    assert_eq!(imported.padstacks.len(), 2);
    assert!(!imported.package.silkscreen.is_empty());
    assert!(!imported.mechanical.is_empty());
    assert!(!imported.package.courtyard.vertices.is_empty());
}

#[test]
fn imports_kicad_footprint_reuses_import_map_package_identity() {
    let path = write_temp_footprint(
        "mapped.kicad_mod",
        r#"(footprint "MappedFootprint"
  (layer "F.Cu")
  (fp_line (start -1 -0.8) (end 1 -0.8) (layer "F.SilkS") (width 0.12))
  (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Paste" "F.Mask"))
)"#,
    );
    let import_key = footprint_package_import_key(&path);
    let mapped_uuid = uuid::Uuid::new_v4();
    let mut import_map = BTreeMap::new();
    import_map.insert(
        import_key.clone(),
        crate::substrate::ImportMapEntry {
            import_key: import_key.clone(),
            object_id: mapped_uuid,
            source_shard_id: uuid::Uuid::new_v4(),
            source_tool: "kicad".to_string(),
            source_path: path.display().to_string(),
            source_object_ref: import_key.clone(),
            source_hash: "sha256:test".to_string(),
        },
    );

    let (imported, report) =
        import_footprint_document_with_import_map(&path, &import_map).expect("footprint imports");

    assert_eq!(imported.package.uuid, mapped_uuid);
    assert_eq!(report.metadata["import_key"], import_key);
    assert!(report.metadata["source_hash"].starts_with("sha256:"));
    assert_eq!(report.metadata["reused_existing_identity"], "true");
}

#[test]
fn imports_kicad_footprint_allocates_deterministic_identity_for_new_import_key() {
    let path = write_temp_footprint(
        "new.kicad_mod",
        r#"(footprint "NewFootprint"
  (layer "F.Cu")
  (fp_line (start -1 -0.8) (end 1 -0.8) (layer "F.SilkS") (width 0.12))
  (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Paste" "F.Mask"))
)"#,
    );

    let (first, first_report) = import_footprint_document(&path).expect("first import");
    let (second, second_report) = import_footprint_document(&path).expect("second import");

    assert_eq!(first.package.uuid, second.package.uuid);
    assert_eq!(
        first_report.metadata["import_key"],
        footprint_package_import_key(&path)
    );
    assert_eq!(first_report.metadata["reused_existing_identity"], "false");
    assert_eq!(second_report.metadata["reused_existing_identity"], "false");
}

#[test]
fn imports_kicad_footprint_drilled_padstack() {
    let path = write_temp_footprint(
        "header.kicad_mod",
        r#"(footprint "PinHeader_1x03_P2.54mm_Vertical"
  (layer "F.Cu")
  (fp_circle (center 0 0) (end 1.2 0) (layer "F.SilkS") (width 0.12))
  (pad "1" thru_hole circle (at 0 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask"))
)"#,
    );

    let (imported, _report) = import_footprint_document(&path).expect("footprint should import");
    assert_eq!(imported.package.pads.len(), 1);
    assert_eq!(imported.padstacks.len(), 1);
    assert_eq!(imported.padstacks[0].drill_nm, Some(1_000_000));
}

#[test]
fn imports_kicad_footprint_texts_on_uppercase_silkscreen_layers() {
    let path = write_temp_footprint(
        "uppercase-silk.kicad_mod",
        r#"(footprint "UppercaseSilk"
  (layer "F.Cu")
  (property "Reference" "U?" (at 0 -1.65 0) (layer "F.SILKS"))
  (property "Value" "UppercaseSilk" (at 0 1.65 0) (layer "F.SILKS"))
  (fp_line (start -1 -0.8) (end 1 -0.8) (layer "F.SILKS") (width 0.12))
  (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Paste" "F.Mask"))
)"#,
    );

    let (imported, _report) = import_footprint_document(&path).expect("footprint should import");
    let silk_texts = imported
        .package
        .silkscreen
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(silk_texts.contains(&"U?"));
    assert!(silk_texts.contains(&"UppercaseSilk"));
}
