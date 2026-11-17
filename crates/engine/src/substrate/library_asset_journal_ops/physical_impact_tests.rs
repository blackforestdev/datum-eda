use super::*;
use crate::library_assets::{
    AssetDigest, AssetProvenance, AssetRevision, LocalCounters, encode_metadata,
};

fn sealed(raw: &[u8]) -> VerifiedAsset {
    let provenance: AssetProvenance = serde_json::from_value(serde_json::json!({
        "schema_version":1,"authors":[],"created_at":null,"change_actor":null,"changed_at":null,
        "change_summary":null,"origin_kind":"unknown","source_assets":[],"sources":[],
        "rights":{"license_expression":null,"attribution":[],"restrictions":[],"evidence":[]},"evidence":[]
    })).unwrap();
    let revision = AssetRevision {
        schema_version: 1,
        asset_id: serde_json::from_value(
            serde_json::from_slice::<Value>(raw).unwrap()["uuid"].clone(),
        )
        .unwrap(),
        kind: AssetKind::Padstack,
        payload_schema_version: 1,
        payload_ref: AssetDigest::blob(raw),
        parents: vec![],
        dependencies: vec![],
        provenance_ref: provenance.fingerprint().unwrap(),
        extensions: None,
    };
    let record = encode_metadata(&revision).unwrap();
    let provenance = encode_metadata(&provenance).unwrap();
    VerifiedAsset::read(
        &revision.reference().unwrap(),
        &record,
        &provenance,
        raw,
        raw.len() + record.len() + provenance.len(),
        LocalCounters::default(),
    )
    .unwrap()
}
fn bytes(extra: &str) -> Vec<u8> {
    format!(r#"{{"schema_version":1,"uuid":"00000000-0000-0000-0000-000000000001","name":"Copper","aperture":{{"circle":{{"diameter_nm":1000000}}}}{extra}}}"#).into_bytes()
}

#[test]
fn physical_guard_preserves_large_fractional_unknown_precision_independently_of_admission() {
    let exact = bytes(r#", "vendor":9007199254740993.1"#);
    let target = sealed(&exact);
    let rounded: Value = serde_json::from_slice(&exact).unwrap();
    assert!(rounded["vendor"].as_number().unwrap().is_f64());
    assert_eq!(target.payload().value(), &rounded);
    assert!(!physical_matches(AssetKind::Padstack, &target, &rounded).unwrap());
}

#[test]
fn physical_guard_ignores_only_owned_root_descriptions_and_classified_counters() {
    let target = sealed(&bytes(
        r#", "vendor":{"object_revision":1,"tags":["engineering"]}"#,
    ));
    let mut native = target.payload().value().clone();
    native["name"] = "Readable name".into();
    native["object_revision"] = 42.into();
    assert!(physical_matches(AssetKind::Padstack, &target, &native).unwrap());
    native["vendor"]["object_revision"] = 2.into();
    assert!(!physical_matches(AssetKind::Padstack, &target, &native).unwrap());
    native["vendor"]["object_revision"] = 1.into();
    native["vendor"]["tags"] = serde_json::json!(["changed engineering"]);
    assert!(!physical_matches(AssetKind::Padstack, &target, &native).unwrap());
}

#[test]
fn frozen_matching_refuses_wrong_birth_and_real_compensated_rebirth() {
    use crate::api::native_write::{self as native, WriteProvenance};
    use crate::substrate::component_pad_source::{capture_root, source_incarnation};
    use crate::substrate::{CommitSource, ProjectResolver, SourceShardTaxon};
    let root = std::env::temp_dir().join(format!("datum_frozen_birth_{}", Uuid::new_v4()));
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Frozen birth control".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let id = Uuid::new_v4();
    let provenance = || {
        WriteProvenance::new(
            "physical-birth-control",
            CommitSource::Test,
            "Author unused Padstack",
        )
    };
    let prepared = (native::registry::find_native_write_verb("datum.library.create")
        .unwrap()
        .build)(
        &native::registry::NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        serde_json::json!({"kind":"padstacks","object_id":id,"pool_path":"pool",
            "fields":{"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}}),
    )
    .unwrap();
    native::commit_prepared(&mut model, &root, prepared).unwrap();
    let original = capture_root(&model, id, SourceShardTaxon::PoolPadstack).unwrap();
    let mut portable = model
        .materialized_pool_object_revision(id, "padstacks", original.object_revision)
        .unwrap();
    portable.as_object_mut().unwrap().remove("object_revision");
    let target = sealed(&serde_json::to_vec(&portable).unwrap());
    let sources = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert!(matches_frozen(&model, &sources, &original, AssetKind::Padstack, &target).unwrap());
    assert_eq!(
        source_incarnation(&model, &original.relative_path, id),
        original.incarnation
    );
    let mut wrong = original.clone();
    wrong.incarnation = Some(Uuid::new_v4());
    assert!(!matches_frozen(&model, &sources, &wrong, AssetKind::Padstack, &target).unwrap());
    assert!(original.incarnation.is_some());
    wrong.incarnation = None;
    assert!(!matches_frozen(&model, &sources, &wrong, AssetKind::Padstack, &target).unwrap());
    // Actual selected-target compensation, not a fabricated accepted history:
    // Create -> Undo(Delete) -> Redo(Create). Content/counters can repeat while
    // the journal's source-birth owner correctly identifies a new incarnation.
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert!(!model.objects.contains_key(&id));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let reborn = capture_root(&model, id, SourceShardTaxon::PoolPadstack).unwrap();
    assert_eq!(reborn.object_revision, original.object_revision);
    assert_eq!(reborn.owner_value_hash, original.owner_value_hash);
    assert_eq!(reborn.record_hash, original.record_hash);
    assert_ne!(reborn.incarnation, original.incarnation);
    assert_eq!(
        source_incarnation(&model, &reborn.relative_path, id),
        reborn.incarnation
    );
    let sources = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert!(!matches_frozen(&model, &sources, &original, AssetKind::Padstack, &target).unwrap());
    assert!(matches_frozen(&model, &sources, &reborn, AssetKind::Padstack, &target).unwrap());
    // A Pad child has its own birth even when its Footprint root survives.
    // Use ordinary library edits/compensation, with no fabricated journal.
    let package = Uuid::new_v4();
    let footprint = Uuid::new_v4();
    let pad = Uuid::new_v4();
    for (kind, object, fields) in [
        ("packages", package, serde_json::json!({"name":"Body"})),
        (
            "footprints",
            footprint,
            serde_json::json!({"name":"Land pattern","package":package,
            "pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":id,"layer":1}}}),
        ),
    ] {
        let prepared = (native::registry::find_native_write_verb("datum.library.create")
            .unwrap()
            .build)(
            &native::registry::NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            serde_json::json!({"kind":kind,"object_id":object,"pool_path":"pool","fields":fields}),
        )
        .unwrap();
        native::commit_prepared(&mut model, &root, prepared).unwrap();
    }
    let owner = capture_root(&model, footprint, SourceShardTaxon::PoolFootprint).unwrap();
    let child = crate::substrate::component_pad_source::capture_pad(&model, &owner, pad).unwrap();
    assert!(same_incarnation(&model, &child));
    let mut wrong_child = child.clone();
    wrong_child.incarnation = Some(Uuid::new_v4());
    assert!(!same_incarnation(&model, &wrong_child));
    let mut empty = model
        .materialized_pool_object_revision(footprint, "footprints", owner.object_revision)
        .unwrap();
    empty["pads"] = serde_json::json!({});
    let prepared = native::library::build_pool_library_write(
        &model,
        provenance(),
        None,
        vec![native::library::PoolLibraryOperationSpec::Set {
            target: native::library::PoolLibraryObjectTarget::new("pool", "footprints", footprint),
            object: empty,
        }],
    )
    .unwrap();
    native::commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(!same_incarnation(&model, &child));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let owner_now = capture_root(&model, footprint, SourceShardTaxon::PoolFootprint).unwrap();
    let child_now =
        crate::substrate::component_pad_source::capture_pad(&model, &owner_now, pad).unwrap();
    assert_eq!(owner_now.incarnation, owner.incarnation);
    assert_ne!(child_now.incarnation, child.incarnation);
    assert!(!same_incarnation(&model, &child));
    assert!(same_incarnation(&model, &child_now));
    std::fs::remove_dir_all(root).unwrap();
}
