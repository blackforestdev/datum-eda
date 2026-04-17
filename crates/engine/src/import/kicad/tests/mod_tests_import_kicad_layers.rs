use crate::error::EngineError;
use crate::import::kicad::parser_helpers::{kicad_layer_name_to_id, resolve_layer_id};

#[test]
fn hardcoded_layer_fallback_is_bounded_to_recognized_names() {
    assert_eq!(kicad_layer_name_to_id("F.Cu"), Some(0));
    assert_eq!(kicad_layer_name_to_id("B.Cu"), Some(31));
    assert_eq!(kicad_layer_name_to_id("F.Paste"), Some(35));
    assert_eq!(kicad_layer_name_to_id("B.Paste"), Some(34));
    assert_eq!(kicad_layer_name_to_id("F.SilkS"), Some(37));
    assert_eq!(kicad_layer_name_to_id("B.SilkS"), Some(36));
    assert_eq!(kicad_layer_name_to_id("F.Mask"), Some(39));
    assert_eq!(kicad_layer_name_to_id("B.Mask"), Some(38));
    assert_eq!(kicad_layer_name_to_id("Edge.Cuts"), Some(44));

    assert_eq!(kicad_layer_name_to_id("In1.Cu"), None);
    assert_eq!(kicad_layer_name_to_id("User.9"), None);
    assert_eq!(kicad_layer_name_to_id(""), None);
}

#[test]
fn resolve_layer_prefers_parsed_table_over_hardcoded_fallback() {
    let mut table = std::collections::HashMap::new();
    table.insert("F.Cu".to_string(), 2);
    table.insert("In1.Cu".to_string(), 4);

    assert_eq!(resolve_layer_id("F.Cu", &table).unwrap(), 2);
    assert_eq!(resolve_layer_id("In1.Cu", &table).unwrap(), 4);
}

#[test]
fn resolve_layer_falls_back_to_hardcoded_when_table_missing_entry() {
    let table = std::collections::HashMap::new();
    assert_eq!(resolve_layer_id("F.Cu", &table).unwrap(), 0);
    assert_eq!(resolve_layer_id("Edge.Cuts", &table).unwrap(), 44);
}

#[test]
fn resolve_layer_errors_explicitly_on_unknown_name() {
    let table = std::collections::HashMap::new();
    let err = resolve_layer_id("Mystery.Layer", &table)
        .expect_err("unknown layer name must fail explicitly, not silently collapse to F.Cu");
    match err {
        EngineError::Import(msg) => {
            assert!(
                msg.contains("Mystery.Layer"),
                "error message must name the unresolved layer; got {msg}"
            );
            assert!(
                msg.contains("unknown"),
                "error message must flag the name as unknown; got {msg}"
            );
        }
        other => panic!("expected EngineError::Import, got {other:?}"),
    }
}
