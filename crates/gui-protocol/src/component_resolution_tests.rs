//! Scalar authoring-input admission only; these are not CAD fixtures or workflow proof.
use eda_engine::api::native_write::library_definition::{
    part_display_identity, validate_part_identity,
};
use serde_json::json;

#[test]
fn unspecified_procurement_requires_explicit_authored_name() {
    let unnamed = json!({"manufacturer":"", "mpn":"", "name":"  "});
    assert!(validate_part_identity(&unnamed).is_err());
    let named = json!({"manufacturer":"", "mpn":"", "name":" Authored name "});
    assert!(validate_part_identity(&named).is_ok());
    assert_eq!(
        part_display_identity(&named).as_deref(),
        Some("Authored name")
    );
    assert!(validate_part_identity(&json!({"manufacturer":"", "mpn":"", "name":7})).is_err());
    assert!(
        validate_part_identity(&json!({"manufacturer":null, "mpn":"", "name":"Authored name"}))
            .is_err()
    );
    let orderable = json!({"manufacturer":"Maker", "mpn":"Order code", "name":"Authored name"});
    assert!(validate_part_identity(&orderable).is_ok());
    assert_eq!(
        part_display_identity(&orderable).as_deref(),
        Some("Maker Order code")
    );
}
