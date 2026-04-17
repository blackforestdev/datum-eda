use super::*;

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
    assert_eq!(imported.package.pads.len(), 2);
    assert_eq!(imported.padstacks.len(), 2);
    assert!(!imported.package.silkscreen.is_empty());
    assert!(!imported.mechanical.is_empty());
    assert!(!imported.package.courtyard.vertices.is_empty());
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
