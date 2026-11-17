use std::collections::BTreeMap;

use serde_json::json;
use uuid::Uuid;

use super::*;

pub(super) fn provenance() -> AssetProvenance {
    serde_json::from_value(json!({
        "schema_version":1, "authors":[], "created_at":null,
        "change_actor":null,"changed_at":null,"change_summary":null,
        "origin_kind":"unknown","source_assets":[],"sources":[],
        "rights":{"license_expression":null,"attribution":[],"restrictions":[],"evidence":[]},
        "evidence":[]
    }))
    .unwrap()
}

pub(super) fn asset(
    kind: AssetKind,
    value: serde_json::Value,
    dependencies: Vec<Dependency>,
) -> (AssetRevisionRef, VerifiedAsset) {
    let payload = serde_json::to_vec(&value).unwrap();
    let provenance = provenance();
    let revision = AssetRevision {
        schema_version: 1,
        asset_id: serde_json::from_value(value["uuid"].clone()).unwrap(),
        kind,
        payload_schema_version: 1,
        payload_ref: AssetDigest::blob(&payload),
        parents: vec![],
        dependencies,
        provenance_ref: provenance.fingerprint().unwrap(),
        extensions: None,
    };
    let reference = revision.reference().unwrap();
    let verified = VerifiedAsset::read(
        &reference,
        &encode_metadata(&revision).unwrap(),
        &encode_metadata(&provenance).unwrap(),
        &payload,
        usize::MAX,
        LocalCounters::default(),
    )
    .unwrap();
    (reference, verified)
}

fn unit(name: &str) -> (AssetRevisionRef, VerifiedAsset) {
    asset(
        AssetKind::Unit,
        json!({"schema_version":1,"uuid":Uuid::from_u128(1),
        "name":name,"manufacturer":"","pins":{},"tags":[]}),
        vec![],
    )
}

fn symbol(unit: &AssetRevisionRef, dependencies: bool) -> (AssetRevisionRef, VerifiedAsset) {
    asset(
        AssetKind::Symbol,
        json!({"schema_version":1,"uuid":Uuid::from_u128(2),
        "name":"Symbol","unit":unit.asset_id}),
        if dependencies {
            vec![Dependency {
                role: "unit".into(),
                payload_path: vec!["unit".into()],
                target: unit.clone(),
            }]
        } else {
            vec![]
        },
    )
}

fn limits() -> ClosureLimits {
    ClosureLimits {
        max_nodes: 32,
        max_depth: 16,
    }
}

#[test]
fn omitted_or_unavailable_references_cannot_borrow_current_pool_definitions() {
    let (unit_ref, unit) = unit("Unit");
    let (root, symbol_record) = symbol(&unit_ref, false);
    let assets = BTreeMap::from([(unit_ref.clone(), unit), (root.clone(), symbol_record)]);
    assert!(inspect_closure(&[root], &assets, limits()).is_err());
    let (root, symbol_record) = symbol(&unit_ref, true);
    let assets = BTreeMap::from([(root.clone(), symbol_record)]);
    assert!(inspect_closure(&[root], &assets, limits()).is_err());
}

#[test]
fn exact_closure_is_stable_and_refuses_conflicting_revisions_or_limits() {
    let (old_ref, old) = unit("Before");
    let (new_ref, new) = unit("After");
    let (root, symbol) = symbol(&old_ref, true);
    let assets = BTreeMap::from([
        (old_ref.clone(), old),
        (new_ref.clone(), new),
        (root.clone(), symbol),
    ]);
    let closure = inspect_closure(&[root.clone()], &assets, limits()).unwrap();
    assert_eq!(closure.assets().len(), 2);
    assert!(closure.graph_report().valid);
    assert!(!closure.assets().contains_key(&new_ref));
    let repeated = inspect_closure(&[old_ref, root.clone()], &assets, limits()).unwrap();
    assert_eq!(closure.digest(), repeated.digest());
    assert!(inspect_closure(&[root.clone(), new_ref], &assets, limits()).is_err());
    assert!(
        inspect_closure(
            &[root],
            &assets,
            ClosureLimits {
                max_nodes: 1,
                max_depth: 1
            }
        )
        .is_err()
    );
}

#[test]
fn changed_provenance_cannot_keep_a_previously_requested_revision() {
    let (reference, verified) = unit("Unit");
    let revision = verified.revision();
    let mut changed = provenance();
    changed.change_summary = Some("Unverified changed provenance".into());
    assert!(
        VerifiedAsset::read(
            &reference,
            &encode_metadata(revision).unwrap(),
            &encode_metadata(&changed).unwrap(),
            &serde_json::to_vec(&verified.payload().value()).unwrap(),
            usize::MAX,
            LocalCounters::default()
        )
        .is_err()
    );
}

#[test]
fn sealed_records_cannot_be_substituted_under_a_different_lookup_key() {
    let (reference, verified) = unit("Unit");
    let mut fake = reference.clone();
    fake.asset_id = Uuid::from_u128(900);
    let available = BTreeMap::from([(fake.clone(), verified)]);
    let error = inspect_closure(&[fake], &available, limits())
        .err()
        .unwrap();
    assert!(error.to_string().contains("sealed verified revision"));

    let (reference, verified) = unit("Unit");
    let (root, symbol_record) = symbol(&reference, true);
    let (_, replacement) = unit("Different contents");
    let available = BTreeMap::from([(reference, replacement), (root.clone(), symbol_record)]);
    assert!(inspect_closure(&[root], &available, limits()).is_err());
}

#[test]
fn provenance_refuses_bad_namespaces_and_times_without_inventing_missing_facts() {
    let mut value = provenance();
    for key in ["", "plain", ".org", "org.", "org..name", "org.bad name"] {
        value.extensions = Some(BTreeMap::from([(key.into(), json!(true))]));
        assert!(value.fingerprint().is_err(), "{key}");
    }
    value.extensions = Some(BTreeMap::from([(
        "org.example.evidence".into(),
        json!(true),
    )]));
    for time in [
        "yesterday",
        "2026-02-29",
        "2026-13-01",
        "2026-11-16T25:00:00Z",
        "2026-11-16T10:20:30",
        "2026-11-16T10:20:30+25:00",
    ] {
        value.created_at = Some(time.into());
        assert!(value.fingerprint().is_err(), "{time}");
    }
    for time in [
        "2024-02-29",
        "2026-11-16T10:20Z",
        "2026-11-16T10:20:30.001200-07:00",
    ] {
        value.created_at = Some(time.into());
        value.changed_at = Some(time.into());
        let bytes = encode_metadata(&value).unwrap();
        let restored: AssetProvenance = decode_metadata(&bytes, bytes.len()).unwrap();
        assert!(restored.fingerprint().is_ok());
        assert_eq!(restored.created_at.as_deref(), Some(time));
        assert_eq!(restored.changed_at.as_deref(), Some(time));
    }
    value.created_at = None;
    value.changed_at = None;
    assert!(value.fingerprint().is_ok());
    assert!(value.authors.is_empty());
}

#[test]
fn default_map_owner_is_contextual_and_wrong_owner_keeps_the_existing_diagnostic() {
    fn graph(owner: Uuid) -> (AssetRevisionRef, BTreeMap<AssetRevisionRef, VerifiedAsset>) {
        let (unit_ref, unit_record) = unit("Unit");
        let (symbol_ref, symbol_record) = symbol(&unit_ref, true);
        let entity_id = Uuid::from_u128(3);
        let gate_id = Uuid::from_u128(31);
        let entity = json!({"schema_version":1,"uuid":entity_id,"name":"Entity",
            "prefix":"U","manufacturer":"","tags":[],"gates":{
                gate_id.to_string():{"uuid":gate_id,"name":"G","unit":unit_ref.asset_id,"symbol":symbol_ref.asset_id}}});
        let (entity_ref, entity_record) = asset(
            AssetKind::Entity,
            entity,
            vec![
                Dependency {
                    role: "symbol".into(),
                    payload_path: vec!["gates".into(), gate_id.to_string(), "symbol".into()],
                    target: symbol_ref.clone(),
                },
                Dependency {
                    role: "unit".into(),
                    payload_path: vec!["gates".into(), gate_id.to_string(), "unit".into()],
                    target: unit_ref.clone(),
                },
            ],
        );
        let (package_ref, package_record) = asset(
            AssetKind::Package,
            json!({"schema_version":1,"uuid":Uuid::from_u128(4),"name":"Package"}),
            vec![],
        );
        let (footprint_ref, footprint_record) = asset(
            AssetKind::Footprint,
            json!({"schema_version":1,"uuid":Uuid::from_u128(5),"name":"Footprint",
                "package":package_ref.asset_id,"courtyard":{"vertices":[],"closed":true}}),
            vec![Dependency {
                role: "package".into(),
                payload_path: vec!["package".into()],
                target: package_ref.clone(),
            }],
        );
        let (map_ref, map_record) = asset(
            AssetKind::PinPadMap,
            json!({"schema_version":1,"uuid":Uuid::from_u128(6),"part":owner,
                "footprint":footprint_ref.asset_id,"mappings":{}}),
            vec![Dependency {
                role: "footprint".into(),
                payload_path: vec!["footprint".into()],
                target: footprint_ref.clone(),
            }],
        );
        let mut part = crate::api::native_write::library::pool_part_payload(
            Uuid::from_u128(7),
            entity_ref.asset_id,
            package_ref.asset_id,
            "",
            "",
            "100R",
            "Contextual owner unit test",
            "",
            "Unknown",
        );
        part["default_footprint"] = json!(footprint_ref.asset_id);
        part["default_pin_pad_map"] = json!(map_ref.asset_id);
        let (part_ref, part_record) = asset(
            AssetKind::Part,
            part,
            [
                ("default_footprint", footprint_ref.clone()),
                ("default_pin_pad_map", map_ref.clone()),
                ("entity", entity_ref.clone()),
                ("package", package_ref.clone()),
            ]
            .into_iter()
            .map(|(name, target)| Dependency {
                role: name.into(),
                payload_path: vec![name.into()],
                target,
            })
            .collect(),
        );
        (
            part_ref.clone(),
            BTreeMap::from([
                (unit_ref, unit_record),
                (symbol_ref, symbol_record),
                (entity_ref, entity_record),
                (package_ref, package_record),
                (footprint_ref, footprint_record),
                (map_ref, map_record),
                (part_ref, part_record),
            ]),
        )
    }
    // This is an ownership/closure control, not a manufacturable fixture claim.
    let (root, available) = graph(Uuid::from_u128(7));
    let closure = inspect_closure(&[root], &available, limits()).unwrap();
    assert_eq!(closure.assets().len(), 7);
    assert!(
        closure.graph_report().valid,
        "{:?}",
        closure.graph_report().diagnostics
    );
    let (root, available) = graph(Uuid::from_u128(99));
    let closure = inspect_closure(&[root], &available, limits()).unwrap();
    assert!(!closure.graph_report().valid);
    assert!(
        closure
            .graph_report()
            .diagnostics
            .iter()
            .any(|d| d.code == "part_mismatch")
    );
}
