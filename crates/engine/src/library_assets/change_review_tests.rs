use super::*;
use crate::library_assets::{
    AssetDigest, AssetProvenance, AssetRevision, LocalCounters, closure_tests::provenance,
};
use serde_json::json;
use uuid::Uuid;

fn record(
    kind: AssetKind,
    raw: Vec<u8>,
    dependencies: Vec<Dependency>,
    provenance: AssetProvenance,
) -> (VerifiedAsset, Vec<u8>) {
    let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let revision = AssetRevision {
        schema_version: 1,
        asset_id: serde_json::from_value(value["uuid"].clone()).unwrap(),
        kind,
        payload_schema_version: 1,
        payload_ref: AssetDigest::blob(&raw),
        parents: vec![],
        dependencies,
        provenance_ref: provenance.fingerprint().unwrap(),
        extensions: None,
    };
    let revision_bytes = encode_metadata(&revision).unwrap();
    let provenance_bytes = encode_metadata(&provenance).unwrap();
    let limit = raw.len() + revision_bytes.len() + provenance_bytes.len();
    let asset = VerifiedAsset::read(
        &revision.reference().unwrap(),
        &revision_bytes,
        &provenance_bytes,
        &raw,
        limit,
        LocalCounters::default(),
    )
    .unwrap();
    (asset, raw)
}
fn input(pair: &(VerifiedAsset, Vec<u8>)) -> ReviewInput<'_> {
    ReviewInput {
        asset: &pair.0,
        raw_payload: pair.0.payload().raw_bytes(),
    }
}
fn unit(extra: &str) -> Vec<u8> {
    format!(r#"{{"schema_version":1,"uuid":"00000000-0000-0000-0000-000000000001","name":"Unit","manufacturer":"","pins":{{}},"tags":[]{extra}}}"#).into_bytes()
}

#[test]
fn exact_unknown_precision_is_visible_even_when_parsed_values_round_equal() {
    let before = record(
        AssetKind::Unit,
        unit(r#", "vendor":0.10000000000000001"#),
        vec![],
        provenance(),
    );
    let after = record(
        AssetKind::Unit,
        unit(r#", "vendor":0.1"#),
        vec![],
        provenance(),
    );
    assert_eq!(before.0.payload().value(), after.0.payload().value());
    let review = compare(Some(input(&before)), Some(input(&after))).unwrap();
    assert!(review.payload_semantics_changed && review.payload_bytes_changed);
    let field = review.fields.iter().find(|c| c.path == ["vendor"]).unwrap();
    assert_eq!(field.category, ChangeCategory::Unclassified);
    assert_ne!(field.before, field.after);
    assert_eq!(review.checks, CheckCoverage::NotEvaluated);
}

#[test]
fn byte_only_and_equivalent_decimal_edits_are_disclosed_without_false_engineering_change() {
    let before = record(
        AssetKind::Unit,
        unit(r#", "vendor":0.10"#),
        vec![],
        provenance(),
    );
    let after = record(
        AssetKind::Unit,
        unit(r#", "vendor":1e-1"#),
        vec![],
        provenance(),
    );
    let review = compare(Some(input(&before)), Some(input(&after))).unwrap();
    assert!(review.payload_bytes_changed);
    assert!(!review.payload_semantics_changed);
    assert!(review.fields.is_empty());
    assert_ne!(review.before, review.after);
}

#[test]
fn geometry_units_mapping_rows_and_unknown_counter_names_are_not_hidden() {
    let stack = |diameter| {
        serde_json::to_vec(&json!({"schema_version":1,"uuid":Uuid::from_u128(2),
            "name":"Copper", "aperture":{"circle":{"diameter_nm":diameter,"width_nm":diameter}},
        "vendor":{"object_revision":diameter}}))
        .unwrap()
    };
    let before = record(AssetKind::Padstack, stack(1000000), vec![], provenance());
    let after = record(AssetKind::Padstack, stack(1100000), vec![], provenance());
    let review = compare(Some(input(&before)), Some(input(&after))).unwrap();
    let copper = review
        .fields
        .iter()
        .find(|c| c.path == ["aperture", "circle", "diameter_nm"])
        .unwrap();
    assert_eq!(copper.category, ChangeCategory::Geometry);
    assert_eq!(copper.unit, Some(ReviewUnit::Nanometres));
    assert_eq!(copper.before.as_deref(), Some("1e6"));
    let unknown = review
        .fields
        .iter()
        .find(|c| c.path == ["vendor", "object_revision"])
        .unwrap();
    assert_eq!(unknown.category, ChangeCategory::Unclassified);
    assert_eq!(unknown.unit, None);
    let unknown_dimension = review
        .fields
        .iter()
        .find(|c| c.path == ["aperture", "circle", "width_nm"])
        .unwrap();
    assert_eq!(unknown_dimension.category, ChangeCategory::Unclassified);
    assert_eq!(unknown_dimension.unit, None);
    let map = |pin| {
        serde_json::to_vec(&json!({"schema_version":1,"uuid":Uuid::from_u128(3),
        "part":Uuid::from_u128(4),"mappings":{Uuid::from_u128(5).to_string():{
            "gate":Uuid::from_u128(6),"pin":pin}}}))
        .unwrap()
    };
    let before = record(
        AssetKind::PinPadMap,
        map(Uuid::from_u128(7)),
        vec![],
        provenance(),
    );
    let after = record(
        AssetKind::PinPadMap,
        map(Uuid::from_u128(8)),
        vec![],
        provenance(),
    );
    let review = compare(Some(input(&before)), Some(input(&after))).unwrap();
    assert_eq!(review.fields.len(), 1);
    assert_eq!(review.fields[0].category, ChangeCategory::Mapping);
    assert_eq!(review.fields[0].path.last().unwrap(), "pin");
    assert_ne!(review.fields[0].before, review.fields[0].after);
}

#[test]
fn dependency_repin_and_common_provenance_have_separate_exact_changes() {
    let first_unit = record(AssetKind::Unit, unit(""), vec![], provenance());
    let second_unit = record(
        AssetKind::Unit,
        unit(r#", "rating":"reviewed""#),
        vec![],
        provenance(),
    );
    let raw = serde_json::to_vec(&json!({"schema_version":1,"uuid":Uuid::from_u128(10),
        "name":"Body","unit":first_unit.0.reference().asset_id,
        "standards_basis":"Recorded basis","check_state":{"status":"passed"}}))
    .unwrap();
    let dep = |unit: &VerifiedAsset| Dependency {
        role: "unit".into(),
        payload_path: vec!["unit".into()],
        target: unit.reference().clone(),
    };
    let before = record(
        AssetKind::Symbol,
        raw.clone(),
        vec![dep(&first_unit.0)],
        provenance(),
    );
    let mut attribution = provenance();
    attribution.change_summary = Some("Describe a reviewed change".into());
    let after = record(
        AssetKind::Symbol,
        raw,
        vec![dep(&second_unit.0)],
        attribution,
    );
    let review = compare(Some(input(&before)), Some(input(&after))).unwrap();
    assert!(!review.payload_bytes_changed && !review.payload_semantics_changed);
    assert_eq!(review.dependencies.len(), 1);
    assert!(review.engineering_basis_changed);
    assert_eq!(
        review.dependencies[0].before.as_ref(),
        Some(first_unit.0.reference())
    );
    assert_eq!(
        review.dependencies[0].after.as_ref(),
        Some(second_unit.0.reference())
    );
    assert!(
        review
            .provenance
            .iter()
            .any(|c| c.path == ["change_summary"])
    );
    assert_eq!(review.checks, CheckCoverage::NotEvaluated);
    // A recorded Passed assertion remains source content, never executed checks.
    let added = compare(None, Some(input(&after))).unwrap();
    assert_eq!(added.comparison, ReviewComparison::FirstManagedRecord);
    assert!(
        added
            .fields
            .iter()
            .any(|c| c.category == ChangeCategory::RecordedCheckClaim)
    );
    assert!(
        added
            .fields
            .iter()
            .any(|c| c.category == ChangeCategory::CheckBasis)
    );
    assert_eq!(added.checks, CheckCoverage::NotEvaluated);
}

#[test]
fn missing_null_arrays_and_foreign_or_substituted_inputs_are_explicit() {
    let before = record(
        AssetKind::Unit,
        unit(r#", "vendor":null, "ordered":["a","b"]"#),
        vec![],
        provenance(),
    );
    let after = record(
        AssetKind::Unit,
        String::from_utf8(unit(r#", "ordered":["b","a"]"#))
            .unwrap()
            .replace("\"name\":\"Unit\"", "\"name\":\"Renamed unit\"")
            .into_bytes(),
        vec![],
        provenance(),
    );
    let review = compare(Some(input(&before)), Some(input(&after))).unwrap();
    assert!(review.fields.iter().any(|c| c.path == ["name"]
        && c.category == ChangeCategory::Property
        && c.after.as_deref() == Some("\"Renamed unit\"")));
    let missing = review.fields.iter().find(|c| c.path == ["vendor"]).unwrap();
    assert_eq!(missing.before.as_deref(), Some("null"));
    assert_eq!(missing.after, None);
    assert_eq!(
        review
            .fields
            .iter()
            .filter(|c| c.path.first().map(String::as_str) == Some("ordered"))
            .count(),
        2
    );
    assert!(
        compare(
            Some(ReviewInput {
                asset: &before.0,
                raw_payload: &after.1
            }),
            Some(input(&after))
        )
        .is_err()
    );
    let foreign_raw = String::from_utf8(unit("")).unwrap().replace(
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000099",
    );
    let foreign = record(
        AssetKind::Unit,
        foreign_raw.into_bytes(),
        vec![],
        provenance(),
    );
    assert!(compare(Some(input(&before)), Some(input(&foreign))).is_err());
    assert!(compare(None, None).is_err());
    let different_kind = record(AssetKind::Padstack, unit(""), vec![], provenance());
    assert!(compare(Some(input(&before)), Some(input(&different_kind))).is_err());
    let removed = compare(Some(input(&before)), None).unwrap();
    assert!(removed.payload_semantics_changed && removed.after.is_none());
    assert!(removed.fields.iter().all(|c| c.after.is_none()));
}

#[test]
fn process_basis_changes_use_the_shared_schema_without_claiming_profile_checks() {
    let raw = |selector| {
        serde_json::to_vec(&json!({"schema_version":1,"uuid":Uuid::from_u128(20),
        "name":"Land pattern","package":Uuid::from_u128(21),"pads":{},
        "courtyard":{"vertices":[],"closed":true},"process_basis":selector}))
        .unwrap()
    };
    let before = record(
        AssetKind::Footprint,
        raw(json!({"kind":"ipc_basis"})),
        vec![],
        provenance(),
    );
    let after = record(
        AssetKind::Footprint,
        raw(json!({"kind":"project_profile",
        "profile_id":Uuid::from_u128(22),"expected_object_revision":1,"expected_value_hash":"0".repeat(64)})),
        vec![],
        provenance(),
    );
    let review = compare(Some(input(&before)), Some(input(&after))).unwrap();
    assert!(
        review
            .fields
            .iter()
            .any(|c| c.path == ["process_basis", "profile_id"]
                && c.category == ChangeCategory::CheckBasis)
    );
    assert!(review.engineering_basis_changed);
    assert_eq!(review.checks, CheckCoverage::NotEvaluated);
}

#[test]
fn schematic_number_change_is_a_designation_with_unchanged_logical_and_physical_mapping() {
    let logical_pin = Uuid::from_u128(31);
    let unit_raw = serde_json::to_vec(&json!({"schema_version":1,"uuid":Uuid::from_u128(30),
        "name":"Logical unit","manufacturer":"","tags":[],"pins":{
            logical_pin.to_string():{"uuid":logical_pin,"name":"Signal","direction":"Input",
                "swap_group":0,"alternates":[]}}}))
    .unwrap();
    let logical = record(AssetKind::Unit, unit_raw, vec![], provenance());
    let dependency = Dependency {
        role: "unit".into(),
        payload_path: vec!["unit".into()],
        target: logical.0.reference().clone(),
    };
    let symbol = |number| {
        serde_json::to_vec(&json!({"schema_version":1,"uuid":Uuid::from_u128(32),
        "name":"Symbol","unit":logical.0.reference().asset_id,"pin_anchors":[{
            "pin":logical_pin,"number":number,"position":{"x":0,"y":0}}]}))
        .unwrap()
    };
    let before = record(
        AssetKind::Symbol,
        symbol("1"),
        vec![dependency.clone()],
        provenance(),
    );
    let after = record(
        AssetKind::Symbol,
        symbol("A1"),
        vec![dependency],
        provenance(),
    );
    let map = record(
        AssetKind::PinPadMap,
        serde_json::to_vec(&json!({"schema_version":1,
        "uuid":Uuid::from_u128(33),"part":Uuid::from_u128(34),"mappings":{
            Uuid::from_u128(35).to_string():{"gate":Uuid::from_u128(36),"pin":logical_pin}}}))
        .unwrap(),
        vec![],
        provenance(),
    );
    let review = compare(Some(input(&before)), Some(input(&after))).unwrap();
    assert_eq!(review.fields.len(), 1);
    assert_eq!(review.fields[0].path, ["pin_anchors", "0", "number"]);
    assert_eq!(review.fields[0].category, ChangeCategory::Designation);
    assert!(review.fields[0].label.starts_with("Designation:"));
    assert_eq!(review.fields[0].before.as_deref(), Some("\"1\""));
    assert_eq!(review.fields[0].after.as_deref(), Some("\"A1\""));
    assert_eq!(review.fields[0].unit, None);
    assert!(review.dependencies.is_empty());
    assert_eq!(
        before.0.payload().value()["pin_anchors"][0]["pin"],
        after.0.payload().value()["pin_anchors"][0]["pin"]
    );
    let physical = compare(Some(input(&map)), Some(input(&map))).unwrap();
    assert!(!physical.payload_bytes_changed && !physical.engineering_basis_changed);
    assert!(physical.fields.is_empty());
    assert!(
        review
            .fields
            .iter()
            .all(|c| c.category != ChangeCategory::Mapping)
    );
}
