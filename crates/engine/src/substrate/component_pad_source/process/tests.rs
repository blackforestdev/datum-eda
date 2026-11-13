use super::*;
use serde_json::json;

#[test]
fn preserved_roundrect_process_requires_explicit_ratio_and_keeps_radius() {
    let mut raw = preserved();
    raw["shape"] = json!("round_rect");
    raw["width"] = json!(800000);
    raw["height"] = json!(950000);
    raw["solder_mask_margin_nm"] = json!(0);
    raw["solder_paste_margin_nm"] = json!(0);
    raw["solder_paste_margin_ratio_ppm"] = json!(0);
    assert!(
        preserved_placed_process(&raw)
            .unwrap_err()
            .to_string()
            .contains("roundrect_rratio_ppm")
    );
    raw["roundrect_rratio_ppm"] = json!(200000);
    let result = preserved_placed_process(&raw).unwrap();
    let crate::pool::process::EffectiveProcessAperture::RoundRect { radius_nm, .. } =
        result.effective.paste
    else {
        panic!("rounded paste source must remain rounded")
    };
    assert_eq!(radius_nm.integer_nm().unwrap(), 160000);
    raw["solder_paste_margin_nm"] = json!(1);
    assert!(
        preserved_placed_process(&raw)
            .unwrap_err()
            .to_string()
            .contains("rounded process margins")
    );
}

fn preserved() -> Value {
    json!({"uuid": Uuid::new_v4(), "package": Uuid::new_v4(), "name": "preserved",
        "net": null, "position": {"x": 7, "y": 11}, "layer": 30,
        "shape": "circle", "diameter": 101, "mask_layers": [2], "paste_layers": [3],
        "solder_mask_margin_nm": 10, "solder_paste_margin_nm": -2,
        "solder_paste_margin_ratio_ppm": 100000})
}

#[test]
fn process_preserved_requires_used_raw_inputs_but_empty_membership_suppresses() {
    let original = preserved();
    let resolved = preserved_placed_process(&original).unwrap();
    let crate::pool::process::EffectiveProcessAperture::Circle { diameter_nm } =
        resolved.effective.paste
    else {
        panic!()
    };
    assert_eq!(
        diameter_nm.a,
        crate::board::nominal_geometry::Rational::new(1071, 10).unwrap()
    );
    for field in [
        "shape",
        "diameter",
        "mask_layers",
        "paste_layers",
        "solder_mask_margin_nm",
        "solder_paste_margin_nm",
        "solder_paste_margin_ratio_ppm",
    ] {
        let mut missing = original.clone();
        missing.as_object_mut().unwrap().remove(field);
        let error = preserved_placed_process(&missing).unwrap_err().to_string();
        assert!(error.contains(field), "{field}: {error}");
    }
    let mut suppressed = original;
    suppressed["mask_layers"] = json!([]);
    suppressed["paste_layers"] = json!([]);
    for field in [
        "shape",
        "diameter",
        "solder_mask_margin_nm",
        "solder_paste_margin_nm",
        "solder_paste_margin_ratio_ppm",
    ] {
        suppressed.as_object_mut().unwrap().remove(field);
    }
    let resolved = preserved_placed_process(&suppressed).unwrap();
    assert_eq!(
        resolved.effective.mask,
        crate::pool::process::EffectiveProcessAperture::Suppressed
    );
    assert_eq!(
        resolved.effective.paste,
        crate::pool::process::EffectiveProcessAperture::Suppressed
    );
}
