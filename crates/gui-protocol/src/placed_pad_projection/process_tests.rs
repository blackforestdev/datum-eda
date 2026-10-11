//! Raw preserved authority reaches the scene before historical field defaults.
use super::*;
use eda_engine::substrate::component_pad_source::ComponentPadProjection;
use serde_json::json;
use uuid::Uuid;

#[test]
fn process_capable_raw_preserved_inputs_share_exact_geometry_and_refuse_missing_authority() {
    let id = Uuid::new_v4();
    let pad = json!({"uuid": id, "package": Uuid::new_v4(), "name": "raw preserved",
        "net": null, "position": {"x": 7, "y": 11}, "layer": 30,
        "shape": "circle", "diameter": 101, "mask_layers": [2], "paste_layers": [4],
        "solder_mask_margin_nm": 10, "solder_paste_margin_nm": -2,
        "solder_paste_margin_ratio_ppm": 100000});
    let mut board = json!({"process_source_capability": {"version": 1,
        "front": {"copper_layer": 30, "mask_layer": 2, "paste_layer": 4},
        "back": {"copper_layer": 1, "mask_layer": 8, "paste_layer": 9}},
        "pads": {id.to_string(): pad}});
    let projection = ComponentPadProjection::default();
    let processes = native_processes(&board, &projection).unwrap();
    let mut expected =
        eda_engine::substrate::component_pad_source::process::preserved_placed_process(
            &board["pads"][id.to_string()],
        )
        .unwrap();
    expected.source_capability =
        eda_engine::substrate::process_operations::capability_from_board(&board).unwrap();
    assert_eq!(processes[&id], expected);
    let eda_engine::pool::process::EffectiveProcessAperture::Circle { diameter_nm } =
        expected.effective.paste
    else {
        panic!()
    };
    assert_eq!(
        diameter_nm.a,
        eda_engine::board::nominal_geometry::Rational::new(1071, 10).unwrap()
    );
    board["pads"][id.to_string()]
        .as_object_mut()
        .unwrap()
        .remove("solder_paste_margin_nm");
    assert!(
        native_processes(&board, &projection)
            .unwrap_err()
            .to_string()
            .contains("solder_paste_margin_nm")
    );
    board
        .as_object_mut()
        .unwrap()
        .remove("process_source_capability");
    assert!(native_processes(&board, &projection).unwrap().is_empty());

    let mut missing = ComponentPadProjection::default();
    let physical: eda_engine::board::PlacedPad =
        serde_json::from_value(board["pads"][id.to_string()].clone()).unwrap();
    missing.query_pads.insert(
        id,
        eda_engine::board::query_pad::QueryPadGeometry::Integer(physical),
    );
    board["process_source_capability"] = json!({"version": 1,
        "front": {"copper_layer": 30, "mask_layer": 2, "paste_layer": 4},
        "back": {"copper_layer": 1, "mask_layer": 8, "paste_layer": 9}});
    board["pads"] = json!({});
    let error = native_processes(&board, &missing).unwrap_err().to_string();
    assert!(
        error.contains(&id.to_string()) && error.contains("lacks frozen native process authority")
    );
}
