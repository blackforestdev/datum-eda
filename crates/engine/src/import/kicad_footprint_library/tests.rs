use super::*;
use crate::pool::{PadstackAperture, PadstackMaskPolicy};

const RESISTOR: &str =
    "/usr/share/kicad/footprints/Resistor_SMD.pretty/R_0603_1608Metric.kicad_mod";

#[test]
#[ignore = "real installed KiCad footprint source; run explicitly for source proof"]
fn real_resistor_preserves_rounded_copper_and_exact_courtyard() {
    let converted = convert(Path::new(RESISTOR)).unwrap();
    assert_eq!(converted.footprint.name, "R_0603_1608Metric");
    assert_eq!(converted.footprint.pads.len(), 2);
    for stack in &converted.padstacks {
        assert_eq!(
            stack.aperture,
            Some(PadstackAperture::RoundRect {
                width_nm: 800_000,
                height_nm: 950_000,
                rratio_ppm: 250_000
            })
        );
        assert_eq!(stack.mask_policy, PadstackMaskPolicy::Inherit);
    }
    assert_eq!(converted.footprint.courtyard.vertices.len(), 4);
    let bounds = converted.footprint.courtyard.bounding_box().unwrap();
    assert_eq!(bounds.min.x, -1_480_000);
    assert_eq!(bounds.max.y, 730_000);
    assert!(!converted.footprint.fab.is_empty());
    assert!(converted.footprint.ipc_basis.is_none());
    assert!(converted.warnings.iter().any(|w| w.contains("not loaded")));
    assert_eq!(
        convert(Path::new(RESISTOR)).unwrap().footprint,
        converted.footprint
    );
}

#[test]
fn unsupported_physical_features_refuse_instead_of_silent_replacement() {
    let root =
        std::env::temp_dir().join(format!("datum-footprint-refusal-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("unsafe.kicad_mod");
    for feature in [
        r#"(pad "1" smd custom (at 0 0) (size 1 1) (layers "F.Cu" "F.Mask"))"#,
        r#"(pad "1" smd rect (at 0 0 45) (size 1 2) (layers "F.Cu" "F.Mask"))"#,
        r#"(pad "" np_thru_hole circle (at 0 0) (size 3 3) (drill 3) (layers "*.Cu" "*.Mask"))"#,
        r#"(zone (net 0))"#,
    ] {
        std::fs::write(
            &path,
            format!("(footprint \"Unsafe\" (layer \"F.Cu\") {feature})"),
        )
        .unwrap();
        assert!(convert(&path).is_err(), "feature {feature} must refuse");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn repeated_pad_numbers_have_distinct_physical_identities() {
    let root = std::env::temp_dir().join(format!(
        "datum-footprint-identities-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("repeated.kicad_mod");
    std::fs::write(
        &path,
        r#"(footprint "Repeated" (layer "F.Cu")
      (pad "1" smd rect (at -1 0) (size 1 1) (layers "F.Cu"))
      (pad "1" smd rect (at 1 0) (size 1 1) (layers "F.Cu")))"#,
    )
    .unwrap();
    let converted = convert(&path).unwrap();
    assert_eq!(converted.footprint.pads.len(), 2);
    assert_ne!(converted.padstacks[0].uuid, converted.padstacks[1].uuid);
    assert_eq!(converted.package.terminals.len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn footprint_process_overrides_preserve_zero_and_refuse_partial_inheritance() {
    use crate::pool::PadstackPastePolicy as P;
    let root =
        std::env::temp_dir().join(format!("datum-footprint-process-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("process.kicad_mod");
    let source = |version: &str, global: &str, local: &str| {
        std::fs::write(
            &path,
            format!(
                r#"(footprint "Process" {version} (layer "F.Cu") {global}
          (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Mask" "F.Paste") {local}))"#
            ),
        )
        .unwrap();
        convert(&path)
    };
    let version = "(version 20241229)";
    let absent = source(version, "", "").unwrap();
    assert_eq!(absent.padstacks[0].mask_policy, PadstackMaskPolicy::Inherit);
    assert_eq!(absent.padstacks[0].paste_policy, P::Inherit);
    let zero = source(
        version,
        "",
        "(solder_mask_margin 0) (solder_paste_margin 0) (solder_paste_margin_ratio 0)",
    )
    .unwrap();
    assert_eq!(
        zero.padstacks[0].mask_policy,
        PadstackMaskPolicy::ExpansionNm(0)
    );
    assert_eq!(zero.padstacks[0].paste_policy, P::ExpansionNm(0));
    let inherited = source(
        version,
        "(solder_mask_margin 0.2) (solder_paste_margin 0.1) (solder_paste_ratio 0)",
        "(solder_mask_margin 0)",
    )
    .unwrap();
    assert_eq!(
        inherited.padstacks[0].mask_policy,
        PadstackMaskPolicy::ExpansionNm(0)
    );
    assert_eq!(inherited.padstacks[0].paste_policy, P::ExpansionNm(100_000));
    for (global, local) in [
        ("", "(solder_paste_margin 0)"),
        ("", "(solder_paste_margin_ratio 0)"),
        ("(solder_paste_ratio 0)", ""),
        ("(solder_paste_margin 0)", ""),
        ("(solder_paste_margin 0) (solder_paste_ratio 0.1)", ""),
    ] {
        assert!(
            source(version, global, local).is_err(),
            "unresolved or unsupported pair {global} {local}"
        );
    }
    for legacy in ["", "(version 20221018)", "(version 20990101)"] {
        assert!(
            source(legacy, "", "(solder_mask_margin 0)").is_err(),
            "ambiguous source version {legacy}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn footprint_text_rotation_preserves_normal_angles_and_refuses_narrowing_overflow() {
    let root = std::env::temp_dir().join(format!("datum-footprint-angle-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("angle.kicad_mod");
    let write = |angle: i64| {
        std::fs::write(
            &path,
            format!(
                r#"(footprint "Angle" (layer "F.Cu")
          (property "Reference" "REF**" (at 0 0 {angle}) (layer "F.SilkS"))
          (pad "1" smd circle (at 0 0) (size 1 1) (layers "F.Cu")))"#
            ),
        )
        .unwrap();
    };
    for angle in [-90, 0, 90, 180, 270] {
        write(angle);
        let native = convert(&path).unwrap();
        let rotations: Vec<_> = native
            .footprint
            .silkscreen
            .iter()
            .filter_map(|p| match p {
                crate::pool::Primitive::Text { rotation, .. } => Some(i64::from(*rotation)),
                _ => None,
            })
            .collect();
        assert_eq!(rotations, [angle]);
    }
    for angle in [
        i64::from(i32::MAX) + 1,
        4_294_967_296,
        i64::from(i32::MIN) - 1,
    ] {
        write(angle);
        assert!(
            convert(&path)
                .unwrap_err()
                .to_string()
                .contains("native angle range")
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
